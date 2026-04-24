use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ComposerProps {
    pub value: String,
    pub on_input: EventHandler<String>,
    pub on_submit: EventHandler<()>,
    #[props(default)]
    pub placeholder: Option<String>,
    #[props(default = false)]
    pub busy: bool,
}

#[component]
pub fn Composer(props: ComposerProps) -> Element {
    let placeholder = props
        .placeholder
        .unwrap_or_else(|| "告诉 Pig Studio 接下来要做什么".into());
    let on_input = props.on_input.clone();
    let on_submit = props.on_submit.clone();

    rsx! {
        div {
            class: "studio-surface sticky bottom-0",
            div {
                class: "flex flex-col gap-3 p-4",
                div { class: "flex items-start justify-between gap-3",
                    div {
                        p { class: "studio-kicker", "Composer" }
                        p { class: "mt-1 text-base font-semibold", "继续当前 agent session" }
                    }
                    span {
                        class: if props.busy {
                            "badge badge-warning border-none px-3 py-3 font-medium"
                        } else {
                            "badge badge-ghost border-none bg-base-200/80 px-3 py-3 font-medium"
                        },
                        if props.busy { "运行中" } else { "可发送" }
                    }
                }
                textarea {
                    class: "studio-textarea min-h-28 w-full rounded-md px-3 py-2 text-sm leading-6",
                    value: props.value,
                    placeholder: placeholder,
                    disabled: props.busy,
                    oninput: move |event| on_input.call(event.value()),
                }
                div {
                    class: "flex flex-wrap items-center justify-between gap-3",
                    span {
                        class: "text-sm text-base-content/58",
                        if props.busy { "正在运行，请等待事件流更新或审批结果。" } else { "支持多轮连续会话，发送内容会追加到当前上下文。" }
                    }
                    button {
                        class: if props.busy {
                            "btn btn-primary btn-sm rounded-md px-4 btn-disabled"
                        } else {
                            "btn btn-primary btn-sm rounded-md px-4"
                        },
                        disabled: props.busy,
                        onclick: move |_| on_submit.call(()),
                        "发送"
                    }
                }
            }
        }
    }
}
