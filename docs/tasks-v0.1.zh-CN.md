# Berry Studio v0.1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use ordered `Step N` sections for sequential tracking.

**Goal:** 基于 `docs/architecture-v0.1.zh-CN.md` 与 `docs/prd-v0.1.zh-CN.md`，将仓库重构为 Berry Studio v0.1 的 Dioxus Desktop Rust Workspace，并交付项目管理、会话生命周期、运行事件流、审批、恢复与运行时设置的 MVP。

**Architecture:** 严格采用架构文档定义的分层结构：`Presentation -> Application -> Domain -> Infrastructure`。Presentation 使用 Dioxus Desktop + Tailwind CSS v4 + daisyUI 5 构建桌面 UI；Application 通过用例与事件总线编排流程；Domain 封装 `Project / Session / Run` 聚合与状态机；Infrastructure 提供 SQLite、`pi-mono`、设置、文件系统、平台与 worktree 能力。

**Tech Stack:** Rust stable、Dioxus Desktop、Tokio、Serde、SQLite、tracing、Tailwind CSS v4、daisyUI 5、Bun。

---

## 0. 实施前提与迁移原则

1. **本计划覆盖并替代上一版 `docs/tasks-v0.1.zh-CN.md`**。上一版基于 `Next.js + Tauri`，本次以 `docs/architecture-v0.1.zh-CN.md` 为唯一技术基线。
2. **仓库将从“前端 + Tauri”迁移为“Rust Workspace + Dioxus Desktop”**。现有 `src/`、`src-tauri/`、`next.config.ts`、`tsconfig.json`、`postcss.config.mjs` 等视为旧实现，直到新桌面应用闭环跑通后再删除。
3. **Bun 仍保留**，但职责收缩为样式工具链与顶层脚本调度：`bun run build` 负责编译 CSS 并触发 Cargo 构建，保证团队命令习惯不变。
4. **daisyUI / Tailwind 约束**：只使用 Tailwind utility 与 daisyUI class，不新增自定义视觉 CSS 规则；样式源文件只负责引入插件与主题。
5. **MVP 聚焦单机本地能力**：不做团队协作、云编排、多 Agent 扩展、IDE 替代。
6. **持久化边界**：Berry Studio 只存项目元数据、会话索引、事件、审批、设置与恢复所需非敏感数据；敏感值不落 SQLite。
7. **删除项目策略**：默认只解除应用关联，不删除真实项目目录与外部运行时数据。
8. **worktree 策略**：Git 项目可选，非 Git 项目始终 direct；worktree 失败不得阻塞普通会话创建。
9. **资源加载策略**：迁移 SQL 与样式等运行时资源不得依赖 repo root 或当前工作目录。迁移 SQL 使用编译期内嵌；CSS 使用桌面 bundle resource 方案（不使用编译期内嵌 CSS），并在打包形态验证可用。

## 1. 目标文件结构

### Root

- Create: `Cargo.toml` — Rust workspace 清单。
- Create: `rust-toolchain.toml` — 固定 stable 工具链。
- Modify: `package.json` — 样式编译与顶层 build/format 脚本。
- Modify: `bun.lock` — 固定样式工具链依赖版本。
- Create: `assets/styles/app.css` — Tailwind + daisyUI 入口。
- Create: `assets/styles/generated.css` — 编译产物与仓库内基线文件，作为桌面 bundle resource 供 Dioxus Desktop 加载。
- Create: `migrations/0001_init.sql` — 初始数据库 schema。

### Crates

- Create: `crates/shared-kernel/` — ID、错误、时间、事件信封等通用基础。
- Create: `crates/domain/` — `Project`、`Session`、`Run`、`Approval`、`RuntimeHealth`。
- Create: `crates/app-core/` — 用例、端口、事件总线、恢复编排。
- Create: `crates/infra-sqlite/` — SQLite 仓储实现与迁移加载。
- Create: `crates/infra-pimono/` — `pi-mono` 适配器、输出流解析、审批回传。
- Create: `crates/infra-settings/` — 设置存储、路径检查、平台目录、运行时定位。
- Create: `crates/ui-components/` — Dioxus 组件、展示模型、状态 badge 映射。
- Create: `crates/app-desktop/` — 入口、依赖装配、应用状态、恢复启动。

