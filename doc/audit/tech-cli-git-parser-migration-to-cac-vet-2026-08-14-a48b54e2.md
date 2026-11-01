# Technology vet: cli-git parser migration to CAC

Status:
 complete with no validated CAC integration shape.
 Lifecycle phase is terminal hard-gate result.
 Started and last updated on 2026-08-14.

Subject:
 cli-git parser migration to CAC.

Decision scope:
 assess whether CAC is practical for any or all repository-owned parser roles in
`@monochromatic-dev/git-policy-cli`.
Do not evaluate other external CLI parser technologies.

Governing skill:

- Commit is `a05818ad70a40e5769a36de669697ba109891b31`.
- SHA-256 is `393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`.

Compatibility fingerprint:
 `a48b54e274b6d6bda3057027fbcc2ced424ea8c0a00b03a2ce868425a534f986`.

Active audit owner:
 Pi session `01a00016-a9a0-747d-8174-191632f05b8e`.

Prior compatible report:
 none found.
The incompatible broader-scope report is
`doc/audit/tech-cli-git-parser-migration-to-cac-vet-2026-08-14.md`.
It stopped before external evidence collection when the user limited this audit to CAC.

## Context

The user asked whether migrating cli-git to CAC is practical,
then explicitly limited this session to CAC.
CAC is interpreted as the open-source npm CLI framework.
The target is `package/git-policy/cli/`.
The incumbent is inspected only as CAC's replacement-parity baseline.
No other external CLI parser technology will be discovered,
screened,
audited,
scored,
or ranked.
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
- one public MJS application artifact,
  private workspace code bundled,
  every external runtime edge declared in packed dependencies,
  and side-effect-free package import;
- exact pathspec and raw argv fidelity until cli-git explicitly owns a transformation;
- fail-closed guarded Git command classification;
- complete management grammar,
  output routing,
  early-help,
  and exit contracts from `package/git-policy/cli/SPEC.md`;
- relevant upstream and cli-git consumer-boundary validation;
- wrapper-added `wide-commit` latency at or below the maintained 925-millisecond ceiling.

### Single-artifact clarification

The initial shorthand said "one self-contained MJS artifact."
Before scoring,
consumer evidence showed that reading this as "no declared external dependency" contradicts the frozen baseline and
canonical package contract:

- `package/git-policy/cli/README.md` says the tarball contains one `dist/final/node/index.mjs` application artifact;
- the same README permits fallback only to names in cli-git's packed runtime dependencies;
- `package/git-policy/cli/SPEC.md` likewise permits those declared package edges during installed artifact resolution;
- the unchanged application artifact already imports declared external packages such as `valibot`,
  `yuku-parser`,
  and `yuku-ast`.

The requirement therefore means one emitted public application MJS,
no extra private workspace artifact or chunk,
and every external runtime edge declared in the packed manifest.
It does not mean zero installed dependencies.
An externalized CAC import without a packed CAC dependency would still fail.
This is a clarification of the pre-existing baseline contract,
not a candidate-specific relaxation,
so the compatibility fingerprint remains unchanged.
The clarification was recorded before rating CAC in commit `c659bdc88` on 2026-08-14.

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

This is a CAC-only capability audit rather than a category selection.
Search results naming another parser are out of scope and will not become candidates.

### Npm registry

Use the official npm registry package and search APIs with default relevance order and no negative filter.
Run each literal lookup or query:

- exact package metadata for `cac`;
- exact version metadata for the current `cac` release;
- registry search `cac cli`;
- registry search `cac command line parser`.

Continue each search query until the registry reports exhaustion
 or two consecutive complete pages add no new CAC provenance,
package,
or taxonomy evidence.

### GitHub

Use the repository named by official npm metadata.
Inspect its code,
license,
releases,
CI,
security policy,
contribution policy,
issues,
pull requests,
and organization ownership.
Run these literal GitHub searches scoped to that repository where the API supports scope:

- `allowUnknownOptions`;
- `--`;
- `rawArgs`;
- `process.exit`;
- `help`;
- `TypeScript`;
- `Node 22`;
- `Windows`.

For tracker maintenance,
inspect every issue created or updated in the last twelve months when at most twenty exist,
otherwise the ten most recently updated issues.
Inspect the ten most recently updated pull requests by the same method.

### Broader web

Use the configured web search provider for each literal query:

- `CAC npm TypeScript CLI framework official`;
- `CAC allow unknown options parse argv`;
- `CAC double dash passthrough raw argv`;
- `CAC process exit help behavior`;
- `CAC CLI framework Git argv`;
- `CAC npm security vulnerability`.

The provider exposes no page cursor through this harness.
Record that limit and follow claims back to npm or CAC's repository.
Do not evaluate another parser surfaced by these searches.

### This repository

Run uncapped searches for:

- lowercase whole-word `cac`;
- uppercase whole-word `CAC`;
- `@optique` and `Optique` for migration history;
- `parseArgv` and `tryParseArgv`;
- `management-parser`;
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

1. Collect every new CAC synonym or feature term from official metadata.
2. Append one de-duplicated CAC-only query round to each applicable source.
3. Freeze the expanded schedule.
4. Record later terms without adding queries.

## Query ledger and saturation

### Npm registry initial schedule

Exact metadata lookups resolved `cac` latest and `cac@7.0.0` through the official registry.
They identified version 7.0.0,
MIT licensing,
Node `>=20.19.0`,
the repository `cacjs/cac`,
and the signed npm tarball.

The search `cac cli` reported 346,631 results.
The first 50-result page contained `cac`.
The complete pages at offsets 50 and 100 added no CAC package,
provenance source,
or taxonomy evidence.

The search `cac command line parser` reported 593,845 results.
The first 50-result page contained `cac`.
The complete pages at offsets 50 and 100 added no CAC package,
provenance source,
or taxonomy evidence.

These broad totals contain other technologies that are excluded by the user's CAC-only scope.
No result became another candidate.
The two-page CAC evidence saturation rule is met for both registry searches.

### GitHub schedule

Official npm metadata named `https://github.com/cacjs/cac`.
The source clone is
`~/temp/agent/cac-2026-08-14` at signed release tag `v7.0.0`,
commit `77f602fcb2d1e75d24f5ecd94d5bf667acaa857a`.

Uncapped clone searches completed for:

- `allowUnknownOptions`;
- literal `--` handling;
- `rawArgs`;
- `process.exit`;
- help;
- TypeScript;
- Node 22;
- Windows;
- `mri`;
- numeric coercion;
- unused args.

The search schedule located the parsing,
validation,
help,
type,
test,
CI,
and release paths.
It also located current tracker reports for numeric coercion and multi-word commands.
No recursively expanded technology query was added.

### Broader web initial schedule

The six initial searches consistently resolved the official npm package,
`cacjs/cac`,
release `v7.0.0`,
source files,
and current issues.
The security query added independent package-security indexes,
but primary GitHub Advisory Database searches found no advisory whose affected npm package is exactly `cac` or `mri`.

