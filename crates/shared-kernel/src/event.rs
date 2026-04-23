use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    pub seq: i64,
    pub created_at: DateTime<Utc>,
    pub payload: T,
}

impl<T> EventEnvelope<T> {
    pub fn new(seq: i64, created_at: DateTime<Utc>, payload: T) -> Self {
        Self {
            seq,
            created_at,
            payload,
        }
    }
}
