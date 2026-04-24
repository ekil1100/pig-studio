# Pig Studio v0.1 Requirements Design

## 1. 文档目的

本文档定义 Pig Studio v0.1 重做版需求。该版本将 Pig Studio 明确为 `pi code agent superpower gui`：一个面向开发者的桌面 GUI，用于管理项目、创建 agent session、与底层 agent 对话，并复用 `pi-mono` / pi coding agent 已有的 session、配置、环境变量和扩展能力。

本文档覆盖产品定位、核心交互、功能需求、信息架构、关键流程、数据边界、验收标准与待定问题。后续实现、架构设计和任务拆解应以本文档为 v0.1 的主需求依据。

## 2. 产品定义

Pig Studio 是基于 `pi-mono` / pi coding agent runtime 的桌面图形工作台。它不是独立的新 agent runtime，也不是通用聊天应用，而是基于 `pi-mono` 底层能力构建的 GUI layer。

核心定义：

1. Pig Studio 负责提供项目、会话、对话、运行状态、审批、配置入口与可视化反馈。
2. `pi-mono` / pi coding agent runtime 负责真实 agent 执行、session 持久化、模型配置、thinking 配置、环境变量、skill、prompt、extension 和底层运行语义。
3. Pig Studio 应尽量共享 `pi-mono` 的状态与配置，避免创建一套无法与 CLI/底层 agent 互通的平行体系。

## 3. 产品定位

Pig Studio 是 `pi code agent superpower gui`。

面向用户：

1. 希望打开桌面应用后直接使用 coding agent 的开发者。
2. 希望用更清晰的 GUI 管理 project、session、模型、thinking level、worktree 和上下文用量的人。
3. 希望保留 CLI/agent 原生能力，同时获得类似 Codex 的会话交互体验和类似 Linear 的简洁工作界面的人。

产品不是：

1. 不是新的 agent provider 聚合器。
2. 不是 pi code agent 的替代实现。
3. 不是完整 IDE。
4. 不是云端协作平台。
5. 不是通用 AI chat 产品。

## 4. 设计原则

### 4.1 底层共享优先

Pig Studio 必须优先读取、复用和尊重 `pi-mono` / pi coding agent 的现有能力。

共享范围：

1. 内置 `pi-mono` runtime 状态。
2. session 历史。
3. 环境变量。
4. 配置文件。
5. model 与 thinking level 配置。
6. skill / prompt / extension。
7. 项目运行目录与 agent 工作上下文。

GUI 只能保存必要的 UI 索引、窗口状态、最近项目、连接元数据和展示偏好。除非明确需要，不复制 `pi-mono` 的核心配置和 session 数据。

### 4.2 开箱即用优先

Pig Studio v0.1 的默认体验必须是零额外安装。用户下载安装 Pig Studio 后，应能直接打开应用、选择项目，并在已有 pi provider 认证可用时启动 agent session。

默认交付要求：

1. 应用包内应自带可运行的 `pi-mono` / pi coding agent runtime。
2. 默认使用内置 runtime，不要求用户提前安装 `pi`、`pi-mono`、Node、Bun 或 npm package。
3. v0.1 不提供外部 runtime 切换入口，避免引入版本兼容、路径解析和调试复杂度。
4. Provider、model 和认证依赖 pi 现有体系；Pig Studio 不自建 provider 管理和凭证存储。
5. 如果内置 runtime 损坏、版本不兼容或缺少平台支持，应给出明确错误和修复入口，而不是要求用户自行排查命令行。

### 4.3 Provider 体系复用

Pig Studio 必须依赖 pi 的 provider、model 和认证体系，不重新实现一套 provider abstraction。

复用范围：

1. pi 内置 provider 列表。
2. pi 的 model registry。
3. `~/.pi/agent/auth.json` 中的 API key / OAuth token。
4. 环境变量中的 provider key，例如 `ANTHROPIC_API_KEY`、`OPENAI_API_KEY`。
5. `~/.pi/agent/models.json` 中的 custom provider / custom model。
6. pi 支持的 subscription / OAuth 登录状态。

Pig Studio 的职责：