The web provider exposes no page cursor.
Registry and repository evidence independently established package identity and source coverage.
Other parser names in results were excluded without screening under the user's CAC-only scope.

### Repository schedule

Uncapped whole-word searches found no lowercase `cac` or uppercase `CAC` reference outside the new assessment documents.
The target package declares no CAC dependency.

The `parseArgv`,
`tryParseArgv`,
and `management-parser` searches located the current replacement boundary.
The Optique history sanity search found 182 matches across package and documentation scope,
including current historical comments and prior decisions.
The source and package manifest confirm that Optique is no longer a runtime dependency.

### Expansion round

Official metadata added the terms `Command And Conquer`,
`mri`,
`rawArgs`,
`allowUnknownOptions`,
numeric coercion,
and unused args.

The registry expansion query `cac command and conquer mri` reported 1,643,715 results.
Its first 50-result page contained `cac`.
The complete pages at offsets 50 and 100 added no CAC evidence.

The GitHub expansion searched every added term in the pinned clone and current tracker.
The web expansion ran:

- `CAC mri numeric coercion option values issue`;
- `cacjs CAC allowUnknownOptions rawArgs mri`;
- `cacjs CAC unused args v7`;
- `cacjs CAC Command And Conquer CLI package provenance`.

It corroborated issue `cacjs/cac#165`,
issue `cacjs/cac#162`,
the v7 unused-argument release change,
the official release,
and npm provenance.
The expansion round is frozen.

### Terminal discovery result

CAC-only discovery is saturated with the one user-named candidate.
The source classes identify no second CAC package or independent implementation.
Category-wide alternative saturation is intentionally excluded by the user's explicit scope.

## Candidate ledger

### Incumbent parity baseline

Discovery source:
 current repository manifest,
source,
tests,
history,
and user request to assess replacement.

Role:
 replacement-parity baseline only.
It is not an external technology candidate in this CAC-only audit.

Screening result:
 not applicable.
Its current behavior and maintained tests define what CAC integration must preserve.

### CAC 7.0.0

Discovery source:
 user-named candidate,
then official npm metadata and `cacjs/cac` release `v7.0.0`.

Base category:
 inspectable open-source local technology.

Overlays:
 incumbent replacement,
high-trust,
human auditability,
and multi-platform.
The native,
Wasm,
and prebuilt overlay is not applicable to the published runtime after inspecting the tarball and inlined parser source.

Screening result:
 serious alternative.

Screening evidence:

- MIT license text is present in source and tarball;
- npm metadata declares Node `>=20.19.0`,
  compatible with cli-git's narrower Node range;
- published metadata declares no runtime,
  optional,
  or bundled dependency;
- the build statically inlines `mri` 1.2.0,
  whose MIT source is separately inspectable;
- the tarball has five files,
  41,198 unpacked bytes,
  no lifecycle script,
  one 19,503-byte ESM runtime,
  and one 4,871-byte declaration file;
- npm integrity is
  `sha512-tixWYgm5ZoOD+3g6UTea91eow5z6AAHaho3g0V9CNSNb45gM8SmflpAc+GRd1InC4AqN/07Unrgp56Y94N9hJQ==`;
- measured tarball SHA-512 is
  `b62c566209b9668383fb783a51379af757a8c39cfa0001da868de0d15f4235235be3980cf1299f96901cf8645dd489c2e00a8dff4ed49eb829e7a63de0df6125`;
- npm publish and SLSA attestations bind that digest to `cacjs/cac`,
  tag `v7.0.0`,
  commit `77f602fcb2d1e75d24f5ecd94d5bf667acaa857a`,
  and `.github/workflows/release.yml`;
- GitHub's advisory database returned no exact affected-package record for npm `cac` or the inlined `mri`.

Capability and consumer parity remain pending,
so no hard-gate conclusion or recommendation follows from screening.

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
retain as replacement-parity baseline for CAC validation.

### CAC package identity and provenance

Candidate and version:
 `cac@7.0.0`,
release tag `v7.0.0`,
commit `77f602fcb2d1e75d24f5ecd94d5bf667acaa857a`.

Claim and relevance:
 exact source and published artifact are identifiable and inspectable.
This establishes the source-to-package boundary required before runtime validation.

Gate:
 open-source license,
provenance,
inspectability,
and reproducibility.

Status:
 pass for targeted screening;
rebuild comparison remains pending.

Primary evidence accessed 2026-08-14:

- `https://registry.npmjs.org/cac/7.0.0`;
- `https://registry.npmjs.org/-/npm/v1/attestations/cac@7.0.0`;
- `https://github.com/cacjs/cac/releases/tag/v7.0.0`;
- clone `~/temp/agent/cac-2026-08-14` at the pinned commit;
- tarball `~/temp/agent/cac-artifact-2026-08-14/cac-7.0.0.tgz`.

Relevant source excerpts:

```ts
// cacjs/cac package.json:22-26
"engines": {
  "node": ">=20.19.0"
}
```

```ts
// cacjs/cac tsdown.config.ts:3-12
export default lib(
  {
    inlineDeps: ['mri'],
  },
  {
    inputOptions: {
      resolve: {
        alias: {
          mri: 'mri/lib/index.mjs',
```

Outcome:
advance CAC to targeted source and runtime validation.

### CAC source and test surface checkpoint

Candidate and version:
 `cac@7.0.0` plus inlined `mri@1.2.0`.

Claim and relevance:
 the production parser is compact enough to audit
  but its declared dependency count understates the inlined MRI boundary.

Gate:
 human auditability,
dependency surface,
and test quality.

Status:
 targeted evidence collected;
final rating pending execution.

Primary evidence accessed 2026-08-14:

- CAC clone has six production TypeScript files and 960 physical source lines;
- a rough blank-and-comment exclusion leaves 728 lines;
- CAC has one 211-line test file with fifteen syntactic registrations,
  including one helper invoked three times,
  which produces seventeen executed cases;
- no fuzzing or mutation harness is present;
- coverage is configured through a reusable workflow;
- the most recent sampled Codecov report on merged PR `#172` reported 66.06 percent line coverage;
- MRI adds 119 source lines and 446 test lines across twenty-three test calls;
- CAC CI builds and tests Node 22,
  24,
  and 25 on Ubuntu and Windows;
- release-commit run `22491031174` passed all six Ubuntu and Windows matrix jobs for Node 22,
  24,
  and 25;
- the same release-commit run passed its separate Deno job;
- the run's lint job failed,
  making the overall workflow red even though all runtime jobs passed;
- GitHub had expired the failed lint log and returned HTTP 410,
  so this audit cannot identify that job's exact diagnostic;
- CAC's own workflow does not run a macOS job.

Outcome:
source is inspectable,
but cli-git's macOS requirement must be validated at the consumer boundary
 and upstream test breadth remains a scored concern.

### CAC lexical-preservation source trace

