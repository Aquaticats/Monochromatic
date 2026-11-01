# Pi dynamic workflows replacement vet report

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Audit metadata

- Status:
   named candidate excluded;
   discovery reopened after independent review
- Lifecycle phase:
   complete;
   no recommendation
- Subject:
   `pi-dynamic-workflows` replacement for `@narumitw/pi-subagents`
- Scope:
   evaluate whether `@quintinshaw/pi-dynamic-workflows` is safe and suitable to replace the installed
  `@narumitw/pi-subagents` Pi extension.
- Start date:
   2026-07-10
- Last updated:
   2026-07-10
- Governing skill commit:
   `3b6d1bd6ac0c6eb5704152ddb00e2b69ddcf653b`
- Governing skill SHA-256:
   `71c50a51d0f0086f789e350ef43824f8aead66435f9ab92d94aae751d16d8359`
- Compatibility fingerprint:
  `d6adb5e5bf6999a490f5b116b145b3fa4318ef611f18bf3d9cad01ff445299f2`
- Active audit owner:
   current Pi session;
   harness does not expose a stable session identifier
- Prior compatible report:
   none found under `doc/audit/`;
   the related
  `doc/handover/subagent-extension-audit.md` predates this candidate-specific report and does not carry a compatibility
  fingerprint.

## Compatibility fingerprint input

```json
{"baseCategories":["inspectable open-source local technology"],"decisionScope":"Evaluate whether @quintinshaw/pi-dynamic-workflows is safe and suitable to replace the installed @narumitw/pi-subagents Pi extension.","deployment":{"architecture":"x86_64","operatingSystem":"Linux","piVersion":"0.80.6","runtime":"Node 26.5.0"},"hardConstraints":["Complete observable child prompts, progress, tool calls, outputs, status, and errors in user-facing UI","Inspectable open-source execution path with compatible license and mapped build provenance","No ambient credentials or unbounded third-party execution during validation","Parent-set per-subagent timeout including parallel children","Read-only subagent capability enforced by explicit tool allowlist without ambient extensions","Runs on installed Pi 0.80.6 with Node 26.5.0 on Linux x86_64","User can interrupt every running foreground, parallel, and background child from the UI"],"incumbentName":"@narumitw/pi-subagents","incumbentVersion":"0.13.0","overlays":["high-trust execution in an agent extension","human auditability","incumbent dependency replacement"],"schemaVersion":1,"subject":"pi-dynamic-workflows replacement for @narumitw/pi-subagents","trustBoundary":"High-trust Pi extension that launches model agents with repository, filesystem, process, network, and credential-adjacent access."}
```

The fingerprint input uses NFC strings,
 sorted set-valued arrays,
 recursively sorted deployment keys,
 JSON canonical
serialization,
 and SHA-256 over the exact UTF-8 bytes.

## Context

### Measured deployment

- `/var/home/user/.pi/agent/settings.json` loads `npm:@narumitw/pi-subagents` globally.
- The installed manifest at
  `/var/home/user/.pi/agent/npm/node_modules/@narumitw/pi-subagents/package.json` identifies version `0.13.0`,
   MIT
  licensing,
   one runtime dependency on `typebox`,
   and source repository
  <https://github.com/narumiruna/pi-extensions/tree/main/extensions/pi-subagents>.
- The running host reports Pi `0.80.6`,
   Node `26.5.0`,
   Linux,
   and `x86_64`.
- The project-level `.pi/settings.json` deliberately contains no package list,
   so the replacement concerns the global Pi
  workflow rather than project configuration.
- The installed incumbent contains `1,980` physical lines across its TypeScript files,
   measured with `wc --lines`.
  Code-only size and cloned-source parity remain pending.

### Prior settled requirements

The related audit handover at `doc/handover/subagent-extension-audit.md` records requirements that remain applicable to
this replacement request:

- observable child prompts,
   progress,
   tool calls,
   outputs,
   status,
   and errors;
- user interruption for foreground,
   parallel,
   and background children;
- parent-selected timeout per child;
- a read-only capability profile;
- source,
   test,
   CI,
   dependency,
   maintenance,
   and integration inspection.

The current request names a replacement rather than reopening those requirements,
 so this audit carries them forward
instead of asking a rubber-stamp question.

### Component classification

- `@narumitw/pi-subagents`:
   inspectable open-source local technology.
- `@quintinshaw/pi-dynamic-workflows`:
   inspectable open-source local technology,
   pending source and artifact provenance
  confirmation.
- Active overlays:
   incumbent dependency replacement,
   high-trust agent execution,
   and human auditability.
- Managed service gate:
   not applicable because neither compared component is a hosted control plane.
- SaaS historical and operational gates:
   not applicable for the same reason.
- Proprietary local technology gate:
   not applicable because both named components publish source under an asserted MIT
  license;
   the license text and package-source parity still require confirmation.
- Sensitive-data compliance overlay:
   not separately active because the request sets no residency,
   retention,
   or regulated
  data requirement.
   Credential-adjacent execution remains covered by the high-trust overlay.
- Multi-platform overlay:
   not active as a hard requirement because the measured deployment is Linux `x86_64`.
   Any broader
  platform claim made by a finalist will still be checked before receiving score credit.
- Native,
   Wasm,
   generated-code,
   and prebuilt overlays:
   pending dependency and release-artifact inspection.

## Resolved and unresolved preferences

- Resolved conditional preference:
   if existing tools fail and a custom implementation becomes necessary,
   use Zellij as
  its operator-control base so the human can enter,
   inspect,
   steer,
   and interrupt child sessions directly.
- This preference does not authorize building the fallback and does not relax the existing-tools-first gate.
- Unresolved preferences:
   none that can repair a named candidate's hard-gate failure.

## Hard constraints

- Complete observable child prompts,
   progress,
   tool calls,
   outputs,
   status,
   and errors in user-facing UI.
- User interruption of every running foreground,
   parallel,
   and background child from the UI.
- Parent-set per-subagent timeout,
   including parallel children.
- Read-only subagent capability enforced by an explicit tool allowlist without ambient extensions.
- Inspectable open-source execution path with compatible license and mapped build provenance.
- Successful execution on installed Pi `0.80.6` with Node `26.5.0` on Linux `x86_64`.
- No ambient credentials or unbounded third-party execution during validation.

## Initial candidate ledger

### `@narumitw/pi-subagents` `0.13.0`

- Discovery source:
   installed global Pi package and local package lock.
- Base category:
   inspectable open-source local technology.
- Overlays:
   incumbent replacement baseline,
   high-trust agent execution,
   human auditability.
- Screening result:
   pending targeted parity audit;
   retained because replacement parity requires keeping the incumbent.

