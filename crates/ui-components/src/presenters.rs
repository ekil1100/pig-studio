use coss_ui_dioxus::BadgeVariant;
use domain::{RuntimeHealth, SessionStatus};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusBadgeMeta {
    pub label: &'static str,
    pub variant: BadgeVariant,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionBannerView {
    pub title: String,
    pub body: String,
    pub action_label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectTreeItemView {
    pub project_id: String,
    pub project_name: String,
    pub sessions: Vec<SessionTreeItemView>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionTreeItemView {
    pub session_id: String,
    pub title: String,
    pub badge: StatusBadgeMeta,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimelineEntryView {
    pub title: String,
    pub body: String,
    pub meta: String,
    pub tone_class: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApprovalCardView {
    pub approval_id: String,
    pub title: String,
    pub summary: String,
    pub request_id: String,
}

pub fn session_status_badge(status: SessionStatus) -> StatusBadgeMeta {
    match status {
        SessionStatus::Idle => StatusBadgeMeta {
            label: "空闲",
            variant: BadgeVariant::Ghost,
        },
        SessionStatus::Running => StatusBadgeMeta {
            label: "运行中",
            variant: BadgeVariant::Info,
        },
        SessionStatus::WaitingApproval => StatusBadgeMeta {
            label: "等待审批",
            variant: BadgeVariant::Warning,
        },
        SessionStatus::Blocked => StatusBadgeMeta {
            label: "已阻塞",
            variant: BadgeVariant::Secondary,
        },
        SessionStatus::Completed => StatusBadgeMeta {
            label: "已完成",
            variant: BadgeVariant::Success,
        },
        SessionStatus::Failed => StatusBadgeMeta {
            label: "失败",
            variant: BadgeVariant::Error,
        },
        SessionStatus::Interrupted => StatusBadgeMeta {
            label: "已中断",
            variant: BadgeVariant::Neutral,
        },
    }
}

pub fn build_session_banner(status: SessionStatus) -> SessionBannerView {
    match status {
        SessionStatus::Blocked => SessionBannerView {
            title: "运行环境不可用".into(),
            body: "当前会话无法继续执行，请检查 Pi 运行时、配置目录或工作目录可访问性。".into(),
            action_label: Some("去设置修复".into()),
        },
        SessionStatus::WaitingApproval => SessionBannerView {
            title: "等待你的审批".into(),
            body: "代理已经运行到敏感操作，请确认是批准还是拒绝。".into(),
            action_label: Some("查看审批".into()),
        },
        SessionStatus::Interrupted => SessionBannerView {
            title: "上次运行已中断".into(),
            body: "应用重启或外部进程退出后，这个会话需要你重新决定下一步。".into(),
            action_label: Some("重新恢复".into()),
        },
        SessionStatus::Failed => SessionBannerView {
            title: "运行失败".into(),
            body: "查看错误详情并决定是否在当前上下文中重试。".into(),
            action_label: Some("查看错误".into()),
        },
        _ => SessionBannerView {
            title: "会话已就绪".into(),
            body: "你可以继续发送 prompt，或者切换到其他项目会话。".into(),
            action_label: None,
        },
    }
}

pub fn runtime_health_summary(health: &RuntimeHealth) -> (&'static str, BadgeVariant) {
    if health.available {
        ("运行时可用", BadgeVariant::Success)
    } else {
        ("运行时异常", BadgeVariant::Error)
    }
}