Candidate and version:
 `cac@7.0.0` with inlined `mri@1.2.0`.

Claim and relevance:
 CAC does not preserve every option lexeme and token role required by cli-git's current Git-region parser.
The behavior is implemented in MRI and then normalized again by CAC.

Gate:
 replacement parity,
fail-closed Git classification,
and human auditability.

Status:
 scored concern for management-only use;
potential hard failure for Git-region replacement pending artifact reproduction.

Primary source:
 clone `~/temp/agent/mri-2026-08-14` at `v1.2.0`,
commit `e73e9f9d5b02124d14ac17dac2c4801687d3e99a`,
and CAC clone at the pinned candidate commit.

MRI converts an untyped option value to a number whenever unary plus produces a non-NaN number
(`src/index.js:5-11`):

```js
function toVal(out, key, val, opts) {
  var x, old=out[key], nxt=(
    !!~opts.string.indexOf(key) ? (val == null || val === true ? '' : String(val))
    : typeof val === 'boolean' ? val
    : !!~opts.boolean.indexOf(key) ? (val === 'false' ? false : val === 'true' || (out._.push((x = +val,x * 0 === 0) ? x : val),!!val))
    : (x = +val,x * 0 === 0) ? x : val
```

MRI treats a following dash-led token as another option rather than a value
(`src/index.js:89-95`):

```js
name = arg.substring(j, idx);
val = arg.substring(++idx) || (i+1 === len || (''+args[i+1]).charCodeAt(0) === 45 || args[++i]);
arr = (j === 2 ? [name] : name);
```

CAC supplies MRI aliases and boolean names but no `string` option,
then applies optional transforms after MRI has already changed the value
(`cacjs/cac src/utils.ts:48-69`,
`src/cac.ts:282-331`).
An upstream `type: [String]` transform therefore cannot restore a lost leading plus or zero.
Open issue `cacjs/cac#165` reports the same 7.0.0 numeric coercion,
and its 2026-06-17 comment reports the leading-plus loss with `type: [String]`.

CAC stores the caller's full argv reference in `rawArgs` at `src/cac.ts:192`.
A consumer can reparse that array,
but doing so recreates the parser logic the dependency was meant to replace.

Outcome:
artifact and adapter probes must distinguish management-only use from Git-region use.
A broad migration cannot claim lexical preservation merely because `rawArgs` remains accessible.

### CAC boolean-name source trace

Candidate and version:
 `cac@7.0.0`.

Claim and relevance:
 option declarations normalize kebab-case names before MRI receives boolean metadata.
This can make a boolean spelling consume the following token,
which is unsafe for Git's many kebab-case flags.

Gate:
 replacement parity and fail-closed Git classification.

Status:
 potential hard failure for Git-region replacement pending artifact reproduction.

Primary source:

```ts
// cacjs/cac src/option.ts:27-39
this.names = removeBrackets(rawName)
  .split(',')
  .map((v: string) => {
    let name = v.trim().replace(/^-{1,2}/, '')
    if (name.startsWith('no-')) {
      this.negated = true
      name = name.replace(/^no-/, '')
    }
    return camelcaseOptionName(name)
  })
```

`getMriOptions` then registers only those normalized names
(`src/utils.ts:48-69`).
Open PR `cacjs/cac#169` demonstrates `--include-locked` consuming the following positional and adds raw-name tracking,
but it has no review or merge as of 2026-08-14.

Outcome:
current 7.0.0 source has no configuration that registers raw kebab spelling as MRI's boolean key.
Consumer-side scanning can work around it only by retaining owned token-role logic.

### CAC process and type boundary

Candidate and version:
 `cac@7.0.0`.

Claim and relevance:
 CAC permits caller-supplied argv and throws typed runtime errors,
but help output and public result typing do not match cli-git's pure parser seam directly.

Gate:
 process integration,
TypeScript compatibility,
and human auditability.

Status:
 scored concern;
management adapter validation pending.

Primary source:

- `parse(argv, { run: false })` accepts explicit argv at `src/cac.ts:174-190`;
- parsing mutates `rawArgs`,
  `args`,
  `options`,
  and matched-command state at `src/cac.ts:192-232`;
- command events dispatch during parsing at `src/cac.ts:211-224` even when `run` is false;
- built-in help is checked before command validation at `src/cac.ts:234-253`;
- help and version write through `console.info` at `src/command.ts:239-254`;
- unknown-option,
  required-value,
  and unused-argument checks run only from `runMatchedCommand` at `src/cac.ts:341-352`;
- shipped `ParsedArgv.options`,
  `OptionConfig`,
  actions,
  and `runMatchedCommand` use `any` in `dist/index.d.ts`.

Outcome:
a management-only adapter can avoid process exit and pass explicit argv,
but exact help must remain caller-owned and cli-git must project CAC's `any` result
 through its own validated type boundary.

### Maintenance audit

Candidate and version:
 `cac@7.0.0` and repository activity through 2026-08-14.

Claim and relevance:
 releases and maintainer-authored work resumed in 2026,
while behavior reports and external fixes receive limited public tracker response.

Gate:
 maintenance and release health.

Status:
 scored concern,
not a hard failure.

Primary evidence:
 GitHub API issue,
pull-request,
release,
commit,
and timeline data captured under
`~/temp/agent/cac-artifact-2026-08-14/`.

Findings:

- version 7.0.0 was published on 2026-02-27;
  the preceding stable release,
  6.7.14,
  was published on 2022-08-29;
- the repository had 36 commits from 2025-08-14 through the query date;
  31 used maintainer Kevin Deng's email,
  while the remaining five each used a different author email;
- five issues were created or updated in that period,
  so every one was inspected;
- those issues received no maintainer comment in the measured period;
  issue `#151` received one maintainer closure action after its fix reached 7.0.0;
- current numeric-coercion issue `#165` and multi-word-command issue `#170` have no assignee,
  label,
  milestone,
  or maintainer response;
- the ten most recently updated pull requests were inspected;
- maintainer `sxzz` merged PR `#172` 958 seconds after creation and merged PR `#171` 504,640 seconds after creation;
- PR `#171` received one maintainer approval;
- current behavior-fix PRs `#169` and `#173` have no review;
- no security policy,
fuzz harness,
or mutation harness is present;
- the issue template welcomes bug reports,
feature requests,
and reproduction links and states no ban on outside or AI-assisted contributions.

Outcome:
maintenance is concentrated in one current maintainer and releases reach npm,
but unresolved behavior fixes and absent public triage lower confidence for integration-specific defects.

## Execution manifests

### Published-artifact behavior matrix

Candidate:
 `cac@7.0.0`.

Pinned artifact:
 `~/temp/agent/cac-artifact-2026-08-14/cac-7.0.0.tgz`,
SHA-512
`b62c566209b9668383fb783a51379af757a8c39cfa0001da868de0d15f4235235be3980cf1299f96901cf8645dd489c2e00a8dff4ed49eb829e7a63de0df6125`.

