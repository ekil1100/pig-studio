use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_kernel::ProjectId;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub root_path: PathBuf,
    pub pinned: bool,
    pub last_opened_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    pub fn new(name: impl Into<String>, root_path: PathBuf, now: DateTime<Utc>) -> Self {
        Self {
            id: ProjectId::new(),
            name: name.into(),
            root_path,
            pinned: false,
            last_opened_at: Some(now),
            created_at: now,
            updated_at: now,
        }
    }
}
