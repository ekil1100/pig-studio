use app_core::{
    InMemoryEventBus, RuntimeSettings, SettingsStorePort,
    use_cases::{
        create_project, create_session, reconcile_active_runs, respond_approval, resume_session,
        send_prompt, update_runtime_settings,
    },
};
use chrono::{DateTime, Utc};
use domain::{Approval, ApprovalDecision, Project, Run, RuntimeHealth, Session, SessionStatus};
use infra_pimono::{PiMonoAdapter, StdProcessRunner};
use infra_settings::{
    ConfigDirectoryLookupResult, ConfigDirectorySource, FsService, PlatformService, RuntimeLocator,
    RuntimeLookupResult, RuntimeSource, SettingsStore, WorktreeService,
};
use infra_sqlite::{Database, RestoredSessionView, StoredEvent};
use serde_json::Value;
use shared_kernel::{AppError, AppResult, ApprovalId, ProjectId, SessionId};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};
use ui_components::{
    ApprovalCardView, ProjectTreeItemView, SessionTreeItemView, TimelineEntryView,
    build_session_banner, runtime_health_summary, session_status_badge,
};

use crate::state::{ActiveSessionState, NoticeState, NoticeTone, WorkspaceState};

pub struct DesktopServices {
    pub database: Database,
    pub settings_store: SettingsStore,
    pub runtime_locator: RuntimeLocator,
    pub runtime_adapter: PiMonoAdapter<StdProcessRunner>,
    pub workspace_service: WorktreeService,
    pub event_bus: InMemoryEventBus,
}

impl DesktopServices {
    pub fn bootstrap() -> AppResult<Self> {
        let platform = PlatformService;
        let app_data_dir = platform.app_data_dir("pig-studio")?;
        fs::create_dir_all(&app_data_dir)
            .map_err(|error| AppError::Infrastructure(error.to_string()))?;

        Ok(Self {
            database: Database::open(app_data_dir.join("pig-studio.sqlite3"))?,
            settings_store: SettingsStore::new(app_data_dir.join("settings.json")),
            runtime_locator: RuntimeLocator::new(FsService, platform),
            runtime_adapter: PiMonoAdapter::new(StdProcessRunner),
            workspace_service: WorktreeService::new(FsService),
            event_bus: InMemoryEventBus::default(),
        })
    }

    pub fn in_memory_for_preview() -> AppResult<Self> {
        Ok(Self {
            database: Database::in_memory()?,
            settings_store: SettingsStore::new(PathBuf::from(
                "/tmp/pig-studio-preview-settings.json",
            )),
            runtime_locator: RuntimeLocator::new(FsService, PlatformService),
            runtime_adapter: PiMonoAdapter::new(StdProcessRunner),
            workspace_service: WorktreeService::new(FsService),
            event_bus: InMemoryEventBus::default(),
        })
    }

    pub fn resolved_runtime_context(&self) -> ResolvedRuntimeContext {
        let settings = self.settings_store.load().unwrap_or_default();
        let env = std::env::vars().collect::<BTreeMap<_, _>>();
        let runtime_lookup = self.runtime_locator.locate(&settings, &env);
        let config_lookup = self
            .runtime_locator
            .locate_config_directory(&settings, &env);
        ResolvedRuntimeContext {
            settings: settings.into(),
            runtime_lookup,
            config_lookup,
        }
    }

    pub fn runtime_lookup_result(&self) -> RuntimeLookupResult {
        self.resolved_runtime_context().runtime_lookup
    }

