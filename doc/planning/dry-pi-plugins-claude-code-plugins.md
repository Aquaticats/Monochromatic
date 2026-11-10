# Plan: DRY `pi-plugins` and `claude-code-plugin`

Status:
 plan complete,
 grilled,
 all decisions resolved,
 and implementation executed.
 Sequencing completed as E2b,
 E1,
 E2a,
 E3 with per-sub-step commits.

## Goal

Remove duplicated logic between `package/pi-plugin/*` and
`package/claude-code-plugin/*` by lifting genuinely shared agent-harness logic
into `package/agent-harness-shared/*`,
following the pattern already proven there three times.
Host-specific adapters and protocol types stay in their host clusters.

## Baseline policy (already decided)

`doc/planning/package-category-rebalance.md` sets the governing rule for these
two clusters:

> `statusline` and `terminal-title` each exist under both `pi/` and
> `claude-code-plugin/`,
>  and `spawn` appears as `pi/spawn` and
> `claude-code-plugin/claude-spawn`.
>  They share a concept but no code,
>  and they
> bind to different host APIs,
>  so each stays under its host cluster.
>  Extract an
> `agent-harness-shared/` package only if real shared agent-harness logic
> emerges,
>  as it did for `current-time-context`.

The bar is **real shared agent-harness logic emerged**,
 not "same concept."
Two host clusters with different protocols is the expected default,
 not a
defect to fix:

- pi plugin:
  in-process event emitter.
  Entry is `export default async function(pi: ExtensionAPI)`,
   registers
  `pi.on('tool_call', (event, ctx) => ...)`,
   returns pi-shaped decisions.
  Types come from `@earendil-works/pi-coding-agent`.
- Claude Code plugin:
  subprocess stdin/stdout JSON.
  Entry is a `#!/usr/bin/env node` shim calling
  `runHookPlugin({ parser, handler, writer })` from
  `package/claude-code-plugin/source/src/runtime/handler-runtime.ts`.
  Types come from `@monochromatic-dev/claude-code-plugin-hook-type`.

## Current state inventory

The DRY vehicle already exists at `package/agent-harness-shared/` with three
shared packages,
 each consumed by both clusters:

- `agent-harness-shared/terminal-title`
  (`@monochromatic-dev/agent-harness-shared-terminal-title`):
  consumed by `package/pi-plugin/terminal-title` and
  `package/claude-code-plugin/source`.
  Both sides are reduced to a host adapter table mapping tool names
  (`bash`/`Bash`,
  `read`/`Read`) to shared `ToolTitleEntry` builders.
  DRY complete.
- `agent-harness-shared/current-time-context`
  (`@monochromatic-dev/agent-harness-shared-current-time-context`):
  consumed by `package/pi-plugin/current-time-context` (a thin
  `pi.on('before_agent_start')` wrapper) and
  `package/claude-code-plugin/source` (the `prompt-time` handler re-exports
  the same `formatTimeContext`).
  DRY complete.
- `agent-harness-shared/shell-command-analyzer`
  (`@monochromatic-dev/agent-harness-shared-shell-command-analyzer`):
  consumed by `package/pi-plugin/guardrail`,
  `package/pi-plugin/auto-mode`,
   and
  `package/claude-code-plugin/source` (guardrail).
  Owns `analyzeShellCommand`,
  `extractParamRefs`,
  `looksLikePath`.
  Shell-command parsing DRY complete.

Established pattern,
 proven three times:
shared pure-logic module in `agent-harness-shared/` plus thin host adapters
that own the protocol boundary (event shape in,
 decision shape out) and the
tool-name or input-field vocabulary.

## Per-family assessment

The complete set of concepts that appear under both clusters:
`guardrail`,
`spawn` versus `claude-spawn`,
`statusline`,
`terminal-title`,
 and
`current-time-context` (pi) versus `prompt-time` (claude).
No other pi-plugins package shares real logic with a claude-code-plugin package
(advisor,
 auto-mode,
 morph-compact,
 search-fetch,
 thinking-defaults are pi-only;
correction-reminder,
 stop-reminders,
 session-start-housekeeping,
bash-output-filter,
 research-agent are claude-only).

### terminal-title

DRY complete.
Nothing to do.
Reference implementation of the shared-module-plus-host-adapter pattern that E1,
 E2,
 and E3 extend.

### current-time-context (pi) versus prompt-time (claude)

DRY complete.
Shared `formatTimeContext`;
 both hosts are thin adapters.
Nothing to do.

### guardrail

