// OS-integrated tools available to the agent.

#[derive(Debug, Clone)]
pub enum ToolRequest {
    InspectSystem,
    InspectServices,
    InspectLogs,
    ProposeConfig { description: String },
    ApplyConfig { proposal_id: String },
    Rollback,
    AuditLog,
}

impl ToolRequest {
    pub fn describe(&self) -> &str {
        match self {
            ToolRequest::InspectSystem => "inspect system hardware and state",
            ToolRequest::InspectServices => "inspect systemd services",
            ToolRequest::InspectLogs => "inspect system logs",
            ToolRequest::ProposeConfig { .. } => "propose a Nix config change",
            ToolRequest::ApplyConfig { .. } => "apply an approved config change",
            ToolRequest::Rollback => "show or trigger rollback",
            ToolRequest::AuditLog => "view the activity log",
        }
    }
}
