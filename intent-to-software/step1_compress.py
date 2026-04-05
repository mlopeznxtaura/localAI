import requests

OLLAMA_URL = "http://localhost:11434/api/chat"
MODEL = "gemma4:26b-optimized"

with open("user_prompt.txt", "r") as f:
    raw_prompt = f.read()


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
You are a prompt compressor.
The user will give you a messy description of software they want to build.
Your job is to return ONE clean sentence that captures exactly what they want.
Nothing else. No explanation. No questions. Just the one sentence.
Respond in the same language the user wrote in.
"""

result = ask_gemma(system_prompt, raw_prompt)

with open("compressed_intent.txt", "w", encoding="utf-8") as f:
    f.write(result)

print("Step 1 complete.")
print("Compressed intent:", result)