### `@quintinshaw/pi-dynamic-workflows` `2.12.1`

- Discovery source:
   user-named candidate;
   official repository
  <https://github.com/QuintinShaw/pi-dynamic-workflows> and npm registry record
  <https://registry.npmjs.org/@quintinshaw%2Fpi-dynamic-workflows/latest>.
- Base category:
   inspectable open-source local technology.
- Overlays:
   incumbent replacement candidate,
   high-trust agent execution,
   human auditability;
   native,
   Wasm,
   generated,
   and
  prebuilt status pending.
- Screening result:
   serious alternative pending hard-gate confirmation.
   The npm registry reports version `2.12.1`,
   MIT,
  one runtime dependency on `acorn`,
   and `100` published files.
   These are discovery facts,
   not trust evidence.

### `pi-subagents` `0.34.0`

- Discovery source:
   prior local audit plus npm and web rediscovery at <https://github.com/nicobailon/pi-subagents>.
- Base category:
   inspectable open-source local technology.
- Overlays:
   incumbent replacement candidate,
   high-trust execution,
   human auditability.
- Screening result:
   serious alternative in the prior audit;
   further promotion stops because required discovery is blocked.

### `pi-dynamic-workflows` `1.0.1`

- Discovery source:
   npm and web search at <https://github.com/Michaelliv/pi-dynamic-workflows>.
- Base category:
   inspectable open-source local technology.
- Overlays:
   incumbent replacement candidate,
   high-trust execution,
   human auditability.
- Screening result:
   discovered upstream implementation;
   source relationship to the user-named fork requires targeted
  provenance comparison.

### `pi-dynamic-workflows-oc-style` `0.2.3`

- Discovery source:
   npm and web search at <https://github.com/gtnotacoder/pi-dynamic-workflows>.
- Base category:
   inspectable open-source local technology.
- Overlays:
   incumbent replacement candidate,
   high-trust execution,
   human auditability.
- Screening result:
   discovered fork;
   pending hard-gate evidence.

### `pi-taskflow` `0.1.8`

- Discovery source:
   npm,
   GitHub,
   and web expansion at <https://github.com/heggria/taskflow>.
- Base category:
   inspectable open-source local technology.
- Overlays:
   incumbent replacement candidate,
   high-trust execution,
   human auditability.
- Screening result:
   discovered graph-oriented alternative;
   pending hard-gate evidence.

### `pi-workflow-engine` `0.10.3`

- Discovery source:
   npm and web expansion at <https://github.com/timbrinded/pi-workflow-engine>.
- Base category:
   inspectable open-source local technology.
- Overlays:
   incumbent replacement candidate,
   high-trust execution,
   human auditability.
- Screening result:
   discovered orchestration alternative;
   pending hard-gate evidence.

### `pi-swarm` `0.9.5`

- Discovery source:
   npm,
   GitHub expansion,
   and web expansion at <https://github.com/gjczone/pi-swarm>.
- Base category:
   inspectable open-source local technology.
- Overlays:
   incumbent replacement candidate,
   high-trust execution,
   human auditability.
- Screening result:
   discovered team-orchestration alternative;
   pending hard-gate evidence.

### `pi-crew` `0.9.32`

- Discovery source:
   prior local audit plus npm rediscovery at <https://github.com/baphuongna/pi-crew>.
- Base category:
   inspectable open-source local technology.
- Overlays:
   incumbent replacement candidate,
   high-trust execution,
   human auditability.
- Screening result:
   serious alternative in the prior audit;
   further promotion stops because required discovery is blocked.

### `pi-multiagent` `0.9.8`

- Discovery source:
   prior local audit plus web rediscovery at <https://github.com/Tiziano-AI/pi-multiagent>.
- Base category:
   inspectable open-source local technology.
- Overlays:
   incumbent replacement candidate,
   high-trust execution,
   human auditability.
- Screening result:
   serious alternative in the prior audit;
   further promotion stops because required discovery is blocked.

### Prior lightweight alternatives

The prior audit also carries forward `mjakl/pi-subagent`,
 `jwu/pi-subagents`,
 `@e9n/pi-subagent`,
`@the-forge-flow/sub-agents-pi`,
 and the official Pi subagent example.
 Each has inspectable source,
 but the prior source and
runtime review identified missing operator-interruption,
 per-child-timeout,
 UI,
 test,
 or current-Pi evidence.
 They remain
ledger entries rather than invented alternatives;
 no preliminary soft score removes them.

### Ledger limit caused by source block

The frozen searches discovered many additional plausible packages and repositories.
 GitHub's cap prevents enumerating the
required source class,
 so this ledger cannot honestly claim completeness.
 Under the terminal discovery rule,
 incomplete
candidate enumeration ends in no recommendation rather than arbitrary finalist selection.

No custom implementation can be recommended unless every ready-to-use technology fails a named hard constraint.

## Preliminary evidence limits

- Repository and registry descriptions are promotional or metadata evidence.
   They do not establish behavior,
   provenance,
  safety,
   maintenance quality,
   or replacement parity.
- The GitHub page currently shows `121` stars,
   `38` forks,
   `84` commits,
   `34` tags,
   and release `2.12.1` dated 2026-07-10 at
  <https://github.com/QuintinShaw/pi-dynamic-workflows>.
   These counts are low-signal discovery context only.
- No candidate recommendation is made at this phase.

## Frozen criteria and weights

Hard gates remain outside arithmetic.
 Every surviving finalist receives each criterion at equal depth.

- Human auditability and attack-surface concentration,
   weight `5`:
   the prior audit explicitly prioritizes clarity and
  human auditability over feature breadth.
   Measure source lines and files,
   runtime and same-author dependencies,
   control
  flow,
   state and event boundaries,
   generated code,
   and where filesystem,
   process,
   network,
   and credentials concentrate.
- Incumbent-path parity,
   weight `1`:
   compare single,
   parallel,
   sequential,
   aggregation,
   custom agent,
   custom timeout,
  tool-capability,
   result-delivery,
   and operator-control paths actually exposed by the installed incumbent.
- Failure handling and lifecycle reliability,
   weight `1`:
   inspect timeout,
   cancellation,
   descendant cleanup,
   retry,
  malformed-event,
   persistence,
   resume,
   and concurrent-run behavior.
- Test and validation strength,
   weight `1`:
   inspect unit,
   integration,
   end-to-end,
   property,
   fuzz,
   mutation,
   platform,
  coverage,
   and required CI evidence,
   then reproduce relevant suites.
- Maintenance and release discipline,
   weight `1`:
   measure recent issue and pull-request handling,
   maintainer-authored work,
  release latency,
   version alignment,
   release provenance,
   and maintainer concentration.