    fn runtime_settings(&self) -> AppResult<RuntimeSettings> {
        self.settings_store.load().map(Into::into)
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedRuntimeContext {
    pub settings: RuntimeSettings,
    pub runtime_lookup: RuntimeLookupResult,
    pub config_lookup: ConfigDirectoryLookupResult,
}

impl ResolvedRuntimeContext {
    pub fn effective_environment(&self) -> BTreeMap<String, String> {
        let mut env = self.settings.environment.clone();
        if let Some(config_dir) = self.config_lookup.resolved_path.as_ref() {
            let config_dir = config_dir.to_string_lossy().into_owned();
            env.entry("PI_CODING_AGENT_DIR".into())
                .or_insert_with(|| config_dir.clone());
            env.entry("PI_CONFIG_DIR".into())
                .or_insert_with(|| config_dir.clone());
            env.entry("PI_MONO_CONFIG_DIR".into()).or_insert(config_dir);
        }
        env
    }

    pub fn has_runtime_override(&self) -> bool {
        self.settings.runtime_path.is_some()
    }

    pub fn has_config_dir_override(&self) -> bool {
        self.settings.config_dir.is_some()
    }
}

#[derive(Clone)]
struct StaticSettingsStore {
    settings: RuntimeSettings,
}

impl SettingsStorePort for StaticSettingsStore {
    fn load(&self) -> AppResult<RuntimeSettings> {
        Ok(self.settings.clone())
    }

    fn save(&self, _settings: &RuntimeSettings) -> AppResult<()> {
        Ok(())
    }
}

pub struct DesktopModel {
    services: DesktopServices,
    selected_project_id: Option<ProjectId>,
    selected_session_id: Option<SessionId>,
    settings_open: bool,
    prefer_worktree: bool,
    notice: Option<NoticeState>,
    workspace: WorkspaceState,
}

impl DesktopModel {
    pub fn bootstrap_or_preview() -> Self {
        match Self::try_bootstrap() {
            Ok(model) => model,
            Err(error) => {
                let mut model =
                    match DesktopServices::in_memory_for_preview().and_then(Self::from_services) {
                        Ok(model) => model,
                        Err(fallback_error) => Self::preview_with_notice(format!(
                            "启动失败，且预览模式不可用：{fallback_error}"
                        )),
                    };
                model.notice = Some(NoticeState {
                    tone: NoticeTone::Warning,
                    message: format!("桌面数据目录初始化失败，已回退到预览模式：{error}"),
                });
                model.reload();
                model
            }
        }
    }

    pub fn workspace(&self) -> &WorkspaceState {
        &self.workspace
    }

    pub fn refresh(&mut self) {
        self.reload();
    }

    pub fn select_project(&mut self, project_id: impl Into<String>) {
        self.selected_project_id = Some(ProjectId::from_string(project_id));
        self.selected_session_id = None;
        self.clear_notice();
        self.reload();
    }

    pub fn select_session(&mut self, session_id: impl Into<String>) {
        let session_id = SessionId::from_string(session_id);
        self.selected_session_id = Some(session_id.clone());
        if let Ok(Some(session)) = self
            .services
            .database
            .session_repository()
            .find(&session_id)
        {
            self.selected_project_id = Some(session.project_id);
        }
        self.clear_notice();
        if let Err(error) = self.restore_bound_session(&session_id) {
            self.set_notice(NoticeTone::Warning, error.to_string());
        }
        self.reload();
    }

    pub fn open_settings(&mut self) {
        self.settings_open = true;
        self.clear_notice();
        self.reload();
    }

    pub fn close_settings(&mut self) {
        self.settings_open = false;
        self.reload();
    }

    pub fn create_project(&mut self, name: &str, root_path: &str) -> bool {
        let root_path = root_path.trim();
        if root_path.is_empty() {
            self.set_notice(NoticeTone::Error, "请先选择项目目录。".into());
            self.reload();
            return false;
        }

        let root_path_buf = PathBuf::from(root_path);
        let name = if name.trim().is_empty() {
            infer_project_name(&root_path_buf)
        } else {
            name.trim().to_owned()
        };

        let projects = self.services.database.project_repository();
        match create_project::execute(
            &projects,
            &self.services.workspace_service,
            &self.services.event_bus,
            create_project::CreateProjectInput {
                name,
                root_path: root_path_buf,
            },
            Utc::now(),
        ) {
            Ok(project) => {
                self.selected_project_id = Some(project.id.clone());
                self.selected_session_id = None;
                self.set_notice(NoticeTone::Success, format!("已添加项目：{}", project.name));
                self.reload();
                true
            }
            Err(error) => {
                self.set_notice(NoticeTone::Error, error.to_string());
                self.reload();
                false
            }
        }
    }

    pub fn create_session(&mut self, title: &str) -> bool {
        let title = if title.trim().is_empty() {
            "新会话".to_owned()
        } else {
            title.trim().to_owned()
        };

        let Some(project_id) = self.selected_project_id.clone() else {
            self.set_notice(NoticeTone::Error, "请先选择一个项目。".into());
            self.reload();
            return false;
        };

        let project_repository = self.services.database.project_repository();
        let Some(project) = project_repository.list().ok().and_then(|projects| {
            projects
                .into_iter()
                .find(|project| project.id == project_id)
        }) else {
            self.set_notice(NoticeTone::Error, "当前项目不存在或已被移除。".into());
            self.reload();
            return false;
        };

        let sessions = self.services.database.session_repository();
        match create_session::execute(
            &sessions,
            &self.services.workspace_service,
            &self.services.event_bus,
            create_session::CreateSessionInput {
                project: project.clone(),
                title,
                prefer_worktree: self.prefer_worktree,
            },
            Utc::now(),
        ) {
            Ok(session) => {
                self.selected_project_id = Some(project.id.clone());
                self.selected_session_id = Some(session.id.clone());
                let _ = project_repository.mark_opened(&project.id, Utc::now());
                self.set_notice(
                    NoticeTone::Success,
                    format!("已创建会话：{}", session.title),
                );
                self.reload();
                true
            }
            Err(error) => {
                self.set_notice(NoticeTone::Error, error.to_string());
                self.reload();
                false
            }
        }
    }

    pub fn rename_active_session(&mut self, title: &str) -> bool {
        let title = title.trim();
        if title.is_empty() {
            self.set_notice(NoticeTone::Error, "请输入新的会话标题。".into());
            self.reload();
            return false;
        }

        let Some(session_id) = self.selected_session_id.clone() else {
            self.set_notice(NoticeTone::Error, "当前没有选中的会话。".into());
            self.reload();
            return false;
        };

        let sessions = self.services.database.session_repository();
        match sessions.rename(&session_id, title, Utc::now()) {
            Ok(()) => {
                self.set_notice(NoticeTone::Success, "会话标题已更新。".into());
                self.reload();
                true
            }
            Err(error) => {
                self.set_notice(NoticeTone::Error, error.to_string());
                self.reload();
                false
            }
        }
    }

    pub fn delete_active_session(&mut self) -> bool {
        let Some(session_id) = self.selected_session_id.clone() else {
            self.set_notice(NoticeTone::Error, "当前没有可删除的会话。".into());
            self.reload();
            return false;
        };

        let sessions = self.services.database.session_repository();
        match sessions.soft_delete(&session_id, Utc::now()) {
            Ok(()) => {
                self.selected_session_id = None;
                self.set_notice(NoticeTone::Warning, "会话已移入历史记录。".into());
                self.reload();
                true
            }
            Err(error) => {
                self.set_notice(NoticeTone::Error, error.to_string());
                self.reload();
                false
            }
        }
    }

    pub fn create_followup_session_from_active(&mut self) -> bool {
        let Some(session_id) = self.selected_session_id.clone() else {
            self.set_notice(NoticeTone::Error, "当前没有可延续的历史会话。".into());
            self.reload();
            return false;
        };

        let sessions = self.services.database.session_repository();
        let Some(session) = sessions.find(&session_id).unwrap_or(None) else {
            self.set_notice(NoticeTone::Error, "当前会话不存在。".into());
            self.reload();
            return false;
        };
        self.selected_project_id = Some(session.project_id.clone());
        let next_title = format!("{} · 新会话", session.title);
        self.create_session(&next_title)
    }

    pub fn save_runtime_settings(&mut self, runtime_path: &str) -> bool {
        let runtime_path = runtime_path.trim();
        let mut settings = match self.services.runtime_settings() {
            Ok(settings) => settings,
            Err(error) => {
                self.set_notice(NoticeTone::Error, error.to_string());
                self.reload();
                return false;
            }
        };
        settings.runtime_path = if runtime_path.is_empty() {
            None
        } else {
            Some(PathBuf::from(runtime_path))
        };
        self.persist_settings(settings, "运行时设置已保存。")
    }

    pub fn set_runtime_binary_override(&mut self, runtime_path: PathBuf) -> bool {
        self.persist_runtime_overrides(Some(runtime_path), None, "已保存自定义运行时二进制。")
    }

    pub fn set_config_dir_override(&mut self, config_dir: PathBuf) -> bool {
        self.persist_runtime_overrides(None, Some(config_dir), "已保存自定义配置目录。")
    }

    pub fn clear_runtime_overrides(&mut self) -> bool {
        let mut settings = match self.services.runtime_settings() {
            Ok(settings) => settings,
            Err(error) => {
                self.set_notice(NoticeTone::Error, error.to_string());
                self.reload();
                return false;
            }
        };
        settings.runtime_path = None;
        settings.config_dir = None;
        self.persist_settings(settings, "已恢复自动检测。")
    }

    pub fn refresh_runtime_detection(&mut self) {
        let context = self.services.resolved_runtime_context();
        self.set_notice(
            if context.runtime_lookup.health.available {
                NoticeTone::Success
            } else {
                NoticeTone::Warning
            },
            if let Some(path) = context.runtime_lookup.resolved_path {
                format!("已重新检测到 Pi 运行时：{}", path.display())
            } else {
                context
                    .runtime_lookup
                    .health
                    .reason
                    .clone()
                    .unwrap_or_else(|| "暂未检测到可用的 Pi 运行时。".into())
            },
        );
        self.reload();
    }

    fn persist_runtime_overrides(
        &mut self,
        runtime_path: Option<PathBuf>,
        config_dir: Option<PathBuf>,
        success_message: &str,
    ) -> bool {
        let mut settings = match self.services.runtime_settings() {
            Ok(settings) => settings,
            Err(error) => {
                self.set_notice(NoticeTone::Error, error.to_string());
                self.reload();
                return false;
            }
        };
        if let Some(runtime_path) = runtime_path {
            settings.runtime_path = Some(runtime_path);
        }
        if let Some(config_dir) = config_dir {
            settings.config_dir = Some(config_dir);
        }
        self.persist_settings(settings, success_message)
    }

    fn persist_settings(&mut self, settings: RuntimeSettings, success_message: &str) -> bool {
        let now = Utc::now();
        match update_runtime_settings::execute(
            &self.services.settings_store,
            &self.services.event_bus,
            settings,
            now,
        ) {
            Ok(_) => {
                let runtime_lookup = self.services.runtime_lookup_result();
                if runtime_lookup.health.available
                    && let Some(session_id) = self.selected_session_id.clone()
                {
                    let sessions = self.services.database.session_repository();
                    if let Ok(Some(session)) = sessions.find(&session_id)
                        && session.status == SessionStatus::Blocked
                    {
                        let _ = sessions.update_status(&session.id, SessionStatus::Idle, now);
                    }
                }
                self.set_notice(NoticeTone::Success, success_message.into());
                self.reload();
                true
            }
            Err(error) => {
                self.set_notice(NoticeTone::Error, error.to_string());
                self.reload();
                false
            }
        }
    }

    pub fn send_prompt(&mut self, prompt: &str) -> bool {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            self.set_notice(NoticeTone::Error, "请输入 prompt。".into());
            self.reload();
            return false;
        }

        let Some(session_id) = self.selected_session_id.clone() else {
            self.set_notice(NoticeTone::Error, "请先选择或创建一个会话。".into());
            self.reload();
            return false;
        };

        let sessions = self.services.database.session_repository();
        let Some(mut session) = sessions.find(&session_id).unwrap_or(None) else {
            self.set_notice(NoticeTone::Error, "当前会话不存在。".into());
            self.reload();
            return false;
        };

        if session.last_run_at.is_none() && session.title == "新会话" {
            let generated_title = suggest_session_title_from_prompt(prompt);
            if let Err(error) = sessions.rename(&session.id, &generated_title, Utc::now()) {
                self.set_notice(NoticeTone::Error, error.to_string());
                self.reload();
                return false;
            }
            session.title = generated_title;
        }

        let runtime_context = self.services.resolved_runtime_context();
        let Some(runtime_path) = runtime_context.runtime_lookup.resolved_path.clone() else {
            let _ = sessions.update_status(&session.id, SessionStatus::Blocked, Utc::now());
            self.set_notice(
                NoticeTone::Warning,
                runtime_context
                    .runtime_lookup
                    .health
                    .reason
                    .clone()
                    .unwrap_or_else(|| "未找到可用的 Pi 运行时。".into()),
            );
            self.reload();
            return false;
        };

        let now = Utc::now();
        let runtime_session_id = session
            .pimono_session_id
            .clone()
            .unwrap_or_else(|| session.id.as_str().to_owned());
        if session.pimono_session_id.as_deref() != Some(runtime_session_id.as_str()) {
            if let Err(error) = sessions.bind_runtime_session(&session.id, &runtime_session_id, now)
            {
                self.set_notice(NoticeTone::Error, error.to_string());
                self.reload();
                return false;
            }
            session.pimono_session_id = Some(runtime_session_id.clone());
        }
        if let Err(error) = sessions.mark_run_started(&session.id, now) {
            self.set_notice(NoticeTone::Error, error.to_string());
            self.reload();
            return false;
        }
        session.last_run_at = Some(now);
        session.updated_at = now;

        let environment = self
            .runtime_environment_for_session(&session.id, runtime_context.effective_environment());
        if let Some(db_path) = self.worker_db_path() {
            self.spawn_send_prompt_worker(
                db_path,
                session.clone(),
                runtime_path.clone(),
                prompt.into(),
                environment,
            );
            self.set_notice(
                NoticeTone::Success,
                "已发送 prompt，正在实时接收输出。".into(),
            );
            self.reload();
            return true;
        }

        let runs = self.services.database.run_repository();
        let approvals = self.services.database.approval_repository();
        let events = self.services.database.event_repository();
        match send_prompt::execute(
            &runs,
            &sessions,
            &approvals,
            &events,
            &self.services.runtime_adapter,
            &self.services.event_bus,
            send_prompt::SendPromptInput {
                session: session.clone(),
                request: app_core::StartSessionRunRequest {
                    runtime_path,
                    workspace_cwd: session.workspace_cwd.clone(),
                    pimono_session_id: session.pimono_session_id.clone(),
                    prompt: prompt.into(),
                    env: self.runtime_environment_for_session(
                        &session.id,
                        runtime_context.effective_environment(),
                    ),
                },
            },
            now,
        ) {
            Ok(_) => {
                self.set_notice(NoticeTone::Success, "已发送 prompt。".into());
                self.reload();
                true
            }
            Err(error) => {
                self.set_notice(NoticeTone::Error, error.to_string());
                self.reload();
                false
            }
        }
    }

    pub fn respond_to_approval(&mut self, approval_id: &str, approve: bool) -> bool {
        let Some(session_id) = self.selected_session_id.clone() else {
            self.set_notice(NoticeTone::Error, "当前没有选中的会话。".into());
            self.reload();
            return false;
        };

        let sessions = self.services.database.session_repository();
        let Some(session) = sessions.find(&session_id).unwrap_or(None) else {
            self.set_notice(NoticeTone::Error, "当前会话不存在。".into());
            self.reload();
            return false;
        };

        let runtime_context = self.services.resolved_runtime_context();
        let Some(runtime_path) = runtime_context.runtime_lookup.resolved_path.clone() else {
            self.set_notice(
                NoticeTone::Warning,
                "当前运行时不可用，无法提交审批决策。".into(),
            );
            self.reload();
            return false;
        };

        let approval_repository = self.services.database.approval_repository();
        let approval = match approval_repository
            .list_pending_by_session(&session_id)
            .and_then(|approvals| {
                approvals
                    .into_iter()
                    .find(|approval| approval.id.as_str() == approval_id)
                    .ok_or_else(|| AppError::NotFound(format!("approval not found: {approval_id}")))
            }) {
            Ok(approval) => approval,
            Err(error) => {
                self.set_notice(NoticeTone::Error, error.to_string());
                self.reload();
                return false;
            }
        };

        let decision = if approve {
            ApprovalDecision::Approve
        } else {
            ApprovalDecision::Reject
        };

        let events = self.services.database.event_repository();
        match respond_approval::execute(
            &approval_repository,
            &self.services.runtime_adapter,
            &events,
            &self.services.event_bus,
            respond_approval::RespondApprovalInput {
                approval_id: ApprovalId::from_string(approval_id),
                session_id: session.id.clone(),
                run_id: approval.run_id.clone(),
                request: app_core::RespondApprovalRequest {
                    runtime_path,
                    workspace_cwd: session.workspace_cwd.clone(),
                    request_id: approval.request.request_id.clone(),
                    approve,
                    env: self.runtime_environment_for_session(
                        &session.id,
                        runtime_context.effective_environment(),
                    ),
                },
                decision,
            },
            Utc::now(),
        ) {
            Ok(()) => {
                let _ = sessions.update_status(&session.id, SessionStatus::Running, Utc::now());
                let resumed = match self.restore_bound_session(&session.id) {
                    Ok(resumed) => resumed,
                    Err(error) => {
                        self.set_notice(NoticeTone::Warning, error.to_string());
                        self.reload();
                        return true;
                    }
                };
                self.set_notice(
                    NoticeTone::Success,
                    if approve {
                        if resumed {
                            "审批已批准，并已继续附着当前运行。"
                        } else {
                            "审批已批准。"
                        }
                    } else if resumed {
                        "审批已拒绝，并已刷新当前运行状态。"
                    } else {
                        "审批已拒绝。"
                    }
                    .into(),
                );
                self.reload();
                true
            }
            Err(error) => {
                self.set_notice(NoticeTone::Error, error.to_string());
                self.reload();
                false
            }
        }
    }

    pub fn resume_selected_session(&mut self) -> bool {
        let Some(session_id) = self.selected_session_id.clone() else {
            self.set_notice(NoticeTone::Error, "当前没有可恢复的会话。".into());
            self.reload();
            return false;
        };

        match self.restore_bound_session(&session_id) {
            Ok(true) => {
                self.set_notice(NoticeTone::Success, "已重新附着到当前会话运行。".into());
                self.reload();
                true
            }
            Ok(false) => {
                self.set_notice(NoticeTone::Info, "当前会话没有可恢复的远端绑定。".into());
                self.reload();
                false
            }
            Err(error) => {
                self.set_notice(NoticeTone::Warning, error.to_string());
                self.reload();
                false
            }
        }
    }

    fn restore_bound_session(&mut self, session_id: &SessionId) -> AppResult<bool> {
        let sessions = self.services.database.session_repository();

        let Some(session) = sessions.find(session_id)? else {
            return Err(AppError::NotFound(format!(
                "session not found: {session_id}"
            )));
        };
        if !matches!(
            session.status,
            SessionStatus::Running | SessionStatus::Interrupted
        ) {
            return Ok(false);
        }

        let Some(pimono_session_id) = session.pimono_session_id.clone() else {
            return Ok(false);
        };

        let runtime_context = self.services.resolved_runtime_context();
        let Some(runtime_path) = runtime_context.runtime_lookup.resolved_path.clone() else {
            if matches!(
                session.status,
                SessionStatus::Running | SessionStatus::Interrupted
            ) {
                sessions.update_status(&session.id, SessionStatus::Blocked, Utc::now())?;
            }
            return Err(AppError::Validation(
                runtime_context
                    .runtime_lookup
                    .health
                    .reason
                    .unwrap_or_else(|| "当前运行时不可用，无法恢复会话绑定。".into()),
            ));
        };

        let Some(active_run) = self.find_latest_active_run(&session.id)? else {
            return Ok(false);
        };

        let request = app_core::ResumeSessionRequest {
            runtime_path,
            workspace_cwd: session.workspace_cwd.clone(),
            pimono_session_id,
            env: self.runtime_environment_for_session(
                &session.id,
                runtime_context.effective_environment(),
            ),
        };

        if let Some(db_path) = self.worker_db_path() {
            self.spawn_resume_worker(db_path, session, active_run, request);
            return Ok(true);
        }

        let runs = self.services.database.run_repository();
        let approvals = self.services.database.approval_repository();
        let events = self.services.database.event_repository();
        resume_session::execute(
            &runs,
            &sessions,
            &approvals,
            &events,
            &self.services.runtime_adapter,
            &self.services.event_bus,
            resume_session::ResumeSessionInput {
                session,
                active_run,
                request,
            },
            Utc::now(),
        )?;

        Ok(true)
    }

    fn worker_db_path(&self) -> Option<PathBuf> {
        self.services.database.path().map(PathBuf::from)
    }

    fn runtime_environment_for_session(
        &self,
        session_id: &SessionId,
        mut env: BTreeMap<String, String>,
    ) -> BTreeMap<String, String> {
        if let Some(database_path) = self.services.database.path() {
            if let Some(app_data_dir) = PathBuf::from(database_path).parent().map(PathBuf::from) {
                let runtime_session_dir =
                    app_data_dir.join("pi-sessions").join(session_id.as_str());
                env.insert(
                    "PIG_STUDIO_PI_SESSION_DIR".into(),
                    runtime_session_dir.to_string_lossy().into_owned(),
                );
            }
        }
        env
    }

    fn spawn_send_prompt_worker(
        &self,
        db_path: PathBuf,
        session: Session,
        runtime_path: PathBuf,
        prompt: String,
        env: BTreeMap<String, String>,
    ) {
        std::thread::spawn(move || {
            let task = || -> AppResult<()> {
                let database = Database::open(&db_path)?;
                let sessions = database.session_repository();
                let runs = database.run_repository();
                let approvals = database.approval_repository();
                let events = database.event_repository();
                let adapter = PiMonoAdapter::new(StdProcessRunner);
                let bus = InMemoryEventBus::default();

                send_prompt::execute(
                    &runs,
                    &sessions,
                    &approvals,
                    &events,
                    &adapter,
                    &bus,
                    send_prompt::SendPromptInput {
                        session: session.clone(),
                        request: app_core::StartSessionRunRequest {
                            runtime_path,
                            workspace_cwd: session.workspace_cwd.clone(),
                            pimono_session_id: session.pimono_session_id.clone(),
                            prompt,
                            env,
                        },
                    },
                    Utc::now(),
                )
                .map(|_| ())
            };

            if let Err(error) = task() {
                tracing::error!("background send_prompt worker failed: {error}");
            }
        });
    }

    fn spawn_resume_worker(
        &self,
        db_path: PathBuf,
        session: Session,
        active_run: Run,
        request: app_core::ResumeSessionRequest,
    ) {
        std::thread::spawn(move || {
            let task = || -> AppResult<()> {
                let database = Database::open(&db_path)?;
                let sessions = database.session_repository();
                let runs = database.run_repository();
                let approvals = database.approval_repository();
                let events = database.event_repository();
                let adapter = PiMonoAdapter::new(StdProcessRunner);
                let bus = InMemoryEventBus::default();

                resume_session::execute(
                    &runs,
                    &sessions,
                    &approvals,
                    &events,
                    &adapter,
                    &bus,
                    resume_session::ResumeSessionInput {
                        session,
                        active_run,
                        request,
                    },
                    Utc::now(),
                )
            };

            if let Err(error) = task() {
                tracing::error!("background resume_session worker failed: {error}");
            }
        });
    }

    fn find_latest_active_run(&self, session_id: &SessionId) -> AppResult<Option<Run>> {
        let runs = self
            .services
            .database
            .run_repository()
            .list_by_session(session_id)?;
        let latest_non_terminal = runs
            .iter()
            .rev()
            .find(|run| !run.status.is_terminal())
            .cloned();
        Ok(latest_non_terminal.or_else(|| runs.into_iter().last()))
    }

    fn try_bootstrap() -> AppResult<Self> {
        let services = DesktopServices::bootstrap()?;
        Self::from_services(services)
    }

    fn from_services(services: DesktopServices) -> AppResult<Self> {
        let mut model = Self {
            services,
            selected_project_id: None,
            selected_session_id: None,
            settings_open: false,
            prefer_worktree: true,
            notice: None,
            workspace: WorkspaceState::default(),
        };
        model.reconcile_active_runs();
        model.reload();
        if let Some(session_id) = model.selected_session_id.clone()
            && let Err(error) = model.restore_bound_session(&session_id)
        {
            model.notice = Some(NoticeState {
                tone: NoticeTone::Warning,
                message: error.to_string(),
            });
            model.reload();
        } else if model.selected_session_id.is_some() {
            model.reload();
        }
        Ok(model)
    }

    fn preview_with_notice(message: String) -> Self {
        Self {
            services: DesktopServices::in_memory_for_preview()
                .expect("preview services should always be available"),
            selected_project_id: None,
            selected_session_id: None,
            settings_open: false,
            prefer_worktree: true,
            notice: Some(NoticeState {
                tone: NoticeTone::Warning,
                message,
            }),
            workspace: WorkspaceState::default(),
        }
    }

    fn reconcile_active_runs(&mut self) {
        let runs = self.services.database.run_repository();
        let sessions = self.services.database.session_repository();
        let events = self.services.database.event_repository();
        let context = self.services.resolved_runtime_context();
        let settings = StaticSettingsStore {
            settings: RuntimeSettings {
                runtime_path: context.runtime_lookup.resolved_path.clone(),
                config_dir: context.config_lookup.resolved_path.clone(),
                environment: context.effective_environment(),
                last_checked_at: context.settings.last_checked_at,
            },
        };
        if let Err(error) = reconcile_active_runs::execute(
            &runs,
            &sessions,
            &events,
            &self.services.runtime_adapter,
            &settings,
            &self.services.event_bus,
            Utc::now(),
        ) {
            self.notice = Some(NoticeState {
                tone: NoticeTone::Warning,
                message: format!("启动时对账活跃运行失败：{error}"),
            });
        }
    }

    fn reload(&mut self) {
        match self.build_workspace() {
            Ok(workspace) => self.workspace = workspace,
            Err(error) => {
                self.workspace = WorkspaceState {
                    settings_open: self.settings_open,
                    notice: Some(NoticeState {
                        tone: NoticeTone::Error,
                        message: error.to_string(),
                    }),
                    ..WorkspaceState::default()
                };
            }
        }
    }

    fn build_workspace(&mut self) -> AppResult<WorkspaceState> {
        let project_repository = self.services.database.project_repository();
        let session_repository = self.services.database.session_repository();
        let event_repository = self.services.database.event_repository();
        let projects = project_repository.list()?;
        let runtime_context = self.services.resolved_runtime_context();

        let mut latest_session: Option<Session> = None;
        let mut project_views = Vec::new();
        for project in &projects {
            let sessions = session_repository.list_by_project(&project.id)?;
            for session in &sessions {
                let should_replace = latest_session
                    .as_ref()
                    .map(|current| session.updated_at > current.updated_at)
                    .unwrap_or(true);
                if should_replace {
                    latest_session = Some(session.clone());
                }
            }

            project_views.push(ProjectTreeItemView {
                project_id: project.id.as_str().to_owned(),
                project_name: project.name.clone(),
                sessions: sessions
                    .into_iter()
                    .map(|session| SessionTreeItemView {
                        session_id: session.id.as_str().to_owned(),
                        title: session.title,
                        badge: session_status_badge(session.status),
                    })
                    .collect(),
            });
        }

        if self.selected_project_id.is_none() {
            self.selected_project_id = latest_session
                .as_ref()
                .map(|session| session.project_id.clone())
                .or_else(|| projects.first().map(|project| project.id.clone()));
        }
        if let Some(project_id) = self.selected_project_id.clone() {
            if !projects.iter().any(|project| project.id == project_id) {
                self.selected_project_id = latest_session
                    .as_ref()
                    .map(|session| session.project_id.clone())
                    .or_else(|| projects.first().map(|project| project.id.clone()));
            }
        }

        if self.selected_session_id.is_none() {
            self.selected_session_id = latest_session.as_ref().map(|session| session.id.clone());
        }
        if let Some(session_id) = self.selected_session_id.clone() {
            if session_repository.find(&session_id)?.is_none() {
                self.selected_session_id =
                    latest_session.as_ref().map(|session| session.id.clone());
            }
        }

        let active_session = self
            .selected_session_id
            .clone()
            .map(|session_id| {
                self.build_active_session(
                    &projects,
                    &event_repository,
                    &session_repository,
                    session_id,
                )
            })
            .transpose()?;

        if let Some(active_session) = &active_session {
            self.selected_project_id =
                Some(ProjectId::from_string(active_session.project_id.clone()));
        }

        Ok(WorkspaceState {
            projects: project_views,
            active_project_id: self
                .selected_project_id
                .as_ref()
                .map(|project_id| project_id.as_str().to_owned()),
            active_session_id: self
                .selected_session_id
                .as_ref()
                .map(|session_id| session_id.as_str().to_owned()),
            active_session,
            settings_open: self.settings_open,
            runtime_path: runtime_context
                .runtime_lookup
                .resolved_path
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default(),
            runtime_source_label: runtime_source_label(
                runtime_context.runtime_lookup.source.as_ref(),
            )
            .into(),
            config_dir: runtime_context
                .config_lookup
                .resolved_path
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default(),
            config_dir_source_label: config_dir_source_label(
                runtime_context.config_lookup.source.as_ref(),
            )
            .into(),
            has_runtime_override: runtime_context.has_runtime_override(),
            has_config_dir_override: runtime_context.has_config_dir_override(),
            runtime_health: runtime_context.runtime_lookup.health,
            notice: self.notice.clone(),
        })
    }

    fn build_active_session(
        &self,
        projects: &[Project],
        event_repository: &infra_sqlite::EventRepository,
        session_repository: &infra_sqlite::SessionRepository,
        session_id: SessionId,
    ) -> AppResult<ActiveSessionState> {
        let restored = session_repository
            .restore_view(&session_id)?
            .ok_or_else(|| AppError::NotFound(format!("session not found: {}", session_id)))?;
        let project = projects
            .iter()
            .find(|project| project.id == restored.session.project_id)
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "project not found for session: {}",
                    restored.session.project_id
                ))
            })?;
        let timeline = build_timeline(
            &restored,
            event_repository.list_by_session(&restored.session.id)?,
            &self.services.runtime_lookup_result().health,
        )?;

        Ok(ActiveSessionState {
            project_id: project.id.as_str().to_owned(),
            project_name: project.name.clone(),
            session_id: restored.session.id.as_str().to_owned(),
            session_name: restored.session.title.clone(),
            status_badge: session_status_badge(restored.session.status),
            banner: build_session_banner(restored.session.status),
            timeline,
            approvals: restored
                .pending_approvals
                .into_iter()
                .map(|approval| ApprovalCardView {
                    approval_id: approval.id.as_str().to_owned(),
                    title: approval.request.request_type,
                    summary: approval.request.request_payload_json,
                    request_id: approval.request.request_id,
                })
                .collect(),
        })
    }

    fn set_notice(&mut self, tone: NoticeTone, message: String) {
        self.notice = Some(NoticeState { tone, message });
    }

    fn clear_notice(&mut self) {
        self.notice = None;
    }
}