Top-level command:

```text
podman run --memory=2g --cpus=2 --pids-limit=128 --ulimit nofile=1024:1024 --rm --network none --read-only ... node /probe/behavior-probe.mjs
```

Reachable candidate command tree:
 import one extracted ESM file and call its in-process parsing API.
The source path contains no filesystem,
network,
process-execution,
plugin,
native,
Wasm,
or generated-command call.
The harness does not call an action that performs external work.

Inspected files:
 CAC `src/*.ts`,
MRI `src/index.js`,
published `dist/index.js`,
source and published manifests,
and tarball contents.

Expected reads:
 read-only Node image,
read-only harness,
and read-only extracted CAC package.

Expected writes:
 bounded anonymous container state only;
read-only root plus a 64 MiB `/tmp` tmpfs.

Subprocesses:
 Podman runtime and one Node process.
Candidate code spawns none.

Network:
 disabled.

Image:
 local `docker.io/library/node:24-slim`,
Node `v24.18.0`,
image ID `2f35c3d18013b7d65e31c40f0602e4c0a65a18efc65c16e2b98497f13f4da921`,
digest `sha256:d45d78e7929b46875bbd4e29bea672d5bc48186c6c3588306521c815e78352d6`.

Credentials and environment:
 no home mount,
no repository mount,
no ambient credential environment,
and no network.

Outputs:
 one JSON object containing exact results for positive controls,
management grammar cases,
Git-region parity cases,
help behavior,
event dispatch,
and error classes.

Success condition:
 process exits zero,
every catalog case produces a captured result,
and positive controls prove the harness distinguishes expected values.

Failure condition:
 import failure,
uncaptured throw,
missing case,
unexpected external effect,
resource limit,
or nonzero process exit.

Stop condition:
 any undeclared read,
write,
subprocess,
network,
native,
or Wasm boundary requires manifest revision before continuing.

### Management-parser parity probe

Candidate:
 `cac@7.0.0` management-only integration shape.

Pinned inputs:

- the same extracted CAC artifact and digest as the published-artifact behavior matrix;
- current parser source from disposable worktree
  `~/temp/agent/mono-cac-probe.kbXvUyzU` at assessment commit `2c9760515`;
- current read-only workspace dependencies mounted only for the incumbent parser's Valibot import;
- local harness `cac-management-probe.mjs` in the disposable worktree.

Top-level command:

```text
podman run --memory=2g --cpus=2 --pids-limit=128 --ulimit nofile=1024:1024 --rm --network none --read-only ... node /probe/cac-management-probe.mjs
```

Reachable command tree:
 one Node process imports current pure management-parser source and the pinned CAC ESM artifact.
It constructs a new CAC instance per case,
parses explicit synthetic argv,
runs CAC validation,
normalizes refusal sentinels,
and compares JSON results.
No command action performs external work.

Inspected files:
 behavior-matrix source set,
current `management-parser.ts`,
current `parser/argv.ts`,
and the complete harness.

Expected reads:
 read-only Node image,
disposable worktree,
current workspace dependency directory,
and extracted CAC artifact.

Expected writes:
 bounded anonymous container state only;
read-only root plus a 64 MiB `/tmp` tmpfs.

Subprocesses:
 Podman runtime and one Node process.
Imported code spawns none.

Network:
 disabled.

Image and resource limits:
 identical to the published-artifact behavior matrix.

Credentials and environment:
 no home mount,
no repository write mount,
no ambient credential environment,
and no network.

Outputs:
 one JSON object with total cases,
equal cases,
mismatches,
and every compared result.
The catalog covers help,
trust,
untrust,
status,
check,
fix,
unknown options,
missing values,
repetition,
joined values,
numeric-looking policy IDs,
dash-led policy IDs,
pre-separator positionals,
and exact post-`--` pathspecs.

Positive control:
 ordinary one-level commands,
booleans,
text policy values,
and post-`--` pathspecs must match the incumbent.
A catalog with no equal rows is a harness failure rather than evidence.

Success condition:
 process exits zero,
all catalog rows execute,
and the JSON count equals the literal harness catalog.
Parity is an observed result,
not a success condition.

Failure condition:
 import failure,
uncaptured throw,
missing row,
unexpected external effect,
resource limit,
or nonzero process exit.

Stop condition:
 any undeclared effect or command boundary requires manifest revision before continuing.

### Numeric-preservation upstream prototype

Candidate:
 `cac@7.0.0` source at the pinned release commit.

Prototype workspace:
 fresh disposable clone
`~/temp/agent/upstream-prototype.lgbktubA` with verified `cacjs/cac` origin,
tag `v7.0.0`,
and commit `77f602fcb2d1e75d24f5ecd94d5bf667acaa857a`.

Candidate fix:
 add MRI `string` metadata for every nonboolean declared option,
then add upstream-style cases for exact `001`,
`+2`,
and `type: [String]` arrays.
The patch is
`~/temp/agent/cac-artifact-2026-08-14/cac-numeric-preservation.patch`.

Top-level verification command:

```text
podman run --memory=2g --cpus=2 --pids-limit=128 --ulimit nofile=1024:1024 --rm --network none --read-only ... node /prototype/numeric-probe.mjs
```

Reachable command tree:
 one Node process imports CAC TypeScript source through Node's type stripping and the audited MRI 1.2.0 source fixture.
It parses four literal option values,
asserts exact results,
and writes one JSON line.

Inspected files:
 CAC production source,
candidate diff,
probe,
MRI production source,
and synthetic MRI package manifest.

Expected reads:
 read-only prototype clone,
read-only Node image,
and read-only MRI fixture.

Expected writes:
 bounded anonymous container state only;
read-only root plus a 64 MiB `/tmp` tmpfs.

Subprocesses:
 Podman runtime and one Node process.
Candidate and fixture code spawn none.

Network:
 disabled.

Image,
credentials,
environment,
and resource limits:
 identical to the published-artifact behavior matrix.

Pre-patch condition:
 restore `src/utils.ts` and `tests/index.test.ts` from the pinned tag,
run the same assertion harness,
and require nonzero exit caused by the lexical mismatch.

Post-patch condition:
 reapply the recorded patch,
run the same harness,
and require exit zero with exact strings and exact string arrays.

Failure condition:
 pre-patch passes,
post-patch fails,
output differs,
or an undeclared effect appears.

Stop condition:
 any undeclared command or effect requires manifest revision before continuing.

### Upstream Node CI and rebuild validation

Candidate:
 `cac@7.0.0` source at the pinned release commit.

Source input:
 read-only clone `~/temp/agent/cac-2026-08-14` at
`77f602fcb2d1e75d24f5ecd94d5bf667acaa857a`.

#### Dependency-fetch phase

Top-level operations:

