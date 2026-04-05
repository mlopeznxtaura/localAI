# NEXTAURA_ENV

Paste this at the start of any Claude dev session.  
Keep it updated. The more accurate this is, the less broken code you get.

Last updated: 2026-04-04 (America/Los_Angeles)  
Source: `NEXTAURA ENV.pdf`

## SYSTEM

- Host OS: Microsoft Windows 11 Pro (10.0.26200, Build 26200)
- WSL Version: WSL 2 (WSL 2.5.9.0; kernel 6.6.87.2-1)
- WSL Distro: Ubuntu 24.04.4 LTS
- Windows username: archi
- WSL username: archi
- Windows host IP from WSL (default gateway): 172.30.80.1
- Ollama endpoint (WSL local): 127.0.0.1:11434 (reachable)
- Ollama endpoint (Windows local): 127.0.0.1:11434 (reachable)

## HARDWARE

- CPU: AMD Ryzen 9 7950X 16-Core Processor (16C/32T)
- GPU: NVIDIA GeForce RTX 5090 (also AMD Radeon(TM) Graphics iGPU)
- RAM: 136,516,042,752 bytes (~127.14 GiB / 128 GB)
- CUDA Version (nvidia-smi): 13.1
- NVIDIA Driver Version (nvidia-smi): 591.74

## LANGUAGE RUNTIMES (WSL)

- Rust: rustc 1.94.1 (e408947bf 2026-03-25)
- Cargo: cargo 1.94.1 (29ea6fb6a 2026-03-24)
- Node: v22.22.2
- Python: Python 3.12.3

## FRAMEWORKS & TOOLS

- Tauri Version: 2.0.0 (Rust `tauri`/`tauri-build`; JS `@tauri-apps/api`/`@tauri-apps/cli` are `^2.0.0`)
- React / Vite: React `^19.0.0`, Vite `^6.0.0`
- Ollama Version (WSL): 0.20.0
- Ollama Models Pulled (WSL):
  - gemma4:26b-optimized
  - VladimirGav/gemma4-26b-16GB-VRAM:latest
  - nemoclaw not a model (tool): `nemoclaw v0.0.4` at `/home/archi/.local/bin/nemoclaw`
  - nemotron-cascade-2:30b
  - gemma4:26b
- GitHub Handle: 1archit3ct1
- Git default branch: main

## KEY PATHS (WSL)

- Repo root: `/mnt/d/NextAura/projects2/localAI`
- Intent → Software pipeline: `/mnt/d/NextAura/projects2/localAI/intent-to-software`
- CEO Agent app: `/mnt/d/NextAura/projects/CEO/NEXTaura-ceo-agent`
- Orchestrator / agent loop: `/mnt/d/NextAura/projects/CEO/NEXTaura-ceo-agent/agent_runner.sh` (plus `scripts/scheduler_tick.py`)
- task_status.json: `/mnt/d/NextAura/projects/CEO/NEXTaura-ceo-agent/task_status.json`
- AGENTS.md (spawn-point): `/mnt/d/NextAura/orchestrator/SPAWN/STOP/AGENTS.md`
- Ollama scripts dir: `/mnt/d/NextAura/projects/CEO/NEXTaura-ceo-agent/scripts`

## KEY PATHS (Windows)

- Repo root (Windows side): `D:\NextAura\projects2\localAI`
- CEO Agent app: `D:\NextAura\projects\CEO\NEXTaura-ceo-agent`

## ACTIVE BLOCKERS

- WSL → `172.30.80.1:11434` times out; use `127.0.0.1:11434` inside WSL (or reconfigure Ollama to listen on `0.0.0.0` if you truly need cross-host access).
- (Resolved) “nemoclaw not found in WSL PATH” — it exists at `/home/archi/.local/bin/nemoclaw`.

## KNOWN WORKING

- WSL local Ollama endpoint reachable at `127.0.0.1:11434`
- Windows local Ollama endpoint reachable at `127.0.0.1:11434`
- Intent-to-Software pipeline scripts exist under `intent-to-software/` (`step1_compress.py` … `step6_build.py`)

## CLOUD CREDITS (time-sensitive)

- DashScope / Alibaba: TODO
- IBM Cloud: credits balance 198.19 (expires 2026-04-27) + 18000 (expires 2026-09-26); month-to-date billable cost 1.8057 (USD) on Code Engine
- AWS: latest cost report (2026-04-01 → 2026-04-02) blended cost 0 USD (estimated)
- Azure: `azure_budget.json` is empty (budget/credits not captured)

## CURRENT PROJECT FOCUS

<!-- One sentence. What are you about to ask Claude to build or fix? -->

## ARCHITECTURE NOTES

- Agent orchestration: flat-file state machine (`task_status.json` as ground truth)
- Repo-wide write mutex pattern
- LLM = stateless worker, not self-orchestrating manager
- Spawn-point pattern via `AGENTS.md`
- CEO app: Tauri + React, live agent loop, Ollama-backed coding model
- Qvrm: agent trust protocol, deployed via IBM Cloud Code Engine
- AURA: AI-native video editor (moment graph, intent-driven)

## WHAT CLAUDE SHOULD NEVER DO IN THIS REPO

- No stubs or placeholders — complete implementations only
- No assumptions about file paths — always confirm first
- No generating code that touches `task_status.json` schema without showing the diff
- No modifying `AGENTS.md` without explicit instruction

## TO UPDATE (commands)

- `rustc --version`
- `node --version`
- `python3 --version`
- `cargo --version`
- `ollama --version`
- `ollama list`
- `cat /etc/os-release | grep VERSION`
