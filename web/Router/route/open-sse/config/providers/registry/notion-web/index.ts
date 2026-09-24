import type { RegistryEntry } from "../../shared.ts";

// Notion AI Web (Unofficial/Experimental).
// Live catalog comes from cookie-auth POST /api/v3/getAvailableModels (models route).
// The registry seed below is the offline fallback when discovery fails.
// Catalog ids are real web-picker labels (fable-5, gpt-5.6-sol).
// (The OmniRoute merge referenced a services/notionWebModels.ts module that was
// never copied over; the seed is inlined here instead of inventing new data.)
export const notion_webProvider: RegistryEntry = {
  id: "notion-web",
  alias: "nw",
  format: "openai",
  executor: "notion-web",
  baseUrl: "https://app.notion.com/api/v3/runInferenceTranscript",
  authType: "apikey",
  authHeader: "cookie",
  passthroughModels: true,
  models: [
    { id: "fable-5", name: "Fable 5 (Notion)" },
    { id: "gpt-5.6-sol", name: "GPT-5.6 Sol (Notion)" },
  ],
};
