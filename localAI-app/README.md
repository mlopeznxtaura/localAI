# `localAI-app` — Tauri UI for Intent → Software

This is the desktop "control surface" for the stateless pipeline in `../intent-to-software/`.

## Run

```powershell
cd D:\NextAura\projects2\localAI\localAI-app
npm install
npm run tauri dev
```

The UI calls Rust commands which execute the Python step scripts and updates the progress UI in real time.
