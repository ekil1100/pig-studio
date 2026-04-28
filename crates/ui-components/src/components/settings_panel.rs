use crate::presenters::runtime_health_summary;
use coss_ui_dioxus::{Badge, Button, ButtonSize, ButtonVariant, Card};
use dioxus::prelude::*;
use domain::RuntimeHealth;

#[derive(Props, Clone, PartialEq)]
pub struct SettingsPanelProps {
    pub runtime_path: String,
    pub runtime_source_label: String,
    pub config_dir: String,
    pub config_dir_source_label: String,
    pub has_runtime_override: bool,
    pub has_config_dir_override: bool,
    pub health: RuntimeHealth,
    pub on_refresh: EventHandler<()>,
    pub on_pick_runtime_binary: EventHandler<()>,
    pub on_pick_config_dir: EventHandler<()>,
    pub on_clear_overrides: EventHandler<()>,
}

#[component]
pub fn SettingsPanel(props: SettingsPanelProps) -> Element {
    let (summary, badge_class) = runtime_health_summary(&props.health);
    let on_refresh = props.on_refresh.clone();
    let on_pick_runtime_binary = props.on_pick_runtime_binary.clone();
    let on_pick_config_dir = props.on_pick_config_dir.clone();
    let on_clear_overrides = props.on_clear_overrides.clone();
    let runtime_reason = props.health.reason.clone().unwrap_or_else(|| {
        props
            .health
            .version
            .clone()
            .map(|version| format!("已检测到版本：{version}"))
            .unwrap_or_else(|| "Pig Studio 会优先自动检测 Pi 运行时。".into())
    });

    rsx! {
        Card { class: "overflow-hidden",
            div { class: "flex flex-col gap-5 p-5",
                div {
                    class: "flex flex-wrap items-center justify-between gap-3",
                    div {
                        p { class: "studio-kicker", "Runtime" }
                        h3 { class: "mt-1 text-lg font-semibold", "运行时检测" }
                        p { class: "mt-2 text-sm leading-6 text-foreground/62", "默认自动检测 Pi 二进制与配置目录；只有检测失败时才建议手动覆盖。" }
                    }
                    Badge { variant: badge_class, "{summary}" }
                }

                div { class: "grid gap-3 md:grid-cols-2",
                    div { class: "rounded-md border border-border bg-muted/60 p-3",
                        div { class: "text-xs font-medium text-foreground/45", "Pi 二进制" }
                        div { class: "mt-3 break-all text-sm font-medium leading-6",
                            if props.runtime_path.is_empty() {
                                "未检测到"
                            } else {
                                "{props.runtime_path}"
                            }
                        }
                        div { class: "mt-2 text-xs text-foreground/55", "来源：{props.runtime_source_label}" }
                    }
                    div { class: "rounded-md border border-border bg-muted/60 p-3",
                        div { class: "text-xs font-medium text-foreground/45", "Pi 配置目录" }
                        div { class: "mt-3 break-all text-sm font-medium leading-6",
                            if props.config_dir.is_empty() {
                                "未检测到，将依赖 Pi 默认行为"
                            } else {
                                "{props.config_dir}"
                            }
                        }
                        div { class: "mt-2 text-xs text-foreground/55", "来源：{props.config_dir_source_label}" }
                    }
                }

                p {
                    class: "rounded-md bg-muted/70 px-3 py-2 text-sm leading-6 text-foreground/62",
                    "{runtime_reason}"
                }

                div {
                    class: "flex flex-wrap justify-end gap-2",
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        class: "rounded-md",
                        onclick: move |_| on_refresh.call(()),
                        "重新检测"
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        class: "rounded-md",
                        onclick: move |_| on_pick_runtime_binary.call(()),
                        "选择自定义二进制"
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        class: "rounded-md",
                        onclick: move |_| on_pick_config_dir.call(()),
                        "选择自定义配置目录"
                    }
                    if props.has_runtime_override || props.has_config_dir_override {
                        Button {
                            variant: ButtonVariant::WarningOutline,
                            size: ButtonSize::Sm,
                            class: "rounded-md",
                            onclick: move |_| on_clear_overrides.call(()),
                            "恢复自动检测"
                        }
                    }
                }
            }
        }
    }
}
