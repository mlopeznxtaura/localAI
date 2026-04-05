# WSL Ollama Setup - Single Shared Models Directory

## ✅ Setup Complete!

**ONE models directory** shared between WSL and Windows:
- **Location:** `D:\NextAura\models\ollama`

## Architecture

```
D:\NextAura\models\ollama  ← Single shared directory
         ↑
    WSL Ollama (primary)
    - Runs in Ubuntu WSL
    - Listens on 0.0.0.0:11434
    - Stores models in Windows directory

    Port Forwarding
    127.0.0.1:11434 → WSL IP:11434

    VS Code
    - Connects to http://localhost:11434
    - Sees all models
```

## How It Works

1. **WSL Ollama** is the ONLY Ollama instance running
2. **Models stored** in `D:\NextAura\models\ollama` (Windows path)
3. **Port forwarding** redirects localhost:11434 → WSL:11434
4. **VS Code** connects seamlessly to localhost:11434

## Available Models

| Model | Status | Size | Notes |
|-------|--------|------|-------|
| **gemma4:26b-optimized** | ✅ **RECOMMENDED** | 17 GB | Thinking enabled, optimal params |
| **gemma4:26b** | ✅ Working | 17 GB | MoE - 4B active params, efficient |
| **nemotron-cascade-2:30b** | ✅ Working | 24 GB | MoE architecture |
| **VladimirGav/gemma4-26b-16GB-VRAM:latest** | ✅ Working | 14 GB | Optimized for 16GB VRAM |
| ~~gemma4:31b~~ | ❌ Removed | - | Too large for GPU (falls back to CPU) |
| ~~aravhawk/qwen3.5-opus-4.6:27b~~ | ❌ Removed | - | Incompatible architecture (qwen35) |

## GPU Info
- **GPU:** NVIDIA GeForce RTX 5090 (32GB VRAM)
- **CUDA:** 13.1
- **Ollama:** 0.20.0

## Quick Commands

### Pull new model
```bash
wsl ollama pull <model-name>
```

### List models
```bash
wsl ollama list
```

### Start/Stop WSL Ollama
```bash
wsl bash -c "sudo systemctl start ollama"
wsl bash -c "sudo systemctl stop ollama"
```

### Test a model
```bash
# First load takes 30-45 seconds (model loading to GPU)
wsl ollama run gemma4:26b-optimized "Hello"

# Subsequent requests are fast (model stays loaded for 5 minutes)
wsl ollama run gemma4:26b-optimized "What is 2+2?"
```

**Note:** Initial model load takes 30-45 seconds. Keep `--keep-alive` default (5m) to avoid reloading.

## Gemma 4 Best Practices (from Official Manual)

### 1. Sampling Parameters (STANDARD for all use cases)
```
temperature=1.0
top_p=0.95
top_k=64
```

### 2. Thinking Mode Configuration

**Enable Thinking:**
Add `<|think|>` token at the start of system prompt:
```
<|think|>
You are a helpful AI assistant...
```

**Disable Thinking:**
Remove the `<|think|>` token from system prompt.

**Output Structure (when thinking enabled):**
```
<|channel>thought
[Internal reasoning]
<channel|>
[Final answer]
```

**Note:** For all models except E2B/E4B, if thinking is disabled, the model still generates empty thought tags:
```
<|channel>thought
<channel|>
[Final answer]
```

### 3. Multi-Turn Conversations
**CRITICAL:** In conversation history, only include the FINAL response.
Do NOT include thinking content from previous turns.

**Correct:**
```
User: What is 2+2?
Assistant: 4

User: Multiply by 3
Assistant: 12
```

**Incorrect:**
```
User: What is 2+2?
Assistant: <|channel>thought...<channel|>4

User: Multiply by 3
Assistant: <|channel>thought...<channel|>12
```

### 4. Multimodal Input Order
For optimal performance with images/audio:
- **Place image/audio content BEFORE text** in your prompt
- Example: `[IMAGE] What do you see in this picture?`

### 5. Variable Image Resolution
Gemma 4 supports configurable visual token budgets:

