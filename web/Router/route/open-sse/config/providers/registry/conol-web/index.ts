import type { RegistryEntry } from "../../shared.ts";

export const conol_webProvider: RegistryEntry = {
  id: "conol-web",
  alias: "cnl",
  format: "openai",
  executor: "conol-web",
  baseUrl: "https://conol.ai/api/sessions",
  authType: "apikey",
  authHeader: "cookie",
  passthroughModels: true,
  // No static catalog: conol.ai exposes no public /models endpoint, so the
  // live chat catalog is fetched via passthrough (same pattern as helixmind).
  // (The OmniRoute merge referenced a services/conolModels.ts module that was
  // never copied over; dropped the dangling import instead of inventing data.)
  models: [],
};
