# Changelog

All notable changes to this project are documented in this file.
Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased] - 2026-09-22

### Added

- **Wired 83 missing providers into the runtime `REGISTRY`**
  (`route/open-sse/config/providers/index.ts` 186 → 269 entries): every
  registry folder copied in the 2026-08-22 OmniRoute merge that had a UI
  definition and a valid entry is now registered (incl. `muse-code`,
  `qwen-cloud`, `ghe-copilot`, the `aion`–`zylo-api` gateway batch, `mlx-gemma` +
  `mlx-qwen`, `tinycms-web`). Only `magnific` stays out — it is an image-registry
  entry (`MAGNIFIC_IMAGE_PROVIDER`), not a chat provider. Method: cross-checked
  each folder's `export const` name, provider `id`, and `baseUrl` before wiring;
  no alias collisions.
- **`src/lib/ai/provider-registry.test.ts`** (vitest, 4 tests): every registry
  entry resolves a `baseUrl`/`baseUrls`, ids match keys with format + executor,
  aliases are unique, and every registered provider has a UI definition.

### Fixed

- **`muse-code` had no `baseUrl`** — `buildProviderUrl`/`DefaultExecutor` fell
  through to `undefined`, so every Muse request failed with no base URL. Now
  `https://api.meta.ai/v1/responses` (Meta Model API Responses surface, verified
  against dev.meta.ai docs) + `modelsUrl https://api.meta.ai/v1/models`; dropped
  the dead `reasoningTransport` field (not part of `RegistryEntry`).
- **`bedrock` had no `baseUrl`** — added region default
  `https://bedrock-runtime.us-east-1.amazonaws.com` (matches
  `BEDROCK_DEFAULT_REGION`; the custom executor builds per-region URLs and never
  reads it — same pattern as `vertex`). Key validation is unaffected (specialty
  `validateBedrockProvider` runs first).
- **Removed 65 duplicate provider blocks** appended with 4-space indent by the
  2026-08-22 merge (`gateways.ts` ×36, `regional.ts` ×8, `web-cookie.ts` ×8,
  `oauth.ts` ×4, `audio.ts` ×3, `noauth.ts` ×3, `specialty-media.ts` ×2,
  `frontier-labs.ts` ×1). All pairs verified token-identical; last-wins copies
  kept, so runtime behavior is unchanged.
- **Removed 2 cross-file duplicates**: `firecrawl` from
  `apikey/specialty-media.ts` (kept richer `search.ts` entry with `webSearch`
  kind + authHint), `sarvam` from `apikey/frontier-labs.ts` (kept `regional.ts`
  entry whose alias matches the registry).
- **Repaired 4 dangling registry imports** referencing modules the merge never
  copied (`services/conolModels.ts`, `hyperagentModels.ts`, `promptqlModels.ts`,
  `notionWebModels.ts` — each broke `REGISTRY` load): `conol-web`/`hyperagent`/
  `promptql` now use `models: []` with `passthroughModels: true` (live
  discovery); `notion-web` inlines its two documented web-picker labels
  (`fable-5`, `gpt-5.6-sol`).
- **Cleared 8 type errors in newly-wired registry files** (repo `tsc` errors in
  `route/open-sse/config/providers/` 8 → 0): removed dead top-level fields no
  runtime path reads (`reasoningTransport`, `alternateFormats` ×3,
  `messagesUrl`, provider-level `unsupportedParams`,
  `requiresPlainStringContent`) with the endpoint knowledge preserved as
  comments; `openference` oauth now uses `clientIdEnv: OPENFERENCE_CLIENT_ID`
  (no embedded public client id exists); `aihorde/imageModels.ts` dangling
  import replaced with an explicit empty-list stub.
- **`scripts/generate-providers.ts` field bleed**: `extractProviderName/Hint/
  Url` and `hasFreeField` used unbounded `[\s\S]*?` matches, so any provider
  missing a field inherited a later provider's value (e.g. `ghe-copilot`
  website `openference.com`, `plamo` hint from `sea-lion`, truncated
  `authHint: "Click "`). Extraction is now scoped to the provider's own block,
  quote-aware, with `apiHint` fallback. `readRegistryModels` no longer emits a
  phantom `{id: <providerId>}` model, no longer drops each file's first model,
  and captures full names (apostrophe-safe).

### Changed

- **Regenerated `src/lib/ai/providers.generated.ts`** — still 433 providers,
  now with 1,716 models (was ~1,740 with phantoms); 192 corrected `hasFree`
  flags, 181 corrected `authHint`s, 15 corrected websites from the bleed fix.
- **README.md**: registry metric 253 → 269 wired entries, model total 1,716,
  `firecrawl` table row moved APIKey → Search (313/14), category counts
  corrected (`frontier-labs` 29 → 28, `specialty-media` 28 → 27 — the old table
  double-counted the duplicated `muse-code`).

### Verified

- New vitest file passes: 4/4 (`vitest run src/lib/ai/provider-registry.test.ts`).
- Related suites pass: `src/lib/utils/{registry,url,format}.test.ts` 16/16.
- `tsc --noEmit`: 0 errors in `route/open-sse/config/providers/`, the generator
  script, and UI constants (repo total 6,974 → 6,966; remaining errors are
  pre-existing in untouched areas).
- Structural audits re-run clean: 0 duplicate keys, 0 cross-file dupes, 0
  dangling registry imports, 269/269 imports↔entries consistent with key↔id
  match, all 18 touched files transpile.

