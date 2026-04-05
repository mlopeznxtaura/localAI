import json
import requests

OLLAMA_URL = "http://localhost:11434/api/chat"
MODEL = "gemma4:26b-optimized"

with open("features.json", "r") as f:
    features = json.load(f)


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
except Exception:
    print("Gemma returned invalid JSON. Raw output:")
    print(result)
    dag = {"nodes": [], "build_order": []}

with open("dag.json", "w") as f:
    json.dump(dag, f, indent=2)

print("Step 4 complete.")
print("Build order:", dag.get("build_order", []))
