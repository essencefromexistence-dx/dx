<!-- ===== docs/src/ai/agent-panel.md ===== -->
---
title: AI Coding Agent - Zed Agent Panel
description: Use Zed's AI coding agent to generate, refactor, and debug code with tool calling and multi-model support.
---

# Agent Panel

The Agent Panel is where you interact with AI agents that can read, write, and run code in your project.
It's the core of Zed's AI code editing experience — use it for code generation, refactoring, debugging, documentation, and general questions.

Open it with {#action agent::NewThread} from [the Command Palette](../command-palette.md) or click the ✨ icon in the status bar.

## Getting Started {#getting-started}

If you're using the Agent Panel for the first time, configure either a model for the [Zed Agent](./zed-agent.md) or an [External Agent](./external-agents.md).

- Use [LLM Providers](./llm-providers.md) for Zed-hosted models, API access, subscriptions, gateways, and local models.
- Use [External Agents](./external-agents.md) for ACP-integrated agents.
- Use [AI Quick Start](./quick-start.md) if you are not sure which path to choose.

## Overview {#overview}

With an LLM provider or External Agent configured, type in the message editor and press `enter` to submit.
Expand the editor with {#kb agent::ExpandMessageEditor} if you need more room.

Responses stream in with indicators showing [which tools](./tools.md) the model is using.
The sections below cover what you can do from here.

> Note that some Agent Panel features may not be available for every External Agent. Restoring threads from history, token usage display, and similar features depend on the agent integration.
> Their availability varies depending on the agent.

### Creating New Threads {#new-thread}

By default, the Agent Panel uses Zed's first-party agent.

Start a new thread with {#kb agent::NewThread}, or open the "New Thread…" menu using the agent selector button on the left (in the empty state) or the `+` icon in the top-right of the panel toolbar. You can also open that menu with {#kb agent::ToggleNewThreadMenu}.

From the "New Thread…" menu you can:

- Pick **Zed Agent** or any installed [External Agent](./external-agents.md) to start a new thread with that agent.
- Choose **New From Summary** to start a fresh Zed Agent thread seeded with a summary of the current conversation — useful for compacting long threads as you approach the context window limit.
- Choose **Terminal** to open a terminal thread directly in the Agent Panel — see [Terminal Threads](#terminal-threads) for details.

{#action agent::NewExternalAgentThread} creates a new thread with the specified External Agent id.

You can also start a new thread from the [Threads Sidebar](./parallel-agents.md#threads-sidebar), scoped to a specific project — see [Running Multiple Threads](./parallel-agents.md#running-multiple-threads).

### Managing Multiple Threads {#multiple-threads}

You can run multiple agent threads at once, each working independently with its own agent, context window, and conversation history. Open the Threads Sidebar with {#kb multi_workspace::ToggleWorkspaceSidebar} to see all your threads grouped by project. Click any thread to switch to it, or use the thread switcher ({#kb agents_sidebar::ToggleThreadSwitcher}) to cycle between recent threads without opening the sidebar.

Threads you're no longer working on can be archived by hovering over them in the sidebar and clicking the archive icon, or selecting them and pressing {#kb agent::ArchiveSelectedThread}. The Thread History holds all your threads across all projects, sorted chronologically, and you can restore them at any time.

If two threads might edit the same files, you can isolate one in a new Git worktree. Use the worktree picker in the title bar to pick which worktree the agent runs in, or create a new one. See [Worktree Isolation](./parallel-agents.md#worktree-isolation) for details.

For more details on the Threads Sidebar and managing multiple projects, see [Parallel Agents](./parallel-agents.md).

### Editing Messages {#editing-messages}

Any message that you send to the model is editable.
You can click on the card that contains your message and re-submit it with an adjusted prompt and/or new pieces of context.

### Queueing Messages

Messages sent while the agent is in the generating state get, by default, queued.

By default, queued messages get sent once the agent finishes generating. If you want a queued message to reach the Zed Agent sooner—interrupting it at its next step (usually between a tool call and a response) rather than waiting for it to finish—toggle "Steer" on that message. Steering is only available for the Zed Agent, since Zed can't detect turn boundaries for external agents.

You can edit or remove (an individual or all) queued messages.
You can also still interrupt the agent immediately if you want by either clicking on the stop button or by clicking the "Send Now" (double-enter) on a queued message.

### Context Menu {#context-menu}

Right-click on any agent response in the thread view to access a context menu with the following actions:

- **Copy**: Copies the currently selected text as plain text (available when text is selected).
- **Copy as Markdown**: Copies the currently selected text as Markdown (available when text is selected).
- **Copy This Agent Response**: Copies the full text of the agent response you right-clicked on.
- **Scroll to Top / Scroll to Bottom**: Scrolls to the beginning or end of the thread, depending on your current position.
- **Open Thread as Markdown**: Opens the entire thread as a Markdown file in a new tab.

### Navigating the Thread {#navigating-the-thread}

In long conversations, use the scroll arrow buttons at the bottom of the panel to jump to your most recent prompt or to the very beginning of the thread. You can also scroll the thread using arrow keys, Page Up/Down, Home/End, and Shift+Page Up/Down to jump between messages, when the thread pane is focused.

When focus is in the message editor, you can also use {#kb agent::ScrollOutputPageUp}, {#kb agent::ScrollOutputPageDown}, {#kb agent::ScrollOutputToTop}, {#kb agent::ScrollOutputToBottom}, {#kb agent::ScrollOutputLineUp}, and {#kb agent::ScrollOutputLineDown} to navigate the thread, or {#kb agent::ScrollOutputToPreviousMessage} and {#kb agent::ScrollOutputToNextMessage} to jump between your prompts.

### Thread titles {#thread-titles}

Thread titles are auto-generated based on the content of the conversation.
But you can also edit them manually by clicking the title and typing, or regenerate them by clicking the "Regenerate Thread Title" button in the ellipsis menu in the top right of the panel.

### Following the Agent {#following-the-agent}

Follow the agent as it reads and edits files by clicking the crosshair icon at the bottom left of the panel.
Your editor will jump to each file the agent touches.

You can also hold `cmd`/`ctrl` when submitting a message to automatically follow.

### Keeping the System Awake {#prevent-idle-sleep}

While an agent thread is running, Zed asks the operating system not to idle-sleep, so long-running turns are not interrupted when you step away.
The display can still turn off and the screen can still lock; only system sleep is held back, and only while a thread is generating, not while it waits for your confirmation.

Turn this off with the `agent.prevent_idle_sleep` setting.
If the operating system rejects the request, Zed logs the reason and continues without it.

### Get Notified {#get-notified}

If you send a prompt to the Agent and then put Zed in the background, you can choose to be notified when its generation wraps up via:

- a visual desktop notification from your operating system
- a sound notification

These notifications can be used together or individually, and you can use the `agent.notify_when_agent_waiting` and `agent.play_sound_when_agent_done` settings keys to customize that, including turning both off entirely.

### Reviewing Changes {#reviewing-changes}

Once the agent has made changes to your project, the panel will surface which files, how many of them, and how many lines have been edited.

To see which files specifically have been edited, expand the accordion bar that shows up right above the message editor or click the `Review Changes` button ({#kb agent::OpenAgentDiff}), which opens a special multi-buffer tab with all changes.

You can accept or reject each individual change hunk, or the whole set of changes made by the agent.

Edit diffs can also appear inline in individual files with the same
keep/reject hunk controls as the multi-buffer review pane. This temporarily overrides the buffer's git diff while review is active. Enable it by setting `agent.single_file_review` to `true` in your settings.

## Terminal Threads {#terminal-threads}

The Agent Panel can host Terminal Threads alongside your agent threads. For opening, closing, notifications, terminal titles, and CLI/TUI-specific setup, see [Terminal Threads](./terminal-threads.md).

## Adding Context {#adding-context}

The agent can search your codebase to find relevant context, but providing it explicitly improves response quality and reduces latency.

Add context by typing `@` in the message editor.
You can mention files, directories, symbols, previous threads, [skills](./skills.md), diagnostics, branch diffs, and URLs to fetch.

When you paste multi-line code selections copied from a buffer, Zed automatically formats them as @-mentions with the file context.
To paste content without this automatic formatting, use {#kb agent::PasteRaw} to paste raw text directly.

### Selection as Context

Additionally, you can also select text in a buffer or terminal and add it as context by using the {#kb agent::AddSelectionToThread} keybinding, running the {#action agent::AddSelectionToThread} action, or choosing the "Selection" item in the `+` menu in the message editor.

### Images as Context

It's also possible to attach images in your prompt for providers that support vision models.
OpenAI GPT-4o and later, Anthropic Claude 3 and later, Google Gemini 1.5 and 2.0, and Bedrock vision models (Claude 3+, Amazon Nova Pro and Lite, Meta Llama 3.2 Vision, Mistral Pixtral) all support image inputs.

To add an image, you can either search in your project's folder by @-mentioning it, or drag it from your file system directly into the Agent Panel message editor.
Copying an image and pasting it is also supported.

## Token Usage and Compaction {#token-usage}

Zed surfaces how many tokens you are consuming for your currently active thread near the profile selector in the panel's message editor.

Zed automatically compacts long Zed Agent threads as they approach the configured token threshold. Compaction summarizes earlier messages and replaces them in the model context with that summary, leaving more room for the next turn. The thread shows a **Context Compacted** entry that you can expand to inspect the summary. You can compact manually by typing `/compact` in the message editor.

If the selected model's context window is too small for automatic compaction (less than 80000 tokens), a banner appears above the message editor as you approach the token limit. Use **Start New Thread** from that banner, or choose **New From Summary** from the New Thread menu (the `+` button on the top right), to continue in a new thread seeded with a summary. You can also @-mention a past thread in a new one.

Configure automatic compaction with `agent.auto_compact`. See [Agent Settings](./agent-settings.md#automatic-compaction) for options.

## Changing Models {#changing-models}

After you've configured your LLM providers—either via [API access](./use-api-access.md) or through [Zed-hosted models](../account/zed-hosted-models.md)—you can switch between their models by clicking on the model selector on the message editor or by using the {#kb agent::ToggleModelSelector} keybinding.

> The same model can be offered via multiple providers - for example, Claude Sonnet 4.5 is available via Zed Pro, OpenRouter, Anthropic directly, and more.
> Make sure you've selected the correct model **_provider_** for the model you'd like to use, delineated by the logo to the left of the model in the model selector.

### Favoriting Models

You can mark specific models as favorites either through the model selector, by clicking on the star icon button that appears as you hover the model, or through your settings via the `agent.favorite_models` settings key.

Cycle through your favorites with {#kb agent::CycleFavoriteModels} without opening the model selector.

## Using Tools and Profiles {#using-tools}

The Agent Panel supports tool calling, which enables agentic editing. Zed includes [built-in tools](./tools.md) for searching your codebase, editing files, running terminal commands, and more.

Use [Agent Profiles](./agent-profiles.md) to choose which built-in tools and MCP tools are available in a Zed Agent thread. Use [Tool Permissions](./tool-permissions.md) to control whether permission-gated tool calls are allowed, denied, or confirmed.

You can add external tools with [MCP Servers](./mcp.md).

### Model Support {#model-support}

Tool calling needs to be individually supported by each model and model provider.
Therefore, despite the presence of built-in tools, some models may not have the ability to pick them up.
You should see a "No tools" label if you select a model that falls into this case.

All [Zed-hosted models](../account/zed-hosted-models.md) support tool calling out-of-the-box.

### MCP Servers {#mcp-servers}

Similarly to the built-in tools, some models may not support all tools included in a given MCP Server.
Zed's UI will inform you about this via a warning icon that appears close to the model selector.

## Errors and Debugging {#errors-and-debugging}

If you hit an error or unusual LLM behavior, open the thread as Markdown with {#action agent::OpenActiveThreadAsMarkdown} and attach it to your GitHub issue.

You can also open threads as Markdown by clicking on the file icon button, to the right of the thumbs down button, when focused on the panel's editor.

## Feedback {#feedback}

You can rate agent responses to help improve Zed's system prompt and tools.

> **Warning:** Rating an AI response sends the conversation thread to Zed. The
> conversation thread includes your messages, AI responses, and thread metadata.
> See [Feedback and Training Data](./ai-improvement.md) and
> [AI Privacy](./privacy-and-security.md) for more information.
> **_If you don't want data persisted on Zed's servers, don't rate_**.
> We will not collect data for improving Zed's agent experience without you
> explicitly rating responses.

To help improve Zed's system prompt and tools, rate responses with the thumbs up/down controls at the end of each response.
In case of a thumbs down, a new text area will show up where you can add more specifics about what happened.

You can provide feedback on the thread at any point after the agent responds, and multiple times within the same thread.

<!-- ===== docs/src/development/macos.md ===== -->
---
title: Building Zed for macOS
description: "Guide to building zed for macos for Zed development."
---

# Building Zed for macOS

## Repository

Clone the [Zed repository](https://github.com/zed-industries/zed).

## Dependencies

- Install [rustup](https://www.rust-lang.org/tools/install)

- Install [Xcode](https://apps.apple.com/us/app/xcode/id497799835?mt=12) from the macOS App Store or from the [Apple Developer](https://developer.apple.com/download/all/) website. The Apple Developer download requires a developer account.

> Launch Xcode after installation and install the macOS components (the default option).

- Install [Xcode command line tools](https://developer.apple.com/xcode/resources/)

  ```sh
  xcode-select --install
  ```

- Ensure that the Xcode command line tools are using your newly installed copy of Xcode:

  ```sh
  sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
  sudo xcodebuild -license accept
  ```

- Install `cmake` (required by [a dependency](https://docs.rs/wasmtime-c-api-impl/latest/wasmtime_c_api/))

  ```sh
  brew install cmake
  ```

Release bundling with `script/bundle-mac` requires the macOS 26 SDK or newer, provided by Xcode 26 or newer.
The bundled app requires macOS 12.0 (Monterey) or later.

## Building Zed from Source

Once you have the dependencies installed, you can build Zed using [Cargo](https://doc.rust-lang.org/cargo/).

For a debug build:

```sh
cargo run
```

For a release build:

```sh
cargo run --release
```

And to run the tests:

```sh
cargo test --workspace
```

## Visual Regression Tests

Zed includes visual regression tests that capture screenshots of real Zed windows and compare them against baseline images. These tests require macOS with Screen Recording permission.

### Prerequisites

You must grant Screen Recording permission to your terminal:

1. Run the visual test runner once - macOS will prompt for permission
2. Or manually: System Settings > Privacy & Security > Screen Recording
3. Enable your terminal app (e.g., Terminal.app, iTerm2, Ghostty)
4. Restart your terminal after granting permission

### Running Visual Tests

```sh
cargo run -p zed --bin zed_visual_test_runner --features visual-tests
```

### Baseline Images

Baseline images are stored in `crates/zed/test_fixtures/visual_tests/` but are
**gitignored** to avoid bloating the repository. You must generate them locally
before running tests.

#### Initial Setup

Before making any UI changes, generate baseline images from a known-good state:

```sh
git checkout origin/main
UPDATE_BASELINE=1 cargo run -p zed --bin zed_visual_test_runner --features visual-tests
git checkout -
```

This creates baselines that reflect the current expected UI.

#### Updating Baselines

When UI changes are intentional, update the baseline images after your changes:

```sh
UPDATE_BASELINE=1 cargo run -p zed --bin zed_visual_test_runner --features visual-tests
```

> **Note:** In the future, baselines may be stored externally. For now, they
> remain local-only to keep the git repository lightweight.

## Sampling released builds

How to get a symbolicated CPU profile from a released (non-dev) Zed instance.
Use this when Zed is using a lot of CPU.

Released macOS binaries are stripped of local symbols, so `sample` and Instruments show raw addresses for most frames.
The debug symbols for every release are archived: `script/bundle-mac` uploads `zed.dwarf` to Sentry before stripping.

### During the incident

- Run `sample Zed 10 -f zed-sample.txt` (adjust the process name for Preview or Nightly).
- Get the exact build: type {#action zed::About} in the command palette and copy the version and commit.

The `zed-sample.txt` file can be sent to Zed together with the exact version.

### Later

This can be done by Zed staff.

- Find the binary UUID in the `Binary Images` section at the bottom of the sample output.
- Download the matching `zed.dwarf` from the Sentry project's Debug Files page by searching for that UUID.
- Resolve the addresses:
  `atos -o zed.dwarf -l <load address> <address...>`
  Each unresolved frame in the sample output prints its `load address` and absolute address.

To profile a released build on your own machine with full symbols, download the matching `zed.dwarf`, convert it into a `.dSYM` bundle Spotlight can index, and re-run `sample`:

```sh
mkdir -p Zed.dSYM/Contents/Resources/DWARF
cp zed.dwarf Zed.dSYM/Contents/Resources/DWARF/zed
```

Frames then resolve automatically, including file and line information.

## Troubleshooting

### Error compiling metal shaders

```sh
error: failed to run custom build command for gpui v0.1.0 (/Users/path/to/zed)`**

xcrun: error: unable to find utility "metal", not a developer tool or in PATH
```

Try `sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer`

If you're on macOS 26, try `xcodebuild -downloadComponent MetalToolchain`.
If that command fails, run `xcodebuild -runFirstLaunch` and try downloading the toolchain again.

### Cargo errors claiming that a dependency is using unstable features

Try `cargo clean` and `cargo build`.

### Error: 'dispatch/dispatch.h' file not found

If you encounter an error similar to:

```sh
src/platform/mac/dispatch.h:1:10: fatal error: 'dispatch/dispatch.h' file not found

Caused by:
  process didn't exit successfully

  --- stdout
  cargo:rustc-link-lib=framework=System
  cargo:rerun-if-changed=src/platform/mac/dispatch.h
  cargo:rerun-if-env-changed=TARGET
  cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS_aarch64-apple-darwin
  cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS_aarch64_apple_darwin
  cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS
```

This file is part of Xcode. Make sure the Xcode command line tools are installed and the path is set correctly:

```sh
xcode-select --install
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
```

Additionally, set the `BINDGEN_EXTRA_CLANG_ARGS` environment variable:

```sh
export BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$(xcrun --show-sdk-path)"
```

Then clean and rebuild the project:

```sh
cargo clean
cargo run
```

### Tests failing due to `Too many open files (os error 24)`

This error seems to be caused by OS resource constraints. Installing and running tests with `cargo-nextest` should resolve the issue.

- `cargo install cargo-nextest --locked`
- `cargo nextest run --workspace --no-fail-fast`

## Tips & Tricks

### Avoiding continual rebuilds

If Zed continually rebuilds root crates, you may be opening the Zed codebase itself in your development build.

This causes problems because `cargo run` exports a bunch of environment
variables which are picked up by the `rust-analyzer` that runs in the development
build of Zed. These environment variables are in turn passed to `cargo check`, which
invalidates the build cache of some of the crates we depend on.

To avoid this, run the built binary against a different project, for example `cargo run ~/path/to/other/project`.

### Speeding up verification

If you build Zed frequently, macOS may keep verifying new builds, which can add a few seconds to each iteration.

To fix this, you can:

- Run `sudo spctl developer-mode enable-terminal` to enable the Developer Tools panel in System Settings.
- In System Settings, search for "Developer Tools" and add your terminal (e.g. iTerm or Ghostty) to the list under "Allow applications to use developer tools"
- Restart your terminal.

Thanks to the nextest developers for publishing [this](https://nexte.st/docs/installation/macos/#gatekeeper).

<!-- ===== docs/src/installation.md ===== -->
---
title: Install Zed - macOS, Linux, Windows
description: Download and install Zed on macOS, Linux, or Windows. Includes Homebrew, direct download, and package manager options.
---

# Installing Zed

## Download Zed

### macOS

Get the latest stable builds via [the download page](https://zed.dev/download). If you want to download our preview build, you can find it on its [releases page](https://zed.dev/releases/preview). After the first manual installation, Zed will periodically check for install updates.

You can also install Zed stable via Homebrew:

```sh
brew install --cask zed
```

As well as Zed preview:

```sh
brew install --cask zed@preview
```

### Windows

Get the latest stable builds via [the download page](https://zed.dev/download). If you want to download our preview build, you can find it on its [releases page](https://zed.dev/releases/preview). After the first manual installation, Zed will periodically check for install updates.

Additionally, you can install Zed using winget:

```sh
winget install -e --id ZedIndustries.Zed
```

### Linux

For most Linux users, the easiest way to install Zed is through our installation script:

```sh
curl -f https://zed.dev/install.sh | sh
```

You can now optionally specify a **version** of Zed to install using the `ZED_VERSION` environment variable:

```sh
# Install the latest stable version (default)
curl -f https://zed.dev/install.sh | sh

# Install a specific version
curl -f https://zed.dev/install.sh | ZED_VERSION=0.216.0 sh
```

To install the preview build, which receives updates about a week ahead of stable:

```sh
curl -f https://zed.dev/install.sh | ZED_CHANNEL=preview sh
```

This script supports `x86_64` and `AArch64`, as well as common Linux distributions: Ubuntu, Arch, Debian, RedHat, CentOS, Fedora, and more.

If Zed is installed using this installation script, it can be uninstalled at any time by running the shell command `zed --uninstall`. The shell will then prompt you whether you'd like to keep your preferences or delete them. After making a choice, you should see a message that Zed was successfully uninstalled.

If this script is insufficient for your use case, you run into problems running Zed, or there are errors in uninstalling Zed, please see our [Linux-specific documentation](./linux.md).

## System Requirements

### macOS

Zed requires macOS 12.0 (Monterey) or later.

| Version            | Codename    | Apple Status   | Zed Status          |
| ------------------ | ----------- | -------------- | ------------------- |
| macOS 27.x         | Golden Gate | Supported      | Supported           |
| macOS 26.x         | Tahoe       | Supported      | Supported           |
| macOS 15.x         | Sequoia     | Supported      | Supported           |
| macOS 14.x         | Sonoma      | EOL 2026-09-14 | Supported           |
| macOS 13.x         | Ventura     | EOL 2025-09-15 | Supported           |
| macOS 12.x (12.3+) | Monterey    | EOL 2024-09-16 | Supported           |
| macOS 12.0–12.2    | Monterey    | EOL 2024-09-16 | Partially Supported |

The macOS releases labelled "Partially Supported" (Monterey 12.0–12.2) do not support screen sharing via Zed Collaboration.
Screen sharing uses [ScreenCaptureKit.framework](https://developer.apple.com/documentation/screencapturekit/), which is available on macOS 12.3 and newer.

Apple support status and EOL dates are from [endoflife.date](https://endoflife.date/macos).

#### Mac Hardware

Zed supports machines with Intel (x86_64) or Apple (aarch64) processors that meet the above macOS requirements:

- MacBook Pro (Early 2015 and newer)
- MacBook Air (Early 2015 and newer)
- MacBook (Early 2016 and newer)
- Mac Mini (Late 2014 and newer)
- Mac Pro (Late 2013 or newer)
- iMac (Late 2015 and newer)
- iMac Pro (all models)
- Mac Studio (all models)

### Linux

Zed supports 64-bit Intel/AMD (x86_64) and 64-bit Arm (aarch64) processors.

Zed requires a Vulkan 1.3 driver and the following desktop portals:

- `org.freedesktop.portal.FileChooser`
- `org.freedesktop.portal.OpenURI`
- `org.freedesktop.portal.Secret` or `org.freedesktop.Secrets`

### Windows

Zed supports the following Windows releases:
| Version | Zed Status |
| ------------------------- | ------------------- |
| Windows 11, version 22H2 and later | Supported |
| Windows 10, version 1903 and later | Supported |

A 64-bit operating system is required to run Zed.

#### Windows Hardware

Zed supports machines with x64 (Intel, AMD) or Arm64 (Qualcomm) processors that meet the following requirements:

- Graphics: A GPU that supports DirectX 11 (most PCs from 2012+).
- Driver: Current NVIDIA/AMD/Intel/Qualcomm driver (not the Microsoft Basic Display Adapter).

### FreeBSD

Not yet available as an official download. Can be built [from source](./development/freebsd.md).

### Web

Not supported at this time. See our [Platform Support issue](https://github.com/zed-industries/zed/issues/5391).

<!-- ===== docs/src/macos.md ===== -->
---
title: Zed on macOS
description: "Zed is developed primarily on macOS, making it a first-class platform with full feature support."
---

# Zed on macOS

Zed is developed primarily on macOS, making it a first-class platform with full feature support.

## Installing Zed

Download Zed from the [download page](https://zed.dev/download). The download is a `.dmg` file—open it and drag Zed to your Applications folder.

For the preview build, which receives updates about a week ahead of stable, visit the [preview releases page](https://zed.dev/releases/preview).

After installation, Zed checks for updates automatically and prompts you when a new version is available.

### Homebrew

You can also install Zed using Homebrew:

```sh
brew install --cask zed
```

For the preview version:

```sh
brew install --cask zed@preview
```

### Building from Source

To build Zed from source, see the [macOS development documentation](./development/macos.md).

## System Requirements

- macOS 12.0 (Monterey) or later
- Apple Silicon (M1/M2/M3/M4) or Intel processor

Zed uses Metal for GPU-accelerated rendering, which is available on all supported macOS versions.

## Installing the CLI

Zed includes a command-line tool for opening files and projects from Terminal. To install it:

1. Open Zed
2. Open the command palette with `Cmd+Shift+P`
3. Run {#action cli::InstallCliBinary}

This creates a `zed` command in `/usr/local/bin`. You can then open files and folders:

```sh
zed .                    # Open current folder
zed file.txt             # Open a file
zed project/ file.txt    # Open a folder and a file
```

See the [CLI Reference](./reference/cli.md) for all available options.

## Uninstall

1. Quit Zed if it's running
2. Drag Zed from Applications to the Trash
3. Optionally, remove your settings and extensions:

```sh
rm -rf ~/.config/zed
rm -rf ~/Library/Application\ Support/Zed
rm -rf ~/Library/Caches/Zed
rm -rf ~/Library/Logs/Zed
rm -rf ~/Library/Saved\ Application\ State/dev.zed.Zed.savedState
```

If you installed the CLI, remove it with:

```sh
rm /usr/local/bin/zed
```

## Troubleshooting

### Zed won't open or shows "damaged" warning

If macOS reports that Zed is damaged or can't be opened, it's likely a Gatekeeper issue. Try:

1. Right-click (or Control-click) on Zed in Applications
2. Select "Open" from the context menu
3. Click "Open" in the dialog that appears

This tells macOS to trust the application.

If that doesn't work, remove the quarantine attribute:

```sh
xattr -cr /Applications/Zed.app
```

### CLI command not found

If the `zed` command isn't available after installation:

1. Check that `/usr/local/bin` is in your PATH
2. Try reinstalling the CLI via {#action cli::InstallCliBinary} in the command palette
3. Open a new terminal window to reload your PATH

### Can't install CLI {#cant-install-cli}

{#action cli::InstallCliBinary} writes a `zed` symlink to `/usr/local/bin`, which requires administrator privileges. If your macOS account isn't in the `admin` group, Zed can't create that symlink and will report that it can't install the CLI automatically.

Instead, you can add an alias pointing to the `cli` binary bundled inside the app. The path depends on where Zed is installed:

```sh
# Default install (Zed in /Applications)
alias zed="/Applications/Zed.app/Contents/MacOS/cli"

# User install (Zed in ~/Applications)
alias zed="$HOME/Applications/Zed.app/Contents/MacOS/cli"

# Preview build (Zed Preview in ~/Applications)
alias zed="$HOME/Applications/Zed Preview.app/Contents/MacOS/cli"
```

Add the line that matches your install to your shell configuration file. Use `~/.zshrc` for Zsh (the default on modern macOS) or `~/.bashrc` for Bash.

After you restart your shell, you will be able to use `zed` from your terminal:

```sh
zed .              # Open current folder
zed file.txt       # Open a file
```

### GPU or rendering issues

Zed uses Metal for rendering. If you experience graphical glitches:

1. Ensure macOS is up to date
2. Restart your Mac to reset the GPU state
3. Check Activity Monitor for GPU pressure from other apps

### High memory or CPU usage

If Zed uses more resources than expected:

1. Check for runaway language servers in the terminal output ({#action zed::OpenLog})
2. Try disabling extensions one by one to identify conflicts
3. For large projects, consider using [project settings](./reference/all-settings.md#file-scan-exclusions) to exclude unnecessary folders from indexing

For additional help, see the [Troubleshooting guide](./troubleshooting.md) or visit the [Zed Discord](https://discord.gg/zed-community).

<!-- ===== docs/src/remote-development.md ===== -->
---
title: Remote Development in Zed - SSH Workflows
description: Use remote development in Zed to edit code over SSH with local UI performance, remote terminals, language servers, and tasks.
---

# Remote Development

Remote Development lets you edit code on a remote server while running Zed locally. The UI stays responsive because it runs on your machine, while language servers, tasks, and terminals run on the server.

For day-to-day workflows, pair remote development with [Tasks](./tasks.md),
[Terminal](./terminal.md), and [Debugger](./debugger.md).

## Overview

Remote development requires two computers, your local machine that runs the Zed UI and the remote server which runs a Zed headless server. The two communicate over SSH, so you will need to be able to SSH from your local machine into the remote server to use this feature.

![Architectural overview of Zed Remote Development](https://zed.dev/img/remote-development/diagram.png)

On your local machine, Zed runs its UI, talks to language models, uses Tree-sitter to parse and syntax-highlight code, and stores unsaved changes and recent projects. The source code, language servers, tasks, and the terminal all run on the remote server. [AI features](./ai/overview.md) work in remote sessions, including the Agent Panel and Inline Assistant.

> **Note:** The original version of remote development sent traffic via Zed's servers. As of Zed v0.157 you can no longer use that mode.

## Setup

1. Download and install the latest [Zed](https://zed.dev/releases). You need at least Zed v0.159.
1. Use {#kb projects::OpenRemote} to open the "Remote Projects" dialog.
1. Click "Connect New Server" and enter the command you use to SSH into the server. See [Supported SSH options](#supported-ssh-options) for options you can pass.
1. Your local machine will attempt to connect to the remote server using the `ssh` binary on your path. Assuming the connection is successful, Zed will download the server on the remote host and start it.
1. Once the Zed server is running, you will be prompted to choose a path to open on the remote server.
   > **Note:** Zed does not currently handle opening very large directories (for example, `/` or `~` that may have >100,000 files) very well. We are working on improving this, but suggest in the meantime opening only specific projects, or subfolders of very large mono-repos.

For simple cases where you don't need any SSH arguments, you can run `zed ssh://[<user>@]<host>[:<port>]/<path>` to open a remote folder/file directly. The CLI also accepts the scp style `zed ssh://[<user>@]<host>:~/project` or `zed ssh://[<user>@]<host>:/absolute/path`. If you'd like to hotlink into an SSH project, use a link of the format: `zed://ssh/[<user>@]<host>[:<port>]/<path>`.

## Supported platforms

The remote machine must be able to run Zed's server. The following platforms should work, though note that we have not exhaustively tested every Linux distribution:

- macOS 12.0 (Monterey) or later (Intel or Apple Silicon)
- Linux (x86_64 or arm64, we do not yet support 32-bit platforms)
- Windows (x86_64 or arm64)

## Configuration

The list of remote servers is stored in your settings file {#kb zed::OpenSettings}. You can edit this list using the Remote Projects dialog {#kb projects::OpenRemote}, which provides some robustness - for example it checks that the connection can be established before writing it to the settings file.

```json [settings]
{
  "ssh_connections": [
    {
      "host": "192.168.1.10",
      "projects": [{ "paths": ["~/code/zed/zed"] }]
    }
  ]
}
```

Zed shells out to the `ssh` on your path, and so it will inherit any configuration you have in `~/.ssh/config` for the given host. That said, if you need to override anything you can configure the following additional options on each connection:

```json [settings]
{
  "ssh_connections": [
    {
      "host": "192.168.1.10",
      "projects": [{ "paths": ["~/code/zed/zed"] }],
      // any argument to pass to the ssh master process
      "args": ["-i", "~/.ssh/work_id_file"],
      "port": 22, // defaults to 22
      // defaults to your username on your local machine
      "username": "me"
    }
  ]
}
```

There are two additional Zed-specific options per connection, `upload_binary_over_ssh` and `nickname`:

```json [settings]
{
  "ssh_connections": [
    {
      "host": "192.168.1.10",
      "projects": [{ "paths": ["~/code/zed/zed"] }],
      // by default Zed will download the server binary from the internet on the remote.
      // When this is true, it'll be downloaded to your laptop and uploaded over SSH.
      // This is useful when your remote server has restricted internet access.
      "upload_binary_over_ssh": true,
      // Shown in the Zed UI to help distinguish multiple hosts.
      "nickname": "lil-linux"
    }
  ]
}
```

If you use the command line to open a connection to a host by doing `zed ssh://192.168.1.10/~/.vimrc`, then extra options are read from your settings file by finding the first connection that matches the host/username/port of the URL on the command line.

Additionally it's worth noting that while you can pass a password on the command line `zed ssh://user:password@host/~`, we do not support writing a password to your settings file. If you're connecting repeatedly to the same host, you should configure key-based authentication.

## Remote Development on Windows (SSH)

Zed on Windows supports SSH remoting and will prompt for credentials when needed.

If you encounter authentication issues, confirm that your SSH key agent is running (e.g., ssh-agent or your Git client's agent) and that ssh.exe is on PATH.

### Troubleshooting SSH on Windows

When prompted for credentials, use the graphical askpass dialog. If it doesn't appear, check for credential manager conflicts and that GUI prompts aren't blocked by your terminal.

## WSL Support

Zed supports opening folders inside of WSL natively on Windows.

### Opening a local folder in WSL

To open a local folder inside a WSL container, use the {#action projects::OpenFolderInWsl} action and select the folder you want to open. You will be presented with a list of available WSL distributions to open the folder in.

### Opening a folder already in WSL

To open a folder that's already located inside of a WSL container, use the {#action projects::OpenWsl} action and select the WSL distribution. The distribution will be added to the `Remote Projects` window where you will be able to open the folder.

## Port forwarding

If you'd like to be able to connect to ports on your remote server from your local machine, you can configure port forwarding in your settings file. This is particularly useful for developing websites so you can load the site in your browser while working.

```json [settings]
{
  "ssh_connections": [
    {
      "host": "192.168.1.10",
      "port_forwards": [{ "local_port": 8080, "remote_port": 80 }]
    }
  ]
}
```

This will cause requests from your local machine to `localhost:8080` to be forwarded to the remote machine's port 80. Under the hood this uses the `-L` argument to ssh.

By default these ports are bound to localhost, so other computers in the same network as your development machine cannot access them. You can set the local_host to bind to a different interface, for example, 0.0.0.0 will bind to all local interfaces.

```json [settings]
{
  "ssh_connections": [
    {
      "host": "192.168.1.10",
      "port_forwards": [
        {
          "local_port": 8080,
          "remote_port": 80,
          "local_host": "0.0.0.0"
        }
      ]
    }
  ]
}
```

These ports also default to the `localhost` interface on the remote host. If you need to change this, you can also set the remote host:

```json [settings]
{
  "ssh_connections": [
    {
      "host": "192.168.1.10",
      "port_forwards": [
        {
          "local_port": 8080,
          "remote_port": 80,
          "remote_host": "docker-host"
        }
      ]
    }
  ]
}
```

## Zed settings

When opening a remote project there are three relevant settings locations:

- The local Zed settings (in `~/.zed/settings.json` on macOS or `~/.config/zed/settings.json` on Linux) on your local machine.
- The server Zed settings (in the same place) on the remote server.
- The project settings (in `.zed/settings.json` or `.editorconfig` of your project)

Both the local Zed and the server Zed read the project settings, but they are not aware of the other's main settings file.

Which settings file you should use depends on the kind of setting you want to make:

- Project settings should be used for things that affect the project: indentation settings, which formatter / language server to use, etc.
- Server settings should be used for things that affect the server: paths to language servers, proxy settings, etc.
- Local settings should be used for things that affect the UI: font size, etc.

In addition any extensions you have installed locally will be propagated to the remote server. This means that language servers, etc. will run correctly.

## Proxy Configuration

The remote server will not use your local machine's proxy configuration because they may be under different network policies. If your remote server requires a proxy to access the internet, you must configure it on the remote server itself.

In most cases, your remote server will already have proxy environment variables configured. Zed will automatically use them when downloading language servers, communicating with LLM models, etc.

If needed, you can set these environment variables in the server's shell configuration (e.g., `~/.bashrc`):

```bash
export http_proxy="http://proxy.example.com:8080"
export https_proxy="http://proxy.example.com:8080"
export no_proxy="localhost,127.0.0.1"
```

Alternatively, you can configure the proxy in the remote machine's `~/.config/zed/settings.json` (Linux) or `~/.zed/settings.json` (macOS):

```json
{
  "proxy": "http://proxy.example.com:8080"
}
```

See the [proxy documentation](./reference/all-settings.md#network-proxy) for supported proxy types and additional configuration options.

## Initializing the remote server

Once you provide the SSH options, Zed shells out to `ssh` on your local machine to create a ControlMaster connection with the options you provide.

Any prompts that SSH needs will be shown in the UI, so you can verify host keys, type key passwords, etc.

Once the master connection is established, Zed will check to see if the remote server binary is present in `~/.zed_server` on the remote, and that its version matches the current version of Zed that you're using.

If it is not there or the version mismatches, Zed will try to download the latest version. By default, it will download from `https://zed.dev` directly, but if you set: `{"upload_binary_over_ssh":true}` in your settings for that server, it will download the binary to your local machine and then upload it to the remote server.

If you'd like to maintain the server binary yourself you can. You can either download our prebuilt versions from [GitHub](https://github.com/zed-industries/zed/releases), or [build your own](https://zed.dev/docs/development):

```bash
cargo build --release --package remote_server
llvm-objcopy --strip-debug target/release/remote_server
```

Stripping debug symbols matches the packaging step in [`script/bundle-linux`](https://github.com/zed-industries/zed/blob/main/script/bundle-linux) and keeps the binary closer to the size of Zed's prebuilt downloads. Prefer `llvm-objcopy` because GNU `objcopy` may not understand newer LLVM-emitted CREL sections.

The official Linux release builds a musl/static remote server. If you need that, follow the current target and flags in [`script/bundle-linux`](https://github.com/zed-industries/zed/blob/main/script/bundle-linux); for x86-64 Linux, that looks like:

```bash
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --package remote_server --target x86_64-unknown-linux-musl
llvm-objcopy --strip-debug target/x86_64-unknown-linux-musl/release/remote_server
```

If you do this, you must upload it to `~/.zed_server/zed-remote-server-{RELEASE_CHANNEL}-{VERSION}` on the server, for example `~/.zed_server/zed-remote-server-stable-0.217.3+stable.105.80433cb239e868271457ac376673a5f75bc4adb1`. The version must exactly match the version of Zed itself you are using.

## Maintaining the SSH connection

Once the server is initialized, Zed will create new SSH connections (reusing the existing ControlMaster) to run the remote development server.

Each connection tries to run the development server in proxy mode. This mode will start the daemon if it is not running, and reconnect to it if it is. This way when your connection drops and is restarted, you can continue to work without interruption.

In the case that reconnecting fails, the daemon will not be re-used. That said, unsaved changes are by default persisted locally, so that you do not lose work. You can always reconnect to the project at a later date and Zed will restore unsaved changes.

If you are struggling with connection issues, you should be able to see more information in the Zed log `cmd-shift-p Open Log`. If you are seeing things that are unexpected, please file a [GitHub issue](https://github.com/zed-industries/zed/issues/new) or reach out in the #support forums on [Discord](https://zed.dev/community-links).

## Supported SSH Options

Under the hood, Zed shells out to the `ssh` binary to connect to the remote server. We create one SSH control master per project, and then use that to multiplex SSH connections for the Zed protocol itself, any terminals you open and tasks you run. We read settings from your SSH config file, but if you want to specify additional options to the SSH control master you can configure Zed to set them.

When typing in the "Connect New Server" dialog, you can use bash-style quoting to pass options containing a space. Once you have created a server it will be added to the `"ssh_connections": []` array in your settings file. You can edit the settings file directly to make changes to SSH connections.

Supported options:

- `-p` / `-l` - these are equivalent to passing the port and the username in the host string.
- `-L` / `-R` for port forwarding
- `-i` - to use a specific key file
- `-o` - to set custom options
- `-J` / `-w` - to proxy the SSH connection
- `-F` for specifying an `ssh_config`
- And also... `-4`, `-6`, `-A`, `-B`, `-C`, `-D`, `-I`, `-K`, `-P`, `-X`, `-Y`, `-a`, `-b`, `-c`, `-i`, `-k`, `-l`, `-m`, `-o`, `-p`, `-w`, `-x`, `-y`

Note that we deliberately disallow some options (for example `-t` or `-T`) that Zed will set for you.

## Known Limitations

- You can't open files from the remote Terminal by typing the `zed` command.

## See also

- [Running & Testing](./running-testing.md): Run tasks, terminal commands, and
  debugger sessions while you work remotely.
- [Git Worktrees](./git.md#git-worktrees): Create and switch between linked
  Git worktrees. Zed supports the worktree picker in remote projects when the
  remote connection is active.
- [Configuring Zed](./configuring-zed.md): Manage shared and project settings,
  including `.zed/settings.json`.
- [Agent Panel](./ai/agent-panel.md): Use AI workflows in remote projects.
- [Remote Development on zed.dev](https://zed.dev/remote-development): Product
  overview and release updates.

