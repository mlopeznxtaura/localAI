Build Manual: Intent → Software

What We’re Building
A program that takes what someone wants in plain English and builds them working software. The user describes something in any language. The system figures out what it looks like, what it needs, and builds it file by file. Locally. No internet required. A beautiful interface sits on top so anyone can use it.

Before You Write a Single Line of Code
Open your WSL terminal.
Check Ollama is running:

ollama list


You should see gemma4. If not:

ollama pull gemma4


Make the project folder:

mkdir intent-to-software
cd intent-to-software


Install Python dependencies:

pip3 install requests beautifulsoup4


How to Think About This
Your brain is Gemma. Your short-term memory is the context window. Your notepad is files on disk.
The problem: Gemma forgets everything between calls. By step 7 it’s forgotten step 1.
The solution: after every step, write the result to a file. Start the next step fresh. Read the file instead of remembering.
This is the entire architecture. Every step is the same pattern:

read input file → call Gemma → write output file → done


The Pipeline

user_prompt.txt
↓
step1_compress.py → compressed_intent.txt
↓
step2_mockui.py → mock_ui.html
↓
step3_parse.py → features.json
↓
step4_dag.py → dag.json
↓
step5_tasks.py → tasks.json + tests.json
↓
step6_build.py → /output/ (the actual software)
↓
step7_ui/ → Tauri app (the control surface)


Step 1: Prompt Compression
What it does: Takes messy user input. Returns one clean sentence.
Why: Users don’t write clean instructions. Gemma works better with precise input. Clean it first before anything else.
Create step1_compress.py:

import requests

with open("user_prompt.txt", "r") as f:
raw_prompt = f.read()

def ask_gemma(system, user):
response = requests.post(
"http://172.30.80.1:11434/api/chat",
json={
"model": "gemma4",
"messages": [
{"role": "system", "content": system},
{"role": "user", "content": user}
],
"stream": False
}
)
return response.json()["message"]["content"]

system_prompt = """
You are a prompt compressor.
The user will give you a messy description of software they want to build.
Your job is to return ONE clean sentence that captures exactly what they want.
Nothing else. No explanation. No questions. Just the one sentence.
Respond in the same language the user wrote in.
"""

result = ask_gemma(system_prompt, raw_prompt)

with open("compressed_intent.txt", "w") as f:
f.write(result)

print("Step 1 complete.")
print("Compressed intent:", result)


Create user_prompt.txt with something messy:

i want like an app where i can track my expenses and like
maybe see charts and stuff and like be able to add categories
and maybe export it idk like something simple but useful


Run it:

python3 step1_compress.py
cat compressed_intent.txt


Step 2: Mock UI Generation
What it does: Takes the compressed intent. Generates an HTML file showing what the app should look like.
Why: We need to see the app before we build it. The mock becomes the source of truth for what features exist.
Create step2_mockui.py:

import requests

with open("compressed_intent.txt", "r") as f:
intent = f.read()

def ask_gemma(system, user):
response = requests.post(
"http://172.30.80.1:11434/api/chat",
json={
"model": "gemma4",
"messages": [
{"role": "system", "content": system},
{"role": "user", "content": user}
],
"stream": False
}
)
return response.json()["message"]["content"]

system_prompt = """
You are a UI designer.
The user will describe software they want.
Return a single complete HTML file showing what this app looks like.
Use clean HTML and inline CSS.
Include all buttons, forms, inputs, and sections the app would need.
Make it look real. Label every button and input clearly.
Respond in the same language as the intent.
Return ONLY raw HTML starting with <!DOCTYPE html>. No explanation. No markdown.
"""

result = ask_gemma(system_prompt, intent)

if "```html" in result:
result = result.split("```html")[1].split("```")[0]
elif "```" in result:
result = result.split("```")[1].split("```")[0]

with open("mock_ui.html", "w") as f:
f.write(result)

print("Step 2 complete.")
print("Mock UI written to mock_ui.html")


Run it:

python3 step2_mockui.py


Open mock_ui.html in a browser to see what was generated.

Step 3: Parse the UI
What it does: Reads the HTML mock. Pulls out every button, input, and function. Writes them as a structured list.
Why: We need a machine-readable feature list before we can plan the build.
Create step3_parse.py:

from bs4 import BeautifulSoup
import json
import requests

with open("mock_ui.html", "r") as f:
html = f.read()

soup = BeautifulSoup(html, "html.parser")

features = []

for button in soup.find_all("button"):
text = button.get_text(strip=True)
if text:
features.append({"type": "button", "label": text})

for input_tag in soup.find_all("input"):
label = (input_tag.get("placeholder") or
input_tag.get("name") or
input_tag.get("type", "input"))
features.append({"type": "input", "label": label})

for form in soup.find_all("form"):
features.append({"type": "form", "label": form.get("id") or "form"})

for heading in soup.find_all(["h1","h2","h3"]):
text = heading.get_text(strip=True)
if text:
features.append({"type": "section", "label": text})

def ask_gemma(system, user):
response = requests.post(
"http://172.30.80.1:11434/api/chat",
json={
"model": "gemma4",
"messages": [
{"role": "system", "content": system},
{"role": "user", "content": user}
],
"stream": False
}
)
return response.json()["message"]["content"]

system_prompt = """
You are a software analyst.
Look at this HTML and return a JSON array of features this app needs.
Each feature should have: name, description, inputs, outputs.
Return ONLY valid JSON. No explanation. No markdown.
"""

gemma_features_raw = ask_gemma(system_prompt, html)

try:
if "```json" in gemma_features_raw:
gemma_features_raw = gemma_features_raw.split("```json")[1].split("```")[0]
elif "```" in gemma_features_raw:
gemma_features_raw = gemma_features_raw.split("```")[1].split("```")[0]
gemma_features = json.loads(gemma_features_raw)
except:
gemma_features = []

output = {
"parsed_elements": features,
"gemma_features": gemma_features
}

with open("features.json", "w") as f:
json.dump(output, f, indent=2)

print("Step 3 complete.")
print(f"Found {len(features)} elements, {len(gemma_features)} features")


Run it:

python3 step3_parse.py
cat features.json


Step 4: Build the DAG
What it does: Takes the feature list. Works backwards from each feature to figure out what it depends on. Produces a dependency graph with build order.
Why: You can’t build a chart before you have data. You can’t have data before you have storage. The DAG figures out the correct order.
Create step4_dag.py:

import json
import requests

with open("features.json", "r") as f:
features = json.load(f)

def ask_gemma(system, user):
response = requests.post(
"http://172.30.80.1:11434/api/chat",
json={
"model": "gemma4",
"messages": [
{"role": "system", "content": system},
{"role": "user", "content": user}
],
"stream": False
}
)
return response.json()["message"]["content"]

system_prompt = """
You are a software architect.
Given a list of features, return a DAG showing what each feature depends on.
Work backwards from the most complex features to the most basic.
Return ONLY valid JSON:
{
"nodes": [
{
"id": "feature_name",
"description": "what it does",
"depends_on": ["other_feature"]
}
],
"build_order": ["first_to_build", "second_to_build"]
}
No explanation. No markdown. Just the JSON.
"""

result = ask_gemma(system_prompt, json.dumps(features))

try:
if "```json" in result:
result = result.split("```json")[1].split("```")[0]
elif "```" in result:
result = result.split("```")[1].split("```")[0]
dag = json.loads(result)
except:
print("Gemma returned invalid JSON. Raw output:")
print(result)
dag = {"nodes": [], "build_order": []}

with open("dag.json", "w") as f:
json.dump(dag, f, indent=2)

print("Step 4 complete.")
print("Build order:", dag.get("build_order", []))


Run it:

python3 step4_dag.py
cat dag.json


