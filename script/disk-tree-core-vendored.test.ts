import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import test from "node:test";

// `crates/disk_tree_core` is vendored verbatim from tobi/disktree. The whole
// reason it is a separate crate with no gpui dependency is that re-syncing
// upstream stays a file copy instead of a merge.
//
// That only holds if the sources are actually untouched, so this guard pins the
// SHA-256 of every vendored file. Anything that reformats or "improves" one
// fails here instead of quietly becoming a merge conflict six months from now.
//
// `inspirations/` is not tracked by this repository, so the hashes are recorded
// here rather than diffed against it. Taken from inspirations/disktree at commit
// 7c0661c (tag v0.10.1).
//
// `lib.rs` is deliberately absent: it carries the provenance header and one
// crate-level `clippy::disallowed_methods` allow, both documented in the file.
// Its assertions live further down.

const UPSTREAM_COMMIT = "7c0661c";
const UPSTREAM_TAG = "v0.10.1";

const VENDORED_SHA256: Record<string, string> = {
  "access.rs": "af6f1a6ef451c3647698a56b30a4e0c56ada6e89b04a6fc99cbe38926acad527",
  "classify.rs": "044e137976ffda0341d390127b7a2b4525d8007d4db748747d6b5c9a5e90dad7",
  "export.rs": "de8f447169ccb31bb8931316ca7c4b59e10b3fd028fda59d1b591fdc3477f890",
  "filter.rs": "d9f5d6a54161c2c5536c266f8a5e51717597cb3ac0fe22686285faad5f3958a9",
  "insights.rs": "e4b8e4c3c3460f9b6c862726dff0954007d6feb74ddd7ba84ec3ad0b2310632e",
  "mft.rs": "39ca75ceccf7e5b1d8fc4ed0632d7bcacce9bbbf7d4e9fe6f3f6d5f7cf62a4c6",
  "removal.rs": "7bbeea353a9cb96eda683864489a3112494cdb9c8714f5ac7162490742d516bf",
  "scan.rs": "368730a4ba901b8b9fbe92f5cfb997215cf129d0993418319569667fe2ff2539",
  "scan_threads.rs": "62f28ae95c8452cb8e24106a9df3dd7fc60b32dd3157516716cedae3adbb2c1f",
  "size.rs": "22798fc6d8829174389286e91583b0b4477cc7dba61b6c7d1672f7cd24a0b71d",
  "space.rs": "93b8501afcc5eaed17b60f1ce76b63e8f0bd1639a0d25b712ab7cf522cce47ad",
  "tree.rs": "c3bdfd0a81520c1cd2bc230461f7591b6217007ce91a80578a5670fb0d40c022",
  "windows.rs": "3b1ad45f784507f781958e1b4cd91eb59bd55d04564bf7ee3c533bc53a77bd25",
};

// `treemap.rs` is the one file we extended rather than copied. The two additions
// are additive only - no upstream line was changed, removed or reformatted - so
// the re-sync is still a merge with a known-good base. What was added, and why:
//
//   * `LayoutOptions::rem` - the rem the two header heights are expressed in.
//   * `Tile::header_rems` - echoes the band heights back onto each tile.
//
// Together they let the painter fill the band the layout *actually reserved*
// without re-deriving it from `LayoutOptions`, which may describe a different rem
// than the one currently on screen. Without them a header label has to guess, and
// a guessed band drifts from its tile as interface zoom changes.
//
// The assertions below pin the "additive only" property, which is what makes this
// file different from the other thirteen rather than a hole in the guard.
const EXTENDED_WITH_ADDITIONS: Record<string, string[]> = {
  "treemap.rs": [
    "pub rem: f32,",
    "pub header_rems: Option<(f32, f32)>,",
    "rem: 16.0,",
    "header_rems: None,",
    "header_rems: subdividable.then_some((",
  ],
};

const SRC = "crates/disk_tree_core/src";

test("every vendored source file is byte-identical to upstream", () => {
  // 13 hashed + 1 extended = the 14 files that are upstream's.
  assert.equal(Object.keys(VENDORED_SHA256).length, 13);
  assert.equal(Object.keys(EXTENDED_WITH_ADDITIONS).length, 1);

  for (const [name, expected] of Object.entries(VENDORED_SHA256)) {
    const path = `${SRC}/${name}`;
    const actual = createHash("sha256")
      .update(readFileSync(path))
      .digest("hex");
    assert.equal(
      actual,
      expected,
      `${name} was modified. It is vendored from tobi/disktree@${UPSTREAM_COMMIT} (${UPSTREAM_TAG}); put the change in crates/project_panel/src/disk_tree/ instead, or re-vendor deliberately and update this hash.`,
    );
  }
});

