#!/usr/bin/env python3
"""Adapt upstream's render_frame/record_frame plumbing to carry our liquid-glass needs.

Upstream's `record_frame(scene, frame_view, clear_color)` cannot serve our
`refresh_liquid_glass_backdrop`, which must copy from the frame TEXTURE and needs the
frame dimensions. Rather than move our feature off upstream's structure, we thread the
texture + size through the two private methods (upstream's structure kept, our feature
re-attached). Our `record_frame` body is otherwise already merged.
"""
P = r"G:/Dx/code/crates/gpui_wgpu/src/wgpu_renderer.rs"

raw = open(P, "r", encoding="utf-8", newline="").read()
had_crlf = "\r\n" in raw
text = raw.replace("\r\n", "\n")

EDITS = [
    # A: draw() -> render_frame now hands over the frame texture too
    ("        if let Err(error) = core.render_frame(\n"
     "            scene,\n"
     "            &frame_view,\n",
     "        if let Err(error) = core.render_frame(\n"
     "            scene,\n"
     "            &frame.texture,\n"
     "            &frame_view,\n"),
    # B: render_frame gains the texture param
    ("    fn render_frame(\n"
     "        &mut self,\n"
     "        scene: &Scene,\n"
     "        target_view: &wgpu::TextureView,\n",
     "    fn render_frame(\n"
     "        &mut self,\n"
     "        scene: &Scene,\n"
     "        frame_texture: &wgpu::Texture,\n"
     "        target_view: &wgpu::TextureView,\n"),
    # C: render_frame -> record_frame forwards texture + size
    ("        self.record_frame(scene, target_view, clear_color)\n",
     "        self.record_frame(scene, frame_texture, target_view, size, clear_color)\n"),
    # D: record_frame gains texture + size params
    ("    fn record_frame(\n"
     "        &mut self,\n"
     "        scene: &Scene,\n"
     "        frame_view: &wgpu::TextureView,\n"
     "        clear_color: wgpu::Color,\n"
     "    ) -> Result<wgpu::SubmissionIndex> {\n",
     "    fn record_frame(\n"
     "        &mut self,\n"
     "        scene: &Scene,\n"
     "        frame_texture: &wgpu::Texture,\n"
     "        frame_view: &wgpu::TextureView,\n"
     "        size: Size<DevicePixels>,\n"
     "        clear_color: wgpu::Color,\n"
     "    ) -> Result<wgpu::SubmissionIndex> {\n"),
    # E: our backdrop capture now uses the threaded texture + size
    ("                                &frame.texture,\n"
     "                                self.surface_config.width,\n"
     "                                self.surface_config.height,\n",
     "                                frame_texture,\n"
     "                                size.width.0 as u32,\n"
     "                                size.height.0 as u32,\n"),
]

for i, (old, new) in enumerate(EDITS):
    n = text.count(old)
    assert n == 1, f"edit {chr(65+i)}: expected 1 match, found {n}"
    text = text.replace(old, new)
    print(f"edit {chr(65+i)}: applied (1 match)")

if had_crlf:
    text = text.replace("\n", "\r\n")
open(P, "w", encoding="utf-8", newline="").write(text)
print(f"wrote {P} (crlf={had_crlf})")
