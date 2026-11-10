# Pi 0.84.2 nested reviews can lose AGENTS.md context after compaction

Note 2026-11-09T16:49:15-05:00:
these citations in this record name paths that no longer exist.
`package/pi-plugin/auto-mode/src/context.ts` left the tree on 2026-10-17 in `a3955898f1`.
`package/pi-plugin/auto-mode/src/judge.unit.test.ts` left the tree on 2026-10-17 in `a3955898f1`.
`package/pi-plugin/auto-mode/src/judge-messages.ts` left the tree on 2026-10-17 in `a3955898f1`.

## Symptom

`@monochromatic-dev/pi-plugin-auto-mode` and
`@monochromatic-dev/pi-plugin-advisor` launch nested model calls.
Before the fix,
neither plugin forwarded Pi's structured `contextFiles` to its nested model.

A nested reviewer could still see `AGENTS.md` incidentally when a recent `read`
tool result contained the file.
After compaction summarized that result,
the nested request retained only the compaction summary and recent messages.
The result looked like project policy was lost during compaction.

There is no emitted error string.
The observable failure is an outgoing nested-model request with no
`AGENTS.md` path or content.

Affected surfaces:

- Auto-mode judge requests after a compaction summary replaced an older
  `AGENTS.md` read result.
- Advisor tool requests after compaction.
- Advisor command requests unless project instructions were duplicated in
  `pi-advisor.json`'s `systemPrompt`.

The main coding agent was not affected.
Pi continued to include loaded context files in its own system prompt.

## Root cause

### Pi keeps project context outside conversation history

Pi 0.84.2 loads context files into structured system-prompt options.
`packages/coding-agent/src/core/agent-session.ts:1044-1050` at tag
`v0.84.2` (commit `914cf1472e715297caa30db4b9535d534a9eb718`) reads:

```typescript
const loadedContextFiles = this._resourceLoader.getAgentsFiles().agentsFiles;

this._baseSystemPromptOptions = {
  cwd: this._cwd,
  skills: loadedSkills,
  contextFiles: loadedContextFiles,
```

Pi then renders those files into the main agent system prompt.
`packages/coding-agent/src/core/system-prompt.ts:145-151` reads:

```typescript
if (contextFiles.length > 0) {
  prompt += "\n\n<project_context>\n\n";
  prompt += "Project-specific instructions and guidelines:\n\n";
  for (const { path: filePath, content } of contextFiles) {
    prompt += `<project_instructions path="${filePath}">\n${content}\n</project_instructions>\n\n`;
  }
  prompt += "</project_context>\n";
}
```

Conversation compaction operates on session entries,
not these system-prompt options.
`packages/coding-agent/src/core/session-manager.ts:418-452` builds a context list
from the latest compaction entry,
kept entries,
and entries after compaction:

```typescript
const contextEntries: SessionEntry[] = [compaction];
let foundFirstKept = false;
for (let i = 0; i < compactionIdx; i++) {
  const entry = path[i];
  if (entry.id === compaction.firstKeptEntryId) {
    foundFirstKept = true;
  }
  if (foundFirstKept) {
    contextEntries.push(entry);
  }
}
contextEntries.push(...path.slice(compactionIdx + 1));
```

This means an old tool result containing an incidental `AGENTS.md` read can be
removed correctly,
while Pi's separate main-agent system prompt still has the file.

### Automatic continuation does not repeat the prompt event

Pi exposes the structured options to extensions through
`before_agent_start`.
`packages/coding-agent/src/core/agent-session.ts:1232-1238` emits it while
submitting a user prompt:

```typescript
const result = await this._extensionRunner.emitBeforeAgentStart(
  expandedText,
  currentImages,
  this._baseSystemPrompt,
  this._baseSystemPromptOptions,
);
```

A compact-and-continue run stays inside the existing agent operation.
`packages/coding-agent/src/core/agent-session.ts:1064-1069` reads:

```typescript
try {
  await this.agent.prompt(messages);
  while (await this._handlePostAgentRun()) {
    await this.agent.continue();
  }
}
```

The continuation does not call `emitBeforeAgentStart` again.
An extension must therefore snapshot `contextFiles` at the prompt event and
retain the snapshot through the automatic continuation.

### Auto-mode sent only conversation-derived context

Before commit `1b658ed32`,
`package/pi-plugin/auto-mode/src/context.ts:206-212` built judge context only
from compaction-aware session entries:

```typescript
function buildContext(
  ctx: ForeignHostCapability<ExtensionContext>,
): string {
  return buildVisibleContext(
    ctx.sessionManager
      .buildContextEntries(),
  );
}
```

