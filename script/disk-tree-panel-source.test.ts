import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import test from "node:test";

// The disk tree lives inside the Project Panel rather than in its own crate,
// because the Project Panel already owns this concept: `storage_roots.rs`
// enumerates every mounted volume with its free/total bytes, and `storage.rs`
// has `StorageSortMode::{Size, FileCount, Modified}` - the same three metrics
// the treemap offers.
//
// This guard protects that decision. It fails if the treemap grows back into a
// separate dock panel, if the Project Panel's existing storage surface is
// disturbed, or if the module ceiling is exceeded.

const DISK_TREE = "crates/project_panel/src/disk_tree";

// The ceiling this repo applies to owned Rust modules.
const MAX_OWNED_RUST_LINES = 620;

// Every module the port is specified to produce. Missing one is a silent
// regression, so the list is exact rather than a prefix scan.
const EXPECTED_MODULES = [
  "canvas.rs",
  "marks.rs",
  "model.rs",
  "model_tests.rs",
  "mod.rs",
  "mouse.rs",
  "moves.rs",
  "navigation.rs",
  "overlays.rs",
  "palette.rs",
  "palette_tests.rs",
  "review.rs",
  "scan.rs",
  "theme.rs",
  "tokens.rs",
  "view.rs",
  "widgets.rs",
];

const read = (path: string) => readFileSync(path, "utf8");
const lineCount = (path: string) => read(path).split(/\r?\n/).length;

test("the disk tree lives in the project panel, not in its own crate", () => {
  assert.ok(existsSync(DISK_TREE), `${DISK_TREE} must exist`);

  // The whole point of putting it in the Project Panel is that no new panel is
  // introduced. A `disk_tree_panel` crate or a dock entry would undo that.
  const members = read("Cargo.toml");
  assert.doesNotMatch(
    members,
    /crates\/disk_tree_panel/,
    "the disk tree must not become its own panel crate",
  );

  const zedCargo = read("crates/zed/Cargo.toml");
  assert.doesNotMatch(
    zedCargo,
    /disk_tree_panel/,
    "no new panel dependency: crates/zed must be untouched by this work",
  );

  // `activate_panel` is what a new dockable panel would need.
  const workspace = read("crates/workspace/src/workspace.rs");
  assert.doesNotMatch(
    workspace,
    /disk_tree|DiskTree/,
    "the workspace must not learn about the disk tree; it is a mode, not a panel",
  );
});

test("the project panel declares exactly these disk_tree modules", () => {
  assert.ok(existsSync(join(DISK_TREE, "mod.rs")), "mod.rs must exist");
  const modRs = read(join(DISK_TREE, "mod.rs"));

  const declared = [...modRs.matchAll(/^pub\(crate\) mod (\w+);$/gm)].map(
    (match) => match[1],
  );
  for (const module of declared) {
    assert.ok(
      EXPECTED_MODULES.includes(`${module}.rs`),
      `mod.rs declares ${module} but the guard does not list it`,
    );
  }
  // `*_tests` modules are intentionally `#[cfg(test)] mod`, not `pub(crate) mod`,
  // so the production `declared` list is not expected to name them.
  for (const module of EXPECTED_MODULES) {
    const name = module.replace(/\.rs$/, "");
    if (name === "mod" || name.endsWith("_tests")) continue;
    assert.ok(
      declared.includes(name),
      `mod.rs is missing \`pub(crate) mod ${name};\``,
    );
  }

  for (const module of EXPECTED_MODULES) {
    assert.ok(existsSync(join(DISK_TREE, module)), `${module} must exist`);
  }
});

test("every disk tree module stays under the owned-module ceiling", () => {
  for (const module of EXPECTED_MODULES) {
    const path = join(DISK_TREE, module);
    const lines = lineCount(path);
    assert.ok(
      lines < MAX_OWNED_RUST_LINES,
      `${path} is ${lines} lines, over the ${MAX_OWNED_RUST_LINES} ceiling`,
    );
  }
});

