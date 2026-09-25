# Merge conflicts — `merge-20260924` vs upstream Zed

Generated 2026-09-24. Measured with `git merge-tree` (in-memory — the worktree and all refs were left untouched).

- ours: `merge-20260924` = `c701504082` (identical to `dev`)
- theirs: `upstream/main` = `baee1ca6de` (zed-industries/zed, fetched 2026-09-24)

## Summary

**`merge-20260924` cannot be merged with upstream at all.**

```
$ git merge-tree --write-tree merge-20260924 upstream/main
fatal: refusing to merge unrelated histories
```

`dev` (and therefore `merge-20260924`) shares **no common ancestor** with upstream:
`git merge-base merge-20260924 upstream/main` returns empty. There is no merge base, so a
"conflict count" here is not a reviewable quantity.

Forced with `--allow-unrelated-histories` (merged tree `633e06456c`):

| metric | value |
|---|---|
| conflict records | **1784** |
| affected paths | **2025** |
| `add/add` | 1543 |
| `distinct types` (LICENSE symlink vs regular file) | 241 |
| `content` | 0 |

Zero `content` conflicts is expected: with an empty merge base every file that differs is a
whole-file `add/add` collision. **This is not a merge you can review.**

---

## The mergeable branch is `main`

`main` (`942ece3607`) DOES share history with upstream (base `5e1fd392f6`).
Merging fresh upstream into it gives **126 conflicts** (merged tree `60aff41fdf`):

| type | count |
|---|---|
| content | 125 |
| modify/delete | 1 |

### By area

| area | count |
|---|---|
| `crates/editor` | 23 |
| `crates/git_ui` | 14 |
| `crates/settings_ui` | 7 |
| `crates/gpui_linux` | 7 |
| `assets` | 5 |
| `crates/zed` | 4 |
| `crates/gpui_web` | 4 |
| `crates/gpui_macos` | 4 |
| `crates/agent_ui` | 4 |
| `crates/ui` | 3 |
| `crates/title_bar` | 3 |
| `crates/search` | 3 |
| `crates/project_panel` | 3 |
| `crates/markdown` | 3 |
| `crates/extensions_ui` | 3 |
| `crates/sidebar` | 2 |
| `crates/settings_content` | 2 |
| `crates/settings` | 2 |
| `crates/markdown_preview` | 2 |
| `crates/gpui_wgpu` | 2 |
| `crates/git_ui_core` | 2 |
| `crates/ai_onboarding` | 2 |
| `(root)` | 2 |
| `crates/tasks_ui` | 1 |
| `crates/tab_switcher` | 1 |
| `crates/settings_json` | 1 |
| `crates/recent_projects` | 1 |
| `crates/picker` | 1 |
| `crates/outline_panel` | 1 |
| `crates/outline` | 1 |
| `crates/opencode` | 1 |
| `crates/onboarding` | 1 |
| `crates/language_models` | 1 |
| `crates/keymap_editor` | 1 |
| `crates/image_viewer` | 1 |
| `crates/gpui_windows` | 1 |
| `crates/gpui_apple` | 1 |
| `crates/gpui` | 1 |
| `crates/file_finder` | 1 |
| `crates/debugger_ui` | 1 |
| `crates/command_palette` | 1 |
| `crates/collab_ui` | 1 |
| `crates/agent` | 1 |

### Full list — `main` vs upstream (126)

```
CONFLICT (content): Merge conflict in Cargo.lock
CONFLICT (content): Merge conflict in Cargo.toml
CONFLICT (content): Merge conflict in assets/keymaps/default-linux.json
CONFLICT (content): Merge conflict in assets/keymaps/default-macos.json
CONFLICT (content): Merge conflict in assets/keymaps/default-windows.json
CONFLICT (content): Merge conflict in assets/keymaps/vim.json
CONFLICT (content): Merge conflict in assets/settings/default.json
CONFLICT (content): Merge conflict in crates/agent/src/thread.rs
CONFLICT (content): Merge conflict in crates/agent_ui/Cargo.toml
CONFLICT (content): Merge conflict in crates/agent_ui/src/agent_diff.rs
CONFLICT (content): Merge conflict in crates/agent_ui/src/agent_panel.rs
CONFLICT (content): Merge conflict in crates/agent_ui/src/entry_view_state.rs
CONFLICT (content): Merge conflict in crates/ai_onboarding/src/ai_onboarding.rs
CONFLICT (content): Merge conflict in crates/ai_onboarding/src/plan_definitions.rs
CONFLICT (content): Merge conflict in crates/collab_ui/src/collab_panel.rs
CONFLICT (content): Merge conflict in crates/command_palette/src/command_palette.rs
CONFLICT (content): Merge conflict in crates/debugger_ui/src/debugger_panel.rs
CONFLICT (content): Merge conflict in crates/editor/src/actions.rs
CONFLICT (content): Merge conflict in crates/editor/src/bookmarks.rs
CONFLICT (content): Merge conflict in crates/editor/src/code_context_menus.rs
CONFLICT (content): Merge conflict in crates/editor/src/completions.rs
CONFLICT (content): Merge conflict in crates/editor/src/diagnostics.rs
CONFLICT (content): Merge conflict in crates/editor/src/display_map.rs
CONFLICT (content): Merge conflict in crates/editor/src/display_map/block_map.rs
CONFLICT (content): Merge conflict in crates/editor/src/document_symbols.rs
CONFLICT (content): Merge conflict in crates/editor/src/editor.rs
CONFLICT (content): Merge conflict in crates/editor/src/editor_settings.rs
CONFLICT (content): Merge conflict in crates/editor/src/editor_tests.rs
CONFLICT (content): Merge conflict in crates/editor/src/element.rs
CONFLICT (content): Merge conflict in crates/editor/src/git.rs
CONFLICT (content): Merge conflict in crates/editor/src/git/blame.rs
CONFLICT (content): Merge conflict in crates/editor/src/hover_links.rs
CONFLICT (content): Merge conflict in crates/editor/src/items.rs
CONFLICT (content): Merge conflict in crates/editor/src/mouse_context_menu.rs
CONFLICT (content): Merge conflict in crates/editor/src/movement.rs
CONFLICT (content): Merge conflict in crates/editor/src/navigation.rs
CONFLICT (content): Merge conflict in crates/editor/src/scroll/autoscroll.rs
CONFLICT (content): Merge conflict in crates/editor/src/selection.rs
CONFLICT (content): Merge conflict in crates/editor/src/selections_collection.rs
CONFLICT (content): Merge conflict in crates/editor/src/split.rs
CONFLICT (content): Merge conflict in crates/extensions_ui/src/components/extension_card.rs
CONFLICT (modify/delete): crates/extensions_ui/src/extension_suggest.rs deleted in upstream/main and modified in main.  Version main of crates/extensions_ui/src/extension_suggest.rs left in tree.
CONFLICT (content): Merge conflict in crates/extensions_ui/src/extensions_ui.rs
CONFLICT (content): Merge conflict in crates/file_finder/src/file_finder.rs
CONFLICT (content): Merge conflict in crates/git_ui/Cargo.toml
CONFLICT (content): Merge conflict in crates/git_ui/src/blame_ui.rs
CONFLICT (content): Merge conflict in crates/git_ui/src/branch_picker.rs
CONFLICT (content): Merge conflict in crates/git_ui/src/commit_tooltip.rs
CONFLICT (content): Merge conflict in crates/git_ui/src/commit_view.rs
CONFLICT (content): Merge conflict in crates/git_ui/src/conflict_view.rs
CONFLICT (content): Merge conflict in crates/git_ui/src/git_graph.rs
CONFLICT (content): Merge conflict in crates/git_ui/src/git_panel.rs
CONFLICT (content): Merge conflict in crates/git_ui/src/git_panel_settings.rs
CONFLICT (content): Merge conflict in crates/git_ui/src/git_ui.rs
CONFLICT (content): Merge conflict in crates/git_ui/src/multi_diff_view.rs
CONFLICT (content): Merge conflict in crates/git_ui/src/project_diff.rs
CONFLICT (content): Merge conflict in crates/git_ui/src/solo_diff_view.rs
CONFLICT (content): Merge conflict in crates/git_ui/src/text_diff_view.rs
CONFLICT (content): Merge conflict in crates/git_ui_core/src/file_diff_view.rs
CONFLICT (content): Merge conflict in crates/git_ui_core/src/worktree_picker.rs
CONFLICT (content): Merge conflict in crates/gpui/src/window.rs
CONFLICT (content): Merge conflict in crates/gpui_apple/src/metal_renderer.rs
CONFLICT (content): Merge conflict in crates/gpui_linux/Cargo.toml
CONFLICT (content): Merge conflict in crates/gpui_linux/src/linux/headless/client.rs
CONFLICT (content): Merge conflict in crates/gpui_linux/src/linux/platform.rs
CONFLICT (content): Merge conflict in crates/gpui_linux/src/linux/wayland/client.rs
CONFLICT (content): Merge conflict in crates/gpui_linux/src/linux/wayland/window.rs
CONFLICT (content): Merge conflict in crates/gpui_linux/src/linux/x11/client.rs
CONFLICT (content): Merge conflict in crates/gpui_linux/src/linux/x11/window.rs
CONFLICT (content): Merge conflict in crates/gpui_macos/Cargo.toml
CONFLICT (content): Merge conflict in crates/gpui_macos/src/dispatcher.rs
CONFLICT (content): Merge conflict in crates/gpui_macos/src/platform.rs
CONFLICT (content): Merge conflict in crates/gpui_macos/src/window.rs
CONFLICT (content): Merge conflict in crates/gpui_web/Cargo.toml
CONFLICT (content): Merge conflict in crates/gpui_web/src/events.rs
CONFLICT (content): Merge conflict in crates/gpui_web/src/gpui_web.rs
CONFLICT (content): Merge conflict in crates/gpui_web/src/platform.rs
CONFLICT (content): Merge conflict in crates/gpui_wgpu/src/shaders.wgsl
CONFLICT (content): Merge conflict in crates/gpui_wgpu/src/wgpu_renderer.rs
CONFLICT (content): Merge conflict in crates/gpui_windows/Cargo.toml
CONFLICT (content): Merge conflict in crates/image_viewer/src/image_viewer.rs
CONFLICT (content): Merge conflict in crates/keymap_editor/src/keymap_editor.rs
CONFLICT (content): Merge conflict in crates/language_models/src/provider/opencode.rs
CONFLICT (content): Merge conflict in crates/markdown/Cargo.toml
CONFLICT (content): Merge conflict in crates/markdown/src/markdown.rs
CONFLICT (content): Merge conflict in crates/markdown/src/mermaid.rs
CONFLICT (content): Merge conflict in crates/markdown_preview/Cargo.toml
CONFLICT (content): Merge conflict in crates/markdown_preview/src/markdown_preview_view.rs
CONFLICT (content): Merge conflict in crates/onboarding/src/onboarding.rs
CONFLICT (content): Merge conflict in crates/opencode/src/opencode.rs
CONFLICT (content): Merge conflict in crates/outline/src/outline.rs
CONFLICT (content): Merge conflict in crates/outline_panel/src/outline_panel.rs
CONFLICT (content): Merge conflict in crates/picker/src/picker.rs
CONFLICT (content): Merge conflict in crates/project_panel/src/project_panel.rs
CONFLICT (content): Merge conflict in crates/project_panel/src/project_panel_tests.rs
CONFLICT (content): Merge conflict in crates/project_panel/src/tests/undo.rs
CONFLICT (content): Merge conflict in crates/recent_projects/src/remote_servers.rs
CONFLICT (content): Merge conflict in crates/search/Cargo.toml
CONFLICT (content): Merge conflict in crates/search/src/buffer_search.rs
CONFLICT (content): Merge conflict in crates/search/src/project_search.rs
CONFLICT (content): Merge conflict in crates/settings/src/settings_store.rs
CONFLICT (content): Merge conflict in crates/settings/src/vscode_import.rs
CONFLICT (content): Merge conflict in crates/settings_content/src/editor.rs
CONFLICT (content): Merge conflict in crates/settings_content/src/fallible_options.rs
CONFLICT (content): Merge conflict in crates/settings_json/Cargo.toml
CONFLICT (content): Merge conflict in crates/settings_ui/src/components/dropdown.rs
CONFLICT (content): Merge conflict in crates/settings_ui/src/components/font_picker.rs
CONFLICT (content): Merge conflict in crates/settings_ui/src/components/icon_theme_picker.rs
CONFLICT (content): Merge conflict in crates/settings_ui/src/components/ollama_model_picker.rs
CONFLICT (content): Merge conflict in crates/settings_ui/src/components/theme_picker.rs
CONFLICT (content): Merge conflict in crates/settings_ui/src/page_data.rs
CONFLICT (content): Merge conflict in crates/settings_ui/src/settings_ui.rs
CONFLICT (content): Merge conflict in crates/sidebar/src/sidebar.rs
CONFLICT (content): Merge conflict in crates/sidebar/src/sidebar_tests.rs
CONFLICT (content): Merge conflict in crates/tab_switcher/src/tab_switcher.rs
CONFLICT (content): Merge conflict in crates/tasks_ui/Cargo.toml
CONFLICT (content): Merge conflict in crates/title_bar/src/application_menu.rs
CONFLICT (content): Merge conflict in crates/title_bar/src/title_bar.rs
CONFLICT (content): Merge conflict in crates/title_bar/src/update_version.rs
CONFLICT (content): Merge conflict in crates/ui/src/components/button/button_like.rs
CONFLICT (content): Merge conflict in crates/ui/src/components/context_menu.rs
CONFLICT (content): Merge conflict in crates/ui/src/components/image.rs
CONFLICT (content): Merge conflict in crates/zed/resources/info/Permissions.plist
CONFLICT (content): Merge conflict in crates/zed/src/zed.rs
CONFLICT (content): Merge conflict in crates/zed/src/zed/app_menus.rs
CONFLICT (content): Merge conflict in crates/zed/src/zed/quick_action_bar/preview.rs
```

---

## `merge-20260924` (forced) — affected paths by area

