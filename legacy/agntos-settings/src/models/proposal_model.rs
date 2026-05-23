#[derive(Debug, Clone)]
pub struct Proposal {
    pub proposal_id: String,
    pub summary: String,
    pub nix_changes: String,
    pub rollback_guidance: String,
    pub created_at: String,
    pub status: ProposalStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Applied,
    Dismissed,
}

#[derive(Debug, Clone)]
pub struct ProposalModel {
    pub proposals: Vec<Proposal>,
}

impl ProposalModel {
    pub fn new() -> Self {
        Self {
            proposals: Vec::new(),
        }
    }

    pub fn refresh(&mut self, config_dir: &str) {
        let dir = format!("{}/proposals", config_dir);
        let mut new_proposals = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    if let Ok(raw) = std::fs::read_to_string(&path) {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                            let proposal_id = v
                                .get("id")
                                .and_then(|i| i.as_str())
                                .unwrap_or("?")
                                .to_string();
                            let summary = v
                                .get("summary")
                                .and_then(|s| s.as_str())
                                .unwrap_or("")
                                .to_string();
                            let nix = v
                                .get("nix_changes")
                                .and_then(|s| s.as_str())
                                .unwrap_or("")
                                .to_string();
                            let rollback = v
                                .get("rollback_guidance")
                                .and_then(|s| s.as_str())
                                .unwrap_or("")
                                .to_string();
                            new_proposals.push(Proposal {
                                proposal_id,
                                summary,
                                nix_changes: nix,
                                rollback_guidance: rollback,
                                created_at: "".to_string(),
                                status: ProposalStatus::Pending,
                            });
                        }
                    }
                }
            }
        }
        self.proposals = new_proposals;
    }

    pub fn proposal_id_at(&self, index: usize) -> Option<&str> {
        self.proposals.get(index).map(|p| p.proposal_id.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposal_model_starts_empty() {
        let m = ProposalModel::new();
        assert!(m.proposals.is_empty());
    }

    #[test]
    fn proposal_model_refresh_no_directory() {
        let tmp = std::env::temp_dir().join("agntos-test-proposals-nonexistent");
        let _ = std::fs::remove_dir_all(&tmp);
        let mut m = ProposalModel::new();
        m.refresh(&tmp.to_string_lossy());
        assert!(m.proposals.is_empty());
    }

    #[test]
    fn proposal_model_refresh_valid_proposals() {
        let tmp = std::env::temp_dir().join("agntos-test-proposals-valid");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("proposals")).unwrap();
        std::fs::write(
            tmp.join("proposals/p-abc.json"),
            r#"{"id":"p-abc","summary":"Install nginx","nix_changes":"...","rollback_guidance":"remove"}"#,
        ).unwrap();
        let mut m = ProposalModel::new();
        m.refresh(&tmp.to_string_lossy());
        assert_eq!(m.proposals.len(), 1);
        assert_eq!(m.proposals[0].proposal_id, "p-abc");
        assert_eq!(m.proposals[0].summary, "Install nginx");
        assert_eq!(m.proposal_id_at(0), Some("p-abc"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn proposal_model_refresh_skips_invalid() {
        let tmp = std::env::temp_dir().join("agntos-test-proposals-invalid");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("proposals")).unwrap();
        std::fs::write(tmp.join("proposals/bad.json"), "not json").unwrap();
        let mut m = ProposalModel::new();
        m.refresh(&tmp.to_string_lossy());
        assert!(m.proposals.is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
