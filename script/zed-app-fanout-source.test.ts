import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const zedSource = read("crates/zed/src/zed.rs");
const workspaceSource = read("crates/workspace/src/workspace.rs");
const mainSource = read("crates/zed/src/main.rs");
const multiWorkspaceSource = read("crates/workspace/src/multi_workspace.rs");

const functionBody = (source: string, name: string) => {
  const start = source.indexOf(`fn ${name}(`);
  assert.ok(start >= 0, `expected ${name}`);

  const bodyStart = source.indexOf("{", start);
  assert.ok(bodyStart > start, `expected ${name} body`);

  let depth = 0;
  for (let index = bodyStart; index < source.length; index += 1) {
    const char = source[index];
    if (char === "{") {
      depth += 1;
    } else if (char === "}") {
      depth -= 1;
      if (depth === 0) {
        return source.slice(start, index + 1);
      }
    }
  }

  assert.fail(`expected ${name} body to close`);
};

const indexOfPattern = (body: string, pattern: string | RegExp) => {
  if (typeof pattern === "string") {
    return body.indexOf(pattern);
  }
  return body.match(pattern)?.index ?? -1;
};

const assertBefore = ({
  body,
  before,
  after,
  message,
}: {
  body: string;
  before: string | RegExp;
  after: string | RegExp;
  message: string;
}) => {
  const beforeIndex = indexOfPattern(body, before);
  const afterIndex = indexOfPattern(body, after);
  assert.ok(beforeIndex >= 0, `missing ${before}`);
  assert.ok(afterIndex >= 0, `missing ${after}`);
  assert.ok(beforeIndex < afterIndex, message);
};

test("quit fanout caps are named constants", () => {
  // The window cap is a caller-side concern: `zed::quit` still collects and
  // bounds its own window list before handing it to `workspace`.
  assert.match(zedSource, /const MAX_QUIT_WORKSPACE_WINDOWS: usize = 512;/);
  // The workspace and flush-task caps moved with the bounded collector into
  // `workspace::collect_flush_tasks`.
  assert.match(
    workspaceSource,
    /const MAX_QUIT_WORKSPACES_PER_WINDOW: usize = 512;/,
  );
  assert.match(
    workspaceSource,
    /const MAX_QUIT_SERIALIZATION_FLUSH_TASKS: usize = 8_192;/,
  );
  assert.match(mainSource, /const MAX_CHANNEL_NOTE_OPEN_PROMISES: usize = 256;/);
  assert.match(mainSource, /const MAX_DUMP_GPUI_ACTION_DEFINITIONS: usize = 16_384;/);
});

