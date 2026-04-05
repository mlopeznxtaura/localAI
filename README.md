# NextAura LocalAI — Intent -> Software (Gemma 4 + Ollama)

Most AI coding tools require cloud APIs. This runs entirely on your hardware.

The LLM has **no memory** — every step reads a file, calls Gemma once, writes a file, and exits.  
That constraint **is the architecture**.

## Repo layout

- `intent-to-software/`: 6-step stateless pipeline (engine)
- `step7_ui/`: Tauri desktop UI (submission/demo surface)

## Run the UI (WSL)

Prereqs:
- Ollama running with a Gemma 4 model pulled
- Python venv bootstrapped for pipeline deps

```bash
cd /mnt/d/NextAura/projects2/localAI/intent-to-software
bash ./bootstrap_venv.sh

cd /mnt/d/NextAura/projects2/localAI/step7_ui
npm install
npm run tauri dev
```

Flow:
1) Connect GitHub (Device Flow) — required (so exports go to the user's GitHub account)
2) Paste a prompt and click **Build**
3) Open the generated output folder, or export to GitHub

Requirements for export:
- `gh` CLI installed and available in PATH
- `git` installed and available in PATH

GitHub behavior:
- Export writes to the currently authenticated user's GitHub account.
- Repos can be public or private.
- Default repo name is auto-suggested from the prompt/intent (slugified).
- Device-flow token is stored locally (gitignored) under `intent-to-software/.localai/`.

## Where files go

- Generated project: `intent-to-software/output/`
- Build metadata: `intent-to-software/output/build_manifest.json`
- Step 6 rolling context: `intent-to-software/output/PROJECT_CONTEXT.md`
- Last UI/backend error: `intent-to-software/ui_last_error.txt`

## Headless pipeline (no UI)

```bash
cd /mnt/d/NextAura/projects2/localAI/intent-to-software
. .venv/bin/activate
python run.py
```