- Resource and cost governance,
   weight `1`:
   inspect concurrency,
   agent-count,
   token,
   timeout,
   process,
   filesystem,
   and
  background-run bounds.
   Unused breadth receives no credit.
- Operator ergonomics beyond hard gates,
   weight `1`:
   compare transcript navigation,
   status detail,
   error diagnosis,
  steering,
   persistence visibility,
   and configuration clarity after the mandatory observability and interruption gates
  pass.
- Migration and reversibility,
   weight `1`:
   compare configuration change,
   agent-definition compatibility,
   stored state,
  coexistence conflicts,
   rollback,
   and whether replacing one tool changes the parent model's orchestration contract.

Each rating uses `0` through `4` and records high,
 medium,
 or low confidence.
 Unspecified priorities retain weight `1`.
The only elevated weight comes from the settled auditability preference.
 Candidate-specific evidence cannot change these
weights.

Maximum baseline score for every finalist is:

```text
(5 + 1 + 1 + 1 + 1 + 1 + 1 + 1) * 4 = 48
```

## Query schedule and discovery ledger

### Initial frozen schedule

No candidate-specific rating was assigned before this schedule was recorded.

#### npm registry

For each literal query,
 request results ordered by npm search quality from offset `0` in complete pages of `100`,
 then
continue until the registry is exhausted or two consecutive complete pages add no screening survivor:

- `pi subagent`
- `pi subagents`
- `pi dynamic workflows`
- `pi workflow orchestration`
- `@narumitw/pi-subagents alternative`
- `pi coding agent multi agent`

Record package name,
 version,
 repository URL,
 publisher,
 date,
 keywords,
 and screening status.
 A package without a mapped
public source repository fails the inspectability gate.

#### GitHub repositories, topics, code, and releases

For each literal repository query,
 order by `updated` descending with `100` results per page.
 Continue until exhaustion or
two consecutive complete pages add no screening survivor:

- `pi-coding-agent subagent in:name,description,readme`
- `pi-coding-agent subagents in:name,description,readme`
- `pi dynamic workflows in:name,description,readme`
- `pi workflow orchestration agents in:name,description,readme`
- `narumitw pi-subagents alternative in:name,description,readme`
- `topic:pi-package subagents`

For candidates discovered from repositories,
 inspect repository topics,
 package manifests,
 releases,
 organization
projects,
 and code references before screening promotion.

#### Broader web

Run each literal query through the configured Exa search with Linkup fallback.
 These providers expose a finite result set
rather than a page cursor in this harness,
 so record the returned count and complete the schedule at that result set:

- `Pi coding agent subagent extension`
- `Pi coding agent dynamic workflow extension`
- `Pi dynamic workflows alternative`
- `@narumitw/pi-subagents alternative`
- `Pi multi-agent orchestration extension npm GitHub`
- `Pi code mode subagents`

#### This repository and installed system

Run and record these literal local searches without result caps
or negative-match filters beyond excluding dependency and
Git metadata where stated:

- fixed string `@narumitw/pi-subagents` across the repository;
- terms `pi-subagents`,
   `dynamic-workflows`,
   and `subagent` across `.agents`,
   `.pi`,
   `docs`,
   and manifests;
- candidate reports under `doc/audit/`;
- relevant decisions under `doc/decision/`;
- parallel orchestration implementations under `package/pi-plugin/`;
- installed package manifests and lock entries under `/var/home/user/.pi/agent/npm/`.

### Expansion round

The initial web and registry results introduced `taskflow`,
 `workflow engine`,
 `interactive subagents`,
 `code mode`,
`agent swarm`,
 and `workflow TUI`.
 The one allowed de-duplicated expansion round is now frozen:

#### npm registry expansion

- `pi taskflow`
- `pi workflow engine`
- `pi interactive subagents`
- `pi code mode subagents`
- `pi agent swarm`
- `pi workflow TUI`

Use the same quality-only ordering,
 page size,
 and stop rule as the initial npm schedule.

#### GitHub expansion

- `pi-taskflow in:name`
- `pi-workflow-engine in:name`
- `pi-interactive-subagents in:name`
- `pi-codeMode in:name`
- `pi-swarm in:name`
- `pi-loom in:name`

Use the same updated-descending ordering,
 page size,
 and stop rule as the initial GitHub schedule.

#### Broader-web expansion

- `Pi taskflow subagents`
- `Pi workflow engine subagents`
- `Pi interactive subagents extension`
- `Pi code mode subagents extension`
- `Pi agent swarm extension`
- `Pi workflow TUI extension`

No later taxonomy term will add another query.

### Initial query execution

#### npm registry pages

Each request used `size=100`,
 `quality=1`,
 `popularity=0`,
 and `maintenance=0` at
<https://registry.npmjs.org/-/v1/search>.
 Returned counts and registry-reported totals were:

- `pi subagent`:
   pages `1`,
   `2`,
   and `3` each returned `100`;
   total `14,187`.
- `pi subagents`:
   pages `1`,
   `2`,
   and `3` each returned `100`;
   total `14,068`.
- `pi dynamic workflows`:
   pages `1`,
   `2`,
   and `3` each returned `100`;
   total `177,682`.
- `pi workflow orchestration`:
   pages `1`,
   `2`,
   and `3` each returned `100`;
   total `104,204`.
- `@narumitw/pi-subagents alternative`:
   pages `1`,
   `2`,
   and `3` each returned `100`;
   total `32,042`.
- `pi coding agent multi agent`:
   pages `1`,
   `2`,
   and `3` each returned `100`;
   total `175,594`.

Every third page still introduced plausible public-source Pi orchestration packages.
 Examples new at page `3` included
`@kky42/pi-flow`,
 `pi-conductor`,
 `pi-workflow-engine`,
 `pi-agent-workflows`,
 `@aphotic/pi-mux-subagents`,
`pi-analyst-worker-orchestrator`,
 `pi-submarine`,
 `@adamjen/pi-interactive-subagents`,
 and `pi-fast-subagent`.
 The page
stop condition was therefore not met.

The initial npm pages also recovered the user-named candidate,
 incumbent,
 and prior-audit alternatives including
`pi-subagents`,
 `pi-crew`,
 `pi-taskflow`,
 `pi-swarm`,
 `pi-multiagent`,
 `pi-dynamic-workflows`,
`pi-dynamic-workflows-oc-style`,
 `@johnnywu/pi-subagents`,
 `@e9n/pi-subagent` through its repository,
 and
`@the-forge-flow/sub-agents-pi` through prior repository evidence.
 Numerous metadata-only packages lacked a public source
