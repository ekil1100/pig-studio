use crate::{presenters::ProjectTreeItemView, theme::SIDEBAR_MENU_CLASS};
use coss_ui_dioxus::{Badge, BadgeSize, Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SidebarProps {
    pub projects: Vec<ProjectTreeItemView>,
    #[props(default)]
    pub active_project_id: Option<String>,
    #[props(default)]
    pub active_session_id: Option<String>,
    pub on_pick_project_folder: EventHandler<()>,
    pub on_create_session: EventHandler<()>,
    pub on_select_project: EventHandler<String>,
    pub on_select_session: EventHandler<String>,
    pub on_open_settings: EventHandler<()>,
}

#[component]
pub fn Sidebar(props: SidebarProps) -> Element {
    let can_create_session = props.active_project_id.is_some();
    let on_pick_project_folder = props.on_pick_project_folder.clone();
    let on_create_session = props.on_create_session.clone();
    let on_select_project = props.on_select_project.clone();
    let on_select_session = props.on_select_session.clone();
    let on_open_settings = props.on_open_settings.clone();

    rsx! {
        aside {
            class: "w-[292px] shrink-0 border-r border-border bg-card",
            div {
                class: "flex h-full min-h-screen flex-col gap-4 px-3 py-4",
                div {
                    class: "space-y-3",
                    div {
                        class: "flex items-center gap-3 px-2 py-2",
                        div {
                            class: "flex h-8 w-8 items-center justify-center rounded-md bg-primary text-sm font-semibold text-primary-foreground",
                            "P"
                        }
                        div {
                            p { class: "text-sm font-semibold", "Pig Studio" }
                            p { class: "text-xs text-foreground/55", "Agent Session Workspace" }
                        }
                    }
                    div {
                        class: "flex items-center justify-between px-1",
                        h2 { class: "studio-kicker", "Workspace" }
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Xs,
                            class: "rounded-md px-2 text-foreground/70",
                            onclick: move |_| on_open_settings.call(()),
                            "运行时"
                        }
                    }
                    div {
                        class: "rounded-md border border-border bg-muted/35",
                        div {
                            class: "flex flex-col gap-2 p-3",
                            h3 { class: "text-sm font-medium", "添加项目" }
                            p {
                                class: "text-xs leading-5 text-foreground/58",
                                "直接选择本地文件夹即可，不需要手动输入项目路径。"
                            }
                            Button {
                                size: ButtonSize::Sm,
                                class: "min-h-8 h-8 rounded-md",
                                onclick: move |_| on_pick_project_folder.call(()),
                                "选择项目文件夹"
                            }
                        }
                    }
                    div {
                        class: "rounded-md border border-border bg-muted/35",
                        div {
                            class: "flex flex-col gap-2 p-3",
                            h3 { class: "text-sm font-medium", "新建会话" }
                            p {
                                class: "text-xs leading-5 text-foreground/58",
                                if props.active_project_id.is_some() {
                                    "会话标题会自动生成，无需手动输入。"
                                } else {
                                    "先从下方选择一个项目，再开始新会话。"
                                }
                            }
                            Button {
                                size: ButtonSize::Sm,
                                class: "min-h-8 h-8 rounded-md",
                                disabled: !can_create_session,
                                onclick: move |_| on_create_session.call(()),
                                "开始新会话"
                            }
                        }
                    }
                }

                div {
                    class: "min-h-0 flex-1 overflow-y-auto pr-1",
                    div { class: "mb-3 flex items-center justify-between px-1",
                        span { class: "studio-kicker", "Projects" }
                        span { class: "text-xs text-foreground/45", "{props.projects.len()} 个项目" }
                    }
                    ul { class: SIDEBAR_MENU_CLASS,
                        for project in props.projects {
                            li {
                                details {
                                    class: "rounded-md border border-border bg-card p-1",
                                    open: props.active_project_id.as_deref() == Some(project.project_id.as_str()),
                                    summary {
                                        class: if props.active_project_id.as_deref() == Some(project.project_id.as_str()) {
                                            "rounded px-2 py-2 text-sm font-medium text-primary"
                                        } else {
                                            "rounded px-2 py-2 text-sm font-medium text-foreground/78"
                                        },
                                        onclick: {
                                            let project_id = project.project_id.clone();
                                            let on_select_project = on_select_project.clone();
                                            move |_| on_select_project.call(project_id.clone())
                                        },
                                        div { class: "flex min-w-0 items-center gap-3",
                                            span { class: "studio-dot" }
                                            span { class: "truncate", "{project.project_name}" }
                                        }
                                    }
                                    ul {
                                        class: "mt-1 flex flex-col gap-1",
                                        for session in project.sessions {
                                            li {
                                                Button {
                                                    variant: ButtonVariant::Bare,
                                                    class: if props.active_session_id.as_deref() == Some(session.session_id.as_str()) {
                                                        "h-auto min-h-0 w-full justify-between rounded px-2 py-2 text-primary hover:bg-primary/10"
                                                    } else {
                                                        "h-auto min-h-0 w-full justify-between rounded px-2 py-2 text-foreground/72 hover:bg-muted/80"
                                                    },
                                                    onclick: {
                                                        let session_id = session.session_id.clone();
                                                        let on_select_session = on_select_session.clone();
                                                        move |_| on_select_session.call(session_id.clone())
                                                    },
                                                    div { class: "flex min-w-0 flex-1 flex-col items-start gap-1 text-left",
                                                        span { class: "w-full truncate text-sm font-medium", "{session.title}" }
                                                        span { class: "text-[11px] text-foreground/45", "Agent session" }
                                                    }
                                                    Badge { variant: session.badge.variant, size: BadgeSize::Sm, class: "border-none", "{session.badge.label}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div {
                    class: "border-t border-border/70 pt-4",
                    Button {
                        variant: ButtonVariant::Ghost,
                        class: "h-auto w-full justify-start rounded-md px-2 py-2 text-left",
                        onclick: move |_| on_open_settings.call(()),
                        div {
                            class: "flex flex-col items-start gap-1",
                            span { class: "font-medium", "设置与运行时" }
                            span { class: "text-xs text-foreground/55", "检查 Pi 运行时和配置目录" }
                        }
                    }
                }
            }
        }
    }
}
