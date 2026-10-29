# Subagent extension audit

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/mutation-test` is now `package/mutation-test/cli`.

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Purpose

Evaluate open-source Pi extensions and a minimal custom design for subagent orchestration,
 prioritizing clarity and human auditability for both the parent agent and the user.

## Locked context

- Compare in-process and child-process execution models equally.
- Require a read-only subagent option;
   write access may exist when the extension communicates capabilities clearly.
- Include a minimal custom extension only as a fallback comparison after surveying existing options.
- Target the installed Pi and Node workflow.
- Require a user-facing UI that exposes the complete observable child activity:
   prompts,
   progress,
   tool calls,
   outputs,
   status,
   and errors.
   Hidden model reasoning is outside the observable requirement.
- Require user interruption of any running subagent from that UI,
   including parallel and background children.
- Require the parent model to set a custom timeout per subagent,
   not only one global timeout.
- Treat this as an agent/plugin trust-boundary review.
   Inspect source,
   tests,
   CI,
   dependencies,
   maintenance,
   and integration behavior before recommending anything.

## Current findings

- The exposed `subagent` tool reports six registered names:
   `scout`,
   `planner`,
   `reviewer`,
   `worker`,
   `general`,
   and `general-purpose`.
- Shell-visible user and project agent directories did not explain that registry,
   so the registry source remains an open investigation item.
- A read-write probe succeeded:
   `/tmp/agent/hello.txt` contains `Hello`.
- All six registered names accepted read-only diagnostic probes.
- A broad delegated source-audit attempt timed out for every shard
   at 180 seconds without producing usable final reports;
   continue with narrower bounded probes and direct source reads.

## Investigation state

- Context fork:
   complete.
- Candidate inventory:
   complete.
   UI observability,
   interruption,
   and per-subagent timeout control are hard filters.
- Source and maintenance audit:
   complete.
- Integration validation:
   host loading complete;
   child execution and interactive interruption remain unproven.
- Recommendation:
   complete for evaluation;
   adoption awaits user selection and an implementation-level end-to-end proof.

## Screened candidate set

Meaningful open-source candidates found through GitHub and npm searches:

- `nicobailon/pi-subagents`,
   npm `0.34.0`,
   package.json declares MIT,
   cloned at
  `/tmp/agent/nicobailon-pi-subagents-20260709`.
- `mjakl/pi-subagent`,
   npm `2.1.0`,
   MIT,
   cloned at `/tmp/agent/mjakl-pi-subagent-20260709`.
- `jwu/pi-subagents`,
   npm `2.1.0`,
   MIT,
   cloned at `/tmp/agent/jwu-pi-subagents-20260709`.
- `@e9n/pi-subagent` from `espennilsen/pi`,
   npm `0.1.0`,
   MIT,
   cloned at
  `/tmp/agent/espennilsen-pi-20260709`.
- `@the-forge-flow/sub-agents-pi`,
   npm `0.2.4`,
   MIT,
   cloned at
  `/tmp/agent/monsieurbarti-sub-agents-pi-20260709`.
- `pi-multiagent`,
   npm `0.9.8`,
   MIT,
   cloned at `/tmp/agent/tiziano-pi-multiagent-20260709`.
- `pi-crew`,
   npm `0.9.29`,
   MIT,
   cloned at `/tmp/agent/baphuongna-pi-crew-20260709`.
- `cmf/pi-subagent` was screened out from finalist depth:
   its repository has two commits,
   no
  release,
   no stars,
   and no declared license in the GitHub metadata.

## Hard-filter evidence so far

- `nicobailon/pi-subagents` has explicit TUI rendering,
   child tool-call progress,
   transcript
  views,
   fleet status,
   interrupt and stop actions,
   and per-run `timeoutMs` in
  `src/extension/schemas.ts`,
   `src/extension/tool-description.ts`,
   and
  `src/runs/background/async-execution.ts`.
   Its parallel task schema currently exposes no
  per-task timeout field,
   so this remains a possible mismatch with the per-subagent timeout
  requirement.
- `jwu/pi-subagents` renders live tool calls in `extensions/subagent-render.ts` and aborts the
  child on the parent signal in `extensions/subagent-executor.ts`,
   but the inspected tool schema
  has no subagent timeout or user-facing control action.
- `@e9n/pi-subagent` captures full child messages and has abort-aware one-shot execution plus
  pool kill operations in `extensions/pi-subagent/src/runner.ts`,
   `rpc-agent.ts`,
   `pool.ts`,
   and
  `tool.ts`.
   Its timeout is a settings-level value,
   not yet verified as a parent-set per-child
  value.
- `pi-multiagent` has an operator widget,
   raw event inspection,
   cancel control,
   per-step
  `timeoutSecondsPerStep`,
   and detailed child event handling.
   Its own skill says normal live
  assistant and tool activity is not all parent-visible by default,
   so it needs careful testing
  against the complete-observability requirement.
- `@the-forge-flow/sub-agents-pi` explicitly advertises live tool-call streaming,
   full transcript
  expansion,
   an interruptible TUI panel,
   and per-call tool allowlists in its README and source.
  Its current package surface does not show a parent-set timeout parameter.
- `pi-crew` has a rich dashboard,
   transcript viewer,
   cancellation,
   and a large test suite,
   but
  its README states that the project is mostly AI-developed and not hardened.
   The cloned source
  is also materially larger than the other candidates and its timeout controls need to be
  separated from global run,
   no-output,
   and workflow-step settings.

## Validation probes

Host verification measured Pi `0.80.6` and Node `v26.5.0`.
 Real Pi RPC smoke runs loaded
`@the-forge-flow/sub-agents-pi`,
 `nicobailon/pi-subagents`,
 `pi-crew`,
 and `pi-multiagent` from
cloned source paths.
 Each returned a successful `get_state` response;
 Forge,
 Nico,
 Pi Crew,
 and
Multiagent also emitted extension UI requests.
 Pi Crew emitted status and widget registrations,
Forge and Nico emitted widget registrations,
 and Multiagent emitted its live widget.
 This proves
host loading and registration,
 not complete child execution,
 complete live history,
 or interactive
control.

All containerized validation runs used a 2 GB memory cap and 2 CPU cap.

- `mjakl/pi-subagent`:
   `npm install --no-package-lock && npm test` passed all 37 tests.
  Its test suite does not cover the missing UI control or timeout features.
- `@the-forge-flow/sub-agents-pi`:
   `npm install --no-package-lock && npm test` passed 136 tests
  across 12 files.
   `npm run test:coverage` fails because `@vitest/coverage-v8` is not declared.
  Installation reported six dependency vulnerabilities and deprecated `@mariozechner` Pi peers.
- `nicobailon/pi-subagents`:
   with Git available,
   1081 of 1082 tests passed.
   The repeated failure is
  `test/unit/watchdog-lsp-diagnostics.test.ts`,
   malformed language-server JSON,
   with `write EPIPE`
  from `src/watchdog/lsp-diagnostics.ts:318`.
   A first run without Git had 30 environment-driven
  failures;
   those disappeared after Git was installed in the disposable container.
- `@e9n/pi-subagent`:
   typecheck passed and its `node --test` command found zero tests.
   Installation
  reported one high-severity vulnerability.
- `pi-multiagent`:
   install without a frozen lockfile added 110 packages,
   then stopped because pnpm
  ignored build scripts for `@google/genai`,
   `koffi`,
   and `protobufjs` pending approval.
   The clone
  has no `pnpm-lock.yaml`,
   so the repository's frozen-lockfile validation path is not runnable as
  checked out.
- `pi-crew`:
   the full test command reached 5991 tests,
   with 5987 passing,
   3 skipped,
   and one
  failure in `test/unit/settings-store-cov.test.ts` where a root container could write a path the
  test expected to reject.
   Re-running that file as the mapped non-root user passed all 13 tests;
  the full root-container command was still not green.
- `jwu/pi-subagents`:
   using the fully qualified Bun image,
   `bun run lint && bun test` passed
  formatting,
   typecheck,
   and 111 tests across 7 files.
   Bun reported two blocked postinstall
  scripts requiring trust approval.

## Evidence rules

- Do not recommend a candidate from metadata alone.
- Clone serious open-source candidates under `~/temp/agent/` and record exact paths.
- Read production source,
   tests,
   CI,
   dependency manifests,
   and security-sensitive boundaries.
- Search for fuzzing,
   property testing,
   mutation testing,
   and coverage evidence;
   record absence explicitly.
- Run each candidate's complete validation task and exercise the Pi integration boundary where feasible.
- Compare code volume,
   runtime dependencies,
   architecture shape,
   security-code concentration,
   and rendering or generated-code surface.
- Keep the final ranking and rejection reasons in this handover
  until a decision document is created after user selection.

## Maintenance signals

GitHub API snapshots used repository metadata and issue or pull-request activity for the 12 months ending 2026-07-09.
 The issue query used a 100-item page cap.

- `nicobailon/pi-subagents`:
   2,478 stars,
   339 forks,
   100 returned issue items,
   82 with OWNER,
   MEMBER,
   or COLLABORATOR comments,
   and 24 with labels or assignees.
   The current pull-request page returned 100 items,
   64 merged and 51 with maintainer author associations.
- `mjakl/pi-subagent`:
   68 stars,
   3 issue items,
   2 with maintainer comments,
   and 5 pull requests.
   GitHub source declares `3.0.0`,
   while npm reports `2.1.0`,
   so installation source must be pinned explicitly.
- `jwu/pi-subagents`:
   0 stars,
   no recent issue items,
   and one pull request.
   The source and npm versions both report `2.1.0`.
- `@e9n/pi-subagent`:
   the containing `espennilsen/pi` repository has 111 stars and 2 open issues,
   but the extension itself has no separate repository activity surface.
- `@the-forge-flow/sub-agents-pi`:
   0 stars,
   no recent issue items,
   and 12 pull requests.
   The package is current at `0.2.4`,
   but its Pi peer packages are deprecated `@mariozechner` names.
- `pi-multiagent`:
   16 stars,
   5 issue items,
   3 with labels or assignees,
   and 9 pull requests.
   Its public source and package versions align at `0.9.8`.
- `pi-crew`:
   34 stars,
   29 issue items,
   20 with maintainer comments,
   12 with labels or assignees,
   and 6 pull requests.
   It has the strongest public self-audit documentation but also the largest source surface.

## Custom extension size and effort estimate

The estimate uses Tokei code lines so comments and blank lines do not distort comparisons.
 Measured baselines are:

- Official Pi subagent example:
   995 production code lines.
   It is a reference implementation,
  not a hard-filter-complete operator experience.
- `@the-forge-flow/sub-agents-pi`:
   1,421 production and 1,773 test code lines.
   It lacks
  per-child timeout and complete live history.
- `jwu/pi-subagents`:
   1,800 production and 2,311 test code lines.
   It lacks targeted operator
  interruption and per-child timeout.
- Existing local `package/pi-plugin/spawn`:
   1,659 production and 1,236 test code lines.
  Its documentation-heavy project style occupies 3,424 physical production lines,
   which is a
  useful warning that physical line count will be materially higher than code-only count.
- Local `package/cli/mutation-test` plus its fixture:
   4,718 current code lines excluding
  licenses.
   Its initial implementation series added 9,603 physical lines across 63 files in 18
  commits on 2026-07-05.
   The commits span 17:43:39 to 19:57:40,
   although commit timestamps do not
  measure the full work interval.
   The user reports completing the framework in under 16 hours
  with AI assistance.
   That establishes local observed throughput above 295 code lines or 600
  physical lines per hour for a framework that includes parser operators,
   container
  orchestration,
   timeout handling,
   taint-aware retries,
   CLI behavior,
   tests,
   and fixtures.

This is a planning estimate for a hard-filter-complete implementation,
 not an observed size.
Production is estimated at 2,800 to 4,000 code lines:

- Public tool schema,
   child descriptors,
   capability profiles,
   and validation:
   300 to 450.
- Strict JSONL child protocol,
   spawn configuration,
   event capture,
   and error translation:
  550 to 800.
- Foreground,
   parallel,
   and background lifecycle state,
   individual deadlines,
   process-tree
  cancellation,
   escalation,
   reaping,
   and race handling:
   700 to 1,050.
- Complete event store,
   ordering,
   transcript formatting,
   and parent-facing result assembly:
  400 to 650.
- Scrollable TUI,
   child selection,
   status,
   error display,
   and per-child or all-child controls:
  650 to 800.
- Integration glue and logging not naturally included in those modules:
   200 to 250.

Tests and fixtures are estimated at 3,500 to 5,500 code lines.
 This includes unit tests for every
exported branch plus a PTY or fake-Pi harness and real Pi boundary tests for complete event
ordering,
 independent deadlines,
 selected-child interruption,
 descendant cleanup,
 background
jobs,
 malformed JSONL,
 read-only profiles,
 repeated interruption,
 and spawn or exit races.
 Package
configuration,
 README content,
 and operator documentation add roughly 300 to 600 physical lines.

The resulting adoption-ready package is estimated at 6,300 to 9,500 code lines,
 or approximately
10,000 to 15,000 physical lines after this repository's required TSDoc,
 logging,
 spacing,
 and test
fixtures.
 A thin demonstration could be smaller,
 but it would not prove the hard filters and is
not the relevant adoption estimate.

The previous conventional estimate of 18 to 28 engineer-days is rejected as inconsistent with
this repository's measured AI-assisted delivery rate.
 Scaling the projected 6,300 to 9,500 code
lines from the under-16-hour mutation-framework anchor gives a straight-line upper estimate of
approximately 21 to 33 hours.
 The planning range is 24 to 40 focused AI-assisted hours,
 with a
32-hour point estimate.
 This is 3 to 5 focused 8-hour days and retains margin for interactive TUI
and child-process integration risk:

- Contract,
   threat model,
   and capability profiles:
   2 to 3 hours.
- Child protocol,
   event persistence,
   and result assembly:
   5 to 7 hours.
- Concurrency,
   independent deadlines,
   cancellation,
   escalation,
   and cleanup:
   5 to 8 hours.
- Complete transcript UI and interruption controls:
   5 to 8 hours.
- Read-only enforcement and adversarial tests:
   3 to 5 hours.
- Pi boundary harness,
   packaging,
   documentation,
   and review fixes:
   4 to 9 hours.

The estimate assumes the same engineer,
 AI-assisted workflow,
 repository familiarity,
 and quality
bar demonstrated by the mutation framework.
 It also assumes Linux process-group cleanup,
 Pi
`0.80.6`,
 Node,
 no inherited child extensions by default,
 and read-only defined as an explicit
tool allowlist with ambient extensions
disabled.
 Cross-platform descendant cleanup,
 an operating-system sandbox,
 persistent transcript
search,
 or upstream compatibility across multiple Pi versions are outside the range.
 Reusing
local spawn-plugin code may reduce effort,
 but no reduction is credited until its interfaces are
shown to fit strict JSONL event capture and interactive lifecycle control.

## Synthesis

No existing candidate is a clean pass on all hard requirements.
 The most important unresolved integration gap is a real interactive Pi `0.80.6` run
 proving complete live event visibility,
 selected-child interruption,
 independent per-child deadlines,
 descendant cleanup,
 and read-only enforcement against ambient extensions.

Personal ranking by fit for the stated requirements and auditability:

1. A minimal custom extension,
    unvalidated until implemented.
    Use a single explicit child descriptor with `id`,
    `systemPrompt`,
    `task`,
    `tools`,
    `cwd`,
    and `timeoutMs`;
    capture every JSONL child event;
    show a scrollable complete transcript;
    expose selected-child and all-child interruption;
    use `--no-extensions` unless explicitly granted;
    preserve an explicit read-only profile.
2. `nicobailon/pi-subagents`,
    the closest existing implementation.
    It needs per-task timeout fields and a clean current-Pi integration proof;
    its measured 39,712 production code lines are a substantial audit surface.
3. `@the-forge-flow/sub-agents-pi`,
    the clearest small reference.
    Its measured 1,421 production code lines and selected-child TUI kill are attractive,
    but it needs a timeout field,
    complete live history,
    current `@earendil-works` imports,
    and a fixed coverage dependency.
4. `pi-multiagent`,
    strong authority and raw-event foundations,
    but its live UI summarizes activity and its timeout is shared per step rather than individually specified.
5. `pi-crew`,
    broad dashboard and transcript features,
    but its measured 76,740 production code lines and explicit not-hardened warning make it a poor auditability fit.
6. `jwu/pi-subagents`,
    measured 1,800 code lines with 111 passing tests and expanded tool logs,
    but no user control action or per-child timeout.
7. `@e9n/pi-subagent`,
    measured 2,541 code lines with full message capture and pool kill internals,
    but no tests,
    no adequate operator UI,
    and settings-level timeout only.
8. `mjakl/pi-subagent`,
    measured 37 passing tests and a small surface,
    but no timeout or operator control action.
9. The official Pi subagent example and `cmf/pi-subagent` remain reference or low-signal implementations,
    not viable hard-filter finalists.

Do not create a decision document until the user selects whether to patch
an existing extension or implement the custom design.
 If the user selects one,
 write the rejected alternatives and exact pinned source version under `doc/decision/`.

## Continuation checklist

1. Preserve this synthesis during any context compaction.
2. If the user requests implementation,
    prototype the custom event contract in a disposable path before editing the main worktree.
3. Exercise the real Pi integration boundary before adopting any candidate.
4. Write `doc/decision/<project>.md` after the user selects an option.