URL and failed inspectability immediately;
 examples include `pi-side-agents`,
 `@lebronj/pi-suite`,
 and
`@anishthite/pi-better-workflows`.

#### GitHub repository pages

Each request used `per_page=100`,
 `sort=updated`,
 and `order=desc` through the GitHub Search API:

- `pi-coding-agent subagent in:name,description,readme`:
   pages `1` to `3` each returned `100`;
   total `1,192`.
- `pi-coding-agent subagents in:name,description,readme`:
   pages `1` to `3` each returned `100`;
   total `1,195`.
- `pi dynamic workflows in:name,description,readme`:
   pages `1` to `3` each returned `100`;
   total `23,764`.
- `pi workflow orchestration agents in:name,description,readme`:
   pages `1` to `3` each returned `100`;
   total `12,067`.
- `narumitw pi-subagents alternative in:name,description,readme`:
   page `1` returned `3`;
   exhausted.
- `topic:pi-package subagents`:
   page `1` returned `15`;
   exhausted.

Using repository name,
 description,
 and topics only,
 the first three pages introduced `49` unique plausible Pi
subagent or orchestration repositories.
 Page `3` still added plausible repositories,
 including
`vnedyalk0v/pi-subagent-kernel`,
 `QuintinShaw/pi-dynamic-workflows`,
 and
`introspection-recipes/pi-multi-agent-research`,
 so the two-page no-survivor rule was not met.

GitHub's official current search documentation states that each search returns at most `1,000` results and searches at
most `4,000` matching repositories:
<https://docs.github.com/en/rest/search/search?apiVersion=2022-11-28#about-search> and
<https://docs.github.com/en/rest/search/search?apiVersion=2022-11-28#search-scope-limits>.
 Four frozen queries report
between `1,192` and `23,764` matches.
 The provider cap arrives before exhaustion,
 while new plausible candidates still
appear in sampled pages.

#### Broader-web result sets

The configured search returned `10` results for each frozen query,
 with no include or exclude filters.
 Across the sets it
found the user-named candidate and alternatives or comparable technologies including:

- `tintinweb/pi-subagents`,
   `mjakl/pi-subagent`,
   `jwu/pi-subagents`,
   `espennilsen/pi`,
   and the official Pi example;
- `QuintinShaw/pi-dynamic-workflows`,
   `Michaelliv/pi-dynamic-workflows`,
  `gtnotacoder/pi-dynamic-workflows`,
   `timbrinded/pi-workflow-engine`,
   and `heggria/taskflow`;
- `betaHi/pi-loom`,
   `5queezer/pi-subflow`,
   `joelhooks/pi-workflow-os`,
   and `umutbasal/pi-workflows`;
- `Tiziano-AI/pi-multiagent`,
   `KristjanPikhof/pi-agents-team`,
   `sandalsoft/pi-fleet`,
   and
  `Hor1zonZzz/pi-codeMode`;
- `davidsunglee/pi-mux-subagents`,
   `Whamp/pi-interactive-subagents`,
   and Catdaemon's subagent package.

These finite result sets are complete for the harness-visible provider response,
 not evidence that the broader web is
exhaustive.

#### Local source class

The uncapped local searches produced:

- `10` fixed-string incumbent matches across the repository;
- `112` broader subagent or workflow matches in `.agents`,
   `.pi`,
   `docs`,
   and manifests;
- `42` relevant report,
   decision,
   and handover paths;
- `854` orchestration-related matches under `package/pi-plugin/`;
- `100` incumbent package and lock matches under `/var/home/user/.pi/agent/npm/`.

The local source class is exhausted for the frozen paths.
 The existing implementation at `package/pi-plugin/spawn/`
and prior `doc/handover/subagent-extension-audit.md` evidence remain parallel-system inputs,
 not ready-made replacements.

### Expansion execution

The npm expansion used the same quality-only API parameters.
 Every query returned three complete pages of `100` results,
and every registry total exceeded the sampled `300`:

- `pi taskflow`:
   total `12,971`;
- `pi workflow engine`:
   total `148,581`;
- `pi interactive subagents`:
   total `131,742`;
- `pi code mode subagents`:
   total `1,837,310`;
- `pi agent swarm`:
   total `103,795`;
- `pi workflow TUI`:
   total `102,809`.

The expansion confirms that npm token search is not an uncapped category enumeration substitute for GitHub.
 Its broad
matching introduces unrelated packages deep into the result set and does not prove exhaustion.

The GitHub exact-name expansion exhausted every query:

- `pi-taskflow in:name`:
   `23` results;
- `pi-workflow-engine in:name`:
   `12` results;
- `pi-interactive-subagents in:name`:
   `3` results;
- `pi-codeMode in:name`:
   `6` results;
- `pi-swarm in:name`:
   `255` results across pages of `100`,
   `100`,
   and `55`;
- `pi-loom in:name`:
   `50` results.

The broader-web expansion returned `10` results per query and rediscovered `heggria/taskflow`,
`timbrinded/pi-workflow-engine`,
 `QuintinShaw/pi-dynamic-workflows`,
 `betaHi/pi-loom`,
`Hor1zonZzz/pi-codeMode`,
 `gjczone/pi-swarm`,
 `nicobailon/pi-subagents`,
 and several interactive-subagent forks.
 It added
no taxonomy query because the single expansion round was already frozen.

### Discovery status

**Reopened after independent review.**
 The initial run treated GitHub's documented `1,000`-result search cap as a terminal
block.
 An independent review pointed out that the frozen queries were too broad (`pi dynamic workflows` matched `23,764`
repositories) and that non-overlapping date partitions bring each partition under the cap.
 A probe partition using
`created:2025-08-01..2025-09-01` returned `10`,
 `10`,
 `681`,
 and `100` results across the four base queries;
 the largest
partition total was `681`,
 under the cap.
 Date-range partitioning is therefore a viable uncapped enumeration path that the
initial run did not exhaust.

Partitioned enumeration across `12` half-month windows from `2025-08-01` to `2026-07-15` was started
but not completed in
this session.
 Discovery is no longer terminally blocked;
 it is incomplete.
 The named candidate was still excluded on its
own hard-gate failures (independent of discovery),
 so the incomplete discovery does not rescue it.
 A follow-up session that
completes partitioned enumeration,
 screens survivors,
 and validates finalists at equal depth could still reach a
recommendation among the other ready-made alternatives.

## Evidence records

### Source and artifact identity

#### Candidate `@quintinshaw/pi-dynamic-workflows` `2.12.1`

