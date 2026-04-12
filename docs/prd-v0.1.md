# Berry Studio PRD v0.1

## 1. Product Definition

Berry Studio is a native cross-platform desktop app built on top of `pi-mono` for creating, running, and managing `agent sessions`.

The product is centered around the user's local projects and provides a focused workspace for interacting with agents through persistent sessions, execution history, approvals, and runtime settings.

## 2. Positioning

Berry Studio is an `agent session workspace`, not a general AI chat app and not a full IDE.

It is designed for users who want to work with agents inside real local projects through a native desktop application without depending on a web-based tech stack.

## 3. Product Goals

1. Let users open local projects and organize work by project.
2. Let users create and resume `agent sessions`.
3. Let users interact with an agent in an ongoing session with clear execution feedback.
4. Let users observe session activity, status, and approvals in one place.
5. Provide a native cross-platform desktop experience built around `pi-mono`.

## 4. Non-Goals

1. Not a multi-provider CLI aggregator.
2. Not an IDE replacement in the first version.
3. Not a cloud orchestration platform.
4. Not a team collaboration product in the first version.
5. Not a generic chatbot product.

## 5. Core Terms

### Project
A local folder opened by the user in Berry Studio.

### Agent
An agent capability provided and orchestrated by `pi-mono`.

### Agent Session
A persistent working session for an agent within a specific project.

### Run
A single execution cycle triggered by the user inside a session.

### Approval
A user decision for a sensitive action requested during a session run.

## 6. Target Users

### Primary Users
1. Developers working in local code repositories.
2. Users who want a dedicated desktop interface for agent-based work.
3. Users who need project-based session history and recovery.

### User Mindset
Users are not looking for a novelty chat interface. They want a reliable workspace that helps them start an agent session quickly, see what the agent is doing, and come back to unfinished work later.

## 7. Core User Scenarios

1. A user opens a local project and starts a new `agent session`.
2. A user sends prompts and iterates with the agent inside that session.
3. A user reviews execution progress, output, and approvals while the session is running.
4. A user closes the app and later resumes the same session from the project sidebar.
5. A user updates runtime settings for `pi-mono` and uses the new configuration in future sessions.

## 8. MVP Scope

### 8.1 Project Management
1. Add or open a local folder as a project.
2. Show projects in the left sidebar.
3. Persist the list of recent or pinned projects locally.

### 8.2 Session Management
1. Create a new `agent session` under a project.
2. List sessions under each project.
3. Open an existing session.
4. Persist session metadata and session history locally.
5. Rename and delete sessions.

### 8.3 Session Interaction
1. Show the conversation and execution stream in the main area.
2. Allow the user to send prompts in an active session.
3. Show session states such as idle, running, waiting for approval, completed, and failed.
4. Support continuous multi-turn interaction within a session.

### 8.4 Runtime Visibility
1. Display execution events in chronological order.
2. Show errors clearly.
3. Show approval requests and let users respond.
4. Preserve enough run history for session recovery and debugging.

### 8.5 Settings
1. Provide a settings entry in the lower area of the left sidebar.
2. Configure `pi-mono` runtime location and required environment settings.
3. Show runtime health or availability state.

### 8.6 Platform
1. Native desktop app.
2. Cross-platform support for macOS, Windows, and Linux.
3. No dependency on a web-based application shell as the primary product architecture.

## 9. Functional Requirements

### FR-1 Project Model
1. The system must allow users to add a local folder as a project.
2. The system must persist project metadata locally.
3. The system must show the project's sessions beneath that project in the sidebar.

### FR-2 Session Lifecycle
1. The system must allow creating a session from a project context.
2. The system must persist sessions across app restarts.
3. The system must support reopening a previous session.
4. The system must support deleting a session with confirmation.

### FR-3 Messaging and Runs
1. The system must allow the user to submit input to the current session.
2. The system must show in-progress output while a run is active.
3. The system must append the result to the session history.

### FR-4 Approvals
1. The system must surface approval requests during a run.
2. The system must let the user approve or reject the request.
3. The system must record the approval outcome in session history.

### FR-5 Persistence
1. The system must store project and session metadata locally.
2. The system must restore the previous state after restart.
3. The system must preserve enough event history to make a resumed session understandable.

### FR-6 Settings
1. The system must provide a settings screen.
2. The system must validate whether `pi-mono` is available.
3. The system must show configuration problems in a clear way.

## 10. Information Architecture

### Left Sidebar
1. Project list.
2. Expandable sessions under each project.
3. New session entry point.
4. Settings button in the lower-right area of the sidebar.

### Main Area
1. Session header.
2. Session conversation and event stream.
3. Input composer.

## 11. Page Layout

### Left Sidebar
- Hierarchy: `Project -> Session`
- Each project can expand to reveal its sessions.
- The settings button lives at the lower-right corner of the sidebar.

### Main Session Area
- Header: project name, session name, status.
- Body: conversation and execution history.
- Footer: input box, send action, run status.

## 12. UX Principles

1. Project-first navigation.
2. Session continuity over one-off chatting.
3. Clear execution state at all times.
4. Native, fast, and calm interaction.
5. Sensitive actions should always be visible and explicit.

## 13. Success Criteria for MVP

1. A user can open a project and create a session in under one minute.
2. A user can restart the app and successfully resume a previous session.
3. A user can understand whether a session is running, blocked, failed, or waiting for approval.
4. A user can manage all core session actions without leaving the app.

## 14. Open Questions

1. Should a project support multiple agent types in the future, even if v0.1 only uses `pi-mono`?
2. What exact approval categories need to exist in v0.1?
3. How much raw execution detail should be visible in the main session stream versus a secondary inspector view?
4. Should deleting a project remove local session history or only detach the project from the app?

## 15. Summary

Berry Studio v0.1 is a native desktop `agent session workspace` built on `pi-mono`.

Its MVP is intentionally narrow:
1. Open projects.
2. Create and resume agent sessions.
3. Talk to the agent.
4. Observe runs and approvals.
5. Manage runtime settings in a simple native desktop app.
