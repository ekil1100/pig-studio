use domain::SessionStatus;
use ui_components::{build_session_banner, session_status_badge};

#[test]
fn maps_waiting_approval_to_warning_badge() {
    let meta = session_status_badge(SessionStatus::WaitingApproval);
    assert_eq!(meta.label, "等待审批");
    assert_eq!(meta.class_name, "badge badge-warning");
}

#[test]
fn blocked_state_suggests_fix_settings_action() {
    let view = build_session_banner(SessionStatus::Blocked);
    assert!(view.action_label.unwrap().contains("设置"));
}
