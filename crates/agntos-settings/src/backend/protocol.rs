use agnt_common::wire::{ClientMessage, ServerMessage};

pub fn serialize(msg: &ClientMessage) -> String {
    serde_json::to_string(msg).unwrap()
}

pub fn deserialize(raw: &str) -> Result<ServerMessage, String> {
    serde_json::from_str(raw).map_err(|e| format!("Protocol error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agnt_common::wire::ClientMessage;

    #[test]
    fn serialize_chat() {
        let msg = ClientMessage::Chat { prompt: "hello".to_string() };
        let json = serialize(&msg);
        assert!(json.contains("\"type\":\"chat\""));
    }

    #[test]
    fn deserialize_token() {
        let json = r#"{"type":"token","content":"hi"}"#;
        let msg = deserialize(json).unwrap();
        match msg {
            ServerMessage::Token { content } => assert_eq!(content, "hi"),
            _ => panic!("expected Token"),
        }
    }

    #[test]
    fn deserialize_error_on_invalid() {
        let result = deserialize("not json");
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_event() {
        let json = r#"{"type":"event","event":"test","data":{"foo":1}}"#;
        let msg = deserialize(json).unwrap();
        match msg {
            ServerMessage::Event { event, .. } => assert_eq!(event, "test"),
            _ => panic!("expected Event"),
        }
    }
}
