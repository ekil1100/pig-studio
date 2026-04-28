# Pig Studio 技术架构方案 v0.1（中文）

## 1. 目标与范围

本方案基于 `docs/prd-v0.1.zh-CN.md`，覆盖 Pig Studio v0.1 MVP 的技术实现路径，目标是：

1. 支持按本地项目组织工作空间。
2. 支持创建、恢复、删除 `agent sessions`。
3. 支持在会话中进行持续交互并查看运行事件流。
4. 支持审批请求的展示与决策。
5. 支持 `pi-mono` 运行时配置与健康检查。

本方案聚焦单机本地能力，不包含团队协作与云端编排。

前置约束说明：

1. v0.1 采用 Dioxus Desktop 作为桌面 UI 技术路线。
2. PRD 已同步为“桌面端交付为唯一形态，不提供浏览器 SaaS 版本”。
3. 在此口径下，Tailwind CSS + coss UI 语义样式 仅作为本地桌面渲染层样式系统，不改变产品桌面定位。

## 2. 技术栈

- 语言：`Rust`（stable 最新）
- UI：`Dioxus`（最新稳定版，Desktop）
- 样式：`Tailwind CSS v4` + `coss UI 语义样式`
- 本地存储：`SQLite`
- 异步运行时：`Tokio`
- 序列化：`Serde`
- 日志与追踪：`tracing` + `tracing-subscriber`

版本策略：

1. Rust 工具链随 stable 最新版本迭代。
2. Cargo 依赖使用最新稳定版本，按迭代周期集中升级与回归验证。
3. 前端样式工具链通过 lockfile 固定，确保跨平台一致构建。

## 3. 总体架构

采用分层架构：`Presentation -> Application -> Domain -> Infrastructure`。

### 3.1 Presentation（界面层）

- 基于 Dioxus Desktop 构建主窗口与页面布局。
- 组件划分：`Sidebar`、`SessionHeader`、`EventTimeline`、`Composer`、`ApprovalPanel`、`SettingsPanel`。
- UI 组件链路：`base-ui-dioxus -> coss-ui-dioxus -> ui-components/app-desktop`。
- `base-ui-dioxus` 提供无样式 Dioxus primitive，参考 Base UI 的 headless parts 组合方式。
- `coss-ui-dioxus` 封装 coss 风格变体、尺寸和 Tailwind class，应用层直接引用组件而不是拼接组件 class。
- Tailwind + coss UI token 提供统一主题与状态视觉映射。

### 3.2 Application（应用层）

- 编排业务用例，典型命令：
  - `CreateProject`
  - `CreateSession`
  - `SendPrompt`
  - `RespondApproval`
  - `ResumeSession`
  - `UpdateRuntimeSettings`
- 提供事件总线，连接运行事件与 UI 刷新。

### 3.3 Domain（领域层）

- 核心聚合：
  - `Project`
  - `Session`
  - `Run`
- 核心值对象：
  - `SessionStatus`（Idle / Running / WaitingApproval / Blocked / Completed / Failed / Interrupted）
  - `RunStatus`（Queued / Running / WaitingApproval / Completed / Failed / Interrupted）
  - `ApprovalDecision`（Approve / Reject）
  - `RuntimeHealth`

### 3.4 Infrastructure（基础设施层）

- `PiMonoAdapter`：与 `pi-mono` 进程通信并消费输出流。
- `Repository`：SQLite 的持久化实现。
- `SettingsStore`：运行时路径、环境变量与配置读取。
- `FsService`：本地文件系统访问和项目路径校验。
- `PlatformService`：统一处理跨平台目录、路径、环境注入与可执行定位。

## 4. 关键模块设计

1. `project-service`
   - 添加本地目录为项目
   - 最近/置顶项目管理
2. `session-service`
   - 创建、打开、重命名、删除会话
   - 会话索引与恢复入口
3. `run-orchestrator`
   - 触发 run
   - 接收流式事件并持久化
   - 驱动会话状态流转
4. `approval-service`
   - 接收审批请求
   - 记录用户决策并继续/终止运行
5. `timeline-service`
   - 聚合消息、事件、审批、错误，按时间线输出
6. `settings-service`
   - 维护 `pi-mono` 路径与环境配置
   - 执行健康检查并输出可读诊断
7. `workspace-service`
   - 管理会话与工作目录绑定关系
   - 在 Git 项目中按需提供 worktree 创建、复用、清理与冲突检测