## [Unreleased] - 2026-08-22

### Added

- **Provider catalog merged from `G:\Dx\OmniRoute` (+73 definitions, +86 registry
  catalogs, +4 icons) — catalog now at 433 providers / ~1,740 models**:
  - NoAuth (+3): `devin-cli-agentic`, `felo-web`, `zcode`
  - OAuth (+4): `ghe-copilot`, `xai-oauth`, `openference`, `raycast`
  - WebCookie (+8): `chatgpt-web-codex`, `microsoft-designer-web`,
    `tencent-aistudio-web`, `tinycms-web`, `promptql`, `notion-web`, `hyperagent`,
    `conol-web`
  - APIKey gateways (+38): `cheaperinference`, `freebuff`, `zylo-api`, `electronhub`,
    `llm-kiwi`, `literouter`, `mnn-ai`, `meganova-ai`, `speka`, `tokenreply`,
    `yolo-auto`, `dxnt`, `cloudcode-one`, `ofoxai`, `zerolimitai`, `chatanywhere`,
    `poixe-ai`, `naga-ai`, `chat-oripe`, `freeinference`, `free-ai`, `dahl`,
    `freetheai`, `g4f-groq`, `g4f-gemini`, `g4f-pollinations`, `g4f-ollama`,
    `g4f-nvidia`, `token-kiosk`, `chenzk`, `navy`, `aion`, `routeway`, `nara`,
    `naga-ac`, `void-ai`, `helixmind`, `tabitoken`
  - Frontier labs (+1): `muse-code`
  - Regional (+9): `qwen-cloud`, `qwen-cloud-token-plan`, `sealion`, `clova-studio`,
    `internlm`, `ant-ling`, `sarvam`, `plamo`, `typhoon`
  - Specialty media (+3): `magnific`, `segmind`, `mixedbread`
  - Local (+2): `mlx-gemma`, `mlx-qwen`
  - Search (+2): `firecrawl`, `x-search`
  - Audio (+3): `gladia`, `rev-ai`, `speechmatics`
- **86 model registry folders** copied from OmniRoute
  (`route/open-sse/config/providers/registry/`) → registry now covers 253 providers.
- **4 provider icons** copied → 464 total in `public/providers/`.
- **README.md**: added complete provider index — all 433 providers listed in collapsible
  per-category tables (NoAuth 12 / OAuth 27 / WebCookie 35 / APIKey 314 / Local 14 /
  Search 13 / Audio 12 / Proxy 2 / CloudAgent 3 / System 1), generated from
  `providers.generated.ts` between `PROVIDER-LIST` markers.

### Changed

- **`scripts/generate-providers.ts`**: fixed `ROUTE_DIR` to resolve to the repo root
  and the registry reader path to `route/open-sse/...`; regeneration now reads all
  15 category files (was silently reading a non-existent `route/src/...` tree and
  producing a stale catalog).

### Fixed

- **`better-sqlite3` upgraded 12.11.1 → 13.0.3** (in `optionalDependencies`):
  v13 ships N-API prebuilds and no longer calls Node's `RemoveEnvironmentCleanupHook`
  from the `Statement` destructor. This eliminated the native assertion
  (`Assertion failed: (env) != nullptr` / `node::RemoveEnvironmentCleanupHook`) that
  crashed `next build` during page-data collection on Node 24 (`Router/.next` was
  produced; production `next build` now completes).

### Verified

- `next build` (Turbopack, `NEXT_BUILD_CPUS=1`) completes: `Router/.next/BUILD_ID`
  present, no FATAL/assertion errors in the build log.
- `better-sqlite3@13.0.3` native binding loads on Node 24.20.0: in-memory DB create +
  query returned SQLite `3.53.4`.

- Regenerated `src/lib/ai/providers.generated.ts`: **433 providers / ~1,740 models**
  (APIKey 314, WebCookie 35, OAuth 27, Local 14, Audio 12, NoAuth 12, Search 13,
  CloudAgent 3, Proxy 2, System 1); file parses cleanly; DEFAULT ids intact.
- All 15 edited category files parse via `bun build --no-bundle`.
- Provider-definition count now matches generated catalog exactly (433 === 433).
- OpenCode Zen free tier via live curl: `/zen/v1/models` returns 64 models;
  `POST /chat/completions` with `x-preview-f-free` and `nemotron-3.5-lightning-free`
  returned HTTP 200 with `"cost":"0"`; `big-pickle` rate-limited (429) under load.

### Consumed downstream (dx-code)

- **dx-code `assets/settings/default.json`**: generated and merged 223
  `openai_compatible` + 10 `anthropic_compatible` provider entries from this
  repo's registry catalogs — dx-code now ships with 240 additional configured
  providers out of the box.
- **dx-code `crates/opencode`**: added free-tier models `x-preview-f-free`
  (DX Alpha Preview — free-unlimited launch offer, new default free model),
  `nemotron-3.5-lightning-free`, `laguna-s-2.1-free`,
  `muse-spark-1.2-contributor-free`; all use `Authorization: Bearer public`.

### Known issues

- 26 providers still lack icons (see TODO.md).
- ~110 orphan icons remain in `public/providers/` without definitions.
- Possible id collisions between OmniRoute and Router variants
  (`openference` vs `openference-api`, `devin` vs `devin-cli`).

## 0.1.0 (2026-07-10)

- Initial monorepo scaffolding
