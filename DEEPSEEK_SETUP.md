# DeepSeek R1:32B Local Setup Guide

This chat application has been configured to use DeepSeek R1:32B model running locally instead of Google Gemini API.

## Prerequisites

### Install Ollama

1. **Download and install Ollama:**
   ```bash
   # macOS
   brew install ollama
   
   # Or download from: https://ollama.ai/download
   ```

2. **Start Ollama service:**
   ```bash
   ollama serve
   ```
   This will start Ollama on `http://localhost:11434`

### Install DeepSeek R1:32B Model

3. **Pull the DeepSeek R1:32B model:**
   ```bash
   ollama pull deepseek-r1:32b
   ```
   
   **Note:** This is a large model (~20GB). Make sure you have sufficient disk space and RAM (recommended 32GB+ RAM).

4. **Verify the model is installed:**
   ```bash
   ollama list
   ```
   You should see `deepseek-r1:32b` in the list.

## Configuration

### Environment Variables

The application uses the following configuration:

- **Default API URL:** `http://localhost:11434/v1/chat/completions`
- **Model:** `deepseek-r1:32b`
- **Custom URL:** Set `DEEPSEEK_API_URL` in `.env` file if using a different endpoint

### .env File

```env
# DeepSeek API Configuration
# Default: http://localhost:11434/v1/chat/completions (Ollama)
# DEEPSEEK_API_URL=http://localhost:11434/v1/chat/completions
```

## Running the Application

1. **Start Ollama (if not already running):**
   ```bash
   ollama serve
   ```

2. **Start the Rust backend:**
   ```bash
   cargo run
   ```
   You should see: `Using DeepSeek API at: http://localhost:11434/v1/chat/completions`

3. **Start the React frontend:**
   ```bash
   cd frontend
   PORT=3003 npm start
   ```

## Testing the AI Integration

The AI will respond to:
- Questions (containing "?", "what", "how", "why", etc.)
- Greetings ("hello", "hi", "hey")
- Direct mentions ("ai", "bot", "assistant")
- Help requests

### Example Test Messages:
- "Hello!"
- "What is the weather like?"
- "Can you help me?"
- "AI, how are you?"

## Troubleshooting

### Common Issues:

1. **"DeepSeek API request failed with status: 404"**
   - Make sure Ollama is running: `ollama serve`
   - Verify the model is installed: `ollama list`

2. **"Failed to send request to DeepSeek API"**
   - Check if Ollama is accessible: `curl http://localhost:11434/api/tags`
   - Verify firewall settings

3. **Model not found error:**
   - Pull the model: `ollama pull deepseek-r1:32b`
   - Check available models: `ollama list`

4. **Out of memory errors:**
   - DeepSeek R1:32B requires significant RAM (32GB+ recommended)
   - Consider using a smaller model like `deepseek-r1:7b` or `deepseek-r1:14b`

### Alternative Models:

If DeepSeek R1:32B is too large for your system, you can use smaller variants:

```bash
# Smaller alternatives
ollama pull deepseek-r1:7b
ollama pull deepseek-r1:14b
```

Then update the model name in `src/main.rs`:
```rust
model: "deepseek-r1:7b".to_string(),  // or deepseek-r1:14b
```

## Performance Notes

- **First response:** May be slower as the model loads into memory
- **Subsequent responses:** Should be faster once the model is loaded
- **Memory usage:** Monitor system RAM usage, especially with the 32B model
- **CPU usage:** DeepSeek models can be CPU-intensive without GPU acceleration

## GPU Acceleration (Optional)

For better performance, you can enable GPU acceleration:

```bash
# For NVIDIA GPUs
ollama serve --gpu

# For Apple Silicon Macs (Metal)
# GPU acceleration is enabled by default
```

---

**Your chat application is now configured to use DeepSeek R1:32B locally!** 🚀