1. 展示当前 provider/model 是否可用。
2. 展示可用模型列表。
3. 引导用户配置 pi provider 认证。
4. 在无可用模型时进入 onboarding / diagnostic 状态。
5. 不在 Pig Studio 数据库中保存 API key、OAuth token 或 provider secret。

如果 RPC v0.1 暂不提供完整登录命令，Pig Studio 应先提供配置路径、环境变量说明和打开 pi 认证/配置入口；后续版本再做原生 GUI 登录。

### 4.4 GUI 是操作台，不是解释页

首屏应直接提供可操作的工作台，而不是营销式介绍页。用户进入应用后，应能立即看到项目列表、session 入口、运行状态和可执行操作。

### 4.5 交互参考 Codex

核心交互可以参考 Codex：

1. 左侧为项目与 session 导航。
2. 中央为当前 session 对话和执行流。
3. 底部 composer 作为主要输入区。
4. composer 下方或附近提供 model、thinking level、运行位置、token/context 数据。
5. 运行中清晰展示 agent 正在做什么、是否等待审批、是否失败。

### 4.6 视觉参考 Linear

界面风格参考 Linear app：

1. 低装饰、低饱和、浅色优先。
2. 边框和留白组织信息，避免重阴影和玻璃拟态。
3. 侧栏紧凑，内容区可扫描。
4. 卡片只用于真正需要分组的内容，不做大面积装饰卡片。
5. 控件圆角克制，信息层级明确。

## 5. 核心术语

### 5.1 pi-mono / pi coding agent runtime

底层 agent runtime、CLI、RPC 和 SDK 能力提供方。Pig Studio v0.1 默认随应用内置该 runtime，并优先通过官方 RPC mode 集成；后续可在需要更深控制时引入 SDK sidecar。

### 5.2 内置 Runtime

随 Pig Studio 应用包一起分发的 `pi-mono` / pi coding agent runtime。它是默认执行后端，负责开箱即用体验。内置 runtime 不应破坏用户已有 `~/.pi/agent`、项目 `.pi/`、`.agents/skills/` 等共享资源语义。

### 5.3 Project

用户在 Pig Studio 中创建或打开的代码工作入口。Project 可以指向本地目录、WSL 路径或 SSH 远端目录。

### 5.4 Session

pi coding agent runtime 在某个 project 下的持久会话。一个 project 可以创建多个 session。Session 历史应与 `pi-mono` 共享，GUI 打开的历史应能被 CLI 和底层 agent 识别。

### 5.5 Connection

Project 的访问方式。v0.1 支持：

1. `local`：本机文件系统目录。
2. `wsl`：Windows 主机下的 WSL distro 路径。
3. `ssh`：远端主机路径。

### 5.6 Workspace Mode

Session 的工作目录策略。

1. `local`：直接在项目目录或指定 cwd 中运行。
2. `worktree`：为 session 创建或绑定 Git worktree。

### 5.7 Worktree

Git 项目的隔离工作树，用于降低多个 session 并行修改同一仓库时的冲突风险。v0.1 的 worktree 创建逻辑可参考 `wt` / `worktrunk`：<https://github.com/max-sixty/worktrunk>。

### 5.8 Model

当前 session 使用的模型。模型来源应使用 `pi-mono` 配置，GUI 提供选择入口但不自行定义独立 provider 体系。

### 5.9 Provider Auth

由 pi 管理的 provider 认证状态。来源包括 `~/.pi/agent/auth.json`、环境变量、`models.json` 和 pi 支持的 OAuth/subscription 登录。Pig Studio 只读取和展示状态，不持久化敏感凭证。

### 5.10 Thinking Level

当前 session 使用的思考强度。具体枚举与可用值以 `pi-mono` 配置为准，GUI 负责展示和切换。

### 5.11 Context Limit Percentage

当前 session 已使用上下文占最大上下文限制的比例，用于提示用户是否接近上下文窗口上限。

### 5.12 Token Usage

当前 session 或当前 run 的 token 使用数据，包括输入、输出、缓存、总量等，具体字段以 `pi-mono` RPC / SDK 可提供的数据为准。

## 6. v0.1 范围