### Legacy（最终删除）

- Delete: `src/**`
- Delete: `src-tauri/**`
- Delete: `next.config.ts`
- Delete: `tsconfig.json`
- Delete: `postcss.config.mjs`
- Delete: `eslint.config.mjs`
- Delete: `.eslintrc.js`
- Delete: `components.json`
- Delete: `public/**`

---

### Task 1: 建立 Rust Workspace、Dioxus Desktop 壳与样式流水线

**Files:**

- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Modify: `package.json`
- Modify: `bun.lock`
- Create: `assets/styles/app.css`
- Create: `assets/styles/generated.css`
- Create: `crates/app-desktop/Cargo.toml`
- Create: `crates/app-desktop/src/main.rs`
- Create: `crates/app-desktop/src/lib.rs`
- Create: `crates/app-desktop/src/app.rs`
- Create: `crates/app-desktop/tests/bootstrap_smoke.rs`
- **Step 1: 创建根 workspace 清单与 crate 骨架**

Run:

```bash
mkdir -p crates assets/styles migrations
cargo new crates/app-desktop --bin --vcs none
```

Expected: 生成 `crates/app-desktop/`，仓库根目录具备迁移为 workspace 的基础结构。

- **Step 2: 写根 `Cargo.toml` 与 `rust-toolchain.toml`**

```toml
[workspace]
members = [
  "crates/app-desktop",
]
resolver = "2"
```

`rust-toolchain.toml` 最小内容：

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

- **Step 3: 配置 Bun 样式脚本并安装样式依赖**

Run:

```bash
bun add -d @tailwindcss/cli daisyui @biomejs/biome
cargo install --locked dioxus-cli
```

Expected: 样式工具链依赖安装完成并写入 `bun.lock`，且 `dx --version` 可用（供后续 `bundle:desktop` 使用）。

把 `package.json` 改成至少包含：

```json
{
  "scripts": {
    "build:styles": "bunx @tailwindcss/cli -i ./assets/styles/app.css -o ./assets/styles/generated.css --minify",
    "watch:styles": "bunx @tailwindcss/cli -i ./assets/styles/app.css -o ./assets/styles/generated.css --watch",
    "build": "bun run build:styles && cargo build --workspace",
    "format": "cargo fmt --all && bunx biome format --write docs package.json assets/styles"
  }
}
```

- **Step 4: 写 Tailwind v4 + daisyUI 入口 CSS**

`assets/styles/app.css` 最小内容：

```css
@import 'tailwindcss';
@plugin "daisyui" {
  themes:
    light --default,
    dark --prefersdark;
}
```

- **Step 5: 生成并提交 `generated.css` 基线文件**

Run:

```bash
bun run build:styles
```

Expected: `assets/styles/generated.css` 已生成并纳入版本管理，后续开发与 CI 在首次 Rust 编译前不因缺失样式产物失败。

- **Step 6: 先写失败测试，再实现最小 Dioxus 桌面壳**

`crates/app-desktop/tests/bootstrap_smoke.rs`：

```rust
use app_desktop::app::build_initial_shell;

#[test]
fn builds_initial_shell_with_empty_workspace() {
    let shell = build_initial_shell();
    assert!(shell.sidebar_open);
    assert!(shell.active_project_id.is_none());
    assert!(shell.active_session_id.is_none());
}
```

Run:

```bash
cargo test -p app-desktop --test bootstrap_smoke
```

Expected: 先失败，补齐 `build_initial_shell()` 与最小 `ShellState` 后通过。

- **Step 7: 让 Dioxus Desktop 以可打包方式加载 CSS**

在 `crates/app-desktop/src/main.rs` 中创建桌面入口，CSS 必须通过桌面 bundle resource 解析加载，不允许依赖 `assets/styles/generated.css` 的仓库相对路径。

- **Step 8: 运行样式编译与 workspace 冒烟构建**

Run:

```bash
bun run build:styles
cargo check -p app-desktop
```

