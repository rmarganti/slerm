use std::time::SystemTime;

use crate::runtime::{PtyBackend, SessionId, SpawnProcessRequest, TerminalSession, TerminalSize};

/// In-memory backend for exercising runtime service behavior before real PTY integration.
#[derive(Clone, Debug)]
pub struct MockPtyBackend {
    next_session_id: u64,
    pub spawned: Vec<SpawnProcessRequest>,
    pub killed: Vec<SessionId>,
    pub resized: Vec<(SessionId, TerminalSize)>,
    pub written: Vec<(SessionId, Vec<u8>)>,
}

impl Default for MockPtyBackend {
    fn default() -> Self {
        Self {
            next_session_id: 1,
            spawned: Vec::new(),
            killed: Vec::new(),
            resized: Vec::new(),
            written: Vec::new(),
        }
    }
}

impl MockPtyBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PtyBackend for MockPtyBackend {
    fn spawn(&mut self, request: SpawnProcessRequest) -> anyhow::Result<TerminalSession> {
        let id = SessionId(self.next_session_id);
        self.next_session_id += 1;
        let terminal_id = request.terminal_id;
        self.spawned.push(request);

        Ok(TerminalSession {
            id,
            terminal_id,
            started_at: SystemTime::now(),
        })
    }

    fn kill(&mut self, session_id: SessionId) -> anyhow::Result<()> {
        self.killed.push(session_id);
        Ok(())
    }

    fn resize(&mut self, session_id: SessionId, size: TerminalSize) -> anyhow::Result<()> {
        self.resized.push((session_id, size));
        Ok(())
    }

    fn write(&mut self, session_id: SessionId, bytes: &[u8]) -> anyhow::Result<()> {
        self.written.push((session_id, bytes.to_vec()));
        Ok(())
    }
}