1. Copy the read-only source into private scratch.
2. Activate the manifest-pinned `pnpm@10.30.3` through Node 24 Corepack.
3. Run `pnpm install --frozen-lockfile --ignore-scripts`.
4. Enumerate every installed package manifest containing
   `preinstall`,
   `install`,
   `postinstall`,
   or `prepare`.
5. Record scratch byte use and stop if it exceeds 1.5 GiB.

Reachable command tree:
 container shell,
Corepack,
pnpm,
registry fetches,
filesystem extraction,
and manifest enumeration.
The `--ignore-scripts` gate prevents package lifecycle execution.

Network:
 enabled only for Corepack and lockfile package retrieval.
Expected endpoint is `registry.npmjs.org` from Corepack and pinned lock metadata.
No source or script may choose another endpoint.

Writes:
 private `~/temp/agent/cac-upstream-validation-2026-08-14` scratch only.
No repository,
real home,
or credential path is mounted writable.

Stop condition:
 any lifecycle execution,
unexpected endpoint,
undeclared command,
or disk ceiling breach stops validation before CI commands.

#### Offline CI phase

Top-level operations in order:

```text
corepack pnpm run build
corepack pnpm run typecheck
corepack pnpm run lint
corepack pnpm run test
```

Statically discovered root subprocesses:

- `tsdown` for the ESM and declaration build;
- `tsgo --noEmit` for type checking;
- `eslint --cache .` for lint;
- `vitest` for seventeen executed cases,
  including three helper-registered example subprocess cases.

The pinned development graph includes native build tooling used by tsdown and tsgo.
It is not part of CAC's published runtime.
The phase remains inside the resource-bounded,
secret-free container and uses the frozen lockfile tree produced with lifecycle scripts disabled.

Network:
 disabled.

Reads and writes:
 private scratch is writable for `dist`,
cache,
and test output;
Node image is read-only;
no real repository or home is mounted.

Subprocesses:
 Podman runtime,
Corepack,
pnpm,
the four declared package CLIs,
and their bounded worker or example Node processes.

Image and ceilings:
 local Node 24.18.0 image and digest from the artifact matrix,
2 GiB memory,
2 CPUs,
256 processes,
1,024 file descriptors,
and 1.5 GiB scratch ceiling.

Credentials and environment:
 disposable HOME,
Corepack,
pnpm,
and npm cache paths under scratch;
no ambient credentials.

Expected outputs:
 four zero exits,
upstream test summary,
rebuilt `dist/index.js` and `dist/index.d.ts`,
artifact hashes,
and exact diff against the extracted npm artifact.

Success condition:
 all four commands pass and rebuilt artifacts are explained byte-for-byte or by an inspected deterministic difference.

Failure condition:
 any command failure,
undeclared effect,
resource ceiling,
or unexplained artifact difference.

Deno and non-Linux platform jobs remain separate validation items.

### Disposable cli-git production-style integration

Candidate:
 published `cac@7.0.0` through a catalog-pinned dependency in a disposable repository worktree.

Workspace:
 `~/temp/agent/mono-cac-probe.kbXvUyzU` at baseline commit
`2c97605157965903408f73619adc4344f48fc82e`.
The worktree is outside the evaluation branch and will not be committed or merged.

Prototype boundary:

- replace only management routing in `management-parser.ts`;
- add sibling `management-cac-parser.ts` for a fresh CAC instance,
  exact policy-value placeholders,
  lone-dash preservation,
  runtime checks around CAC's `any` option result,
  and stable refusal mapping;
- retain all Git-region parsers,
  authored help strings,
  action types,
  management runtime,
  diagnostics,
  and tests;
- catalog-pin CAC 7.0.0 in the disposable worktree only.

Dependency operation:
 run filtered `pnpm install --ignore-scripts` to regenerate the disposable lock and package links.
Lifecycle scripts remain disabled.
No package install mutation reaches the evaluation worktree.

Mise trust isolation:
 the first dependency command stopped before pnpm because mise 2026.7.0 emitted
`Config files in .../mise.toml are not trusted.`
Mise documents that trust is machine-local state under `MISE_STATE_DIR`.
The revised operation sets `MISE_STATE_DIR` to private scratch,
trusts the disposable worktree's explicit `mise.toml` there,
and passes the same isolated state directory to every worktree mise command.
It does not modify the user's ordinary mise trust registry.
The diagnosis and runnable proof are in
`doc/troubleshooting/mise-disposable-worktree-trust.md`.

Top-level verification operations:

```text
mise run //package/git-policy/cli:lint:types
mise run //package/git-policy/cli:lint:oxlint
mise run //package/git-policy/cli:build
mise run //package/git-policy/cli:test:unit
mise run //package/git-policy/cli:pack:npm
mise run //package/git-policy/cli:test:built:trust
```

Additional probes:

- compare an expanded 52-case incumbent catalog with prototype results;
- import the built package root and call its exported authoring API;
- invoke the packed shadow `git` command for help,
  trust help,
  invalid usage,
  and selected management argument cases;
- inspect the final artifact set for one MJS application file,
  a declared CAC runtime edge,
  and no extra chunks;
- compare package and built-artifact bytes against the unchanged baseline;
- run lifecycle latency only after establishing unchanged-build timing spread and a positive control.

Reachable command tree:
 pnpm with lifecycle execution disabled;
repo-owned mise task programs;
TypeScript checker;
oxlint;
Rolldown;
module-test child processes;
packaging;
Podman consumer fixtures;
the built shadow executable;
and subprocesses already declared by the inspected package tasks.

Candidate code execution:
 CAC and inlined MRI run during management tests,
consumer commands,
and bundled artifact invocations.
They do not spawn subprocesses or perform I/O themselves.

Network:
 dependency retrieval may use the package registry during filtered install.
All candidate behavior probes and maintained tests use no candidate-controlled network.
Existing package tasks that bootstrap disposable containers retain their inspected package-manager
 and apt network behavior.

Reads and writes:
 only the disposable worktree,
its ignored dependency/build trees,
private scratch,
and task-declared disposable container filesystems.
User repository state,
Git configuration,
trust registry,
and shared package state are not verification targets.

Credentials and environment:
 no candidate command receives ambient credentials.
Built trust and lifecycle tasks use their existing explicit disposable fixtures.

Resource ceilings:
 existing cli-git Podman tasks cap memory and CPUs as declared in `package/git-policy/cli/mise.toml`.
Filtered install and host-side lint/build remain package-scoped.
No stress or fan-out benchmark is authorized.

Expected outputs:
 zero exits,
unchanged diagnostics and 52-case results,
one public MJS application artifact with every runtime edge declared,
passing package import and CLI consumer commands,
measured byte deltas,
and timing distributions.

Failure condition:
 any behavior mismatch,
type or lint finding,
test failure,
undeclared CAC runtime import or missing packed dependency,
side-effectful package import,
undeclared write,
resource breach,
or latency contract failure.

Stop condition:
 any undeclared command boundary or effect requires manifest revision before continuing.

### Node 22.18 combined-shape validation

