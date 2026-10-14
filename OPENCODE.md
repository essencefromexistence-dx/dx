# OpenCode Go — Step 5 model: how to call it

Model: **Step 5 Preview Free** (`step-5-preview-free`) — free for a limited
time, unlimited usage while the preview lasts. Docs: `https://opencode.ai/docs/go/`.

## Endpoint

- URL: `https://opencode.ai/zen/go/v1/chat/completions` (OpenAI-compatible)
- Model id: `step-5-preview-free`
- OpenCode config spelling: `opencode-go/step-5-preview-free`
- Models inventory: `https://opencode.ai/zen/go/v1/models`

## Required headers

`Authorization: Bearer <key>` alone is **not** enough — raw curl without the
session headers fails with `MissingSessionID` (verified 2026-10-11):

| Header | Value |
|---|---|
| `Authorization` | `Bearer $OPENCODE_GO_API_KEY` |
| `Content-Type` | `application/json` |
| `x-opencode-session` | any stable id per conversation, e.g. `dxbot-chat` |
| `User-Agent` | your own agent name, e.g. `dxbot/1.0` (not a generic SDK name) |

## Env (see `.env.example` — values stay empty here on purpose)

| Var | Meaning |
|---|---|
| `OPENCODE_GO_API_KEY` | your Go `oc_sk_...` key (paste locally, never commit) |
| `OPENCODE_GO_MODEL` | `opencode-go/step-5-preview-free` |
| `OPENCODE_API_KEY` | Zen key fallback (leave empty unless you have one) |

Copy `.env.example` to `.env` (git-ignored) and fill the key there.

## Test it

PowerShell:

```powershell
$body = '{"model":"step-5-preview-free","messages":[{"role":"user","content":"reply with the word ok"}]}'
curl.exe -s -X POST "https://opencode.ai/zen/go/v1/chat/completions" `
  -H "Authorization: Bearer $env:OPENCODE_GO_API_KEY" `
  -H "Content-Type: application/json" `
  -H "x-opencode-session: test-session-1" `
  -H "User-Agent: my-coding-agent/1.0" `
  -d $body
```

Bash:

```bash
curl -s -X POST https://opencode.ai/zen/go/v1/chat/completions \
  -H "Authorization: Bearer $OPENCODE_GO_API_KEY" \
  -H "Content-Type: application/json" \
  -H "x-opencode-session: test-session-1" \
  -H "User-Agent: my-coding-agent/1.0" \
  -d '{"model":"step-5-preview-free","messages":[{"role":"user","content":"reply with the word ok"}]}'
```

Success looks like: `{"id":"gen-...","object":"chat.completion",
"model":"step-5-preview-free","choices":[{"message":{"role":"assistant",
"content":"ok",...}}]}`.

| Symptom | Cause |
|---|---|
| `MissingSessionID` | `x-opencode-session` header missing |
| `401 Invalid API key` | key empty, wrong, or pasted with extra words (must be exactly `oc_sk_...`) |
| `server_error: Endpoint is unavailable` | transient upstream — retry |

## Key hygiene

The key is a bearer secret: keep it in `.env` (ignored) or a password
manager, never in chat logs, docs, or committed files. If a key was ever
pasted somewhere readable, rotate it in the OpenCode console and update the
one line in `.env`.
