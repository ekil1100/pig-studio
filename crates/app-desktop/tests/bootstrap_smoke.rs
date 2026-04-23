use app_desktop::app::build_initial_shell;

#[test]
fn builds_initial_shell_with_empty_workspace() {
    let shell = build_initial_shell();

    assert!(shell.sidebar_open);
    assert!(shell.active_project_id.is_none());
    assert!(shell.active_session_id.is_none());
}
