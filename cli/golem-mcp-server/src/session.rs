//! Session management and client state tracking
//!
//! Manages client sessions and tracks initialization state for MCP connections.

use golem_mcp_types::{
    error::{McpError, McpResult},
    protocol::{McpCapabilities, McpClientInfo},
    transport::ConnectionInfo,
};
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

/// Session manager for tracking client connections
#[derive(Debug)]
pub struct SessionManager {
    /// Active client sessions
    sessions: HashMap<SessionId, ClientSession>,
}

/// Unique session identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(Uuid);

/// Client session information
#[derive(Debug, Clone)]
pub struct ClientSession {
    /// Session identifier
    pub id: SessionId,
    /// Client information
    pub client_info: Option<McpClientInfo>,
    /// Client capabilities
    pub client_capabilities: Option<McpCapabilities>,
    /// Connection information
    pub connection_info: ConnectionInfo,
    /// Session state
    pub state: SessionState,
    /// Session creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last activity time
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

/// Session state tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionState {
    /// Session created but not initialized
    Created,
    /// Client sent initialize request
    Initializing,
    /// Session fully initialized and ready
    Ready,
    /// Session is closing
    Closing,
    /// Session closed
    Closed,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    /// Create a new session manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Create a new client session
    pub fn create_session(&mut self, connection_info: ConnectionInfo) -> SessionId {
        let session_id = SessionId(Uuid::new_v4());
        let now = chrono::Utc::now();

        let session = ClientSession {
            id: session_id,
            client_info: None,
            client_capabilities: None,
            connection_info,
            state: SessionState::Created,
            created_at: now,
            last_activity: now,
        };

        debug!("Creating new session: {}", session_id.0);
        self.sessions.insert(session_id, session);
        session_id
    }

    /// Initialize a session with client information
    ///
    /// # Errors
    ///
    /// Returns `McpError::InvalidRequest` if the session doesn't exist or is in an invalid state.
    pub fn initialize_session(
        &mut self,
        session_id: SessionId,
        client_info: McpClientInfo,
        client_capabilities: McpCapabilities,
    ) -> McpResult<()> {
        let session = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| McpError::invalid_request("Session not found".to_owned()))?;

        if session.state != SessionState::Created {
            return Err(McpError::invalid_request(
                "Session already initialized".to_owned(),
            ));
        }

        info!(
            "Initializing session {} for client: {}",
            session_id.0, client_info.name
        );

        session.client_info = Some(client_info);
        session.client_capabilities = Some(client_capabilities);
        session.state = SessionState::Initializing;
        session.last_activity = chrono::Utc::now();