#[derive(Clone, Debug)]
pub struct StartupSnapshot {
    pub projects: Vec<Project>,
    pub sessions: Vec<Session>,
    pub pending_approvals: Vec<Approval>,
    pub runtime_health: RuntimeHealth,
    pub active_session_id: Option<SessionId>,
    pub unrecoverable_running_sessions: BTreeSet<SessionId>,
}

impl Default for StartupSnapshot {
    fn default() -> Self {
        Self {
            projects: Vec::new(),
            sessions: Vec::new(),
            pending_approvals: Vec::new(),
            runtime_health: RuntimeHealth {
                available: false,
                version: None,
                reason: Some("runtime not checked".into()),
                checked_at: None,
            },
            active_session_id: None,
            unrecoverable_running_sessions: BTreeSet::new(),
        }
    }
}

pub fn recover_workspace(snapshot: StartupSnapshot) -> WorkspaceState {
    let mut projects = Vec::new();
    for project in &snapshot.projects {
        let project_sessions = snapshot
            .sessions
            .iter()
            .filter(|session| session.project_id == project.id)
            .map(|session| {
                let effective_status =
                    effective_status(session, &snapshot.unrecoverable_running_sessions);
                SessionTreeItemView {
                    session_id: session.id.as_str().to_owned(),
                    title: session.title.clone(),
                    badge: session_status_badge(effective_status),
                }
            })
            .collect();
        projects.push(ProjectTreeItemView {
            project_id: project.id.as_str().to_owned(),
            project_name: project.name.clone(),
            sessions: project_sessions,
        });
    }

    let active_session = snapshot
        .active_session_id
        .as_ref()
        .and_then(|active_session_id| {
            let session = snapshot
                .sessions
                .iter()
                .find(|session| &session.id == active_session_id)?;
            let project = snapshot
                .projects
                .iter()
                .find(|project| project.id == session.project_id)?;
            let effective_status =
                effective_status(session, &snapshot.unrecoverable_running_sessions);
            let banner = build_session_banner(effective_status);
            let approvals = snapshot
                .pending_approvals
                .iter()
                .map(|approval| ApprovalCardView {
                    approval_id: approval.id.as_str().to_owned(),
                    title: approval.request.request_type.clone(),
                    summary: approval.request.request_payload_json.clone(),
                    request_id: approval.request.request_id.clone(),
                })
                .collect::<Vec<_>>();
            let (runtime_summary, _) = runtime_health_summary(&snapshot.runtime_health);

            Some(ActiveSessionState {
                project_id: project.id.as_str().to_owned(),
                project_name: project.name.clone(),
                session_id: session.id.as_str().to_owned(),
                session_name: session.title.clone(),
                status_badge: session_status_badge(effective_status),
                banner,
                timeline: vec![TimelineEntryView {
                    title: "恢复工作区".into(),
                    body: format!("启动时已恢复 {} 的会话上下文。", session.title),
                    meta: runtime_summary.into(),
                    tone_class: "divider",
                }],
                approvals,
            })
        });

    WorkspaceState {
        projects,
        active_project_id: active_session
            .as_ref()
            .map(|session| session.project_id.clone()),
        active_session_id: active_session
            .as_ref()
            .map(|session| session.session_id.clone()),
        active_session,
        settings_open: false,
        runtime_path: String::new(),
        runtime_source_label: "未检测到".into(),
        config_dir: String::new(),
        config_dir_source_label: "未检测到".into(),
        has_runtime_override: false,
        has_config_dir_override: false,
        runtime_health: snapshot.runtime_health,
        notice: None,
    }
}

