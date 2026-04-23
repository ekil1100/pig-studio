use crate::{presenters::ProjectTreeItemView, theme::SIDEBAR_MENU_CLASS};
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
            class: "w-96 border-r border-base-300 bg-base-100 p-4",
            div {
                class: "flex h-full flex-col gap-4",
                div {
                    class: "space-y-3",
                    div {
                        class: "flex items-center justify-between",
                        h2 { class: "text-lg font-semibold", "项目工作区" }
                        button {
                            class: "btn btn-outline btn-sm",
                            onclick: move |_| on_open_settings.call(()),
                            "运行时"
                        }
                    }
                    div {
                        class: "card bg-base-200 shadow-sm",
                        div {
                            class: "card-body gap-3 p-4",
                            h3 { class: "card-title text-base", "添加项目" }
                            p {
                                class: "text-xs text-base-content/60",
                                "直接选择本地文件夹即可，不需要手动输入项目路径。"
                            }
                            button {
                                class: "btn btn-primary btn-sm",
                                onclick: move |_| on_pick_project_folder.call(()),
                                "选择项目文件夹"
                            }
                        }
                    }
                    div {
                        class: "card bg-base-200 shadow-sm",
                        div {
                            class: "card-body gap-3 p-4",
                            h3 { class: "card-title text-base", "新建会话" }
                            p {
                                class: "text-xs text-base-content/60",
                                if props.active_project_id.is_some() {
                                    "会话标题会自动生成，无需手动输入。"
                                } else {
                                    "先从下方选择一个项目，再开始新会话。"
                                }
                            }
                            button {
                                class: if can_create_session { "btn btn-primary btn-sm" } else { "btn btn-primary btn-sm btn-disabled" },
                                disabled: !can_create_session,
                                onclick: move |_| on_create_session.call(()),
                                "开始新会话"
                            }
                        }
                    }
                }

                div {
                    class: "min-h-0 flex-1 overflow-y-auto",
                    ul { class: SIDEBAR_MENU_CLASS,
                        for project in props.projects {
                            li {
                                details {
                                    open: true,
                                    summary {
                                        class: if props.active_project_id.as_deref() == Some(project.project_id.as_str()) {
                                            "font-semibold text-primary"
                                        } else {
                                            "font-medium"
                                        },
                                        onclick: {
                                            let project_id = project.project_id.clone();
                                            let on_select_project = on_select_project.clone();
                                            move |_| on_select_project.call(project_id.clone())
                                        },
                                        "{project.project_name}"
                                    }
                                    ul {
                                        for session in project.sessions {
                                            li {
                                                button {
                                                    class: if props.active_session_id.as_deref() == Some(session.session_id.as_str()) {
                                                        "btn btn-sm btn-primary btn-soft justify-between"
                                                    } else {
                                                        "btn btn-ghost btn-sm justify-between"
                                                    },
                                                    onclick: {
                                                        let session_id = session.session_id.clone();
                                                        let on_select_session = on_select_session.clone();
                                                        move |_| on_select_session.call(session_id.clone())
                                                    },
                                                    span { class: "truncate", "{session.title}" }
                                                    span { class: session.badge.class_name, "{session.badge.label}" }
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
}
