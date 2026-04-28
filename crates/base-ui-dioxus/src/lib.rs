use dioxus::prelude::*;

fn join_classes(parts: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    parts
        .into_iter()
        .map(|part| part.as_ref().trim().to_owned())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Props, Clone, PartialEq)]
pub struct ButtonPrimitiveProps {
    #[props(default)]
    pub class: String,
    #[props(default)]
    pub disabled: bool,
    #[props(default = "button".to_owned())]
    pub button_type: String,
    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,
    pub children: Element,
}

#[component]
pub fn ButtonPrimitive(props: ButtonPrimitiveProps) -> Element {
    let onclick = props.onclick;

    rsx! {
        button {
            class: props.class,
            disabled: props.disabled,
            r#type: "{props.button_type}",
            onclick: move |event| {
                if let Some(handler) = onclick {
                    handler.call(event);
                }
            },
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct BadgePrimitiveProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn BadgePrimitive(props: BadgePrimitiveProps) -> Element {
    rsx! {
        span { class: props.class, {props.children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AlertPrimitiveProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn AlertPrimitive(props: AlertPrimitiveProps) -> Element {
    rsx! {
        div { class: props.class, role: "status", {props.children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct InputPrimitiveProps {
    #[props(default)]
    pub class: String,
    #[props(default)]
    pub value: String,
    #[props(default)]
    pub placeholder: String,
    #[props(default = "text".to_owned())]
    pub input_type: String,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub on_input: Option<EventHandler<String>>,
}

#[component]
pub fn InputPrimitive(props: InputPrimitiveProps) -> Element {
    let on_input = props.on_input;

    rsx! {
        input {
            class: props.class,
            r#type: "{props.input_type}",
            value: props.value,
            placeholder: props.placeholder,
            disabled: props.disabled,
            oninput: move |event| {
                if let Some(handler) = on_input {
                    handler.call(event.value());
                }
            },
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TextareaPrimitiveProps {
    #[props(default)]
    pub class: String,
    #[props(default)]
    pub value: String,
    #[props(default)]
    pub placeholder: String,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub on_input: Option<EventHandler<String>>,
}

#[component]
pub fn TextareaPrimitive(props: TextareaPrimitiveProps) -> Element {
    let on_input = props.on_input;

    rsx! {
        textarea {
            class: props.class,
            value: props.value,
            placeholder: props.placeholder,
            disabled: props.disabled,
            oninput: move |event| {
                if let Some(handler) = on_input {
                    handler.call(event.value());
                }
            },
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SurfacePrimitiveProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn SurfacePrimitive(props: SurfacePrimitiveProps) -> Element {
    rsx! {
        div { class: props.class, {props.children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct BreadcrumbPrimitiveProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn BreadcrumbPrimitive(props: BreadcrumbPrimitiveProps) -> Element {
    rsx! {
        nav { class: props.class, aria_label: "Breadcrumb", {props.children} }
    }
}

pub fn compose_class(base: &str, class: &str) -> String {
    join_classes([base, class])
}
