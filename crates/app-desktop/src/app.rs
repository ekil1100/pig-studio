use crate::{
    bootstrap::DesktopModel,
    state::{NoticeTone, WorkspaceState},
};
use coss_ui_dioxus::{
    Alert, AlertVariant, Button, ButtonSize, ButtonVariant, Card, Input, Surface,
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
            tokio::time::sleep(Duration::from_millis(1500)).await;
            model.write().refresh();
        }
    });

    let workspace_state: WorkspaceState = model.read().workspace().clone();
    let shell_state = shell.read().clone();

    rsx! {
        document::Stylesheet { href: APP_STYLESHEET }

        div {
            class: "min-h-screen bg-transparent text-foreground",
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
                    class: "flex min-h-screen flex-1 flex-col p-4 pl-3",
                    div {
                        class: "mx-auto flex w-full max-w-[1180px] flex-1 flex-col gap-4 pb-4",
                        if let Some(notice) = workspace_state.notice.clone() {
                            Alert {
                                variant: notice_variant(&notice.tone),
                                span { "{notice.message}" }
                            }
                        }

                        if let Some(active_session) = workspace_state.active_session.clone() {
                            SessionHeader {
                                project_name: active_session.project_name.clone(),
                                session_name: active_session.session_name.clone(),
                                badge: active_session.status_badge.clone(),
                            }

                            Surface {
                                class: "flex flex-col gap-4 px-5 py-5",
                                div {
                                    class: "flex flex-wrap items-start justify-between gap-4",
                                    div {
                                        p { class: "studio-kicker", "Session Controls" }
                                        h2 { class: "mt-1 text-base font-semibold", "会话维护" }
                                        p { class: "mt-2 text-sm leading-6 text-foreground/62", "支持重命名、删除当前会话，以及查看运行时自动检测结果。" }
                                    }
                                    div { class: "flex flex-wrap gap-2",
                                        Button {
                                            variant: ButtonVariant::Outline,
                                            size: ButtonSize::Sm,
                                            class: "rounded-md",
                                            onclick: move |_| model.write().open_settings(),
                                            "运行时"
                                        }
                                        if matches!(
                                            active_session.status_badge.label,
                                            "已中断" | "已阻塞"
                                        ) {
                                            Button {
                                                variant: ButtonVariant::Secondary,
                                                size: ButtonSize::Sm,
                                                class: "rounded-md",
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
                                        Button {
                                            variant: ButtonVariant::DestructiveOutline,
                                            size: ButtonSize::Sm,
                                            class: "rounded-md",
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
                                    class: "grid gap-3 lg:grid-cols-[minmax(0,1fr)_auto]",
                                    label {
                                        class: "flex flex-col gap-2",
                                        span { class: "text-xs font-medium text-foreground/45", "会话标题" }
                                        Input {
                                            class: "h-10 w-full rounded-md px-3",
                                            value: rename_title_draft.read().clone(),
                                            placeholder: active_session.session_name.clone(),
                                            on_input: move |value| rename_title_draft.set(value),
                                        }
                                    }
                                    div { class: "flex items-end",
                                        Button {
                                            size: ButtonSize::Sm,
                                            class: "rounded-md px-4",
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

                            Surface {
                                class: "px-5 py-4",
                                div {
                                    class: "flex flex-wrap items-start justify-between gap-4",
                                    div {
                                        p { class: "text-base font-semibold", "{active_session.banner.title}" }
                                        p { class: "mt-2 text-sm leading-6 text-foreground/62", "{active_session.banner.body}" }
                                    }
                                    if let Some(action_label) = active_session.banner.action_label.clone() {
                                        if action_label.contains("设置") {
                                            Button {
                                                variant: ButtonVariant::Outline,
                                                size: ButtonSize::Sm,
                                                class: "rounded-md",
                                                onclick: move |_| {
                                                    model.write().open_settings();
                                                },
                                                "{action_label}"
                                            }
                                        } else if action_label.contains("恢复") {
                                            Button {
                                                variant: ButtonVariant::Outline,
                                                size: ButtonSize::Sm,
                                                class: "rounded-md",
                                                onclick: move |_| {
                                                    model.write().resume_selected_session();
                                                },
                                                "{action_label}"
                                            }
                                        }
                                    }
                                }
                            }

                            if workspace_state.settings_open {
                                RuntimeSettingsSection { workspace_state: workspace_state.clone(), model }
                                div {
                                    class: "flex justify-end",
                                    Button {
                                        variant: ButtonVariant::Ghost,
                                        size: ButtonSize::Sm,
                                        class: "rounded-md",
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
                            EmptyWorkspace {
                                workspace_state: workspace_state.clone(),
                                on_pick_project_folder: move |_| {
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
                                on_open_settings: move |_| model.write().open_settings(),
                                on_close_settings: move |_| model.write().close_settings(),
                                model,
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EmptyWorkspace(
    workspace_state: WorkspaceState,
    on_pick_project_folder: EventHandler<()>,
    on_open_settings: EventHandler<()>,
    on_close_settings: EventHandler<()>,
    mut model: Signal<DesktopModel>,
) -> Element {
    rsx! {
        Surface {
            class: "flex flex-1 flex-col gap-6 overflow-hidden p-6",
            div {
                class: "grid gap-6 lg:grid-cols-[minmax(0,1fr)_320px]",
                div { class: "space-y-5",
                    div { class: "inline-flex items-center gap-2 rounded-md bg-muted px-2.5 py-1.5 text-xs font-medium text-foreground/65",
                        span { class: "studio-dot" }
                        span { "Pig Studio v0.1" }
                    }
                    div { class: "space-y-3",
                        h1 { class: "max-w-2xl text-2xl font-semibold text-foreground", "选择项目开始" }
                        p { class: "max-w-2xl text-sm leading-6 text-foreground/62", "会话、事件和审批会按项目组织，当前没有打开的会话。" }
                    }
                    div { class: "flex flex-wrap gap-3",
                        Button {
                            size: ButtonSize::Sm,
                            class: "rounded-md px-4",
                            onclick: move |_| on_pick_project_folder.call(()),
                            "选择项目文件夹"
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            class: "rounded-md px-4",
                            onclick: move |_| on_open_settings.call(()),
                            "查看运行时检测"
                        }
                    }
                }

                div { class: "flex flex-col gap-3",
                    Card { class: "p-5",
                        p { class: "studio-kicker", "Runtime Summary" }
                        p { class: "mt-2 text-sm font-semibold", "{runtime_summary(&workspace_state)}" }
                        p { class: "mt-3 text-sm leading-6 text-foreground/60", "Pig Studio 会优先自动检测 Pi 二进制和配置目录。只有检测失败时才建议手动覆盖。" }
                    }
                    Card { class: "p-5",
                        p { class: "studio-kicker", "Workflow" }
                        div { class: "mt-3 space-y-4",
                            div {
                                p { class: "text-sm font-semibold", "1. 打开项目" }
                                p { class: "mt-1 text-sm leading-6 text-foreground/58", "先从左侧工作区选择本地文件夹，项目名会自动从目录名推断。" }
                            }
                            div {
                                p { class: "text-sm font-semibold", "2. 创建会话" }
                                p { class: "mt-1 text-sm leading-6 text-foreground/58", "会话会按项目持久化，你可以在应用重启后直接恢复历史入口。" }
                            }
                            div {
                                p { class: "text-sm font-semibold", "3. 观察事件和审批" }
                                p { class: "mt-1 text-sm leading-6 text-foreground/58", "运行中、等待审批、失败和中断都会在主区域清晰显示。" }
                            }
                        }
                    }
                }
            }

            if workspace_state.settings_open {
                RuntimeSettingsSection { workspace_state: workspace_state.clone(), model }
                div {
                    class: "flex justify-end",
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        class: "rounded-md",
                        onclick: move |_| on_close_settings.call(()),
                        "收起设置"
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

fn notice_variant(tone: &NoticeTone) -> AlertVariant {
    match tone {
        NoticeTone::Info => AlertVariant::Info,
        NoticeTone::Success => AlertVariant::Success,
        NoticeTone::Warning => AlertVariant::Warning,
        NoticeTone::Error => AlertVariant::Error,
    }
}
