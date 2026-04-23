use crate::{presenters::ApprovalCardView, theme::CARD_CLASS};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ApprovalPanelProps {
    pub approvals: Vec<ApprovalCardView>,
    pub on_approve: EventHandler<String>,
    pub on_reject: EventHandler<String>,
}

#[component]
pub fn ApprovalPanel(props: ApprovalPanelProps) -> Element {
    let on_approve = props.on_approve.clone();
    let on_reject = props.on_reject.clone();

    rsx! {
        div { class: "flex flex-col gap-3",
            for approval in props.approvals {
                div { class: CARD_CLASS,
                    div { class: "card-body gap-3 p-4",
                        div {
                            class: "flex items-center justify-between gap-3",
                            h3 { class: "card-title text-base", "{approval.title}" }
                            span { class: "badge badge-warning", "待审批" }
                        }
                        p { class: "text-sm text-base-content/80", "{approval.summary}" }
                        p { class: "text-xs text-base-content/60", "请求 ID: {approval.request_id}" }
                        div {
                            class: "flex gap-2",
                            button {
                                class: "btn btn-primary btn-sm",
                                onclick: {
                                    let approval_id = approval.approval_id.clone();
                                    let on_approve = on_approve.clone();
                                    move |_| on_approve.call(approval_id.clone())
                                },
                                "批准"
                            }
                            button {
                                class: "btn btn-outline btn-sm",
                                onclick: {
                                    let approval_id = approval.approval_id.clone();
                                    let on_reject = on_reject.clone();
                                    move |_| on_reject.call(approval_id.clone())
                                },
                                "拒绝"
                            }
                        }
                    }
                }
            }
        }
    }
}