        Ok(())
    }

    /// Mark session as ready
    ///
    /// # Errors
    ///
    /// Returns `McpError::InvalidRequest` if the session doesn't exist or is in an invalid state.
    pub fn mark_session_ready(&mut self, session_id: SessionId) -> McpResult<()> {
        let session = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| McpError::invalid_request("Session not found".to_owned()))?;

        if session.state != SessionState::Initializing {
            return Err(McpError::invalid_request(
                "Session not in initializing state".to_owned(),
            ));
        }

        session.state = SessionState::Ready;
        session.last_activity = chrono::Utc::now();

        info!("Session {} is now ready", session_id.0);
        Ok(())
    }

    /// Update session activity
    pub fn update_activity(&mut self, session_id: SessionId) {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.last_activity = chrono::Utc::now();
        }
    }

    /// Get session information
    #[must_use]
    pub fn get_session(&self, session_id: SessionId) -> Option<&ClientSession> {
        self.sessions.get(&session_id)
    }

    /// List all active sessions
    #[must_use]
    pub fn list_sessions(&self) -> Vec<&ClientSession> {
        self.sessions.values().collect()
    }

    /// Close a specific session
    ///
    /// # Errors
    ///
    /// Returns `McpError::InvalidRequest` if the session doesn't exist.
    pub async fn close_session(&mut self, session_id: SessionId) -> McpResult<()> {
        let session = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| McpError::invalid_request("Session not found".to_owned()))?;

        debug!("Closing session: {}", session_id.0);
        session.state = SessionState::Closing;

        // Perform cleanup here if needed

        session.state = SessionState::Closed;
        self.sessions.remove(&session_id);

        info!("Session {} closed", session_id.0);
        Ok(())
    }

    /// Close all sessions
    ///
    /// # Errors
    ///
    /// Returns errors encountered during session cleanup.
    pub async fn close_all_sessions(&mut self) -> McpResult<()> {
        let session_ids: Vec<SessionId> = self.sessions.keys().copied().collect();

        for session_id in session_ids {
            if let Err(e) = self.close_session(session_id).await {
                tracing::error!("Error closing session {}: {}", session_id.0, e);
            }
        }

        Ok(())
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&mut self, max_idle_duration: chrono::Duration) {
        let now = chrono::Utc::now();
        let expired_sessions: Vec<SessionId> = self
            .sessions
            .values()
            .filter(|session| {
                let idle_duration = now - session.last_activity;
                idle_duration > max_idle_duration
            })
            .map(|session| session.id)
            .collect();

        for session_id in expired_sessions {
            debug!("Removing expired session: {}", session_id.0);
            self.sessions.remove(&session_id);
        }
    }

    /// Get session statistics
    #[must_use]
    pub fn get_statistics(&self) -> SessionStatistics {
        let total_sessions = self.sessions.len();
        let mut state_counts = HashMap::new();

        for session in self.sessions.values() {
            *state_counts.entry(session.state).or_insert(0) += 1;
        }

        SessionStatistics {
            total_sessions,
            ready_sessions: *state_counts.get(&SessionState::Ready).unwrap_or(&0),
            initializing_sessions: *state_counts.get(&SessionState::Initializing).unwrap_or(&0),
            closing_sessions: *state_counts.get(&SessionState::Closing).unwrap_or(&0),
        }
    }
}

/// Session statistics
#[derive(Debug, Clone)]
pub struct SessionStatistics {
    /// Total number of sessions
    pub total_sessions: usize,
    /// Number of ready sessions
    pub ready_sessions: usize,
    /// Number of initializing sessions
    pub initializing_sessions: usize,
    /// Number of closing sessions
    pub closing_sessions: usize,
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use golem_mcp_types::{protocol::McpImplementation, transport::ConnectionState};
    use std::collections::HashMap;

    #[test]
    fn test_session_lifecycle() {
        let mut manager = SessionManager::new();
        let connection_info = ConnectionInfo {
            state: ConnectionState::Connected,
            connected_at: Some(chrono::Utc::now()),
            last_activity: Some(chrono::Utc::now()),
            messages_sent: 0,
            messages_received: 0,
            metadata: HashMap::new(),
        };

        // Create session
        let session_id = manager.create_session(connection_info);
        let session = manager.get_session(session_id).expect("Session should exist after creation");
        assert_eq!(session.state, SessionState::Created);

        // Initialize session
        let client_info = McpClientInfo {
            name: "test-client".to_owned(),
            version: "1.0.0".to_owned(),
            implementation: Some(McpImplementation {
                name: "test".to_owned(),
                version: "1.0.0".to_owned(),
                metadata: HashMap::new(),
            }),
        };
        let capabilities = McpCapabilities {
            tools: None,
            resources: None,
            prompts: None,
            logging: None,
        };

        manager
            .initialize_session(session_id, client_info, capabilities)
            .expect("Client capabilities should be valid");
        let session = manager.get_session(session_id).expect("Session should exist");
        assert_eq!(session.state, SessionState::Initializing);

        // Mark ready
        manager.mark_session_ready(session_id).expect("Session should be markable as ready");
        let session = manager.get_session(session_id).expect("Session should exist after marking ready");
        assert_eq!(session.state, SessionState::Ready);
    }

    #[tokio::test]
    async fn test_close_session() {
        let mut manager = SessionManager::new();
        let connection_info = ConnectionInfo::default();
        let session_id = manager.create_session(connection_info);

        assert!(manager.get_session(session_id).is_some());
        manager.close_session(session_id).await.expect("Session should close successfully");
        assert!(manager.get_session(session_id).is_none());
    }
}
