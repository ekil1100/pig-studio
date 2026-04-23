use super::{
    approval_decision_from_str, approval_decision_to_str, approval_status_from_str,
    approval_status_to_str, parse_ts, ts,
};
use app_core::ApprovalRepositoryPort;
use chrono::{DateTime, Utc};
use domain::{Approval, ApprovalDecision, ApprovalRequest, ApprovalStatus};
use rusqlite::{Connection, params};
use shared_kernel::{AppError, AppResult, ApprovalId, RunId, SessionId};
use std::rc::Rc;

#[derive(Clone)]
pub struct ApprovalRepository {
    connection: Rc<Connection>,
}

impl ApprovalRepository {
    pub fn new(connection: Rc<Connection>) -> Self {
        Self { connection }
    }

    pub fn create(&self, approval: &Approval) -> AppResult<()> {
        self.connection
            .execute(
                "INSERT INTO approvals (id, run_id, request_id, correlation_id, request_type, request_payload_json, status, decision, created_at, decided_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    approval.id.as_str(),
                    approval.run_id.as_str(),
                    approval.request.request_id,
                    approval.request.correlation_id,
                    approval.request.request_type,
                    approval.request.request_payload_json,
                    approval_status_to_str(approval.status),
                    approval.decision.map(approval_decision_to_str),
                    ts(approval.created_at),
                    approval.decided_at.map(ts),
                ],
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        Ok(())
    }

    pub fn record_decision(
        &self,
        approval_id: &ApprovalId,
        status: ApprovalStatus,
        decision: ApprovalDecision,
        decided_at: DateTime<Utc>,
    ) -> AppResult<()> {
        self.connection
            .execute(
                "UPDATE approvals SET status = ?2, decision = ?3, decided_at = ?4 WHERE id = ?1",
                params![
                    approval_id.as_str(),
                    approval_status_to_str(status),
                    approval_decision_to_str(decision),
                    ts(decided_at),
                ],
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        Ok(())
    }

    pub fn list_pending_by_session(&self, session_id: &SessionId) -> AppResult<Vec<Approval>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT a.id, a.run_id, a.request_id, a.correlation_id, a.request_type, a.request_payload_json, a.status, a.decision, a.created_at, a.decided_at
                 FROM approvals a
                 INNER JOIN runs r ON r.id = a.run_id
                 WHERE r.session_id = ?1 AND a.status = 'pending'
                 ORDER BY a.created_at ASC",
            )
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        let rows = statement
            .query_map(params![session_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, Option<String>>(9)?,
                ))
            })
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        rows.map(|row| {
            let (
                id,
                run_id,
                request_id,
                correlation_id,
                request_type,
                request_payload_json,
                status,
                decision,
                created_at,
                decided_at,
            ) = row.map_err(|error| AppError::Infrastructure(error.to_string()))?;
            Ok(Approval {
                id: ApprovalId::from_string(id),
                run_id: RunId::from_string(run_id),
                request: ApprovalRequest {
                    request_id,
                    correlation_id,
                    request_type,
                    request_payload_json,
                },
                status: approval_status_from_str(&status)?,
                decision: decision
                    .as_deref()
                    .map(approval_decision_from_str)
                    .transpose()?,
                created_at: parse_ts(created_at)?,
                decided_at: decided_at.map(parse_ts).transpose()?,
            })
        })
        .collect::<AppResult<Vec<Approval>>>()
    }
}

impl ApprovalRepositoryPort for ApprovalRepository {
    fn create(&self, approval: &Approval) -> AppResult<()> {
        ApprovalRepository::create(self, approval)
    }

    fn record_decision(
        &self,
        approval_id: &ApprovalId,
        status: ApprovalStatus,
        decision: ApprovalDecision,
        decided_at: DateTime<Utc>,
    ) -> AppResult<()> {
        ApprovalRepository::record_decision(self, approval_id, status, decision, decided_at)
    }
}
