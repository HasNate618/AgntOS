#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SessionState {
    Disconnected,
    InitSent,
    Ready,
    Chatting,
    AwaitingApproval,
}

pub struct Session {
    pub state: SessionState,
    pub profile: String,
    pub model: String,
    pub pending_proposals: Vec<String>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            state: SessionState::Disconnected,
            profile: String::new(),
            model: String::new(),
            pending_proposals: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_starts_disconnected() {
        let s = Session::new();
        assert_eq!(s.state, SessionState::Disconnected);
        assert!(s.profile.is_empty());
        assert!(s.model.is_empty());
        assert!(s.pending_proposals.is_empty());
    }

    #[test]
    fn session_state_transitions() {
        let mut s = Session::new();
        assert_eq!(s.state, SessionState::Disconnected);
        s.state = SessionState::InitSent;
        assert_eq!(s.state, SessionState::InitSent);
    }

    #[test]
    fn session_holds_profile_info() {
        let mut s = Session::new();
        s.profile = "local".to_string();
        s.model = "qwen".to_string();
        s.pending_proposals = vec!["p-abc".to_string()];
        assert_eq!(s.profile, "local");
        assert_eq!(s.model, "qwen");
        assert_eq!(s.pending_proposals.len(), 1);
    }
}
