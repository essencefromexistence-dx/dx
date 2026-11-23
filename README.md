<div align="center">

<img src="assets/icons/dx-icon.svg" alt="Dx" width="120"/>

<br/>

# Dx: Enhanced Development Experience

**One company. One toolchain. Every surface of the development workflow.**

[![Organization](https://img.shields.io/badge/github-essencefromexistence--dx-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/essencefromexistence-dx)
[![Repositories](https://img.shields.io/badge/repositories-32-5c4ee5?style=for-the-badge)](https://github.com/essencefromexistence-dx?tab=repositories)
[![Editor binary](https://img.shields.io/badge/editor-dx--code-0A66C2?style=for-the-badge)](crates/zed/Cargo.toml)
[![Rust](https://img.shields.io/badge/rust-1.98.1-DEA584?style=for-the-badge&logo=rust&logoColor=white)](rust-toolchain.toml)
[![See on Product Hunt](https://img.shields.io/badge/See%20on%20Product%20Hunt-Dx-DA552F?style=for-the-badge&logo=producthunt&logoColor=white)](https://www.producthunt.com/posts/dx)

<br/>

<table>
  <tr>
    <td align="center"><a href="#-dx-browser">🌐 Dx Browser</a></td>
    <td align="center"><a href="#-all-dx-tools">📦 All DX tools</a></td>
    <td align="center"><a href="#-how-the-tools-fit-together">🔗 How they fit</a></td>
  </tr>
  <tr>
    <td align="center"><a href="#-quick-start">⚡ Quick start</a></td>
    <td align="center"><a href="#-repository-map">🗺️ This repository</a></td>
    <td align="center"><a href="#-documentation">📖 Documentation</a></td>
    <td align="center"><a href="#-license-and-lineage">⚖️ License &amp; lineage</a></td>
  </tr>
</table>

</div>

<br/>

<a id="dx-browser"></a>

<div align="center">

## 🌐 Dx Browser

</div>

**Dx Browser** is the Dx family's Ladybird-based web browser — "Welcome to a
truly calmer internet." Ladybird is an independent engine built from scratch,
not a fork of Blink, WebKit or Gecko ([ladybird.org](https://ladybird.org)).

Its home is
[**essencefromexistence-dx/browser**](https://github.com/essencefromexistence-dx/browser):
[README](https://github.com/essencefromexistence-dx/browser#readme) ·
[Releases](https://github.com/essencefromexistence-dx/browser/releases) ·
[Issues](https://github.com/essencefromexistence-dx/browser/issues) ·
[Discussions](https://github.com/essencefromexistence-dx/browser/discussions).

<br/>

<a id="all-dx-tools"></a>

<div align="center">

## 📦 All DX tools

</div>

### Entry point and developer surfaces

| Tool | Repository | Binary / route | What it does |
| --- | --- | --- | --- |
| **Dx CLI** | [`cli`](https://github.com/essencefromexistence-dx/cli) | `dx` / `dx-cli` | Mission control for the terminal: installs and updates the registered tools, drives media and file browsing, and carries the Codex-grade TUI experiments. It is the hub entry point for the whole toolchain. |
| **Dx Code** | [`dx`](https://github.com/essencefromexistence-dx/dx) | `dx-code` | This repository — the AI-native editor with the bundled web workspaces, agent screens, DX panels and local model providers. |

### Engineering and build tools

| Tool | Repository | Binary / route | What it does |
| --- | --- | --- | --- |
| **Check** | [`check`](https://github.com/essencefromexistence-dx/check) | `dx-check` / `dx check` | Project-health engine. Lint, format, structure, web performance, E2E readiness and launch scoring are collected into one machine-readable receipt. |
| **DCP** | [`dcp`](https://github.com/essencefromexistence-dx/dcp) | `dx-dcp` / `dx dcp` | Development Context Protocol — a binary-first replacement for MCP as the context transport between tools and agents. |
| **Driven** | [`driven`](https://github.com/essencefromexistence-dx/driven) | `dx-driven` / `dx driven` | Professional AI-assisted development orchestrator with a binary-first architecture for repeatable agent workflows. |
| **Forge** | [`forge`](https://github.com/essencefromexistence-dx/forge) | `dx-forge` / `dx forge`, `dx add` | Fast version control for massive media assets — tracks large binary collections that ordinary source control handles badly. |
| **Grimoire** | [`grimoire`](https://github.com/essencefromexistence-dx/grimoire) | `dx-grimoire` / `dx grimoire` | Reverse engineering from the terminal or an agent. One CLI and MCP server for binaries, applications and runtime behavior. |
| **Serializer** | [`serializer`](https://github.com/essencefromexistence-dx/serializer) | `dx-serializer` / `dx serializer` | Token-efficient serialization for LLM context windows, with high-performance binary encoding. |
| **Style** | [`style`](https://github.com/essencefromexistence-dx/style) | `dx-style` / `dx style` | Binary-first CSS engine using zero-copy parsing and DX Serializer output. |
| **Token** | [`token`](https://github.com/essencefromexistence-dx/token) | `dx-token` / `dx token` | Counts tokens across files and folders with multiple tokenizers and a rainbow TUI output. |
| **Tree** | [`tree`](https://github.com/essencefromexistence-dx/tree) | `dx-tree` / `dx tree` | Fast cross-platform directory-tree visualizer for inspecting large project trees. |
| **Script** | [`script`](https://github.com/essencefromexistence-dx/script) | `dx-script` / `dx script` | Command runner for the DX ecosystem — the shared way to invoke DX workflows consistently. |
| **i18n** | [`i18n`](https://github.com/essencefromexistence-dx/i18n) | `dx-i18n` / `dx i18n` | Multi-provider translation engine with Google, DeepL and other translation APIs, plus TTS/STT workflows. |
| **Icon** | [`icon`](https://github.com/essencefromexistence-dx/icon) | `dx-icon` / `dx icon` | Icon search backed by FST indexes and semantic search — the catalog behind DX icon panels. |
| **www** | [`www`](https://github.com/essencefromexistence-dx/www) | `dx-www`; `dx www`, `dx new`, `dx dev`, `dx run`, `dx graph`, `dx deploy` | Rust-owned web framework and server runtime for React and Next-familiar `tsx` apps. |
| **Skilltune** | registry entry only | `dx-skilltune` / `dx skilltune` | Declared in `TOOL_REGISTRY`; no repository is currently listed under the organization. |

### Media, web and communication

| Tool | Repository | Binary / route | What it does |
| --- | --- | --- | --- |
| **Media** | [`media`](https://github.com/essencefromexistence-dx/media) | `dx-media` / `dx media` | Universal digital asset acquisition: one command fetches any media from anywhere into a consistent local representation. |
| **Metasearch** | [`metasearch`](https://github.com/essencefromexistence-dx/metasearch) | `dx-metasearch` / `dx metasearch` | Fast, privacy-respecting metasearch engine in Rust, inspired by SearXNG. |
| **Video player** | [`video-player`](https://github.com/essencefromexistence-dx/video-player) | `dx-video-player` (staged) | Reliable local media playback, including terminal-rendered video. |
| **OS** | [`os`](https://github.com/essencefromexistence-dx/os) | `dx-os` / `dx os` | System monitor with DX branding for Linux, macOS and Windows. |

<br/>

<a id="how-the-tools-fit-together"></a>

<div align="center">

## 🔗 How the tools fit together

</div>

| Mechanism | Where it is defined | What it guarantees |
| --- | --- | --- |
| Installer contract | `TOOL_REGISTRY` in `cli/src/update_registry.rs` | Maps a short tool name to its repository, binary name and install path |
| Install layout | Windows `%LOCALAPPDATA%\dx\bin` | One `dx-<tool>` executable per registered tool |
| Release assets | `dx-<tool>-<rust-target-triple>[.exe]` | The same artifact naming across the published targets |
| Routing | `dx` manifest at the hub root | Maps short names to local folders for development work |
| Achievements | `docs/<tool>.md` in the hub | One verified page per folder, not a claim about the whole company |

A tool can be staged (present in the registry but not yet published), hidden
(routed only through `dx`), or app-only (no registered binary). The tables above
keep those distinctions instead of treating every repository as a shipped CLI.

<br/>

<a id="quick-start"></a>

<div align="center">

## ⚡ Quick start — Dx Code

</div>

The Dx Code tree pins Rust **1.98.1** in `rust-toolchain.toml`.

```powershell
# Show the available build recipes
just --list

# Build the editor, launch it, and copy the binary to G:\Dx\bin
just run

# Build without launching
just build
```

For the rest of the toolchain, install or update through the `dx` CLI — it reads
the same registry that produced the tables above.

<br/>

<a id="repository-map"></a>

<div align="center">

## 🗺️ This repository

</div>

| Path | Purpose |
| --- | --- |
| `assets/` | Published static bundles, icons and license notices |
| `script/`, `tooling/`, `ci/`, `nix/`, `.github/` | Build, verification and CI helpers |
| `justfile`, `.cargo/` | Local build recipes and Cargo target/linker configuration |
| `Cargo.toml`, `Cargo.lock` | Workspace manifest and pinned lockfile |
| `CHANGELOG.md` | Sync history |

<br/>

<a id="documentation"></a>

<div align="center">

## 📖 Documentation

</div>

<table>
  <tr><th align="left">Document</th><th align="left">What it covers</th></tr>
  <tr><td nowrap><b><a href="CHANGELOG.md">CHANGELOG.md</a></b></td><td>Sync and change history for this repository</td></tr>
  <tr><td nowrap><b><a href="justfile">justfile</a></b></td><td>Authoritative local build and launch recipes</td></tr>
</table>

<br/>

<a id="license-and-lineage"></a>

<div align="center">

## ⚖️ License and lineage

</div>

The editor sources in this repository derive from the Zed workspace and are
licensed **GPL-3.0-or-later**; see [`LICENSE-GPL`](LICENSE-GPL),
[`LICENSE-APACHE`](LICENSE-APACHE) and `legal/`. The upstream relationship is
tracked through the `upstream` remote and recorded in `CHANGELOG.md`. Other DX
tools carry their own licensing in their own repositories.

**[⬆ Back to top](#dx-enhanced-development-experience)**

</div>
