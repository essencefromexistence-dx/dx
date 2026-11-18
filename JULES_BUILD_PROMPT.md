# Jules build prompt - dx-code, Linux + Windows (+ why not macOS)

> Feed this to Jules verbatim. The repo is `essence-dx/dx`, branch `main`.
> ```powershell
> jules new --repo essence-dx/dx "<paste the TASK text below>"
> ```

---

## TASK FOR JULES

Build the `dx-code` release binaries for Linux and Windows from the
`essence-dx/dx` repository (branch `main`, HEAD `5de1bc6d75`), upload each
produced artifact to catbox.moe, and report the download URLs.

`aarch64-apple-darwin` **cannot** be produced here - see the Targets table. A
Linux-only result is a **success**, not a failure. Report exactly which targets
produced a binary and which did not.

### 0. Preflight - do this first, and STOP if it fails

- `rustc --version` must report **1.98.1**. The repo pins it in
  `rust-toolchain.toml`; do not use a newer toolchain even if it is the default.
- `cargo metadata --locked --format-version 1 > /dev/null` must exit **0**.
  If it fails, the likely cause is the `dx-media` git dependency
  (`Cargo.toml:303`: `git = "https://github.com/essence-dx/media"`,
  `rev = "caacb6134a455bea4e0eb6b58d522e96af230e4a"`), which is consumed by
  `crates/media_panel` and therefore by `zed`. **STOP and report the exact
  error text**; do not edit `Cargo.toml` or `Cargo.lock` to work around it.
- No LFS preflight is needed: this repo's `.gitattributes` is entirely
  commented out, so there are no git-LFS pointer files to resolve.

### 1. What to build

Package and binary: **`-p zed --bin dx-code`**. The fork's binary is
`dx-code`, *not* upstream's `zed` - `crates/zed/Cargo.toml` declares
`[[bin]] name = "dx-code"` and `default-run = "dx-code"`, so `--bin zed`
fails.

Profile: `--release` together with the repo's own size config, which is what
`script/bundle-linux` uses:

```bash
cargo --config .cargo/bundle-config.toml build --release ...
```

`.cargo/bundle-config.toml` sets `[profile.release] opt-level = "z"` with
`opt-level = 3` for the heavy crates (`gpui`, `cli`, `editor`, `language`,
`project`, `remote_server`, ...) plus the Linux rpath flags. The plain
`release` profile produces a bigger binary that can exceed the 200 MB anonymous
catbox cap.

### 2. Targets

| Target | Expected to work | Notes |
| --- | --- | --- |
| `x86_64-unknown-linux-gnu` | **Yes** (native) | primary path |
| `x86_64-pc-windows-msvc` | Attempt it - it is the fragile one | needs `cargo-xwin`, `clang`, `nasm`, `cmake`, `ninja` |
| `aarch64-apple-darwin` | **No** | needs `MacOSX.sdk`, which ships only in Xcode and is SLA-restricted to Apple hardware; there is no legitimate Linux cross toolchain |

Linux dependencies - `script/linux` is the authority, install them with
`apt-get` if the sandbox allows:

```text
gcc g++ libasound2-dev libfontconfig-dev libgit2-dev libglib2.0-dev
libgtk-3-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev libssl-dev
libva-dev libvulkan1 libwayland-dev libwebkit2gtk-4.1-dev libx11-xcb-dev
libxkbcommon-x11-dev libzstd-dev make cmake clang lld llvm jq git curl
gettext-base elfutils libsqlite3-dev musl-tools musl-dev build-essential
pipewire xdg-desktop-portal
```

If a package is unavailable in the sandbox, say which one rather than skipping
it silently - some of them decide whether the build can link at all.

The repo already sets `crt-static` and `windows_slim_errors` for Windows
targets in `.cargo/config.toml`, so `cargo-xwin` inherits the right CRT flags.
Expect `libwebrtc`, `windows_resources` and the MSVC linker to be where a
Windows cross-build breaks.

### 3. Build

```bash
# Linux (native)
rustup target add x86_64-unknown-linux-gnu
cargo --config .cargo/bundle-config.toml build --release \
    --target x86_64-unknown-linux-gnu -p zed --bin dx-code

# Windows (cross, via xwin)
rustup target add x86_64-pc-windows-msvc
cargo-xwin build --release --target x86_64-pc-windows-msvc \
    --config .cargo/bundle-config.toml -p zed --bin dx-code
```

### 4. Verify before uploading

- **Linux**: run the binary. It must print a version, not crash:
  `./target/x86_64-unknown-linux-gnu/release/dx-code --version`. If `--version`
  does not exist, use `--help`. A binary that cannot execute is not an
  artifact - do not upload it.
- **Windows**: an `.exe` cannot be executed on the Linux sandbox. Verify only
  that the file exists, is a PE image (`file`, or an `MZ` header check), and is
  non-trivial in size (well over 10 MB). State explicitly that it was **not**
  executed.
- Record `sha256` and the exact byte size for every artifact.

### 5. Upload to catbox (anonymous, gets a URL)

```bash
curl -sS -F "reqtype=fileupload" -F "fileToUpload=@<path-to-binary>" \
  https://catbox.moe/user/api.php
```

One call per artifact. The response body is the plain URL, e.g.
`https://files.catbox.moe/abc123.bin`.

### 6. Report format - this is what gets read back

End your final message with exactly:

```text
ARTIFACTS
- linux x86_64-unknown-linux-gnu: <catbox-url>  (sha256 <hash>, <size> bytes) BUILT
- windows x86_64-pc-windows-msvc: <catbox-url or SKIPPED — reason>
- macos aarch64-apple-darwin: SKIPPED — <reason>
```

If catbox rejects an upload (for example over 200 MB, or a 5xx), report the
exact API response text rather than retrying silently.

### 7. Known hazards to respect

- `essence-dx/dx` is **private**. Catbox anonymous uploads are **publicly
  downloadable** by anyone with the URL - treat every uploaded binary as public
  and say so in the report.
- `aws-lc-sys`, `libgit2-sys` and friends compile C/C++ and need a working
  linker; a sandbox without a C toolchain cannot build them.
- The workspace is large (~150 crates) with heavy native dependencies. A cold
  build can exceed a sandbox's time or disk limit. If the build stops on time
  or disk, report where it stopped rather than restarting from scratch.
- **Do not** `git push`, open PRs, or modify tracked files. This task builds and
  reports only.

---

MIT OR Apache-2.0
updated: 2026-10-15 00:00 UTC
