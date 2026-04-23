use crate::{
    models::{
        InspectRunStatusRequest, PiMonoEvent, RespondApprovalRequest, ResumeSessionRequest,
        RunInspection, StartSessionRunRequest,
    },
    process::{ProcessRunner, StreamingProcessSink},
    stream_parser::StreamParser,
};
use app_core::{
    InspectRunStatusRequest as CoreInspectRunStatusRequest, PiMonoAdapterPort,
    RespondApprovalRequest as CoreRespondApprovalRequest,
    ResumeSessionRequest as CoreResumeSessionRequest, RunInspection as CoreRunInspection,
    RuntimeEvent, RuntimeEventSink, StartSessionRunRequest as CoreStartSessionRunRequest,
};
use shared_kernel::{AppError, AppResult};
use std::{collections::BTreeMap, fs, path::Path};

pub trait PiMonoEventSink {
    fn push(&self, event: PiMonoEvent);
}

struct RuntimeEventSinkAdapter<'a> {
    sink: &'a dyn RuntimeEventSink,
}

impl PiMonoEventSink for RuntimeEventSinkAdapter<'_> {
    fn push(&self, event: PiMonoEvent) {
        let runtime_event = match event {
            PiMonoEvent::SessionBound { pimono_session_id } => {
                RuntimeEvent::SessionBound { pimono_session_id }
            }
            PiMonoEvent::RunStarted {
                pimono_run_id,
                pimono_session_id,
            } => RuntimeEvent::RunStarted {
                pimono_run_id,
                pimono_session_id,
            },
            PiMonoEvent::TextDelta { text } => RuntimeEvent::TextDelta { text },
            PiMonoEvent::ApprovalRequested {
                request_id,
                request_type,
                payload_json,
            } => RuntimeEvent::ApprovalRequested {
                request_id,
                request_type,
                payload_json,
            },
            PiMonoEvent::RunFailed { code, message } => RuntimeEvent::RunFailed { code, message },
            PiMonoEvent::RunCompleted => RuntimeEvent::RunCompleted,
        };
        let _ = self.sink.push(runtime_event);
    }
}

#[derive(Debug, Clone)]
pub struct PiMonoAdapter<R> {
    runner: R,
    parser: StreamParser,
}

impl<R> PiMonoAdapter<R>
where
    R: ProcessRunner,
{
    pub fn new(runner: R) -> Self {
        Self {
            runner,
            parser: StreamParser,
        }
    }

    pub fn start_session_run(
        &self,
        request: &StartSessionRunRequest,
        sink: &impl PiMonoEventSink,
    ) -> AppResult<()> {
        let args = if is_pi_cli(&request.runtime_path) {
            build_pi_prompt_args(&request.prompt, &request.env)?
        } else {
            let mut args = vec![
                "session".to_string(),
                "run".to_string(),
                request.prompt.clone(),
            ];
            if let Some(session_id) = &request.pimono_session_id {
                args.push("--session-id".to_string());
                args.push(session_id.clone());
            }
            args
        };
        self.run_and_stream(
            &request.runtime_path,
            &args,
            &request.workspace_cwd,
            &request.env,
            sink,
        )
    }

    pub fn resume_session(
        &self,
        request: &ResumeSessionRequest,
        sink: &impl PiMonoEventSink,
    ) -> AppResult<()> {
        if is_pi_cli(&request.runtime_path) {
            return Err(AppError::External(
                "当前 Pi CLI 不支持恢复到已经结束的历史流。请基于当前上下文开始新会话。".into(),
            ));
        }

        let args = vec![
            "session".to_string(),
            "resume".to_string(),
            "--session-id".to_string(),
            request.pimono_session_id.clone(),
        ];
        self.run_and_stream(
            &request.runtime_path,
            &args,
            &request.workspace_cwd,
            &request.env,
            sink,
        )
    }

    pub fn respond_approval(&self, request: &RespondApprovalRequest) -> AppResult<()> {
        if is_pi_cli(&request.runtime_path) {
            return Err(AppError::External(
                "当前 Pi CLI 尚未接入审批回传协议。".into(),
            ));
        }

        let decision = if request.approve { "approve" } else { "reject" };
        let args = vec![
            "approval".to_string(),
            "respond".to_string(),
            "--request-id".to_string(),
            request.request_id.clone(),
            "--decision".to_string(),
            decision.to_string(),
        ];
        let output = self.runner.run(
            &request.runtime_path,
            &args,
            &request.workspace_cwd,
            &request.env,
        )?;
        if output.exit_code == Some(0) {
            Ok(())
        } else {
            Err(AppError::External(
                output.stderr_lines.join("\n").trim().to_owned(),
            ))
        }
    }

    pub fn inspect_run_status(
        &self,
        request: &InspectRunStatusRequest,
    ) -> AppResult<RunInspection> {
        if is_pi_cli(&request.runtime_path) {
            return Err(AppError::External(
                "当前 Pi CLI 不支持 detached run inspect。".into(),
            ));
        }

        let args = vec![
            "run".to_string(),
            "inspect".to_string(),
            "--run-id".to_string(),
            request.pimono_run_id.clone(),
        ];
        let output = self.runner.run(
            &request.runtime_path,
            &args,
            &request.workspace_cwd,
            &request.env,
        )?;

        let mut events = Vec::new();
        for line in output.stdout_lines {
            events.extend(self.parser.parse_chunk(&line));
        }
        let terminal_event = events.into_iter().find(|event| {
            matches!(
                event,
                PiMonoEvent::RunCompleted | PiMonoEvent::RunFailed { .. }
            )
        });

        Ok(RunInspection {
            running: terminal_event.is_none() && output.exit_code == Some(0),
            terminal_event,
        })
    }

    fn run_and_stream(
        &self,
        runtime_path: &std::path::Path,
        args: &[String],
        workspace_cwd: &std::path::Path,
        env: &std::collections::BTreeMap<String, String>,
        sink: &impl PiMonoEventSink,
    ) -> AppResult<()> {
        struct AdapterStreamingSink<'a> {
            parser: StreamParser,
            event_sink: &'a dyn PiMonoEventSink,
        }

        impl StreamingProcessSink for AdapterStreamingSink<'_> {
            fn stdout_line(&mut self, line: String) -> AppResult<()> {
                for event in self.parser.parse_chunk(&line) {
                    self.event_sink.push(event);
                }
                Ok(())
            }

            fn stderr_line(&mut self, line: String) -> AppResult<()> {
                self.event_sink.push(PiMonoEvent::RunFailed {
                    code: None,
                    message: line,
                });
                Ok(())
            }
        }

        let mut streaming_sink = AdapterStreamingSink {
            parser: self.parser,
            event_sink: sink,
        };
        let exit_code = self.runner.run_streaming(
            runtime_path,
            args,
            workspace_cwd,
            env,
            &mut streaming_sink,
        )?;

        if exit_code == Some(0) {
            Ok(())
        } else {
            Err(AppError::External("runtime command failed".into()))
        }
    }
}