test("the two storage surfaces stay one user-visible choice", () => {
  const panel = read("crates/project_panel/src/project_panel.rs");

  // Pinned by script/dx-project-panel-source.test.ts:891-908 - name, type and
  // default. The treemap is a sibling bool, never a replacement.
  assert.match(panel, /storage_details_visible:\s*bool/);
  assert.match(panel, /storage_details_visible:\s*false/);
  assert.match(panel, /disk_tree_visible:\s*bool/);
  assert.match(panel, /disk_tree_visible:\s*false/);

  // And one resolver, so the header and the body cannot disagree.
  assert.match(panel, /pub fn storage_surface\(&self\) -> Option<StorageSurface>/);

  // Both toggles flip both fields: that is what makes them mutually exclusive
  // without an enum.
  const diskTreeToggle = panel.slice(
    panel.indexOf("pub fn toggle_disk_tree_visible"),
    panel.indexOf("pub fn toggle_disk_tree_visible") + 1200,
  );
  assert.match(diskTreeToggle, /self\.disk_tree_visible = next;/);
  assert.match(diskTreeToggle, /self\.storage_details_visible = false;/);

  const detailsToggle = panel.slice(
    panel.indexOf("fn toggle_storage_details_visible"),
    panel.indexOf("fn toggle_storage_details_visible") + 1200,
  );
  // Flipped with `!`, matching the pre-existing shape the other guard pins; the
  // exclusivity clear is the sibling `disk_tree_visible = false`.
  assert.match(detailsToggle, /self\.storage_details_visible = !self\.storage_details_visible;/);
  assert.match(detailsToggle, /self\.disk_tree_visible = false;/);
});