Purpose:
 exercise the exact lower supported Node boundary for CAC plus the typed cli-git adapter.

Image retrieval:
 fetch official `docker.io/library/node:22.18.0-slim` because it is not present locally,
then record its resolved digest before candidate execution.
Network is allowed only for this image retrieval.

Top-level runtime operations in the network-disabled container:

- run the 52-case baseline-versus-candidate parity harness;
- import the built candidate application artifact;
- run management help;
- run trust help;
- run invalid trust,
  check,
  and fix option forms and assert exit `2` plus authored usage routing.

Reads:
 read-only candidate worktree,
read-only evaluation worktree for the incumbent comparator,
read-only evidence harness,
and read-only Node image.

Writes:
 64 MiB `/tmp` tmpfs and host evidence files only.

Subprocesses:
 Podman runtime and bounded Node invocations.
Candidate CAC/MRI code spawns none.

Credentials:
 none mounted or forwarded.

Resource ceilings:
 2 GiB memory,
2 CPUs,
128 processes,
and 1,024 file descriptors.

Success condition:
 52 parity matches,
zero side-effect output on import,
exact help routing,
and three invalid command forms exiting `2` with no stdout.

Failure condition:
 any mismatch,
unsupported syntax or API,
output drift,
undeclared effect,
or wrong exit.

Stop condition:
 any undeclared command or effect requires manifest revision before continuing.

## Hard-gate exits

### CAC as the shared Git-region parser

Outcome:
 fail.

Reason:
 the pinned artifact does not preserve required argv semantics:

- `001` becomes number `1`;
- `+310000` becomes number `310000`;
- `type: [String]` produces arrays containing already-damaged strings;
- declared `--message -a` reports a missing message and parses `-a` as another flag;
- lone `-` disappears rather than remaining positional;
- declared boolean `--dry-run target` becomes option value `dryRun: 'target'` and removes the positional;
- unknown option spellings and consumed values are normalized into an object rather than retained as the incumbent
  `unknownOptions` token sequence.

These are hard failures for cli-git's raw-argv fidelity and fail-closed classification constraints.
Reading and rescanning `CAC.rawArgs` would retain the current owned parser responsibility rather than make CAC the
 parser.

### CAC as the only cli-git parser

Outcome:
 fail.

Reason:
this shape includes the failed Git-region role.
Management-command success cannot offset a hard failure in forwarded Git classification.

### CAC for management commands only

Outcome:
 fail at the final package runtime gate.

Reason:
one-level command routing,
boolean flags,
ordinary string policy IDs,
unknown-option rejection,
unused-argument rejection,
repetition,
and post-`--` pathspec capture can match the incumbent only with an owned exact-value scanner and runtime validators.
That adapted shape reached 52-case parity on the Linux host's Node 26.7.0 runtime.

The exact lower supported runtime then rejected both the unchanged and CAC-integrated application artifacts before
package import:

```text
Node.js v22.18.0
SyntaxError: Unexpected identifier 'output'
```

The token is an emitted `await using` declaration.
A source-level candidate probe separately reached shared logger initialization and failed because Node 22.18.0 has no
`Error.isError`.
Node's 24.0.0 release notes identify explicit resource management and `Error.isError` as Node 24 additions.

This is a pre-existing cli-git engine-contract defect,
not a CAC defect:
CAC's own Node 22 matrix passes,
and the unchanged baseline artifact fails the same import.
It nevertheless means the complete migration shape cannot satisfy the frozen package range
`^22.18.0 || >=24.11.0`.
Hard gates apply to the resulting package,
not only to code introduced by the candidate.
The durable diagnosis is
`doc/troubleshooting/cli-git-node-22-runtime-contract.md`.

## Validation results

### Published artifact on Linux x64

Command:
 the published-artifact behavior matrix from the recorded manifest.

Environment:
 Node 24.18.0 on Linux x64,
network disabled,
read-only container,
2 GiB memory,
2 CPUs,
128-process ceiling,
and 1,024-file-descriptor ceiling.

Exit:
 zero.

Positive controls:

- ordinary text option remained `text`;
- tokens after `--` remained exact strings;
- a declared option after a positional parsed successfully;
- one-level command dispatch,
  unknown-option rejection after validation,
  unused-argument rejection,
  and direct pathspec capture all executed.

Observed failures are listed in the hard-gate exit for Git-region parsing.
Additional integration observations:

- `parse(..., { run: false })` still dispatched a command event;
- built-in help wrote through `console.info`;
- `trust --help --unknown` printed help and returned instead of validating the unknown option;
- allowing unknown options retained exact `rawArgs` but reduced `--mystery value tail` to
  `options.mystery = 'value'` and positional `tail`.

Evidence:

- harness SHA-256 is
  `1c4a4f94e3a822c04eb63e859c2db804884ee7f93a3711105f1b59b771a38938`;
- output is
  `~/temp/agent/cac-artifact-2026-08-14/behavior-output.json`;
- output SHA-256 is
  `28187f009af73513f5e602fade8b68040938730019a1eef32cbe610578b69a3b`.

### Management grammar without a lexical adapter

A 41-case comparison exercised current `parseManagementArgs` and a direct CAC mapping in the bounded container.
Thirty-seven cases matched.
Four did not:

- policy ID `001` became number `1`;
- policy ID `+2` became number `2`;
- policy ID `-x` was refused;
- policy ID `--all` was refused instead of being consumed as the `--policy` value.

Positive-control ordinary commands,
text IDs,
unknown options,
missing values,
joined values,
repetition,
and pathspecs matched.
The unadapted output SHA-256 is
`c81b9d978d8d34dec739a9a20cfdba0e72467ded97b373a1efc54225b24e1db6`.

### Management grammar with owned policy normalization

The second prototype added an owned linear scan that:

- stops at a real pathspec separator;
- captures exact separated and joined `--policy` values;
- replaces each value with a non-dash,
  nonnumeric placeholder before CAC parsing;
- restores exact policy lexemes after CAC validation.

All 44 catalog cases matched the current parser.
The expanded catalog added joined numeric and empty values plus a pathspec named `--policy`.
The process exited zero and stderr was empty.

Evidence:

- disposable worktree harness is
  `~/temp/agent/mono-cac-probe.kbXvUyzU/cac-management-probe.mjs`;
- harness SHA-256 is
  `2ba5fb3d3e938e03650202e9ba1d6e47333b599a1b8c50ab17aa87b05d830162`;
- adapted output is
  `~/temp/agent/cac-artifact-2026-08-14/management-parity-adapted-output.json`;
- adapted output SHA-256 is
  `ee22911deadb1d948e90cf7adda81529d21a2e30c59a1de46953b5d1ad0ad33c`.

The prototype adapter and its policy scanner occupy physical lines 9 through 94 of the 160-line harness.
The comparison catalog begins at line 95.
Current `management-parser.ts` lines 129 through 265 contain the replaceable specs and parse implementation,
while its help constants,
action type,
and the 415-line shared argv parser remain required.
This is not production line-count evidence because project TSDoc,
runtime validation,
logging,
and lint requirements were intentionally absent from the disposable probe.

