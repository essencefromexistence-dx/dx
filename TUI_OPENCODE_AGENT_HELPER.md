# TUI_OPENCODE_AGENT_HELPER — dx-code builds on THIS machine (Windows host)

Audience: an AI agent running in WSL/Linux on this same box whose job is to
build dx-code for **Linux** here. Read all of this before touching anything.

## 0. Hard rules (do not break these)

1. **CircleCI is reserved for macOS only.** Do NOT create, edit, push, or
   trigger any CircleCI Linux job. Never `git push` the `dev` branch to the
   `essencefromexistence24/desktop` remote — a push auto-triggers CircleCI and
   burns paid credits.
2. **Work stays local and uncommitted** unless the user explicitly says commit.
   The tree currently has uncommitted changes (see §5) — do not lose them.
3. **G: drive is the workspace disk and regularly hits 0 bytes free.**
   Check `Get-PSDrive G` (Windows) / `df -h` before any big build. A full
   release build peaks around ~33GB of `target/` usage.
4. Long builds: launch detached and poll with SHORT checks. Aborting the wait
   kills the build (Ctrl-C exit, safe to resume) — every abort restarts hours
   of work. Never leave a `Start-Sleep 300` style wait running.

## 1. Host facts (verified, not guessed)

- OS: Windows 11, CPU Ryzen 5 5600G (6 cores / 12 threads), RAM 23.3GB.
- Pagefile: ~1.1GB on C: → total commit limit only ~24.5GB. (C: itself is
  nearly full, ~245MB free — do not put anything big on C:.)
- Workspace disk G: (`G:\Dx\desktop` repo, `G:\Dx\bin` output binaries).
- Rust: 1.95.0 `x86_64-pc-windows-gnu` (GNU toolchain, MinGW `dlltool` at
  `G:\Dev\Tools\Scoop\apps\mingw\current\bin`), pinned by `rust-toolchain.toml`.
  `size.exe` / `nm.exe` / `objdump.exe` in that same MinGW bin dir work for
  measuring the exe (section sizes).
- Package manager available: Scoop (`G:\Dev\Tools\Scoop`). No winget, no
  Docker. Inno Setup 6.7.3 was installed via
  `scoop bucket add nonportable && scoop install innosetup-np`
  → `ISCC.exe` at `G:\Dev\Tools\Scoop\apps\innosetup-np\current\ISCC.exe`.

## 2. How the Windows release was built (exact recipe)

Binary built: `G:\Dx\bin\dx-code.exe` = 224,874,257 bytes (~214 MiB).
Installer: `G:\Dx\bin\DxCode-Setup-x86_64.exe` = 95,493,913 bytes (~91 MiB).

Release profile (`Cargo.toml` `[profile.release]`, already in tree):

```toml
[profile.release]
debug = false
lto = "thin"
opt-level = "z"
panic = "abort"      # NOTE: panics abort the process instead of unwinding
codegen-units = 1

[profile.release.package.git_ui]
codegen-units = 16

[profile.release.package]
zed = { codegen-units = 16 }
project = { codegen-units = 16 }
project_panel = { codegen-units = 16 }
agent_ui = { codegen-units = 16 }
workspace = { codegen-units = 16 }
```

Build command (PowerShell, repo root). Jobs=4 + incremental=1 is the proven
working combo on this box. Do NOT exceed 4 jobs — jobs=12 OOMs (see §4):

```powershell
$env:CARGO_BUILD_JOBS = "4"
$env:CARGO_INCREMENTAL = "1"
cargo build --release --locked -p zed --bin dx-code
```

For builds longer than a few minutes, launch detached via WMI so a dropped
tool call does not kill it (do NOT use `cmd /c "set X=Y && ..."` — cmd folds
the space before `&&` into the value, e.g. jobs becomes `"12 "` and cargo
rejects it; use PowerShell `$env:` assignments instead):

```powershell
$r = Invoke-CimMethod -ClassName Win32_Process -MethodName Create -Arguments @{
  CommandLine = 'powershell -NoProfile -Command "$env:CARGO_BUILD_JOBS = ''4''; $env:CARGO_INCREMENTAL = ''1''; cargo build --release --locked -p zed --bin dx-code > G:\Temp\UserTemp\opencode\dxb.log 2>&1"'
  CurrentDirectory = 'G:\Dx\desktop'
}
# poll with: Get-Process cargo, rustc ; Get-Content <log> -Tail 3
```

After link, stage runtime files next to the exe (GNU toolchain needs the DLL;
MSVC links it statically — do not skip this or launch dies with 0xC0000135):

