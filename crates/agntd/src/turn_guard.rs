use serde_json::Value;
use std::collections::HashMap;

const MAX_STEPS: usize = 48;
const MAX_SAME_TOOL: usize = 3;
const MAX_CONSECUTIVE_ERRORS: usize = 5;

pub struct TurnGuard {
    steps: usize,
    repeat_counts: HashMap<(String, String), usize>,
    consecutive_errors: usize,
}

impl TurnGuard {
    pub fn new() -> Self {
        Self {
            steps: 0,
            repeat_counts: HashMap::new(),
            consecutive_errors: 0,
        }
    }

    pub fn record_llm_step(&mut self) -> Option<String> {
        self.steps += 1;
        if self.steps > MAX_STEPS {
            Some(format!(
                "Stopped after {MAX_STEPS} tool rounds in one turn. \
                 Break the task into smaller steps or use /cancel."
            ))
        } else {
            None
        }
    }

    pub fn record_tool(&mut self, name: &str, args: &Value, success: bool) -> Option<String> {
        if success {
            self.consecutive_errors = 0;
        } else {
            self.consecutive_errors += 1;
            if self.consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                return Some(
                    "Stopped after five failed tools in a row. \
                     Summarize what failed and ask the user how to proceed."
                        .to_string(),
                );
            }
        }

        let args_key = serde_json::to_string(args).unwrap_or_default();
        let count = self
            .repeat_counts
            .entry((name.to_string(), args_key))
            .or_insert(0);
        *count += 1;
        if *count >= MAX_SAME_TOOL {
            return Some(format!(
                "Stopped: repeated `{name}` with the same arguments {MAX_SAME_TOOL} times."
            ));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn stops_on_repeated_tool() {
        let mut g = TurnGuard::new();
        let args = json!({"command": "ls"});
        assert!(g.record_tool("run_bash", &args, false).is_none());
        assert!(g.record_tool("run_bash", &args, false).is_none());
        assert!(g.record_tool("run_bash", &args, false).is_some());
    }

    #[test]
    fn stops_on_step_cap() {
        let mut g = TurnGuard::new();
        for _ in 0..MAX_STEPS {
            assert!(g.record_llm_step().is_none());
        }
        assert!(g.record_llm_step().is_some());
    }
}
