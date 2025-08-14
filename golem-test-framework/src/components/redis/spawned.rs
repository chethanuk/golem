// Copyright 2024-2025 Golem Cloud
//
// Licensed under the Golem Source License v1.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://license.golem.cloud/LICENSE
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::components::redis::Redis;
use crate::components::ChildProcessLogger;
use async_trait::async_trait;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{info, Level};

#[cfg(target_os = "windows")]
use testcontainers::{Container, Docker, Image};
#[cfg(target_os = "windows")]
use testcontainers_modules::redis::Redis as RedisImage;

pub struct SpawnedRedis {
    port: u16,
    prefix: String,
    child: Arc<Mutex<Option<Child>>>,
    valid: AtomicBool,
    _logger: ChildProcessLogger,
    #[cfg(target_os = "windows")]
    _container: Option<Container<RedisImage>>,
}

impl SpawnedRedis {
    pub fn new_default() -> Self {
        Self::new(
            super::DEFAULT_PORT,
            "".to_string(),
            Level::DEBUG,
            Level::ERROR,
        )
    }

    pub fn new(port: u16, prefix: String, out_level: Level, err_level: Level) -> Self {
        info!("Starting Redis on port {}", port);

        let host = "localhost".to_string();
        
        #[cfg(target_os = "windows")]
        let use_local_redis = std::env::var("GOLEM_USE_LOCAL_REDIS").unwrap_or_default() == "true";
        
        #[cfg(target_os = "windows")]
        let (child, container) = if use_local_redis {
            info!("Using local Redis server on Windows (GOLEM_USE_LOCAL_REDIS=true)");
            let child = Command::new("cmd")
                .arg("/C")
                .arg("redis-server")
                .arg("--port")
                .arg(port.to_string())
                .arg("--save")
                .arg("")
                .arg("--appendonly")
                .arg("no")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to spawn local redis server");
            (Some(child), None)
        } else {
            info!("Using Testcontainers Redis on Windows (default)");
            let docker = Docker::default();
            let redis_image = RedisImage::default();
            let container = docker.run(redis_image);
            let mapped_port = container.get_host_port_ipv4(6379);
            info!("Redis container started on port {}", mapped_port);
            (None, Some(container))
        };
        
        #[cfg(not(target_os = "windows"))]
        let mut child = Command::new("redis-server")
            .arg("--port")
            .arg(port.to_string())
            .arg("--save")
            .arg("")
            .arg("--appendonly")
            .arg("no")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn redis server");

        #[cfg(target_os = "windows")]
        let final_port = if let Some(ref container) = container {
            container.get_host_port_ipv4(6379)
        } else {
            port
        };

        #[cfg(not(target_os = "windows"))]
        let final_port = port;

        #[cfg(target_os = "windows")]
        let logger = if let Some(ref mut child_proc) = child {
            ChildProcessLogger::log_child_process("[redis]", out_level, err_level, child_proc)
        } else {
            ChildProcessLogger::default()
        };

        #[cfg(not(target_os = "windows"))]
        let logger = ChildProcessLogger::log_child_process("[redis]", out_level, err_level, &mut child);

        super::wait_for_startup(&host, final_port, Duration::from_secs(10));

        Self {
            port: final_port,
            prefix,
            #[cfg(target_os = "windows")]
            child: Arc::new(Mutex::new(child)),
            #[cfg(not(target_os = "windows"))]
            child: Arc::new(Mutex::new(Some(child))),
            valid: AtomicBool::new(true),
            _logger: logger,
            #[cfg(target_os = "windows")]
            _container: container,
        }
    }

    fn blocking_kill(&self) {
        info!("Stopping Redis");
        if let Some(mut child) = self.child.lock().unwrap().take() {
            self.valid.store(false, Ordering::Release);
            let _ = child.kill();
        }
    }
}

#[async_trait]
impl Redis for SpawnedRedis {
    fn assert_valid(&self) {
        if !self.valid.load(Ordering::Acquire) {
            std::panic!("Redis has been closed")
        }
    }

    fn private_host(&self) -> String {
        "localhost".to_string()
    }

    fn private_port(&self) -> u16 {
        self.port
    }

    fn prefix(&self) -> &str {
        &self.prefix
    }

    async fn kill(&self) {
        self.blocking_kill();
    }
}

impl Drop for SpawnedRedis {
    fn drop(&mut self) {
        self.blocking_kill();
    }
}