test("the one extended file is extended additively and nothing else", () => {
  const path = `${SRC}/treemap.rs`;
  const source = readFileSync(path, "utf8");

  assert.equal(
    VENDORED_SHA256["treemap.rs"],
    undefined,
    "treemap.rs is extended, not byte-identical; it must not also claim a hash",
  );

  // Each documented addition is present.
  for (const line of EXTENDED_WITH_ADDITIONS["treemap.rs"]) {
    assert.ok(source.includes(line), `treemap.rs is missing: ${line}`);
  }

  // And it is *only* extended: every upstream struct field, every upstream
  // constructor site and every upstream test must survive untouched. Checked as
  // counts, because a renamed field keeps the same words.
  const fieldCount = (name: string) =>
    (source.match(new RegExp(`^    pub ${name}:`, "gm")) ?? []).length;
  for (const field of [
    "kind",
    "rect",
    "depth",
    "header",
    "max_depth",
    "padding",
    "padding_outer",
    "min_tile",
    "max_children",
    "header",
    "header_inner",
  ]) {
    assert.ok(fieldCount(field) >= 1, `treemap.rs lost field ${field}`);
  }

  // The upstream test module is still here. Losing it would mean the layout's
  // own squarify and nesting tests silently stopped compiling.
  assert.ok(source.includes("fn squarify_fills_the_area()"));
  assert.ok(source.includes("fn hit_returns_the_deepest_tile()"));
  assert.ok(
    source.includes("fn tile_crumbs_address_the_scanned_tree_not_the_drawn_node()"),
  );
  assert.ok(
    source.includes("fn a_filtered_layout_shows_only_the_matches_at_their_size()"),
  );
});

test("the vendored file set has not grown or shrunk", () => {
  // lib.rs is excluded from the hash map on purpose - it is the one file we
  // edited outright. Everything else must be exactly these 14.
  assert.ok(VENDORED_SHA256["access.rs"], "sanity: access.rs is listed");
  assert.equal(
    VENDORED_SHA256["lib.rs"],
    undefined,
    "lib.rs carries our provenance header; it must not be hashed as upstream-identical",
  );
  assert.equal(
    Object.keys(VENDORED_SHA256).length + Object.keys(EXTENDED_WITH_ADDITIONS).length,
    14,
  );
});

test("disk_tree_core imports no gpui and no ui crate", () => {
  // This is the invariant that makes an upstream re-sync a file copy rather
  // than a merge: the correctness half must stay independent of our UI.
  for (const name of [
    ...Object.keys(VENDORED_SHA256),
    ...Object.keys(EXTENDED_WITH_ADDITIONS),
    "lib.rs",
  ]) {
    const source = readFileSync(`${SRC}/${name}`, "utf8");
    assert.doesNotMatch(
      source,
      /\bgpui(::|_)::/,
      `${name} must not reference gpui`,
    );
    assert.doesNotMatch(
      source,
      /\bui::(Theme|Button|Icon|InteractiveElement)\b/,
      `${name} must not reference our ui crate`,
    );
    assert.doesNotMatch(
      source,
      /\b(disk_tree_core|project_panel)::/,
      `${name} must not reference our own crates`,
    );
  }
});

test("lib.rs keeps its documented provenance and its single lint allow", () => {
  const lib = readFileSync(`${SRC}/lib.rs`, "utf8");

  assert.match(
    lib,
    new RegExp(UPSTREAM_COMMIT),
    "lib.rs must name the upstream commit it was vendored from",
  );
  assert.match(lib, /LICENSE-MIT-disktree/, "lib.rs must point at the MIT notice");
  assert.match(
    lib,
    /Tobi L/,
    "lib.rs must carry the upstream copyright line",
  );

  // Exactly one allow, and it must be the documented process-spawn one.
  const allows = lib.match(/^#!\[allow\(/gm) ?? [];
  assert.equal(
    allows.length,
    1,
    "lib.rs should carry exactly one crate-level allow; a second one needs the same scrutiny",
  );
  assert.match(lib, /^#!\[allow\(clippy::disallowed_methods\)\]$/m);
  assert.match(
    lib,
    /clippy\.toml:9-14/,
    "the allow must cite clippy.toml so the next reader can re-check the policy",
  );
});

test("disk_tree_core carries the upstream MIT notice and the GPL symlink text", () => {
  const mit = readFileSync("crates/disk_tree_core/LICENSE-MIT-disktree", "utf8");
  assert.match(mit, /MIT License/);
  assert.match(mit, /Copyright \(c\) 2026 Tobi L/);
  assert.match(
    mit,
    /Permission is hereby granted, free of charge/,
    "the MIT grant itself must survive verbatim",
  );
  assert.match(mit, /THE SOFTWARE IS PROVIDED "AS IS"/);

  assert.equal(
    readFileSync("crates/disk_tree_core/LICENSE-GPL", "utf8"),
    "../../LICENSE-GPL",
    "LICENSE-GPL must name the root licence at the right relative depth",
  );
});

test("the manifest documents both deliberate deviations from upstream", () => {
  const manifest = readFileSync("crates/disk_tree_core/Cargo.toml", "utf8");

  assert.match(
    manifest,
    /unsafe_code/,
    "the manifest must record why upstream's `unsafe_code = \"deny\"` is not inherited",
  );
  assert.match(
    manifest,
    /zed-industries\/trash-rs/,
    "the manifest must record which trash fork the removal path relies on",
  );
  assert.match(
    manifest,
    /\[lints\]\s*\nworkspace = true/,
    "the crate must inherit the workspace lint set",
  );
  assert.match(manifest, /^license = "GPL-3\.0-or-later"$/m);
  assert.doesNotMatch(
    manifest,
    /^license-file/m,
    "script/check-licenses:13 forbids license-file",
  );

  // Inheriting a dependency must not re-declare its source.
  assert.match(manifest, /^trash\.workspace = true$/m);
  assert.doesNotMatch(
    manifest,
    /trash = \{ git/,
    "trash must come from the workspace so fs and disk_tree_core share one package",
  );
});