fn build_timeline(
    restored: &RestoredSessionView,
    events: Vec<StoredEvent>,
    runtime_health: &RuntimeHealth,
) -> AppResult<Vec<TimelineEntryView>> {
    if events.is_empty() {
        let (runtime_summary, _) = runtime_health_summary(runtime_health);
        return Ok(vec![TimelineEntryView {
            title: "会话已创建".into(),
            body: format!(
                "{} 已准备就绪，可以继续发送 prompt。",
                restored.session.title
            ),
            meta: runtime_summary.into(),
            tone_class: "divider",
        }]);
    }

    events.into_iter().map(event_to_timeline_entry).collect()
}

fn event_to_timeline_entry(event: StoredEvent) -> AppResult<TimelineEntryView> {
    let meta = format_timestamp(event.created_at);
    match event.event_type.as_str() {
        "session_bound" => {
            let pimono_session_id = json_string_field(&event.payload_json, "pimono_session_id")?
                .unwrap_or_else(|| "unknown-session".into());
            Ok(TimelineEntryView {
                title: "会话已绑定".into(),
                body: format!("已绑定远端会话：{pimono_session_id}"),
                meta,
                tone_class: "divider divider-accent",
            })
        }
        "run_started" => {
            let pimono_run_id = json_string_field(&event.payload_json, "pimono_run_id")?
                .unwrap_or_else(|| "unknown-run".into());
            Ok(TimelineEntryView {
                title: "运行已启动".into(),
                body: format!("远端运行 ID：{pimono_run_id}"),
                meta,
                tone_class: "divider divider-info",
            })
        }
        "text_delta" => {
            let text = json_string_field(&event.payload_json, "text")?
                .unwrap_or_else(|| event.payload_json.clone());
            Ok(TimelineEntryView {
                title: "输出片段".into(),
                body: text,
                meta,
                tone_class: "divider",
            })
        }
        "approval_requested" => {
            let request_type = json_string_field(&event.payload_json, "request_type")?
                .unwrap_or_else(|| "approval".into());
            let payload_json = json_string_field(&event.payload_json, "payload_json")?
                .unwrap_or_else(|| event.payload_json.clone());
            Ok(TimelineEntryView {
                title: format!("审批请求 · {request_type}"),
                body: payload_json,
                meta,
                tone_class: "divider divider-warning",
            })
        }
        "approval_decision" => {
            let request_id = json_string_field(&event.payload_json, "request_id")?
                .unwrap_or_else(|| "unknown-request".into());
            let decision = json_string_field(&event.payload_json, "decision")?
                .unwrap_or_else(|| "unknown".into());
            let body = match decision.as_str() {
                "approve" => format!("已批准审批请求 {request_id}。"),
                "reject" => format!("已拒绝审批请求 {request_id}。"),
                _ => format!("审批请求 {request_id} 已记录决策：{decision}。"),
            };
            Ok(TimelineEntryView {
                title: "审批决策".into(),
                body,
                meta,
                tone_class: "divider divider-accent",
            })
        }
        "run_completed" => Ok(TimelineEntryView {
            title: "运行完成".into(),
            body: "代理已完成当前运行。".into(),
            meta,
            tone_class: "divider divider-success",
        }),
        "run_failed" => {
            let code = json_string_field(&event.payload_json, "code")?;
            let message = json_string_field(&event.payload_json, "message")?
                .unwrap_or_else(|| "未知错误".into());
            Ok(TimelineEntryView {
                title: match code {
                    Some(code) => format!("运行失败 · {code}"),
                    None => "运行失败".into(),
                },
                body: message,
                meta,
                tone_class: "divider divider-error",
            })
        }
        _ => Ok(TimelineEntryView {
            title: format!("事件 · {}", event.event_type),
            body: event.payload_json,
            meta,
            tone_class: "divider",
        }),
    }
}