### 6.1 必须支持

1. 随应用内置可运行的 `pi-mono` / pi coding agent runtime。
2. 首次启动无需用户额外安装 `pi`、`pi-mono`、Node、Bun 或 npm package。
3. 检测内置 runtime 是否可用。
4. 读取 pi provider/model/auth 状态。
5. 无可用 provider/model 时提供 onboarding / diagnostic 状态。
6. 读取 `pi-mono` 配置、环境变量、session 历史和扩展能力入口。
7. 创建 project 或打开已有 project。
8. Project 支持 local / WSL / SSH 连接类型。
9. 每个 project 可以创建多个 session。
10. 在 provider/model 可用时，在主交互区与 agent 对话。
11. 在 composer 附近调整 model。
12. 在 composer 附近调整 thinking level。
13. 创建 session 时选择 local 或 worktree 模式。
14. 对 Git project 支持分支切换。
15. 展示 token 使用数据。
16. 展示 context limit percentage。
17. 展示运行状态、审批、错误和失败恢复入口。
18. 提供 Linear 风格的桌面 UI。

### 6.2 应该支持

1. 对已存在的 `pi-mono` / pi coding agent session 做索引和恢复。
2. 将 session 历史按 project 分组展示。
3. SSH/WSL project 的环境变量和配置继承对应连接上下文。
4. worktree 创建失败时允许用户退回 local 模式。
5. 支持 session 重命名、归档或删除 GUI 索引。
6. 支持打开底层配置文件、skill、prompt、extension 目录。

### 6.3 v0.1 暂不支持

1. 多用户协作。
2. 云端 session 同步。
3. 多 agent provider 抽象。
4. 完整 IDE 编辑器能力。
5. 自定义 agent runtime 实现。
6. 自动合并多个 session 的代码修改。
7. 要求用户手动安装或升级底层 runtime 才能完成基础使用。
8. 在 Pig Studio 中自建独立 provider / credential 系统。

## 7. 信息架构

### 7.1 App Shell

主窗口采用三段式布局：

1. 左侧 Sidebar：project、session、连接和设置入口。
2. 中央 Session Area：当前 session 的消息、运行事件和审批。
3. 底部 Composer：输入 prompt、选择 model、选择 thinking level、查看 token/context 状态。

### 7.2 Sidebar

Sidebar 应包含：

1. 应用标识与当前 agent runtime 状态。
2. Project 列表。
3. 每个 project 下的 session 列表。
4. 新建 project / 打开 project 操作。
5. 当前连接类型标识：local / WSL / SSH。
6. 设置入口。

Project item 展示：

1. 项目名称。
2. 根路径或远端路径摘要。
3. 连接类型。
4. 当前活跃 session 数量或最近 session 状态。

Session item 展示：

1. session 标题。
2. session 状态。
3. model 简写。
4. context percentage。
5. 最近更新时间。

### 7.3 Session Area

Session Area 包含：

1. Session Header：project、session title、branch、workspace mode、status。
2. Conversation Stream：用户消息、agent 输出、工具调用、审批、错误。
3. Execution Detail：可折叠展示底层运行事件和命令输出。
4. Approval Panel：等待用户确认的敏感操作。
5. Error Recovery：失败原因和可执行恢复操作。

### 7.4 Composer

Composer 是主要操作区，包含：

1. prompt 输入框。
2. send / stop / resume 操作。
3. model selector。
4. thinking level selector。
5. workspace mode selector：local / worktree。
6. branch selector。
7. token usage。
8. context limit percentage。
9. 当前 cwd / connection 摘要。

## 8. 核心用户流程

### 8.1 首次启动

1. 应用启动。
2. 加载应用内置 runtime。
3. 检测内置 runtime 是否可执行、版本是否兼容、RPC mode 是否可用。
4. 读取 pi provider、auth 和 model 可用性。
5. 读取 `pi-mono` 配置、环境变量、session 历史、skill / prompt / extension 信息。
6. 加载最近 project 和可恢复 session。
7. 若没有可用 provider/model，展示配置引导，不阻断项目浏览和历史查看。

验收：

