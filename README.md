<div align="center">

<img src="assets/icons/dx_web_tools/www-dark-transparent.svg" alt="Dx Code" width="120"/>

<br/>

# 🚀 Dx Code — the DX code editor

**An AI-native code editor built on Zed, with DX web workspaces, agent panels and local-model providers.**

[![Repo](https://img.shields.io/badge/github-essencefromexistence--dx%2Fdx-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/essencefromexistence-dx/dx)
[![Rust](https://img.shields.io/badge/rust-1.98.1-DEA584?style=for-the-badge&logo=rust&logoColor=white)](rust-toolchain.toml)
[![License](https://img.shields.io/badge/license-GPL--3.0--or--later-blue?style=for-the-badge)](LICENSE-GPL)
[![Upstream](https://img.shields.io/badge/upstream-zed%201.30.0-404040?style=for-the-badge)](https://github.com/zed-industries/zed)

<br/>

<table>
  <tr>
    <td align="center"><a href="#-what-it-is">💠 What it is</a></td>
    <td align="center"><a href="#-what-is-inside">🧩 Inside</a></td>
    <td align="center"><a href="#-web-workspaces">🌐 Web workspaces</a></td>
    <td align="center"><a href="#-ai-agents-and-models">🤖 AI, agents &amp; models</a></td>
  </tr>
  <tr>
    <td align="center"><a href="#-quick-start">⚡ Quick start</a></td>
    <td align="center"><a href="#-repository-map">🗺️ Repository map</a></td>
    <td align="center"><a href="#-upstream-relationship">🔁 Upstream</a></td>
    <td align="center"><a href="#-documentation">📖 Docs</a></td>
  </tr>
</table>

</div>

<br/>

<a id="what-it-is"></a>

<div align="center">

## 💠 What it is

</div>

Dx Code is a fork of the [Zed](https://github.com/zed-industries/zed) editor. It keeps
Zed's GPUI rendering, workspace model and editor workflows, and adds the DX surface:
an AI screen, a web-preview pane that serves the bundled DX web projects, DX panels
(media, search, check, skills, plugins, forge) and first-class local model providers.

The shipped binary is **`dx-code`** (`crates/zed/Cargo.toml:58`). Configuration and
debug artefacts live under **`.dx/code`** rather than Zed's paths
(`crates/paths/src/paths.rs:533`).

| Fact | Value | Source |
| --- | --- | --- |
| Binary | `dx-code` | `crates/zed/Cargo.toml:58` |
| Base version | Zed `1.30.0` | `crates/zed/Cargo.toml:5` |
| Toolchain | Rust `1.98.1` | `rust-toolchain.toml` |
| Workspace | 263 top-level crates | `crates/` |
| Authored web projects | 14 under `web/` | `web/` |
| Published web bundles | 15 under `assets/web/` | `assets/web/` |
| License | GPL-3.0-or-later (Zed lineage) | `LICENSE-GPL`, `LICENSE-APACHE` |

> These are repository facts, not release claims. No build, runtime or performance
> result is asserted here — see [Documentation](#-documentation) for how to check
> the tree yourself.

<br/>

<a id="what-is-inside"></a>

<div align="center">

## 🧩 What is inside

</div>

<table>
  <tr><th align="left">Area</th><th align="left">What the tree contains</th><th align="left">Where</th></tr>
  <tr><td align="left" nowrap>🧑‍💻 <b>Editor core</b></td><td align="left">GPUI, workspace, editor, project panel, terminal, settings</td><td align="left"><code>crates/gpui*</code>, <code>crates/editor</code>, <code>crates/workspace</code></td></tr>
  <tr><td align="left" nowrap>🤖 <b>AI screen &amp; agents</b></td><td align="left">Agent profiles, ACP agent options, provider picker, chat surface</td><td align="left"><code>crates/agent*</code>, <code>crates/acp_thread</code></td></tr>
  <tr><td align="left" nowrap>🌐 <b>Web preview</b></td><td align="left">Hosts the bundled static web projects behind DX-local URLs</td><td align="left"><code>crates/web_preview*</code>, <code>assets/web/</code></td></tr>
  <tr><td align="left" nowrap>📦 <b>DX panels</b></td><td align="left">Media, metasearch, check, skills, plugins, forge surfaces</td><td align="left"><code>crates/media_panel</code>, <code>crates/metasearch*</code></td></tr>
  <tr><td align="left" nowrap>🧠 <b>Local providers</b></td><td align="left">llama.cpp, Mistral and Ollama model providers</td><td align="left"><code>crates/llama_cpp</code>, <code>crates/mistral</code>, <code>crates/ollama</code></td></tr>
  <tr><td align="left" nowrap>🎨 <b>DX icons</b></td><td align="left">Icon set used by the web-tool launchers and panels</td><td align="left"><code>assets/icons/dx_web_tools/</code></td></tr>
</table>

<br/>

<a id="web-workspaces"></a>

<div align="center">

## 🌐 Web workspaces

</div>

The AI screen's bottom launcher opens the bundled projects in the web-preview pane.
Each project has an authored source tree under `web/` and a published static bundle
under `assets/web/` (plus `music`, which ships as a bundle).

<div align="center">

<img src="assets/icons/dx_web_tools/3d-dark-transparent.svg" width="36" alt="3D"/>
<img src="assets/icons/dx_web_tools/cms-dark-transparent.svg" width="36" alt="Cms"/>
<img src="assets/icons/dx_web_tools/design-dark-transparent.svg" width="36" alt="Design"/>
<img src="assets/icons/dx_web_tools/graph-dark-transparent.svg" width="36" alt="Graph"/>
<img src="assets/icons/dx_web_tools/graphics-dark-transparent.svg" width="36" alt="Graphics"/>
<img src="assets/icons/dx_web_tools/media-dark-transparent.svg" width="36" alt="Media"/>
<img src="assets/icons/dx_web_tools/metasearch-dark-transparent.svg" width="36" alt="Metasearch"/>
<img src="assets/icons/dx_web_tools/presentations-dark-transparent.svg" width="36" alt="Presentations"/>
<img src="assets/icons/dx_web_tools/router-dark-transparent.svg" width="36" alt="Router"/>
<img src="assets/icons/dx_web_tools/shader-dark-transparent.svg" width="36" alt="Shader"/>
<img src="assets/icons/dx_web_tools/spreadsheets-dark-transparent.svg" width="36" alt="Spreadsheets"/>
<img src="assets/icons/dx_web_tools/train-dark-transparent.svg" width="36" alt="Train"/>
<img src="assets/icons/dx_web_tools/video-dark-transparent.svg" width="36" alt="Video"/>
<img src="assets/icons/dx_web_tools/whiteboard-dark-transparent.svg" width="36" alt="Whiteboard"/>

</div>

<table>
  <tr><th align="left">Source (<code>web/</code>)</th><th align="left">Published bundle (<code>assets/web/</code>)</th></tr>
  <tr><td align="left"><code>3D</code></td><td align="left"><code>3d</code></td></tr>
  <tr><td align="left"><code>Cms</code></td><td align="left"><code>cms</code></td></tr>
  <tr><td align="left"><code>Design</code></td><td align="left"><code>Design</code></td></tr>
  <tr><td align="left"><code>Graph</code></td><td align="left"><code>graph</code></td></tr>
  <tr><td align="left"><code>Graphics</code></td><td align="left"><code>Graphics</code></td></tr>
  <tr><td align="left"><code>Media</code></td><td align="left"><code>media</code></td></tr>
  <tr><td align="left"><code>Metasearch</code></td><td align="left"><code>metasearch</code></td></tr>
  <tr><td align="left"><code>Presentations</code></td><td align="left"><code>Presentations</code></td></tr>
  <tr><td align="left"><code>Router</code></td><td align="left"><code>router</code></td></tr>
  <tr><td align="left"><code>Shader</code></td><td align="left"><code>shader</code></td></tr>
  <tr><td align="left"><code>Spreadsheets</code></td><td align="left"><code>Spreadsheets</code></td></tr>
  <tr><td align="left"><code>Train</code></td><td align="left"><code>Train</code></td></tr>
  <tr><td align="left"><code>Video</code></td><td align="left"><code>Video</code></td></tr>
  <tr><td align="left"><code>Whiteboard</code></td><td align="left"><code>whiteboard</code></td></tr>
</table>

<br/>

<a id="ai-agents-and-models"></a>

<div align="center">

## 🤖 AI, agents and models

</div>

| Capability | Notes |
| --- | --- |
| Agent profiles | Agent / Search / Media / Ask surfaces in the AI screen (`AI_PROFILES.md`) |
| ACP agents | Claude, Codex, Cursor, GitHub Copilot and OpenCode options in the agent dropdown |
| Local model providers | `llama_cpp`, `mistral`, `ollama` crates |
| OpenCode Go model | Endpoint, headers and env are documented in [`OPENCODE.md`](OPENCODE.md) |
| Check → AI | Check-panel diagnostics can be handed to the AI panel |
| Media → Metasearch | Media panel is wired to the metasearch workflow |

Provider endpoints and credentials stay caller-owned. See [`OPENCODE.md`](OPENCODE.md)
for the exact request shape used by the OpenCode Go model.

<br/>

<a id="quick-start"></a>

<div align="center">

## ⚡ Quick start

</div>

The tree pins Rust **1.98.1** in `rust-toolchain.toml`. From the repository root:

```powershell
# Show the available build recipes
just --list

# Recommended local loop: build the dx-code binary, launch it, copy it to G:\Dx\bin
just run

# Build without launching
just build

# Direct Cargo equivalent used by the recipes
cargo build --locked -p zed --bin dx-code
```

> The `just` recipes guard the build with a free-space check (18 GB for a fresh
> build, 4 GB incremental) and pin Cargo output to `G:\Dx\code\target`. On the
> Windows GNU toolchain, `WebView2Loader.dll` must sit next to `dx-code`; the
> recipes copy it from the cached `webview2-com-sys` build output when possible.

Built binaries are placed in `G:\Dx\bin\` by `just run`. The editor writes its
DX-specific state under `.dx/code`.

<br/>

<a id="repository-map"></a>

<div align="center">

## 🗺️ Repository map

</div>

| Path | Purpose |
| --- | --- |
| `crates/` | Rust workspace — 263 top-level crates (editor, GPUI, agents, providers) |
| `web/` | 14 authored web projects (Next.js and DX www framework projects) |
| `assets/` | Published static web bundles, icons and licenses |
| `assets/icons/dx_web_tools/` | DX web-tool icon set used by the AI-screen launcher |
| `docs/src/` | Upstream Zed documentation carried with the fork |
| `merge-tools/` | Upstream-sync evidence and tooling |
| `script/`, `tooling/`, `ci/`, `nix/`, `.github/` | Build, verification and CI helpers |
| `justfile`, `.cargo/` | Local build recipes and Cargo target/linker config |
| `Cargo.toml`, `Cargo.lock` | Workspace manifest and pinned lockfile |
| `CHANGELOG.md`, `TODO.md` | Merge/sync history and open work |

<br/>

<a id="upstream-relationship"></a>

<div align="center">

## 🔁 Upstream relationship

</div>

This repository tracks [`zed-industries/zed`](https://github.com/zed-industries/zed)
through the `upstream` remote. The last integrated upstream tip is recorded in the
merge commits; `CHANGELOG.md` and `TODO.md` keep the per-sync conflict resolutions
and the open items.

| Remote | URL | Role |
| --- | --- | --- |
| `origin` | `https://github.com/essencefromexistence-dx/dx.git` | this fork |
| `upstream` | `https://github.com/zed-industries/zed.git` | Zed source |

When working on upstream syncs, keep the fork's DX surface intact and resolve
conflicts in the repo that owns the file. Historical merge notes such as
`MERGE.md` and `MERGE-CONFLICTS-20260924.md` document earlier syncs — they are
records, not current status.

<br/>

<a id="documentation"></a>

<div align="center">

## 📖 Documentation

</div>

<table>
  <tr><th align="left">Document</th><th align="left">What it covers</th></tr>
  <tr><td nowrap><b><a href="OPENCODE.md">OPENCODE.md</a></b></td><td>OpenCode Go model endpoint, headers and environment</td></tr>
  <tr><td nowrap><b><a href="AI_PROFILES.md">AI_PROFILES.md</a></b></td><td>AI screen profile notes</td></tr>
  <tr><td nowrap><b><a href="DX_DESKTOP.md">DX_DESKTOP.md</a></b></td><td>Historical desktop/web-preview analysis</td></tr>
  <tr><td nowrap><b><a href="docs/src/ai/sandboxing.md">Sandboxing</a></b></td><td>Upstream Zed sandboxing notes</td></tr>
  <tr><td nowrap><b><a href="docs/src/development/ui-checklist.md">UI checklist</a></b></td><td>Upstream UI review checklist</td></tr>
  <tr><td nowrap><b><a href="docs/src/extensions/publishing/overview.md">Extension publishing</a></b></td><td>Upstream extension publishing overview</td></tr>
  <tr><td nowrap><b><a href="CHANGELOG.md">CHANGELOG.md</a></b></td><td>Sync and change history</td></tr>
  <tr><td nowrap><b><a href="TODO.md">TODO.md</a></b></td><td>Open work, with evidence and done-when</td></tr>
  <tr><td nowrap><b><a href="justfile">justfile</a></b></td><td>Authoritative local build/launch recipes</td></tr>
</table>

<br/>

<div align="center">

## 📄 License

Zed-lineage code is licensed **GPL-3.0-or-later** (`LICENSE-GPL`); additional
license notices are in `LICENSE-APACHE` and `legal/`.

**[⬆ Back to top](#-dx-code--the-dx-code-editor)**

</div>