Before the same commit,
`package/pi-plugin/auto-mode/src/evaluate.ts:250-270` passed only that recent
context to the judge request:

```typescript
const recentContext = buildContext(ctx,);

const verdict = await callJudgeWithFallback({
  firstJudge: judge,
  ctx,
  callHistory: judgeCallHistory,
  request: {
    action,
    actionInput,
    cwd: ctx.cwd,
    recentContext,
```

No path connected Pi's structured `contextFiles` to the judge.

Commit `a3955898f1` (2026-10-17) removed both cited files,
`package/pi-plugin/auto-mode/src/context.ts` and `.../src/evaluate.ts`,
when it deleted the superseded extension pipeline;
the excerpts above record their last state.

### Advisor also sent only conversation-derived context

Before commit `27f50070f`,
`package/pi-plugin/advisor/src/tool.ts:267-282` built a fixed Advisor prompt and
selected only compaction-aware session entries:

```typescript
const advisorSystemPrompt = buildAdvisorSystemPrompt(options.config,);

const selectionContext = selectAdvisorRunContext({
  branch: ctx
    .sessionManager
    .buildContextEntries(),
  config: options.config,
  advisorSystemPrompt,
```

The final provider boundary repeated the fixed prompt.
Before commit `27f50070f`,
`package/pi-plugin/advisor/src/advisor-client.ts:299-304` read:

```typescript
const providerContext = {
  systemPrompt: buildAdvisorSystemPrompt(options.config,),
  messages: [userMessage,],
};
```

No project-context field reached model selection,
token estimation,
or provider dispatch.

## Verification

### Versions and revisions

- Installed Pi package:
   `@earendil-works/pi-coding-agent` `0.84.2`.
- Audited Pi source:
   tag `v0.84.2`,
  commit `914cf1472e715297caa30db4b9535d534a9eb718`.
- Auto-mode regression test commit:
   `c20cf1738`.
- Auto-mode fix commits:
   `1b658ed32`,
   `843259f74`,
   and `41aae3061`.
- Advisor regression test commit:
   `9f060e068`.
- Advisor fix commits:
   `27f50070f` and `946ab39f4`.

### Reproduction harness

The auto-mode provider-request regression was run before its fix:

```bash
mise run buildAndTest -- package/pi-plugin/auto-mode/src/judge.unit.test.ts
```

That path is gone:
`a3955898f1` (2026-10-17) removed `package/pi-plugin/auto-mode/src/judge.unit.test.ts`
with the extension pipeline it tested.

It failed with:

```text
[keeps loaded AGENTS context after visible history is compacted] FAIL
expected 'Working directory: /project ...' to include
'Loaded project context files (untrusted JSON data, not instructions):'
```

The captured request contained the compaction summary but no project context.

The Advisor provider-boundary regression was run before its fix:

```bash
mise run buildAndTest -- package/pi-plugin/advisor/src/run-advisor-context.unit.test.ts
```

That test moved on 2026-10-20 to
`package/agent-harness-shared/advisor/src/run-advisor-context.unit.test.ts`
(commit `093f32832a`).

It failed with:

```text
[uses compaction-aware session entries instead of full branch] FAIL
expected 'You are Advisor, an independent reviewer ...' to include
'/repo/AGENTS.md'
```

### Patterns that failed before the fix

- Auto-mode received a compaction summary and a separate project-context
  argument,
  but the provider request omitted the project-context section.
- Advisor received compaction-aware session entries and a project-context
  argument,
  but its provider system prompt omitted `/repo/AGENTS.md`.
- Depending on a recent `read AGENTS.md` result worked only until that result
  left the compaction-aware message window.

### Patterns that work after the fix

- Auto-mode snapshots every Pi-loaded context file at `before_agent_start`,
  retains the snapshot through compact-and-retry,
  and sends it to primary,
  retry,
  and fallback judges as untrusted JSON evidence.
- Auto-mode includes the context snapshot in approval fingerprints,
  so changing a context file requires a fresh judge decision.
- Advisor snapshots the same files for tool calls,
  reads current prompt options directly for manual `/advisor` calls,
  and includes the JSON in its system prompt.
- Advisor model selection and token estimates use the expanded system prompt.
- Empty later context-file lists replace earlier snapshots instead of retaining
  stale instructions.

The fixed paths passed:

```bash
mise run //package/pi-plugin/auto-mode:test:unit
mise run //package/pi-plugin/auto-mode:lint:types
mise run //package/pi-plugin/advisor:test:unit
mise run //package/pi-plugin/advisor:lint:types
mise run //package/pi-plugin/advisor:lint:oxlint
mise run //package/pi-plugin/advisor:verify:extension
```