1. 用户无需额外安装底层 agent 即可进入可用状态。
2. 用户能看到最近项目或创建/打开项目入口。
3. 不要求用户重复配置已经存在于 `pi-mono` 的设置。
4. 无可用 provider/model 时，用户能明确看到缺失原因和下一步配置入口。

### 8.1.1 Provider/Auth Onboarding

1. 应用通过 `pi-mono` 查询可用 models。
2. 若存在可用 model，默认选中 pi 当前默认模型或第一个可用模型。
3. 若不存在可用 model，展示 provider 配置引导。
4. 引导应说明可用认证来源：`~/.pi/agent/auth.json`、环境变量、`models.json`、pi OAuth/subscription。
5. 引导应提供打开配置目录、复制环境变量示例、重新检测入口。
6. 若 RPC 支持 provider 登录，Pig Studio 可以提供 GUI 登录入口；若不支持，则仅提供清晰外部引导。

验收：

1. Pig Studio 不保存 API key / OAuth token 到自己的数据库。
2. 已经通过 pi 配置过 provider 的用户无需重复配置。
3. 新用户能知道为什么暂时不能发送 prompt。
4. 用户完成 pi provider 配置后，点击重新检测即可看到可用 model。

### 8.2 创建或打开 Project

1. 用户点击 `New Project` 或 `Open Project`。
2. 用户选择连接类型：local / WSL / SSH。
3. 用户选择或输入项目路径。
4. 应用校验路径可访问性、Git 状态和当前 runtime 可运行性。
5. Project 进入 Sidebar。

验收：

1. local project 能直接选择本地目录。
2. WSL project 能识别 distro 与 Linux path。
3. SSH project 基于系统 `ssh` / `ssh-agent` / `ssh config` 建立连接。
4. SSH project 能保存 host、user、port、remote path 等非敏感连接元数据，但不保存密码、私钥或 passphrase。
5. 非 Git project 仍可创建 session。

### 8.3 创建 Session

1. 用户在 project 下点击新建 session。
2. 用户选择 workspace mode：local 或 worktree。
3. 若选择 worktree，用户选择 base branch 和目标 branch。
4. 应用按策略创建或绑定 worktree。
5. 调用 `pi-mono` RPC / SDK 创建底层 session。
6. GUI session 与底层 session id 建立索引。

验收：

1. 一个 project 可创建多个 session。
2. session 必须能被恢复。
3. worktree 失败时给出明确原因和退回 local 的操作。
4. session 历史不应成为 GUI 私有数据。

### 8.4 与 Agent 对话

1. 用户在 composer 输入 prompt。
2. 用户可在发送前调整 model 和 thinking level。
3. 用户发送 prompt。
4. Session Area 实时展示用户消息、agent 输出、工具调用、审批请求和错误。
5. 底部持续更新 token usage 与 context limit percentage。

验收：

1. 发送 prompt 后能看到 agent 响应流。
2. 运行中可停止。
3. 等待审批时输入区状态明确。
4. token 和 context 数据不会遮挡输入操作。

### 8.5 切换模型和 Thinking Level

1. GUI 从 `pi-mono` 配置读取可用 model 和 thinking level。
2. 用户在 composer 控件中切换。
3. 切换结果写回或传递给 `pi-mono` 的 session/run 配置。
4. UI 展示当前生效配置。

验收：

1. 可选项来自底层配置，不硬编码独立模型列表。
2. 切换后下一次 run 使用新配置。
3. 如果底层不支持运行中切换，UI 必须明确提示“下一次消息生效”。

### 8.6 Worktree 与 Branch

1. Git project 支持 branch selector。
2. local 模式下，branch selector 反映当前项目目录分支。
3. worktree 模式下，用户可为 session 创建独立 branch/worktree。
4. worktree 创建策略参考 `wt` / `worktrunk`。
5. UI 显示当前 workspace path、base branch、active branch。

验收：

1. Git project 可切换分支。
2. 非 Git project 隐藏或禁用 branch/worktree 控件。
3. worktree 状态在 session header 和 composer 附近可见。

### 8.7 恢复历史 Session