fn json_string_field(payload_json: &str, field: &str) -> AppResult<Option<String>> {
    let value: Value = serde_json::from_str(payload_json)
        .map_err(|error| AppError::Infrastructure(error.to_string()))?;
    Ok(value
        .get(field)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned))
}

fn effective_status(session: &Session, unrecoverable: &BTreeSet<SessionId>) -> SessionStatus {
    if session.status == SessionStatus::Running && unrecoverable.contains(&session.id) {
        SessionStatus::Interrupted
    } else {
        session.status
    }
}

fn format_timestamp(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn infer_project_name(root_path: &std::path::Path) -> String {
    root_path
        .file_name()
        .map(|name| name.to_string_lossy().trim().to_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "新项目".into())
}

fn suggest_session_title_from_prompt(prompt: &str) -> String {
    let collapsed = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    let title = collapsed.trim();
    if title.is_empty() {
        return "新会话".into();
    }

    let mut truncated = String::new();
    for ch in title.chars().take(36) {
        truncated.push(ch);
    }
    if title.chars().count() > 36 {
        truncated.push('…');
    }
    truncated
}

fn runtime_source_label(source: Option<&RuntimeSource>) -> &'static str {
    match source {
        Some(RuntimeSource::Settings) => "应用覆盖",
        Some(RuntimeSource::Env) => "环境变量",
        Some(RuntimeSource::Path) => "系统 PATH",
        Some(RuntimeSource::PlatformDefault) => "平台默认位置",
        None => "未检测到",
    }
}