| area | count |
|---|---|
| `crates/gpui` | 96 |
| `crates/editor` | 64 |
| `crates/ui` | 57 |
| `crates/project` | 49 |
| `crates/agent` | 48 |
| `crates/agent_ui` | 41 |
| `crates/zed` | 40 |
| `crates/grammars` | 36 |
| `tooling/xtask` | 33 |
| `crates/collab` | 29 |
| `crates/vim` | 25 |
| `crates/language_models` | 25 |
| `crates/workspace` | 24 |
| `crates/git_ui` | 22 |
| `crates/settings_ui` | 21 |
| `crates/edit_prediction_cli` | 21 |
| `crates/extension_host` | 20 |
| `crates/debugger_ui` | 20 |
| `assets` | 20 |
| `crates/gpui_windows` | 19 |
| `crates/language` | 18 |
| `crates/gpui_web` | 18 |
| `crates/gpui_linux` | 18 |
| `crates/settings_content` | 15 |
| `crates/proto` | 15 |
| `crates/languages` | 15 |
| `crates/edit_prediction` | 15 |
| `crates/mermaid_render` | 14 |
| `crates/repl` | 12 |
| `crates/language_core` | 12 |
| `crates/git` | 12 |
| `crates/util` | 11 |
| `crates/settings` | 11 |
| `crates/remote` | 11 |
| `crates/livekit_client` | 11 |
| `crates/gpui_macos` | 11 |
| `crates/eval_cli` | 11 |
| `crates/collab_ui` | 11 |
| `crates/terminal_view` | 10 |
| `crates/recent_projects` | 10 |
| `crates/project_panel` | 10 |
| `crates/markdown` | 10 |
| `crates/context_server` | 10 |
| `crates/client` | 10 |
| `crates/title_bar` | 9 |
| `crates/terminal` | 9 |
| `crates/gpui_wgpu` | 9 |
| `crates/dev_container` | 9 |
| `crates/worktree` | 8 |
| `crates/theme` | 8 |
| `crates/language_tools` | 8 |
| `crates/language_model_core` | 8 |
| `crates/language_model` | 8 |
| `crates/gpui_macros` | 8 |
| `crates/edit_prediction_metrics` | 8 |
| `crates/diagnostics` | 8 |
| `crates/cloud_api_types` | 8 |
| `crates/acp_thread` | 8 |
| `crates/text` | 7 |
| `crates/search` | 7 |
| `crates/scheduler` | 7 |
| `crates/picker` | 7 |
| `crates/open_ai` | 7 |
| `crates/onboarding` | 7 |
| `crates/multi_buffer` | 7 |
| `crates/migrator` | 7 |
| `crates/git_hosting_providers` | 7 |
| `crates/fs` | 7 |
| `crates/extensions_ui` | 7 |
| `crates/extension` | 7 |
| `crates/edit_prediction_context` | 7 |
| `crates/benchmarks` | 7 |
| `crates/agent_servers` | 7 |
| `extensions/proto` | 6 |
| `crates/zeta_prompt` | 6 |
| `crates/sidebar` | 6 |
| `crates/remote_server` | 6 |
| `crates/refineable` | 6 |
| `crates/platform_title_bar` | 6 |
| `crates/open_path_prompt` | 6 |
| `crates/livekit_api` | 6 |
| `crates/keymap_editor` | 6 |
| `crates/json_schema_store` | 6 |
| `crates/http_client` | 6 |
| `crates/fuzzy_nucleo` | 6 |
| `crates/fuzzy` | 6 |
| `crates/dap` | 6 |
| `crates/call` | 6 |
| `crates/audio` | 6 |
| `crates/anthropic` | 6 |
| `crates/ai_onboarding` | 6 |
| `(root)` | 6 |
| `nix` | 5 |
| `crates/which_key` | 5 |
| `crates/ui_input` | 5 |
| `crates/toolchain_selector` | 5 |
| `crates/theme_settings` | 5 |
| `crates/theme_selector` | 5 |
| `crates/tasks_ui` | 5 |
| `crates/syntax_theme` | 5 |
| `crates/sqlez` | 5 |
| `crates/sandbox` | 5 |
| `crates/rpc` | 5 |
| `crates/prompt_store` | 5 |
| `crates/outline_panel` | 5 |
| `crates/markdown_preview` | 5 |
| `crates/lsp` | 5 |
| `crates/line_ending_selector` | 5 |
| `crates/language_extension` | 5 |
| `crates/inspector_ui` | 5 |
| `crates/http_proxy` | 5 |
| `crates/google_ai` | 5 |
| `crates/go_to_line` | 5 |
| `crates/file_finder` | 5 |
| `crates/encoding_selector` | 5 |
| `crates/edit_prediction_ui` | 5 |
| `crates/copilot_chat` | 5 |
| `crates/copilot` | 5 |
| `crates/component_preview` | 5 |
| `crates/command_palette` | 5 |
| `crates/cloud_llm_client` | 5 |
| `crates/cloud_api_client` | 5 |
| `crates/cli` | 5 |
| `crates/channel` | 5 |
| `crates/bedrock` | 5 |
| `crates/agent_settings` | 5 |
| `tooling/perf` | 4 |
| `extensions/html` | 4 |
| `extensions/glsl` | 4 |
| `crates/ztracing` | 4 |
| `crates/zed_actions` | 4 |
| `crates/x_ai` | 4 |
| `crates/windows_resources` | 4 |
| `crates/watch` | 4 |
| `crates/ui_prompt` | 4 |
| `crates/task` | 4 |
| `crates/tab_switcher` | 4 |
| `crates/svg_preview` | 4 |
| `crates/sum_tree` | 4 |
| `crates/sqlez_macros` | 4 |
| `crates/snippets_ui` | 4 |
| `crates/snippet_provider` | 4 |
| `crates/settings_json` | 4 |
| `crates/schema_generator` | 4 |
| `crates/reqwest_client` | 4 |
| `crates/remote_connection` | 4 |
| `crates/release_channel` | 4 |
| `crates/project_symbols` | 4 |
| `crates/paths` | 4 |
| `crates/outline` | 4 |
| `crates/opencode` | 4 |
| `crates/open_router` | 4 |
| `crates/ollama` | 4 |
| `crates/notifications` | 4 |
| `crates/node_runtime` | 4 |
| `crates/mistral` | 4 |
| `crates/miniprofiler_ui` | 4 |
| `crates/lmstudio` | 4 |
| `crates/language_selector` | 4 |
| `crates/language_models_cloud` | 4 |
| `crates/install_cli` | 4 |
| `crates/input_latency_ui` | 4 |
| `crates/image_viewer` | 4 |
| `crates/icons` | 4 |
| `crates/http_client_tls` | 4 |
| `crates/html_to_markdown` | 4 |
| `crates/gpui_util` | 4 |
| `crates/gpui_tokio` | 4 |
| `crates/gpui_shared_string` | 4 |
| `crates/gpui_platform` | 4 |
| `crates/file_icons` | 4 |
| `crates/feedback` | 4 |
| `crates/feature_flags` | 4 |
| `crates/extension_cli` | 4 |
| `crates/extension_api` | 4 |
| `crates/explorer_command_injector` | 4 |
| `crates/etw_tracing` | 4 |
| `crates/edit_prediction_types` | 4 |
| `crates/docs_preprocessor` | 4 |
| `crates/deepseek` | 4 |
| `crates/crashes` | 4 |
| `crates/copilot_ui` | 4 |
| `crates/codestral` | 4 |
| `crates/buffer_diff` | 4 |
| `crates/breadcrumbs` | 4 |
| `crates/auto_update_helper` | 4 |
| `crates/assets` | 4 |
| `crates/askpass` | 4 |
| `crates/agent_skills` | 4 |
| `crates/activity_indicator` | 4 |
| `crates/action_log` | 4 |
| `crates/acp_tools` | 4 |
| `.github` | 4 |
| `tooling/compliance` | 3 |
| `script/danger` | 3 |
| `legal` | 3 |
| `extensions/test-extension` | 3 |
| `crates/ztracing_macro` | 3 |
| `crates/zlog_settings` | 3 |
| `crates/zed_env_vars` | 3 |
| `crates/zed_credentials_provider` | 3 |
| `crates/worktree_benchmarks` | 3 |
| `crates/web_search_providers` | 3 |
| `crates/web_search` | 3 |
| `crates/vim_mode_setting` | 3 |
| `crates/util_macros` | 3 |
| `crates/ui_macros` | 3 |
| `crates/theme_importer` | 3 |
| `crates/theme_extension` | 3 |
| `crates/telemetry_events` | 3 |
| `crates/telemetry` | 3 |
| `crates/system_specs` | 3 |
| `crates/streaming_diff` | 3 |
| `crates/snippet` | 3 |
| `crates/shell_command_parser` | 3 |
| `crates/settings_profile_selector` | 3 |
| `crates/settings_macros` | 3 |
| `crates/session` | 3 |
| `crates/rope` | 3 |
| `crates/project_benchmarks` | 3 |
| `crates/oauth_callback_server` | 3 |
| `crates/net` | 3 |
| `crates/menu` | 3 |
| `crates/language_onboarding` | 3 |
| `crates/journal` | 3 |
| `crates/feature_flags_macros` | 3 |
| `crates/eval_utils` | 3 |
| `crates/env_var` | 3 |
| `crates/debug_adapter_extension` | 3 |
| `crates/dap_adapters` | 3 |
| `crates/credentials_provider` | 3 |
| `crates/component` | 3 |
| `crates/command_palette_hooks` | 3 |
| `crates/aws_http_client` | 3 |
| `crates/auto_update_ui` | 3 |
| `crates/auto_update` | 3 |
| `script/flatpak` | 2 |
| `extensions/workflows` | 2 |
| `crates/zlog` | 2 |
| `crates/time_format` | 2 |
| `crates/prettier` | 2 |
| `crates/fs_benchmarks` | 2 |
| `crates/editor_benchmarks` | 2 |
| `crates/debugger_tools` | 2 |
| `crates/db` | 2 |
| `crates/collections` | 2 |
| `crates/clock` | 2 |
| `script/zed-local` | 1 |
| `script/what-is-deployed` | 1 |
| `script/verify-macos-document-icon` | 1 |
| `script/upload-nightly` | 1 |
| `script/upload-extension-cli` | 1 |
| `script/update_top_ranking_issues` | 1 |
| `script/update-json-schemas` | 1 |
| `script/trigger-release` | 1 |
| `script/trigger-docs-build` | 1 |
| `script/test-docs-suggest-batch` | 1 |
| `script/terms` | 1 |
| `script/squawk` | 1 |
| `script/snap-try` | 1 |
| `script/snap-build` | 1 |
| `script/shellcheck-scripts` | 1 |
| `script/setup-sccache.ps1` | 1 |
| `script/setup-sccache` | 1 |
| `script/sentry-fetch` | 1 |
| `script/select-sentry-crash-candidates` | 1 |
| `script/seed-db` | 1 |
| `script/run-unit-evals` | 1 |
| `script/run-local-minio` | 1 |
| `script/run-background-agent-mvp-local` | 1 |
| `script/reset_db` | 1 |
| `script/remote-server` | 1 |
| `script/randomized-test-minimize` | 1 |
| `script/randomized-test-ci` | 1 |
| `script/prompts` | 1 |
| `script/prettier` | 1 |
| `script/new-crate` | 1 |
| `script/mitm-proxy.sh` | 1 |
| `script/metal-debug` | 1 |
| `script/linux` | 1 |
| `script/licenses` | 1 |
| `script/lib` | 1 |
| `script/kube-shell` | 1 |
| `script/install.sh` | 1 |
| `script/install-linux` | 1 |
| `script/install-cmake` | 1 |
| `script/import-themes` | 1 |
| `script/histogram` | 1 |
| `script/github-track-duplicate-bot-effectiveness.py` | 1 |
| `script/github-pr-status` | 1 |
| `script/github-community-pr-board.py` | 1 |
| `script/github-clean-issue-types.py` | 1 |
| `script/github-check-new-issue-for-duplicates.py` | 1 |
| `script/get-released-version` | 1 |
| `script/get-release-notes-since` | 1 |
| `script/get-pull-requests-since` | 1 |
| `script/get-crate-version` | 1 |
| `script/generate-terms-rtf` | 1 |
| `script/generate-licenses-csv` | 1 |
| `script/generate-licenses` | 1 |
| `script/generate-action-metadata` | 1 |
| `script/freebsd` | 1 |
| `script/drop-test-dbs` | 1 |
| `script/draft-release-notes` | 1 |
| `script/download-wasi-sdk` | 1 |
| `script/docs-suggest-publish` | 1 |
| `script/docs-suggest` | 1 |
| `script/docs-strip-preview-callouts` | 1 |
| `script/digital-ocean-db.sh` | 1 |
| `script/determine-release-channel` | 1 |
| `script/deploy-collab` | 1 |
| `script/debug-cli` | 1 |
| `script/create-draft-release` | 1 |
| `script/crate-dep-graph` | 1 |
| `script/community-pr-track-mapping.json` | 1 |
| `script/collab-flamegraph` | 1 |
| `script/clippy` | 1 |
| `script/clear-target-dir-if-larger-than` | 1 |
| `script/cherry-pick` | 1 |
| `script/check-todos` | 1 |
| `script/check-links` | 1 |
| `script/check-licenses` | 1 |
| `script/check-keymaps` | 1 |
| `script/cargo-timing-info.js` | 1 |
| `script/cargo` | 1 |
| `script/bundle-windows.ps1` | 1 |
| `script/bundle-mac` | 1 |
| `script/bundle-linux` | 1 |
| `script/bundle-freebsd` | 1 |
| `script/bump-zed-version` | 1 |
| `script/bump-nightly` | 1 |
| `script/bump-gpui-version` | 1 |
| `script/bump-extension-cli` | 1 |
| `script/build-docker` | 1 |
| `script/bootstrap` | 1 |
| `.cloudflare` | 1 |
| `.agents` | 1 |

### Full list — `merge-20260924` forced (2025 paths)