Expected: CSS 生成成功，`app-desktop` 可编译。

- **Step 9: 提交基础壳与工具链**

```bash
git add Cargo.toml rust-toolchain.toml package.json bun.lock assets/styles/app.css assets/styles/generated.css crates/app-desktop
git commit -m "chore: bootstrap rust workspace and dioxus shell"
```

### Task 2: 建立 `shared-kernel` 与 `domain` 状态模型

**Files:**

- Create: `crates/shared-kernel/Cargo.toml`
- Create: `crates/shared-kernel/src/lib.rs`
- Create: `crates/shared-kernel/src/id.rs`
- Create: `crates/shared-kernel/src/error.rs`
- Create: `crates/shared-kernel/src/clock.rs`
- Create: `crates/shared-kernel/src/event.rs`
- Create: `crates/domain/Cargo.toml`
- Create: `crates/domain/src/lib.rs`
- Create: `crates/domain/src/project.rs`
- Create: `crates/domain/src/session.rs`
- Create: `crates/domain/src/run.rs`
- Create: `crates/domain/src/approval.rs`
- Create: `crates/domain/src/runtime.rs`
- **Step 1: 创建 `shared-kernel` 与 `domain` crate**

Run:

```bash
cargo new crates/shared-kernel --lib --vcs none
cargo new crates/domain --lib --vcs none
```

Expected: 两个 crate 生成完成。

- **Step 2: 先写 `SessionStatus` 与 `RunStatus` 失败测试**

在 `crates/domain/src/session.rs` 中先写测试：

```rust
#[test]
fn waiting_approval_is_not_terminal() {
    assert!(!SessionStatus::WaitingApproval.is_terminal());
}

#[test]
fn failed_is_terminal() {
    assert!(SessionStatus::Failed.is_terminal());
}
```

Run:

```bash
cargo test -p domain session::tests
```

Expected: 初始失败。

- **Step 3: 实现核心枚举与聚合根**

最少定义：

- `Project`
- `Session`
- `Run`
- `ApprovalDecision`
- `ApprovalStatus`
- `RuntimeHealth`

示例：

```rust
pub enum SessionStatus {
    Idle,
    Running,
    WaitingApproval,
    Blocked,
    Completed,
    Failed,
    Interrupted,
}
```

- **Step 4: 为 `Blocked` / `Interrupted` / `Failed` 边界补测试**

新增测试：

1. `Blocked` 表示不可执行但未产生新失败运行。
2. `Interrupted` 表示已启动运行中断。
3. `Failed` 表示某次运行执行后错误结束。

- **Step 5: 在 `shared-kernel` 实现统一错误与事件信封**

```rust
pub struct EventEnvelope<T> {
    pub seq: i64,
    pub created_at: DateTime<Utc>,
    pub payload: T,
}
```

- **Step 6: 跑领域层测试**

Run:

```bash
cargo test -p shared-kernel
cargo test -p domain
```

Expected: 两个 crate 测试通过。

- **Step 7: 提交领域模型层**

```bash
git add crates/shared-kernel crates/domain
git commit -m "feat: add shared kernel and domain models"
```

### Task 3: 建立 `infra-sqlite` 与本地持久化模型

**Files:**

- Create: `crates/infra-sqlite/Cargo.toml`
- Create: `crates/infra-sqlite/src/lib.rs`
- Create: `crates/infra-sqlite/src/db.rs`
- Create: `crates/infra-sqlite/src/migrations.rs`
- Create: `crates/infra-sqlite/src/repositories/project_repository.rs`
- Create: `crates/infra-sqlite/src/repositories/session_repository.rs`
- Create: `crates/infra-sqlite/src/repositories/run_repository.rs`
- Create: `crates/infra-sqlite/src/repositories/approval_repository.rs`
- Create: `crates/infra-sqlite/tests/sqlite_repositories.rs`
- Create: `migrations/0001_init.sql`
- **Step 1: 创建 `infra-sqlite` crate 并接入依赖**

Run:

```bash
cargo new crates/infra-sqlite --lib --vcs none
```

在 `crates/infra-sqlite/Cargo.toml` 中加入 `rusqlite`、`chrono`、`serde_json`、`tempfile`。