```powershell
$dto = "target/release"
$src = Get-ChildItem "$dto/build/webview2-com-sys-*/out/x64/WebView2Loader.dll" |
  Select-Object -First 1 -ExpandProperty FullName
Copy-Item $src "$dto/WebView2Loader.dll" -Force
```

Installer (`crates/zed/resources/windows/dx-code.iss`, AppId already inside):
ships exe + DLL + `assets/web/**`, `lzma2/max`, solid. Compile with ISCC;
output lands in `target/DxCode-Setup-x86_64.exe`. Copy both artifacts to
`G:\Dx\bin`.

## 3. Size history (measured with objdump, all real numbers)

- Start: 537,520,237 bytes exe (`.text` 248MB code, `.rdata` 204MB of which
  ~178MB rust-embedded assets, `.xdata` ~20MB unwind).
- Removed unused `liquid_glass` crate + 36MB `assets/liquid_glass` (−36MB).
- Un-embedded 135MB `assets/web` — serve from disk, see §6 (−135MB).
- `opt-level="z"` + `panic="abort"` + `debug=false` (`.text` 253MB → 83MB,
  unwind 20MB → 5MB).
- Result: exe 224,874,257 bytes (−58%), installer 95,493,913 bytes (<100MB ✓).

## 4. Failure modes already hit (so you don't repeat them)

- **OOM (`memory allocation failed`, `STATUS_STACK_BUFFER_OVERRUN`):** one
  rustc with `codegen-units=1` + thin LTO on a giant crate needs 10–15GB.
  Fix = per-package `codegen-units = 16` (already in tree for the 5 giants).
  Never run release with jobs > 4 on this box.
