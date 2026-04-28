use crate::presenters::StatusBadgeMeta;
use coss_ui_dioxus::{Badge, Breadcrumb, Surface};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SessionHeaderProps {
    pub project_name: String,
    pub session_name: String,
    pub badge: StatusBadgeMeta,
}

#[component]
pub fn SessionHeader(props: SessionHeaderProps) -> Element {
    rsx! {
        Surface {
            class: "flex flex-wrap items-center justify-between gap-3 px-4 py-3",
            div { class: "min-w-0 space-y-1.5",
                Breadcrumb { class: "p-0 text-sm text-foreground/55",
                    ul {
                        li { span { "{props.project_name}" } }
                        li { span { "Session" } }
                    }
                }
                div { class: "flex min-w-0 flex-wrap items-center gap-3",
                    h1 { class: "truncate text-xl font-semibold text-foreground", "{props.session_name}" }
                    Badge { variant: props.badge.variant, "{props.badge.label}" }
                }
                p { class: "studio-muted", "持久会话、执行历史和审批都围绕当前项目上下文展开。" }
            }
        }
    }
}