8. `runtime-locator`
   - 检测三端 `pi-mono` 可执行路径（macOS / Windows / Linux）
   - 按统一回退顺序定位可执行文件（优先级从高到低）：
     1) 用户在应用设置中显式配置的 `runtime_path`
     2) 环境变量 `PI_MONO_PATH`
     3) 系统 `PATH` 中可解析到的 `pi-mono` / `pi-mono.exe`
     4) 平台默认安装目录候选（由 `PlatformService` 提供，每端可有多候选）
   - 当某一层级命中且可执行通过基础校验（存在、可执行、可读取版本）时立即返回，不再继续向低优先级回退
   - 所有候选尝试结果必须输出结构化诊断（命中来源、失败原因、最终选中路径）
   - 统一环境变量拼装与注入策略

## 5. 数据存储模型（SQLite）

建议最小表结构如下：

### 5.1 `projects`

- `id`
- `name`
- `root_path`
- `pinned`
- `last_opened_at`
- `created_at`
- `updated_at`

### 5.2 `sessions`

- `id`
- `project_id`
- `title`
- `status`
- `pimono_session_id`
- `workspace_cwd`
- `workspace_mode`（direct / worktree）
- `worktree_path`（nullable）
- `last_run_at`
- `created_at`
- `updated_at`
- `deleted_at`

### 5.3 `runs`

- `id`
- `session_id`
- `pimono_run_id`
- `trigger_input`
- `status`
- `started_at`
- `ended_at`
- `error_code`
- `error_message`

### 5.4 `events`

- `id`
- `session_id`
- `run_id`
- `seq`
- `event_type`
- `payload_json`
- `created_at`

### 5.5 `approvals`

- `id`
- `run_id`
- `request_id`
- `correlation_id`
- `request_type`
- `request_payload_json`
- `status`
- `decision`
- `created_at`
- `decided_at`

### 5.6 `app_settings`

- `key`
- `value_json`
- `updated_at`

设计原则：

1. `events` 采用 append-only，保证恢复与审计可追溯。
2. `sessions.status` 保存快照态，完整上下文由时间线事件重建。
3. `sessions.pimono_session_id` 作为“可继续同一 agent session”的主锚点。
4. `workspace_mode` 默认为 `direct`，仅在 Git 项目且用户启用时切换为 `worktree`。
5. `approvals.status` 采用状态机驱动：`Pending / Approved / Rejected / Expired / Interrupted`。
6. v0.1 不在本地数据库持久化敏感凭证；敏感值由 `pi-mono` 或系统凭证存储管理，本地仅保留可恢复所需的非敏感索引。

## 6. 核心流程

### 6.1 新建会话与运行

1. 用户在项目下创建会话。
2. 用户输入 prompt 并发送。
3. `run-orchestrator` 调用 `PiMonoAdapter` 发起运行。
4. 流式输出转为事件写入 `events`，同时推送 UI。
5. 运行结束后更新 `runs` 与 `sessions.status`。

### 6.2 审批流程

1. 运行中收到审批请求事件。
2. UI 展示审批卡片并将状态置为 `WaitingApproval`。
3. 用户批准或拒绝。
4. 决策写入 `approvals` 与 `events`。
5. 编排层向 `pi-mono` 回传结果，继续或终止当前运行。

### 6.3 重启恢复流程

1. 启动时加载项目列表与最近会话索引。
2. 打开会话时优先根据 `pimono_session_id + workspace_cwd` 尝试恢复远端会话绑定。
3. 若绑定恢复成功，再按 `events + runs + approvals` 重建完整上下文。
4. 若绑定恢复失败，进入降级模式：会话可读、不可直接继续，用户可选择“基于当前上下文新建会话”。
5. 若绑定恢复成功且上次状态为 `WaitingApproval`，通过 `approvals` 中 `Pending` 请求重建待处理审批卡片并允许继续决策。
6. 若绑定恢复失败且存在 `Pending` 审批，将其标记为 `Interrupted` 或 `Expired`，并提示用户基于上下文新建会话。

### 6.4 活跃运行对账（Active Run Reconciliation）

1. 应用启动时扫描上次状态为 `Running` 的会话与 `runs`。
2. 若存在 `pimono_run_id`，优先向 `pi-mono` 查询运行真实状态。
3. 查询结果为运行中时，重新附着输出流并恢复实时展示。
4. 查询结果为已结束时，按真实结果补写终态事件并更新 `sessions.status`。
5. 查询失败或运行不存在时，将 run 标记为 `Interrupted`，并在 UI 明确提示“上次运行中断，可重试”。