1. 应用启动时读取 `pi-mono` session 历史。
2. GUI 将 session 归属到已知 project。
3. 用户点击历史 session。
4. 应用恢复底层 session 绑定和 GUI 展示状态。
5. 如果恢复失败，显示原因和“基于上下文新建 session”入口。

验收：

1. 关闭应用后重新打开，历史 session 仍可见。
2. CLI 创建的 session 若能识别 project，应出现在 GUI 中。
3. GUI 创建的 session 不应阻断 CLI 继续使用。

## 9. 功能需求

### FR-1 内置 Runtime 检测

系统必须内置可执行的 `pi-mono` / pi coding agent runtime，并检测内置 runtime 是否可用。

必须展示：

1. 检测状态。
2. 可执行路径。
3. 版本信息。
4. 配置目录。
5. 失败原因。
6. 重新检测入口。
7. RPC mode 可用性。

默认行为：

1. 首次启动默认使用 `bundled` runtime。
2. 用户不需要在系统 PATH 中安装 `pi` / `pi-mono`。
3. v0.1 不提供外部 runtime path 配置。
4. 内置 runtime 不可用时，应展示修复、重新下载或重新安装 Pig Studio 的明确入口。

打包要求：

1. 应用包必须包含目标平台可执行 runtime 或等价 sidecar。
2. 如果 runtime 依赖 Node.js，则 Node.js 运行环境必须由应用包提供或编译进 runtime，不得要求用户自行安装。
3. 应用启动时不得通过 npm/bun 临时安装 runtime 作为常规路径。

### FR-2 底层状态共享

系统必须与 `pi-mono` / pi coding agent 共享核心状态。

必须共享：

1. session 历史。
2. 环境变量。
3. 配置。
4. model 配置。
5. thinking level 配置。
6. skill / prompt / extension。
7. provider auth 状态。

系统不得将 GUI 配置作为底层配置的替代品。GUI 配置只能作为展示索引和用户偏好。

### FR-2.1 Provider/Auth 复用

系统必须复用 pi 的 provider、model 和认证机制。

必须支持读取：

1. pi 内置 provider 和 model registry。
2. `~/.pi/agent/auth.json` 中已保存的认证状态。
3. 环境变量中的 provider API key。
4. `~/.pi/agent/models.json` 中的 custom provider / custom model。
5. pi 支持的 OAuth/subscription 登录状态。

必须展示：

1. 当前是否存在可用 model。
2. 当前默认 model。
3. 可用 provider/model 列表。
4. 无可用 model 时的配置引导。
5. 认证和 models 配置路径。

不得：

1. 在 Pig Studio 数据库保存 API key、OAuth token 或 provider secret。
2. 自建独立 provider registry。
3. 将 Pig Studio 的 model 配置作为 pi 配置的替代品。

### FR-2.2 Runtime 集成协议

v0.1 优先通过 `pi --mode rpc` 集成底层 agent。

必须覆盖：

1. prompt / steer / follow-up / abort。
2. get_state / get_messages / get_session_stats。
3. get_available_models / set_model。
4. set_thinking_level / cycle_thinking_level。
5. switch_session / new_session / fork。
6. get_commands，用于展示 skill / prompt / extension 命令入口。
7. extension UI request / response，用于审批、选择、输入等 GUI 交互。

SDK sidecar 可以作为后续增强路径，但不应成为 v0.1 基础可用性的前置条件。

### FR-3 Project 管理

系统必须允许用户创建或打开 project。

Project 必须支持：

1. local。
2. WSL。
3. SSH。

SSH 支持范围：

1. v0.1 只支持基于系统 `ssh`、`ssh-agent` 和 `ssh config` 的认证。
2. Pig Studio 不保存 SSH 密码、私钥或 passphrase。
3. SSH 连接失败时，应展示底层 ssh 错误和下一步排查建议。
4. 密码托管、密钥托管、跳板机向导和复杂凭证管理不进入 v0.1。

Project 必须保存：

1. project id。
2. project name。
3. connection type。
4. root path 或 remote path。
5. 最近打开时间。
6. 可选 pinned 状态。

### FR-4 Session 管理

系统必须允许 project 下创建多个 session。