- Clone:
   `/tmp/agent/quintinshaw-pi-dynamic-workflows-20260710` at
  `df334361b1149b7b9a129c720c0d8c287838be8a`,
   exact annotated tag `v2.12.1`,
   origin
  <https://github.com/QuintinShaw/pi-dynamic-workflows>.
- npm artifact:
   `205,649` bytes,
   SHA-512
  `WjKcQHLEENutv2bnqEeKq5hvi2/qeyQKvVQtDH6RQFuRey1hXnKjh3o+Iy5W+nN2Jv3goL0rngy56AdZDZTf0A==`,
   `100`
  files.
   The downloaded bytes matched the registry integrity value at
  <https://registry.npmjs.org/@quintinshaw%2Fpi-dynamic-workflows/2.12.1>.
- Every published `src/` and `extensions/` file matched the tagged clone byte for byte.
   The MIT license also matched.
- The npm SLSA provenance maps the artifact to the tag,
   workflow `.github/workflows/release.yml`,
   and commit
  `df334361b1149b7b9a129c720c0d8c287838be8a`:
  <https://registry.npmjs.org/-/npm/v1/attestations/@quintinshaw%2fpi-dynamic-workflows@2.12.1>.
- Release provenance status:
   pass.
   The release job runs `npm ci`,
   the default test task,
   a tag-to-version check,
   TypeScript
  build,
   and `npm publish --provenance` in `.github/workflows/release.yml:15-59`.
- License status:
   pass for MIT compatibility.
- Candidate-specific native,
   Wasm,
   downloaded-runtime,
   and install-hook status:
   pass.
   The published package declares one
  runtime dependency,
   pure-JavaScript `acorn`;
   it declares no `preinstall`,
   `install`,
   or `postinstall` script.
   Pi and
  `typebox` remain host peer boundaries rather than bundled candidate dependencies.

#### Incumbent `@narumitw/pi-subagents` `0.13.0`

- Clone:
   `/tmp/agent/narumiruna-pi-extensions-20260710` at
  `462c38cbeabd821a32f3731409658cf36e92c5f4`,
   exact tag `v0.13.0`,
   origin
  <https://github.com/narumiruna/pi-extensions>.
- npm artifact:
   `21,614` bytes,
   SHA-512
  `18d4NEDM+dIQahzDukeg2RebMDx5vitXS44hgtRSLmPGTJAZfL4sB1SHc1UuUxV1rIFFAc1Us6n/xxd2cfmxcw==`,
   `5` files.
   Published
  source and MIT license matched the tag byte for byte.
- Registry provenance is present at
  <https://registry.npmjs.org/-/npm/v1/attestations/@narumitw%2fpi-subagents@0.13.0>.

### High-trust security audit

#### Hard failure: generated workflow code can escape the advertised VM boundary

Candidate source explicitly acknowledges the boundary failure at `src/workflow.ts:235-239`:

```ts
Using the vm realm's own Math/Date/Reflect (not host objects) means this adds
no host-`Function` escape. Note: vm is not a security sandbox — an injected
bridge function's `.constructor` is still the host Function, so a determined
script could bypass this. The guard is best-effort against ACCIDENTAL
nondeterminism from trusted (user / guided-LLM) scripts, not a security wall.
```

The runtime injects the host `agent`,
 orchestration,
 logging,
 checkpoint,
 and other bridge functions into the context,
then executes the model-produced body at `src/workflow.ts:879-911`:

```ts
const context = vm.createContext({
  agent,
  parallel,
  pipeline,
  // ... more host bridge functions ...
});
const wrapped = `${DETERMINISM_PRELUDE}\n(async () => {\n${body}\n})()`;
const result = await new vm.Script(wrapped).runInContext(context);
```

A bounded Node reproduction used an environment variable containing only `sentinel-only`;
 it read that host value and
confirmed host `fetch` visibility through the same async bridge-function constructor path:

```text
cwd: /tmp/agent/pi-dynamic-artifacts
command: VM_ESCAPE_SENTINEL=sentinel-only node --input-type=module-typescript -e '<vm reproduction>'
OS/architecture: Linux x86_64
Node: 26.5.0
exit: 0
output: sentinel-only
second output: ["object","function"]
```

This is not hidden malware.
 It is a documented design compromise whose README nevertheless calls the script sandboxed
and says `fs` and network are unavailable at `README.md:70`,
 `README.md:88`,
 and `README.md:201`.
 In this trust boundary,
a repository prompt injection can influence the parent model's generated workflow body,
 so "guided-LLM" does not make
that body trusted.
 The script can reach ambient process environment and host networking.
 **Security-boundary hard gate:
fail.**

#### Hard failure: agent tool allowlists do not constrain the real Pi session

The candidate filters a locally created tool array at `src/agent.ts:368-375`,
 but passes it only as `customTools` at
`src/agent.ts:411-431`.
 It does not pass Pi SDK `tools`,
 `excludeTools`,
 `noTools`,
 or an extension-free resource loader:

```ts
const customTools = applyToolPolicy([...baseTools, ...(options.tools ?? [])], options.toolNames,
  options.disallowedToolNames);
const { session } = await createAgentSession({
  cwd: runCwd,
  agentDir,
  settingsManager: SettingsManager.create(this.cwd, agentDir),
  customTools,
});
```

Installed Pi `0.80.6` creates a default resource loader and activates `read`,
 `bash`,
 `edit`,
 and `write` whenever the
caller omits the SDK-level tool policy at
`node_modules/.pnpm/@earendil-works+pi-coding-agent@0.80.6/node_modules/@earendil-works/pi-coding-agent/dist/core/sdk.js:63-77`
and `:131-135`.
 Its session constructor also enables all extension tools at
`dist/core/agent-session.js:127-150` and `:1911-1977`.

Consequences:

- an `agentType` allowlist that says only `read` does not remove SDK `bash`,
   `edit`,
   or `write`;
- global and project extensions load into child sessions and their tools become active;
- the candidate's `systemTools` intentionally bypass its own policy at `src/agent.ts:377-379`.

The candidate tests verify that a filtered array reaches a fake agent runner.
 They do not inspect the real session's active
registry.
 `CONTRIBUTING.md` correctly warns that real SDK behavior differs from mocks,
 but the current implementation does
not apply the SDK boundary.
 **Read-only and no-ambient-extension hard gate:
 fail.**

#### No covert network or telemetry path found

An uncapped search across production source found no analytics,
 telemetry service,
 webhook,
 encoded payload,
 install hook,
fixed exfiltration endpoint,
 or hidden downloader.
 Network access exists only through model providers inherited from Pi and
the explicit `web_search` and `web_fetch` tools in `src/web-tools.ts`.
 The security result is therefore "unsafe boundary",