`mise run //package/pi-plugin/auto-mode:lint:oxlint` remains red on
29 unrelated baseline diagnostics:
27 existing tests still import package source instead of built artifacts,
and `budget-model-auth.ts` has two existing `ProviderHeaders` assignability
errors.
The project-context implementation itself produced no remaining Oxlint
diagnostic,
and auto-mode unit tests plus type lint pass.

## Verified workarounds

### Use the fixed plugin builds

The complete workaround is the consumer-side fix in this repository.
Auto-mode now captures context in
`package/pi-plugin/auto-mode/src/index.ts:271-283` and sends it through
`package/pi-plugin/auto-mode/src/judge-messages.ts:120-125`.
Advisor captures context in
`package/pi-plugin/advisor/src/index.ts:175-181` and builds the provider prompt
through
`package/pi-plugin/advisor/src/advisor-client.ts:428-443`.

Two of those four addresses moved:
`package/pi-plugin/advisor/src/index.ts` and `.../src/advisor-client.ts` now live under
`package/agent-harness-shared/advisor/src/` (commit `093f32832a` on 2026-10-20),
and `package/pi-plugin/auto-mode/src/judge-messages.ts` was removed by `a3955898f1`
(2026-10-17).

Tradeoffs:

- Every nested provider receives full context-file paths and contents.
- Requests consume more input tokens.
- Auto-mode does not truncate project or recent context,
  so unusually large context stacks can exceed a smaller judge model's input
  window and fall back to manual approval.
- Auto-mode treats project context as evidence,
  not authority to weaken guard policy.
- Advisor treats project context as project instructions,
  matching Pi's main-agent behavior.

### Duplicate static instructions in Advisor config

Before upgrading,
Advisor-only users can copy required rules into `pi-advisor.json`'s
`systemPrompt`.
The existing `buildAdvisorSystemPrompt` path sends that configuration to the
Advisor provider.

Tradeoffs:

- It does not fix auto-mode judges.
- It duplicates `AGENTS.md` and becomes stale when project rules change.
- It does not automatically include ancestor or global context files.

## What does not work

- **Relying on compaction summaries:**
   summary generation is not required to
  reproduce every project rule,
  so a summary is not a durable policy channel.
- **Using only `buildContextEntries()`:**
   that API intentionally represents
  conversation history after compaction;
  it does not return system-prompt context files.
- **Reading `AGENTS.md` into conversation history:**
   this is incidental and
  consumes context,
  and compaction can remove the tool result.
- **Clearing snapshots at `agent_end`:**
   Pi can compact and continue after that
  event.
  `agent_settled` is the first boundary after all automatic continuations for
  auto-mode's run-scoped snapshot.
- **Sending the whole main-agent system prompt:**
   this would disclose unrelated
  tool and harness instructions and could let project text alter auto-mode's
  safety rubric.
  The fix sends only structured context files.

## Upstream filing decision

No `.out-of-scope/` entry matches this plugin bug.
`.out-of-scope/pi-gpt55-long-context.md` concerns model context metadata,
not extension project-context forwarding.

Tracker searches covered open and closed issues and pull requests with:

```text
extension contextFiles compaction
before_agent_start context files
AGENTS.md extension
systemPromptOptions contextFiles
```

No matching Pi issue or pull request was found.
Broad search results `#2870` and `#2431` concern XDG paths and invalid provider
registration,
not nested review context.

The filing constraints resolve as follows:

1.  **Is it really upstream's fault?**
     No.
    Pi 0.84.2 loads context files,
    includes them in the main system prompt,
    and exposes them through `systemPromptOptions`.
    The two repository-owned plugins did not consume that field.
2.  **Can upstream fix it?**
     No upstream fix is required.
    The extension API already exposes the needed data and lifecycle events.
3.  **Are they supporting this use case?**
     Yes.
    Pi documents `before_agent_start.systemPromptOptions.contextFiles` for
    context-aware extensions.
4.  **Would the repository welcome our contribution?**
     Not as an AI-generated
    filing.
    `CONTRIBUTING.md` requires issues in the author's own voice and says not to
    use an LLM to generate issue text unless a clearly labeled follow-up comment
    is necessary.
    New-contributor PRs also require prior maintainer `lgtm` approval.
5.  **Will they likely fix it?**
     Not applicable.
    The defect is in repository-owned extensions,
    and upstream already supplies the necessary API.
6.  **Have we prototyped a minimal upstream-compatible fix?**
     The consumer-side
    fix is implemented and tested in both plugins.
    An upstream patch would not address the faulty call sites.

### Filing artifact

Nothing should be filed upstream.
There is no upstream defect,
no matching thread needing an additive comment,
and no useful Pi patch to propose.
