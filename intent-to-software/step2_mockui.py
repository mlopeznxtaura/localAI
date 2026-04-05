import requests

OLLAMA_URL = "http://localhost:11434/api/chat"
MODEL = "gemma4:26b-optimized"

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

with open("mock_ui.html", "w", encoding="utf-8") as f:
    f.write(result)

print("Step 2 complete.")
print("Mock UI written to mock_ui.html")
