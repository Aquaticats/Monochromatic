# Pi context-management plugin

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Decision record for the agentic context-management plugin wired into this Pi harness.

- **Context-fork answers**
  - Workload:
     personal or experimental.
  - Data sensitivity:
     conversation history may be processed by a third-party plugin;
     no special compliance constraints.
- **Decision**:
   do not use `npm:pi-context`.
   Removed from `~/.pi/agent/settings.json` on 2026-06-15.
- **Interaction note**:
   this repo also installs `/var/home/user/Monochromatic/package/pi-plugin/morph-compact`,
   which replaces Pi's default compaction.
   Any future context-management plugin must be checked against that package before installation.

Measurements and source reads in this file were taken on 2026-06-15
from the live repo and the cloned candidate repositories under `/tmp/agent`.

## Candidates surveyed

<table>
<thead>
<tr>
<th align="left">Candidate</th>
<th align="left">Source audited</th>
<th align="left">Validation run</th>
<th align="left">Nature</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">`pi-context` (`ttttmr/pi-context`)</td>
<td align="left">Yes, `/tmp/agent/pi-context-20260615`</td>
<td align="left">`npm run typecheck` passes; manual Pi CLI test only</td>
<td align="left">Third-party Pi package</td>
</tr>
<tr>
<td align="left">`pi-context-tools` (`theduke/pi-context-tools`)</td>
<td align="left">Yes, `/tmp/agent/pi-context-tools-20260615`</td>
<td align="left">`npm run typecheck` and `npm run lint` pass; tests are `echo &quot;No tests yet&quot;`</td>
<td align="left">Third-party Pi package</td>
</tr>
<tr>
<td align="left">`pi-boomerang` (`nicobailon/pi-boomerang`)</td>
<td align="left">Yes, `/tmp/agent/pi-boomerang-20260615`</td>
<td align="left">`npm run test` passes (176 vitest tests)</td>
<td align="left">Third-party Pi package</td>
</tr>
<tr>
<td align="left">Pi built-in auto-compaction and `/tree`</td>
<td align="left">N/A (core feature)</td>
<td align="left">Daily use in this repo</td>
<td align="left">Pi core, no plugin needed</td>
</tr>
</tbody>
</table>

## Source-audit findings for `pi-context`

### What it does

The plugin registers two slash commands and three tools:

- `/acm` enables agentic context management for the session and captures an `ExtensionCommandContext`.
- `/context` opens a TUI dashboard of token usage.
- `context_checkpoint` labels a session node.
- `context_timeline` renders a structural map of the active branch.
- `context_compact` creates a summarized continuation branch from a checkpoint.

Source files read:

- `src/index.ts` (484 lines):
   tool and command registration,
   tree traversal,
   compact orchestration.
- `src/context.ts` (159 lines):
   `/context` TUI dashboard.
- `src/utils.ts` (6 lines):
   token formatting helper.
- `skills/context-management/SKILL.md` and reference docs:
   prompt instructions for the agent.

### Code-quality observations

- **Module-level mutable state**:
   `CommandCtx`,
   `CompactParams` are single global variables in `src/index.ts:18-20`.
   Concurrent compactions or rapid session events could race.
   The code itself calls `ctx.abort()` on `turn_end` while a compact is pending,
   which limits concurrency in practice.
- **Deferred navigation via `setTimeout`**:
   `src/index.ts:465-500` defers `navigateTree` and `sendMessage`
   with a zero-timeout to work around `agent_end` firing
   before the agent is fully idle.
   This is fragile to Pi core timing changes.
- **Regex use is bounded**:
   `resolveTargetId` uses `/^[0-9a-f]{8,}$/i` only to decide whether a target looks like an ID;
   it does not extract data.
   Acceptable for this boundary.
- **Iterative tree walks**:
   the maintainer converted recursive DFS to iterative stack-based traversal
   after issue #19 (stack overflow on deep trees);
   the fix is in the current source.
- **No automated tests**:
   only `test/test.md`,
   a manual Pi CLI smoke test.
   No unit tests,
   no CI.
- **No license file**:
   `package.json` says MIT,
   but the repository contains no `LICENSE` file;
   GitHub API returns `licenseInfo: null`.
- **Version installed vs latest**:
   this Pi profile has `pi-context@1.1.4`;
   npm latest is `2.0.0-beta.0`.
   The 2.
  x line renames the tools from `context_tag`/`context_log`/`context_checkout`
  to `context_checkpoint`/`context_timeline`/`context_compact`.