not "backdoor".

### Lifecycle and operator-control audit

#### Hard failure: per-agent timeout reports completion without aborting the agent

The candidate starts `agentRunner.run(...)` with the workflow-wide signal at `src/workflow.ts:494-528`.
 Its timeout helper
at `src/workflow.ts:1156-1177` races that promise against a rejecting timer:

```ts
return await Promise.race([promise, timeoutPromise]);
```

The timeout does not create or abort a per-agent controller.
 The original agent promise therefore keeps running,
 can keep
calling tools and spending tokens,
 and can overlap retry attempts.
 If worktree isolation was active,
 the outer `finally`
can remove the worktree while the timed-out session still uses it.
 The timeout unit test at `tests/agent.test.ts:607-632`
checks only the early `null` result;
 it does not assert cancellation or absence of later side effects.
 **Per-child hard
timeout gate:
 fail.**

The incumbent uses a detached subprocess group and sends `SIGTERM`,
 then `SIGKILL` after a grace period at
`extensions/pi-subagents/src/subagents.ts:370-397` and `:482-497`;
 its timeout is materially harder than the candidate's.

#### Hard failure: operator actions address runs, not selected children

The navigator resolves pause and stop through `state.activeRunId(model)` and calls `manager.pause(id)` or
`manager.stop(id)` at `src/workflow-ui.ts:1045-1053`.
 Agent detail view has no child-specific stop action.
 The in-process
child has no Zellij,
 tmux,
 terminal pane,
 or child TUI for direct human interaction.
 **Every-child user-interruption gate:
fail.**

#### Hard failure: observable history is intentionally incomplete

`src/agent-history.ts:20-27` caps history at `40` entries,
 `2,000` characters per entry,
 and `20,000` total characters.
`src/agent-history.ts:90-95` keeps only the newest entries and truncates text.
 Persisted run state stores that compact array,
not the underlying in-memory Pi session.
 Open pull request
<https://github.com/QuintinShaw/pi-dynamic-workflows/pull/54> independently proposes durable subagent transcripts.
**Complete observable child activity gate:
 fail.**

### Replacement parity

The installed incumbent exposes one parent tool with single,
 parallel,
 chain,
 fan-in aggregator,
 custom agent,
 per-task
working-directory,
 per-task thinking,
 and per-task timeout fields at
`extensions/pi-subagents/src/subagents.ts:605-653`.
 The candidate exposes a model-generated JavaScript orchestration
contract with parallel,
 pipeline,
 nested workflow,
 persistence,
 background execution,
 and richer model routing.
 It is a
broader but different parent-model interface,
 not a drop-in replacement.

- Candidate advantage:
   persistent run history,
   background result delivery,
   model tiers,
   token accounting,
   resume,
  worktrees,
   and orchestration combinators.
- Incumbent advantage:
   `1,723` code lines in `2` source files versus candidate `6,732` code lines in `33` source files;
  process-group timeout and teardown;
   direct declarative chain and aggregator schema.
- Candidate migration cost:
   prompts and agent guidance must move from the `subagent` schema to generated workflow scripts;
  stored agent definitions are partially reused,
   but current tool policy is not enforced at the SDK boundary.
- Conditional custom fallback:
   the recorded Zellij preference directly addresses the candidate's missing human control,
  but no custom design is eligible while ready-made technologies remain unexhausted.

### Test, CI, and maintenance audit

- Candidate source:
   `6,732` code lines in `33` non-test TypeScript files;
   tests:
   `14,054` physical lines across `37` test
  files.
   Incumbent source:
   `1,723` code lines in `2` files.
- Candidate lockfile contains `283` package entries because it locks development and Pi peer validation.
   Published runtime
  adds one direct dependency,
   `acorn`;
   the installed host supplies Pi,
   TUI,
   and `typebox` peers.
- Default CI runs `npm ci` and `npm test` on Ubuntu with Node `22` in `.github/workflows/ci.yml:14-26`.
   There is no Windows,
  macOS,
   Node `26`,
   or multi-Pi-version job.
- Unit tests include `fast-check` properties for model-spec parsing.
   Searches found no fuzz harness,
   mutation suite,
  coverage command,
   published coverage report,
   or end-to-end test job.
   `CONTRIBUTING.md` requires maintainers to run real
  model checks manually for runtime changes,
   so those checks are policy rather than reproducible CI evidence.
- GitHub API sample for the year ending 2026-07-10 found `24` issues and `35` pull requests.
   Following the frozen method,
  this audit inspected the `10` most recently updated of each at
  <https://github.com/QuintinShaw/pi-dynamic-workflows/issues> and
  <https://github.com/QuintinShaw/pi-dynamic-workflows/pulls>.
- Of the issue sample,
   `7` are closed and `3` open.
   Owner actions linked or closed implemented issues;
   maintainer comments
  appeared on `4` sampled issues.
   This is active release maintenance with selective public discussion,
   not abandonment.
- Of the pull-request sample,
   `3` merged in `22.4`,
   `81.2`,
   and `167.1` hours;
   `4` remain open and `3` closed unmerged.
  Maintainer prose comments appeared on `7` sampled pull requests,
   but GitHub recorded no formal review event in the sample.
- GitHub lists `34` releases from `v1.0.0` on 2026-05-30 to `v2.12.1` on 2026-07-10.
   Contributor data assigns `63` of `84`
  commits to the owner,
   exactly `75%`,
   so bus-factor concentration remains material.
   Primary release record:
  <https://github.com/QuintinShaw/pi-dynamic-workflows/releases/tag/v2.12.1>.

### Candidate hard-gate outcome

`@quintinshaw/pi-dynamic-workflows` `2.12.1` exits before finalist validation.
 It passes source availability,
 MIT license,
artifact mapping,
 release provenance,
 candidate-specific native/Wasm,
 and category-fit gates.

Decisive hard failures (each independently sufficient to exclude):

- High-trust security boundary:
   the `node:vm` sandbox is escapable via an injected bridge function's `.constructor`,
  contradicting the README's sandbox claim.
   Reproduced.
- Per-child hard timeout:
   `timeoutMs` reports a recoverable `null` without cancelling the agent;
   the child keeps running.
  Reproduced.
- Enforced read-only capability:
   `agentType` tool allowlists filter only a local array;
   the real Pi SDK session still
  activates `read`,
   `bash`,
   `edit`,
   and `write`.
   Reproduced.

Qualified findings (not counted as decisive until the requirement interpretation is confirmed):

- Per-child operator interruption:
   UI `pause` and `stop` target whole runs,
   not individual children.
   This is a hard
  failure only if "interrupt any running subagent" was frozen to mean selecting an individual child without stopping
  siblings.
   The locked requirement is ambiguous on that point.
   Run-level stop does halt the run's children collectively.
