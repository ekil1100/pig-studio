use infra_pimono::{PiMonoEvent, StreamParser};

#[test]
fn parses_session_bound_event() {
    let parser = StreamParser;
    let events = parser.parse_chunk(r#"{"type":"session_bound","session_id":"sess-1"}"#);

    assert_eq!(
        events,
        vec![PiMonoEvent::SessionBound {
            pimono_session_id: "sess-1".into(),
        }]
    );
}

#[test]
fn parses_run_started_event() {
    let parser = StreamParser;
    let events =
        parser.parse_chunk(r#"{"type":"run_started","run_id":"run-1","session_id":"sess-1"}"#);

    assert_eq!(
        events,
        vec![PiMonoEvent::RunStarted {
            pimono_run_id: "run-1".into(),
            pimono_session_id: Some("sess-1".into()),
        }]
    );
}

#[test]
fn parses_text_delta_from_plain_text_output() {
    let parser = StreamParser;
    let events = parser.parse_chunk("hello world\n");

    assert_eq!(
        events,
        vec![PiMonoEvent::TextDelta {
            text: "hello world".into()
        }]
    );
}

#[test]
fn parses_approval_requested_event() {
    let parser = StreamParser;
    let events = parser.parse_chunk(
        r#"{"type":"approval_requested","request_id":"req-1","request_type":"filesystem.delete","payload":{"path":"README.md"}}"#,
    );

    assert_eq!(
        events,
        vec![PiMonoEvent::ApprovalRequested {
            request_id: "req-1".into(),
            request_type: "filesystem.delete".into(),
            payload_json: r#"{"path":"README.md"}"#.into(),
        }]
    );
}

#[test]
fn parses_run_failed_event() {
    let parser = StreamParser;
    let events = parser.parse_chunk(r#"{"type":"run_failed","code":"E_RUNTIME","message":"boom"}"#);

    assert_eq!(
        events,
        vec![PiMonoEvent::RunFailed {
            code: Some("E_RUNTIME".into()),
            message: "boom".into(),
        }]
    );
}

#[test]
fn parses_run_completed_event() {
    let parser = StreamParser;
    let events = parser.parse_chunk(r#"{"type":"run_completed"}"#);

    assert_eq!(events, vec![PiMonoEvent::RunCompleted]);
}

#[test]
fn parses_pi_json_session_header() {
    let parser = StreamParser;
    let events =
        parser.parse_chunk(r#"{"type":"session","id":"pi-session-1","cwd":"/tmp/project"}"#);

    assert_eq!(
        events,
        vec![PiMonoEvent::SessionBound {
            pimono_session_id: "pi-session-1".into(),
        }]
    );
}

#[test]
fn parses_pi_json_message_delta() {
    let parser = StreamParser;
    let events = parser.parse_chunk(
        r#"{"type":"message_update","assistantMessageEvent":{"type":"text_delta","delta":"hello from pi"}}"#,
    );

    assert_eq!(
        events,
        vec![PiMonoEvent::TextDelta {
            text: "hello from pi".into(),
        }]
    );
}

#[test]
fn parses_pi_json_agent_end_as_run_completed() {
    let parser = StreamParser;
    let events = parser.parse_chunk(r#"{"type":"agent_end","messages":[]}"#);

    assert_eq!(events, vec![PiMonoEvent::RunCompleted]);
}