### Numeric-preservation source prototype

The [troubleshooting record](../troubleshooting/cac-option-value-coercion.md) contains the full source trace,
workarounds,
duplicate search,
upstream filing decision,
and additive comment draft.
The matching patch is
`doc/troubleshooting/cac-option-value-coercion.patch`.

The fresh upstream clone matched the pinned origin,
tag,
and commit.
The same four-case assertion harness ran before and after the source patch in the recorded bounded container.

- Pre-patch exit was `1` with an assertion mismatch.
- Post-patch exit was `0`.
- Post-patch output preserved `'001'`,
  `'+2'`,
  `['001']`,
  and `['+2']` exactly.
- Post-patch stderr was empty.
- Patch SHA-256 is
  `b3362641520169a1068f172ecdd6e0e2e9b4f8663bc8061bec2f276c221eb0d2`.

This proves the numeric bug is fixable upstream.
It does not change the audit's hard failure for broad cli-git migration because the patch does not fix dash-led values,
lone `-`,
kebab-case boolean metadata,
or unknown-token facts.

### Upstream build and Node CI result

The dependency-fetch phase installed the frozen lock graph with lifecycle scripts disabled.
It scanned 1,771 package-manifest paths and found 111 lifecycle-bearing paths,
including pnpm-link duplicates.
No lifecycle ran.
The only non-`prepare` scripts discovered were esbuild's `postinstall` and simple-git-hooks' `postinstall`;
both remained disabled.
The resulting scratch tree used 455,847,337 bytes,
below its recorded 1.5 GiB ceiling.

The offline Node 24.18.0 phase then passed all four upstream commands:

- `build` produced `dist/index.js` and `dist/index.d.ts` in 85 ms;
- `typecheck` exited zero;
- `lint` exited zero;
- `test` executed seventeen cases,
  including example subprocess cases,
  with seventeen passes and no failures in 649 ms.

The source copy still identified commit
`77f602fcb2d1e75d24f5ecd94d5bf667acaa857a` and had no tracked source diff after validation.
The rebuilt artifacts matched the published npm files byte-for-byte:

- `dist/index.js` SHA-256:
  `01af40eab1e1de3d543e740fa73c0095ce188c752300dd25d90ef0cd32a5d7c9`;
- `dist/index.d.ts` SHA-256:
  `25265ad103164bfc85707531963d66c59b84a230e3551cf5bc336166a74ae93c`.

This closes source-to-artifact reproducibility and the upstream Linux/Node 24 suite.
The release-commit Actions evidence additionally closes CAC's Windows runtime matrix for Node 22,
24,
and 25,
but not combined cli-git adapter behavior.

### Production-style cli-git integration checkpoint

The disposable adapter moved CAC behind `parseManagementArgs` while retaining authored help,
management actions,
all Git-region parsers,
and a dedicated exact-value scanner.
The first typed version exposed a new parity requirement:
CAC represents repeated boolean flags as arrays.
Runtime validation initially refused repeated `--yes` and `--all`.
After the boundary accepted nonempty boolean arrays,
the expanded catalog matched the incumbent in all 52 cases.
The added cases cover lone dashes,
placeholder collisions,
policy value `--`,
joined dash values,
unknown joined options,
and mixed exact policies and pathspecs.

Evidence:

- parity harness SHA-256:
  `aa0e9ea2025bc55cea22020a0bd2ecbe20610a5791751dbd974bde024c7da9c2`;
- parity output SHA-256:
  `357df839b1ffaa31026fde5eafdb71dc261adb55a2e6c75ef012bc91f145cfac`;
- complete disposable integration patch SHA-256:
  `5a3bdaf4aabcc7f001463089e3975126ad8a257bb80c89daa31781d42e2bade4`;
- result:
  52 matches and zero mismatches on Linux/Node 26.7.0.

The production-shaped CAC module occupies 570 physical lines and 291 measured nonblank,
noncomment lines.
Together with its delegated `management-parser.ts`,
the candidate occupies 720 physical and 360 measured code lines versus the incumbent's 265 physical and 159 code lines.
The measured deltas are 455 physical and 201 code lines.
This line-count evidence belongs only to the net-removal criterion;
auditability,
migration burden,
and seam fit use separate evidence to avoid double counting.
CAC does not simplify the management implementation under current exact-value,
type,
TSDoc,
logging,
and lint requirements.
The current scanner remains necessary and gains handling for placeholders,
lone dashes,
repeated booleans,
and CAC's untyped option result.

Validation results so far:

- package TypeScript check passed;
- final production build passed in 73.40 ms;
- final application MJS is 1,037,963 bytes,
  8,354 bytes or 0.8114 percent above the unchanged 1,029,609-byte baseline artifact;
- declaration output remains byte-identical at 23,078 bytes,
  SHA-256 `3ef9146c9691ed28b227b18914728898c1c4e30d1a2a9562c6b8a7795059cee2`;
- packed manifest declares exact runtime dependency `cac: 7.0.0`;
- tarball contains one `dist/final/node/index.mjs` application artifact and one declaration file;
- package build,
  pack,
  and built trust consumer task passed;
- side-effect-free package import produced no stderr and exposed `definePolicy` as a function;
- built management help and trust help matched authored usage and wrote no stderr;
- built invalid trust usage exited `2`,
  wrote no stdout,
  and emitted the authored management usage;
- complete oxlint reported the same zero warnings and 96 pre-existing test-import errors on the candidate and unchanged
  baseline;
- the positive-control candidate run had reported and then cleared every finding in `management-cac-parser.ts`;
- the full unit run passed the cli-git entry-point group and all other reported groups but hit one transient
  `.git/config` lock collision in `git-worktree-identity.unit.test.ts`;
- an isolated rerun of that exact failing file passed.

User-boundary evidence hashes:

- package import:
  `99f58a2937560ff7888b2027ae1513b28a492d57fae05d693a6de57af9bbdb37`;
- management help:
  `32e9929172672fd98a12ba5402c62f207115db3d8e5f5bad0a2c1ba6735a74b6`;
- trust help:
  `3c08247bf89efa748037dd55a7ce914337344e023bbc70ef70459e40b7e91976`;
- invalid usage stderr:
  `bc2f0468f345f8eaa39cb71b6b6ca5c6d5cd42ead79005136293130be0877e19`;
- packed tarball:
  `1d0689e888cbe1962bea831769dc1a6d6fb60400f4e8f9309664cde7932f972a`.

### Candidate lifecycle result

The maintained lifecycle task passed all eleven budgets in its 2 GiB,
two-CPU,
tmpfs Node 24 container.
Each scenario used six warmups and thirty measured runs over 2,048 tracked files.

The hard-gate `wide-commit` result was:

- 300.6160 ms median;
- 311.0310 ms p95;
- 316.5200 ms maximum;
- 4.5437 ms median absolute deviation;
- 925 ms budget.

The no-config startup result,
where unconditional package loading most directly exposes CAC's added module edge,
was 87.5854 ms median,
91.6632 ms p95,
and 94.3790 ms maximum against a 275 ms budget.
No scenario exceeded its contract.
First candidate evidence SHA-256 is
`6d2eb6cdb834a3c193fbc1fbeb6a5f57bc1d64f5eb2a47f2fa128a1239304740`.

Two unchanged-baseline runs and two candidate runs establish run-to-run median bands.
The benchmark's direct-versus-wrapper samples are the positive control:
they visibly resolve more than eighty milliseconds of wrapper-added work in the startup scenarios.

`wide-commit` produced:

- baseline median band 292.2756 to 292.6579 ms;
- candidate median band 299.5193 to 300.6160 ms;
- candidate-minus-baseline band 6.8614 to 8.3404 ms.

The nonoverlapping wide-commit bands show a measured regression,
but it remains far below the 925 ms contract.
The movement is not uniform:
no-config candidate medians were 0.2577 to 4.5207 ms lower than the baseline band,
and several other scenario bands overlap or move in opposite directions.
Do not infer a universal startup cost from the wide-commit delta.

Evidence hashes:

- baseline run one:
  `43dab7b1ea57dccaf26d2da03fce28a5a842a62cc0eb0dec0e6a94d4449dbf9b`;
- baseline run two:
  `99f2da23342905eba3a3ace9a93f63b16aedc89249b25b8499136122a68694a8`;
- candidate run two:
  `a236e3e92fa0242d1d89005c9ac54e15244fb4f8f5cf1903bb452071ba93762d`;
- median-band comparison:
  `151d57b99cad0e976733f1295f5aaec285976b5d2f430bdd9bdcdf45a098b0bf`.

The task's fixed output path overwrote the first candidate JSON during run two.
The first run's full-file hash and command output remain recorded,
and its summary medians were preserved in
`candidate-latency-1-summary.json` with SHA-256
`976a8d9c3b528768645dc901b350cc19e4157705ce34d6c1293eb1d597de02fc`.
The two baseline files,
second candidate file,
and comparison file remain complete.
The latency comparison is not load-bearing for the rejection.

This passes the absolute latency gate.
The later Node 22 package failure still exits the management-only shape before scoring.
The benchmark JSON labels both artifacts with baseline revision `2c9760515` because the prototype was uncommitted;
artifact paths and hashes,
not that embedded revision field,
distinguish the runs.

### Node 22.18 lower-bound result

Official `node:22.18.0-slim` resolved to digest
`sha256:0d130e2ee18e88e1561375276daced6bff032539200173f2daf48c2e33f38ff5`.
The network-disabled probe failed before the planned 52-case combined validation could complete.

- unchanged built artifact import exited `1` on retained `await using`;
- CAC candidate built artifact import exited `1` on the same syntax;
- candidate source harness exited `1` because shared logger code called absent `Error.isError`;
- Linux/Node 26.7.0 is the positive control for typed 52-case parity,
  package import,
  authored help,
  invalid usage,
  type checking,
  and unit execution;
- Linux/Node 24.18.0 is the positive control for the built-trust consumer and maintained lifecycle benchmark.

Baseline stderr SHA-256 is
`37b72db335be0c716f8b24248cf7cddd0bb082017358e89c89b00af6d3f18a47`.
Candidate stderr SHA-256 is
`02ca3774d5b3beadc967a14d6e88d842db192665a035b914549a7c2990327f1a`.

Validation stopped at this hard gate.
No combined macOS or Windows adapter run,
second full unit run,
or remote disposable branch is warranted for a candidate that cannot load on the declared lower runtime.

## Score arithmetic

Not applicable.
All CAC integration shapes failed at least one hard gate,
and the governing skill forbids offsetting hard failures with points.
The management-only shape therefore never became a validated finalist.

## Sensitivity

Not applicable.
There is no scored finalist or ordering to perturb.
Changing weights cannot repair raw-argv fidelity or the package's Node 22 load failure.

## Pros and cons

### CAC as Git-region or complete parser

Pros:

- compact MIT source with reproducible SLSA-bound artifact;
- no declared runtime dependency and an auditable inlined MRI implementation;
- successful upstream Ubuntu and Windows runtime matrix.

Cons:

- changes numeric-looking values;
- refuses dash-led declared values;
- loses lone dash;
- mishandles relevant kebab-case booleans;
- normalizes away unknown token facts;
- fails exact Git argv and fail-closed classifier requirements.

### CAC for management commands only

Pros:

- routes the closed one-level command namespace;
- rejects unknown options and unused arguments;
- reproduces 52 incumbent cases with an owned adapter;
- preserves authored output and exit contracts in the built Linux/Node 26.7.0 command probes;
- passes every absolute lifecycle budget;
- adds only 8,354 bytes to the application artifact.

Cons:

- retains a custom exact-value scanner and every Git parser;
- adds 455 physical and 201 measured code lines across the management boundary;
- needs runtime checks for CAC's `any` option declarations and repeated-boolean arrays;
- retains authored help because CAC writes built-in help through `console.info`;
- introduces stateful parse and throw/catch behavior into the pure-parser seam;
- cannot make the complete package load on its declared Node 22.18 floor;
- has no remaining practical simplification benefit after parity work.

## Ranking and recommendation

There is no validated CAC ranking.

- CAC as a shared Git-region parser and as the complete parser both exit on raw-argv hard failures.
- CAC for management commands only is the closest shape,
  but exits on the resulting package's Node 22 hard gate.

Do not migrate cli-git to CAC.
Retain the repository-owned parsers.
Even if cli-git's pre-existing runtime floor is later raised to Node 24 or its output is made Node 22-compatible,
the management-only prototype adds code and keeps the exact-value scanner,
so CAC provides no practical simplification to justify the dependency and regression surface.
Other external parser technologies remain outside this audit.

## Evidence limits

The historical Optique timing is motivation for a latency gate,
not evidence about CAC.
The expanded typed adapter is disposable evaluation code,
not an authorized migration.
The Node 22 failure is a pre-existing cli-git contract defect rather than a CAC defect,
but hard gates apply to the resulting package.
CAC's release-commit Windows tests passed.
The typed combined parity and command probes ran on Linux/Node 26.7.0;
the built-trust and lifecycle consumers ran on Linux/Node 24.18.0.
No typed 52-case Node 24 or combined macOS/Windows adapter run completed before the Node 22 stop.
GitHub had expired CAC's failed release-commit lint log,
so its exact diagnostic remains unknown.
The first full cli-git unit run had one Git fixture lock collision that passed on isolated retry;
validation stopped at the later hard gate before a second complete run.
The two disposable repository worktrees were removed after preserving the integration patch and hashed evidence under
`~/temp/agent/cac-artifact-2026-08-14/`.
