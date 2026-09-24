import type { RegistryEntry } from "../../shared.ts";
import { buildOpenAiCompatibleRegistryEntry } from "../../shared.ts";

export const helixmindProvider: RegistryEntry = buildOpenAiCompatibleRegistryEntry({
  id: "helixmind",
  alias: "helixmind",
  baseUrl: "https://helixmind.online/v1/chat/completions",
  modelsUrl: "https://helixmind.online/v1/models",
  responsesBaseUrl: "https://helixmind.online/v1/responses",
  // NOTE: helixmind also serves an Anthropic-compatible surface at
  // https://helixmind.online/v1/messages (x-api-key). There is currently no
  // RegistryEntry field for it, so it is documented here rather than as a
  // dead `alternateFormats` key (removed: no runtime path reads it).
  models: [],
  passthroughModels: true,
});