| Budget | Use Case |
|--------|----------|
| 70 | Fast classification, basic captioning |
| 140 | General image understanding |
| 280 | Detailed analysis, moderate OCR |
| 560 | High-detail OCR, document parsing |
| 1120 | Maximum detail, small text reading |

### 6. Native System Prompt Support
Gemma 4 has native support for system, user, and assistant roles.
Use standard chat format for best results.

## Maintenance

### After Windows Restart (WSL IP may change)
Run as Administrator:
```
D:\NextAura\projects2\localAI\update-ollama-connection.bat
```

### Check if WSL Ollama is running
```bash
wsl ollama list
```

### Test connection from Windows
```bash
curl http://localhost:11434/api/tags
```

### Check GPU usage
```bash
wsl nvidia-smi
```

### View Ollama logs
```bash
wsl bash -c "sudo journalctl -u ollama --no-pager -n 50"
```

## Gemma 4 Models

### gemma4:26b (Mixture of Experts) - RECOMMENDED
- Total Parameters: 25.2B
- Active Parameters: 3.8B (efficient!)
- Context: 256K tokens
- Best for: Reasoning, coding, general tasks
- VRAM Usage: ~17GB

### VladimirGav/gemma4-26b-16GB-VRAM
- Optimized version for 16GB VRAM systems
- Smaller quantization
- Good for systems with limited VRAM

### Recommended Settings
```
temperature=1.0
top_p=0.95
top_k=64
```

### Enable Thinking Mode
Add `<|think|>` token at the start of system prompt to enable reasoning.

## Troubleshooting

### VS Code shows "Unable to verify Ollama server version"
1. Run `update-ollama-connection.bat` as Administrator
2. Restart VS Code
3. Verify: `curl http://localhost:11434/api/tags`

### Connection refused
1. Check WSL Ollama: `wsl ollama list`
2. Start if needed: `wsl bash -c "sudo systemctl start ollama"`
3. Check port forwarding: `netsh interface portproxy show all`

### Model hangs or times out
- Model may be too large for GPU VRAM
- Check logs: `wsl bash -c "sudo journalctl -u ollama --no-pager -n 20"`
- Look for "offloaded 0/X layers to GPU" (means CPU fallback = slow)
- Solution: Use smaller model or MoE variant

### Model shows "unknown architecture"
- Model format incompatible with current Ollama version
- Solution: Remove model and find compatible alternative
- Check Ollama version: `wsl ollama --version`

### Models not showing in VS Code
1. Verify models exist: `wsl ollama list`
2. Check directory: `dir D:\NextAura\models\ollama\blobs`
3. Restart WSL Ollama: `wsl bash -c "sudo systemctl restart ollama"`
4. Restart VS Code

## Known Issues

### gemma4:31b (Dense) - Not Recommended
- **Issue:** Falls back to CPU (0/61 layers on GPU)
- **Reason:** 31B dense model + KV cache exceeds 32GB VRAM
- **Symptom:** Extremely slow, times out
- **Solution:** Use gemma4:26b (MoE) instead - similar performance, much faster

### aravhawk/qwen3.5-opus-4.6:27b - Incompatible
- **Issue:** "unknown model architecture: 'qwen35'"
- **Reason:** Ollama 0.20.0 doesn't support this architecture yet
- **Solution:** Wait for Ollama update or use alternative model

## Configuration

- **Models Directory:** `D:\NextAura\models\ollama`
- **WSL Service:** `/etc/systemd/system/ollama.service`
- **VS Code Config:** `D:\Users\archi\AppData\Roaming\Code\User\chatLanguageModels.json`
- **Port Forwarding:** `127.0.0.1:11434 → WSL_IP:11434`

## Citations

### Qwen3.5-Opus-4.6 Distilled Model (Reference Only - Not Installed)
```bibtex
@misc{jackrong_qwen35_opus_distilled,
  title        = {Qwen3.5-27B-Claude-4.6-Opus-Reasoning-Distilled},
  author       = {Jackrong},
  year         = {2026},
  publisher    = {Hugging Face},
  howpublished = {\url{https://huggingface.co/Jackrong/Qwen3.5-27B-Claude-4.6-Opus-Reasoning-Distilled}}
}
```