test("quit bounds window materialization before push loops", () => {
  const collectWindows = functionBody(zedSource, "collect_quit_workspace_windows");
  assertBefore({
    body: collectWindows,
    before: "workspace_windows.len() >= MAX_QUIT_WORKSPACE_WINDOWS",
    after: "workspace_windows.push(window);",
    message: "quit window fanout must check the cap before collecting windows",
  });
  assert.match(
    collectWindows,
    /log::warn!\([\s\S]*MAX_QUIT_WORKSPACE_WINDOWS/,
    "window truncation must be visible",
  );
  assert.match(
    collectWindows,
    /return None;/,
    "window truncation must cancel quit instead of skipping unchecked windows",
  );

  const quit = functionBody(zedSource, "quit");
  assert.match(
    quit,
    /let Some\(mut workspace_windows\)/,
    "quit must fail closed if window collection exceeds the cap",
  );
  assertBefore({
    body: quit,
    before: "collect_quit_workspace_windows(cx)",
    after: "workspace_windows.sort_by_key",
    message: "quit must cap workspace windows before sorting/display ordering",
  });
  assert.doesNotMatch(
    quit,
    /collect::<Vec<_>>\(\)/,
    "quit should not collect unbounded window or workspace vectors directly",
  );
  assert.match(
    quit,
    /workspace::prepare_windows_to_quit\(&workspace_windows, cx\)\.await/,
    "quit must delegate window preparation and serialization flush to `workspace`",
  );
});

test("collect_flush_tasks bounds workspace and flush-task fanout before extending", () => {
  const collectFlushTasks = functionBody(workspaceSource, "collect_flush_tasks");
  assert.match(
    collectFlushTasks,
    /-> \(Vec<Task<\(\)>>, bool\)/,
    "the collector must report truncation instead of silently dropping flush tasks",
  );
  assertBefore({
    body: collectFlushTasks,
    before: "workspace_count > MAX_QUIT_WORKSPACES_PER_WINDOW",
    after: "flush_tasks.extend(",
    message: "per-window workspace fanout must be capped before tasks are created",
  });
  assertBefore({
    body: collectFlushTasks,
    before: "flush_tasks.len().saturating_add(required_flush_tasks)",
    after: "flush_tasks.extend(",
    message: "flush task fanout must be capped before tasks are created",
  });
  assertBefore({
    body: collectFlushTasks,
    before: "pending_removal_task_count()",
    after: "flush_pending_serialization(window, cx)",
    message: "quit must check flush capacity before draining pending removal tasks",
  });
  assert.match(
    collectFlushTasks,
    /log::warn!\([\s\S]*MAX_QUIT_WORKSPACES_PER_WINDOW/,
    "workspace truncation must be visible",
  );
  assert.match(
    collectFlushTasks,
    /log::warn!\([\s\S]*MAX_QUIT_SERIALIZATION_FLUSH_TASKS/,
    "flush task truncation must be visible",
  );
  assert.match(
    collectFlushTasks,
    /return false;/,
    "overflow must cancel quit instead of skipping serialization tasks",
  );
  assertBefore({
    body: collectFlushTasks,
    before: "let mut truncated = false;",
    after: "break;",
    message: "the collector must stop at the first over-cap window",
  });

  const flushWindows = functionBody(workspaceSource, "flush_windows_serialization");
  assert.match(
    flushWindows,
    /\) -> bool \{/,
    "flush_windows_serialization must surface the truncation signal",
  );
  assert.match(
    flushWindows,
    /let \(flush_tasks, truncated\) = collect_flush_tasks/,
    "flush_windows_serialization must use the bounded collector",
  );
  assertBefore({
    body: flushWindows,
    before: "collect_flush_tasks(workspace_windows, cx)",
    after: "futures::future::join_all(flush_tasks).await",
    message: "flush tasks must be capped before join_all",
  });
  assertBefore({
    body: flushWindows,
    before: "futures::future::join_all(flush_tasks).await",
    after: "!truncated",
    message: "already-collected flush tasks must be awaited before cancelling",
  });

  const prepareToQuit = functionBody(workspaceSource, "prepare_windows_to_quit");
  assert.match(
    prepareToQuit,
    /if !flush_windows_serialization\(workspace_windows, cx\)\.await \{[\s\S]*?return false;/,
    "an over-cap serialization flush must cancel the quit",
  );

  const pendingRemovalTaskCount = functionBody(
    multiWorkspaceSource,
    "pending_removal_task_count",
  );
  assert.match(pendingRemovalTaskCount, /filter\(\|task\| !task\.is_ready\(\)\)/);
  assert.match(pendingRemovalTaskCount, /\.count\(\)/);
});

test("channel-note opens are capped before promise fanout and join_all", () => {
  const handleOpenRequest = functionBody(mainSource, "handle_open_request");
  const channelStart = handleOpenRequest.indexOf(
    "if !request.open_channel_notes.is_empty() || request.join_channel.is_some()",
  );
  assert.ok(channelStart >= 0, "expected channel-note branch");
  const channelEnd = handleOpenRequest.indexOf("} else if let Some(task) = task", channelStart);
  assert.ok(channelEnd > channelStart, "expected channel-note branch end");
  const channelBranch = handleOpenRequest.slice(channelStart, channelEnd);

  assert.match(
    channelBranch,
    /log::warn!\([\s\S]*MAX_CHANNEL_NOTE_OPEN_PROMISES/,
    "channel-note truncation must be visible",
  );
  assertBefore({
    body: channelBranch,
    before: "open_channel_notes.len() > MAX_CHANNEL_NOTE_OPEN_PROMISES",
    after: "let mut promises = Vec::with_capacity(",
    message: "channel notes must be checked before promise vector allocation",
  });
  assertBefore({
    body: channelBranch,
    before: ".take(MAX_CHANNEL_NOTE_OPEN_PROMISES)",
    after: "promises.push(",
    message: "channel-note promise fanout must be capped before push",
  });
  assertBefore({
    body: channelBranch,
    before: ".take(MAX_CHANNEL_NOTE_OPEN_PROMISES)",
    after: "future::join_all(promises).await",
    message: "channel-note promises must be capped before join_all",
  });
});

test("action definition dump is bounded before schema materialization and sort", () => {
  const dumpActions = functionBody(mainSource, "dump_all_gpui_actions");
  assertBefore({
    body: dumpActions,
    before: "actions.len() >= MAX_DUMP_GPUI_ACTION_DEFINITIONS",
    after: "let schema = (action.json_schema)(&mut generator)",
    message: "action dump must cap entries before schema generation",
  });
  assertBefore({
    body: dumpActions,
    before: "actions.len() >= MAX_DUMP_GPUI_ACTION_DEFINITIONS",
    after: "actions.push(ActionDef",
    message: "action dump must check the cap before collecting action definitions",
  });
  assertBefore({
    body: dumpActions,
    before: "actions.len() >= MAX_DUMP_GPUI_ACTION_DEFINITIONS",
    after: "actions.sort_by_key",
    message: "action definitions must be capped before sorting/display",
  });
  assert.match(
    dumpActions,
    /eprintln!\([\s\S]*MAX_DUMP_GPUI_ACTION_DEFINITIONS/,
    "action dump truncation must be visible before JSON is emitted",
  );
  assert.doesNotMatch(
    dumpActions,
    /collect::<Vec<ActionDef>>\(\)/,
    "action definitions should not be collected through an unbounded iterator",
  );
});