Parsing already shared via `agent-harnesses-shell-command-analyzer`.
One verbatim duplicate remains:
`invokesBunTest` is identical in
`package/pi-plugin/guardrail/src/bash-guard.ts` and
`package/claude-code-plugin/source/src/handlers/guardrail.ts`,
same JSDoc,
 same
`info.name === 'bun' && info.args[0] === 'test'` predicate over
`analyzeShellCommand` output.
The deny prose is near-identical;
 only the output shape differs
(pi returns `{ block, reason }`;
claude returns `PreToolUseOutput.hookSpecificOutput.permissionDecision`).
The Agent-resume block (claude only) and the gitignore path-guard (pi only) are
genuinely host-specific and stay put.

### spawn versus claude-spawn

Not yet shared.
This is the biggest opportunity.
`session-finder.ts` is structurally near-identical on both sides:
`SESSION_NOT_FOUND` sentinel,
`readParentPid`,
`readPidMapping`,
`walkProcessTreeFrom`,
`readByPidDir`,
`findByMostRecent`,
`findCallingSession`,
the same procfs-walk,
 PID-mapping,
 newest-candidate mechanism.
Both files even import `splitWhitespace` from their own `text-scan`.
Only the `PidMapping` payload shape (pi session path versus claude session path)
and a couple of signatures differ.
`text-scan`:
`package/pi-plugin/spawn/src/text-scan.ts` is an 85-line subset
(`isWhitespace` plus `splitWhitespace`) of
`package/claude-code-plugin/source/src/lib/text-scan.ts` (535 lines,
used by five handlers).
pi cannot import from `claude-code-plugin/source` without an inverted
cross-cluster dependency,
 so this is a genuine lift candidate,
 not a
re-export.

### statusline

Conceptual overlap only;
 the shared kernel is narrow.
The two sides parse different input shapes:
pi reads HTTP response headers from the `after_provider_response` event into a
generic `RateLimitSnapshot` with `usedPercent`,
`windowSeconds`,
`paceScale`,
`sampledAtMs` (`package/pi-plugin/statusline/src/rate-limit-parse-helpers.ts`);
claude reads JSON `rate_limits.five_hour` and `rate_limits.seven_day` tiers and
derives `elapsed = windowSeconds - (resets_at - now)` at render time
(`package/claude-code-plugin/statusline/src/statusline.ts`).
`package/pi-plugin/statusline/README.md` already states it "ports only the
projected-overflow warning behavior from `claude-code-plugin/statusline`."
The genuinely shared kernel is the projected-overrun model,
 the `->N%` marker,
relative-time formatting,
 and threshold constants.
The arithmetic has diverged (pace and sampled-at versus derived-elapsed),
 so
unifying needs a design decision before any extraction.

## Recommended extractions

Each extraction follows the additive-first discipline recorded in
`doc/planning/extract-refactor-guardrail.md`:
land the shared code as an additive,
 verified,
 committed addition;
migrate each consumer leaving the tree green;
delete the old duplicate only once no consumer references it.
Verify with the byte-equal stdout fixtures method documented in
`package/claude-code-plugin/README.md` (capture baseline
`dist/final/node/index.mjs` fixtures covering every decision path,
 replay
through the new entry,
 require byte-equal stdout).

### E1: fold `invokesBunTest` into `shell-command-analyzer`

This adds no new package,
 but first renames the existing package to fix its
known naming mistake.
The path stays `package/agent-harness-shared/shell-command-analyzer`;
 the
package name becomes
`@monochromatic-dev/agent-harness-shared-shell-command-analyzer`.
`invokesBunTest` is a predicate over `analyzeShellCommand` output,
 so it belongs
alongside `extractParamRefs` and `looksLikePath` in that existing shared package,
which is already a dependency of both guardrails.

Steps:

1.  Rename the package in
    `package/agent-harness-shared/shell-command-analyzer/package.json` and all
    workspace dependency/import sites from
    `@monochromatic-dev/agent-harnesses-shell-command-analyzer` to
    `@monochromatic-dev/agent-harness-shared-shell-command-analyzer`.
    Run the affected package type checks and commit the pure rename separately.
2.  Add `invokesBunTest(command)` and `BUN_TEST_BAN_REASON` to
    `package/agent-harness-shared/shell-command-analyzer/src/`,
     exported from
    its `src/index.ts`,
     with unit tests.
3.  Commit as a standalone,
     verified,
     green-tree addition.
