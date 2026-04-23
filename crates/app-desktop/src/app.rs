use crate::{
    bootstrap::DesktopModel,
    state::{NoticeTone, WorkspaceState},
};
use dioxus::prelude::*;
use std::{path::PathBuf, time::Duration};
use ui_components::{
    ApprovalPanel, Composer, EventTimeline, SessionHeader, SettingsPanel, Sidebar,
};

const APP_STYLESHEET: Asset = asset!("/assets/generated.css");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellState {
    pub sidebar_open: bool,
    pub active_project_id: Option<String>,
    pub active_session_id: Option<String>,
}

pub fn build_initial_shell() -> ShellState {
    ShellState {
        sidebar_open: true,
        active_project_id: None,
        active_session_id: None,
    }
}

#[component]
pub fn App() -> Element {
    let mut shell = use_signal(build_initial_shell);
    let mut model = use_signal(DesktopModel::bootstrap_or_preview);
    let mut prompt_draft = use_signal(String::new);
    let mut rename_title_draft = use_signal(String::new);

    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_millis(350)).await;
            model.write().refresh();
        }
    });

    let workspace_state: WorkspaceState = model.read().workspace().clone();
    let shell_state = shell.read().clone();

    rsx! {
        document::Stylesheet { href: APP_STYLESHEET }

        div {
            class: "min-h-screen bg-base-200 text-base-content",
            div {
                class: "flex min-h-screen",
                if shell_state.sidebar_open {
                    Sidebar {
                        projects: workspace_state.projects.clone(),
                        active_project_id: workspace_state.active_project_id.clone(),
                        active_session_id: workspace_state.active_session_id.clone(),
                        on_pick_project_folder: move |_| {
                            if let Some(folder) = pick_folder() {
                                let created = model.write().create_project("", &folder.to_string_lossy());
                                if created {
                                    let active_project_id = model.read().workspace().active_project_id.clone();
                                    let active_session_id = model.read().workspace().active_session_id.clone();
                                    shell.set(ShellState {
                                        sidebar_open: true,
                                        active_project_id,
                                        active_session_id,
                                    });
                                }
                            }
                        },
                        on_create_session: move |_| {
                            let created = model.write().create_session("");
                            if created {
                                if let Some(active_session) = model.read().workspace().active_session.as_ref() {
                                    rename_title_draft.set(active_session.session_name.clone());
                                }
                                let active_project_id = model.read().workspace().active_project_id.clone();
                                let active_session_id = model.read().workspace().active_session_id.clone();
                                shell.set(ShellState {
                                    sidebar_open: true,
                                    active_project_id,
                                    active_session_id,
                                });
                            }
                        },
                        on_select_project: move |project_id: String| {
                            model.write().select_project(project_id.clone());
                            shell.set(ShellState {
                                sidebar_open: true,
                                active_project_id: Some(project_id),
                                active_session_id: None,
                            });
                        },
                        on_select_session: move |session_id: String| {
                            model.write().select_session(session_id.clone());
                            if let Some(active_session) = model.read().workspace().active_session.as_ref() {
                                rename_title_draft.set(active_session.session_name.clone());
                            }
                            shell.set(ShellState {
                                sidebar_open: true,
                                active_project_id: model.read().workspace().active_project_id.clone(),
                                active_session_id: Some(session_id),
                            });
                        },
                        on_open_settings: move |_| {
                            model.write().open_settings();
                        },
                    }
                }

                main {
                    class: "flex min-h-screen flex-1 flex-col gap-4 p-6",
                    if let Some(notice) = workspace_state.notice.clone() {
                        div {
                            class: notice_class(&notice.tone),
                            span { "{notice.message}" }
                        }
                    }

                    if let Some(active_session) = workspace_state.active_session.clone() {
                        SessionHeader {
                            project_name: active_session.project_name.clone(),
                            session_name: active_session.session_name.clone(),
                            badge: active_session.status_badge.clone(),
                        }
                        div {
                            class: "card bg-base-100 shadow-sm",
                            div {
                                class: "card-body gap-3 p-4",
                                div {
                                    class: "flex flex-wrap items-center justify-between gap-3",
                                    div {
                                        h3 { class: "font-medium", "会话维护" }
                                        p { class: "text-xs text-base-content/60", "支持重命名、删除当前会话，以及查看运行时自动检测结果。" }
                                    }
                                    div { class: "flex gap-2",
                                        button {
                                            class: "btn btn-outline btn-sm",
                                            onclick: move |_| {
                                                model.write().open_settings();
                                            },
                                            "运行时"
                                        }
                                        button {
                                            class: "btn btn-error btn-outline btn-sm",
                                            onclick: move |_| {
                                                model.write().delete_active_session();
                                                shell.set(ShellState {
                                                    sidebar_open: true,
                                                    active_project_id: model.read().workspace().active_project_id.clone(),
                                                    active_session_id: model.read().workspace().active_session_id.clone(),
                                                });
                                            },
                                            "删除会话"
                                        }
                                    }
                                }
                                div {
                                    class: "flex flex-wrap items-end gap-3",
                                    label {
                                        class: "form-control flex-1 gap-2",
                                        span { class: "label-text text-xs", "会话标题" }
                                        input {
                                            class: "input input-bordered w-full",
                                            value: rename_title_draft.read().clone(),
                                            placeholder: active_session.session_name.clone(),
                                            oninput: move |event| rename_title_draft.set(event.value()),
                                        }
                                    }
                                    button {
                                        class: "btn btn-primary btn-sm",
                                        onclick: move |_| {
                                            let next_title = rename_title_draft.read().clone();
                                            if model.write().rename_active_session(&next_title) {
                                                if let Some(active_session) = model.read().workspace().active_session.as_ref() {
                                                    rename_title_draft.set(active_session.session_name.clone());
                                                }
                                            }
                                        },
                                        "重命名"
                                    }
                                }
                            }
                        }
                        div {
                            class: "alert alert-info",
                            div {
                                class: "flex flex-1 items-start justify-between gap-3",
                                div {
                                    p { class: "font-medium", "{active_session.banner.title}" }
                                    p { class: "text-sm opacity-80", "{active_session.banner.body}" }
                                }
                                div { class: "flex flex-wrap gap-2",
                                    if let Some(action_label) = active_session.banner.action_label.clone() {
                                        if action_label.contains("设置") {
                                            button {
                                                class: "btn btn-sm btn-outline",
                                                onclick: move |_| {
                                                    model.write().open_settings();
                                                },
                                                "{action_label}"
                                            }
                                        } else if action_label.contains("恢复") {
                                            button {
                                                class: "btn btn-sm btn-outline",
                                                onclick: move |_| {
                                                    model.write().resume_selected_session();
                                                },
                                                "{action_label}"
                                            }
                                        }
                                    }
                                    if matches!(
                                        active_session.status_badge.label,
                                        "已中断" | "已阻塞"
                                    ) {
                                        button {
                                            class: "btn btn-sm btn-primary btn-soft",
                                            onclick: move |_| {
                                                if model.write().create_followup_session_from_active() {
                                                    if let Some(active_session) = model.read().workspace().active_session.as_ref() {
                                                        rename_title_draft.set(active_session.session_name.clone());
                                                    }
                                                    shell.set(ShellState {
                                                        sidebar_open: true,
                                                        active_project_id: model.read().workspace().active_project_id.clone(),
                                                        active_session_id: model.read().workspace().active_session_id.clone(),
                                                    });
                                                }
                                            },
                                            "基于当前上下文新建会话"
                                        }
                                    }
                                }
                            }
                        }
                        if workspace_state.settings_open {
                            RuntimeSettingsSection { workspace_state: workspace_state.clone(), model }
                            div {
                                class: "flex justify-end",
                                button {
                                    class: "btn btn-ghost btn-sm",
                                    onclick: move |_| model.write().close_settings(),
                                    "收起设置"
                                }
                            }
                        }
                        EventTimeline { entries: active_session.timeline.clone() }
                        if !active_session.approvals.is_empty() {
                            ApprovalPanel {
                                approvals: active_session.approvals.clone(),
                                on_approve: move |approval_id: String| {
                                    model.write().respond_to_approval(&approval_id, true);
                                },
                                on_reject: move |approval_id: String| {
                                    model.write().respond_to_approval(&approval_id, false);
                                },
                            }
                        }
                        Composer {
                            value: prompt_draft.read().clone(),
                            on_input: move |value: String| prompt_draft.set(value),
                            on_submit: move |_| {
                                let prompt = prompt_draft.read().clone();
                                if model.write().send_prompt(&prompt) {
                                    prompt_draft.set(String::new());
                                }
                            },
                            busy: matches!(active_session.status_badge.label, "运行中" | "等待审批"),
                        }
                    } else {
                        div {
                            class: "hero flex-1 rounded-box bg-base-100 shadow-sm",
                            div {
                                class: "hero-content text-center",
                                div {
                                    class: "max-w-3xl space-y-5",
                                    h1 { class: "text-3xl font-semibold", "Pig Studio" }
                                    p {
                                        class: "text-sm text-base-content/70",
                                        "像 Codex App 一样，先选一个项目文件夹，再开始会话。Pi 二进制和配置目录会优先自动检测，不需要手动输入路径。"
                                    }
                                    div { class: "flex flex-wrap justify-center gap-3",
                                        button {
                                            class: "btn btn-primary",
                                            onclick: move |_| {
                                                if let Some(folder) = pick_folder() {
                                                    let created = model.write().create_project("", &folder.to_string_lossy());
                                                    if created {
                                                        shell.set(ShellState {
                                                            sidebar_open: true,
                                                            active_project_id: model.read().workspace().active_project_id.clone(),
                                                            active_session_id: model.read().workspace().active_session_id.clone(),
                                                        });
                                                    }
                                                }
                                            },
                                            "选择项目文件夹"
                                        }
                                        button {
                                            class: "btn btn-outline",
                                            onclick: move |_| model.write().open_settings(),
                                            "查看运行时检测"
                                        }
                                    }
                                    div {
                                        class: "grid gap-3 text-left md:grid-cols-3",
                                        div { class: "card bg-base-200 shadow-sm",
                                            div { class: "card-body gap-2 p-4",
                                                h3 { class: "card-title text-base", "1. 打开项目" }
                                                p { class: "text-sm text-base-content/70", "点击“选择项目文件夹”，项目名会自动从目录名推断。" }
                                            }
                                        }
                                        div { class: "card bg-base-200 shadow-sm",
                                            div { class: "card-body gap-2 p-4",
                                                h3 { class: "card-title text-base", "2. 开始会话" }
                                                p { class: "text-sm text-base-content/70", "点“开始新会话”即可，首条 prompt 会自动生成会话标题。" }
                                            }
                                        }
                                        div { class: "card bg-base-200 shadow-sm",
                                            div { class: "card-body gap-2 p-4",
                                                h3 { class: "card-title text-base", "3. 自动检测 Pi" }
                                                p { class: "text-sm text-base-content/70", "当前运行时：{runtime_summary(&workspace_state)}" }
                                            }
                                        }
                                    }
                                    if workspace_state.settings_open {
                                        RuntimeSettingsSection { workspace_state: workspace_state.clone(), model }
                                        div {
                                            class: "flex justify-center",
                                            button {
                                                class: "btn btn-ghost btn-sm",
                                                onclick: move |_| model.write().close_settings(),
                                                "收起设置"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RuntimeSettingsSection(
    workspace_state: WorkspaceState,
    mut model: Signal<DesktopModel>,
) -> Element {
    rsx! {
        SettingsPanel {
            runtime_path: workspace_state.runtime_path.clone(),
            runtime_source_label: workspace_state.runtime_source_label.clone(),
            config_dir: workspace_state.config_dir.clone(),
            config_dir_source_label: workspace_state.config_dir_source_label.clone(),
            has_runtime_override: workspace_state.has_runtime_override,
            has_config_dir_override: workspace_state.has_config_dir_override,
            health: workspace_state.runtime_health.clone(),
            on_refresh: move |_| model.write().refresh_runtime_detection(),
            on_pick_runtime_binary: move |_| {
                if let Some(path) = pick_runtime_binary() {
                    model.write().set_runtime_binary_override(path);
                }
            },
            on_pick_config_dir: move |_| {
                if let Some(path) = pick_folder() {
                    model.write().set_config_dir_override(path);
                }
            },
            on_clear_overrides: move |_| {
                model.write().clear_runtime_overrides();
            },
        }
    }
}

fn runtime_summary(workspace_state: &WorkspaceState) -> String {
    if workspace_state.runtime_path.is_empty() {
        "未检测到可用运行时".into()
    } else {
        format!(
            "{}（{}）",
            workspace_state.runtime_path, workspace_state.runtime_source_label
        )
    }
}

fn pick_folder() -> Option<PathBuf> {
    rfd::FileDialog::new().pick_folder()
}

fn pick_runtime_binary() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("选择 Pi 运行时二进制")
        .pick_file()
}

fn notice_class(tone: &NoticeTone) -> &'static str {
    match tone {
        NoticeTone::Info => "alert alert-info",
        NoticeTone::Success => "alert alert-success",
        NoticeTone::Warning => "alert alert-warning",
        NoticeTone::Error => "alert alert-error",
    }
}