- **Step 2: 写初始 schema**

`migrations/0001_init.sql` 至少包含：

- `projects`
- `sessions`
- `runs`
- `events`
- `approvals`
- `app_settings`

要求：

1. `events` append-only。
2. `sessions.status` 作为快照态。
3. `deleted_at` 用于软删除。

- **Step 3: 先写仓储测试**

`crates/infra-sqlite/tests/sqlite_repositories.rs`：

```rust
#[test]
fn appends_events_without_overwriting_previous_rows() {
    // 创建临时库 -> 插入两条 event -> 断言 count == 2 且 seq 单调递增
}

#[test]
fn restores_pending_approval_for_waiting_session() {
    // 写入 waiting_approval session + pending approval -> 读取恢复视图
}
```

Run:

```bash
cargo test -p infra-sqlite --test sqlite_repositories
```

Expected: 初始失败。

- **Step 4: 实现数据库初始化与迁移加载（可打包）**

`crates/infra-sqlite/src/migrations.rs` 必须在打包形态可用：仅允许使用 `include_str!` 内嵌 `migrations/0001_init.sql`；不得读取 repo 根目录相对路径。首次启动时执行迁移。

- **Step 5: 实现项目、会话、运行、审批仓储**

仓储职责：

- `ProjectRepository`：增删查、置顶、最近打开时间
- `SessionRepository`：创建、重命名、软删除、列表、快照状态更新
- `RunRepository`：创建 run、更新终态、按会话查询
- `ApprovalRepository`：保存审批请求、决策、挂起状态恢复
- **Step 6: 跑 `infra-sqlite` 全量测试**

Run:

```bash
cargo test -p infra-sqlite
```

Expected: 通过。

- **Step 7: 提交持久化层**

```bash
git add crates/infra-sqlite migrations/0001_init.sql
git commit -m "feat: add sqlite persistence layer"
```

### Task 4: 建立 `infra-settings`、`FsService`、`PlatformService` 与 `runtime-locator`

**Files:**

- Create: `crates/infra-settings/Cargo.toml`
- Create: `crates/infra-settings/src/lib.rs`
- Create: `crates/infra-settings/src/settings_store.rs`
- Create: `crates/infra-settings/src/fs_service.rs`
- Create: `crates/infra-settings/src/platform_service.rs`
- Create: `crates/infra-settings/src/runtime_locator.rs`
- Create: `crates/infra-settings/tests/runtime_health.rs`
- **Step 1: 创建 `infra-settings` crate**

Run:

```bash
cargo new crates/infra-settings --lib --vcs none
```

Expected: crate 创建成功。

- **Step 2: 先写运行时健康检查测试**

`crates/infra-settings/tests/runtime_health.rs`：

```rust
#[test]
fn missing_runtime_path_returns_blocked_health() {
    // path 不存在 -> health.available = false -> reason 包含 missing
}

#[test]
fn existing_executable_returns_available_health() {
    // 指向临时可执行文件 -> health.available = true
}
```

Run:

```bash
cargo test -p infra-settings --test runtime_health
```

Expected: 初始失败。

- **Step 3: 实现设置存储与平台目录解析**

要求：

1. 统一 macOS / Windows / Linux 的 App Data 目录。
2. 设置仅保存运行时路径、环境变量 map、健康检查时间戳。
3. 不保存敏感 token 明文到事件流。

- **Step 4: 实现文件系统与路径校验能力**

`FsService` 至少覆盖：

- 项目目录存在性检查
- Git 仓库识别
- worktree 目录合法性检查
- 当前会话工作目录可访问性检查
- **Step 5: 实现 `runtime-locator` 搜索顺序**

实现固定且可测试的三端回退顺序（macOS / Windows / Linux 同一优先级语义）并返回结构化诊断信息：