Session 必须支持：

1. 创建。
2. 打开。
3. 恢复。
4. 重命名。
5. 停止运行。
6. 删除或归档 GUI 索引。

Session 必须关联：

1. project。
2. `pi-mono` session id / session file。
3. workspace mode。
4. cwd。
5. branch。
6. model。
7. thinking level。

### FR-5 主对话区

系统必须提供可与 agent 对话的主交互区。

主交互区必须展示：

1. 用户输入。
2. agent 输出。
3. 工具调用。
4. 审批请求。
5. 错误。
6. 运行状态。
7. 可折叠执行细节。

### FR-6 Composer 控件

Composer 必须支持：

1. 输入 prompt。
2. 发送消息。
3. 停止运行。
4. 选择 model。
5. 选择 thinking level。
6. 选择 workspace mode。
7. 选择或查看 branch。
8. 查看 token usage。
9. 查看 context limit percentage。

### FR-7 Worktree

系统必须支持 Git project 的 worktree session。

必须支持：

1. 创建 worktree。
2. 绑定已有 worktree。
3. 显示 worktree path。
4. 显示 base branch 和 active branch。
5. 处理创建失败。
6. 允许回退 local 模式。

Worktree 创建逻辑可参考 `wt` / `worktrunk` 的用户体验和目录策略，但具体实现需要适配 Pig Studio 的跨平台 project 模型。

### FR-8 Branch 切换

系统必须为 Git project 提供 branch 状态展示。

应该支持：

1. 查看当前 branch。
2. 切换 branch。
3. 为 worktree session 创建新 branch。
4. 显示未提交变更风险。

### FR-9 Token 与 Context 可见性

系统必须展示 token usage 和 context limit percentage。

显示位置：

1. Composer 附近必须有 compact 状态。
2. Session header 或侧栏 session item 可展示摘要。
3. 详细数据可放入 run detail 或 inspector。

必须支持的状态：

1. 正常。
2. 接近上限。
3. 已超过推荐阈值。
4. 底层暂不可用。

### FR-10 设置与扩展入口

系统必须提供设置入口，用于查看和打开底层共享能力。

设置中必须包括：

1. 当前 runtime 状态。
2. runtime 可执行路径与版本。
3. provider/model 可用状态。
4. 当前默认 model。
5. auth 配置路径。
6. models 配置路径。
7. 环境变量摘要。
8. skill 目录。
9. prompt 目录。
10. extension 目录。
11. 重新检测。

## 10. 非功能需求

### 10.1 性能

1. 应用启动后 2 秒内显示基础 UI。
2. 最近 project 和 session 列表应优先显示，底层深度扫描可异步完成。
3. 对话流追加不应造成明显卡顿。
4. token/context 数据更新不应阻塞输入。

### 10.2 可靠性

1. 底层 agent 不可用时，GUI 应进入只读或诊断模式。
2. SSH/WSL 连接失败时，不应影响 local project。
3. session 恢复失败必须可见。
4. worktree 创建失败必须可恢复。
5. 内置 runtime 是基础能力，不能因为用户未安装 CLI、Node、Bun 或 npm package 而不可用。

### 10.3 安全

1. 不在本地数据库保存 SSH 密码、token、API key。
2. 环境变量展示必须默认隐藏敏感值。
3. 审批请求必须明确展示影响范围。
4. destructive command 必须经过确认。
5. Provider credentials 由 pi 管理，Pig Studio 不复制、不迁移、不导出。
6. SSH credentials 由系统 ssh/ssh-agent/ssh config 管理，Pig Studio 不复制、不迁移、不导出。

### 10.4 跨平台

1. macOS、Windows、Linux 都应支持 local project。
2. Windows 应优先支持 WSL project。
3. SSH project 应跨平台可用。
4. 路径展示必须保留原始平台语义。

### 10.5 可维护性

1. UI 层不得直接操作底层 agent 存储。
2. `pi-mono` 集成必须通过 adapter/port 隔离。
3. Project、Session、Connection、Workspace Mode 应有明确领域模型。
4. 运行事件应保留结构化事件流，便于恢复和调试。
5. RPC 客户端、事件映射、extension UI 协议处理应拆分清楚，避免 UI 直接依赖进程协议细节。

