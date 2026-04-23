use crate::models::PiMonoEvent;
use serde_json::Value;

#[derive(Debug, Default, Clone, Copy)]
pub struct StreamParser;

impl StreamParser {
    pub fn parse_chunk(&self, chunk: &str) -> Vec<PiMonoEvent> {
        chunk
            .lines()
            .filter_map(|line| self.parse_line(line))
            .collect()
    }

    pub fn parse_line(&self, line: &str) -> Option<PiMonoEvent> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        let Ok(value) = serde_json::from_str::<Value>(line) else {
            return Some(PiMonoEvent::TextDelta {
                text: line.to_owned(),
            });
        };

        let event_type = value
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("text_delta");
        match event_type {
            "session_bound" => Some(PiMonoEvent::SessionBound {
                pimono_session_id: value
                    .get("session_id")
                    .or_else(|| value.get("pimono_session_id"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
            }),
            "run_started" => Some(PiMonoEvent::RunStarted {
                pimono_run_id: value
                    .get("run_id")
                    .or_else(|| value.get("pimono_run_id"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                pimono_session_id: value
                    .get("session_id")
                    .or_else(|| value.get("pimono_session_id"))
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
            }),
            "text_delta" => Some(PiMonoEvent::TextDelta {
                text: value
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
            }),
            "approval_requested" => Some(PiMonoEvent::ApprovalRequested {
                request_id: value
                    .get("request_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                request_type: value
                    .get("request_type")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                payload_json: value
                    .get("payload")
                    .cloned()
                    .unwrap_or(Value::Null)
                    .to_string(),
            }),
            "run_failed" => Some(PiMonoEvent::RunFailed {
                code: value
                    .get("code")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                message: value
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown pi runtime failure")
                    .to_owned(),
            }),
            "run_completed" => Some(PiMonoEvent::RunCompleted),
            "session" => {
                value
                    .get("id")
                    .and_then(Value::as_str)
                    .map(|id| PiMonoEvent::SessionBound {
                        pimono_session_id: id.to_owned(),
                    })
            }
            "message_update" => parse_pi_message_update(&value),
            "tool_execution_end" => parse_pi_tool_execution_end(&value),
            "agent_end" => Some(PiMonoEvent::RunCompleted),
            _ => Some(PiMonoEvent::TextDelta {
                text: line.to_owned(),
            }),
        }
    }
}

fn parse_pi_message_update(value: &Value) -> Option<PiMonoEvent> {
    let event = value.get("assistantMessageEvent")?;
    match event.get("type").and_then(Value::as_str) {
        Some("text_delta") => {
            event
                .get("delta")
                .and_then(Value::as_str)
                .map(|delta| PiMonoEvent::TextDelta {
                    text: delta.to_owned(),
                })
        }
        Some("error") => Some(PiMonoEvent::RunFailed {
            code: None,
            message: event
                .get("message")
                .or_else(|| event.get("reason"))
                .and_then(Value::as_str)
                .unwrap_or("pi message stream error")
                .to_owned(),
        }),
        _ => None,
    }
}

fn parse_pi_tool_execution_end(value: &Value) -> Option<PiMonoEvent> {
    if !value
        .get("isError")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return None;
    }

    let message = value
        .get("result")
        .and_then(|result| result.get("content"))
        .and_then(Value::as_array)
        .and_then(|content| content.first())
        .and_then(|block| block.get("text"))
        .and_then(Value::as_str)
        .unwrap_or("pi tool execution failed")
        .to_owned();

    Some(PiMonoEvent::RunFailed {
        code: None,
        message,
    })
}
