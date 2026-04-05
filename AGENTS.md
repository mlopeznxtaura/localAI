# LocalAI (Intent -> Software) — Agent Notes

## What this repo is

Local-first software generation:

- `intent-to-software/` is the 6-step stateless pipeline (engine)
- `step7_ui/` is the Tauri desktop UI (demo/submission surface)

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

