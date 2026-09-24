import type { RegistryEntry } from "../../shared.ts";

// PromptQL playground agent (prompt.ql.app) — unofficial reverse-engineered session.
// Live catalog: GraphQL FetchLlmConfigs; seed below is offline fallback.
export const promptqlProvider: RegistryEntry = {
  id: "promptql",
  alias: "pql",
  format: "openai",
  executor: "promptql",
  baseUrl: "https://data.prompt.ql.app/promptql/playground-v2-hge/v1/graphql",
  authType: "apikey",
  authHeader: "authorization",
  passthroughModels: true,
  // No static catalog: live catalog is GraphQL FetchLlmConfigs via passthrough.
  // (The OmniRoute merge referenced a services/promptqlModels.ts module that
  // was never copied over; dropped the dangling import instead of inventing data.)
  models: [],
};
