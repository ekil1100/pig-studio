use base_ui_dioxus::{
    AlertPrimitive, BadgePrimitive, BreadcrumbPrimitive, ButtonPrimitive, InputPrimitive,
    SurfacePrimitive, TextareaPrimitive, compose_class,
};
use dioxus::prelude::*;

fn join_classes(parts: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    parts
        .into_iter()
        .map(|part| part.as_ref().trim().to_owned())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
    DestructiveOutline,
    WarningOutline,
    Bare,
}

impl Default for ButtonVariant {
    fn default() -> Self {
        Self::Primary
    }
}

impl ButtonVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Primary => "coss-button-primary",
            Self::Secondary => "coss-button-secondary",
            Self::Outline => "coss-button-outline",
            Self::Ghost => "coss-button-ghost",
            Self::DestructiveOutline => "coss-button-destructive-outline",
            Self::WarningOutline => "coss-button-warning-outline",
            Self::Bare => "",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonSize {
    Xs,
    Sm,
    Default,
}

impl Default for ButtonSize {
    fn default() -> Self {
        Self::Default
    }
}

impl ButtonSize {
    fn class(self) -> &'static str {
        match self {
            Self::Xs => "coss-button-xs",
            Self::Sm => "coss-button-sm",
            Self::Default => "",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    #[props(default)]
    pub variant: ButtonVariant,
    #[props(default)]
    pub size: ButtonSize,
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
pub fn Button(props: ButtonProps) -> Element {
    let disabled_class = if props.disabled {
        "coss-button-disabled"
    } else {
        ""
    };
    let class = join_classes([
        "coss-button",
        props.variant.class(),
        props.size.class(),
        disabled_class,
        props.class.as_str(),
    ]);

    rsx! {
        ButtonPrimitive {
            class,
            disabled: props.disabled,
            button_type: props.button_type,
            onclick: props.onclick,
            {props.children}
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadgeVariant {
    Outline,
    Ghost,
    Secondary,
    Neutral,
    Info,
    Success,
    Warning,
    Error,
}

impl Default for BadgeVariant {
    fn default() -> Self {
        Self::Outline
    }
}

impl BadgeVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Outline => "coss-badge-outline",
            Self::Ghost => "coss-badge-ghost",
            Self::Secondary => "coss-badge-secondary",
            Self::Neutral => "coss-badge-neutral",
            Self::Info => "coss-badge-info",
            Self::Success => "coss-badge-success",
            Self::Warning => "coss-badge-warning",
            Self::Error => "coss-badge-error",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadgeSize {
    Sm,
    Default,
}

impl Default for BadgeSize {
    fn default() -> Self {
        Self::Default
    }
}

impl BadgeSize {
    fn class(self) -> &'static str {
        match self {
            Self::Sm => "coss-badge-sm",
            Self::Default => "",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct BadgeProps {
    #[props(default)]
    pub variant: BadgeVariant,
    #[props(default)]
    pub size: BadgeSize,
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let class = join_classes([
        "coss-badge",
        props.variant.class(),
        props.size.class(),
        props.class.as_str(),
    ]);

    rsx! {
        BadgePrimitive { class, {props.children} }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertVariant {
    Info,
    Success,
    Warning,
    Error,
}

impl Default for AlertVariant {
    fn default() -> Self {
        Self::Info
    }
}

impl AlertVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Info => "coss-alert-info",
            Self::Success => "coss-alert-success",
            Self::Warning => "coss-alert-warning",
            Self::Error => "coss-alert-error",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AlertProps {
    #[props(default)]
    pub variant: AlertVariant,
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn Alert(props: AlertProps) -> Element {
    let class = join_classes(["coss-alert", props.variant.class(), props.class.as_str()]);

    rsx! {
        AlertPrimitive { class, {props.children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
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
pub fn Input(props: InputProps) -> Element {
    let class = compose_class("studio-input", &props.class);

    rsx! {
        InputPrimitive {
            class,
            value: props.value,
            placeholder: props.placeholder,
            input_type: props.input_type,
            disabled: props.disabled,
            on_input: props.on_input,
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TextareaProps {
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
pub fn Textarea(props: TextareaProps) -> Element {
    let class = compose_class("studio-textarea", &props.class);

    rsx! {
        TextareaPrimitive {
            class,
            value: props.value,
            placeholder: props.placeholder,
            disabled: props.disabled,
            on_input: props.on_input,
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SurfaceProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn Surface(props: SurfaceProps) -> Element {
    let class = compose_class("studio-surface", &props.class);

    rsx! {
        SurfacePrimitive { class, {props.children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn Card(props: CardProps) -> Element {
    let class = compose_class("studio-card", &props.class);

    rsx! {
        SurfacePrimitive { class, {props.children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct BreadcrumbProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn Breadcrumb(props: BreadcrumbProps) -> Element {
    let class = compose_class("coss-breadcrumb", &props.class);

    rsx! {
        BreadcrumbPrimitive { class, {props.children} }
    }
}
