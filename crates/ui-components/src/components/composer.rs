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
            class: "card bg-base-100 shadow-sm",
            div {
                class: "card-body gap-3 p-4",
                textarea {
                    class: "textarea textarea-bordered min-h-28 w-full",
                    value: props.value,
                    placeholder: placeholder,
                    disabled: props.busy,
                    oninput: move |event| on_input.call(event.value()),
                }
                div {
                    class: "flex items-center justify-between gap-3",
                    span {
                        class: "text-xs text-base-content/60",
                        if props.busy { "正在运行，请等待事件流更新" } else { "支持多轮连续会话" }
                    }
                    button {
                        class: if props.busy { "btn btn-primary btn-sm btn-disabled" } else { "btn btn-primary btn-sm" },
                        disabled: props.busy,
                        onclick: move |_| on_submit.call(()),
                        "发送"
                    }
                }
            }
        }
    }
}
