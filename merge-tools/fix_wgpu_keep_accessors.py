#!/usr/bin/env python3
"""Keep ours' `WgpuRenderer::resources`/`resources_mut` accessors, marked as retained.

They exist so ours' call sites can keep speaking `self.resources()`; the merged code now
reaches `core().resources` directly, so they are currently unused. Per the merge policy
ours' code stays in the tree — `#[allow(dead_code)]` rather than deletion.
"""
P = r"G:/Dx/code/crates/gpui_wgpu/src/wgpu_renderer.rs"

raw = open(P, "r", encoding="utf-8", newline="").read()
had_crlf = "\r\n" in raw
text = raw.replace("\r\n", "\n")

A1 = """    /// vocabulary while `core()`/`core_mut()` remain the underlying accessors.
    fn resources(&self) -> &WgpuResources {"""
B1 = """    /// vocabulary while `core()`/`core_mut()` remain the underlying accessors.
    ///
    /// Kept rather than deleted: the merged call sites now reach
    /// `core().resources` directly, so ours' accessors are currently unused.
    #[allow(dead_code)]
    fn resources(&self) -> &WgpuResources {"""

A2 = """    fn resources_mut(&mut self) -> &mut WgpuResources {
        &mut self
            .core_mut()"""
B2 = """    #[allow(dead_code)]
    fn resources_mut(&mut self) -> &mut WgpuResources {
        &mut self
            .core_mut()"""

for i, (old, new) in enumerate([(A1, B1), (A2, B2)]):
    n = text.count(old)
    assert n == 1, f"edit {i+1}: expected 1 match, found {n}"
    text = text.replace(old, new, 1)
    print(f"edit {i+1}: applied")

if had_crlf:
    text = text.replace("\n", "\r\n")
open(P, "w", encoding="utf-8", newline="").write(text)
print(f"wrote {P} (crlf={had_crlf})")
