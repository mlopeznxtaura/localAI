# LocalAI (Intent -> Software) — Agent Notes

## What this repo is

Local-first software generation:

- `intent-to-software/` is the 6-step stateless pipeline (engine)
- `localAI-app/` is the Tauri desktop UI (demo/submission surface)

## Non-negotiables

- No “one-shot” codegen: Step 6 must build **one task at a time** and run that task’s test before moving on.
- No secrets committed: `.env*`, tokens, and generated output are always ignored.

## Stateless worker architecture

Every step follows:

`read input file -> call Gemma once -> write output file -> exit`

No hidden memory across steps. The filesystem is the state.

## GitHub auth + export

The UI requires GitHub Device Flow so exports go to the **current user’s** GitHub account (public or private repos).

- OAuth Client ID is embedded in the app build.
- Token is stored locally (ignored by git) under `intent-to-software/.localai/`.
- Export uses the GitHub CLI (`gh`) + git to create and push the repo.

## Where files go

- Generated project: `intent-to-software/output/`
- Build metadata: `intent-to-software/output/build_manifest.json`
- Last backend error: `intent-to-software/ui_last_error.txt`

## Environment bootstrap

- System/hardware/path config is in `nxtAURA.env` (untracked, local only).
- Load it at session start so the LLM knows OS, WSL, GPU, Ollama endpoints, repo paths, and runtime versions.

## Active blockers

- WSL → `172.30.80.1:11434` times out; use `127.0.0.1:11434` inside WSL (or reconfigure Ollama to listen on `0.0.0.0` if you truly need cross-host access).

## Cloud credits (time-sensitive)

- IBM Cloud: credits balance 198.19 (expires 2026-04-27) + 18000 (expires 2026-09-26); month-to-date billable cost 1.8057 (USD) on Code Engine
- DashScope / Alibaba: TODO
- AWS: TODO
- Azure: TODO

## Architecture notes

- Agent orchestration: flat-file state machine (`task_status.json` as ground truth)
- Repo-wide write mutex pattern
- LLM = stateless worker, not self-orchestrating manager
- Spawn-point pattern via `AGENTS.md`
- CEO app: Tauri + React, live agent loop, Ollama-backed coding model
- Qvrm: agent trust protocol, deployed via IBM Cloud Code Engine
- AURA: AI-native video editor (moment graph, intent-driven)

## What Claude should NEVER do in this repo

- No stubs or placeholders — complete implementations only
- No assumptions about file paths — always confirm first
- No generating code that touches `task_status.json` schema without showing the diff
- No modifying `AGENTS.md` without explicit instruction