- **Disk full (`os error 112`):** release peaks ~33GB. Worse, ENOSPC
  mid-write leaves TRUNCATED rlibs that cargo trusts → later `E0463: can't
  find crate for X` cascades. Fix = `cargo clean -p <the-E0463-named-crates>
  --release` (e.g. workspace, ui, gpui, ui_input, theme_settings), never a
  blind full clean (a cold rebuild needs 33GB free that we don't have).
- **Flag flips invalidate fingerprints:** changing jobs is free, but flipping
  `CARGO_INCREMENTAL`, profile options, or toolchain = FULL rebuild. Pick
  flags once per build campaign.
- **Killed builds are safe** (Ctrl-C exit, resume keeps cache). Disk-full
  kills are NOT safe (see E0463 above).
- `cargo check --locked -p zed` (≈5 min warm) and `cargo fmt --check` are the
  cheap gates. Run them before any release build.

## 5. Current tree state (must be present for the Linux build)

Uncommitted working-tree changes you must NOT lose: `liquid_glass` crate +
assets deletion, `assets.rs` embed removals (`liquid_glass`, `web`),
settings schema/UI removals, `[profile.release]` tuning + per-package CGU
overrides, `dx-code.iss`, `.circleci/config.yml` (linux-only, currently
failing on apt mirrors — ignore it, Linux is built HERE, not on CI).

## 6. Web assets: disk-serving for the axum/embedded server; PLATFORM embeds still exist

`web_preview/src/server.rs::load_disk_preview_file` tries disk BEFORE embedded
bytes. `find_assets_web_root()` resolves, in order: `$DX_ASSETS_WEB_ROOT`,
`<cwd>/assets/web` (walked upward), `<exe-dir>/assets/web` (walked upward),
then hardcoded dev fallbacks. So: ship an `assets/web` directory next to the
binary and all 9 sites (Train, Video, Design, 3D, …) work with no code
changes. The 135MB in-repo `assets/web` tree is the source of truth — copy it,
do not re-download it.

⚠️ CORRECTION (2026-09-05): removing `web/**/*` from `crates/assets` did NOT
fully un-embed the tree. Three SEPARATE `WebProjectsAssets` RustEmbed derives
still bake all of `assets/web` (135MB each) into platform binaries:
- `crates/web_preview/src/web_preview_view.rs` (~line 34475, inside
  `#[cfg(target_os = "macos")] fn create_native_preview_for_macos_window`,
  `dxcode://` custom-protocol handler) — macOS only, compiled OUT on Windows.
- `crates/web_preview_macos/src/web_preview_view.rs` (~line 2627).
- `crates/web_preview_linux/src/web_preview_view.rs` (~line 90, `#[cfg(target_os
  = "linux")]`) — added to make Linux compile; this is the 135MB inside the
  415MB Linux binary.
Windows is VERIFIED clean: measured `.rdata` is 38MB total, so no 135MB embed
can be hiding there. The remaining rewiring job (= the real size win on
Linux/macOS) is: make those three `dxcode://` handlers disk-first via
`find_assets_web_root()` (same pattern as `server.rs`), then DELETE the
`WebProjectsAssets` embeds entirely — no feature flag needed since the
installer/bin always ships `assets/` next to the binary. If the exe ever runs
without its `assets/` folder, previews 404 — that is the accepted tradeoff
(document it, don't silently re-embed).

## 7. WSL state on this box (your build environment)

- WSL2. Distro `Ubuntu-24.04` is registered but STOPPED and broken: its disk
  `G:\Wsl\Ubuntu24\ext4.vhdx` is missing (`Wsl/Service/CreateInstance/
  MountDisk/HCS/ERROR_PATH_NOT_FOUND`). `G:\Wsl` does not exist.
- C: is full (~245MB free) → the replacement vhdx MUST live on G:
  (tens of GB free at last check — re-verify, release needs ~35GB peak).
- No Docker. `wsl --install` needs Store/network; otherwise
  `wsl --import <Name> G:\Wsl\<Name>\ <rootfs.tar.gz>`.
- Suggested `.wslconfig` (`%USERPROFILE%\.wslconfig`, does not exist yet):
  12–14GB RAM for the distro (host has 23GB), e.g.
  `[wsl2]\nmemory=14GB\nprocessors=6\nswap=8GB\nswapFile=G:\\Temp\\swap.vhdx`.
  With ≤14GB in WSL, use `CARGO_BUILD_JOBS=2` for the release build and keep
  the tree's `codegen-units = 16` overrides (they are what makes giants fit).
- Getting the tree in: the Windows tree has the uncommitted changes from §5.
  Do NOT `git clone` fresh (you'd lose them) and do NOT build on `/mnt/g`
  (9P is slow and breaks exec bits). Copy once, excluding `target/`:
  `cp -a --exclude=target /mnt/g/Dx/desktop ~/dx-code` (or tar pipe),
  INCLUDING `.git` (build stamps `ZED_COMMIT_SHA` from it). Then build native
  inside `~/dx-code`.
- In-WSL recipe: `./script/linux` (sudo apt; EC2 mirrors flaked on CI but a
  local WSL apt run is normally fine) → `./script/download-wasi-sdk` →
  rustup installs pinned 1.95.0 automatically →
  `CARGO_BUILD_JOBS=2 cargo build --release --locked -p zed --bin dx-code`
  → binary at `target/release/dx-code` → copy to
  `/mnt/g/Dx/bin/dx-code-linux-x86_64` plus an `assets/web` copy next to it.
- Smoke test on the binary if a GUI is unavailable: it should at least print
  the clap error on a bad flag (proves DLLs/symbols resolve); real proof is
  launching it in a Linux GUI session.

## 8. Size forensics (measure, don't guess)

- `objdump -h <exe>` (MinGW bin dir, §1): `.text` = code, `.rdata` =
  read-only data incl. all rust-embedded assets. A 135MB embed CANNOT hide
  outside `.rdata` — if `.rdata` is small, the embed is gone. Trust this over
  assumptions about shared/common crates (cfg gates compile them out per-OS).
- Find every embed path: `git grep -n "RustEmbed\|WebProjectsAssets" --
  crates` — every hit is a potential 100MB+ surprise. Check each derive's
  `#[folder]` size with `du` and its `#[cfg]` gates before claiming anything
  about any platform's binary.
- Per-crate proof: check the rlib object sizes
  (`target/<profile>/deps/*<cratename>*`) — a 135MB `.o`/rlib next to a
  small source file means an embed. Cross-check by grepping the final binary
  for unique hashed asset filenames (e.g. `AgentStudio-B4tM9Kyn`).
- ALWAYS measure the binary after a size change and report exe + installer
  bytes. Size work without before/after numbers is guessing.
- UPX: Linux UPX is safe and proven (~30% ratio here); mac-ARM UPX is NOT
  safe; Windows doesn't need it (installer already <100MB).

## 9. What NOT to do

- No CircleCI Linux work (macOS owns that budget).
- No `git push` of `dev` anywhere without an explicit order.
- No new `target/` dirs scattered across drives; keep `target/` in-repo.
- Do not re-add `liquid_glass` or re-embed `assets/web` (that was −171MB).
- Do not "fix" GPUI `LiquidGlass` renderer files (`*_renderer.rs`,
  shaders) — unrelated upstream tech, still used.
