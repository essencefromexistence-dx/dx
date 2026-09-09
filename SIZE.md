# SIZE.md — dx-desktop binary-size record + Linux rebuild playbook

Read this AND `TUI_OPENCODE_AGENT_HELPER.md` before building. This file is the
size story: what was measured, where every megabyte lives, and the exact
recipe that gets Linux under 100MB.

## 1. Measured numbers (do not argue with these, re-measure instead)

Windows x64, GNU toolchain (`objdump -h` on the exe):

| Build | exe bytes | `.text` (code) | `.rdata` (ro-data + embeds) | unwind | Installer (lzma2/max) |
|---|---|---|---|---|---|
| Start | 537,520,237 | ~248MB | ~204MB (~178MB embedded assets) | ~23MB | — |
| glass + web out, O3 | 391,245,051 | ~253MB | ~38MB | ~20MB | 139,468,968 (~133 MiB) |
| + opt-z/abort/nodebug | 224,874,257 | ~83MB | ~38MB | ~5MB | 95,493,913 (~91 MiB) |

Linux x64 (measured 2026-09-06, native WSL Ubuntu 24.04 build): **284,716,344
bytes (~271.6MB)** after §4 rewiring (was 415,473,720 ≈ 395.9MB with the
135MB web embed — delta −130,757,376 bytes; hashed asset names grep 0× in
the new binary, embed confirmed gone). Same app: ~108MB code + ~31MB unwind
+ ~24MB reloc/GOT + static libwebrtc / tree-sitter / wasmtime floor.
Shipped as `G:\Dx\bin\dx-desktop-linux-x86_64`, smoke-tested (clap error on
bad flag proves loader + symbols resolve).

Rule: `.rdata` holds ALL rust-embedded assets. A 135MB embed cannot hide
outside `.rdata` — if `.rdata` is small, the embed is gone. Trust sections
over assumptions.

## 2. The complete embed map (all of it, with gates and status)

| # | Location | Folder | Gate | Status |
|---|---|---|---|---|
| 1 | `crates/assets/src/assets.rs` (`Assets`) | `assets/` incl. `web/**`, ex-liquid_glass | none (all platforms) | `web/**` + `liquid_glass/**` REMOVED. Keep it that way |
| 2 | `crates/web_preview/src/web_preview_view.rs` (~L34475, `WebProjectsAssets`, `dxcode://` handler) | `../../assets/web/` (135MB) | `#[cfg(target_os = "macos")]` — compiled OUT on Windows/Linux | needs disk-first rewiring for macOS builds |
| 3 | `crates/web_preview_macos/.../web_preview_view.rs` (~L2627) | `../../assets/web/` (135MB) | macOS-only crate | needs disk-first rewiring for macOS builds |
| 4 | `crates/web_preview_linux/.../web_preview_view.rs` (~L90, `#[cfg(target_os = "linux")]`) | `../../assets/web/` (135MB) | Linux-only crate, added to make Linux compile | **REWIRE THIS FIRST (Linux)** |
| 5 | `crates/grammars`, `edit_prediction_cli` (2x), `settings`, `agent/templates` | small prompts/grammars | various | audited, small, leave alone |

Smaller embeds (#5) are fine. The ONLY large embeds are #1 (done) and #2–#4.

## 3. Why the Linux binary ended up 415MB

1. **135MB live embed (#4).** The in-app `dxcode://` handler serves ONLY via
   `WebProjectsAssets::get()` — no disk fallback (contrast `server.rs`, which
   has `load_disk_preview_file` first). The embed was added to make Linux
   compile. Correct fix below, not removal-at-all-costs blindly.
2. **Profile wins possibly unclaimed.** The size profile lives in UNCOMMITTED
   `Cargo.toml` (`opt-level="z"`, `panic="abort"`, `debug=false`, per-package
   `codegen-units = 16` for zed/git_ui/project/project_panel/agent_ui/
   workspace). A tree cloned fresh from git does NOT have it — that alone is
   ~108MB code vs 83MB and ~31MB unwind vs 5MB. Check before building (§5).
3. **Structural floor (~230MB).** Static libwebrtc, ~30 tree-sitter grammars,
   wasmtime, relocations. Moves only by deleting features — out of scope.

## 4. Disk-first rewiring (the actual fix, all platforms)

Pattern to copy: `web_preview/src/server.rs::load_disk_preview_file` +
`find_assets_web_root()` (checks `$DX_ASSETS_WEB_ROOT`, `<cwd>/assets/web`,
`<exe-dir>/assets/web`, walking upward). Apply to each `dxcode://` handler
(#2–#4): try disk first, then DELETE the `WebProjectsAssets` derive entirely.
No feature flag — bin/installer always ships `assets/` next to the binary, so
the fallback never fires in practice. If the exe runs without its `assets/`
folder, previews 404: accepted tradeoff, document it, do not re-embed.
Expected: Linux 415MB → ~280MB (then §5 profile if missing → ~230MB).

## 5. Linux rebuild pre-flight (do NOT skip — a blind build costs hours)

From the tree you are about to build, verify ALL of these:

```bash
grep -A6 '\[profile.release\]' Cargo.toml   # opt-z, abort, debug=false, lto thin
grep 'codegen-units = 16' Cargo.toml        # zed, git_ui, project, project_panel, agent_ui, workspace
test ! -e crates/liquid_glass && test ! -e assets/liquid_glass  # glass gone
grep -c 'include = "web' crates/assets/src/assets.rs  # must print 0
git grep -n 'WebProjectsAssets' -- crates | grep -v gpui  # only remaining disk-first TODOs, no live embeds
```

If any check fails, sync the working tree from `G:\Dx\desktop` first (the
uncommitted changes are the whole game).

## 6. Linux build + measure + pack

```bash
./script/linux && ./script/download-wasi-sdk   # needs sudo/network
# WSL RAM ≤14GB → jobs=2. The CGU=16 overrides are what make giants fit.
CARGO_BUILD_JOBS=2 cargo build --release --locked -p zed --bin dx-desktop
```

Then measure, in this order, and report ALL THREE numbers:

1. `ls -l target/release/dx-desktop` (expect ~280MB after §4, ~230MB with §5)
2. `objdump -h` sections (`.text` / `.rdata` sanity vs §1 table)
3. Confirm no 135MB object: the `web_preview_linux` rlib must NOT contain a
   ~135,013,500-byte object; `grep -c <hashed-name>` on the binary must be 0.

Pack: `UPX --lzma` the binary (Linux UPX is safe; ~30% ratio proven here) →
expect sub-100MB. Ship `assets/web` next to it (135MB on disk, exe-relative
resolution already exists). Copy the packed binary to
`/mnt/g/Dx/bin/dx-desktop-linux-x86_64`.

## 7. Standing constraints

- CircleCI = macOS only. No Linux CI jobs, no `dev` pushes (auto-triggers CI).
- Never re-add `liquid_glass`, never re-embed `assets/web` anywhere.
- Never touch GPUI `LiquidGlass` renderer files — unrelated upstream tech.
- Size work without before/after byte counts does not count.
