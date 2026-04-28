# Pig Studio

Pig Studio 是一个基于 `pi` / `pi-mono` 的原生桌面 `agent session workspace`。

当前仓库已经迁移为 `Rust Workspace + Dioxus Desktop` 架构，围绕 `Project -> Session` 工作流提供会话管理、实时运行事件流、审批、恢复与运行时设置的基础能力。

## 当前能力

- `Project -> Session` 桌面工作区与历史会话导航
- 像 Codex App 一样通过系统文件夹选择器添加项目，不需要手动输入项目路径
- 自动检测 Pi 二进制与配置目录，并支持文件选择器方式的自定义覆盖
- `pi` / `pi-mono` 输出的实时流式展示与 SQLite 事件持久化
- 审批请求展示、决策回写与时间线记录
- 启动恢复、运行对账、恢复失败后的只读降级
- `Blocked / Interrupted` 会话上的“基于当前上下文新建会话”入口
- Git worktree 与非 Git direct 模式双路径支持

## 技术栈

- Rust stable
- Dioxus Desktop
- Tailwind CSS v4 + coss UI 语义样式
- base-ui-dioxus -> coss-ui-dioxus -> Pig Studio UI
- SQLite
- Bun + Cargo

## 工作区结构

```text
pig-studio/
├── Cargo.toml
├── assets/
│   └── styles/
├── migrations/
├── crates/
│   ├── app-core/
│   ├── app-desktop/
│   ├── base-ui-dioxus/
│   ├── coss-ui-dioxus/
│   ├── domain/
│   ├── infra-pimono/
│   ├── infra-settings/
│   ├── infra-sqlite/
│   ├── shared-kernel/
│   └── ui-components/
└── docs/
```

## 常用命令

```bash
bun run build
bun run test
bun run format
bun run dev
bun run bundle:desktop
bun run verify:bundle
```

## 关键文档

- 产品需求：`docs/prd-v0.1.zh-CN.md`
- 技术架构：`docs/architecture-v0.1.zh-CN.md`
- 实施任务：`docs/tasks-v0.1.zh-CN.md`
- 手工验收：`docs/manual-qa-v0.1.zh-CN.md`