Step 5: Generate Tasks and Tests
What it does: Turns the DAG into an atomic task list. Writes a validation test for each task at the same time.
Why: This is the machine-executable plan. Each task is one file or one function. The tests verify completion before moving forward.
Create step5_tasks.py:

import json
import requests

with open("dag.json", "r") as f:
dag = json.load(f)

with open("compressed_intent.txt", "r") as f:
intent = f.read()

def ask_gemma(system, user):
response = requests.post(
"http://172.30.80.1:11434/api/chat",
json={
"model": "gemma4",
"messages": [
{"role": "system", "content": system},
{"role": "user", "content": user}
],
"stream": False
}
)
return response.json()["message"]["content"]

tasks_prompt = """
You are a project planner.
Given a DAG of features, return a tasks.json.
Each task must be atomic — one file or one function.
Return ONLY valid JSON array:
[
{
"id": "T001",
"title": "what to build",
"file": "exact/path/to/file.py",
"description": "exactly what this file should contain and do",
"depends_on": [],
"status": "pending"
}
]
No explanation. No markdown. Just the JSON array.
"""

tasks_raw = ask_gemma(tasks_prompt, json.dumps(dag))

try:
if "```json" in tasks_raw:
tasks_raw = tasks_raw.split("```json")[1].split("```")[0]
elif "```" in tasks_raw:
tasks_raw = tasks_raw.split("```")[1].split("```")[0]
tasks = json.loads(tasks_raw)
except:
print("Error parsing tasks. Raw:")
print(tasks_raw)
tasks = []

tests_prompt = """
You are a QA engineer.
For each task write a simple Python test that verifies it was completed correctly.
Tests should check the file exists and contains the right logic.
Return ONLY valid JSON array:
[
{
"task_id": "T001",
"test_code": "import os\\nassert os.path.exists('output/file.py')"
}
]
No explanation. No markdown. Just the JSON array.
"""

tests_raw = ask_gemma(tests_prompt, json.dumps(tasks))

try:
if "```json" in tests_raw:
tests_raw = tests_raw.split("```json")[1].split("```")[0]
elif "```" in tests_raw:
tests_raw = tests_raw.split("```")[1].split("```")[0]
tests = json.loads(tests_raw)
except:
print("Error parsing tests. Raw:")
print(tests_raw)
tests = []

with open("tasks.json", "w") as f:
json.dump(tasks, f, indent=2)

with open("tests.json", "w") as f:
json.dump(tests, f, indent=2)

print("Step 5 complete.")
print(f"Generated {len(tasks)} tasks and {len(tests)} tests")


Run it:

python3 step5_tasks.py
cat tasks.json
cat tests.json


Step 6: The Build Loop
What it does: Reads tasks one at a time. Asks Gemma to write the code. Saves the file. Runs the test. Passes — moves on. Fails — retries up to 3 times. Updates tasks.json after every task so if it crashes you can resume.
Why: This is the actual build. Everything before this was planning. This is execution.
Create step6_build.py:

import json
import requests
import os

def ask_gemma(system, user):
response = requests.post(
"http://172.30.80.1:11434/api/chat",
json={
"model": "gemma4",
"messages": [
{"role": "system", "content": system},
{"role": "user", "content": user}
],
"stream": False
}
)
return response.json()["message"]["content"]

def run_test(test_code):
try:
exec(test_code, {})
return True, None
except Exception as e:
return False, str(e)

def build_task(task, test_code, max_retries=3):
system_prompt = """
You are a senior software engineer.
Write complete working code for the task given.
No stubs. No placeholders. No TODOs. Complete code only.
Return ONLY the raw code. No explanation. No markdown.
"""
for attempt in range(max_retries):
print(f" Attempt {attempt + 1}...")

code = ask_gemma(system_prompt,
f"Task: {task['title']}\n"
f"File: {task['file']}\n"
f"Description: {task['description']}")

if "```" in code:
code = code.split("```")[1].split("```")[0]
if code.startswith("python\n"):
code = code[7:]

