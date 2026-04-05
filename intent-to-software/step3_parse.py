import json
import requests

try:
    from bs4 import BeautifulSoup
except ModuleNotFoundError as e:
    raise SystemExit(
        "Missing dependency: beautifulsoup4 (bs4).\n"
        "Install with:\n"
        "  python3 -m pip install -r requirements.txt\n"
        "Then re-run step 3."
    ) from e

OLLAMA_URL = "http://localhost:11434/api/chat"
MODEL = "gemma4:26b-optimized"

with open("mock_ui.html", "r", encoding="utf-8") as f:
    html = f.read()

soup = BeautifulSoup(html, "html.parser")

features = []

for button in soup.find_all("button"):
    text = button.get_text(strip=True)
    if text:
        features.append({"type": "button", "label": text})

for input_tag in soup.find_all("input"):
    label = (
        input_tag.get("placeholder")
        or input_tag.get("name")
        or input_tag.get("type", "input")
    )
    features.append({"type": "input", "label": label})

for form in soup.find_all("form"):
    features.append({"type": "form", "label": form.get("id") or "form"})

for heading in soup.find_all(["h1", "h2", "h3"]):
    text = heading.get_text(strip=True)
    if text:
        features.append({"type": "section", "label": text})


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
except Exception:
    gemma_features = []

output = {"parsed_elements": features, "gemma_features": gemma_features}

with open("features.json", "w") as f:
    json.dump(output, f, indent=2)

print("Step 3 complete.")
print(f"Found {len(features)} elements, {len(gemma_features)} features")
