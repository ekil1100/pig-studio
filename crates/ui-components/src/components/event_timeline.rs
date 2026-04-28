use crate::presenters::TimelineEntryView;
use coss_ui_dioxus::{Badge, BadgeVariant, Card};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct EventTimelineProps {
    pub entries: Vec<TimelineEntryView>,
}

#[component]
pub fn EventTimeline(props: EventTimelineProps) -> Element {
    rsx! {
        section { class: "flex flex-col gap-4",
            div { class: "flex items-end justify-between gap-3 px-1",
                div {
                    p { class: "studio-kicker", "Timeline" }
                    h2 { class: "mt-1 text-lg font-semibold", "会话事件流" }
                }
                p { class: "text-sm text-foreground/50", "按时间顺序展示运行、输出、错误和恢复信息" }
            }
            for entry in props.entries {
                Card { class: "overflow-hidden",
                    div { class: "grid gap-3 p-4 md:grid-cols-[auto_minmax(0,1fr)]",
                        div { class: "hidden pt-1 md:block",
                            div { class: timeline_marker_class(&entry.tone_class), "" }
                        }
                        div { class: "min-w-0",
                            div {
                                class: "flex flex-wrap items-center justify-between gap-3",
                                h3 { class: "text-sm font-semibold", "{entry.title}" }
                                Badge { variant: BadgeVariant::Ghost, class: "border-none bg-muted/80 px-3 py-3 text-xs font-medium text-foreground/62", "{entry.meta}" }
                            }
                            p { class: "mt-3 whitespace-pre-wrap text-[15px] leading-7 text-foreground/76", "{entry.body}" }
                            div { class: "mt-4 h-px w-full bg-border/80", "" }
                        }
                    }
                }
            }
        }
    }
}

fn timeline_marker_class(tone_class: &str) -> &'static str {
    if tone_class.contains("error") {
        "mt-1 h-2.5 w-2.5 rounded-full bg-error"
    } else if tone_class.contains("warning") {
        "mt-1 h-2.5 w-2.5 rounded-full bg-warning"
    } else if tone_class.contains("success") {
        "mt-1 h-2.5 w-2.5 rounded-full bg-success"
    } else if tone_class.contains("info") {
        "mt-1 h-2.5 w-2.5 rounded-full bg-info"
    } else if tone_class.contains("accent") {
        "mt-1 h-2.5 w-2.5 rounded-full bg-primary"
    } else {
        "mt-1 h-2.5 w-2.5 rounded-full bg-border"
    }
}