### Validation run

```bash
cd /tmp/agent/pi-context-20260615
npm install
npm run typecheck   # passes
npm audit           # 2 vulnerabilities, transitive through @earendil-works/pi-coding-agent
```

The vulnerabilities are in `protobufjs` and `ws` pulled in by the Pi SDK dev dependency,
 not in the plugin's own code.

## Maintenance signals for `pi-context`

- Repository:
   `ttttmr/pi-context`,
   created 2026-02-08,
   205 stars,
   17 forks,
   not archived.
- Last pushed 2026-06-12;
   latest npm publish is `2.0.0-beta.0`.
- Single maintainer (`ttttmr`).
   No releases published on GitHub;
   distribution is via npm only.
- Issue responsiveness:
   maintainer replies to bug reports (issues #5,
   #12,
   #15,
   #21).
   Issue #19 was fixed by an external contributor PR (#20) that was merged within a day.
- Stale dependabot PRs are present (e.g. #4,
   #8,
   #13),
   but the maintainer closed/superseded several in a batch on 2026-05-10.
- No CI,
   no automated test runs,
   no coverage report,
   no fuzzing or mutation testing evidence.

State:
 **active individual maintainer,
 responsive to bugs,
 but immature tooling** (no CI/tests).

## Alternatives evaluated and rejected

### `pi-context-tools` (`theduke/pi-context-tools`)

What it exposes:
 `context_info` and `compact_context` only.

Rejection reason:
 it does not let the active model *manage* context in the agentic sense.
 There are no checkpoints,
 no tags,
 no timeline inspection,
 and no navigation to a previous state.
 It is a thinner "compact on demand" helper.
 For a workload that wants the model to decide when to checkpoint and branch,
 this is insufficient.

Validation:
 `npm run typecheck` and `npm run lint` pass;
 has CI via GitHub Actions;
 no tests;
 has a proper `LICENSE` file.

### `pi-boomerang` (`nicobailon/pi-boomerang`)

What it exposes:
 autonomous task execution with automatic context collapse via `/boomerang`,
 templates,
 chains,
 rethrow loops,
 and an auto-boomerang mode.

Rejection reason:
 it is a task-automation layer,
 not a general context-management layer.
 The model does not get fine-grained checkpoint/timeline/compact tools;
 instead the plugin drives an autonomous sub-session and collapses it.
 That is powerful but solves a different problem.
 It also injects a large system-prompt override (`BOOMERANG MODE ACTIVE`) and manipulates model/thinking levels,
 which is heavier than this use case needs.

Validation:
 `npm run test` passes 176 vitest tests;
 has a changelog;
 more mature testing than `pi-context`.

### Pi built-in auto-compaction and `/tree`

Rejection reason:
 Pi already auto-compacts and lets the user navigate the session tree,
 but the *model* cannot proactively checkpoint or compact without a plugin.
 The user explicitly wants the active model to manage its own context,
 so the built-in tooling alone does not satisfy the requirement.

## Security and abuse vetting

`pi-context` is not a SaaS;
 the six vendor-vetting layers do not apply.
 Relevant security observations:

- The plugin runs inside the Pi extension host with access to `SessionManager`,
   `ExtensionCommandContext`,
   and UI APIs.
   It can read the full conversation history and rewrite it via `branchWithSummary`/`navigateTree`.
- No network calls in the source.
   No telemetry or external exfiltration observed.
- No npm install scripts in the published package.
- The skill prompt is opinionated and can push the model to compact aggressively;
   the backup-tag mechanism mitigates data loss.

## Recommendation

Reject `npm:pi-context` for this Pi profile:

1. **Capability mismatch**:
    the plugin only supports checkpoint,
    timeline inspection,
    and branch-level compacting.
    It cannot prune specific messages or line ranges,
    which limits its usefulness for precise context hygiene.
2. **Maturity gaps**:
    no automated tests,
    no CI,
    no repository `LICENSE` file,
    and a single maintainer.
3. **Interaction risk**:
    this profile already installs `@monochromatic-dev/pi-plugin-morph-compact`,
    a local compaction replacement.
    Running two compaction-touching plugins together is untested and risks session-tree corruption.
4. **Prefer alternatives if the need returns**:
    for simple on-demand compaction use `pi-context-tools`;
    for autonomous task collapse use `pi-boomerang`;
    for agentic checkpoints wait for Pi core support or a plugin with message-level pruning and proper test coverage.

Decision owner and date:
 AI agent,
 2026-06-15.
