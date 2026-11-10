# Technology vet: cli-git parser migration to CAC

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

Status:
 superseded before external evidence collection.
 Lifecycle phase stopped at discovered.
 Started and last updated on 2026-08-14.

The user narrowed the assessment to CAC only.
The active report is
`doc/audit/tech-cli-git-parser-migration-to-cac-vet-2026-08-14-a48b54e2.md`.
This broader-scope report is retained because its compatibility fingerprint includes external alternative discovery.

Subject:
 cli-git parser migration to CAC.

Decision scope:
 assess whether CAC should replace any or all repository-owned parsers in
`@monochromatic-dev/git-policy-cli`,
including management-only and Git-region integration shapes.

Governing skill:

- Commit is `a05818ad70a40e5769a36de669697ba109891b31`.
- SHA-256 is `393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`.

Compatibility fingerprint:
 `a0ef33c2a224a2928cec31b25feec57b9933af367edb6087660a0f485e96be05`.

Active audit owner:
 Pi session `01a00016-a9a0-747d-8174-191632f05b8e`.

Prior compatible report:
 none found.

## Context

The user asked whether migrating cli-git to CAC is practical.
CAC is interpreted as the open-source npm CLI framework.
The target is `package/git-policy/cli/`.
This report is an evaluation,
not adoption authority.
Product code,
dependencies,
configuration,
lockfiles,
and decision records remain unchanged.

The target no longer uses a third-party CLI framework.
Repository history shows that Optique was removed on 2026-07-15 by commits
`1e53f52e02989b152576c0683f6f228b24c40140`,
`4879a44e6a2460ab3c2531744e24a1f2aef27aeb`,
and
`be3c522375d9c2652f44921c7396233c93c2397d`.
The incumbent is therefore the repository-owned parser set.

The current parsing surface has separate roles:

- `src/parse-global-options.ts` locates Git's effective subcommand and applies chained `-C` semantics;
- `src/management-parser.ts` implements the closed `git cli-git` namespace;
- `src/parser/argv.ts` parses declared subsets of Git argv while retaining undeclared options;
- sibling parsers model Git-specific abbreviations,
  short clusters,
  option arity,
  last-option-wins toggles,
  pathspecs,
  and implicit branch creation.

The detailed incumbent inventory and live execution state are in
`doc/handover/cli-git-cac-migration.md`.

## Classification

The incumbent is repository-owned inspectable local technology.
External parser candidates must be inspectable open-source local technologies.

Applicable overlays:

- incumbent dependency replacement;
- high-trust execution,
  because the package shadows `git`,
  classifies and transforms commands,
  and gates safety policies;
- human auditability,
  because parser facts determine whether Git is blocked,
  transformed,
  or forwarded;
- multi-platform behavior across Linux,
  macOS,
  and Windows.

The native,
Wasm,
and prebuilt overlay applies only when a candidate introduces such a boundary.
A candidate with no such boundary records it as not applicable with inspected dependency evidence.
The managed-service,
SaaS,
privacy,
residency,
and browser overlays do not apply because parsing is local and receives argv rather than hosted or browser data.

## Hard constraints

A candidate and its proposed integration shape must satisfy every constraint:

- compatible inspectable open-source license;
- source-to-package provenance;
- no unaudited native,
  Wasm,
  generated,
  downloaded,
  or prebuilt runtime boundary;
- package Node range `^22.18.0 || >=24.11.0`;
- Linux,
  macOS,
  and Windows support;
- one self-contained MJS artifact and side-effect-free package import;
- exact pathspec and raw argv fidelity until cli-git explicitly owns a transformation;
- fail-closed guarded Git command classification;
- complete management grammar,
  output routing,
  early-help,
  and exit contracts from `package/git-policy/cli/SPEC.md`;
- relevant upstream and cli-git consumer-boundary validation;
- wrapper-added `wide-commit` latency at or below the maintained 925-millisecond ceiling.

## Frozen criteria

No priority ordering was supplied,
so every criterion has weight 1:

- net removal of repository-owned parser and adapter code;
- human auditability of parsing and forwarding;
- runtime and bundle overhead;
- direct and transitive runtime dependency surface;
- TypeScript inference and declaration compatibility;
- help and diagnostic control without process-global interception;
- upstream maintenance and release health;
- migration and regression-test burden;
- future management-command extensibility;
- fit with the existing pure-parser plus process-adapter seam.

Each criterion uses the rating scale from 0 through 4.
The maximum score is 40.
Hard-gate failures stay outside arithmetic.
No unresolved preference exists before evidence collection.

