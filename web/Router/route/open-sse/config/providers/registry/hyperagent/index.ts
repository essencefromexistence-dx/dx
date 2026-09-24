import type { RegistryEntry } from "../../shared.ts";

// HyperAgent (hyperagent.com) — unofficial reverse-engineered web session.
// Auth: browser Cookie header. Chat: POST /api/threads/{id}/chat (SSE).
// Credits: GET /api/settings/billing/usage → creditData.creditBlocks.
/** Claude-family agent models on HyperAgent — 1M context (not the 128k openai default). */
const HYPERAGENT_CONTEXT_LENGTH = 1_000_000;

export const hyperagentProvider: RegistryEntry = {
  id: "hyperagent",
  alias: "ha",
  format: "openai",
  executor: "hyperagent",
  baseUrl: "https://hyperagent.com/api/threads",
  authType: "apikey",
  authHeader: "cookie",
  passthroughModels: true,
  /** Used by getTokenLimit when model-specific context is missing. */
  defaultContextLength: HYPERAGENT_CONTEXT_LENGTH,
  // No static catalog: live models are discovered via passthrough.
  // (The OmniRoute merge referenced a services/hyperagentModels.ts module that
  // was never copied over; dropped the dangling import instead of inventing data.)
  models: [],
};
