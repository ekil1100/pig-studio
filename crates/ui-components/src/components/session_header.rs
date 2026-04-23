use crate::presenters::StatusBadgeMeta;
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
        div {
            class: "flex items-center justify-between rounded-box bg-base-100 p-4 shadow-sm",
            div {
                h1 { class: "text-xl font-semibold", "{props.session_name}" }
                p { class: "text-sm text-base-content/70", "{props.project_name}" }
            }
            span { class: props.badge.class_name, "{props.badge.label}" }
        }
    }
}
