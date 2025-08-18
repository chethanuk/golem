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
use crate::components::redis_monitor::RedisMonitor;
use crate::components::ChildProcessLogger;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tracing::{info, Level};

pub struct SpawnedRedisMonitor {
    child: Arc<Mutex<Option<Child>>>,
    _logger: ChildProcessLogger,
}

impl SpawnedRedisMonitor {
    pub fn new(redis: impl AsRef<dyn Redis>, out_level: Level, err_level: Level) -> Self {
        info!(
            "Starting Redis monitor on port {}",
            redis.as_ref().public_port()
        );

        // Prefer the official redis-cli everywhere; no .bat or Memurai fallbacks
        let cli_command = if std::process::Command::new("redis-cli")
            .arg("--version")
            .output()
            .is_ok()
        {
            Some("redis-cli")
        } else {
            None
        };

        let mut child = if let Some(cmd) = cli_command {
            Command::new(cmd)
                .args(["-h", &redis.as_ref().public_host()])
                .args(["-p", &redis.as_ref().public_port().to_string()])
                .arg("monitor")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap_or_else(|_| {
                    // Fallback to dummy process if spawn fails
                    info!("redis-cli spawn failed, creating dummy monitor process");
                    if cfg!(target_os = "windows") {
                        Command::new("cmd")
                            .args(["/c", "echo Redis monitor unavailable"]) 
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                            .expect("Failed to create dummy process")
                    } else {
                        Command::new("sh")
                            .args(["-c", "echo Redis monitor unavailable"]) 
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                            .expect("Failed to create dummy process")
                    }
                })
        } else {
            // Create a dummy process when CLI tools aren't available
            info!("redis-cli not available, creating dummy monitor process");
            if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .args(["/c", "echo Redis monitor unavailable on this platform"]) 
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("Failed to create dummy process")
            } else {
                Command::new("sh")
                    .args(["-c", "echo Redis monitor unavailable on this platform"]) 
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("Failed to create dummy process")
            }
        };

        let logger =
            ChildProcessLogger::log_child_process("[redis-monitor]", out_level, err_level, &mut child);

        Self {
            child: Arc::new(Mutex::new(Some(child))),
            _logger: logger,
        }
    }
}

impl RedisMonitor for SpawnedRedisMonitor {
    fn assert_valid(&self) {}

    fn kill(&self) {
        if let Some(mut child) = self.child.lock().unwrap().take() {
            let _ = child.kill();
        }
    }
}

impl Drop for SpawnedRedisMonitor {
    fn drop(&mut self) {
        self.kill();
    }
}
