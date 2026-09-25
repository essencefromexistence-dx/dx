# -*- coding: utf-8 -*-
"""Resolve the 4 content conflicts of the f8adffe9 sync.

Policy: a genuine semantic merge — ours' structure is the surface, upstream's
semantic change is PORTED onto it.  Each file's upstream delta was read first
(`git diff baee1ca6de..upstream/main -- <file>`), so nothing here is guessed.

⚠️ The worktree files are **CRLF** (`git ls-files --eol` says `w/crlf`), so every
marker match and every inserted body must use the file's own EOL.  Matching on
`'<<<<<<< HEAD\\n'` silently finds nothing.
"""
import re

def load(p):
    return open(p, encoding='utf-8', newline='').read()

def save(p, s):
    open(p, 'w', encoding='utf-8', newline='').write(s)

def eol_of(s):
    return '\r\n' if '\r\n' in s[:4000] else '\n'

def count_conflicts(s):
    return len(re.findall(r'^<<<<<<< HEAD\r?$', s, re.M))

def replace_conflict(src, nth, body, eol):
    m = None
    for i, m in enumerate(re.finditer(r'^<<<<<<< HEAD\r?$', src, re.M), 1):
        if i == nth:
            break
    end = src.index('>>>>>>> upstream/main', m.start())
    end = src.index(eol, end) + len(eol)
    return src[:m.start()] + body.replace('\n', eol) + src[end:]

report = []

# ---------------------------------------------------------------- 1. thread_view
P = 'crates/agent_ui/src/conversation_view/thread_view.rs'
s = load(P); eol = eol_of(s)
assert count_conflicts(s) == 2, "thread_view conflicts=%d" % count_conflicts(s)
s = replace_conflict(s, 1, """    agent_thread_www_preview::{
        AgentThreadCenterSurface, agent_thread_www_preview_hooks,
        is_agent_www_tool_preview_enabled, placeholder_preview_html_data_url,
        preview_url_for_agent_www_tool,
    },
    dx_agent_bridge::DxConfiguredPluginSummary,
""", eol)
s = replace_conflict(s, 1, """    AcpThreadEvent, ContentBlock, Elicitation, ElicitationEntryId, ElicitationStatus,
    SandboxAuthorizationDetails, SandboxFallbackAuthorizationDetails, SandboxNotAppliedReason,
    decode_path_escapes,
""", eol)
assert count_conflicts(s) == 0, "thread_view markers left"
save(P, s)
report.append(("thread_view.rs", "2 hunks: ours' imports minus the 2 names upstream's removal orphaned"))

# --------------------------------------------------------------- 2. project_panel
P = 'crates/project_panel/src/project_panel.rs'
s = load(P); eol = eol_of(s)
assert count_conflicts(s) == 3, "project_panel conflicts=%d" % count_conflicts(s)
s = replace_conflict(s, 1, """        if entry.is_dir() && !push_project_panel_expanded_dir(expanded_dir_ids, entry_id) {
            return;
""", eol)
for _ in range(2):
    s = replace_conflict(s, 1, """                        if entry.is_dir()
                            && !push_project_panel_expanded_dir(expanded_dir_ids, entry.id)
                        {
                            break;
""", eol)
assert count_conflicts(s) == 0, "project_panel markers left"
save(P, s)
report.append(("project_panel.rs", "3 hunks: ours' capped helper + upstream's `entry.is_dir()` guard"))

# --------------------------------------------------------------------- 3. sidebar
P = 'crates/sidebar/src/sidebar.rs'
s = load(P); eol = eol_of(s)
assert count_conflicts(s) == 1, "sidebar conflicts=%d" % count_conflicts(s)
s = replace_conflict(s, 1, "        v_flex()\n", eol)
old = "                        Decorations::Client { .. } => header.mt(px(-1.)),".replace('\n', eol)
new = """                        // Without projects there's no bottom border to match the title bar's.
                        Decorations::Client { .. } => apply_title_bar_insets(
                            header,
                            left_window_controls,
                            right_window_controls,
                            no_open_projects,
                        ),""".replace('\n', eol)
assert s.count(old) == 1, "sidebar nested anchor=%d" % s.count(old)
s = s.replace(old, new)
assert count_conflicts(s) == 0
save(P, s)
report.append(("sidebar.rs", "1 hunk: ours' `v_flex()` + upstream's #64381 insets ported into the nested block"))

# ------------------------------------------------------------------- 4. workspace
P = 'crates/workspace/src/workspace.rs'
s = load(P); eol = eol_of(s)
assert count_conflicts(s) == 2, "workspace conflicts=%d" % count_conflicts(s)
s = replace_conflict(s, 1, """    // dx fork: upstream dropped the `border_radius_tiling` parameter from this wrapper (their
    // callers pass three arguments). We keep our two-tiling implementation behind
    // `client_side_decorations_with_content_flush` and delegate here with `Tiling::default()`,
    // which reproduces upstream's behaviour exactly. Our own call sites that need the flush
    // behaviour call `client_side_decorations_with_content_flush` directly.
    client_side_decorations_with_content_flush(
        element,
        window,
        cx,
        Tiling::default(),
        Tiling::default(),
    )
}

/// Add client-side decorations while allowing selected content edges to fill
/// the window decoration gutter.
pub fn client_side_decorations_with_content_flush(
    element: impl IntoElement,
    window: &mut Window,
    cx: &mut App,
    border_radius_tiling: Tiling,
    content_flush_tiling: Tiling,
) -> Stateful<Div> {
""", eol)
s = replace_conflict(s, 1, """                        .rounded_client_corners(corner_tiling)
                        .when(!content_tiling.top, |div| {
                            div.border_t(theme::CLIENT_SIDE_DECORATION_BORDER)
                        })
                        .when(!content_tiling.bottom, |div| {
                            div.border_b(theme::CLIENT_SIDE_DECORATION_BORDER)
                        })
                        .when(!content_tiling.left, |div| {
                            div.border_l(theme::CLIENT_SIDE_DECORATION_BORDER)
                        })
                        .when(!content_tiling.right, |div| {
                            div.border_r(theme::CLIENT_SIDE_DECORATION_BORDER)
                        })
""", eol)
assert count_conflicts(s) == 0, "workspace markers left"
save(P, s)
report.append(("workspace.rs", "2 hunks: ours' split kept, const dropped, theme constant adopted"))

print("RESOLVED:")
for name, what in report:
    print("  %-24s %s" % (name, what))