## Frozen discovery schedule

### Npm registry

Use the official npm registry search API with 50 results per page,
default relevance order,
and no negative filter.
Run each literal query:

- `command line parser typescript`;
- `cli framework typescript`;
- `argv parser typescript`;
- `zero dependency cli parser`;
- `cac alternative cli`.

For each query,
continue until the registry reports exhaustion or two consecutive complete pages add no screening survivor.

### GitHub repositories

Use GitHub repository search with 100 results per page,
default best-match order,
and no negative filter.
Run each literal query:

- `command line parser language:TypeScript`;
- `cli framework language:TypeScript`;
- `argv parser language:TypeScript`;
- `zero dependency cli parser language:TypeScript`;
- `cac alternative cli language:TypeScript`.

For each query,
continue until GitHub reports exhaustion or two consecutive complete pages add no screening survivor.

### Broader web

Use the configured web search provider for each literal query:

- `CAC npm alternative TypeScript CLI parser`;
- `JavaScript CLI framework allow unknown options parse argv`;
- `TypeScript CLI parser preserve double dash passthrough`;
- `CAC versus Commander CLI framework`;
- `TypeScript Git argv parser library`.

The provider exposes no page cursor through this harness.
Record that limit and corroborate candidates through registry and repository enumeration.

### This repository

Run uncapped searches for:

- `cac`;
- `@optique` and `Optique`;
- `parseArgs`;
- `commander`;
- `yargs`;
- `clipanion`;
- `cleye`;
- `citty`;
- `sade`;
- `mri`;
- `minimist`;
- `arg`;
- `argv` plus `parser`.

Inspect matching manifests,
catalog entries,
lockfile entries,
source,
tests,
troubleshooting records,
plans,
decisions,
audits,
and handovers.

### Expansion round

After the initial schedule:

1.  Collect every new problem-class synonym in candidate metadata.
2.  Append one de-duplicated query round to each applicable source.
3.  Freeze the expanded schedule.
4.  Record later terms without adding queries.

## Query ledger and saturation

No external query has run yet.
No source class is saturated.

## Candidate ledger

### Repository-owned cli-git parsers

Discovery source:
 current repository manifest,
source,
tests,
history,
and user request to assess replacement.

Base category:
 repository-owned inspectable local technology.

Overlays:
 incumbent replacement,
high-trust,
human auditability,
and multi-platform.

Screening result:
 pending complete parity inventory and validation.
Keeping the incumbent remains a candidate.

### CAC

Discovery source:
 user-named candidate.

Base category:
 pending confirmation as inspectable open-source local technology.

Overlays:
 incumbent replacement,
high-trust,
human auditability,
and multi-platform.

Screening result:
 pending official registry,
repository,
license,
provenance,
source,
maintenance,
and runtime evidence.

## Evidence records

### Incumbent repository checkpoint

Candidate and revision:
 repository-owned parsers at
`54ad78083e8baf95c62d3b0682843967722c563d` before assessment documentation commits.

Claim and relevance:
 the incumbent is not one generic parser boundary.
The parser directory has twenty production TypeScript files and 3,953 physical lines,
while management parsing is one closed subset.
A migration must name which role it replaces before code-removal claims are meaningful.

Gate:
 replacement parity and human auditability.

Status:
 pass for inspectability;
validation remains pending.

Primary evidence:

- `package/git-policy/cli/src/management-parser.ts`;
- `package/git-policy/cli/src/parser/argv.ts`;
- `package/git-policy/cli/src/parser/branch-create.ts`;
- `package/git-policy/cli/src/parser/commit.ts`;
- `package/git-policy/cli/SPEC.md:500-566`;
- `doc/decision/cli-git-policies-platform.md:318-343`.

Outcome:
retain as finalist unless later validation fails.

## Execution manifests

No third-party command tree has been executed.
Execution manifests will be written after source inspection and before any candidate install,
build,
test,
or consumer probe.

## Hard-gate exits

None yet.

## Validation results

Not started.

## Score arithmetic

Not started.
Only validated finalists will be scored.

## Sensitivity

Not started.

## Pros and cons

Not started.

## Ranking

No ranking or recommendation exists before discovery,
hard-gate confirmation,
equal-depth validation,
scoring,
and sensitivity analysis finish.

## Evidence limits

The historical Optique timing in repository history is motivation for a latency gate,
not evidence about CAC.
The current audit has not yet inspected CAC source or reproduced candidate behavior.