fn is_pi_cli(runtime_path: &Path) -> bool {
    runtime_path
        .file_stem()
        .and_then(|name| name.to_str())
        .map(|name| name.eq_ignore_ascii_case("pi"))
        .unwrap_or(false)
}

fn build_pi_prompt_args(prompt: &str, env: &BTreeMap<String, String>) -> AppResult<Vec<String>> {
    let mut args = vec!["--mode".into(), "json".into()];

    if let Some(session_dir) = env.get("PIG_STUDIO_PI_SESSION_DIR") {
        let session_dir = Path::new(session_dir);
        fs::create_dir_all(session_dir)
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;
        args.push("--session-dir".into());
        args.push(session_dir.to_string_lossy().into_owned());
        if session_dir_has_sessions(session_dir)? {
            args.push("--continue".into());
        }
    }

    args.push(prompt.to_owned());
    Ok(args)
}

fn session_dir_has_sessions(session_dir: &Path) -> AppResult<bool> {
    let entries =
        fs::read_dir(session_dir).map_err(|error| AppError::Infrastructure(error.to_string()))?;
    for entry in entries {
        let entry = entry.map_err(|error| AppError::Infrastructure(error.to_string()))?;
        let path = entry.path();
        if path.is_file() {
            return Ok(true);
        }
        if path.is_dir() {
            return Ok(true);
        }
    }
    Ok(false)
}

impl<R> PiMonoAdapterPort for PiMonoAdapter<R>
where
    R: ProcessRunner,
{
    fn start_session_run(
        &self,
        request: &CoreStartSessionRunRequest,
        sink: &dyn RuntimeEventSink,
    ) -> AppResult<()> {
        PiMonoAdapter::start_session_run(
            self,
            &StartSessionRunRequest {
                runtime_path: request.runtime_path.clone(),
                workspace_cwd: request.workspace_cwd.clone(),
                pimono_session_id: request.pimono_session_id.clone(),
                prompt: request.prompt.clone(),
                env: request.env.clone(),
            },
            &RuntimeEventSinkAdapter { sink },
        )
    }

    fn resume_session(
        &self,
        request: &CoreResumeSessionRequest,
        sink: &dyn RuntimeEventSink,
    ) -> AppResult<()> {
        PiMonoAdapter::resume_session(
            self,
            &ResumeSessionRequest {
                runtime_path: request.runtime_path.clone(),
                workspace_cwd: request.workspace_cwd.clone(),
                pimono_session_id: request.pimono_session_id.clone(),
                env: request.env.clone(),
            },
            &RuntimeEventSinkAdapter { sink },
        )
    }

    fn respond_approval(&self, request: &CoreRespondApprovalRequest) -> AppResult<()> {
        PiMonoAdapter::respond_approval(
            self,
            &RespondApprovalRequest {
                runtime_path: request.runtime_path.clone(),
                workspace_cwd: request.workspace_cwd.clone(),
                request_id: request.request_id.clone(),
                approve: request.approve,
                env: request.env.clone(),
            },
        )
    }

    fn inspect_run_status(
        &self,
        request: &CoreInspectRunStatusRequest,
    ) -> AppResult<CoreRunInspection> {
        PiMonoAdapter::inspect_run_status(
            self,
            &InspectRunStatusRequest {
                runtime_path: request.runtime_path.clone(),
                workspace_cwd: request.workspace_cwd.clone(),
                pimono_run_id: request.pimono_run_id.clone(),
                env: request.env.clone(),
            },
        )
        .map(|inspection| CoreRunInspection {
            running: inspection.running,
            terminal_event: inspection.terminal_event.map(|event| match event {
                PiMonoEvent::SessionBound { pimono_session_id } => {
                    RuntimeEvent::SessionBound { pimono_session_id }
                }
                PiMonoEvent::RunStarted {
                    pimono_run_id,
                    pimono_session_id,
                } => RuntimeEvent::RunStarted {
                    pimono_run_id,
                    pimono_session_id,
                },
                PiMonoEvent::TextDelta { text } => RuntimeEvent::TextDelta { text },
                PiMonoEvent::ApprovalRequested {
                    request_id,
                    request_type,
                    payload_json,
                } => RuntimeEvent::ApprovalRequested {
                    request_id,
                    request_type,
                    payload_json,
                },
                PiMonoEvent::RunFailed { code, message } => {
                    RuntimeEvent::RunFailed { code, message }
                }
                PiMonoEvent::RunCompleted => RuntimeEvent::RunCompleted,
            }),
        })
    }
}