## 11. 数据边界

Pig Studio 可以持久化：

1. GUI project 索引。
2. project connection metadata。
3. session 与 `pi-mono` session id / session file 的映射。
4. 最近打开时间。
5. pinned 状态。
6. GUI 展示偏好。
7. 非敏感运行摘要缓存。

Pig Studio 不应持久化：

1. SSH 密码。
2. API key。
3. provider token。
4. 完整环境变量敏感值。
5. 与 `pi-mono` session 存储重复的大型历史内容，除非底层不提供恢复能力。

## 12. 建议数据模型

### 12.1 projects

字段：

1. `id`
2. `name`
3. `connection_id`
4. `root_path`
5. `kind`：local / git / non_git
6. `pinned`
7. `last_opened_at`
8. `created_at`
9. `updated_at`

### 12.2 connections

字段：

1. `id`
2. `type`：local / wsl / ssh
3. `label`
4. `host`
5. `user`
6. `wsl_distro`
7. `base_path`
8. `metadata_json`
9. `created_at`
10. `updated_at`

说明：敏感凭证不进入该表。

### 12.3 sessions

字段：

1. `id`
2. `project_id`
3. `agent_session_id`
4. `title`
5. `status`
6. `workspace_mode`
7. `cwd`
8. `worktree_path`
9. `base_branch`
10. `active_branch`
11. `model`
12. `thinking_level`
13. `last_context_percentage`
14. `last_token_usage_json`
15. `last_run_at`
16. `created_at`
17. `updated_at`

### 12.4 events

字段：

1. `id`
2. `session_id`
3. `agent_event_id`
4. `seq`
5. `type`
6. `payload_json`
7. `created_at`

说明：events 用于 GUI 恢复和展示缓存，底层 session 历史仍以 `pi-mono` 为准。

### 12.5 app_settings

字段：

1. `key`
2. `value_json`
3. `updated_at`

## 13. UI 设计要求

### 13.1 整体风格

1. 参考 Linear app。
2. 使用浅色主题作为默认主题。
3. 侧栏窄而清晰。
4. 主区以内容为中心，减少装饰。
5. 使用细边框、浅灰背景和克制蓝色强调。
6. 不使用强烈渐变、玻璃拟态、大面积投影和营销式 hero。

### 13.2 Sidebar

1. 宽度保持在桌面工具合理范围。
2. Project 和 session 列表应支持较高信息密度。
3. Active item 仅用浅色背景和左侧/文字强调，不使用强装饰。
4. 状态 badge 小而可读。

### 13.3 Composer

1. 输入框应始终可见或易于回到。
2. model、thinking level、workspace mode、branch、token/context 作为紧凑 controls 展示。
3. 运行中状态应明确，但不遮挡输入区。
4. 停止和审批操作应清楚可见。

### 13.4 Session Header

必须展示：

1. project name。
2. session title。
3. connection type。
4. workspace mode。
5. branch。
6. status。
7. context percentage 摘要。

### 13.5 Token/Context 展示

建议使用 compact status bar：

1. `Tokens: 12.4k`
2. `Context: 42%`
3. `Model: xxx`
4. `Thinking: medium`

当 context 超过阈值时，使用轻量 warning，而不是弹窗阻断。

## 14. 状态与错误

### 14.1 Agent Runtime 状态

状态：

1. `BundleMissing`：应用包内未找到 runtime。
2. `BundleInvalid`：runtime 文件存在但不可执行或结构不完整。
3. `VersionMismatch`：runtime 版本不满足 Pig Studio v0.1 要求。
4. `RpcUnavailable`：runtime 可执行，但 RPC mode 不可用。
5. `ConfigError`：runtime 可用，但配置读取失败。
6. `Ready`：内置 runtime 和 RPC mode 可用。

### 14.2 Session 状态

状态：

1. `Idle`
2. `Running`
3. `WaitingApproval`
4. `Stopped`
5. `Failed`
6. `Recovering`
7. `Unrecoverable`

### 14.3 Project 连接状态

状态：

