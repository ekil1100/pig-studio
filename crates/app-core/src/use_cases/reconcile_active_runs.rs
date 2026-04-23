use crate::{
    event_bus::{ApplicationEvent, EventBusPort, RuntimeEvent},
    ports::{
        EventRepositoryPort, InspectRunStatusRequest, PiMonoAdapterPort, RunRepositoryPort,
        SessionRepositoryPort, SettingsStorePort,
    },
};
use chrono::{DateTime, Utc};
use domain::{RunStatus, SessionStatus};
use serde_json::json;
use shared_kernel::AppResult;

pub fn execute<R, S, E, A, T, B>(
    runs: &R,
    sessions: &S,
    events: &E,
    adapter: &A,
    settings: &T,
    bus: &B,
    now: DateTime<Utc>,
) -> AppResult<()>
where
    R: RunRepositoryPort,
    S: SessionRepositoryPort,
    E: EventRepositoryPort,
    A: PiMonoAdapterPort,
    T: SettingsStorePort,
    B: EventBusPort,
{
    let runtime_settings = settings.load()?;
    let Some(runtime_path) = runtime_settings.runtime_path else {
        return Ok(());
    };

    for run in runs.list_active()? {
        let Some(session) = sessions.find(&run.session_id)? else {
            continue;
        };

        let Some(pimono_run_id) = run.pimono_run_id.clone() else {
            runs.update_terminal_state(
                &run.id,
                RunStatus::Interrupted,
                now,
                Some("INTERRUPTED".into()),
                Some("missing pimono_run_id".into()),
            )?;
            sessions.update_status(&session.id, SessionStatus::Interrupted, now)?;
            bus.publish(ApplicationEvent::SessionStatusChanged {
                session_id: session.id,
                status: SessionStatus::Interrupted,
            });
            continue;
        };

        let inspection = adapter.inspect_run_status(&InspectRunStatusRequest {
            runtime_path: runtime_path.clone(),
            workspace_cwd: session.workspace_cwd.clone(),
            pimono_run_id,
            env: runtime_settings.environment.clone(),
        });

        match inspection {
            Ok(result) => {
                if result.running {
                    bus.publish(ApplicationEvent::SessionStatusChanged {
                        session_id: session.id,
                        status: SessionStatus::Running,
                    });
                    continue;
                }

                if let Some(event) = result.terminal_event {
                    let (
                        event_type,
                        payload_json,
                        run_status,
                        session_status,
                        error_code,
                        error_message,
                    ) = match event.clone() {
                        RuntimeEvent::RunCompleted => (
                            "run_completed",
                            json!({}).to_string(),
                            RunStatus::Completed,
                            SessionStatus::Completed,
                            None,
                            None,
                        ),
                        RuntimeEvent::RunFailed { code, message } => (
                            "run_failed",
                            json!({ "code": code, "message": message }).to_string(),
                            RunStatus::Failed,
                            SessionStatus::Failed,
                            code,
                            Some(message),
                        ),
                        _ => continue,
                    };
                    events.append_event(
                        &session.id,
                        Some(&run.id),
                        event_type,
                        &payload_json,
                        now,
                    )?;
                    runs.update_terminal_state(
                        &run.id,
                        run_status,
                        now,
                        error_code,
                        error_message,
                    )?;
                    sessions.update_status(&session.id, session_status, now)?;
                    bus.publish(ApplicationEvent::RuntimeEventAppended {
                        session_id: session.id.clone(),
                        run_id: Some(run.id.clone()),
                        event,
                    });
                    bus.publish(ApplicationEvent::SessionStatusChanged {
                        session_id: session.id,
                        status: session_status,
                    });
                    continue;
                }

                runs.update_terminal_state(
                    &run.id,
                    RunStatus::Interrupted,
                    now,
                    Some("INTERRUPTED".into()),
                    Some("run could not be recovered".into()),
                )?;
                sessions.update_status(&session.id, SessionStatus::Interrupted, now)?;
                bus.publish(ApplicationEvent::SessionStatusChanged {
                    session_id: session.id,
                    status: SessionStatus::Interrupted,
                });
            }
            Err(_) => {
                runs.update_terminal_state(
                    &run.id,
                    RunStatus::Interrupted,
                    now,
                    Some("INTERRUPTED".into()),
                    Some("run could not be recovered".into()),
                )?;
                sessions.update_status(&session.id, SessionStatus::Interrupted, now)?;
                bus.publish(ApplicationEvent::SessionStatusChanged {
                    session_id: session.id,
                    status: SessionStatus::Interrupted,
                });
            }
        }
    }

    Ok(())
}
