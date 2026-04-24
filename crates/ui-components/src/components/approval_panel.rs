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
        section { class: "flex flex-col gap-4",
            div { class: "flex items-end justify-between gap-3 px-1",
                div {
                    p { class: "studio-kicker", "Approvals" }
                    h2 { class: "mt-1 text-lg font-semibold", "待处理审批" }
                }
                span { class: "badge badge-warning border-none px-3 py-3 font-medium", "{props.approvals.len()} 条待处理" }
            }
            for approval in props.approvals {
                div { class: CARD_CLASS,
                    div { class: "flex flex-col gap-3 border border-warning/30 bg-warning/6 p-4",
                        div {
                            class: "flex items-center justify-between gap-3",
                            h3 { class: "text-sm font-semibold", "{approval.title}" }
                            span { class: "badge badge-warning border-none px-3 py-3 font-medium", "待审批" }
                        }
                        p { class: "whitespace-pre-wrap text-[15px] leading-7 text-base-content/76", "{approval.summary}" }
                        p { class: "text-sm text-base-content/52", "请求 ID: {approval.request_id}" }
                        div {
                            class: "flex gap-2",
                            button {
                                class: "btn btn-primary btn-sm rounded-md px-4",
                                onclick: {
                                    let approval_id = approval.approval_id.clone();
                                    let on_approve = on_approve.clone();
                                    move |_| on_approve.call(approval_id.clone())
                                },
                                "批准"
                            }
                            button {
                                class: "btn btn-outline btn-sm rounded-md px-4",
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