4.  Migrate `package/pi-plugin/guardrail/src/bash-guard.ts` to import
    `invokesBunTest` and `BUN_TEST_BAN_REASON` from the shared package;
     delete
    the local predicate and local ban-reason constant.
    Run `mise run //package/pi-plugin/guardrail:lint:types` and the guardrail
    tests.
5.  Migrate
    `package/claude-code-plugin/source/src/handlers/guardrail.ts` the same way;
    rebuild and replay baseline fixtures for byte-equal stdout.
6.  Commit each migration separately,
     tree green at every commit.

Optional in the original plan,
 now decided:
the deny prose is byte-identical across both hosts (the same seven-line string
in `package/pi-plugin/guardrail/src/constants.ts` as `BUN_TEST_BLOCK_REASON`
and in `package/claude-code-plugin/source/src/handlers/guardrail.ts` as
`BUN_TEST_DENY_OUTPUT.permissionDecisionReason`).
 Dedup it.
Co-locate the reason with the predicate that detects the violation:
export a `BUN_TEST_BAN_REASON` constant from
`@monochromatic-dev/agent-harness-shared-shell-command-analyzer` alongside
`invokesBunTest`.
Both guardrails import the predicate and the reason as a pair.
Each host still wraps the reason in its own protocol-specific deny shape
(`{ block, reason }` for pi;
`PreToolUseOutput.hookSpecificOutput.permissionDecision`
for claude);
 only the reason string is shared.
The Agent-resume deny (claude only) and path-guard message (pi only) stay
host-specific since they have no counterpart.

Net:
removes one verbatim duplicate plus one byte-identical prose duplicate with no
new package.

### E2: lift spawn session-discovery and text-scan primitives

Highest payoff.
Two related lifts.

### E2a: `agent-harness-shared/session-discovery`

Decision:
extract,
 with a generic interface.
Path name:
`session-discovery`,
 not `spawn-session-finder`,
 because the concept is
"resolve the calling agent session,"
 which spawn consumes but does not own.
Package name:
`@monochromatic-dev/agent-harness-shared-session-discovery`.

The mechanism (procfs parent walk,
`.by-pid/` directory scan,
 newest-mtime
fallback,
 tree-then-fallback composition) is deep and identical across both
hosts;
 the payload and config are host-specific and shallow.
 That is the
codebase-design deep-module shape:
 shared deep mechanism,
 host supplies shallow
adapters.

Generic interface:

```ts
// package/agent-harness-shared/session-discovery
type SessionDiscoveryIo = {
  readonly readParentPid?: (pid: number) => Promise<number | typeof SESSION_NOT_FOUND>;
  readonly readDir?: (path: string) => Promise<readonly string[]>;
  readonly readFile?: (path: string) => Promise<string>;
  readonly statFile?: (path: string) => Promise<{ readonly mtimeMs: number }>;
};

async function findCallingSession<TMapping>(
  opts: {
    readonly byPidDir: string;                        // host resolves from its own paths/env
    readonly io?: SessionDiscoveryIo;                 // tests inject fake procfs/filesystem seams
    readonly parseMapping: (raw: string) => TMapping; // host owns the JSON shape
    readonly startPid: number;                        // usually process.ppid
  },
): Promise<TMapping | typeof SESSION_NOT_FOUND>
```

The shared package owns the default procfs parent-PID reader,
 the process-tree
walk,
 the `.by-pid` directory read,
 the newest-candidate scan,
 and the
 tree-then-fallback composition.
Production callers omit `io`;
 tests inject fake parent-PID and filesystem readers
so every branch runs against throwaway fixtures instead of real procfs or real
coordination state.
Each host keeps a `paths.ts` that resolves `byPidDir` (pi injects `env`;
 claude
reads `process.env.HOME`) and a `PidMapping` type that the shared package is
generic over.
 Only `sessionId` is common to both;
 host-specific fields stay in the
host `PidMapping`.

Reconcile the four divergences found during verification:
1.  Control flow:
     pi walks iteratively,
     claude recurses.
     Both are bounded and
    defensible under AGENTS.md ITR.
     Pick one in the shared package;
     iterative
    avoids recursion-over-flat-array concerns.
2.  Read concurrency:
     adopt claude's `Promise.all([stat, readFile])` for the
    newest-candidate scan;
     strictly faster than pi's sequential read.
3.  Config injection:
     adopt pi's `env`-injection pattern for testability (aligns
    with throwaway fixtures under THR).
4.  IO seams:
     expose optional `io` dependencies so unit tests fake procfs parent
    links and mapping files without touching real host coordination state.