output_path = os.path.join("output", task['file'])
os.makedirs(
os.path.dirname(output_path) if os.path.dirname(output_path) else "output",
exist_ok=True
)

with open(output_path, "w") as f:
f.write(code)

passed, error = run_test(test_code)

if passed:
print(f" ✓ Passed")
return True
else:
print(f" ✗ Failed: {error}")

print(f" ✗ Failed after {max_retries} attempts")
return False

with open("tasks.json", "r") as f:
tasks = json.load(f)

with open("tests.json", "r") as f:
tests = json.load(f)

test_lookup = {t["task_id"]: t["test_code"] for t in tests}

os.makedirs("output", exist_ok=True)

results = []
for task in tasks:
if task.get("status") == "done":
print(f"Skipping {task['id']} (already done)")
continue

print(f"\nBuilding: {task['id']} — {task['title']}")

test_code = test_lookup.get(task["id"], "pass")
success = build_task(task, test_code)

task["status"] = "done" if success else "failed"
results.append({"id": task["id"], "success": success})

with open("tasks.json", "w") as f:
json.dump(tasks, f, indent=2)

print("\n--- Build Complete ---")
done = sum(1 for r in results if r["success"])
print(f"{done}/{len(results)} tasks completed")
print("Check /output/ for your software")


Run it:

python3 step6_build.py
ls output/


Step 7: The Tauri Interface
What it does: Wraps the entire pipeline in a beautiful desktop application. The user never sees a terminal. They see a clean interface, type what they want, watch it build in real time, and receive their software.
Why: This is the front door. This is what the judges see first. This is what a kid in Lagos sees. It has to be beautiful, quiet, and effortless.
First: Scaffold the Tauri App

cd ..
npm create tauri-app@latest intent-ui
cd intent-ui
npm install


Choose: React + TypeScript when prompted.
Install dependencies:

npm install


The Design Direction
Dark. Quiet. Confident. The input is the hero. Progress is ambient. The output reveal feels like something was created.
Replace src/App.tsx with:

import { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type Step = {
id: number;
label: string;
status: "waiting" | "running" | "done" | "failed";
};

const STEPS: Step[] = [
{ id: 1, label: "Compressing intent", status: "waiting" },
{ id: 2, label: "Generating interface", status: "waiting" },
{ id: 3, label: "Parsing features", status: "waiting" },
{ id: 4, label: "Mapping dependencies", status: "waiting" },
{ id: 5, label: "Planning build", status: "waiting" },
{ id: 6, label: "Writing software", status: "waiting" },
];

export default function App() {
const [prompt, setPrompt] = useState("");
const [phase, setPhase] = useState<"idle"|"building"|"done"|"error">("idle");
const [steps, setSteps] = useState<Step[]>(STEPS);
const [outputPath, setOutput] = useState("");
const [language, setLanguage] = useState("en");
const textareaRef = useRef<HTMLTextAreaElement>(null);

useEffect(() => {
if (phase === "idle") textareaRef.current?.focus();
}, [phase]);

const markStep = (id: number, status: Step["status"]) => {
setSteps(prev =>
prev.map(s => s.id === id ? { ...s, status } : s)
);
};

const handleBuild = async () => {
if (!prompt.trim()) return;
setPhase("building");
setSteps(STEPS.map(s => ({ ...s, status: "waiting" })));

try {
// Each step calls the Python script via Tauri backend
// and updates the UI as it progresses
for (let i = 1; i <= 6; i++) {
markStep(i, "running");
await invoke("run_step", { step: i, prompt });
markStep(i, "done");
}
const path = await invoke<string>("get_output_path");
setOutput(path);
setPhase("done");
} catch (e) {
setPhase("error");
}
};

const handleReset = () => {
setPhase("idle");
setPrompt("");
setOutput("");
setSteps(STEPS.map(s => ({ ...s, status: "waiting" })));
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
onChange={e => setPrompt(e.target.value)}
placeholder="An app that tracks my daily water intake and reminds me every two hours..."
rows={5}
onKeyDown={e => {
if (e.key === "Enter" && e.metaKey) handleBuild();
}}
/>
<button
className="build-btn"
onClick={handleBuild}
disabled={!prompt.trim()}
>
Build
</button>
<p className="hint">Any language. Any idea. Runs entirely on your device.</p>
</div>
)}

{phase === "building" && (
<div className="progress">
<div className="wordmark">intent</div>
<div className="steps">
{steps.map(step => (
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
<button className="open-btn" onClick={() =>
invoke("open_output_folder", { path: outputPath })
}>
Open folder
</button>
<button className="reset-btn" onClick={handleReset}>
Build something else
</button>
</div>
</div>
)}

{phase === "error" && (
<div className="done">
<div className="wordmark">intent</div>
<p className="done-msg">Something went wrong.</p>
<button className="reset-btn" onClick={handleReset}>Try again</button>
</div>
)}
</div>
);
}


Replace src/App.css with:

* {
margin: 0;
padding: 0;
box-sizing: border-box;
}

:root {
--bg: #0c0c0c;
--surface: #141414;
--border: #222;
--text: #e8e8e8;
--muted: #555;
--accent: #e8e8e8;
--green: #4ade80;
--red: #f87171;
font-family: -apple-system, "Inter", sans-serif;
}

body {
background: var(--bg);
color: var(--text);
height: 100vh;
overflow: hidden;
}

.app {
height: 100vh;
display: flex;
align-items: center;
justify-content: center;
}

/* ── Composer ─────────────────────────────── */
.composer {
width: 100%;
max-width: 560px;
padding: 0 24px;
display: flex;
flex-direction: column;
gap: 20px;
}

.wordmark {
font-size: 13px;
font-weight: 600;
letter-spacing: 0.15em;
text-transform: uppercase;
color: var(--muted);
}

.tagline {
font-size: 22px;
font-weight: 400;
color: var(--text);
line-height: 1.4;
}

.input {
width: 100%;
background: var(--surface);
border: 1px solid var(--border);
border-radius: 10px;
color: var(--text);
font-size: 15px;
line-height: 1.6;
padding: 16px;
resize: none;
outline: none;
transition: border-color 0.2s;
font-family: inherit;
}

.input:focus {
border-color: #333;
}

.input::placeholder {
color: var(--muted);
}

.build-btn {
align-self: flex-start;
background: var(--text);
color: var(--bg);
border: none;
border-radius: 8px;
font-size: 14px;
font-weight: 600;
padding: 10px 24px;
cursor: pointer;
transition: opacity 0.2s;
}

.build-btn:disabled {
opacity: 0.3;
cursor: default;
}

.build-btn:not(:disabled):hover {
opacity: 0.85;
}

.hint {
font-size: 12px;
color: var(--muted);
}

/* ── Progress ─────────────────────────────── */
.progress {
width: 100%;
max-width: 560px;
padding: 0 24px;
display: flex;
flex-direction: column;
gap: 32px;
}

.steps {
display: flex;
flex-direction: column;
gap: 14px;
}

.step {
display: flex;
align-items: center;
gap: 12px;
opacity: 0.3;
transition: opacity 0.3s;
}

.step--running,
.step--done,
.step--failed {
opacity: 1;
}

.step-dot {
width: 6px;
height: 6px;
border-radius: 50%;
background: var(--muted);
flex-shrink: 0;
transition: background 0.3s;
}

.step--running .step-dot {
background: var(--text);
animation: pulse 1.2s infinite;
}

.step--done .step-dot {
background: var(--green);
}

.step--failed .step-dot {
background: var(--red);
}

.step-label {
font-size: 14px;
color: var(--text);
}

@keyframes pulse {
0%, 100% { opacity: 1; }
50% { opacity: 0.3; }
}

/* ── Done ─────────────────────────────────── */
.done {
width: 100%;
max-width: 560px;
padding: 0 24px;
display: flex;
flex-direction: column;
gap: 16px;
}

.done-msg {
font-size: 22px;
font-weight: 400;
}

.done-path {
font-size: 12px;
color: var(--muted);
font-family: monospace;
}

.done-actions {
display: flex;
gap: 12px;
}

.open-btn {
background: var(--text);
color: var(--bg);
border: none;
border-radius: 8px;
font-size: 14px;
font-weight: 600;
padding: 10px 24px;
cursor: pointer;
}

.reset-btn {
background: transparent;
color: var(--muted);
border: 1px solid var(--border);
border-radius: 8px;
font-size: 14px;
padding: 10px 24px;
cursor: pointer;
}

.reset-btn:hover {
color: var(--text);
border-color: #333;
}


Wire the Tauri Backend
In src-tauri/src/main.rs:

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use std::fs;

#[tauri::command]
fn run_step(step: u32, prompt: String) -> Result<(), String> {
// Write prompt to file for step 1
if step == 1 {
fs::write("../intent-to-software/user_prompt.txt", &prompt)
.map_err(|e| e.to_string())?;
}

let script = format!("step{}_compress.py", step);
let script = match step {
1 => "step1_compress.py",
2 => "step2_mockui.py",
3 => "step3_parse.py",
4 => "step4_dag.py",
5 => "step5_tasks.py",
6 => "step6_build.py",
_ => return Err("Invalid step".to_string()),
};

let output = Command::new("python3")
.arg(script)
.current_dir("../intent-to-software")
.output()
.map_err(|e| e.to_string())?;

if output.status.success() {
Ok(())
} else {
Err(String::from_utf8_lossy(&output.stderr).to_string())
}
}

#[tauri::command]
fn get_output_path() -> String {
let path = std::path::Path::new("../intent-to-software/output");
path.canonicalize()
.map(|p| p.to_string_lossy().to_string())
.unwrap_or_else(|_| "./output".to_string())
}

#[tauri::command]
fn open_output_folder(path: String) {
#[cfg(target_os = "windows")]
Command::new("explorer").arg(&path).spawn().ok();
#[cfg(target_os = "linux")]
Command::new("xdg-open").arg(&path).spawn().ok();
#[cfg(target_os = "macos")]
Command::new("open").arg(&path).spawn().ok();
}

fn main() {
tauri::Builder::default()
.invoke_handler(tauri::generate_handler![
run_step,
get_output_path,
open_output_folder
])
.run(tauri::generate_context!())
.expect("error while running tauri application");
}


Run the app:

npm run tauri dev


The Runner: Full Pipeline Without UI
For testing the pipeline directly without the UI:
Create run.py:

import subprocess
import sys

steps = [
"step1_compress.py",
"step2_mockui.py",
"step3_parse.py",
"step4_dag.py",
"step5_tasks.py",
"step6_build.py",
]

for step in steps:
print(f"\n{'='*40}")
print(f"Running {step}")
print('='*40)
result = subprocess.run([sys.executable, step])
if result.returncode != 0:
print(f"ERROR: {step} failed. Stopping.")
sys.exit(1)

print("\n✓ Pipeline complete. Check /output/")


python3 run.py


When Things Break
Gemma returns garbage JSON: Print the raw response. Look at what actually came back. Usually wrapped in markdown or has extra text. The cleaning code handles most cases.
Ollama connection refused:

curl http://172.30.80.1:11434/api/tags


If that fails, Ollama isn’t running or the WSL IP changed.
A task fails all 3 retries: Open tasks.json. Find that task. Read the description. If it’s vague, Gemma can’t write it. Make the description more specific and re-run step 6 — it skips tasks already marked done.
Tauri can’t find the Python scripts: Check the relative paths in main.rs. Adjust current_dir to point to wherever intent-to-software lives on your machine.

What Success Looks Like
Someone opens the app. Types what they want — in any language. Hits Build. Watches six steps complete quietly. Gets a folder of working software on the other side.
That’s the demo. That’s the video. That’s the submission.​​​​​​​​​​​​​​​​