import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import "./App.css";

type StepStatus = "waiting" | "running" | "done" | "failed";

type Step = {
  id: number;
  label: string;
  status: StepStatus;
};

const BASE_STEPS: Step[] = [
  { id: 1, label: "Compressing intent", status: "waiting" },
  { id: 2, label: "Generating interface", status: "waiting" },
  { id: 3, label: "Parsing features", status: "waiting" },
  { id: 4, label: "Mapping dependencies", status: "waiting" },
  { id: 5, label: "Planning build", status: "waiting" },
  { id: 6, label: "Writing software", status: "waiting" },
];

type Phase = "idle" | "building" | "done" | "error";

function App() {
  const [prompt, setPrompt] = useState("");
  const [phase, setPhase] = useState<Phase>("idle");
  const [steps, setSteps] = useState<Step[]>(BASE_STEPS);
  const [outputPath, setOutputPath] = useState("");
  const [errorText, setErrorText] = useState("");
  const [repoName, setRepoName] = useState(() => {
    return "localai-output";
  });
  const [visibility, setVisibility] = useState<"public" | "private">("public");
  const [exportStatus, setExportStatus] = useState<"idle" | "running" | "done" | "error">("idle");
  const [exportMsg, setExportMsg] = useState("");

  const [ghAuthed, setGhAuthed] = useState<boolean>(false);
  const [ghPhase, setGhPhase] = useState<"idle" | "starting" | "waiting" | "success" | "error">("idle");
  const [ghUserCode, setGhUserCode] = useState("");
  const [ghVerifyUrl, setGhVerifyUrl] = useState("");
  const [ghMsg, setGhMsg] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (phase === "idle") textareaRef.current?.focus();
  }, [phase]);

  useEffect(() => {
    (async () => {
      try {
        const status = await invoke<{ authenticated: boolean }>("github_status");
        setGhAuthed(Boolean(status?.authenticated));
      } catch {
        setGhAuthed(false);
      }
    })();
  }, []);

  const markStep = (id: number, status: StepStatus) => {
    setSteps((prev) => prev.map((s) => (s.id === id ? { ...s, status } : s)));
  };

  const handleBuild = async () => {
    if (!prompt.trim()) return;
    setPhase("building");
    setSteps(BASE_STEPS.map((s) => ({ ...s, status: "waiting" })));
    setOutputPath("");
    setErrorText("");

    const toMessage = (err: unknown) => {
      if (typeof err === "string") return err;
      if (err && typeof err === "object" && "message" in err) return String((err as any).message);
      try {
        return JSON.stringify(err);
      } catch {
        return String(err);
      }
    };

    for (let i = 1; i <= 6; i++) {
      markStep(i, "running");
      try {
        await invoke("run_step", { step: i, prompt });
        markStep(i, "done");
      } catch (e) {
        markStep(i, "failed");
        setErrorText(toMessage(e));
        setPhase("error");
        return;
      }
    }

    const path = await invoke<string>("get_output_path");
    setOutputPath(path);
    try {
      const suggested = await invoke<string>("suggest_repo_name");
      if (suggested) setRepoName(suggested);
    } catch {}
    setPhase("done");
  };

  const handleReset = () => {
    setPhase("idle");
    setPrompt("");
    setOutputPath("");
    setExportStatus("idle");
    setExportMsg("");
    setSteps(BASE_STEPS.map((s) => ({ ...s, status: "waiting" })));
  };

  const handleExport = async () => {
    if (!repoName.trim()) return;
    if (!ghAuthed) {
      setExportStatus("error");
      setExportMsg("GitHub not connected. Click “Connect GitHub” on the first screen (or run: gh auth login).");
      return;
    }
    setExportStatus("running");
    setExportMsg("");
    try {
      const out = await invoke<string>("export_output_to_github", { repo: repoName.trim(), visibility });
      setExportMsg(out || `Exported: ${repoName.trim()}`);
      setExportStatus("done");
    } catch (e) {
      const msg =
        typeof e === "string"
          ? e
          : e && typeof e === "object" && "message" in e
            ? String((e as any).message)
            : String(e);
      setExportMsg(msg);
      setExportStatus("error");
    }
  };

  const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

  const connectGitHub = async () => {
    setGhPhase("starting");
    setGhMsg("");
    try {
      const start = await invoke<{
        device_code: string;
        user_code: string;
        verification_uri: string;
        expires_in: number;
        interval: number;
      }>("github_device_start", { scopes: ["repo"] });

      setGhUserCode(start.user_code);
      setGhVerifyUrl(start.verification_uri);
      const intervalMs = Math.max(3, start.interval) * 1000;
      setGhPhase("waiting");

      // Open browser automatically (best-effort).
      try {
        await openUrl(start.verification_uri);
      } catch {}

      const deadline = Date.now() + Math.max(60, start.expires_in) * 1000;
      while (Date.now() < deadline) {
        const poll = await invoke<any>("github_device_poll", { device_code: start.device_code });
        if (poll?.status === "pending") {
          await sleep(intervalMs);
          continue;
        }
        if (poll?.status === "success") {
          await invoke("github_save_and_login", { access_token: poll.access_token });
          setGhAuthed(true);
          setGhPhase("success");
          setGhMsg("GitHub connected.");
          return;
        }
        if (poll?.status === "error") {
          setGhPhase("error");
          setGhMsg(`${poll.error}${poll.description ? `: ${poll.description}` : ""}`);
          return;
        }
        await sleep(intervalMs);
      }

      setGhPhase("error");
      setGhMsg("Timed out waiting for GitHub authorization.");
    } catch (e) {
      const msg =
        typeof e === "string"
          ? e
          : e && typeof e === "object" && "message" in e
            ? String((e as any).message)
            : String(e);
      setGhPhase("error");
      setGhMsg(msg);
    }
  };

  return (
    <div className="app">
      {phase === "idle" && (
        <div className="composer">
          <div className="wordmark">intent</div>
          <p className="tagline">Describe what you want to build.</p>
          <textarea
            ref={textareaRef}
            className="input"
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            placeholder="An app that tracks my daily water intake and reminds me every two hours..."
            rows={5}
            onKeyDown={(e) => {
              if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) handleBuild();
            }}
            disabled={!ghAuthed}
          />
          <button className="build-btn" onClick={handleBuild} disabled={!ghAuthed || !prompt.trim()}>
            Build
          </button>
          <p className="hint">Any language. Any idea. Runs entirely on your device.</p>
          {!ghAuthed && <div className="auth-msg">Connect GitHub to start. This enables export for your generated project.</div>}

          <div className="auth">
            <div className="auth-title">GitHub (required)</div>
            {ghAuthed ? (
              <div className="auth-ok">Connected</div>
            ) : (
              <div className="auth-row">
                <button className="auth-btn" onClick={connectGitHub} disabled={ghPhase === "starting" || ghPhase === "waiting"}>
                  {ghPhase === "starting" ? "Starting..." : ghPhase === "waiting" ? "Waiting..." : "Connect GitHub"}
                </button>
              </div>
            )}
            {ghPhase === "waiting" && ghUserCode && (
              <div className="auth-help">
                <div className="auth-code">Code: {ghUserCode}</div>
                <div className="auth-url">{ghVerifyUrl}</div>
              </div>
            )}
            {ghMsg && <div className="auth-msg">{ghMsg}</div>}
          </div>
        </div>
      )}

      {phase === "building" && (
        <div className="progress">
          <div className="wordmark">intent</div>
          <div className="steps">
            {steps.map((step) => (
              <div key={step.id} className={`step step--${step.status}`}>
                <span className="step-dot" />
                <span className="step-label">{step.label}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {phase === "done" && (
        <div className="done">
          <div className="wordmark">intent</div>
          <p className="done-msg">Your software is ready.</p>
          <p className="done-path">{outputPath}</p>
          <div className="done-actions">
            <button className="open-btn" onClick={() => invoke("open_output_folder", { path: outputPath })}>
              Open folder
            </button>
            <button className="reset-btn" onClick={handleReset}>
              Build something else
            </button>
          </div>

          <div className="export">
            <div className="export-title">Export to GitHub (optional)</div>
            <div className="export-row">
              <input
                className="export-input"
                value={repoName}
                onChange={(e) => setRepoName(e.currentTarget.value)}
                placeholder="repo-name or owner/repo"
              />
              <select className="export-select" value={visibility} onChange={(e) => setVisibility(e.currentTarget.value as any)}>
                <option value="public">public</option>
                <option value="private">private</option>
              </select>
              <button className="export-btn" onClick={handleExport} disabled={exportStatus === "running"}>
                {exportStatus === "running" ? "Exporting..." : "Export"}
              </button>
            </div>
            {exportMsg && <pre className="error-box">{exportMsg}</pre>}
          </div>
        </div>
      )}

      {phase === "error" && (
        <div className="done">
          <div className="wordmark">intent</div>
          <p className="done-msg">Something went wrong.</p>
          {errorText && <pre className="error-box">{errorText}</pre>}
          <button className="reset-btn" onClick={handleReset}>
            Try again
          </button>
        </div>
      )}
    </div>
  );
}

export default App;