fn config_dir_source_label(source: Option<&ConfigDirectorySource>) -> &'static str {
    match source {
        Some(ConfigDirectorySource::Settings) => "应用覆盖",
        Some(ConfigDirectorySource::Env) => "环境变量",
        Some(ConfigDirectorySource::PlatformDefault) => "平台默认位置",
        None => "未检测到",
    }
}

#[cfg(test)]
mod tests {
    use super::{DesktopModel, DesktopServices};
    use app_core::InMemoryEventBus;
    use chrono::Utc;
    use domain::{Approval, ApprovalRequest, Project, Run, RunStatus, Session, SessionStatus};
    use infra_pimono::{PiMonoAdapter, StdProcessRunner};
    use infra_settings::{
        FsService, PlatformService, RuntimeLocator, SettingsStore, WorktreeService,
    };
    use infra_sqlite::Database;
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        path::{Path, PathBuf},
    };

    fn make_test_services(base: &Path) -> DesktopServices {
        DesktopServices {
            database: Database::in_memory().expect("database"),
            settings_store: SettingsStore::new(base.join("settings.json")),
            runtime_locator: RuntimeLocator::new(FsService, PlatformService),
            runtime_adapter: PiMonoAdapter::new(StdProcessRunner),
            workspace_service: WorktreeService::new(FsService),
            event_bus: InMemoryEventBus::default(),
        }
    }

    fn write_fake_runtime(base: &Path) -> PathBuf {
        let runtime_path = base.join("pi-mono");
        fs::write(
            &runtime_path,
            r#"#!/bin/bash
if [ "$1" = "--version" ]; then
  echo "pi-mono test 0.1.0"
  exit 0
fi
if [ "$1" = "session" ] && [ "$2" = "run" ]; then
  echo '{"type":"session_bound","session_id":"remote-session-1"}'
  echo '{"type":"run_started","run_id":"remote-run-1","session_id":"remote-session-1"}'
  echo '{"type":"text_delta","text":"live output"}'
  exit 0
fi
if [ "$1" = "session" ] && [ "$2" = "resume" ]; then
  echo '{"type":"session_bound","session_id":"remote-session-1"}'
  echo '{"type":"run_started","run_id":"remote-run-1","session_id":"remote-session-1"}'
  echo '{"type":"text_delta","text":"restored output"}'
  echo '{"type":"run_completed"}'
  exit 0
fi
if [ "$1" = "approval" ] && [ "$2" = "respond" ]; then
  exit 0
fi
if [ "$1" = "run" ] && [ "$2" = "inspect" ]; then
  echo '{"type":"run_completed"}'
  exit 0
fi
exit 0
"#,
        )
        .expect("runtime script");
        let mut permissions = fs::metadata(&runtime_path)
            .expect("runtime metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&runtime_path, permissions).expect("runtime permissions");
        runtime_path
    }

    fn write_fake_pi_runtime(base: &Path) -> PathBuf {
        let runtime_path = base.join("pi");
        fs::write(
            &runtime_path,
            r#"#!/bin/bash
if [ "$1" = "--version" ]; then
  echo "0.67.68"
  exit 0
fi
if [ "$1" = "--mode" ] && [ "$2" = "json" ]; then
  echo '{"type":"session","id":"pi-session-1","cwd":"/tmp/project"}'
  echo '{"type":"message_update","assistantMessageEvent":{"type":"text_delta","delta":"hello from pi"}}'
  echo '{"type":"agent_end","messages":[]}'
  exit 0
fi
exit 0
"#,
        )
        .expect("pi runtime script");
        let mut permissions = fs::metadata(&runtime_path)
            .expect("runtime metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&runtime_path, permissions).expect("runtime permissions");
        runtime_path
    }

    #[test]
    fn model_can_create_project_and_session() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let services = make_test_services(temp_dir.path());
        let mut model = DesktopModel::from_services(services).expect("model");
        let project_root = temp_dir.path().join("project-root");
        std::fs::create_dir_all(&project_root).expect("project root");

        assert!(model.create_project("Pig Studio", &project_root.to_string_lossy()));
        assert!(model.create_session("Initial Session"));

        let workspace = model.workspace();
        assert_eq!(workspace.projects.len(), 1);
        assert_eq!(workspace.projects[0].sessions.len(), 1);
        assert_eq!(
            workspace.active_session_id,
            Some(workspace.projects[0].sessions[0].session_id.clone())
        );
    }

    #[test]
    fn saving_runtime_settings_refreshes_runtime_path() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let runtime_path = write_fake_runtime(temp_dir.path());
        let services = make_test_services(temp_dir.path());
        let mut model = DesktopModel::from_services(services).expect("model");

        assert!(model.save_runtime_settings(&runtime_path.to_string_lossy()));
        assert_eq!(
            model.workspace().runtime_path,
            runtime_path.to_string_lossy()
        );
        assert_eq!(model.workspace().runtime_source_label, "应用覆盖");
    }

    #[test]
    fn creates_followup_session_from_interrupted_history() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let services = make_test_services(temp_dir.path());
        let mut model = DesktopModel::from_services(services).expect("model");
        let project_root = temp_dir.path().join("project-root");
        fs::create_dir_all(&project_root).expect("project root");

        assert!(model.create_project("Pig Studio", &project_root.to_string_lossy()));
        assert!(model.create_session("Original Session"));

        let session_id = model.selected_session_id.clone().expect("session id");
        let sessions = model.services.database.session_repository();
        sessions
            .update_status(&session_id, SessionStatus::Interrupted, Utc::now())
            .expect("interrupt session");
        model.refresh();

        assert!(model.create_followup_session_from_active());
        let workspace = model.workspace();
        assert_eq!(workspace.projects[0].sessions.len(), 2);
        assert!(
            workspace
                .active_session
                .as_ref()
                .expect("active session")
                .session_name
                .contains("新会话")
        );
    }

    #[test]
    fn first_prompt_generates_session_title_automatically() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let runtime_path = write_fake_runtime(temp_dir.path());
        let services = make_test_services(temp_dir.path());
        services
            .settings_store
            .save(&infra_settings::AppSettings {
                runtime_path: Some(runtime_path),
                ..Default::default()
            })
            .expect("save settings");

        let project_root = temp_dir.path().join("project");
        fs::create_dir_all(&project_root).expect("project root");

        let mut model = DesktopModel::from_services(services).expect("model");
        assert!(model.create_project("Pig Studio", &project_root.to_string_lossy()));
        assert!(model.create_session(""));
        assert!(model.send_prompt("修复侧边栏交互并自动检测 Pi 运行时"));

        let title = model
            .workspace()
            .active_session
            .as_ref()
            .expect("active session")
            .session_name
            .clone();
        assert!(title.contains("修复侧边栏交互"));
        assert_ne!(title, "新会话");
    }

    #[test]
    fn pi_runtime_from_path_streams_json_events() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let runtime_path = write_fake_pi_runtime(temp_dir.path());
        let services = make_test_services(temp_dir.path());
        services
            .settings_store
            .save(&infra_settings::AppSettings {
                runtime_path: Some(runtime_path.clone()),
                ..Default::default()
            })
            .expect("save settings");

        let project_root = temp_dir.path().join("project");
        fs::create_dir_all(&project_root).expect("project root");

        let mut model = DesktopModel::from_services(services).expect("model");
        assert!(model.create_project("Pig Studio", &project_root.to_string_lossy()));
        assert!(model.create_session(""));
        assert!(model.send_prompt("列出当前工作区状态"));

        let active_session = model
            .workspace()
            .active_session
            .as_ref()
            .expect("active session");
        assert!(
            active_session
                .timeline
                .iter()
                .any(|entry| entry.body.contains("hello from pi"))
        );
        assert!(
            active_session
                .timeline
                .iter()
                .any(|entry| entry.title.contains("会话已绑定"))
        );
    }

    #[test]
    fn send_prompt_binds_remote_run_metadata_from_runtime_events() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let runtime_path = write_fake_runtime(temp_dir.path());
        let services = make_test_services(temp_dir.path());
        services
            .settings_store
            .save(&infra_settings::AppSettings {
                runtime_path: Some(runtime_path),
                ..Default::default()
            })
            .expect("save settings");

        let project_root = temp_dir.path().join("project");
        fs::create_dir_all(&project_root).expect("project root");

        let mut model = DesktopModel::from_services(services).expect("model");
        assert!(model.create_project("Pig Studio", &project_root.to_string_lossy()));
        assert!(model.create_session("Track Metadata"));
        assert!(model.send_prompt("hello"));

        let session_id = model.selected_session_id.clone().expect("session id");
        let session = model
            .services
            .database
            .session_repository()
            .find(&session_id)
            .expect("find session")
            .expect("session");
        let runs = model
            .services
            .database
            .run_repository()
            .list_by_session(&session_id)
            .expect("runs");
        let run = runs.last().expect("latest run");

        assert_eq!(
            session.pimono_session_id.as_deref(),
            Some("remote-session-1")
        );
        assert_eq!(run.pimono_run_id.as_deref(), Some("remote-run-1"));
        assert!(
            model
                .workspace()
                .active_session
                .as_ref()
                .expect("active session")
                .timeline
                .iter()
                .any(|entry| entry.body.contains("remote-run-1"))
        );
    }

    #[test]
    fn startup_rebind_restores_runtime_output_for_running_session() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let runtime_path = write_fake_runtime(temp_dir.path());
        let services = make_test_services(temp_dir.path());
        services
            .settings_store
            .save(&infra_settings::AppSettings {
                runtime_path: Some(runtime_path),
                ..Default::default()
            })
            .expect("save settings");

        let now = Utc::now();
        let project_root = temp_dir.path().join("project");
        fs::create_dir_all(&project_root).expect("project root");

        let project = Project::new("Pig Studio", project_root.clone(), now);
        services
            .database
            .project_repository()
            .create(&project)
            .expect("project");

        let mut session = Session::new(project.id.clone(), "Recovered Session", project_root, now);
        session.status = SessionStatus::Running;
        session.pimono_session_id = Some(session.id.as_str().to_owned());
        services
            .database
            .session_repository()
            .create(&session)
            .expect("session");

        let mut run = Run::new(session.id.clone(), "continue", now);
        run.status = RunStatus::Running;
        services
            .database
            .run_repository()
            .create(&run)
            .expect("run");

        let model = DesktopModel::from_services(services).expect("model");
        let workspace = model.workspace();
        let active_session = workspace.active_session.as_ref().expect("active session");

        assert_eq!(active_session.status_badge.label, "已完成");
        assert!(
            active_session
                .timeline
                .iter()
                .any(|entry| entry.body.contains("restored output"))
        );
    }

    #[test]
    fn approval_response_records_timeline_and_resumes_bound_session() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let runtime_path = write_fake_runtime(temp_dir.path());
        let services = make_test_services(temp_dir.path());
        services
            .settings_store
            .save(&infra_settings::AppSettings {
                runtime_path: Some(runtime_path),
                ..Default::default()
            })
            .expect("save settings");

        let now = Utc::now();
        let project_root = temp_dir.path().join("project");
        fs::create_dir_all(&project_root).expect("project root");

        let project = Project::new("Pig Studio", project_root.clone(), now);
        services
            .database
            .project_repository()
            .create(&project)
            .expect("project");

        let mut session = Session::new(project.id.clone(), "Needs Approval", project_root, now);
        session.status = SessionStatus::WaitingApproval;
        session.pimono_session_id = Some(session.id.as_str().to_owned());
        services
            .database
            .session_repository()
            .create(&session)
            .expect("session");

        let mut run = Run::new(session.id.clone(), "approve this", now);
        run.status = RunStatus::WaitingApproval;
        services
            .database
            .run_repository()
            .create(&run)
            .expect("run");

        let approval = Approval::new(
            run.id.clone(),
            ApprovalRequest {
                request_id: "req-1".into(),
                correlation_id: None,
                request_type: "filesystem.delete".into(),
                request_payload_json: "{\"path\":\"README.md\"}".into(),
            },
            now,
        );
        services
            .database
            .approval_repository()
            .create(&approval)
            .expect("approval");

        let mut model = DesktopModel::from_services(services).expect("model");
        model.select_session(session.id.as_str());
        assert!(model.respond_to_approval(approval.id.as_str(), true));

        let active_session = model
            .workspace()
            .active_session
            .as_ref()
            .expect("active session");
        assert!(
            active_session
                .timeline
                .iter()
                .any(|entry| entry.title.contains("审批决策"))
        );
        assert!(
            active_session
                .timeline
                .iter()
                .any(|entry| entry.body.contains("restored output"))
        );
    }
}