5.  `PidMapping` shape:
     only `sessionId` shared;
     the generic parameter handles
    the rest.

Steps follow the additive-first discipline:
add `package/agent-harness-shared/session-discovery` with tests,
 generic
over `TMapping`;
migrate `package/pi-plugin/spawn/src/session-finder.ts` to a host adapter
that resolves `byPidDir` and supplies the pi `PidMapping` parser;
migrate
`package/claude-code-plugin/source/src/handlers/claude-spawn/session-finder.ts`
to a host adapter the same way.
Verify E2a with shared fake-IO tests,
 Pi host-adapter tests,
 Claude host-adapter
tests with fake `byPidDir`,
 and a `spawn-claude` CLI fixture where practical.
The generic Claude hook byte-equal fixture method remains required for hook
handler migrations,
 but it is not enough for the CLI session-discovery path.
Delete each host's local copies of the shared functions once migrated.

#### E2b: text-scan placement

Decision:
land in `package/agent-harness-shared/text-scan` as a holding place.
The package name is `@monochromatic-dev/agent-harness-shared-text-scan`.
It builds as a neutral TypeScript package because the implementation is pure
string scanning with no Node APIs.
The corrected shell-command-analyzer package name includes the `shared` segment;
do not copy its former missing-`shared` name.
Every current consumer is an agent-harness plugin,
 and
`doc/planning/package-category-rebalance.md` keeps `package/module/*` for
general-purpose TypeScript utilities with wider stewardship,
 so the holding place
keeps the DRY change scoped to the effort that discovered the duplication.

`text-scan` is too generic a name for a `package/module/*` package,
 so the
long-term home is not one `module/text-scan` package.
Tracked in GitHub issue #276:
split the primitives into purposeful `package/module/` packages organized by
concern (character classification,
 token splitting,
 word-boundary phrase
lookup,
 delimiter-range stripping),
 migrate consumers,
 then delete
`agent-harness-shared/text-scan`.
Act on #276 once a non-harness consumer appears or once the generic name starts
hiding distinct concepts.

Lift the claude superset (535 lines) into the shared package.
This is a one-directional relocation of Claude's broader text-scanning library,
not a symmetric merge of equal duplicate files:
`package/pi-plugin/spawn/src/text-scan.ts` is an 85-line subset that only needs
`isWhitespace` and `splitWhitespace`.
Delete the Pi file after migrating its two call sites to direct imports from the
shared package.
The claude `source/src/lib/text-scan.ts` internal import path changes to the
shared package;
 its nine source-file consumers update accordingly.

### E3: statusline unified rate-limit formatter

Decision:
extract the full rate-limit segment formatter,
 not only the projection helper,
and unify both hosts on the Claude statusline policy.
Both hosts render a rate-limit segment when remaining capacity is at or below
50 percent or when projected end-of-window usage exceeds 100 percent.
Severity is green above 25 percent remaining,
 yellow from 10 to 25 percent
remaining,
 and red at 10 percent remaining or below or for any projected
overrun.
This intentionally changes Pi statusline behavior:
 it will no longer be
projection-only.

Before sharing code,
 make `package/claude-code-plugin/statusline` a real
workspace package named `@monochromatic-dev/claude-code-plugin-statusline` with
`package.json`,
`mise.toml`,
 unit tests,
 and the existing README explanation that
Claude Code plugins cannot contribute a main `statusLine` setting.
It remains installed through user-scope Claude settings,
 not through
`.claude-plugin/plugin.json`.

Lift into `package/agent-harness-shared/usage-projection` as
`@monochromatic-dev/agent-harness-shared-usage-projection`:
`RateLimitSnapshot`,
 the unified rate-limit segment formatter,
 projected-overrun
computation,
 the `→N%` marker rendering,
 relative-time formatting,
 severity
selection,
 and threshold constants.
Use `snapshot.sampledAtMs` for burn-rate projection and an explicit
`renderedAtMs` formatter option for relative reset text.
Keep `paceScale` so providers such as Synthetic can normalize fractional quota
regeneration.
The shared formatter accepts style callbacks for green,
 yellow,
 red,
 and
overflow/severity rendering so Pi and Claude keep host-specific color output
while sharing policy and text assembly.

Host-specific parts stay put:
pi's HTTP-header parsers (`anthropic-rate-limit-headers.ts`,
`codex-rate-limit-headers.ts`,
`synthetic-quota-headers.ts`,
`rate-limit-headers.ts`) that build snapshots from the `after_provider_response`
event,
 and claude's JSON `rate_limits.five_hour` and `rate_limits.seven_day` tier
