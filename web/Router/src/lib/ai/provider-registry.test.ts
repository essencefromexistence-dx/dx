import { describe, expect, it } from "vitest";

import { REGISTRY } from "../../../route/open-sse/config/providers/index.ts";
// Importing the UI catalog runs Zod validation at module load — any duplicate
// key with an invalid shape or schema drift throws here.
import { AI_PROVIDERS } from "@/shared/constants/providers";

/**
 * Guards the provider wiring fixed on 2026-09-22:
 * - every runtime registry entry must resolve a baseUrl/baseUrls
 *   (spreads resolved — e.g. kimi-coding via KIMI_CODING_SHARED counts),
 * - registry keys must match entry ids and carry format + executor,
 * - registry aliases must be unique (alias lookup is first-wins).
 */
describe("provider registry audit", () => {
  it("every entry resolves a baseUrl or baseUrls", () => {
    const missing = Object.entries(REGISTRY)
      .filter(([, entry]) => !entry.baseUrl && !entry.baseUrls)
      .map(([id]) => id);
    expect(missing).toEqual([]);
  });

  it("every entry id matches its registry key and has format + executor", () => {
    const bad: string[] = [];
    for (const [key, entry] of Object.entries(REGISTRY)) {
      if (entry.id !== key || !entry.format || !entry.executor) {
        bad.push(key);
      }
    }
    expect(bad).toEqual([]);
  });

  it("registry aliases are unique", () => {
    const seen = new Map<string, string>();
    const dupes: string[] = [];
    for (const entry of Object.values(REGISTRY)) {
      const alias = entry.alias || entry.id;
      const prev = seen.get(alias);
      if (prev && prev !== entry.id) dupes.push(`${alias} (${prev} vs ${entry.id})`);
      seen.set(alias, entry.id);
    }
    expect(dupes).toEqual([]);
  });

  it("every registered provider has a UI definition", () => {
    const missing = Object.keys(REGISTRY).filter(
      (id) => !(AI_PROVIDERS as Record<string, unknown>)[id],
    );
    expect(missing).toEqual([]);
  });
});
