use crate::{presenters::TimelineEntryView, theme::CARD_CLASS};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct EventTimelineProps {
    pub entries: Vec<TimelineEntryView>,
}

#[component]
pub fn EventTimeline(props: EventTimelineProps) -> Element {
    rsx! {
        div { class: "flex flex-col gap-3",
            for entry in props.entries {
                div { class: CARD_CLASS,
                    div { class: "card-body gap-2 p-4",
                        div {
                            class: "flex items-center justify-between gap-3",
                            h3 { class: "font-medium", "{entry.title}" }
                            span { class: "badge badge-outline", "{entry.meta}" }
                        }
                        p { class: "text-sm text-base-content/80", "{entry.body}" }
                        div { class: entry.tone_class, "" }
                    }
                }
            }
        }
    }
}