1. 优先使用应用设置中的 `runtime_path`。
2. 若未命中，回退到环境变量 `PI_MONO_PATH`。
3. 若未命中，回退到系统 `PATH` 中的 `pi-mono` / `pi-mono.exe`。
4. 若未命中，回退到平台默认安装目录候选（由 `PlatformService` 提供）。
5. 任一层级命中且通过“存在 + 可执行 + 可读取版本”校验后立即返回，不再继续向下回退。
6. 诊断结果需记录每一层的尝试结果、失败原因与最终命中来源，便于测试断言与 UI 展示。

```rust
pub struct RuntimeHealth {
    pub available: bool,
    pub version: Option<String>,
    pub reason: Option<String>,
}
```

- **Step 6: 跑基础设施设置层测试**

Run:

```bash
cargo test -p infra-settings
```

Expected: 通过。

- **Step 7: 提交设置与平台服务**

```bash
git add crates/infra-settings
git commit -m "feat: add settings fs and runtime locator services"
```

### Task 5: 建立 `infra-pimono` 与流式事件解析

**Files:**

- Create: `crates/infra-pimono/Cargo.toml`
- Create: `crates/infra-pimono/src/lib.rs`
- Create: `crates/infra-pimono/src/adapter.rs`
- Create: `crates/infra-pimono/src/stream_parser.rs`
- Create: `crates/infra-pimono/src/process.rs`
- Create: `crates/infra-pimono/src/models.rs`
- Create: `crates/infra-pimono/tests/stream_parser.rs`
- **Step 1: 创建 `infra-pimono` crate**

Run:

```bash
cargo new crates/infra-pimono --lib --vcs none
```

Expected: crate 创建成功。

- **Step 2: 先写流式解析失败测试**

`crates/infra-pimono/tests/stream_parser.rs` 至少覆盖：

1. 文本增量输出 -> `TextDelta`
2. 审批请求 -> `ApprovalRequested`
3. 错误输出 -> `RunFailed`
4. 正常结束 -> `RunCompleted`

Run:

```bash
cargo test -p infra-pimono --test stream_parser
```

Expected: 初始失败。

- **Step 3: 实现 `pi-mono` 输出模型与解析器**

示例：

```rust
pub enum PiMonoEvent {
    TextDelta { text: String },
    ApprovalRequested { request_id: String, request_type: String, payload_json: String },
    RunFailed { code: Option<String>, message: String },
    RunCompleted,
}
```

- **Step 4: 实现进程适配器接口**

最小接口：

- `start_session_run(...)`
- `resume_session(...)`
- `respond_approval(...)`
- `inspect_run_status(...)`
- **Step 5: 为“先持久化后广播”准备事件回调接口**

适配器不直接碰 UI，只向上层返回标准化事件流：

```rust
pub trait PiMonoEventSink {
    fn push(&self, event: PiMonoEvent);
}
```

- **Step 6: 跑 `infra-pimono` 测试**

Run:

```bash
cargo test -p infra-pimono
```

Expected: 通过。

- **Step 7: 提交流式适配层**

```bash
git add crates/infra-pimono
git commit -m "feat: add pi-mono adapter and event parser"
```

### Task 6: 建立 `app-core` 用例、端口与事件总线

**Files:**

- Create: `crates/app-core/Cargo.toml`
- Create: `crates/app-core/src/lib.rs`
- Create: `crates/app-core/src/ports.rs`
- Create: `crates/app-core/src/event_bus.rs`
- Create: `crates/app-core/src/use_cases/create_project.rs`
- Create: `crates/app-core/src/use_cases/create_session.rs`
- Create: `crates/app-core/src/use_cases/send_prompt.rs`
- Create: `crates/app-core/src/use_cases/respond_approval.rs`
- Create: `crates/app-core/src/use_cases/resume_session.rs`
- Create: `crates/app-core/src/use_cases/update_runtime_settings.rs`
- Create: `crates/app-core/src/use_cases/reconcile_active_runs.rs`
- Create: `crates/app-core/tests/use_cases.rs`
- **Step 1: 创建 `app-core` crate**

Run:

```bash
cargo new crates/app-core --lib --vcs none
```

Expected: crate 创建成功。

- **Step 2: 先写用例测试，使用 fake ports**

`crates/app-core/tests/use_cases.rs` 至少覆盖：