1. `Connected`
2. `Disconnected`
3. `AuthRequired`
4. `PathMissing`
5. `Unsupported`

### 14.4 错误展示原则

1. 错误应靠近发生位置展示。
2. 必须给出下一步操作。
3. 技术细节默认折叠。
4. 不用泛化文案掩盖底层失败原因。

## 15. 验收标准

v0.1 重做版完成时，应满足：

1. 用户打开应用后无需额外安装底层 agent，即可看到内置 runtime 可用状态。
2. 用户能看到 pi provider/model/auth 状态。
3. 无可用 provider/model 时，用户能看到清晰配置引导，且仍可浏览 project 和历史索引。
4. 已配置 pi provider 的用户无需重复配置即可开始对话。
5. 用户能创建或打开 local project。
6. 用户能创建或打开 WSL project。
7. 用户能配置或打开 SSH project 入口。
8. 一个 project 下能创建多个 session。
9. session 历史与 `pi-mono` 共享，而不是 GUI 私有历史。
10. 用户能在主区与 agent 对话。
11. 用户能在 composer 附近切换 model。
12. 用户能在 composer 附近切换 thinking level。
13. 用户能选择 local 或 worktree session。
14. Git project 能显示并切换 branch。
15. UI 能展示 token usage。
16. UI 能展示 context limit percentage。
17. 运行中、等待审批、失败和恢复失败都有明确状态。
18. 视觉风格接近 Linear：简洁、克制、信息密度合理。

## 16. 实施优先级

优先级用于拆分 v0.1 实施顺序，不表示验收范围缩水。P0 是最小可运行切片，用于尽早验证内置 runtime、RPC、provider onboarding 和 local project 主链路；P0 + P1 才构成 v0.1 完整验收范围。P2 是 v0.1 后的增强项，除非后续明确调整范围。

### P0

1. 内置 `pi-mono` runtime 打包与健康检查。
2. RPC adapter 与事件映射。
3. provider/model/auth 状态读取和 onboarding。
4. local project 创建/打开。
5. session 创建/恢复。
6. 主对话区。
7. model selector。
8. thinking level selector。
9. token/context 展示。
10. Linear 风格 shell。

### P1

1. WSL project。
2. SSH project。
3. worktree session。
4. branch selector。
5. session 历史共享增强。
6. skill / prompt / extension 入口。

### P2

1. 更完整的 SSH 凭证体验。
2. session 搜索。
3. 多 project 快速切换。
4. run detail inspector。
5. token/context 趋势图。

## 17. 待定问题

1. 内置 runtime 是直接打包 `pi` binary，还是使用 Node sidecar + SDK；两者都不得要求用户安装额外依赖。
2. v0.1 是否只支持 RPC mode，还是同时保留旧 CLI 输出解析作为兼容 fallback？
3. RPC v0.1 是否提供完整 provider login/logout 命令；如果不提供，Pig Studio v0.1 仅做配置引导。
4. model 与 thinking level 是全局配置、session 配置，还是每次 run 的临时参数？
5. WSL project 是否需要支持多个 distro 的自动发现？
6. worktree 目录策略是否直接采用 `wt` / `worktrunk`，还是仅借鉴交互逻辑？
7. GUI 删除 session 时，是删除底层 session，还是只删除 GUI 索引？
8. skill / prompt / extension 的编辑能力是否进入 v0.1，还是只提供打开入口？

## 18. 成功标准

Pig Studio v0.1 重做版成功的标准是：用户可以把它当作 `pi-mono` / pi coding agent 的主要桌面入口使用，同时仍保留与 CLI 和底层 agent 的一致性。

用户应能完成以下闭环：

1. 启动应用。
2. 无需额外安装即可确认内置 runtime 可用。
3. 复用已有 pi provider 认证，或看到清晰 provider 配置引导。
4. 打开 local / WSL / SSH project。
5. 创建或恢复 session。
6. 选择 model 和 thinking level。
7. 选择 local 或 worktree 工作模式。
8. 与 agent 对话。
9. 查看 token/context 状态。
10. 处理审批和错误。
11. 关闭后再次打开并恢复历史工作。