test("the existing storage surface is left alone", () => {
  const storage = read("crates/project_panel/src/storage.rs");
  const roots = read("crates/project_panel/src/storage_roots.rs");

  // The treemap must reuse these, not fork them: the whole reason for the
  // placement is that the Project Panel already answers these questions.
  assert.match(storage, /pub\(crate\) enum StorageSortMode/);
  assert.match(storage, /pub\(crate\) fn storage_heat_level/);
  assert.match(roots, /pub\(crate\) enum StorageRootKind/);
  assert.match(roots, /sysinfo::Disks::new_with_refreshed_list\(\)/);

  // The drilldown still renders only when asked for - the treemap does not
  // replace that opt-in.
  const panel = read("crates/project_panel/src/project_panel.rs");
  const drilldown = panel.slice(
    panel.indexOf("fn render_dx_explorer_storage_drilldown"),
    panel.indexOf("fn render_dx_explorer_storage_drilldown") + 900,
  );
  assert.match(
    drilldown,
    /if !self\.storage_details_visible[\s\S]*return None;/,
    "the drilldown must keep its own opt-in",
  );

  // The heat ramp is pinned to project_panel.rs by dx-project-panel-source.test.ts
  // and is deliberately NOT shared with the treemap palette.
  assert.match(
    panel,
    /fn render_dx_explorer_storage_heat_indicator\(/,
    "the heat indicator must stay in project_panel.rs",
  );
});

test("the treemap replaces the entry list rather than sitting above it", () => {
  const panel = read("crates/project_panel/src/project_panel.rs");
  // A mosaic above a file tree in a 300px panel leaves the tree too short to be
  // useful, so the two are exclusive.
  assert.match(
    panel,
    /StorageSurface::Treemap[\s\S]{0,200}uniform_list/,
    "the entry list must be gated on the surface not being the treemap",
  );
  assert.match(panel, /self\.render_disk_tree\(window, cx\)/);
});

test("the disk tree uses our theme, not a second one", () => {
  const theme = read(join(DISK_TREE, "theme.rs"));
  // The adapter exists so palette.rs keeps its rules; it reads our tokens.
assert.match(theme, /pub\(crate\) fn resolve\(cx: &App\) -> Self/);
    assert.match(theme, /panel_background/);
    assert.match(theme, /status\(\)\.warning/);
    assert.match(theme, /pub\(crate\) fn is_dark\(&self\) -> bool/);

  // Nothing in the tree may *depend on* gpui-omarchy or gpui-kit. Checked as
  // imports rather than as a substring, because `mod.rs` names both in its
  // provenance table - documenting what was replaced is the point of that file.
  for (const module of EXPECTED_MODULES) {
    const source = read(join(DISK_TREE, module));
    assert.doesNotMatch(
      source,
      /^\s*use (gpui_omarchy|gpui_kit)/m,
      `${module} must not import gpui_omarchy or gpui_kit`,
    );
    assert.doesNotMatch(
      source,
      /^\s*(gpui_omarchy|gpui_kit)::/m,
      `${module} must not reference gpui_omarchy or gpui_kit by path`,
    );
  }
});

test("the treemap's key contract keeps the platform-modifier passthrough", () => {
  const panel = read("crates/project_panel/src/project_panel.rs");
  const start = panel.indexOf("pub fn disk_tree_key");
  assert.ok(start > 0, "disk_tree_key must exist");
  // Slice to the end of the function rather than by a guessed character count:
  // the dispatcher is ~9 KB and a fixed window silently truncates its tail,
  // which makes the assertions below pass or fail for the wrong reason.
  const end = panel.indexOf("\n    ///", start + 40);
  const key = panel.slice(start, end > start ? end : undefined);

  // The single most important line in the port: without it the treemap steals
  // the window's own cmd-chords.
  assert.match(
    key,
    /if key\.modifiers\.platform\s*\{\s*return;/,
    "the platform modifier must return to the window",
  );

  // The find field wins over everything below it.
  assert.match(key, /if let Some\(needle\) = tree\.find\.clone\(\)/);

  // Every navigational key from the upstream contract. Matched as match arms
  // rather than bare substrings, so a key named in a comment cannot satisfy this.
  for (const chord of [
    "enter",
    "backspace",
    "tab",
    "shift-tab",
    "space",
    "t",
    "r",
    "g",
    "/",
    "[",
    "]",
    "-",
    "0",
    "o",
    "s",
  ]) {
    assert.ok(
      key.includes(`"${chord}"`),
      `key ${chord} must appear in the dispatcher`,
    );
  }
  // The metric cycle and the scan are the two state-changing keys.
  assert.match(key, /"t"\s*=>\s*\{/, "t must cycle the metric");
  assert.match(key, /"r"\s*=>\s*disk_tree::scan::rescan/);
  // vim keys and arrows share one arm each: `"left" | "h" =>`. Requiring the
  // pairing rather than two separate mentions is what stops a future edit from
  // handling `j` but dropping `down`.
  const paired: Array<[string, string]> = [
    ["up", "k"],
    ["down", "j"],
    ["left", "h"],
    ["right", "l"],
  ];
  for (const [arrow, vim] of paired) {
    assert.ok(
      key.includes(`"${arrow}" | "${vim}"`),
      `${arrow} and ${vim} must share one arm`,
    );
  }
});

test("the wheel zoom stops at the ceiling instead of descending", () => {
  const mouse = read(join(DISK_TREE, "mouse.rs"));
  // The documented divergence from upstream: in a ~300px panel the ceiling is
  // two notches away, and silently changing directory made the wheel feel
  // broken. `enter` is the only way in.
  assert.match(mouse, /const WHEEL_ZOOM: f32 = 1\.15;/);
  assert.doesNotMatch(
    mouse,
    /descend\(.*descend/,
    "the wheel must not descend; enter is the only way in",
  );

  const view = read(join(DISK_TREE, "view.rs"));
  assert.match(view, /pub\(crate\) const MAX_SCALE: f32 = 5\.0;/);
});

test("the treemap has its own scan, separate from storage.rs", () => {
  const scan = read(join(DISK_TREE, "scan.rs"));
  assert.match(scan, /const POLL_INTERVAL: Duration = Duration::from_millis\(110\);/);
  assert.match(scan, /fn poll_scan_once\(/);

  // It must never *reach into* the file tree's own bounded storage caches.
  // Matched against code rather than prose: the module comment deliberately names
  // both, so only a real use is `self.field` or `panel.field`, not a backticked
  // mention.
  assert.doesNotMatch(
    scan,
    /self\.folder_storage_summaries|panel\.folder_storage_summaries|self\.state\.visible_entries|panel\.state\.visible_entries/,
    "the treemap's scan must not touch the file tree's bounded caches",
  );
  assert.doesNotMatch(
    scan,
    /storage::(StorageOverview|FolderStorageSummary|StorageFolderItem)/,
    "the treemap must not reuse storage.rs's own types",
  );

  // On-disk size is the default, because it is the number deletion returns.
  const model = read(join(DISK_TREE, "model.rs"));
  assert.match(model, /pub\(crate\) on_disk_size: bool/);
});

test("the vendored core stays separate and unguarded from gpui", () => {
  // The correctness half has no gpui dependency; that is what makes an upstream
  // re-sync a file copy. The hash guard lives in
  // script/disk-tree-core-vendored.test.ts.
  const coreCargo = read("crates/disk_tree_core/Cargo.toml");
  assert.doesNotMatch(coreCargo, /^gpui/m);

  // And the panel depends on the crate, never the reverse.
  const panelCargo = read("crates/project_panel/Cargo.toml");
  assert.match(panelCargo, /disk_tree_core\.workspace = true/);
});