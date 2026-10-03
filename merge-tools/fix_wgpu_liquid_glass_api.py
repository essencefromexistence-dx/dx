#!/usr/bin/env python3
"""Fix the two marker-free integration defects the compile exposed in our liquid-glass port.

Upstream changed two APIs this file's OURS-side liquid-glass code was written against:
  * `create_pipeline` gained a 6th param `texture_layout: Option<&wgpu::BindGroupLayout>`.
    Our `liquid_glass` call never got it. `draw_instances_with_backdrop` sets only two bind
    groups (globals + `instances_with_backdrop`, which already bundles the atlas texture,
    the backdrop and the sampler), so the correct value is `None` — same as `quads`.
  * `atlas.get_texture_info()` now returns `Option<WgpuTextureInfo>`; our call site still
    used it as a plain struct. Guard it like upstream's other call site does.
"""
P = r"G:/Dx/code/crates/gpui_wgpu/src/wgpu_renderer.rs"

raw = open(P, "r", encoding="utf-8", newline="").read()
had_crlf = "\r\n" in raw
text = raw.replace("\r\n", "\n")

EDITS = [
    # 1) liquid_glass pipeline: supply the (absent) texture layout.
    ("            &layouts.instances_with_backdrop,\n"
     "            wgpu::PrimitiveTopology::TriangleStrip,\n",
     "            &layouts.instances_with_backdrop,\n"
     "            // Ours: the backdrop pipeline binds its texture through\n"
     "            // `instances_with_backdrop`, so it needs no separate layout.\n"
     "            None,\n"
     "            wgpu::PrimitiveTopology::TriangleStrip,\n"),
    # 2) get_texture_info is now fallible.
    ("        let tex_info = self.atlas.get_texture_info(texture_id);\n"
     "        let Some(backdrop_view) = self.resources().liquid_glass_backdrop_view.as_ref() else {\n",
     "        let Some(tex_info) = self.atlas.get_texture_info(texture_id) else {\n"
     "            return true;\n"
     "        };\n"
     "        let Some(backdrop_view) = self.resources().liquid_glass_backdrop_view.as_ref() else {\n"),
]

for i, (old, new) in enumerate(EDITS):
    n = text.count(old)
    assert n == 1, f"edit {i+1}: expected 1 match, found {n}"
    text = text.replace(old, new, 1)
    print(f"edit {i+1}: applied")

if had_crlf:
    text = text.replace("\n", "\r\n")
open(P, "w", encoding="utf-8", newline="").write(text)
print(f"wrote {P} (crlf={had_crlf})")