### 6.5 Blocked 状态定义与流转

1. 当运行前置条件不满足且短时间无法自动恢复时，会话进入 `Blocked`，典型场景包括：
   - `pi-mono` 运行时不可用或版本不兼容
   - 必需配置无效（路径不存在、关键配置缺失）
   - 当前会话工作目录不可访问（目录被删除或权限变化）
2. `Blocked` 与 `WaitingApproval` 的边界：
   - `WaitingApproval` 表示运行已正常进行到审批节点，等待用户业务决策
   - `Blocked` 表示运行无法开始或无法继续，等待用户修复环境或配置
3. `Blocked` 与 `Interrupted` 的边界：
   - `Interrupted` 是“已启动运行”在恢复或对账时确认中断
   - `Blocked` 是“运行前或恢复前检查”阶段即发现不可执行
4. `Blocked` 与 `Failed` 的边界：
   - `Failed` 表示一次运行已执行并以错误结束
   - `Blocked` 表示当前不可执行，不产生新的失败运行结果
5. 从 `Blocked` 退出的规则：
   - 用户修复配置并通过健康检查后，状态转为 `Idle`
   - 用户执行“重新绑定会话”成功后，按恢复结果转为 `Idle` 或 `WaitingApproval`
   - 用户放弃修复并新建会话时，原会话保持 `Blocked` 仅作历史查看

## 7. UI 架构映射

### 7.1 左侧边栏

- 项目列表（可置顶）
- 项目下会话树（可展开）
- 新建会话入口
- 底部设置按钮

### 7.2 主会话区域

- 头部：项目名、会话名、状态
- 主体：消息与执行事件流
- 底部：输入框、发送操作、运行状态提示

### 7.3 状态视觉规范（coss UI）

- `idle` -> `coss-badge-ghost`
- `running` -> `coss-badge-info`
- `waiting_approval` -> `coss-badge-warning`
- `blocked` -> `coss-badge-secondary`
- `completed` -> `coss-badge-success`
- `failed` -> `coss-badge-error`
- `interrupted` -> `coss-badge-neutral`

## 8. 工程目录建议（Rust Workspace）

```text
berry-studio/
  Cargo.toml
  crates/
    app-desktop/
    base-ui-dioxus/
    coss-ui-dioxus/
    ui-components/
    app-core/
    domain/
    infra-sqlite/
    infra-pimono/
    infra-settings/
    shared-kernel/
  assets/
    styles/
  migrations/
```

说明：

1. `app-core` 只依赖领域接口，不直接依赖具体数据库实现。
2. `infra-*` 通过 trait 实现注入到应用层。
3. `ui-components` 保持可复用和弱业务耦合。

## 9. 非功能与质量要求

1. 性能
   - 事件流分段加载
   - 时间线渲染节流
2. 可靠性
   - 关键事件先持久化再更新状态
   - 崩溃恢复时保证状态与历史一致
3. 可观测性
   - run 级别日志关联 `run_id`
   - 错误码规范化并可在 UI 直观展示
4. 安全性
   - 本地路径合法性校验
   - 审批默认显式决策，不隐式放行
   - 敏感凭证不写入 SQLite，不写入事件流明文
5. 跨平台一致性
   - 统一 App Data 目录策略（macOS/Windows/Linux）
   - 统一 SQLite 数据库落盘路径与迁移策略
   - 统一 `pi-mono` 可执行发现与回退顺序
   - 路径分隔符与大小写敏感差异通过 `PlatformService` 屏蔽

## 10. 里程碑计划

### M1：最小闭环

- 项目管理
- 会话创建/打开
- prompt 提交与基础事件流展示

### M2：恢复与稳定

- 本地持久化完善
- 重启恢复
- 失败状态与错误可视化

### M3：审批能力

- 审批请求展示
- 批准/拒绝决策
- 决策回写与运行继续

### M4：设置与健康检查

- `pi-mono` 路径与环境配置
- 运行时可用性检测
- 配置异常提示与修复引导

## 11. 风险与待决策项

1. 审批类型的细分粒度需要在 v0.1 冻结前确定。
2. 时间线展示深度需要平衡可读性与调试价值。
3. 删除项目时对历史数据的保留策略需要明确。
4. 为未来多 Agent 类型支持预留扩展点，但 v0.1 不提前复杂化实现。
5. Worktree 生命周期策略需要明确：自动清理还是长期保留。