1. `CreateProject` 会拒绝重复路径。
2. `CreateSession` 在非 Git 项目默认 `direct`。
3. `SendPrompt` 会先创建 run，再消费事件流。
4. `RespondApproval` 会先落库再调用 adapter。
5. `ReconcileActiveRuns` 会把不可恢复运行标成 `Interrupted`。

Run:

```bash
cargo test -p app-core --test use_cases
```

Expected: 初始失败。

- **Step 3: 定义应用层端口**

`crates/app-core/src/ports.rs` 至少声明：

- `ProjectRepositoryPort`
- `SessionRepositoryPort`
- `RunRepositoryPort`
- `ApprovalRepositoryPort`
- `PiMonoAdapterPort`
- `SettingsStorePort`
- `WorkspaceServicePort`
- **Step 4: 实现事件总线**

要求：

1. 应用层只广播标准化 UI 事件。
2. 所有运行事件先通过仓储持久化，再进入总线。
3. 恢复流程也通过统一总线发出状态更新。

- **Step 5: 实现六个核心用例**

与架构文档对齐：

- `CreateProject`
- `CreateSession`
- `SendPrompt`
- `RespondApproval`
- `ResumeSession`
- `UpdateRuntimeSettings`
- **Step 6: 实现活跃运行对账用例**

规则：

- 运行中可附着 -> 恢复输出流
- 运行已结束 -> 补终态事件
- 查询失败或不存在 -> 标记 `Interrupted`
- **Step 7: 跑 `app-core` 测试**

Run:

```bash
cargo test -p app-core
```

Expected: 通过。

- **Step 8: 提交应用层**

```bash
git add crates/app-core
git commit -m "feat: add application use cases and event bus"
```

### Task 7: 建立 `ui-components` 并实现桌面工作区界面

**Files:**

- Create: `crates/ui-components/Cargo.toml`
- Create: `crates/ui-components/src/lib.rs`
- Create: `crates/ui-components/src/theme.rs`
- Create: `crates/ui-components/src/presenters.rs`
- Create: `crates/ui-components/src/components/sidebar.rs`
- Create: `crates/ui-components/src/components/session_header.rs`
- Create: `crates/ui-components/src/components/event_timeline.rs`
- Create: `crates/ui-components/src/components/composer.rs`
- Create: `crates/ui-components/src/components/approval_panel.rs`
- Create: `crates/ui-components/src/components/settings_panel.rs`
- Create: `crates/ui-components/tests/presenters.rs`
- **Step 1: 创建 `ui-components` crate**

Run:

```bash
cargo new crates/ui-components --lib --vcs none
```

Expected: crate 创建成功。

- **Step 2: 先写展示模型测试**

`crates/ui-components/tests/presenters.rs`：

```rust
#[test]
fn maps_waiting_approval_to_warning_badge() {
    let meta = session_status_badge(SessionStatus::WaitingApproval);
    assert_eq!(meta.label, "等待审批");
    assert_eq!(meta.class_name, "badge badge-warning");
}

#[test]
fn blocked_state_suggests_fix_settings_action() {
    let view = build_session_banner(SessionStatus::Blocked);
    assert!(view.action_label.unwrap().contains("设置"));
}
```

Run:

```bash
cargo test -p ui-components --test presenters
```

Expected: 初始失败。

- **Step 3: 实现状态映射与时间线展示模型**

必须覆盖架构文档中的 badge 规范：

- `Idle -> badge-ghost`
- `Running -> badge-info`
- `WaitingApproval -> badge-warning`
- `Blocked -> badge-secondary`
- `Completed -> badge-success`
- `Failed -> badge-error`
- `Interrupted -> badge-neutral`
- **Step 4: 用 Dioxus 组件实现主布局**

组件职责：

- `Sidebar`：`Project -> Session` 树、新建会话、设置入口
- `SessionHeader`：项目名、会话名、状态
- `EventTimeline`：消息、系统事件、错误、审批混排
- `Composer`：输入框、发送按钮、运行状态提示
- `ApprovalPanel`：审批卡片与决策按钮
- `SettingsPanel`：运行时路径、环境变量、健康状态
- **Step 5: 严格使用 daisyUI / Tailwind class**

