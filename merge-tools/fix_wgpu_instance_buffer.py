#!/usr/bin/env python3
"""Move `instance_buffer()` from `impl WgpuRenderer` to `impl WgpuRendererCore`.

Marker-free defect: its ONLY caller is `draw_instances_with_backdrop`, which the merge
placed in the `WgpuRendererCore` impl (upstream's split). `instance_buffer` uses only
`self.resources()`, which Core also provides, so relocating it is behaviour-preserving.
"""
P = r"G:/Dx/code/crates/gpui_wgpu/src/wgpu_renderer.rs"

raw = open(P, "r", encoding="utf-8", newline="").read()
had_crlf = "\r\n" in raw
text = raw.replace("\r\n", "\n")

BLOCK = (
    "    /// The shared instance-data storage buffer, or `None` when instance data\n"
    "    /// travels through the WebGL instance texture instead.\n"
    "    fn instance_buffer(&self) -> Option<&wgpu::Buffer> {\n"
    "        match &self.resources().instance_data {\n"
    "            InstanceData::Storage(buffer) => Some(buffer),\n"
    "            InstanceData::Texture { .. } => None,\n"
    "        }\n"
    "    }\n\n"
)
assert text.count(BLOCK) == 1, f"instance_buffer block count={text.count(BLOCK)}"
text = text.replace(BLOCK, "", 1)
print("removed instance_buffer from impl WgpuRenderer")

# Core's resources_mut is the one-liner form; the Renderer's is the multi-line
# core_mut()/expect() form, so this anchor is unique.
ANCHOR = (
    "    fn resources_mut(&mut self) -> &mut WgpuResources {\n"
    "        &mut self.resources\n"
    "    }\n\n"
)
assert text.count(ANCHOR) == 1, f"Core resources_mut anchor count={text.count(ANCHOR)}"
text = text.replace(ANCHOR, ANCHOR + BLOCK, 1)
print("inserted instance_buffer into impl WgpuRendererCore")

if had_crlf:
    text = text.replace("\n", "\r\n")
open(P, "w", encoding="utf-8", newline="").write(text)
print(f"wrote {P} (crlf={had_crlf})")