```
.agents/skills/gpui-test/SKILL.md
.cloudflare/docs-proxy/src/worker.js
.gitattributes
.github/actionlint.yml
.github/actions/run_tests/action.yml
.github/actions/run_tests_windows/action.yml
.github/workflows/community_pr_board_refresh.yml
.gitignore
Cargo.lock
Cargo.toml
README.md
assets/badge/v0.json
assets/images/zed_logo.svg
assets/keymaps/default-linux.json
assets/keymaps/default-macos.json
assets/keymaps/default-windows.json
assets/keymaps/linux/cursor.json
assets/keymaps/linux/emacs.json
assets/keymaps/linux/jetbrains.json
assets/keymaps/macos/cursor.json
assets/keymaps/macos/emacs.json
assets/keymaps/macos/jetbrains.json
assets/keymaps/vim.json
assets/settings/default.json
assets/settings/default_semantic_token_rules.json
assets/settings/initial_tasks.json
assets/settings/initial_user_settings.json
assets/sounds/agent_done.wav
assets/themes/ayu/ayu.json
assets/themes/gruvbox/gruvbox.json
assets/themes/one/one.json
crates/acp_thread/Cargo.toml
crates/acp_thread/LICENSE-GPL
crates/acp_thread/LICENSE-GPL~merge-20260924
crates/acp_thread/src/acp_thread.rs
crates/acp_thread/src/connection.rs
crates/acp_thread/src/diff.rs
crates/acp_thread/src/mention.rs
crates/acp_thread/src/terminal.rs
crates/acp_tools/Cargo.toml
crates/acp_tools/LICENSE-GPL
crates/acp_tools/LICENSE-GPL~merge-20260924
crates/acp_tools/src/acp_tools.rs
crates/action_log/Cargo.toml
crates/action_log/LICENSE-GPL
crates/action_log/LICENSE-GPL~merge-20260924
crates/action_log/src/action_log.rs
crates/activity_indicator/Cargo.toml
crates/activity_indicator/LICENSE-GPL
crates/activity_indicator/LICENSE-GPL~merge-20260924
crates/activity_indicator/src/activity_indicator.rs
crates/agent/Cargo.toml
crates/agent/LICENSE-GPL
crates/agent/LICENSE-GPL~merge-20260924
crates/agent/src/agent.rs
crates/agent/src/db.rs
crates/agent/src/native_agent_server.rs
crates/agent/src/sandboxing.rs
crates/agent/src/templates.rs
crates/agent/src/templates/experimental_system_prompt.hbs
crates/agent/src/templates/system_prompt.hbs
crates/agent/src/tests/mod.rs
crates/agent/src/thread.rs
crates/agent/src/thread_store.rs
crates/agent/src/tool_permissions.rs
crates/agent/src/tools.rs
crates/agent/src/tools/apply_code_action_tool.rs
crates/agent/src/tools/context_server_registry.rs
crates/agent/src/tools/copy_path_tool.rs
crates/agent/src/tools/create_directory_tool.rs
crates/agent/src/tools/create_thread_tool.rs
crates/agent/src/tools/delete_path_tool.rs
crates/agent/src/tools/diagnostics_tool.rs
crates/agent/src/tools/edit_file_tool.rs
crates/agent/src/tools/edit_session.rs
crates/agent/src/tools/edit_session/reindent.rs
crates/agent/src/tools/edit_session/streaming_fuzzy_matcher.rs
crates/agent/src/tools/edit_session/streaming_parser.rs
crates/agent/src/tools/evals.rs
crates/agent/src/tools/evals/edit_file.rs
crates/agent/src/tools/evals/terminal_tool.rs
crates/agent/src/tools/evals/write_file.rs
crates/agent/src/tools/fetch_tool.rs
crates/agent/src/tools/find_path_tool.rs
crates/agent/src/tools/find_references_tool.rs
crates/agent/src/tools/get_code_actions_tool.rs
crates/agent/src/tools/go_to_definition_tool.rs
crates/agent/src/tools/grep_tool.rs
crates/agent/src/tools/list_agents_and_models_tool.rs
crates/agent/src/tools/list_directory_tool.rs
crates/agent/src/tools/move_path_tool.rs
crates/agent/src/tools/read_file_tool.rs
crates/agent/src/tools/rename_tool.rs
crates/agent/src/tools/skill_tool.rs
crates/agent/src/tools/spawn_agent_tool.rs
crates/agent/src/tools/terminal_tool.rs
crates/agent/src/tools/tool_permissions.rs
crates/agent/src/tools/web_search_tool.rs
crates/agent/src/tools/write_file_tool.rs
crates/agent_servers/Cargo.toml
crates/agent_servers/LICENSE-GPL
crates/agent_servers/LICENSE-GPL~merge-20260924
crates/agent_servers/src/acp.rs
crates/agent_servers/src/agent_servers.rs
crates/agent_servers/src/custom.rs
crates/agent_servers/src/e2e_tests.rs
crates/agent_settings/Cargo.toml
crates/agent_settings/LICENSE-GPL
crates/agent_settings/LICENSE-GPL~merge-20260924
crates/agent_settings/src/agent_profile.rs
crates/agent_settings/src/agent_settings.rs
crates/agent_skills/Cargo.toml
crates/agent_skills/LICENSE-GPL
crates/agent_skills/LICENSE-GPL~merge-20260924
crates/agent_skills/agent_skills.rs
crates/agent_ui/Cargo.toml
crates/agent_ui/LICENSE-GPL
crates/agent_ui/LICENSE-GPL~merge-20260924
crates/agent_ui/src/agent_configuration.rs
crates/agent_ui/src/agent_configuration/configure_context_server_modal.rs
crates/agent_ui/src/agent_configuration/manage_profiles_modal.rs
crates/agent_ui/src/agent_configuration/tool_picker.rs
crates/agent_ui/src/agent_diff.rs
crates/agent_ui/src/agent_model_selector.rs
crates/agent_ui/src/agent_panel.rs
crates/agent_ui/src/agent_registry_ui.rs
crates/agent_ui/src/agent_ui.rs
crates/agent_ui/src/buffer_codegen.rs
crates/agent_ui/src/completion_provider.rs
crates/agent_ui/src/config_options.rs
crates/agent_ui/src/context_server_configuration.rs
crates/agent_ui/src/conversation_view.rs
crates/agent_ui/src/conversation_view/thread_view.rs
crates/agent_ui/src/diagnostics.rs
crates/agent_ui/src/draft_prompt_store.rs
crates/agent_ui/src/entry_view_state.rs
crates/agent_ui/src/inline_assistant.rs
crates/agent_ui/src/inline_prompt_editor.rs
crates/agent_ui/src/language_model_selector.rs
crates/agent_ui/src/mention_set.rs
crates/agent_ui/src/message_editor.rs
crates/agent_ui/src/mode_selector.rs
crates/agent_ui/src/model_selector.rs
crates/agent_ui/src/model_selector_popover.rs
crates/agent_ui/src/profile_selector.rs
crates/agent_ui/src/terminal_inline_assistant.rs
crates/agent_ui/src/terminal_thread_metadata_store.rs
crates/agent_ui/src/test_support.rs
crates/agent_ui/src/thread_import.rs
crates/agent_ui/src/thread_metadata_store.rs
crates/agent_ui/src/thread_worktree_archive.rs
crates/agent_ui/src/threads_archive_view.rs
crates/agent_ui/src/ui.rs
crates/agent_ui/src/ui/end_trial_upsell.rs
crates/agent_ui/src/ui/mention_crease.rs
crates/agent_ui/src/ui/model_selector_components.rs
crates/ai_onboarding/Cargo.toml
crates/ai_onboarding/LICENSE-GPL
crates/ai_onboarding/LICENSE-GPL~merge-20260924
crates/ai_onboarding/src/agent_api_keys_onboarding.rs
crates/ai_onboarding/src/ai_onboarding.rs
crates/ai_onboarding/src/plan_definitions.rs
crates/anthropic/Cargo.toml
crates/anthropic/LICENSE-GPL
crates/anthropic/LICENSE-GPL~merge-20260924
crates/anthropic/src/anthropic.rs
crates/anthropic/src/batches.rs
crates/anthropic/src/completion.rs
crates/askpass/Cargo.toml
crates/askpass/LICENSE-GPL
crates/askpass/LICENSE-GPL~merge-20260924
crates/askpass/src/askpass.rs
crates/assets/Cargo.toml
crates/assets/LICENSE-GPL
crates/assets/LICENSE-GPL~merge-20260924
crates/assets/src/assets.rs
crates/audio/Cargo.toml
crates/audio/LICENSE-GPL
crates/audio/LICENSE-GPL~merge-20260924
crates/audio/src/audio.rs
crates/audio/src/audio_pipeline.rs
crates/audio/src/audio_settings.rs
crates/auto_update/LICENSE-GPL
crates/auto_update/LICENSE-GPL~merge-20260924
crates/auto_update/src/auto_update.rs
crates/auto_update_helper/LICENSE-GPL
crates/auto_update_helper/LICENSE-GPL~merge-20260924
crates/auto_update_helper/src/auto_update_helper.rs
crates/auto_update_helper/src/updater.rs
crates/auto_update_ui/Cargo.toml
crates/auto_update_ui/LICENSE-GPL
crates/auto_update_ui/LICENSE-GPL~merge-20260924
crates/aws_http_client/Cargo.toml
crates/aws_http_client/LICENSE-GPL
crates/aws_http_client/LICENSE-GPL~merge-20260924
crates/bedrock/Cargo.toml
crates/bedrock/LICENSE-GPL
crates/bedrock/LICENSE-GPL~merge-20260924
crates/bedrock/src/bedrock.rs
crates/bedrock/src/models.rs
crates/benchmarks/Cargo.toml
crates/benchmarks/LICENSE-GPL
crates/benchmarks/LICENSE-GPL~merge-20260924
crates/benchmarks/benches/display_map.rs
crates/benchmarks/benches/edit_file_tool.rs
crates/benchmarks/benches/editor_render.rs
crates/benchmarks/src/benchmarks.rs
crates/breadcrumbs/Cargo.toml
crates/breadcrumbs/LICENSE-GPL
crates/breadcrumbs/LICENSE-GPL~merge-20260924
crates/breadcrumbs/src/breadcrumbs.rs
crates/buffer_diff/Cargo.toml
crates/buffer_diff/LICENSE-GPL
crates/buffer_diff/LICENSE-GPL~merge-20260924
crates/buffer_diff/src/buffer_diff.rs
crates/call/Cargo.toml
crates/call/LICENSE-GPL
crates/call/LICENSE-GPL~merge-20260924
crates/call/src/call_impl/diagnostics.rs
crates/call/src/call_impl/mod.rs
crates/call/src/call_impl/room.rs
crates/channel/Cargo.toml
crates/channel/LICENSE-GPL
crates/channel/LICENSE-GPL~merge-20260924
crates/channel/src/channel_buffer.rs
crates/channel/src/channel_store.rs
crates/cli/Cargo.toml
crates/cli/LICENSE-GPL
crates/cli/LICENSE-GPL~merge-20260924
crates/cli/src/cli.rs
crates/cli/src/main.rs
crates/client/Cargo.toml
crates/client/LICENSE-GPL
crates/client/LICENSE-GPL~merge-20260924
crates/client/src/client.rs
crates/client/src/llm_token.rs
crates/client/src/proxy.rs
crates/client/src/telemetry.rs
crates/client/src/test.rs
crates/client/src/user.rs
crates/client/src/zed_urls.rs
crates/clock/LICENSE-GPL
crates/clock/LICENSE-GPL~merge-20260924
crates/cloud_api_client/Cargo.toml
crates/cloud_api_client/LICENSE-APACHE
crates/cloud_api_client/LICENSE-APACHE~merge-20260924
crates/cloud_api_client/src/cloud_api_client.rs
crates/cloud_api_client/src/websocket.rs
crates/cloud_api_types/Cargo.toml
crates/cloud_api_types/LICENSE-APACHE
crates/cloud_api_types/LICENSE-APACHE~merge-20260924
crates/cloud_api_types/src/cloud_api_types.rs
crates/cloud_api_types/src/extension.rs
crates/cloud_api_types/src/internal_api.rs
crates/cloud_api_types/src/plan.rs
crates/cloud_api_types/src/websocket_protocol.rs
crates/cloud_llm_client/Cargo.toml
crates/cloud_llm_client/LICENSE-APACHE
crates/cloud_llm_client/LICENSE-APACHE~merge-20260924
crates/cloud_llm_client/src/cloud_llm_client.rs
crates/cloud_llm_client/src/predict_edits_v3.rs
crates/codestral/Cargo.toml
crates/codestral/LICENSE-GPL
crates/codestral/LICENSE-GPL~merge-20260924
crates/codestral/src/codestral.rs
crates/collab/Cargo.toml
crates/collab/LICENSE-GPL
crates/collab/LICENSE-GPL~merge-20260924
crates/collab/migrations.sqlite/20221109000000_test_schema.sql
crates/collab/migrations/20251208000000_test_schema.sql
crates/collab/src/api/extensions.rs
crates/collab/src/auth.rs
crates/collab/src/db.rs
crates/collab/src/db/queries.rs
crates/collab/src/db/queries/channels.rs
crates/collab/src/db/queries/projects.rs
crates/collab/src/db/queries/rooms.rs
crates/collab/src/db/tables.rs
crates/collab/src/db/tables/worktree_entry.rs
crates/collab/src/entities/user.rs
crates/collab/src/lib.rs
crates/collab/src/main.rs
crates/collab/src/rpc.rs
crates/collab/src/services/user_service.rs
crates/collab/tests/integration/channel_tests.rs
crates/collab/tests/integration/collab_tests.rs
crates/collab/tests/integration/db_tests/db_tests.rs
crates/collab/tests/integration/editor_tests.rs
crates/collab/tests/integration/following_tests.rs
crates/collab/tests/integration/git_tests.rs
crates/collab/tests/integration/integration_tests.rs
crates/collab/tests/integration/random_project_collaboration_tests.rs
crates/collab/tests/integration/remote_editing_collaboration_tests.rs
crates/collab/tests/integration/test_server.rs
crates/collab_ui/Cargo.toml
crates/collab_ui/LICENSE-GPL
crates/collab_ui/LICENSE-GPL~merge-20260924
crates/collab_ui/src/call_stats_modal.rs
crates/collab_ui/src/channel_view.rs
crates/collab_ui/src/collab_panel.rs
crates/collab_ui/src/collab_panel/channel_modal.rs
crates/collab_ui/src/collab_panel/contact_finder.rs
crates/collab_ui/src/notifications/incoming_call_notification.rs
crates/collab_ui/src/notifications/project_shared_notification.rs
crates/collab_ui/src/panel_settings.rs
crates/collections/LICENSE-APACHE
crates/collections/LICENSE-APACHE~merge-20260924
crates/command_palette/Cargo.toml
crates/command_palette/LICENSE-GPL
crates/command_palette/LICENSE-GPL~merge-20260924
crates/command_palette/src/command_palette.rs
crates/command_palette/src/persistence.rs
crates/command_palette_hooks/Cargo.toml
crates/command_palette_hooks/LICENSE-GPL
crates/command_palette_hooks/LICENSE-GPL~merge-20260924
crates/component/Cargo.toml
crates/component/LICENSE-GPL
crates/component/LICENSE-GPL~merge-20260924
crates/component_preview/Cargo.toml
crates/component_preview/LICENSE-GPL
crates/component_preview/LICENSE-GPL~merge-20260924
crates/component_preview/examples/component_preview.rs
crates/component_preview/src/component_preview.rs
crates/context_server/Cargo.toml
crates/context_server/LICENSE-GPL
crates/context_server/LICENSE-GPL~merge-20260924
crates/context_server/src/client.rs
crates/context_server/src/context_server.rs
crates/context_server/src/listener.rs
crates/context_server/src/oauth.rs
crates/context_server/src/protocol.rs
crates/context_server/src/transport.rs
crates/context_server/src/transport/http.rs
crates/copilot/Cargo.toml
crates/copilot/LICENSE-GPL
crates/copilot/LICENSE-GPL~merge-20260924
crates/copilot/src/copilot.rs
crates/copilot/src/copilot_edit_prediction_delegate.rs
crates/copilot_chat/Cargo.toml
crates/copilot_chat/LICENSE-GPL
crates/copilot_chat/LICENSE-GPL~merge-20260924
crates/copilot_chat/src/copilot_chat.rs
crates/copilot_chat/src/responses.rs
crates/copilot_ui/Cargo.toml
crates/copilot_ui/LICENSE-GPL
crates/copilot_ui/LICENSE-GPL~merge-20260924
crates/copilot_ui/src/sign_in.rs
crates/crashes/Cargo.toml
crates/crashes/LICENSE-GPL
crates/crashes/LICENSE-GPL~merge-20260924
crates/crashes/src/crashes.rs
crates/credentials_provider/Cargo.toml
crates/credentials_provider/LICENSE-GPL
crates/credentials_provider/LICENSE-GPL~merge-20260924
crates/dap/Cargo.toml
crates/dap/LICENSE-GPL
crates/dap/LICENSE-GPL~merge-20260924
crates/dap/src/adapters.rs
crates/dap/src/proto_conversions.rs
crates/dap/src/transport.rs
crates/dap_adapters/Cargo.toml
crates/dap_adapters/LICENSE-GPL
crates/dap_adapters/LICENSE-GPL~merge-20260924
crates/db/LICENSE-GPL
crates/db/LICENSE-GPL~merge-20260924
crates/debug_adapter_extension/Cargo.toml
crates/debug_adapter_extension/LICENSE-GPL
crates/debug_adapter_extension/LICENSE-GPL~merge-20260924
crates/debugger_tools/LICENSE-GPL
crates/debugger_tools/LICENSE-GPL~merge-20260924
crates/debugger_ui/Cargo.toml
crates/debugger_ui/LICENSE-GPL
crates/debugger_ui/LICENSE-GPL~merge-20260924
crates/debugger_ui/src/attach_modal.rs
crates/debugger_ui/src/debugger_panel.rs
crates/debugger_ui/src/debugger_ui.rs
crates/debugger_ui/src/dropdown_menus.rs
crates/debugger_ui/src/new_process_modal.rs
crates/debugger_ui/src/persistence.rs
crates/debugger_ui/src/session/running.rs
crates/debugger_ui/src/session/running/breakpoint_list.rs
crates/debugger_ui/src/session/running/console.rs
crates/debugger_ui/src/session/running/memory_view.rs
crates/debugger_ui/src/session/running/module_list.rs
crates/debugger_ui/src/session/running/stack_frame_list.rs
crates/debugger_ui/src/session/running/variable_list.rs
crates/debugger_ui/src/tests/debugger_panel.rs
crates/debugger_ui/src/tests/inline_values.rs
crates/debugger_ui/src/tests/stack_frame_list.rs
crates/debugger_ui/src/tests/variable_list.rs
crates/deepseek/Cargo.toml
crates/deepseek/LICENSE-GPL
crates/deepseek/LICENSE-GPL~merge-20260924
crates/deepseek/src/deepseek.rs
crates/dev_container/Cargo.toml
crates/dev_container/LICENSE-GPL
crates/dev_container/LICENSE-GPL~merge-20260924
crates/dev_container/src/devcontainer_api.rs
crates/dev_container/src/devcontainer_json.rs
crates/dev_container/src/devcontainer_manifest.rs
crates/dev_container/src/docker.rs
crates/dev_container/src/features.rs
crates/dev_container/src/lib.rs
crates/diagnostics/Cargo.toml
crates/diagnostics/LICENSE-GPL
crates/diagnostics/LICENSE-GPL~merge-20260924
crates/diagnostics/src/buffer_diagnostics.rs
crates/diagnostics/src/diagnostic_renderer.rs
crates/diagnostics/src/diagnostics.rs
crates/diagnostics/src/diagnostics_tests.rs
crates/diagnostics/src/items.rs
crates/docs_preprocessor/Cargo.toml
crates/docs_preprocessor/LICENSE-GPL
crates/docs_preprocessor/LICENSE-GPL~merge-20260924
crates/docs_preprocessor/src/main.rs
crates/edit_prediction/Cargo.toml
crates/edit_prediction/LICENSE-GPL
crates/edit_prediction/LICENSE-GPL~merge-20260924
crates/edit_prediction/src/cursor_excerpt.rs
crates/edit_prediction/src/data_collection.rs
crates/edit_prediction/src/edit_prediction.rs
crates/edit_prediction/src/edit_prediction_tests.rs
crates/edit_prediction/src/example_spec.rs
crates/edit_prediction/src/fim.rs
crates/edit_prediction/src/mercury.rs
crates/edit_prediction/src/ollama.rs
crates/edit_prediction/src/prediction.rs
crates/edit_prediction/src/udiff.rs
crates/edit_prediction/src/zed_edit_prediction_delegate.rs
crates/edit_prediction/src/zeta.rs
crates/edit_prediction_cli/Cargo.toml
crates/edit_prediction_cli/LICENSE-GPL
crates/edit_prediction_cli/LICENSE-GPL~merge-20260924
crates/edit_prediction_cli/src/anthropic_client.rs
crates/edit_prediction_cli/src/example.rs
crates/edit_prediction_cli/src/filter_languages.rs
crates/edit_prediction_cli/src/format_prompt.rs
crates/edit_prediction_cli/src/headless.rs
crates/edit_prediction_cli/src/load_project.rs
crates/edit_prediction_cli/src/main.rs
crates/edit_prediction_cli/src/openai_client.rs
crates/edit_prediction_cli/src/parse_output.rs
crates/edit_prediction_cli/src/predict.rs
crates/edit_prediction_cli/src/progress.rs
crates/edit_prediction_cli/src/prompt_assets.rs
crates/edit_prediction_cli/src/pull_examples.rs
crates/edit_prediction_cli/src/qa.rs
crates/edit_prediction_cli/src/reorder_patch.rs
crates/edit_prediction_cli/src/repair.rs
crates/edit_prediction_cli/src/retrieve_context.rs
crates/edit_prediction_cli/src/score.rs
crates/edit_prediction_context/Cargo.toml
crates/edit_prediction_context/LICENSE-GPL
crates/edit_prediction_context/LICENSE-GPL~merge-20260924
crates/edit_prediction_context/src/edit_prediction_context.rs
crates/edit_prediction_context/src/edit_prediction_context_tests.rs
crates/edit_prediction_context/src/editable_context.rs
crates/edit_prediction_context/src/fake_definition_lsp.rs
crates/edit_prediction_metrics/Cargo.toml
crates/edit_prediction_metrics/LICENSE-GPL
crates/edit_prediction_metrics/LICENSE-GPL~merge-20260924
crates/edit_prediction_metrics/src/edit_prediction_metrics.rs
crates/edit_prediction_metrics/src/jumps.rs
crates/edit_prediction_metrics/src/prediction_score.rs
crates/edit_prediction_metrics/src/reversal.rs
crates/edit_prediction_metrics/src/summary.rs
crates/edit_prediction_types/Cargo.toml
crates/edit_prediction_types/LICENSE-GPL
crates/edit_prediction_types/LICENSE-GPL~merge-20260924
crates/edit_prediction_types/src/edit_prediction_types.rs
crates/edit_prediction_ui/Cargo.toml
crates/edit_prediction_ui/LICENSE-GPL
crates/edit_prediction_ui/LICENSE-GPL~merge-20260924
crates/edit_prediction_ui/src/edit_prediction_button.rs
crates/edit_prediction_ui/src/rate_prediction_modal.rs
crates/editor/Cargo.toml
crates/editor/LICENSE-GPL
crates/editor/LICENSE-GPL~merge-20260924
crates/editor/src/actions.rs
crates/editor/src/blink_manager.rs
crates/editor/src/bookmarks.rs
crates/editor/src/bracket_colorization.rs
crates/editor/src/clangd_ext.rs
crates/editor/src/clipboard.rs
crates/editor/src/code_actions.rs
crates/editor/src/code_completion_tests.rs
crates/editor/src/code_context_menus.rs
crates/editor/src/code_lens.rs
crates/editor/src/completions.rs
crates/editor/src/config.rs
crates/editor/src/diagnostics.rs
crates/editor/src/display_map.rs
crates/editor/src/display_map/block_map.rs
crates/editor/src/display_map/custom_highlights.rs
crates/editor/src/display_map/fold_map.rs
crates/editor/src/display_map/inlay_map.rs
crates/editor/src/display_map/invisibles.rs
crates/editor/src/display_map/tab_map.rs
crates/editor/src/display_map/wrap_map.rs
crates/editor/src/document_colors.rs
crates/editor/src/document_symbols.rs
crates/editor/src/edit_prediction.rs
crates/editor/src/edit_prediction_tests.rs
crates/editor/src/editor.rs
crates/editor/src/editor_settings.rs
crates/editor/src/editor_tests.rs
crates/editor/src/element.rs
crates/editor/src/element/header.rs
crates/editor/src/element/mouse.rs
crates/editor/src/fold.rs
crates/editor/src/git.rs
crates/editor/src/git/blame.rs
crates/editor/src/highlight_matching_bracket.rs
crates/editor/src/hover_links.rs
crates/editor/src/hover_popover.rs
crates/editor/src/inlays.rs
crates/editor/src/inlays/inlay_hints.rs
crates/editor/src/input.rs
crates/editor/src/items.rs
crates/editor/src/jsx_tag_auto_close.rs
crates/editor/src/linked_editing_ranges.rs
crates/editor/src/lsp_ext.rs
crates/editor/src/mouse_context_menu.rs
crates/editor/src/movement.rs
crates/editor/src/navigation.rs
crates/editor/src/rewrap.rs
crates/editor/src/runnables.rs
crates/editor/src/rust_analyzer_ext.rs
crates/editor/src/scroll.rs
crates/editor/src/scroll/actions.rs
crates/editor/src/scroll/autoscroll.rs
crates/editor/src/selection.rs
crates/editor/src/selections_collection.rs
crates/editor/src/semantic_tokens.rs
crates/editor/src/signature_help.rs
crates/editor/src/split.rs
crates/editor/src/test.rs
crates/editor/src/test/editor_lsp_test_context.rs
crates/editor/src/test/editor_test_context.rs
crates/editor_benchmarks/LICENSE-GPL
crates/editor_benchmarks/LICENSE-GPL~merge-20260924
crates/encoding_selector/Cargo.toml
crates/encoding_selector/LICENSE-GPL
crates/encoding_selector/LICENSE-GPL~merge-20260924
crates/encoding_selector/src/active_buffer_encoding.rs
crates/encoding_selector/src/encoding_selector.rs
crates/env_var/Cargo.toml
crates/env_var/LICENSE-GPL
crates/env_var/LICENSE-GPL~merge-20260924
crates/etw_tracing/Cargo.toml
crates/etw_tracing/LICENSE-GPL
crates/etw_tracing/LICENSE-GPL~merge-20260924
crates/etw_tracing/etw_tracing.rs
crates/eval_cli/.gitignore
crates/eval_cli/Dockerfile
crates/eval_cli/LICENSE-GPL
crates/eval_cli/LICENSE-GPL~merge-20260924
crates/eval_cli/README.md
crates/eval_cli/script/build-linux
crates/eval_cli/src/headless.rs
crates/eval_cli/src/main.rs
crates/eval_cli/zed_eval/__init__.py
crates/eval_cli/zed_eval/agent.py
crates/eval_cli/zed_eval/pyproject.toml
crates/eval_utils/Cargo.toml
crates/eval_utils/LICENSE-GPL
crates/eval_utils/LICENSE-GPL~merge-20260924
crates/explorer_command_injector/Cargo.toml
crates/explorer_command_injector/LICENSE-GPL
crates/explorer_command_injector/LICENSE-GPL~merge-20260924
crates/explorer_command_injector/src/explorer_command_injector.rs
crates/extension/Cargo.toml
crates/extension/LICENSE-GPL
crates/extension/LICENSE-GPL~merge-20260924
crates/extension/src/extension.rs
crates/extension/src/extension_builder.rs
crates/extension/src/extension_host_proxy.rs
crates/extension/src/extension_manifest.rs
crates/extension_api/Cargo.toml
crates/extension_api/LICENSE-APACHE
crates/extension_api/LICENSE-APACHE~merge-20260924
crates/extension_api/wit/since_v0.8.0/settings.rs
crates/extension_cli/Cargo.toml
crates/extension_cli/LICENSE-GPL
crates/extension_cli/LICENSE-GPL~merge-20260924
crates/extension_cli/src/main.rs
crates/extension_host/Cargo.toml
crates/extension_host/LICENSE-GPL
crates/extension_host/LICENSE-GPL~merge-20260924
crates/extension_host/benches/extension_compilation_benchmark.rs
crates/extension_host/src/capability_granter.rs
crates/extension_host/src/extension_host.rs
crates/extension_host/src/extension_store_test.rs
crates/extension_host/src/headless_host.rs
crates/extension_host/src/wasm_host.rs
crates/extension_host/src/wasm_host/wit.rs
crates/extension_host/src/wasm_host/wit/since_v0_0_1.rs
crates/extension_host/src/wasm_host/wit/since_v0_0_4.rs
crates/extension_host/src/wasm_host/wit/since_v0_0_6.rs
crates/extension_host/src/wasm_host/wit/since_v0_1_0.rs
crates/extension_host/src/wasm_host/wit/since_v0_2_0.rs
crates/extension_host/src/wasm_host/wit/since_v0_3_0.rs
crates/extension_host/src/wasm_host/wit/since_v0_4_0.rs
crates/extension_host/src/wasm_host/wit/since_v0_5_0.rs
crates/extension_host/src/wasm_host/wit/since_v0_6_0.rs
crates/extension_host/src/wasm_host/wit/since_v0_8_0.rs
crates/extensions_ui/Cargo.toml
crates/extensions_ui/LICENSE-GPL
crates/extensions_ui/LICENSE-GPL~merge-20260924
crates/extensions_ui/src/components.rs
crates/extensions_ui/src/components/extension_card.rs
crates/extensions_ui/src/extension_version_selector.rs
crates/extensions_ui/src/extensions_ui.rs
crates/feature_flags/Cargo.toml
crates/feature_flags/LICENSE-GPL
crates/feature_flags/LICENSE-GPL~merge-20260924
crates/feature_flags/src/flags.rs
crates/feature_flags_macros/Cargo.toml
crates/feature_flags_macros/LICENSE-GPL
crates/feature_flags_macros/LICENSE-GPL~merge-20260924
crates/feedback/Cargo.toml
crates/feedback/LICENSE-GPL
crates/feedback/LICENSE-GPL~merge-20260924
crates/feedback/src/feedback.rs
crates/file_finder/Cargo.toml
crates/file_finder/LICENSE-GPL
crates/file_finder/LICENSE-GPL~merge-20260924
crates/file_finder/src/file_finder.rs
crates/file_finder/src/file_finder_tests.rs
crates/file_icons/Cargo.toml
crates/file_icons/LICENSE-GPL
crates/file_icons/LICENSE-GPL~merge-20260924
crates/file_icons/src/file_icons.rs
crates/fs/Cargo.toml
crates/fs/LICENSE-GPL
crates/fs/LICENSE-GPL~merge-20260924
crates/fs/src/fake_git_repo.rs
crates/fs/src/fs.rs
crates/fs/src/fs_watcher.rs
crates/fs/tests/integration/fs_tests.rs
crates/fs_benchmarks/LICENSE-GPL
crates/fs_benchmarks/LICENSE-GPL~merge-20260924
crates/fuzzy/Cargo.toml
crates/fuzzy/LICENSE-GPL
crates/fuzzy/LICENSE-GPL~merge-20260924
crates/fuzzy/src/matcher.rs
crates/fuzzy/src/paths.rs
crates/fuzzy/src/strings.rs
crates/fuzzy_nucleo/Cargo.toml
crates/fuzzy_nucleo/LICENSE-GPL
crates/fuzzy_nucleo/LICENSE-GPL~merge-20260924
crates/fuzzy_nucleo/benches/match_benchmark.rs
crates/fuzzy_nucleo/src/paths.rs
crates/fuzzy_nucleo/src/strings.rs
crates/git/Cargo.toml
crates/git/LICENSE-GPL
crates/git/LICENSE-GPL~merge-20260924
crates/git/src/blame.rs
crates/git/src/commit.rs
crates/git/src/git.rs
crates/git/src/remote.rs
crates/git/src/repository.rs
crates/git/src/status.rs
crates/git/test_data/golden/blame_incremental_complex.json
crates/git/test_data/golden/blame_incremental_not_committed.json
crates/git/test_data/golden/blame_incremental_simple.json
crates/git_hosting_providers/Cargo.toml
crates/git_hosting_providers/LICENSE-GPL
crates/git_hosting_providers/LICENSE-GPL~merge-20260924
crates/git_hosting_providers/src/git_hosting_providers.rs
crates/git_hosting_providers/src/providers.rs
crates/git_hosting_providers/src/providers/chromium.rs
crates/git_hosting_providers/src/settings.rs
crates/git_ui/Cargo.toml
crates/git_ui/LICENSE-GPL
crates/git_ui/LICENSE-GPL~merge-20260924
crates/git_ui/src/blame_ui.rs
crates/git_ui/src/branch_picker.rs
crates/git_ui/src/commit_modal.rs
crates/git_ui/src/commit_tooltip.rs
crates/git_ui/src/commit_view.rs
crates/git_ui/src/conflict_view.rs
crates/git_ui/src/git_graph.rs
crates/git_ui/src/git_panel.rs
crates/git_ui/src/git_panel_settings.rs
crates/git_ui/src/git_picker.rs
crates/git_ui/src/git_ui.rs
crates/git_ui/src/multi_diff_view.rs
crates/git_ui/src/picker_prompt.rs
crates/git_ui/src/project_diff.rs
crates/git_ui/src/remote_output.rs
crates/git_ui/src/repository_selector.rs
crates/git_ui/src/solo_diff_view.rs
crates/git_ui/src/stash_picker.rs
crates/git_ui/src/text_diff_view.rs
crates/go_to_line/Cargo.toml
crates/go_to_line/LICENSE-GPL
crates/go_to_line/LICENSE-GPL~merge-20260924
crates/go_to_line/src/cursor_position.rs
crates/go_to_line/src/go_to_line.rs
crates/google_ai/Cargo.toml
crates/google_ai/LICENSE-GPL
crates/google_ai/LICENSE-GPL~merge-20260924
crates/google_ai/src/completion.rs
crates/google_ai/src/google_ai.rs
crates/gpui/Cargo.toml
crates/gpui/LICENSE-APACHE
crates/gpui/LICENSE-APACHE~merge-20260924
crates/gpui/build.rs
crates/gpui/examples/README.md
crates/gpui/examples/a11y.rs
crates/gpui/examples/anchor.rs
crates/gpui/examples/animation.rs
crates/gpui/examples/data_table.rs
crates/gpui/examples/drag_drop.rs
crates/gpui/examples/focus_visible.rs
crates/gpui/examples/gif_viewer.rs
crates/gpui/examples/gradient.rs
crates/gpui/examples/grid_layout.rs
crates/gpui/examples/hello_world.rs
crates/gpui/examples/image/image.rs
crates/gpui/examples/image_gallery.rs
crates/gpui/examples/image_loading.rs
crates/gpui/examples/input.rs
crates/gpui/examples/list_example.rs
crates/gpui/examples/mouse_pressure.rs
crates/gpui/examples/move_entity_between_windows.rs
crates/gpui/examples/on_window_close_quit.rs
crates/gpui/examples/opacity.rs
crates/gpui/examples/ownership_post.rs
crates/gpui/examples/painting.rs
crates/gpui/examples/paths_bench.rs
crates/gpui/examples/pattern.rs
crates/gpui/examples/popover.rs
crates/gpui/examples/scrollable.rs
crates/gpui/examples/set_menus.rs
crates/gpui/examples/shadow.rs
crates/gpui/examples/svg/svg.rs
crates/gpui/examples/tab_stop.rs
crates/gpui/examples/testing.rs
crates/gpui/examples/text.rs
crates/gpui/examples/text_layout.rs
crates/gpui/examples/text_wrapper.rs
crates/gpui/examples/tree.rs
crates/gpui/examples/uniform_list.rs
crates/gpui/examples/window.rs
crates/gpui/examples/window_positioning.rs
crates/gpui/examples/window_shadow.rs
crates/gpui/resources/windows/gpui.rc
crates/gpui/src/_accessibility.rs
crates/gpui/src/app.rs
crates/gpui/src/app/async_context.rs
crates/gpui/src/app/bench_context.rs
crates/gpui/src/app/context.rs
crates/gpui/src/app/entity_map.rs
crates/gpui/src/app/test_app.rs
crates/gpui/src/app/test_context.rs
crates/gpui/src/arena.rs
crates/gpui/src/asset_cache.rs
crates/gpui/src/color.rs
crates/gpui/src/element.rs
crates/gpui/src/elements/animation.rs
crates/gpui/src/elements/deferred.rs
crates/gpui/src/elements/div.rs
crates/gpui/src/elements/image_cache.rs
crates/gpui/src/elements/img.rs
crates/gpui/src/elements/list.rs
crates/gpui/src/elements/mod.rs
crates/gpui/src/elements/svg.rs
crates/gpui/src/elements/text.rs
crates/gpui/src/elements/uniform_list.rs
crates/gpui/src/executor.rs
crates/gpui/src/geometry.rs
crates/gpui/src/gpui.rs
crates/gpui/src/input.rs
crates/gpui/src/inspector.rs
crates/gpui/src/interactive.rs
crates/gpui/src/key_dispatch.rs
crates/gpui/src/keymap.rs
crates/gpui/src/platform.rs
crates/gpui/src/platform/test.rs
crates/gpui/src/platform/test/platform.rs
crates/gpui/src/platform/test/window.rs
crates/gpui/src/platform/visual_test.rs
crates/gpui/src/platform_scheduler.rs
crates/gpui/src/profiler.rs
crates/gpui/src/profiler/actions.rs
crates/gpui/src/queue.rs
crates/gpui/src/scene.rs
crates/gpui/src/style.rs
crates/gpui/src/styled.rs
crates/gpui/src/svg_renderer.rs
crates/gpui/src/tab_stop.rs
crates/gpui/src/taffy.rs
crates/gpui/src/text_system.rs
crates/gpui/src/text_system/line.rs
crates/gpui/src/text_system/line_layout.rs
crates/gpui/src/text_system/line_wrapper.rs
crates/gpui/src/view.rs
crates/gpui/src/window.rs
crates/gpui/src/window/a11y.rs
crates/gpui_linux/Cargo.toml
crates/gpui_linux/LICENSE-APACHE
crates/gpui_linux/LICENSE-APACHE~merge-20260924
crates/gpui_linux/src/gpui_linux.rs
crates/gpui_linux/src/linux.rs
crates/gpui_linux/src/linux/dispatcher.rs
crates/gpui_linux/src/linux/headless.rs
crates/gpui_linux/src/linux/headless/client.rs
crates/gpui_linux/src/linux/platform.rs
crates/gpui_linux/src/linux/wayland.rs
crates/gpui_linux/src/linux/wayland/client.rs
crates/gpui_linux/src/linux/wayland/clipboard.rs
crates/gpui_linux/src/linux/wayland/cursor.rs
crates/gpui_linux/src/linux/wayland/serial.rs
crates/gpui_linux/src/linux/wayland/window.rs
crates/gpui_linux/src/linux/x11/client.rs
crates/gpui_linux/src/linux/x11/clipboard.rs
crates/gpui_linux/src/linux/x11/window.rs
crates/gpui_macos/Cargo.toml
crates/gpui_macos/LICENSE-APACHE
crates/gpui_macos/LICENSE-APACHE~merge-20260924
crates/gpui_macos/src/dispatcher.rs
crates/gpui_macos/src/display_link.rs
crates/gpui_macos/src/gpui_macos.rs
crates/gpui_macos/src/pasteboard.rs
crates/gpui_macos/src/platform.rs
crates/gpui_macos/src/screen_capture.rs
crates/gpui_macos/src/text_system.rs
crates/gpui_macos/src/window.rs
crates/gpui_macros/Cargo.toml
crates/gpui_macros/LICENSE-APACHE
crates/gpui_macros/LICENSE-APACHE~merge-20260924
crates/gpui_macros/src/bench.rs
crates/gpui_macros/src/derive_inspector_reflection.rs
crates/gpui_macros/src/derive_into_element.rs
crates/gpui_macros/src/gpui_macros.rs
crates/gpui_macros/tests/derive_inspector_reflection.rs
crates/gpui_platform/Cargo.toml
crates/gpui_platform/LICENSE-APACHE
crates/gpui_platform/LICENSE-APACHE~merge-20260924
crates/gpui_platform/src/gpui_platform.rs
crates/gpui_shared_string/Cargo.toml
crates/gpui_shared_string/LICENSE-APACHE
crates/gpui_shared_string/LICENSE-APACHE~merge-20260924
crates/gpui_shared_string/gpui_shared_string.rs
crates/gpui_tokio/Cargo.toml
crates/gpui_tokio/LICENSE-APACHE
crates/gpui_tokio/LICENSE-APACHE~merge-20260924
crates/gpui_tokio/src/gpui_tokio.rs
crates/gpui_util/Cargo.toml
crates/gpui_util/LICENSE-APACHE
crates/gpui_util/LICENSE-APACHE~merge-20260924
crates/gpui_util/src/lib.rs
crates/gpui_web/Cargo.toml
crates/gpui_web/LICENSE-APACHE
crates/gpui_web/LICENSE-APACHE~merge-20260924
crates/gpui_web/examples/hello_web/Cargo.toml
crates/gpui_web/examples/hello_web/LICENSE-APACHE
crates/gpui_web/examples/hello_web/LICENSE-APACHE~merge-20260924
crates/gpui_web/examples/hello_web/index.html
crates/gpui_web/examples/hello_web/main.rs
crates/gpui_web/examples/hello_web/trunk.toml
crates/gpui_web/src/dispatcher.rs
crates/gpui_web/src/display.rs
crates/gpui_web/src/events.rs
crates/gpui_web/src/gpui_web.rs
crates/gpui_web/src/http_client.rs
crates/gpui_web/src/keyboard.rs
crates/gpui_web/src/logging.rs
crates/gpui_web/src/platform.rs
crates/gpui_web/src/window.rs
crates/gpui_wgpu/Cargo.toml
crates/gpui_wgpu/LICENSE-APACHE
crates/gpui_wgpu/LICENSE-APACHE~merge-20260924
crates/gpui_wgpu/benches/layout_line.rs
crates/gpui_wgpu/src/cosmic_text_system.rs
crates/gpui_wgpu/src/shaders.wgsl
crates/gpui_wgpu/src/wgpu_atlas.rs
crates/gpui_wgpu/src/wgpu_context.rs
crates/gpui_wgpu/src/wgpu_renderer.rs
crates/gpui_windows/Cargo.toml
crates/gpui_windows/LICENSE-APACHE
crates/gpui_windows/LICENSE-APACHE~merge-20260924
crates/gpui_windows/build.rs
crates/gpui_windows/src/direct_manipulation.rs
crates/gpui_windows/src/direct_write.rs
crates/gpui_windows/src/directx_atlas.rs
crates/gpui_windows/src/directx_devices.rs
crates/gpui_windows/src/directx_renderer.rs
crates/gpui_windows/src/dispatcher.rs
crates/gpui_windows/src/display.rs
crates/gpui_windows/src/events.rs
crates/gpui_windows/src/gpui_windows.rs
crates/gpui_windows/src/platform.rs
crates/gpui_windows/src/shaders.hlsl
crates/gpui_windows/src/system_settings.rs
crates/gpui_windows/src/util.rs
crates/gpui_windows/src/vsync.rs
crates/gpui_windows/src/window.rs
crates/grammars/Cargo.toml
crates/grammars/LICENSE-GPL
crates/grammars/LICENSE-GPL~merge-20260924
crates/grammars/src/bash/config.toml
crates/grammars/src/c/config.toml
crates/grammars/src/c/highlights.scm
crates/grammars/src/c/indents.scm
crates/grammars/src/cpp/config.toml
crates/grammars/src/cpp/indents.scm
crates/grammars/src/go/highlights.scm
crates/grammars/src/go/outline.scm
crates/grammars/src/grammars.rs
crates/grammars/src/javascript/config.toml
crates/grammars/src/javascript/highlights.scm
crates/grammars/src/javascript/injections.scm
crates/grammars/src/javascript/outline.scm
crates/grammars/src/javascript/overrides.scm
crates/grammars/src/javascript/runnables.scm
crates/grammars/src/json/config.toml
crates/grammars/src/markdown-inline/injections.scm
crates/grammars/src/python/highlights.scm
crates/grammars/src/python/injections.scm
crates/grammars/src/python/runnables.scm
crates/grammars/src/python/semantic_token_rules.json
crates/grammars/src/rust/config.toml
crates/grammars/src/tsx/config.toml
crates/grammars/src/tsx/highlights.scm
crates/grammars/src/tsx/indents.scm
crates/grammars/src/tsx/outline.scm
crates/grammars/src/tsx/overrides.scm
crates/grammars/src/tsx/runnables.scm
crates/grammars/src/typescript/highlights.scm
crates/grammars/src/typescript/indents.scm
crates/grammars/src/typescript/injections.scm
crates/grammars/src/typescript/outline.scm
crates/grammars/src/typescript/runnables.scm
crates/html_to_markdown/Cargo.toml
crates/html_to_markdown/LICENSE-APACHE
crates/html_to_markdown/LICENSE-APACHE~merge-20260924
crates/html_to_markdown/src/html_to_markdown.rs
crates/http_client/Cargo.toml
crates/http_client/LICENSE-APACHE
crates/http_client/LICENSE-APACHE~merge-20260924
crates/http_client/src/github.rs
crates/http_client/src/github_download.rs
crates/http_client/src/http_client.rs
crates/http_client_tls/Cargo.toml
crates/http_client_tls/LICENSE-APACHE
crates/http_client_tls/LICENSE-APACHE~merge-20260924
crates/http_client_tls/src/http_client_tls.rs
crates/http_proxy/Cargo.toml
crates/http_proxy/LICENSE-GPL
crates/http_proxy/LICENSE-GPL~merge-20260924
crates/http_proxy/src/allowlist.rs
crates/http_proxy/src/http_proxy.rs
crates/icons/Cargo.toml
crates/icons/LICENSE-GPL
crates/icons/LICENSE-GPL~merge-20260924
crates/icons/src/icons.rs
crates/image_viewer/Cargo.toml
crates/image_viewer/LICENSE-GPL
crates/image_viewer/LICENSE-GPL~merge-20260924
crates/image_viewer/src/image_viewer.rs
crates/input_latency_ui/Cargo.toml
crates/input_latency_ui/LICENSE-GPL
crates/input_latency_ui/LICENSE-GPL~merge-20260924
crates/input_latency_ui/src/input_latency_ui.rs
crates/inspector_ui/Cargo.toml
crates/inspector_ui/LICENSE-GPL
crates/inspector_ui/LICENSE-GPL~merge-20260924
crates/inspector_ui/src/div_inspector.rs
crates/inspector_ui/src/inspector.rs
crates/install_cli/Cargo.toml
crates/install_cli/LICENSE-GPL
crates/install_cli/LICENSE-GPL~merge-20260924
crates/install_cli/src/install_cli_binary.rs
crates/journal/Cargo.toml
crates/journal/LICENSE-GPL
crates/journal/LICENSE-GPL~merge-20260924
crates/json_schema_store/Cargo.toml
crates/json_schema_store/LICENSE-GPL
crates/json_schema_store/LICENSE-GPL~merge-20260924
crates/json_schema_store/src/json_schema_store.rs
crates/json_schema_store/src/schemas/package.json
crates/json_schema_store/src/schemas/tsconfig.json
crates/keymap_editor/Cargo.toml
crates/keymap_editor/LICENSE-GPL
crates/keymap_editor/LICENSE-GPL~merge-20260924
crates/keymap_editor/src/action_completion_provider.rs
crates/keymap_editor/src/keymap_editor.rs
crates/keymap_editor/src/ui_components/keystroke_input.rs
crates/language/Cargo.toml
crates/language/LICENSE-GPL
crates/language/LICENSE-GPL~merge-20260924
crates/language/src/buffer.rs
crates/language/src/buffer/row_chunk.rs
crates/language/src/buffer_tests.rs
crates/language/src/diagnostic.rs
crates/language/src/diagnostic_set.rs
crates/language/src/file_content.rs
crates/language/src/language.rs
crates/language/src/language_registry.rs
crates/language/src/language_settings.rs
crates/language/src/outline.rs
crates/language/src/proto.rs
crates/language/src/runnable.rs
crates/language/src/syntax_map.rs
crates/language/src/syntax_map/syntax_map_tests.rs
crates/language/src/text_diff.rs
crates/language_core/Cargo.toml
crates/language_core/LICENSE-GPL
crates/language_core/LICENSE-GPL~merge-20260924
crates/language_core/src/code_label.rs
crates/language_core/src/grammar.rs
crates/language_core/src/highlight_map.rs
crates/language_core/src/language_config.rs
crates/language_core/src/language_core.rs
crates/language_core/src/language_name.rs
crates/language_core/src/lsp_adapter.rs
crates/language_core/src/queries.rs
crates/language_core/src/toolchain.rs
crates/language_extension/Cargo.toml
crates/language_extension/LICENSE-GPL
crates/language_extension/LICENSE-GPL~merge-20260924
crates/language_extension/src/extension_lsp_adapter.rs
crates/language_extension/src/language_extension.rs
crates/language_model/Cargo.toml
crates/language_model/LICENSE-GPL
crates/language_model/LICENSE-GPL~merge-20260924
crates/language_model/src/api_key.rs
crates/language_model/src/fake_provider.rs
crates/language_model/src/language_model.rs
crates/language_model/src/registry.rs
crates/language_model/src/request.rs
crates/language_model_core/Cargo.toml
crates/language_model_core/LICENSE-GPL
crates/language_model_core/LICENSE-GPL~merge-20260924
crates/language_model_core/src/language_model_core.rs
crates/language_model_core/src/provider.rs
crates/language_model_core/src/request.rs
crates/language_model_core/src/tool_schema.rs
crates/language_model_core/src/util.rs
crates/language_models/Cargo.toml
crates/language_models/LICENSE-GPL
crates/language_models/LICENSE-GPL~merge-20260924
crates/language_models/src/language_models.rs
crates/language_models/src/provider.rs
crates/language_models/src/provider/anthropic.rs
crates/language_models/src/provider/anthropic_compatible.rs
crates/language_models/src/provider/api_compatible.rs
crates/language_models/src/provider/bedrock.rs
crates/language_models/src/provider/cloud.rs
crates/language_models/src/provider/copilot_chat.rs
crates/language_models/src/provider/deepseek.rs
crates/language_models/src/provider/google.rs
crates/language_models/src/provider/llama_cpp.rs
crates/language_models/src/provider/lmstudio.rs
crates/language_models/src/provider/mistral.rs
crates/language_models/src/provider/ollama.rs
crates/language_models/src/provider/open_ai.rs
crates/language_models/src/provider/open_ai_compatible.rs
crates/language_models/src/provider/open_router.rs
crates/language_models/src/provider/openai_subscribed.rs
crates/language_models/src/provider/opencode.rs
crates/language_models/src/provider/vercel_ai_gateway.rs
crates/language_models/src/provider/x_ai.rs
crates/language_models/src/settings.rs
crates/language_models_cloud/Cargo.toml
crates/language_models_cloud/LICENSE-GPL
crates/language_models_cloud/LICENSE-GPL~merge-20260924
crates/language_models_cloud/src/language_models_cloud.rs
crates/language_onboarding/Cargo.toml
crates/language_onboarding/LICENSE-GPL
crates/language_onboarding/LICENSE-GPL~merge-20260924
crates/language_selector/LICENSE-GPL
crates/language_selector/LICENSE-GPL~merge-20260924
crates/language_selector/src/active_buffer_language.rs
crates/language_selector/src/language_selector.rs
crates/language_tools/Cargo.toml
crates/language_tools/LICENSE-GPL
crates/language_tools/LICENSE-GPL~merge-20260924
crates/language_tools/src/highlights_tree_view.rs
crates/language_tools/src/key_context_view.rs
crates/language_tools/src/lsp_button.rs
crates/language_tools/src/lsp_log_view.rs
crates/language_tools/src/lsp_log_view_tests.rs
crates/languages/Cargo.toml
crates/languages/LICENSE-GPL
crates/languages/LICENSE-GPL~merge-20260924
crates/languages/src/bash.rs
crates/languages/src/c.rs
crates/languages/src/cpp.rs
crates/languages/src/css.rs
crates/languages/src/eslint.rs
crates/languages/src/go.rs
crates/languages/src/json.rs
crates/languages/src/lib.rs
crates/languages/src/python.rs
crates/languages/src/rust.rs
crates/languages/src/typescript.rs
crates/languages/src/vtsls.rs
crates/line_ending_selector/Cargo.toml
crates/line_ending_selector/LICENSE-GPL
crates/line_ending_selector/LICENSE-GPL~merge-20260924
crates/line_ending_selector/src/line_ending_indicator.rs
crates/line_ending_selector/src/line_ending_selector.rs
crates/livekit_api/Cargo.toml
crates/livekit_api/LICENSE-GPL
crates/livekit_api/LICENSE-GPL~merge-20260924
crates/livekit_api/build.rs
crates/livekit_api/src/livekit_api.rs
crates/livekit_api/src/token.rs
crates/livekit_client/Cargo.toml
crates/livekit_client/LICENSE-GPL
crates/livekit_client/LICENSE-GPL~merge-20260924
crates/livekit_client/examples/test_app.rs
crates/livekit_client/src/lib.rs
crates/livekit_client/src/livekit_client.rs
crates/livekit_client/src/livekit_client/linux.rs
crates/livekit_client/src/livekit_client/playback.rs
crates/livekit_client/src/mock_client.rs
crates/livekit_client/src/mock_client/track.rs
crates/livekit_client/src/test.rs
crates/lmstudio/Cargo.toml
crates/lmstudio/LICENSE-GPL
crates/lmstudio/LICENSE-GPL~merge-20260924
crates/lmstudio/src/lmstudio.rs
crates/lsp/Cargo.toml
crates/lsp/LICENSE-GPL
crates/lsp/LICENSE-GPL~merge-20260924
crates/lsp/src/input_handler.rs
crates/lsp/src/lsp.rs
crates/markdown/Cargo.toml
crates/markdown/LICENSE-GPL
crates/markdown/LICENSE-GPL~merge-20260924
crates/markdown/src/html.rs
crates/markdown/src/html/html_minifier.rs
crates/markdown/src/html/html_parser.rs
crates/markdown/src/html/html_rendering.rs
crates/markdown/src/markdown.rs
crates/markdown/src/mermaid.rs
crates/markdown/src/parser.rs
crates/markdown_preview/Cargo.toml
crates/markdown_preview/LICENSE-GPL
crates/markdown_preview/LICENSE-GPL~merge-20260924
crates/markdown_preview/src/markdown_preview.rs
crates/markdown_preview/src/markdown_preview_view.rs
crates/menu/Cargo.toml
crates/menu/LICENSE-GPL
crates/menu/LICENSE-GPL~merge-20260924
crates/mermaid_render/Cargo.toml
crates/mermaid_render/LICENSE-GPL
crates/mermaid_render/LICENSE-GPL~merge-20260924
crates/mermaid_render/src/mermaid_render.rs
crates/mermaid_render/src/postprocess.rs
crates/mermaid_render/src/postprocess/accent_colors.rs
crates/mermaid_render/src/postprocess/accent_colors/class_diagram.rs
crates/mermaid_render/src/postprocess/accent_colors/mindmap.rs
crates/mermaid_render/src/postprocess/accent_colors/sequence_diagram.rs
crates/mermaid_render/src/postprocess/element_fixup.rs
crates/mermaid_render/src/postprocess/inject_css.rs
crates/mermaid_render/src/postprocess/strip_foreignobject.rs
crates/mermaid_render/src/render.rs
crates/mermaid_render/tests/check_invalid_attrs.rs
crates/migrator/Cargo.toml
crates/migrator/LICENSE-GPL
crates/migrator/LICENSE-GPL~merge-20260924
crates/migrator/src/migrations.rs
crates/migrator/src/migrations/m_2025_06_27/settings.rs
crates/migrator/src/migrations/m_2025_10_02/settings.rs
crates/migrator/src/migrator.rs
crates/miniprofiler_ui/Cargo.toml
crates/miniprofiler_ui/LICENSE-GPL
crates/miniprofiler_ui/LICENSE-GPL~merge-20260924
crates/miniprofiler_ui/src/miniprofiler_ui.rs
crates/mistral/Cargo.toml
crates/mistral/LICENSE-GPL
crates/mistral/LICENSE-GPL~merge-20260924
crates/mistral/src/mistral.rs
crates/multi_buffer/Cargo.toml
crates/multi_buffer/LICENSE-GPL
crates/multi_buffer/LICENSE-GPL~merge-20260924
crates/multi_buffer/src/anchor.rs
crates/multi_buffer/src/multi_buffer.rs
crates/multi_buffer/src/multi_buffer_tests.rs
crates/multi_buffer/src/path_key.rs
crates/net/Cargo.toml
crates/net/LICENSE-GPL
crates/net/LICENSE-GPL~merge-20260924
crates/node_runtime/Cargo.toml
crates/node_runtime/LICENSE-GPL
crates/node_runtime/LICENSE-GPL~merge-20260924
crates/node_runtime/src/node_runtime.rs
crates/notifications/Cargo.toml
crates/notifications/LICENSE-GPL
crates/notifications/LICENSE-GPL~merge-20260924
crates/notifications/src/status_toast.rs
crates/oauth_callback_server/Cargo.toml
crates/oauth_callback_server/LICENSE-APACHE
crates/oauth_callback_server/LICENSE-APACHE~merge-20260924
crates/ollama/Cargo.toml
crates/ollama/LICENSE-GPL
crates/ollama/LICENSE-GPL~merge-20260924
crates/ollama/src/ollama.rs
crates/onboarding/Cargo.toml
crates/onboarding/LICENSE-GPL
crates/onboarding/LICENSE-GPL~merge-20260924
crates/onboarding/src/base_keymap_picker.rs
crates/onboarding/src/basics_page.rs
crates/onboarding/src/onboarding.rs
crates/onboarding/src/theme_preview.rs
crates/open_ai/Cargo.toml
crates/open_ai/LICENSE-GPL
crates/open_ai/LICENSE-GPL~merge-20260924
crates/open_ai/src/batches.rs
crates/open_ai/src/completion.rs
crates/open_ai/src/open_ai.rs
crates/open_ai/src/responses.rs
crates/open_path_prompt/Cargo.toml
crates/open_path_prompt/LICENSE-GPL
crates/open_path_prompt/LICENSE-GPL~merge-20260924
crates/open_path_prompt/src/file_finder_settings.rs
crates/open_path_prompt/src/open_path_prompt.rs
crates/open_path_prompt/src/open_path_prompt_tests.rs
crates/open_router/Cargo.toml
crates/open_router/LICENSE-GPL
crates/open_router/LICENSE-GPL~merge-20260924
crates/open_router/src/open_router.rs
crates/opencode/Cargo.toml
crates/opencode/LICENSE-GPL
crates/opencode/LICENSE-GPL~merge-20260924
crates/opencode/src/opencode.rs
crates/outline/Cargo.toml
crates/outline/LICENSE-GPL
crates/outline/LICENSE-GPL~merge-20260924
crates/outline/src/outline.rs
crates/outline_panel/Cargo.toml
crates/outline_panel/LICENSE-GPL
crates/outline_panel/LICENSE-GPL~merge-20260924
crates/outline_panel/src/outline_panel.rs
crates/outline_panel/src/outline_panel_settings.rs
crates/paths/Cargo.toml
crates/paths/LICENSE-GPL
crates/paths/LICENSE-GPL~merge-20260924
crates/paths/src/paths.rs
crates/picker/Cargo.toml
crates/picker/LICENSE-GPL
crates/picker/LICENSE-GPL~merge-20260924
crates/picker/src/head.rs
crates/picker/src/highlighted_match_with_paths.rs
crates/picker/src/picker.rs
crates/picker/src/popover_menu.rs
crates/platform_title_bar/Cargo.toml
crates/platform_title_bar/LICENSE-GPL
crates/platform_title_bar/LICENSE-GPL~merge-20260924
crates/platform_title_bar/src/platform_title_bar.rs
crates/platform_title_bar/src/platforms/platform_linux.rs
crates/platform_title_bar/src/platforms/platform_windows.rs
crates/prettier/LICENSE-GPL
crates/prettier/LICENSE-GPL~merge-20260924
crates/project/Cargo.toml
crates/project/LICENSE-GPL
crates/project/LICENSE-GPL~merge-20260924
crates/project/src/agent_registry_store.rs
crates/project/src/agent_server_store.rs
crates/project/src/bookmark_store.rs
crates/project/src/buffer_store.rs
crates/project/src/context_server_store.rs
crates/project/src/debugger/breakpoint_store.rs
crates/project/src/debugger/dap_command.rs
crates/project/src/debugger/locators/python.rs
crates/project/src/debugger/session.rs
crates/project/src/git_store.rs
crates/project/src/git_store/conflict_set.rs
crates/project/src/image_store.rs
crates/project/src/lsp_command.rs
crates/project/src/lsp_store.rs
crates/project/src/lsp_store/clangd_ext.rs
crates/project/src/lsp_store/code_lens.rs
crates/project/src/lsp_store/document_colors.rs
crates/project/src/lsp_store/document_links.rs
crates/project/src/lsp_store/document_symbols.rs
crates/project/src/lsp_store/folding_ranges.rs
crates/project/src/lsp_store/inlay_hints.rs
crates/project/src/lsp_store/log_store.rs
crates/project/src/lsp_store/lsp_ext_command.rs
crates/project/src/lsp_store/semantic_tokens.rs
crates/project/src/manifest_tree/path_trie.rs
crates/project/src/prettier_store.rs
crates/project/src/project.rs
crates/project/src/project_search.rs
crates/project/src/project_settings.rs
crates/project/src/search.rs
crates/project/src/search_history.rs
crates/project/src/task_inventory.rs
crates/project/src/terminals.rs
crates/project/src/toolchain_store.rs
crates/project/src/worktree_store.rs
crates/project/tests/integration/agent_registry_store.rs
crates/project/tests/integration/bookmark_store.rs
crates/project/tests/integration/context_server_store.rs
crates/project/tests/integration/debugger.rs
crates/project/tests/integration/git_store.rs
crates/project/tests/integration/lsp_command.rs
crates/project/tests/integration/lsp_store.rs
crates/project/tests/integration/project_search.rs
crates/project/tests/integration/project_tests.rs
crates/project/tests/integration/search.rs
crates/project/tests/integration/signature_help.rs
crates/project_benchmarks/LICENSE-GPL
crates/project_benchmarks/LICENSE-GPL~merge-20260924
crates/project_benchmarks/src/main.rs
crates/project_panel/Cargo.toml
crates/project_panel/LICENSE-GPL
crates/project_panel/LICENSE-GPL~merge-20260924
crates/project_panel/benches/sorting.rs
crates/project_panel/src/project_panel.rs
crates/project_panel/src/project_panel_settings.rs
crates/project_panel/src/project_panel_tests.rs
crates/project_panel/src/tests/undo.rs
crates/project_panel/src/undo.rs
crates/project_panel/src/utils.rs
crates/project_symbols/Cargo.toml
crates/project_symbols/LICENSE-GPL
crates/project_symbols/LICENSE-GPL~merge-20260924
crates/project_symbols/src/project_symbols.rs
crates/prompt_store/Cargo.toml
crates/prompt_store/LICENSE-GPL
crates/prompt_store/LICENSE-GPL~merge-20260924
crates/prompt_store/src/prompt_store.rs
crates/prompt_store/src/prompts.rs
crates/proto/Cargo.toml
crates/proto/LICENSE-GPL
crates/proto/LICENSE-GPL~merge-20260924
crates/proto/build.rs
crates/proto/proto/ai.proto
crates/proto/proto/app.proto
crates/proto/proto/buffer.proto
crates/proto/proto/call.proto
crates/proto/proto/core.proto
crates/proto/proto/git.proto
crates/proto/proto/lsp.proto
crates/proto/proto/worktree.proto
crates/proto/proto/zed.proto
crates/proto/src/proto.rs
crates/proto/src/typed_envelope.rs
crates/recent_projects/Cargo.toml
crates/recent_projects/LICENSE-GPL
crates/recent_projects/LICENSE-GPL~merge-20260924
crates/recent_projects/src/dev_container_suggest.rs
crates/recent_projects/src/recent_projects.rs
crates/recent_projects/src/remote_connections.rs
crates/recent_projects/src/remote_servers.rs
crates/recent_projects/src/remote_servers/filter.rs
crates/recent_projects/src/sidebar_recent_projects.rs
crates/recent_projects/src/wsl_picker.rs
crates/refineable/Cargo.toml
crates/refineable/LICENSE-APACHE
crates/refineable/LICENSE-APACHE~merge-20260924
crates/refineable/derive_refineable/Cargo.toml
crates/refineable/derive_refineable/LICENSE-APACHE
crates/refineable/derive_refineable/LICENSE-APACHE~merge-20260924
crates/release_channel/Cargo.toml
crates/release_channel/LICENSE-GPL
crates/release_channel/LICENSE-GPL~merge-20260924
crates/release_channel/src/lib.rs
crates/remote/Cargo.toml
crates/remote/LICENSE-GPL
crates/remote/LICENSE-GPL~merge-20260924
crates/remote/src/protocol.rs
crates/remote/src/remote_client.rs
crates/remote/src/remote_identity.rs
crates/remote/src/transport.rs
crates/remote/src/transport/docker.rs
crates/remote/src/transport/mock.rs
crates/remote/src/transport/ssh.rs
crates/remote/src/transport/wsl.rs
crates/remote_connection/Cargo.toml
crates/remote_connection/LICENSE-GPL
crates/remote_connection/LICENSE-GPL~merge-20260924
crates/remote_connection/src/remote_connection.rs
crates/remote_server/Cargo.toml
crates/remote_server/LICENSE-GPL
crates/remote_server/LICENSE-GPL~merge-20260924
crates/remote_server/src/headless_project.rs
crates/remote_server/src/remote_editing_tests.rs
crates/remote_server/src/server.rs
crates/repl/Cargo.toml
crates/repl/LICENSE-GPL
crates/repl/LICENSE-GPL~merge-20260924
crates/repl/src/components/kernel_options.rs
crates/repl/src/kernels/remote_kernels.rs
crates/repl/src/kernels/ssh_kernel.rs
crates/repl/src/notebook/cell.rs
crates/repl/src/notebook/notebook_ui.rs
crates/repl/src/outputs.rs
crates/repl/src/outputs/plain.rs
crates/repl/src/repl_editor.rs
crates/repl/src/repl_sessions_ui.rs
crates/reqwest_client/Cargo.toml
crates/reqwest_client/LICENSE-APACHE
crates/reqwest_client/LICENSE-APACHE~merge-20260924
crates/reqwest_client/src/reqwest_client.rs
crates/rope/Cargo.toml
crates/rope/LICENSE-GPL
crates/rope/LICENSE-GPL~merge-20260924
crates/rpc/LICENSE-GPL
crates/rpc/LICENSE-GPL~merge-20260924
crates/rpc/src/message_stream.rs
crates/rpc/src/peer.rs
crates/rpc/src/proto_client.rs
crates/sandbox/Cargo.toml
crates/sandbox/LICENSE-GPL
crates/sandbox/LICENSE-GPL~merge-20260924
crates/sandbox/src/macos_seatbelt.rs
crates/sandbox/src/sandbox.rs
crates/scheduler/Cargo.toml
crates/scheduler/LICENSE-APACHE
crates/scheduler/LICENSE-APACHE~merge-20260924
crates/scheduler/src/executor.rs
crates/scheduler/src/scheduler.rs
crates/scheduler/src/test_scheduler.rs
crates/scheduler/src/tests.rs
crates/schema_generator/Cargo.toml
crates/schema_generator/LICENSE-GPL
crates/schema_generator/LICENSE-GPL~merge-20260924
crates/schema_generator/src/main.rs
crates/search/Cargo.toml
crates/search/LICENSE-GPL
crates/search/LICENSE-GPL~merge-20260924
crates/search/src/buffer_search.rs
crates/search/src/project_search.rs
crates/search/src/search.rs
crates/search/src/search_status_button.rs
crates/session/LICENSE-GPL
crates/session/LICENSE-GPL~merge-20260924
crates/session/src/session.rs
crates/settings/Cargo.toml
crates/settings/LICENSE-GPL
crates/settings/LICENSE-GPL~merge-20260924
crates/settings/src/base_keymap_setting.rs
crates/settings/src/content_into_gpui.rs
crates/settings/src/editorconfig_store.rs
crates/settings/src/keymap_file.rs
crates/settings/src/settings.rs
crates/settings/src/settings_file.rs
crates/settings/src/settings_store.rs
crates/settings/src/vscode_import.rs
crates/settings_content/Cargo.toml
crates/settings_content/LICENSE-GPL
crates/settings_content/LICENSE-GPL~merge-20260924
crates/settings_content/src/agent.rs
crates/settings_content/src/editor.rs
crates/settings_content/src/fallible_options.rs
crates/settings_content/src/language.rs
crates/settings_content/src/language_model.rs
crates/settings_content/src/merge_from.rs
crates/settings_content/src/project.rs
crates/settings_content/src/settings_content.rs
crates/settings_content/src/terminal.rs
crates/settings_content/src/theme.rs
crates/settings_content/src/title_bar.rs
crates/settings_content/src/workspace.rs
crates/settings_json/Cargo.toml
crates/settings_json/LICENSE-GPL
crates/settings_json/LICENSE-GPL~merge-20260924
crates/settings_json/src/settings_json.rs
crates/settings_macros/Cargo.toml
crates/settings_macros/LICENSE-GPL
crates/settings_macros/LICENSE-GPL~merge-20260924
crates/settings_profile_selector/LICENSE-GPL
crates/settings_profile_selector/LICENSE-GPL~merge-20260924
crates/settings_profile_selector/src/settings_profile_selector.rs
crates/settings_ui/Cargo.toml
crates/settings_ui/LICENSE-GPL
crates/settings_ui/LICENSE-GPL~merge-20260924
crates/settings_ui/src/components.rs
crates/settings_ui/src/components/dropdown.rs
crates/settings_ui/src/components/font_picker.rs
crates/settings_ui/src/components/icon_theme_picker.rs
crates/settings_ui/src/components/input_field.rs
crates/settings_ui/src/components/number_field.rs
crates/settings_ui/src/components/ollama_model_picker.rs
crates/settings_ui/src/components/section_items.rs
crates/settings_ui/src/components/theme_picker.rs
crates/settings_ui/src/page_data.rs
crates/settings_ui/src/pages.rs
crates/settings_ui/src/pages/audio_input_output_setup.rs
crates/settings_ui/src/pages/audio_test_window.rs
crates/settings_ui/src/pages/edit_prediction_provider_setup.rs
crates/settings_ui/src/pages/skill_creator.rs
crates/settings_ui/src/pages/skills_setup.rs
crates/settings_ui/src/pages/tool_permissions_setup.rs
crates/settings_ui/src/settings_ui.rs
crates/shell_command_parser/Cargo.toml
crates/shell_command_parser/LICENSE-GPL
crates/shell_command_parser/LICENSE-GPL~merge-20260924
crates/sidebar/Cargo.toml
crates/sidebar/LICENSE-GPL
crates/sidebar/LICENSE-GPL~merge-20260924
crates/sidebar/src/sidebar.rs
crates/sidebar/src/sidebar_tests.rs
crates/sidebar/src/thread_switcher.rs
crates/snippet/LICENSE-GPL
crates/snippet/LICENSE-GPL~merge-20260924
crates/snippet/src/snippet.rs
crates/snippet_provider/Cargo.toml
crates/snippet_provider/LICENSE-GPL
crates/snippet_provider/LICENSE-GPL~merge-20260924
crates/snippet_provider/src/registry.rs
crates/snippets_ui/Cargo.toml
crates/snippets_ui/LICENSE-GPL
crates/snippets_ui/LICENSE-GPL~merge-20260924
crates/snippets_ui/src/snippets_ui.rs
crates/sqlez/Cargo.toml
crates/sqlez/LICENSE-GPL
crates/sqlez/LICENSE-GPL~merge-20260924
crates/sqlez/src/connection.rs
crates/sqlez/src/statement.rs
crates/sqlez_macros/Cargo.toml
crates/sqlez_macros/LICENSE-GPL
crates/sqlez_macros/LICENSE-GPL~merge-20260924
crates/sqlez_macros/src/sqlez_macros.rs
crates/streaming_diff/Cargo.toml
crates/streaming_diff/LICENSE-GPL
crates/streaming_diff/LICENSE-GPL~merge-20260924
crates/sum_tree/Cargo.toml
crates/sum_tree/LICENSE-APACHE
crates/sum_tree/LICENSE-APACHE~merge-20260924
crates/sum_tree/src/cursor.rs
crates/svg_preview/Cargo.toml
crates/svg_preview/LICENSE-GPL
crates/svg_preview/LICENSE-GPL~merge-20260924
crates/svg_preview/src/svg_preview_view.rs
crates/syntax_theme/LICENSE-APACHE
crates/syntax_theme/LICENSE-APACHE~merge-20260924
crates/syntax_theme/LICENSE-GPL
crates/syntax_theme/LICENSE-GPL~merge-20260924
crates/syntax_theme/src/syntax_theme.rs
crates/system_specs/Cargo.toml
crates/system_specs/LICENSE-GPL
crates/system_specs/LICENSE-GPL~merge-20260924
crates/tab_switcher/LICENSE-GPL
crates/tab_switcher/LICENSE-GPL~merge-20260924
crates/tab_switcher/src/tab_switcher.rs
crates/tab_switcher/src/tab_switcher_tests.rs
crates/task/LICENSE-GPL
crates/task/LICENSE-GPL~merge-20260924
crates/task/src/task_template.rs
crates/task/src/vscode_format.rs
crates/tasks_ui/Cargo.toml
crates/tasks_ui/LICENSE-GPL
crates/tasks_ui/LICENSE-GPL~merge-20260924
crates/tasks_ui/src/modal.rs
crates/tasks_ui/src/tasks_ui.rs
crates/telemetry/Cargo.toml
crates/telemetry/LICENSE-GPL
crates/telemetry/LICENSE-GPL~merge-20260924
crates/telemetry_events/Cargo.toml
crates/telemetry_events/LICENSE-GPL
crates/telemetry_events/LICENSE-GPL~merge-20260924
crates/terminal/LICENSE-GPL
crates/terminal/LICENSE-GPL~merge-20260924
crates/terminal/src/alacritty.rs
crates/terminal/src/alacritty/hyperlinks.rs
crates/terminal/src/mappings/keys.rs
crates/terminal/src/mappings/mouse.rs
crates/terminal/src/pty_info.rs
crates/terminal/src/terminal.rs
crates/terminal/src/terminal_settings.rs
crates/terminal_view/Cargo.toml
crates/terminal_view/LICENSE-GPL
crates/terminal_view/LICENSE-GPL~merge-20260924
crates/terminal_view/scripts/print256color.sh
crates/terminal_view/scripts/truecolor.sh
crates/terminal_view/src/persistence.rs
crates/terminal_view/src/terminal_element.rs
crates/terminal_view/src/terminal_panel.rs
crates/terminal_view/src/terminal_path_like_target.rs
crates/terminal_view/src/terminal_view.rs
crates/text/LICENSE-GPL
crates/text/LICENSE-GPL~merge-20260924
crates/text/src/anchor.rs
crates/text/src/operation_queue.rs
crates/text/src/patch.rs
crates/text/src/selection.rs
crates/text/src/text.rs
crates/theme/LICENSE-GPL
crates/theme/LICENSE-GPL~merge-20260924
crates/theme/src/default_colors.rs
crates/theme/src/fallback_themes.rs
crates/theme/src/font_family_cache.rs
crates/theme/src/icon_theme.rs
crates/theme/src/styles/colors.rs
crates/theme/src/theme.rs
crates/theme_extension/Cargo.toml
crates/theme_extension/LICENSE-GPL
crates/theme_extension/LICENSE-GPL~merge-20260924
crates/theme_importer/LICENSE-GPL
crates/theme_importer/LICENSE-GPL~merge-20260924
crates/theme_importer/src/vscode/converter.rs
crates/theme_selector/Cargo.toml
crates/theme_selector/LICENSE-GPL
crates/theme_selector/LICENSE-GPL~merge-20260924
crates/theme_selector/src/icon_theme_selector.rs
crates/theme_selector/src/theme_selector.rs
crates/theme_settings/LICENSE-GPL
crates/theme_settings/LICENSE-GPL~merge-20260924
crates/theme_settings/src/schema.rs
crates/theme_settings/src/settings.rs
crates/theme_settings/src/theme_settings.rs
crates/time_format/LICENSE-GPL
crates/time_format/LICENSE-GPL~merge-20260924
crates/title_bar/Cargo.toml
crates/title_bar/LICENSE-GPL
crates/title_bar/LICENSE-GPL~merge-20260924
crates/title_bar/src/application_menu.rs
crates/title_bar/src/collab.rs
crates/title_bar/src/plan_chip.rs
crates/title_bar/src/title_bar.rs
crates/title_bar/src/title_bar_settings.rs
crates/title_bar/src/update_version.rs
crates/toolchain_selector/Cargo.toml
crates/toolchain_selector/LICENSE-GPL
crates/toolchain_selector/LICENSE-GPL~merge-20260924
crates/toolchain_selector/src/active_toolchain.rs
crates/toolchain_selector/src/toolchain_selector.rs
crates/ui/Cargo.toml
crates/ui/LICENSE-GPL
crates/ui/LICENSE-GPL~merge-20260924
crates/ui/src/components.rs
crates/ui/src/components/ai/agent_setup_button.rs
crates/ui/src/components/ai/ai_setting_item.rs
crates/ui/src/components/ai/configured_api_card.rs
crates/ui/src/components/ai/skills_illustration.rs
crates/ui/src/components/ai/thread_item.rs
crates/ui/src/components/avatar.rs
crates/ui/src/components/button/button.rs
crates/ui/src/components/button/button_like.rs
crates/ui/src/components/button/icon_button.rs
crates/ui/src/components/button/split_button.rs
crates/ui/src/components/button/toggle_button.rs
crates/ui/src/components/callout.rs
crates/ui/src/components/collab/collab_notification.rs
crates/ui/src/components/collab/update_button.rs
crates/ui/src/components/context_menu.rs
crates/ui/src/components/count_badge.rs
crates/ui/src/components/data_table.rs
crates/ui/src/components/data_table/table_row.rs
crates/ui/src/components/data_table/tests.rs
crates/ui/src/components/diff_stat.rs
crates/ui/src/components/disclosure.rs
crates/ui/src/components/divider.rs
crates/ui/src/components/dropdown_menu.rs
crates/ui/src/components/gradient_fade.rs
crates/ui/src/components/icon.rs
crates/ui/src/components/image.rs
crates/ui/src/components/indent_guides.rs
crates/ui/src/components/indicator.rs
crates/ui/src/components/keybinding.rs
crates/ui/src/components/label/highlighted_label.rs
crates/ui/src/components/label/label.rs
crates/ui/src/components/list/list_bullet_item.rs
crates/ui/src/components/list/list_header.rs
crates/ui/src/components/list/list_item.rs
crates/ui/src/components/notification/announcement_toast.rs
crates/ui/src/components/popover_menu.rs
crates/ui/src/components/progress/circular_progress.rs
crates/ui/src/components/redistributable_columns.rs
crates/ui/src/components/right_click_menu.rs
crates/ui/src/components/scrollbar.rs
crates/ui/src/components/tab.rs
crates/ui/src/components/toggle.rs
crates/ui/src/components/tree_view_item.rs
crates/ui/src/prelude.rs
crates/ui/src/styles.rs
crates/ui/src/styles/appearance.rs
crates/ui/src/styles/platform.rs
crates/ui/src/styles/typography.rs
crates/ui/src/styles/units.rs
crates/ui/src/traits/animation_ext.rs
crates/ui/src/ui.rs
crates/ui/src/utils.rs
crates/ui/src/utils/constants.rs
crates/ui_input/Cargo.toml
crates/ui_input/LICENSE-GPL
crates/ui_input/LICENSE-GPL~merge-20260924
crates/ui_input/src/input_field.rs
crates/ui_input/src/ui_input.rs
crates/ui_macros/Cargo.toml
crates/ui_macros/LICENSE-GPL
crates/ui_macros/LICENSE-GPL~merge-20260924
crates/ui_prompt/Cargo.toml
crates/ui_prompt/LICENSE-GPL
crates/ui_prompt/LICENSE-GPL~merge-20260924
crates/ui_prompt/src/ui_prompt.rs
crates/util/Cargo.toml
crates/util/LICENSE-APACHE
crates/util/LICENSE-APACHE~merge-20260924
crates/util/src/command.rs
crates/util/src/command/darwin.rs
crates/util/src/markdown.rs
crates/util/src/paths.rs
crates/util/src/redact.rs
crates/util/src/shell.rs
crates/util/src/shell_builder.rs
crates/util/src/util.rs
crates/util_macros/Cargo.toml
crates/util_macros/LICENSE-APACHE
crates/util_macros/LICENSE-APACHE~merge-20260924
crates/vim/LICENSE-GPL
crates/vim/LICENSE-GPL~merge-20260924
crates/vim/src/command.rs
crates/vim/src/helix.rs
crates/vim/src/helix/boundary.rs
crates/vim/src/helix/duplicate.rs
crates/vim/src/helix/object.rs
crates/vim/src/helix/paste.rs
crates/vim/src/helix/select.rs
crates/vim/src/helix/surround.rs
crates/vim/src/insert.rs
crates/vim/src/mode_indicator.rs
crates/vim/src/motion.rs
crates/vim/src/normal.rs
crates/vim/src/normal/change.rs
crates/vim/src/normal/paste.rs
crates/vim/src/normal/scroll.rs
crates/vim/src/normal/search.rs
crates/vim/src/object.rs
crates/vim/src/state.rs
crates/vim/src/surrounds.rs
crates/vim/src/test.rs
crates/vim/src/vim.rs
crates/vim/src/visual.rs
crates/vim/test_data/test_change_w.json
crates/vim_mode_setting/Cargo.toml
crates/vim_mode_setting/LICENSE-GPL
crates/vim_mode_setting/LICENSE-GPL~merge-20260924
crates/watch/Cargo.toml
crates/watch/LICENSE-APACHE
crates/watch/LICENSE-APACHE~merge-20260924
crates/watch/src/watch.rs
crates/web_search/Cargo.toml
crates/web_search/LICENSE-GPL
crates/web_search/LICENSE-GPL~merge-20260924
crates/web_search_providers/Cargo.toml
crates/web_search_providers/LICENSE-GPL
crates/web_search_providers/LICENSE-GPL~merge-20260924
crates/which_key/Cargo.toml
crates/which_key/LICENSE-GPL
crates/which_key/LICENSE-GPL~merge-20260924
crates/which_key/src/which_key.rs
crates/which_key/src/which_key_modal.rs
crates/windows_resources/Cargo.toml
crates/windows_resources/LICENSE-GPL
crates/windows_resources/LICENSE-GPL~merge-20260924
crates/windows_resources/src/windows_resources.rs
crates/workspace/Cargo.toml
crates/workspace/LICENSE-GPL
crates/workspace/LICENSE-GPL~merge-20260924
crates/workspace/src/dock.rs
crates/workspace/src/history_manager.rs
crates/workspace/src/item.rs
crates/workspace/src/modal_layer.rs
crates/workspace/src/multi_workspace.rs
crates/workspace/src/multi_workspace_tests.rs
crates/workspace/src/notifications.rs
crates/workspace/src/pane.rs
crates/workspace/src/pane_group.rs
crates/workspace/src/path_link.rs
crates/workspace/src/persistence.rs
crates/workspace/src/persistence/model.rs
crates/workspace/src/searchable.rs
crates/workspace/src/security_modal.rs
crates/workspace/src/shared_screen.rs
crates/workspace/src/status_bar.rs
crates/workspace/src/tasks.rs
crates/workspace/src/toast_layer.rs
crates/workspace/src/welcome.rs
crates/workspace/src/workspace.rs
crates/workspace/src/workspace_settings.rs
crates/worktree/Cargo.toml
crates/worktree/LICENSE-GPL
crates/worktree/LICENSE-GPL~merge-20260924
crates/worktree/src/ignore.rs
crates/worktree/src/worktree.rs
crates/worktree/src/worktree_settings.rs
crates/worktree/tests/integration/worktree_settings_tests.rs
crates/worktree/tests/integration/worktree_tests.rs
crates/worktree_benchmarks/LICENSE-GPL
crates/worktree_benchmarks/LICENSE-GPL~merge-20260924
crates/worktree_benchmarks/src/main.rs
crates/x_ai/Cargo.toml
crates/x_ai/LICENSE-GPL
crates/x_ai/LICENSE-GPL~merge-20260924
crates/x_ai/src/x_ai.rs
crates/zed/Cargo.toml
crates/zed/LICENSE-GPL
crates/zed/LICENSE-GPL~merge-20260924
crates/zed/build.rs
crates/zed/contents/dev/embedded.provisionprofile
crates/zed/contents/nightly/embedded.provisionprofile
crates/zed/contents/preview/embedded.provisionprofile
crates/zed/contents/stable/embedded.provisionprofile
crates/zed/resources/app-icon-dev.png
crates/zed/resources/app-icon-dev@2x.png
crates/zed/resources/app-icon-nightly.png
crates/zed/resources/app-icon-nightly@2x.png
crates/zed/resources/app-icon-preview.png
crates/zed/resources/app-icon-preview@2x.png
crates/zed/resources/app-icon.png
crates/zed/resources/app-icon@2x.png
crates/zed/resources/flatpak/zed.metainfo.xml.in
crates/zed/resources/info/DocumentTypes.plist
crates/zed/resources/info/Permissions.plist
crates/zed/resources/snap/snapcraft.yaml.in
crates/zed/resources/windows/app-icon-dev.ico
crates/zed/resources/windows/app-icon-nightly.ico
crates/zed/resources/windows/app-icon-preview.ico
crates/zed/resources/windows/app-icon.ico
crates/zed/resources/windows/zed.iss
crates/zed/src/main.rs
crates/zed/src/reliability.rs
crates/zed/src/reliability/hang_detection.rs
crates/zed/src/reliability/hang_detection/telemetry.rs
crates/zed/src/visual_test_runner.rs
crates/zed/src/zed.rs
crates/zed/src/zed/app_menus.rs
crates/zed/src/zed/edit_prediction_registry.rs
crates/zed/src/zed/mac_only_instance.rs
crates/zed/src/zed/move_to_applications.rs
crates/zed/src/zed/open_listener.rs
crates/zed/src/zed/quick_action_bar.rs
crates/zed/src/zed/quick_action_bar/preview.rs
crates/zed/src/zed/quick_action_bar/repl_menu.rs
crates/zed/src/zed/visual_tests.rs
crates/zed_actions/Cargo.toml
crates/zed_actions/LICENSE-GPL
crates/zed_actions/LICENSE-GPL~merge-20260924
crates/zed_actions/src/lib.rs
crates/zed_credentials_provider/Cargo.toml
crates/zed_credentials_provider/LICENSE-GPL
crates/zed_credentials_provider/LICENSE-GPL~merge-20260924
crates/zed_env_vars/Cargo.toml
crates/zed_env_vars/LICENSE-GPL
crates/zed_env_vars/LICENSE-GPL~merge-20260924
crates/zeta_prompt/Cargo.toml
crates/zeta_prompt/LICENSE-GPL
crates/zeta_prompt/LICENSE-GPL~merge-20260924
crates/zeta_prompt/src/multi_region.rs
crates/zeta_prompt/src/udiff.rs
crates/zeta_prompt/src/zeta_prompt.rs
crates/zlog/Cargo.toml
crates/zlog/src/filter.rs
crates/zlog_settings/Cargo.toml
crates/zlog_settings/LICENSE-GPL
crates/zlog_settings/LICENSE-GPL~merge-20260924
crates/ztracing/Cargo.toml
crates/ztracing/LICENSE-APACHE
crates/ztracing/LICENSE-APACHE~merge-20260924
crates/ztracing/src/lib.rs
crates/ztracing_macro/Cargo.toml
crates/ztracing_macro/LICENSE-APACHE
crates/ztracing_macro/LICENSE-APACHE~merge-20260924
extensions/glsl/Cargo.toml
extensions/glsl/LICENSE-APACHE
extensions/glsl/LICENSE-APACHE~merge-20260924
extensions/glsl/extension.toml
extensions/html/Cargo.toml
extensions/html/LICENSE-APACHE
extensions/html/LICENSE-APACHE~merge-20260924
extensions/html/extension.toml
extensions/proto/Cargo.toml
extensions/proto/LICENSE-APACHE
extensions/proto/LICENSE-APACHE~merge-20260924
extensions/proto/extension.toml
extensions/proto/src/language_servers/buf.rs
extensions/proto/src/language_servers/protols.rs
extensions/test-extension/Cargo.toml
extensions/test-extension/LICENSE-APACHE
extensions/test-extension/LICENSE-APACHE~merge-20260924
extensions/workflows/run_tests.yml
extensions/workflows/shared/bump_version.yml
legal/privacy-policy.md
legal/terms.md
legal/third-party-terms.md
nix/build.nix
nix/dev/flake.lock
nix/dev/flake.nix
nix/modules/devshells.nix
nix/modules/packages.nix
rust-toolchain.toml
script/bootstrap
script/build-docker
script/bump-extension-cli
script/bump-gpui-version
script/bump-nightly
script/bump-zed-version
script/bundle-freebsd
script/bundle-linux
script/bundle-mac
script/bundle-windows.ps1
script/cargo
script/cargo-timing-info.js
script/check-keymaps
script/check-licenses
script/check-links
script/check-todos
script/cherry-pick
script/clear-target-dir-if-larger-than
script/clippy
script/collab-flamegraph
script/community-pr-track-mapping.json
script/crate-dep-graph
script/create-draft-release
script/danger/dangerfile.ts
script/danger/package.json
script/danger/pnpm-lock.yaml
script/debug-cli
script/deploy-collab
script/determine-release-channel
script/digital-ocean-db.sh
script/docs-strip-preview-callouts
script/docs-suggest
script/docs-suggest-publish
script/download-wasi-sdk
script/draft-release-notes
script/drop-test-dbs
script/flatpak/bundle-flatpak
script/flatpak/deps
script/freebsd
script/generate-action-metadata
script/generate-licenses
script/generate-licenses-csv
script/generate-terms-rtf
script/get-crate-version
script/get-pull-requests-since
script/get-release-notes-since
script/get-released-version
script/github-check-new-issue-for-duplicates.py
script/github-clean-issue-types.py
script/github-community-pr-board.py
script/github-pr-status
script/github-track-duplicate-bot-effectiveness.py
script/histogram
script/import-themes
script/install-cmake
script/install-linux
script/install.sh
script/kube-shell
script/lib/bump-version.sh
script/licenses/zed-licenses.toml
script/linux
script/metal-debug
script/mitm-proxy.sh
script/new-crate
script/prettier
script/prompts
script/randomized-test-ci
script/randomized-test-minimize
script/remote-server
script/reset_db
script/run-background-agent-mvp-local
script/run-local-minio
script/run-unit-evals
script/seed-db
script/select-sentry-crash-candidates
script/sentry-fetch
script/setup-sccache
script/setup-sccache.ps1
script/shellcheck-scripts
script/snap-build
script/snap-try
script/squawk
script/terms/terms.rtf
script/test-docs-suggest-batch
script/trigger-docs-build
script/trigger-release
script/update-json-schemas
script/update_top_ranking_issues/uv.lock
script/upload-extension-cli
script/upload-nightly
script/verify-macos-document-icon
script/what-is-deployed
script/zed-local
tooling/compliance/Cargo.toml
tooling/compliance/LICENSE-GPL
tooling/compliance/LICENSE-GPL~merge-20260924
tooling/perf/Cargo.toml
tooling/perf/LICENSE-APACHE
tooling/perf/LICENSE-APACHE~merge-20260924
tooling/perf/src/main.rs
tooling/xtask/Cargo.toml
tooling/xtask/LICENSE-GPL
tooling/xtask/LICENSE-GPL~merge-20260924
tooling/xtask/src/main.rs
tooling/xtask/src/tasks.rs
tooling/xtask/src/tasks/workflow_checks.rs
tooling/xtask/src/tasks/workflow_checks/check_run_patterns.rs
tooling/xtask/src/tasks/workflows.rs
tooling/xtask/src/tasks/workflows/after_release.rs
tooling/xtask/src/tasks/workflows/autofix_pr.rs
tooling/xtask/src/tasks/workflows/bump_patch_version.rs
tooling/xtask/src/tasks/workflows/bump_zed_version.rs
tooling/xtask/src/tasks/workflows/cherry_pick.rs
tooling/xtask/src/tasks/workflows/compliance_check.rs
tooling/xtask/src/tasks/workflows/danger.rs
tooling/xtask/src/tasks/workflows/deploy_collab.rs
tooling/xtask/src/tasks/workflows/deploy_docs.rs
tooling/xtask/src/tasks/workflows/extension_auto_bump.rs
tooling/xtask/src/tasks/workflows/extension_bump.rs
tooling/xtask/src/tasks/workflows/extension_tests.rs
tooling/xtask/src/tasks/workflows/extension_workflow_rollout.rs
tooling/xtask/src/tasks/workflows/extensions/bump_version.rs
tooling/xtask/src/tasks/workflows/extensions/run_tests.rs
tooling/xtask/src/tasks/workflows/nix_build.rs
tooling/xtask/src/tasks/workflows/publish_extension_cli.rs
tooling/xtask/src/tasks/workflows/release.rs
tooling/xtask/src/tasks/workflows/release_nightly.rs
tooling/xtask/src/tasks/workflows/run_bundling.rs
tooling/xtask/src/tasks/workflows/run_tests.rs
tooling/xtask/src/tasks/workflows/runners.rs
tooling/xtask/src/tasks/workflows/steps.rs
tooling/xtask/src/tasks/workflows/vars.rs
tooling/xtask/src/workspace.rs
```