- Complete observable transcript:
   persisted history is capped at `40` entries and `20,000` characters.
   This is a hard
  failure for durable revisitable history.
   Live UI observability was not fully traced in this audit and may be broader;
  the open <https://github.com/QuintinShaw/pi-dynamic-workflows/pull/54> corroborates the durable-transcript gap.

The ambient-extension claim (project and global extensions load into child sessions) is source-indicated but not
runtime-reproduced;
 the probe confirmed built-in `bash`/`edit`/`write` remain active,
 which is already decisive.

## Execution manifests

### Candidate hard-gate reproduction harness

- Candidate,
   revision,
   and clone origin:
   `@quintinshaw/pi-dynamic-workflows` `2.12.1`,
   commit
  `df334361b1149b7b9a129c720c0d8c287838be8a`,
   cloned from
  <https://github.com/QuintinShaw/pi-dynamic-workflows> into
  `/tmp/agent/quintinshaw-pi-dynamic-workflows-20260710`.
- Top-level command:
   `podman run --rm --memory=2g --cpus=2 --env=VM_ESCAPE_SENTINEL=sentinel-only --volume /tmp/agent/quintinshaw-pi-dynamic-workflows-20260710:/candidate:ro --volume /tmp/agent/pi-dynamic-artifacts/candidate-hard-gates.mjs:/probe.mjs:ro --workdir /candidate docker.io/library/node:24-slim sh -c 'npm ci --no-audit --no-fund && npm run build && node /probe.mjs'`.
- Statically discovered command tree:
   `npm ci` resolves the lockfile and may run `preinstall`,
   `install`,
   and `postinstall`
  scripts from devDependencies.
   The candidate package itself declares no install script;
   the only devDependency with a
  postinstall is `esbuild` (binary download).
   `npm run build` runs `tsc`.
   `node /probe.mjs` imports built candidate modules
  and the Pi SDK to reproduce three hard-gate findings.
- Expected reads:
   `/candidate/dist/**`,
   `/candidate/node_modules/**`,
   `/candidate/package.json`,
   `/probe.mjs`.
