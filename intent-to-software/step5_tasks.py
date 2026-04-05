import json
import requests

OLLAMA_URL = "http://localhost:11434/api/chat"
MODEL = "gemma4:26b-optimized"

with open("dag.json", "r") as f:
    dag = json.load(f)

with open("compressed_intent.txt", "r") as f:
    intent = f.read()


def ask_gemma(system, user):
    response = requests.post(
        OLLAMA_URL,
        json={
            "model": MODEL,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user},
            ],
            "stream": False,
            "options": {
                "temperature": 1.0,
                "top_p": 0.95,
                "top_k": 64,
            },
        },
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
except Exception:
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
except Exception:
    print("Error parsing tests. Raw:")
    print(tests_raw)
    tests = []

with open("tasks.json", "w") as f:
    json.dump(tasks, f, indent=2)

with open("tests.json", "w") as f:
    json.dump(tests, f, indent=2)

print("Step 5 complete.")
print(f"Generated {len(tasks)} tasks and {len(tests)} tests")