reader that builds snapshots from the statusline JSON.
 Both feed the shared
formatter.

Sizing:
a dedicated `agent-harness-shared/usage-projection` package is justified
because the projection model,
 formatter,
 marker,
 severity policy,
 style seam,
 and
threshold set are substantial enough not to fold into another shared package.

## What stays in host clusters

- Protocol types:
  `package/claude-code-plugin/hook-type` for claude hook-event shapes;
  `@earendil-works/pi-coding-agent` for pi event and context types.
- The `runHookPlugin` stdin/stdout runtime (claude only) and the `pi.on(...)`
  registration (pi only).
- Host-specific guardrail branches:
  Agent-resume blocking (claude),
   gitignore path-guard (pi).
- Per-plugin install containers:
  `.claude-plugin/plugin.json`,
   marketplace wiring.
  The ADR in `package/claude-code-plugin/README.md` already settled that
  per-plugin directories stay as install containers.

## Sequencing

Execution order:
E2b,
 then E1,
 then E2a,
 then E3.
Do not implement any code until all open design decisions in this plan have been
finished through the grilling session.
E2b first because it is the lowest-risk utility lift with no protocol coupling,
and it unblocks E2a whose `session-finder` imports `splitWhitespace` from
text-scan.
 E1 next (smallest logic extraction after its package rename,
re-proves the pattern).
 E2a after its text-scan dependency is shared.
 E3 last
because it now changes Pi statusline policy and packageizes Claude statusline.

Commit granularity:
per sub-step (add shared module;
 migrate pi consumer;
 migrate claude consumer;
delete old duplicate),
 each leaving the tree green.
 This matches the GCE rule
(commit eagerly at the earliest checkpoint) and the additive-first discipline
from `doc/planning/extract-refactor-guardrail.md`.

Each step is independently shippable.

## Execution constraints

From repo rules,
 for whoever executes this plan in a later session:

- Every verification run against disposable fixtures,
   never real coordination
  state.
  `session-finder` reads real procfs and real
  `~/.pi/agent/spawn-results/.by-pid/` or `~/.claude/spawn-results/.by-pid/`
  coordination files;
   tests must use `mktemp -d` plus a fake `byPidDir`,
   not a
  real home.
   See AGENTS.md THR.
- Each migrated plugin verified the way its host exercises it:
  pi side via `pi -e` extension load or the unit-test `pi-test-harness`;
  claude side via the byte-equal stdout fixture replay documented in
  `package/claude-code-plugin/README.md`.
   See AGENTS.md VUB and VB1.
- Each new shared package satisfies all lint rules:
  `require-tsdoc`,
   max-lines
  (split,
   not compress),
   and `no-restricted-syntax/no-regex` that motivated
  `text-scan`.
   See AGENTS.md LN1 and MXL.
- Each new shared package gets `README.md`,
   passes linting with zero errors,
   and
  has tests covering every exported code path before declared done.
  See AGENTS.md PKG and TCV.

## Open decisions

All resolved during grilling:

- E1:
  first rename the existing package to
  `@monochromatic-dev/agent-harness-shared-shell-command-analyzer`,
   then fold
  `invokesBunTest` plus the byte-identical ban prose (as `BUN_TEST_BAN_REASON`)
  into it.
- E2a:
  extract as `agent-harness-shared/session-discovery` with package name
  `@monochromatic-dev/agent-harness-shared-session-discovery`,
   generic over the
  host `PidMapping`;
   adopt pi's env-injection,
   claude's `Promise.all`
  stat-plus-read concurrency,
   iterative tree walk,
   injectable fake-IO seams,
   and
  adapter plus CLI-oriented verification.
- E2b:
  land in `agent-harness-shared/text-scan` with package name
  `@monochromatic-dev/agent-harness-shared-text-scan` as a neutral holding
  package;
   lift the Claude superset;
   delete Pi's local `text-scan.ts` after
  direct imports;
   long-term split into purposeful `package/module/` packages
  tracked in GitHub issue #276.
- E3:
  make `package/claude-code-plugin/statusline` a real package named
  `@monochromatic-dev/claude-code-plugin-statusline`;
   extract the unified full
  formatter as `@monochromatic-dev/agent-harness-shared-usage-projection`;
  adopt the Claude remaining-capacity-or-projection policy for both hosts;
  use separate sample and render times plus host style callbacks.
- Sequencing:
  finish grilling all decisions before code execution;
   then implement E2b,
   E1,
  E2a,
   and E3 with per-sub-step commits.