- Expected writes:
   `/candidate/node_modules/**`,
   `/candidate/dist/**` (build output),
   `/tmp/sdk-cwd`,
  `/tmp/empty-agent-home` (inside the container's private scratch filesystem).
- Expected subprocesses:
   `npm`,
   `node`,
   `tsc`,
   `esbuild`'s installed binary.
- Expected network endpoints:
   `registry.npmjs.org` for `npm ci`,
   then none.
- Resource ceilings:
   2 GiB memory,
   2 CPUs,
   wall-clock bounded by the process tool.
- Credential,
   environment,
   home-directory,
   network,
   and repository-mount policy:
   no ambient credentials;
   only
  `VM_ESCAPE_SENTINEL` is injected as a probe value;
   no real home-directory mount;
   the repository clone is mounted
  read-only;
   the probe script is mounted read-only;
   network is allowed only during `npm ci` and cannot reach
  user secrets.
- Stop conditions:
   the probe prints three JSON lines and exits,
   or the container exceeds its memory or CPU ceiling.

## Hard-gate exits

- `@quintinshaw/pi-dynamic-workflows` `2.12.1`:
   excluded for its own hard-gate failures (security boundary,
   timeout
  cancellation,
   and SDK tool allowlist),
   with two qualified findings (per-child interruption and history completeness).
- Candidate scoring is prohibited.
   Soft feature breadth cannot offset a hard failure.
- Discovery is incomplete,
   not terminally blocked (see Discovery status).
   Other candidates were not fully screened or
  validated;
   this report does not rank them.


## Validation results

### Reproduction environment

- Container image:
   `docker.io/library/node:24-slim`,
   Node `v24.18.0`,
   npm `11.16.0`.
- Isolation:
   `podman run --rm --memory=2g --cpus=2 --userns=keep-id --env=VM_ESCAPE_SENTINEL=sentinel-only`.
- Mounts:
   `/tmp/agent/work:/work:rw,Z` (writable copy of the tagged clone) and the probe script read-only with `:Z`
  relabeling.
   SELinux on the host is `Enforcing`;
   without `:Z` the container could not read host files.
- Build:
   `npm ci --no-audit --no-fund` installed `251` packages;
   `npm run build` (`tsc`) succeeded.
   npm `11` blocked
  install scripts for `@google/genai`,
   `esbuild`,
   and `protobufjs` under its `allowScripts` policy,
   so no third-party
  postinstall ran.
- Probe:
   `/tmp/agent/pi-dynamic-artifacts/candidate-hard-gates.mjs`,
   log at
  `/tmp/agent/pi-dynamic-artifacts/logs/candidate-hard-gates.log`.
   Exit status `0`.

### Reproduced failures

The probe output,
 verbatim:

```text
{"probe":"vm-escape","result":["sentinel-only","function"]}
{"probe":"timeout-immediate","result":null,"lateAgentCompleted":false}
{"probe":"timeout-after-wait","lateAgentCompleted":true}
{"probe":"sdk-tool-policy","candidateFilteredCustomTools":["read"],"realActiveTools":["read","bash","edit","write"]}
```

#### VM boundary escape

A workflow body `return await agent.constructor('return [process.env.VM_ESCAPE_SENTINEL, typeof fetch]')()` returned
`["sentinel-only","function"]`.
 The script read the injected host environment value and confirmed host `fetch` is a
function.
 The README's "sandboxed script" and "fs/network are unavailable" claims are contradicted by this run.
 Hard
failure confirmed.

#### Timeout does not cancel the agent

With `timeoutMs: 5` and an agent that sleeps `80ms`,
 the immediate result was `null` (the recoverable timeout path),
 and
`lateAgentCompleted` was `false` right after the timeout.
 After waiting `120ms`,
 `lateAgentCompleted` became `true`.
 The
timed-out agent kept running,
 completed its body,
 and could have continued calling tools.
 Hard failure confirmed.

#### Tool allowlist does not constrain the real session

The candidate's `applyToolPolicy(..., ['read'], undefined)` produced `["read"]`,
 but `session.getActiveToolNames()`
returned `["read","bash","edit","write"]`.
 The Pi SDK defaults overrode the candidate's allowlist,
 so a "read-only"
agentType still receives `bash`,
 `edit`,
 and `write`.
 Hard failure confirmed.

### CI-equivalent suite

The candidate is excluded,
 so the full `679`-test suite is outside the decisive surface.
 The build and the probe together
constitute the reproducible validation.
 `CONTRIBUTING.md` confirms the unit tests use fake agents that do not surface
these failures,
 which is consistent with the tests passing while the real SDK boundary remains unenforced.

## Score arithmetic and sensitivity

Score:
 not applicable.
 `@quintinshaw/pi-dynamic-workflows` `2.12.1` failed decisive hard gates,
 so it is not a validated
finalist and cannot be scored.
 Hard gates remain outside arithmetic and cannot be offset by points.
 No other candidate
reached finalist validation,
 because discovery was reopened and not completed.
 With no validated finalists,
 the weighted
rubric has no scored subjects and no sensitivity matrix to run.

## Pros, cons, ranking, and recommendation

### `@quintinshaw/pi-dynamic-workflows` `2.12.1`

Pros:

- Strong release cadence and provenance:
   `34` tagged releases in six weeks,
   npm SLSA provenance,
   annotated tag carrying an unverified PGP signature (local Git reported `Can't check signature: No public key`);
   npm SLSA provenance is the verified mapping,
  and tag-to-version verification in CI.
- Real token and cost accounting read from subagent sessions,
   journaled resume with longest-unchanged-prefix replay,
  per-phase model routing,
   git-worktree isolation,
   background runs with result delivery,
   and a rich `/workflows` TUI.
- One direct runtime dependency (`acorn`);
   no install hooks in the published package;
   no covert telemetry or backdoor
  found in production source.

Cons:

- Hard failure:
   the `node:vm` boundary is documented as not a security sandbox,
   yet the README advertises a sandboxed
  script with `fs`/network unavailable.
   A model-generated body can read ambient `process.env` and reach host `fetch` via
  the injected `agent.constructor`.
- Hard failure:
   per-agent `timeoutMs` reports a recoverable `null` without cancelling the agent,
   so a timed-out child
  keeps running,
   keeps spending tokens,
   and can outlive its worktree.
- Hard failure:
   `agentType` tool allowlists filter only a local array;
   the real Pi session still activates
  `read`,
   `bash`,
   `edit`,
   and `write`,
   so "read-only" is not enforced.
- Hard failure:
   operator pause and stop target whole runs;
   no per-child interruption and no direct child TUI exists.
- Hard failure:
   observable history is capped at `40` entries and `20,000` total characters and is not a complete
  transcript.
- Material audit surface:
   `6,732` code lines across `33` source files,
   `283` lockfile packages,
   `75%` owner commit
  concentration,
   and no fuzz,
   mutation,
   coverage,
   or end-to-end CI job.

### `@narumitw/pi-subagents` `0.13.0` (incumbent baseline)

Pros:

- Small surface:
   `1,723` code lines in `2` source files,
   one runtime dependency (`typebox`).
- Real subprocess isolation with process-group `SIGTERM` then `SIGKILL` teardown,
   so a timeout or abort actually stops
  the child.
- Passes `--tools` or `--no-tools` to the `pi` CLI,
   so the allowlist is enforced by the real runtime,
   not a local array.
- Per-task `timeoutMs`,
   `thinkingLevel`,
   `cwd`,
   chain,
   parallel,
   and aggregator fields in the tool schema.

Cons:

- No interactive per-child TUI;
   the user cannot enter or steer a running child session directly.
- Child activity surfaces through streamed status and a final result,
   not a complete persisted transcript.
- `1,980` physical lines;
   single test file;
   no fuzz,
   mutation,
   or coverage evidence.
- The user explicitly asked to replace it,
   so it is the comparison baseline,
   not a recommendation.

### Ranking

1. `@narumitw/pi-subagents` `0.13.0` (incumbent) beats
   `@quintinshaw/pi-dynamic-workflows` `2.12.1` because the candidate
   fails six hard gates that the incumbent's subprocess model does not fail in the same way.
    The incumbent actually
   kills children on timeout and routes tool policy through the real `pi` runtime.
2. `@quintinshaw/pi-dynamic-workflows` `2.12.1` is excluded.
    Its feature breadth is irrelevant once hard gates fail.

This ranking is conditional and incomplete.
 It compares only the incumbent and the single user-named candidate.
 Other
ready-made alternatives exist;
 discovery was reopened and not completed,
 so they are not ranked.

### Recommendation

No recommendation to adopt `@quintinshaw/pi-dynamic-workflows` `2.12.1`.
 Its three decisive hard-gate failures,
reproduced in a disposable,
 secret-free,
 resource-bounded container,
 independently exclude it.
 Soft scoring cannot
rescue a hard failure.

The selection as a whole is incomplete rather than terminally blocked.
 GitHub discovery was initially treated as blocked
by the `1,000`-result cap,
 but an independent review identified date-range partitioning as a viable uncapped path (a
probe partition returned at most `681` results).
 Partitioned enumeration was started but not completed.
 A follow-up
session that finishes partitioned discovery,
 screens survivors,
 and validates finalists at equal depth could still
reach a recommendation among the other ready-made alternatives.

### What the user can do next

The recorded Zellij fallback directly addresses the candidate's missing per-child control and transcript visibility,
 but
it is not eligible for a recommendation here.
 The existing-tools-first gate requires every ready-made technology to fail a
named hard constraint,
 and the blocked GitHub source means that exhaustion is not proven.
 A later session that unblocks
enumeration,
 or that narrows the hard constraints with the user,
 could reopen finalist validation.

If the user instead wants to keep the incumbent and patch its missing pieces,
 the honest path is a separate
implementation request,
 not part of this evaluation.
 This audit did not change product code,
 dependencies,
 configuration,
or any decision record.

## Confidence and evidence limits

- Source and artifact claims cite clone paths,
   commit,
   tag,
   npm integrity,
   and `path:line` ranges.
   Reproduction claims
  cite the exact container command,
   resource ceilings,
   and verbatim probe output.
- GitHub maintenance counts use the frozen year window and the `10` most recently updated items.
   They are a maintainer
  activity sample,
   not a complete tracker census.
- Discovery did not saturate.
   The candidate ledger is deliberately incomplete;
   any "no other alternatives exist" claim
  would be unsupported.
- The VM escape reproduction used a synthetic environment variable and `typeof fetch`;
   it did not attempt a full
  filesystem read or network call.
   The `ERR_VM_DYNAMIC_IMPORT_CALLBACK_MISSING` result shows `import()` is not directly
  available,
   but `process.env` and `fetch` are,
   which is enough to contradict the advertised boundary.
- The SDK allowlist probe ran against installed Pi `0.80.6`.
   A future Pi SDK change that lets callers pass `tools` or
  `noTools` through `createAgentSession` could change this finding;
   the candidate would still need to call it.
