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

// Windows-specific testcontainers imports (grouped for clarity)
#[cfg(target_os = "windows")]
use testcontainers::{runners::SyncRunner, ImageExt, Container};
#[cfg(target_os = "windows")]
use testcontainers_modules::redis::{Redis as RedisImage, REDIS_PORT};

pub struct SpawnedRedis {
    port: u16,
    prefix: String,
    child: Arc<Mutex<Option<Child>>>,
    valid: AtomicBool,
    _logger: ChildProcessLogger,
    // Windows-specific container field (grouped at end for clarity)
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
        
        // Check environment variable for local Redis preference on Windows
        #[cfg(target_os = "windows")]
        let use_local_redis = std::env::var("GOLEM_USE_LOCAL_REDIS")
            .unwrap_or_default() == "true";
        
        #[cfg(target_os = "windows")]
        let (mut child, container) = if use_local_redis {
            info!("Using local Redis server on Windows (GOLEM_USE_LOCAL_REDIS=true)");
            let child = Self::spawn_local_redis_process(port)
                .expect("Failed to spawn local Redis server");
            (Some(child), None)
        } else {
            info!("Using Testcontainers Redis on Windows (default)");
            let container = Self::start_redis_container()
                .expect("Failed to start Redis container");
            let mapped_port = container
                .get_host_port_ipv4(REDIS_PORT)
                .expect("Failed to get Redis container port");
            info!("Redis container started on port {}", mapped_port);
            (None, Some(container))
        };
        
        #[cfg(not(target_os = "windows"))]
        let mut child = Self::spawn_local_redis_process(port)
            .expect("Failed to spawn Redis server");

        // Determine final port (for containers, use mapped port)
        #[cfg(target_os = "windows")]
        let final_port = if let Some(ref container) = container {
            container.get_host_port_ipv4(REDIS_PORT).expect("Failed to get host port")
        } else {
            port
        };

        #[cfg(not(target_os = "windows"))]
        let final_port = port;

        // Set up logging (handle both child process and container scenarios)
        #[cfg(target_os = "windows")]
        let logger = if let Some(ref mut child_proc) = child {
            ChildProcessLogger::log_child_process("[redis]", out_level, err_level, child_proc)
        } else {
            // For containers, create a minimal dummy process for logger compatibility
            let mut dummy_child = Command::new("cmd")
                .arg("/C")
                .arg("echo")
                .arg("Container started")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to create dummy process for container logging");
            ChildProcessLogger::log_child_process("[redis-container]", out_level, err_level, &mut dummy_child)
        };

        #[cfg(not(target_os = "windows"))]
        let logger = ChildProcessLogger::log_child_process("[redis]", out_level, err_level, &mut child);

        super::wait_for_startup(&host, final_port, Duration::from_secs(10));

        Self {
            port: final_port,
            prefix,
            child: Arc::new(Mutex::new(child)),
            valid: AtomicBool::new(true),
            _logger: logger,
            #[cfg(target_os = "windows")]
            _container: container,
        }
    }

    // Helper method to spawn local Redis process (cross-platform)
    fn spawn_local_redis_process(port: u16) -> Result<Child, std::io::Error> {
        #[cfg(target_os = "windows")]
        let command = Command::new("cmd")
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
            .spawn();
            
        #[cfg(not(target_os = "windows"))]
        let command = Command::new("redis-server")
            .arg("--port")
            .arg(port.to_string())
            .arg("--save")
            .arg("")
            .arg("--appendonly")
            .arg("no")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
            
        command
    }
    
    // Helper method to start Redis container (Windows-only)
    #[cfg(target_os = "windows")]
    fn start_redis_container() -> Result<Container<RedisImage>, testcontainers::core::error::TestcontainersError> {
        RedisImage::default()
            .with_tag("7.2")
            .start()
    }

    fn blocking_kill(&self) {
        info!("Stopping Redis");
        
        // Kill child process if present
        if let Some(mut child) = self.child.lock().unwrap().take() {
            self.valid.store(false, Ordering::Release);
            let _ = child.kill();
        }
        
        // Stop container if present (Windows only)
        #[cfg(target_os = "windows")]
        if let Some(ref container) = self._container {
            let _ = container.stop();
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