界面 class 只用如下形式：

```rust
class: "btn btn-primary btn-sm"
class: "badge badge-warning"
class: "menu bg-base-200 rounded-box"
class: "card bg-base-100 shadow-sm"
```

不要引入额外自定义样式文件。

- **Step 6: 跑 `ui-components` 测试并做编译检查**

Run:

```bash
cargo test -p ui-components
cargo check -p ui-components
```

Expected: 通过。

- **Step 7: 提交展示层组件**

```bash
git add crates/ui-components
git commit -m "feat: add dioxus workspace components"
```

### Task 8: 在 `app-desktop` 中完成装配、恢复与桌面交互闭环

**Files:**

- Modify: `crates/app-desktop/Cargo.toml`
- Modify: `crates/app-desktop/src/lib.rs`
- Modify: `crates/app-desktop/src/main.rs`
- Modify: `crates/app-desktop/src/app.rs`
- Create: `crates/app-desktop/src/bootstrap.rs`
- Create: `crates/app-desktop/src/state.rs`
- Create: `crates/app-desktop/tests/recovery_smoke.rs`
- **Step 1: 先写启动恢复测试**

`crates/app-desktop/tests/recovery_smoke.rs` 至少覆盖：

1. 启动时恢复项目列表与最近会话入口。
2. `WaitingApproval` 会话可重建挂起审批视图。
3. 运行中但不可恢复的会话会显示 `Interrupted`。

Run:

```bash
cargo test -p app-desktop --test recovery_smoke
```

Expected: 初始失败。

- **Step 2: 在 `bootstrap.rs` 中装配所有依赖**

把这些实现连起来：

- `infra-sqlite`
- `infra-settings`
- `infra-pimono`
- `app-core`
- `ui-components`
- **Step 3: 实现桌面应用共享状态**

`state.rs` 至少包含：

- 已加载项目列表
- 当前项目 / 当前会话
- 时间线视图
- 设置面板显隐
- 活跃审批列表
- 运行时健康状态
- **Step 4: 接上核心交互**

要求：

1. 添加项目
2. 创建 / 打开 / 重命名 / 删除会话
3. 发送 prompt
4. 批准 / 拒绝审批
5. 打开设置并保存运行时配置

- **Step 5: 实现启动恢复与对账逻辑**

启动流程：

1. 加载项目和最近会话索引。
2. 尝试根据 `pimono_session_id + workspace_cwd` 恢复会话。
3. 恢复失败则降级为只读历史并提供“基于当前上下文新建会话”。
4. 对上次 `Running` 的 run 做真实状态对账。

- **Step 6: 做桌面闭环编译验证**

Run:

```bash
cargo test -p app-desktop
cargo run -p app-desktop
```

Expected: 测试通过；桌面应用能打开并显示工作区。打包产物与“非仓库工作目录启动”验证统一在 Task 9 的 `bundle:desktop` + `verify:bundle` 中执行。

- **Step 7: 提交桌面整合层**

```bash
git add crates/app-desktop
git commit -m "feat: wire desktop app bootstrap and recovery flows"
```

### Task 9: 加入 worktree、验收脚本、迁移收口并删除旧实现

**Files:**

- Create: `crates/app-core/src/use_cases/manage_worktree.rs`
- Create: `crates/infra-settings/src/worktree_service.rs`
- Create: `docs/manual-qa-v0.1.zh-CN.md`
- Create: `scripts/verify-bundle.mjs`
- Modify: `package.json`
- Modify: `bun.lock`
- Modify: `README.md`
- Delete: `src/**`
- Delete: `src-tauri/**`
- Delete: `next.config.ts`
- Delete: `tsconfig.json`
- Delete: `postcss.config.mjs`
- Delete: `eslint.config.mjs`
- Delete: `.eslintrc.js`
- Delete: `components.json`
- Delete: `public/**`
- **Step 1: 先写 worktree 行为测试**

最少覆盖：

1. Git 项目启用 worktree 时生成隔离目录。
2. 非 Git 项目不会尝试创建 worktree。
3. worktree 失败时回退到 `direct`。

Run:

```bash
cargo test -p app-core manage_worktree
```

Expected: 初始失败。

- **Step 2: 实现 `workspace-service` 的 worktree 能力**

职责：

- 创建 / 复用 / 检测 worktree
- 删除会话时决定是否清理 worktree
- 冲突或目录失效时返回明确错误
- **Step 3: 把顶层构建命令收口为最终开发体验**

前置条件：已在 Task 1 Step 3 安装 Dioxus CLI（`dx` 命令可用）。

`package.json` 最终脚本建议：

```json
{
  "scripts": {
    "build:styles": "bunx @tailwindcss/cli -i ./assets/styles/app.css -o ./assets/styles/generated.css --minify",
    "build": "bun run build:styles && cargo build --workspace",
    "dev": "bun run build:styles && cargo run -p app-desktop",
    "bundle:desktop": "dx bundle --package app-desktop",
    "verify:bundle": "bun scripts/verify-bundle.mjs",
    "test": "cargo test --workspace",
    "format": "cargo fmt --all && bunx biome format --write docs package.json assets/styles"
  }
}
```

- **Step 4: 删除旧的 Next / Tauri 代码路径**

确保删除：

- `src/`
- `src-tauri/`
- `next.config.ts`
- `tsconfig.json`
- `postcss.config.mjs`
- `eslint.config.mjs`
- `.eslintrc.js`
- `components.json`
- `public/`

- **Step 5: 清理旧时代 JS 依赖并同步 lockfile**

要求：

1. 从 `package.json` 中移除迁移后不再使用的 Next / React / Tauri / TypeScript / ESLint 相关依赖。
2. 通过 Bun 重新解析依赖并更新 `bun.lock`。
3. 保留当前计划实际需要的样式与脚本依赖（如 `@tailwindcss/cli`、`daisyui`、`@biomejs/biome`）。

- **Step 6: 编写手工验收文档**

`docs/manual-qa-v0.1.zh-CN.md` 至少包含：

1. 一分钟内打开项目并创建会话。
2. 重启后恢复历史会话。
3. 正确区分 `Running / WaitingApproval / Blocked / Failed / Interrupted`。
4. 在应用内完成审批和设置更新。
5. Git 与非 Git 项目的会话流程都可用。

- **Step 7: 运行最终验证**

Run:

```bash
bun run build
cargo test --workspace
bun run bundle:desktop
bun run verify:bundle
```

Expected: 样式编译成功、workspace 全量测试通过、桌面应用可构建；`verify:bundle` 必须从打包产物目录直接启动应用（不是仓库目录 `cargo run`），并验证 migration 与 CSS 资源在真实交付形态可用。

- **Step 8: 提交 MVP 收口**

```bash
git add Cargo.toml package.json bun.lock assets/styles migrations crates scripts docs/manual-qa-v0.1.zh-CN.md README.md
git rm -r src src-tauri public || true
git rm next.config.ts tsconfig.json postcss.config.mjs eslint.config.mjs .eslintrc.js components.json || true
git commit -m "feat: finalize dioxus desktop mvp migration"
```

---

## 建议执行顺序

1. Task 1：先把 workspace、Dioxus 壳和样式流水线建立起来。
2. Task 2～5：再分层补齐 Domain 与三类 Infrastructure。
3. Task 6：接上 Application 用例与事件总线。
4. Task 7～8：实现 UI 与桌面整合、恢复、对账。
5. Task 9：补齐 worktree、清理旧实现并完成验收。

## 关键验收门槛

- 必须遵守 `Presentation -> Application -> Domain -> Infrastructure` 分层，不允许 UI 直接触碰 SQLite 或 `pi-mono` 进程。
- `events` 必须 append-only，且运行事件遵循“先持久化，后广播”。
- 左侧导航必须是 `Project -> Session`，而不是普通聊天列表。
- 设置入口必须位于侧边栏底部区域。
- 状态视觉映射必须符合架构文档中的 daisyUI badge 规范。
- 恢复失败必须显式可见，并提供重建入口，不能静默丢失会话。
- MVP 完成前，旧的 `Next.js + Tauri` 路径不得继续作为主实现保留。

