use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeHealth {
    pub available: bool,
    pub version: Option<String>,
    pub reason: Option<String>,
    pub checked_at: Option<DateTime<Utc>>,
}

impl RuntimeHealth {
    pub fn available(version: impl Into<String>, checked_at: DateTime<Utc>) -> Self {
        Self {
            available: true,
            version: Some(version.into()),
            reason: None,
            checked_at: Some(checked_at),
        }
    }

    pub fn blocked(reason: impl Into<String>, checked_at: DateTime<Utc>) -> Self {
        Self {
            available: false,
            version: None,
            reason: Some(reason.into()),
            checked_at: Some(checked_at),
        }
    }
}
