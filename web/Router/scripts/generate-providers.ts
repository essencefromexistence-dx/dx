/**
 * generate-providers.ts
 * Reads ALL 347 provider definitions from route/ and generates providers.generated.ts
 * Run: bun run scripts/generate-providers.ts
 */

import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROUTE_DIR = path.resolve(__dirname, "..");
const OUT_FILE = path.resolve(__dirname, "..", "src", "lib", "ai", "providers.generated.ts");

interface ModelEntry {
  id: string;
  name: string;
  contextLength?: number;
}

interface ProviderEntry {
  id: string;
  name: string;
  authType: string;
  authHint: string;
  hasFree: boolean;
  website: string;
  category: string;
  models: ModelEntry[];
}

function extractProviderKeys(content: string): string[] {
  const keys: string[] = [];
  const regex = /^\s{2}(['"])([^'"]+)\1\s*:/gm;
  let m: RegExpExecArray | null;
  while ((m = regex.exec(content)) !== null) {
    keys.push(m[2]);
  }
  // Also match unquoted keys with object value
  const regex2 = /^\s{2}([a-zA-Z_][\w-]*)\s*:\s*\{/gm;
  while ((m = regex2.exec(content)) !== null) {
    if (m[1] !== "default") keys.push(m[1]);
  }
  return [...new Set(keys)];
}

/**
 * Slice the source text of a single top-level provider block:
 * from its `  <id>: {` key line to the matching closing `  },`.
 * Field extraction below is scoped to this slice so a provider missing a
 * field can never inherit a later provider's value (cross-provider bleed).
 */
function extractProviderBlock(content: string, providerId: string): string | null {
  const keyRegex = new RegExp(`^  ['"]?${escapeRegex(providerId)}['"]?\\s*:\\s*\\{`, "m");
  const m = keyRegex.exec(content);
  if (!m) return null;
  const start = m.index;
  let depth = 0;
  const lines = content.slice(start).split("\n");
  let consumed = 0;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i] as string;
    for (const ch of line) {
      if (ch === "{") depth++;
      else if (ch === "}") depth--;
    }
    consumed += line.length + 1;
    if (i > 0 && depth === 0) break;
  }
  return content.slice(start, start + consumed);
}

/** First `field: "value"` (or `'value'`) inside a provider block, or null. */
function blockStringField(block: string, field: string): string | null {
  const regex = new RegExp(`${field}\\s*:\\s*(['"])([\\s\\S]*?)\\1`);
  const m = regex.exec(block);
  return m ? (m[2] as string) : null;
}

function extractProviderName(content: string, providerId: string): string {
  const block = extractProviderBlock(content, providerId);
  if (!block) return providerId;
  return blockStringField(block, "name") || providerId;
}

function extractProviderHint(content: string, providerId: string): string {
  const block = extractProviderBlock(content, providerId);
  if (!block) return "";
  // Prefer the short authHint; fall back to apiHint so providers that only
  // document the endpoint still get their own text instead of a neighbor's.
  return blockStringField(block, "authHint") || blockStringField(block, "apiHint") || "";
}

function extractProviderUrl(content: string, providerId: string): string {
  const block = extractProviderBlock(content, providerId);
  if (!block) return "";
  return blockStringField(block, "website") || "";
}

function hasFreeField(content: string, providerId: string): boolean {
  const block = extractProviderBlock(content, providerId);
  if (!block) return false;
  return /hasFree\s*:\s*true/.test(block);
}

function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function readRegistryModels(): Map<string, ModelEntry[]> {
  const models = new Map<string, ModelEntry[]>();
  const registryDir = path.join(ROUTE_DIR, "route", "open-sse", "config", "providers", "registry");
  if (!fs.existsSync(registryDir)) return models;

  const entries = fs.readdirSync(registryDir, { withFileTypes: true });
  for (const entry of entries) {
    if (!entry.isDirectory()) continue;
    const indexFile = path.join(registryDir, entry.name, "index.ts");
    if (!fs.existsSync(indexFile)) continue;

    const content = fs.readFileSync(indexFile, "utf-8");
    const providerId = entry.name;
    const ctxLenMatch = content.match(/defaultContextLength\s*:\s*(\d+)/);
    const defaultCtx = ctxLenMatch ? Number.parseInt(ctxLenMatch[1]) : undefined;

    const providerModels: ModelEntry[] = [];
    const modelRegex = /\{\s*id\s*:\s*['"]([^'"]+)['"]([\s\S]*?)\}/g;
    let m: RegExpExecArray | null;
    while ((m = modelRegex.exec(content)) !== null) {
      const modelId = m[1];
      const rest = m[2];
      // Skip the provider object itself: its opening `{` also matches
      // `\{\s*id` and would otherwise inject a phantom `{id: <providerId>}`
      // model carrying the first real model's name (e.g. sarvam).
      if (/alias\s*:|format\s*:|executor\s*:|authType\s*/.test(rest)) {
        // The skipped span swallowed the first inline model object at its
        // tail (the match ends at that model's closing `}`). Recover it so
        // the first catalog model is not silently dropped.
        if (/models\s*:\s*\[/.test(m[0])) {
          const openIdx = m[0].lastIndexOf("{");
          const tail = openIdx >= 0 ? m[0].slice(openIdx) : "";
          // The tail is a real model object only when it carries no
          // provider-level keys (otherwise it is the provider itself, e.g.
          // `models: [...SPREAD]`, or an alternateFormats entry).
          if (!/alias\s*:|format\s*:|executor\s*:|authType\s*|baseUrl\s*/.test(tail)) {
            const tailId = tail.match(/id\s*:\s*['"]([^'"]+)['"]/)?.[1];
            if (tailId) {
              const tailName = tail.match(/name\s*:\s*(['"])([\s\S]*?)\1/)?.[2];
              const tailCtx = tail.match(/contextLength\s*:\s*(\d+)/);
              providerModels.push({
                id: tailId,
                name: tailName || tailId,
                contextLength: tailCtx ? Number.parseInt(tailCtx[1]) : defaultCtx,
              });
            }
          }
        }
        continue;
      }
      const nameMatch = rest.match(/name\s*:\s*(['"])([\s\S]*?)\1/);
      const ctxMatch = rest.match(/contextLength\s*:\s*(\d+)/);
      providerModels.push({
        id: modelId,
        name: nameMatch?.[2] || modelId,
        contextLength: ctxMatch ? Number.parseInt(ctxMatch[1]) : defaultCtx,
      });
    }
    if (providerModels.length > 0) {
      models.set(providerId, providerModels);
    } else if (content.includes("passthroughModels: true")) {
      models.set(providerId, [{ id: `${providerId}/default`, name: `${providerId} (dynamic)` }]);
    }
  }
  return models;
}

const CATEGORIES: { file: string; authType: string; category: string }[] = [
  { file: "src/shared/constants/providers/noauth.ts", authType: "noauth", category: "NoAuth" },
  { file: "src/shared/constants/providers/oauth.ts", authType: "oauth", category: "OAuth" },
  {
    file: "src/shared/constants/providers/web-cookie.ts",
    authType: "web-cookie",
    category: "WebCookie",
  },
  {
    file: "src/shared/constants/providers/apikey/gateways.ts",
    authType: "apikey",
    category: "APIKey",
  },
  {
    file: "src/shared/constants/providers/apikey/frontier-labs.ts",
    authType: "apikey",
    category: "APIKey",
  },
  {
    file: "src/shared/constants/providers/apikey/inference-hosts.ts",
    authType: "apikey",
    category: "APIKey",
  },
  {
    file: "src/shared/constants/providers/apikey/enterprise-cloud.ts",
    authType: "apikey",
    category: "APIKey",
  },
  {
    file: "src/shared/constants/providers/apikey/regional.ts",
    authType: "apikey",
    category: "APIKey",
  },
  {
    file: "src/shared/constants/providers/apikey/specialty-media.ts",
    authType: "apikey",
    category: "APIKey",
  },
  { file: "src/shared/constants/providers/local.ts", authType: "local", category: "Local" },
  { file: "src/shared/constants/providers/search.ts", authType: "apikey", category: "Search" },
  { file: "src/shared/constants/providers/audio.ts", authType: "apikey", category: "Audio" },
  {
    file: "src/shared/constants/providers/upstream-proxy.ts",
    authType: "proxy",
    category: "Proxy",
  },
  {
    file: "src/shared/constants/providers/cloud-agent.ts",
    authType: "apikey",
    category: "CloudAgent",
  },
  { file: "src/shared/constants/providers/system.ts", authType: "system", category: "System" },
];

function main() {
  const registryModels = readRegistryModels();
  console.log(`Read models for ${registryModels.size} providers from registry`);

  const seenIds = new Set<string>();
  const allProviders: ProviderEntry[] = [];

  for (const { file: relFile, authType, category } of CATEGORIES) {
    const filePath = path.join(ROUTE_DIR, relFile);
    if (!fs.existsSync(filePath)) continue;
    const content = fs.readFileSync(filePath, "utf-8");
    const keys = extractProviderKeys(content);

    for (const id of keys) {
      if (seenIds.has(id)) continue;
      seenIds.add(id);

      const models = registryModels.get(id) || [
        { id: `${id}/default`, name: extractProviderName(content, id) },
      ];

      allProviders.push({
        id,
        name: extractProviderName(content, id),
        authType,
        authHint: extractProviderHint(content, id),
        hasFree: hasFreeField(content, id),
        website: extractProviderUrl(content, id),
        category,
        models,
      });
    }
  }

  console.log(`Generated ${allProviders.length} unique providers`);

  const output = generateOutput(allProviders);
  fs.mkdirSync(path.dirname(OUT_FILE), { recursive: true });
  fs.writeFileSync(OUT_FILE, output);
  console.log(`Written to ${OUT_FILE}`);
}

function generateOutput(providers: ProviderEntry[]): string {
  const lines: string[] = [
    "// Auto-generated by scripts/generate-providers.ts",
    "// Do not edit manually. Run: bun run scripts/generate-providers.ts",
    "",
    `import type { ComponentType } from "react"`,
    "import {",
    "  Bot, Box, Brain, Cloud, Code, Database,",
    "  Gem, GitBranch, Globe, Image, Key, Monitor, Music, Pen,",
    "  Rocket, Search, Sliders, Sparkles, Star, Terminal, User, Users, Volume2, Wind, Zap,",
    `} from "lucide-react"`,
    "",
    `export type AuthType = "noauth" | "oauth" | "apikey" | "web-cookie" | "local" | "proxy" | "system"`,
    `export type ProviderCategory = "NoAuth" | "OAuth" | "WebCookie" | "APIKey" | "Local" | "Search" | "Audio" | "Proxy" | "CloudAgent" | "System"`,
    "",
    "export interface GeneratedModelConfig {",
    "  id: string",
    "  name: string",
    "  contextLength?: number",
    "}",
    "",
    "export interface GeneratedProviderConfig {",
    "  id: string",
    "  name: string",
    "  icon: ComponentType<{ className?: string }>",
    "  authType: AuthType",
    "  authHint: string",
    "  hasFree: boolean",
    "  website: string",
    "  category: ProviderCategory",
    "  models: GeneratedModelConfig[]",
    "  defaultModel: string",
    "}",
    "",
    "function ic(name: string): ComponentType<{ className?: string }> {",
    "  const map: Record<string, ComponentType<{ className?: string }>> = {",
    "    Bot, Box, Brain, Cloud, Code, Database, Gem, GitBranch, Globe, Image, Key,",
    "    Monitor, Music, Pen, Rocket, Search, Sliders, Sparkles, Star, Terminal, User, Users, Volume2, Wind, Zap,",
    "  }",
    "  const k = name.charAt(0).toUpperCase() + name.slice(1)",
    "  return map[k] || Box",
    "}",
    "",
    "export const GENERATED_PROVIDERS: Record<string, GeneratedProviderConfig> = {",
  ];

  for (const p of providers) {
    const iconName = pickIcon(p);
    const defaultModel = p.models[0]?.id || `${p.id}/default`;

    lines.push(`  "${p.id}": {`);
    lines.push(`    id: ${JSON.stringify(p.id)},`);
    lines.push(`    name: ${JSON.stringify(p.name)},`);
    lines.push(`    icon: ic(${JSON.stringify(iconName)}),`);
    lines.push(`    authType: ${JSON.stringify(p.authType)},`);
    lines.push(`    authHint: ${JSON.stringify(p.authHint)},`);
    lines.push(`    hasFree: ${p.hasFree},`);
    lines.push(`    website: ${JSON.stringify(p.website)},`);
    lines.push(`    category: ${JSON.stringify(p.category)},`);
    lines.push("    models: [");
    for (const m of p.models) {
      lines.push(
        `      { id: ${JSON.stringify(m.id)}, name: ${JSON.stringify(m.name)}, contextLength: ${m.contextLength ?? "undefined"} },`,
      );
    }
    lines.push("    ],");
    lines.push(`    defaultModel: ${JSON.stringify(defaultModel)},`);
    lines.push("  },");
  }

  lines.push("}");
  lines.push("");
  lines.push("export const GENERATED_PROVIDER_LIST = Object.values(GENERATED_PROVIDERS)");
  lines.push(`export const DEFAULT_PROVIDER_ID = "opencode"`);
  lines.push(`export const DEFAULT_MODEL_ID = "mimo-v2.5-free"`);
  lines.push("");

  return lines.join("\n");
}

function pickIcon(p: ProviderEntry): string {
  const id = p.id;
  const cat = p.category;

  if (id === "opencode" || id === "opencode-zen") return "Terminal";
  if (id === "openai") return "Sparkles";
  if (id === "anthropic" || id === "claude" || id === "claude-web") return "Bot";
  if (id === "gemini" || id === "gemini-web" || id === "gemini-business") return "Gem";
  if (id === "deepseek" || id === "deepseek-web") return "Brain";
  if (id === "github" || id === "github-models" || id === "gitlab" || id === "gitlab-duo")
    return "GitBranch";
  if (id === "azure-openai" || id === "bedrock" || id === "vertex" || id === "vertex-partner")
    return "Cloud";
  if (id === "runwayml" || id === "haiper" || id === "leonardo" || id === "pollinations")
    return "Image";
  if (
    cat === "Audio" ||
    id === "deepgram" ||
    id === "assemblyai" ||
    id === "elevenlabs" ||
    id === "cartesia" ||
    id === "playht" ||
    id === "inworld" ||
    id === "aws-polly" ||
    id === "soniox"
  )
    return "Music";
  if (
    id === "perplexity" ||
    id === "perplexity-search" ||
    id === "perplexity-web" ||
    id === "exa-search" ||
    id === "tavily-search" ||
    id === "brave-search" ||
    id === "google-pse-search" ||
    id === "youcom-search" ||
    id === "searxng-search"
  )
    return "Search";
  if (id === "openrouter" || id === "synthetic" || id === "requesty") return "Globe";
  if (
    cat === "Local" ||
    id === "ollama" ||
    id === "ollama-local" ||
    id === "ollama-cloud" ||
    id === "ollama-search" ||
    id === "lm-studio" ||
    id === "comfyui" ||
    id === "sdwebui"
  )
    return "Monitor";
  if (cat === "NoAuth") return "Sparkles";
  if (cat === "OAuth") return "Users";
  if (cat === "WebCookie") return "Globe";
  if (cat === "Search") return "Search";
  if (cat === "Local") return "Monitor";
  if (cat === "System") return "Zap";
  return "Key";
}

main();
