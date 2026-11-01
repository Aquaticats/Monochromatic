# Technology vet: wg-allowedips CIDR library

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `@monochromatic-dev/cli-wg-allowedips` is now `@monochromatic-dev/wg-cli-allowedips`.

Status:
 complete.
 Lifecycle phase is recommended.
 Started and last updated on 2026-07-28.

Subject:
 `wg-allowedips` CIDR library.

Decision scope:
 choose the CIDR parsing and subtraction dependency for `@monochromatic-dev/cli-wg-allowedips`.

Governing skill:

- Commit is `a05818ad70a40e5769a36de669697ba109891b31`.
- SHA-256 is `393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`.

Compatibility fingerprint:
 `5463db7e7d141d35d67ed77a3f9b8ca8794e3dcccad51cd976bcc1f8d06e6e62`.

Active audit owner:
 Pi session `019fa84d-b4f0-704e-9646-9e7e04230d16`.

Prior compatible report:
 none found.

## Context

The current specification in `doc/planning/wg-allowedips.md` selects `cidr-tools >=12.1.2`.
The user asked to consider [`beaugunderson/ip-address`][ip-address-repo].
This evaluation compares that candidate against retaining the selected dependency.
It also screens other concrete alternatives found through the required discovery sources.

The CLI accepts IP literals and CIDR blocks for IPv4 and IPv6.
It unions the allowed input and subtracts the union of the disallowed input.
The output is a minimized sorted CIDR cover.
The user explicitly prioritizes the least production code and no special-case machinery.

## Classification

Every package candidate is an inspectable open-source local technology.
The incumbent dependency replacement overlay applies because `cidr-tools` is already selected by the plan.
The multi-platform overlay applies because the Node CLI must behave deterministically on Linux,
 macOS,
 and
Windows.

The high-trust overlay does not apply.
The library processes user-controlled address text but receives no credentials and performs no process execution.
The native and Wasm overlay does not apply to a survivor only if its complete runtime graph remains pure
JavaScript or TypeScript.
The SaaS and sensitive-data overlays do not apply.

## Hard constraints

A candidate must satisfy all of these constraints:

- deterministic operation on Linux,
   macOS,
   and Windows under Node.
  js 22 or newer;
- exact IPv4 and IPv6 union-minus-union with a minimized sorted CIDR cover;
- parsing and validation of IPv4 and IPv6 single addresses and CIDR prefixes;
- inspectable JavaScript or TypeScript source;
- no native code or Wasm;
- no prebuilt or downloaded artifact;
- no generated runtime whose source mapping is unavailable;
- no lifecycle install command;
- an LGPL-3.0-compatible open-source license;
- reproducible validation through the real `wg-allowedips` consumer operations.

No arbitrary start-to-end range syntax is required.
A candidate may combine a library with a small amount of project code.
The amount and complexity of that code are scored rather than used as a hard gate.

## Frozen criteria

Hard-gate failures remain outside arithmetic.
Every surviving finalist receives these criteria and weights:

- Interface fit and production-code footprint has weight 5.
  The user's least-LoC instruction controls this weight.
- Validation clarity and error ergonomics has weight 5.
  The CLI must reject invalid address and prefix input without a second parser subsystem.
- Runtime dependency surface and human auditability has weight 4.
  Unused feature breadth earns no credit.
- Upstream test quality has weight 3.
  Relevant value comes from IPv4 and IPv6 parser boundaries plus set-operation invariants.
- Maintenance and release hygiene has weight 1.
  No stronger preference was stated.

Each rating uses the skill's scale from 0 through 4.
The maximum weighted score is 72.
No unresolved preference remains before evidence collection.

## Frozen discovery schedule

### Npm registry

Run each literal query against the npm registry search API with a page size of 100 and no negative filter:

- `cidr ipv4 ipv6 subtract exclude merge`;
- `ip address cidr aggregate subtract`;
- `subnet ipv6 typescript range`.

Continue each query until the registry reports exhaustion or two complete pages add no screening survivor.
Sort by the registry's default relevance order.

### GitHub repositories

Run each literal query through GitHub repository search with 100 results per page and no negative filter:

- `cidr ipv6 subtract language:TypeScript`;
- `cidr aggregate exclude language:JavaScript`;
- `IP address subnet language:TypeScript`.

Continue each query until GitHub reports exhaustion or two complete pages add no screening survivor.
Use GitHub's default best-match order.

### Broader web

Run these literal queries through the configured web search provider:

- `JavaScript IPv4 IPv6 CIDR subtraction library`;
- `TypeScript CIDR merge exclude package`;
- `ip-address alternatives CIDR`;
- `cidr-tools alternative`.

The provider exposes no page cursor through this harness.
Record that limit and corroborate candidates through the registry and repository searches.

### This repository

Run an uncapped repository search for:

- `cidr-tools`;
- `ip-address`;
- `ip-bigint`;
- CIDR plus `subtract`;
- subnet plus `subtract`.

Inspect every matching plan,
 decision,
 audit,
 package manifest,
 catalog entry,
 and implementation.

### Expansion round

After the initial schedule completes:

1. Collect every new problem-class synonym from candidate metadata.
2. Append one de-duplicated query round to each applicable external source.
3. Freeze the expanded schedule.
4. Record later terms without adding more queries.

## Query ledger and saturation

### Npm registry initial schedule

The registry search API returned 100 results per requested page.
Its total counts changed slightly while the audit ran.
The page rule depends on new screening survivors rather than a stable total.

The query `cidr ipv4 ipv6 subtract exclude merge` reported about 47,662 results.
The first page discovered `cidr-tools`,
 `fast-cidr-tools`,
 `ip-num`,
 `ip-address`,
 and
`cidr-block`.
The page from offset 100 added `ip.js`.
The page from offset 200 added no survivor.
The page from offset 300 added `@h3mantd/ip-kit`.
The pages from offsets 400 and 500 added no survivor.
The query therefore met the two-page saturation rule.

The query `ip address cidr aggregate subtract` reported about 67,381 results.
The first page repeated the main candidates.
The page from offset 100 added `aggregate-cidr`.
The pages from offsets 200 and 300 added no survivor.
The query therefore met the two-page saturation rule.

The query `subnet ipv6 typescript range` reported about 404,185 results.
The first page repeated `ip-address`,
 `ip-num`,
 and `cidr-block`.
The page from offset 100 added `@cldn/ip` and repeated `ip.js`.
The pages from offsets 200 and 300 added no survivor.
The query therefore met the two-page saturation rule.

### Npm registry expansion round

Candidate metadata added the terms `prefix`,
 `pool`,
 and `summarize`.
The de-duplicated expansion query was `cidr prefix pool summarize`.
It reported about 43,648 results.
The first page contained only known candidates.
The pages from offsets 100 and 200 added no survivor.
The expansion source is saturated and the registry schedule is frozen.

### GitHub repository schedule

The query `cidr ipv6 subtract language:TypeScript` returned no repository.
The query `cidr aggregate exclude language:JavaScript` returned no repository.
The query `IP address subnet language:TypeScript` returned 26 repositories and was exhausted on its first
page.
It corroborated `ip-address` and `@cldn/ip` while adding only parser,
 matcher,
 application,
 or IPv4-only
projects.

The expansion query `cidr prefix pool language:TypeScript` returned no repository.
GitHub discovery is exhausted and saturated.

### Broader web schedule

The four frozen searches discovered or corroborated:

- `cidr-tools`;
- `fast-cidr-tools`;
- `@h3mantd/ip-kit`;
- `ip.js`;
- `cidr-block`;
- `ip-address`;
- `ip-num`;
- `@cldn/ip`;
- parser and matcher packages such as `ipaddr.js`,
   `ip-cidr`,
   and `netip-ts`.

The provider exposed no page cursor.
Registry and GitHub enumeration independently covered the plausible candidates and met their saturation rules.
No web-only result remained a screening survivor.

### Repository schedule

The uncapped repository search found `cidr-tools` only in `doc/planning/wg-allowedips.md`.
It found no `ip-address`,
 `ip-bigint`,
 CIDR-subtraction implementation,
 prior decision,
 or prior audit.
A separate address list in `package/config/tofu/hetzner.tf` was unrelated.

### Terminal discovery result

Discovery is saturated with more than one screening survivor.
The frozen expansion round is complete.
Later taxonomy terms will be recorded without adding queries.

## Candidate ledger after discovery

### `cidr-tools`

Discovery source:
 current plan plus npm,
 GitHub,
 and web results.

Screening result:
 validated finalist.
Source,
 upstream,
 platform,
 published-artifact,
 and consumer checks passed.

### `fast-cidr-tools`

Discovery source:
 npm and web results.

Screening result:
 hard-gate failure during published consumer validation.
One exclusion covering multiple disjoint allowed intervals leaves one covered interval in the output.

### `@h3mantd/ip-kit`

Discovery source:
 npm expansion and web results.

Screening result:
 validated finalist.
The published artifact passed after the consumer added family partitioning and host-to-CIDR conversion.

### `ip-num`

Discovery source:
 npm registry and web results.

Screening result:
 hard-gate failure.
Version 1.6.1 declares `preinstall: npx only-allow npm`,
 contrary to the frozen no-install-lifecycle
constraint.

### `ip-address`

Discovery source:
 user nomination plus npm,
 GitHub,
 and web results.

Screening result:
 exited for category mismatch after targeted source confirmation.
Version 10.3.1 exports address parsers and helpers but no collection union,
 subtraction,
 or range-to-CIDR cover.
Replacing `cidr-tools` with it would add a project-owned set engine despite ready-to-use finalists.

### `cidr-block`

Discovery source:
 npm and web results.

Screening result:
 exited for category mismatch.
Its exhaustive published interface covers validation,
 bounds,
 overlap,
 splitting,
 and iteration but no
collection union or subtraction.
Using it would still require a separate set engine.

### `ip.js`

Discovery source:
 npm and web results.

Screening result:
 exited for category mismatch.
It converts one continuous range to a prefix cover but provides no collection union or subtraction.

### `@cldn/ip`

Discovery source:
 npm,
 GitHub,
 and web results.

Screening result:
 exited for category mismatch.
Its published interface covers address and subnet arithmetic plus collection merging but exposes no difference
operation needed by this CLI.

### `ip-bigint`

Discovery source:
 the current candidate's runtime graph and related-project list.

Screening result:
 exited for category mismatch.
It converts address strings and BigInts but does not implement set subtraction or CIDR-cover generation.

### Parser and matcher packages

This group includes `ipaddr.js`,
 `ip-cidr`,
 `netip-ts`,
 `node-cidr`,
 and similar registry results.

Screening result:
 exited for category mismatch.
Their published interfaces parse,
 normalize,
 test containment,
 or split one subnet.
They do not implement both union-minus-union and minimal CIDR-cover output.

### IPv4-only and aggregate-only packages

This group includes `aggregate-cidr`,
 `cidr-lib`,
 and several overlap or matcher packages.

Screening result:
 failed the dual-stack or subtraction hard constraint.

### Project-owned BigInt implementation

Screening result:
 deferred by open-source precedence and the existing-tools rule.
Ready-to-use serious alternatives survived discovery.

## Targeted evidence records

### Artifact identity, license, and provenance

All inspected licenses are compatible with the repository's LGPL-3.0 licensing requirement.
No candidate repository contains a tracked native,
 Wasm,
 or prebuilt executable.
The npm manifests contain no optional runtime dependency or downloaded artifact path.

`cidr-tools` 12.1.3 is BSD-2-Clause.
The npm artifact has integrity
`sha512-nrz3c8ARh18FGrDZG4O6DffMcsOeh/Sw4hsMLSxAxxcCl3OfE2aGinplB1URIE2LSkrSQltL/eYweqw1E9FBuQ==`.
Tag `12.1.3` resolves to commit `a3b61d005c34b8eb91333ea5e78788ae24491d0b`.
The tag-triggered release workflow builds and publishes with npm trusted publishing.
Its only runtime dependency is BSD-2-Clause `ip-bigint` 9.0.7,
 which has no runtime dependencies or lifecycle
scripts.
Evidence is the [registry record][cidr-registry],
 [release][cidr-release],
 and
`.github/workflows/release.yaml:1-27` in clone `~/temp/agent/cidr-tools-2026-07-28`.
Status is pass.

`fast-cidr-tools` 0.3.4 is MIT.
The npm artifact has integrity
`sha512-WQNW+ynysAsI+O3YX2269Ff1wx6+xTyKrtLPN0TaZOf5ZZfFNPS59J0vmCrJbpno5z3vJ5sX4wUHpJL7avuHLg==`.
The registry omits `gitHead`,
 but release commit
`d37506e5fcacc7a04760bd9c1b8c924d877bbc39` has the exact 0.3.4 manifest and its trusted-publishing
workflow run succeeded on 2025-11-07.
The published `foxts ^4.1.0` range currently resolves to MIT `foxts` 4.6.0.
That package installs MIT `fast-escape-html` and `fast-escape-regexp`,
 although the consumed
`foxts/fast-ip-version` subpath imports neither.
None declares an install lifecycle.
Evidence is the [registry record][fast-registry],
 [release commit][fast-release-commit],
 and
`package.json` plus `.github/workflows/publish.yml:1-45` in clone
`~/temp/agent/fast-cidr-tools-0.3.4`.
Status is pass with a larger same-author audit surface.

`@h3mantd/ip-kit` 1.1.0 is MIT and dependency-free.
The npm artifact has integrity
`sha512-oD9D9uHVkz/na6uYFcoQru/46WXlWMjkP8Pqy4xkb8wn8If0DNKEzVxRHvPmg6wSAufw40JF2lsz7Isl75zyWw==`.
Registry `gitHead`,
 tag `v1.1.0`,
 and clone commit all resolve to
`cf077b0316ba484c5e357403e2aeb650b7b2695b`.
Its tag workflow publishes with npm provenance and no install lifecycle.
Evidence is the [registry record][ip-kit-registry] and `.github/workflows/publish-on-tag.yml:1-33` in clone
`~/temp/agent/ip-kit-2026-07-28`.
Status is pass.

`ip-num` 1.6.1 is MIT and has no runtime dependency,
 but its exact registry artifact and source commit
`d566df2b2725fb571e43890b3cb604486e68ceee` declare `preinstall: npx only-allow npm`.
Evidence is the [registry record][ip-num-registry] and `package.json:16-18` in clone
`~/temp/agent/ip-num-2026-07-28`.
Status is hard-gate fail because installation executes a lifecycle command.

`ip-address` 10.3.1 is MIT,
 dependency-free,
 and has integrity
`sha512-1e9d3kb97NHJTIJDZW9rKqW2h6+dFa50Dy0fpPSMQp2ADje5gvKsXmdiK6dwY5t76TaTt5+P5N1Y/LoToIxP6g==`.
Registry `gitHead` is signed commit `be7e626c0d49fccb518899f520a3fb64ee189741`.
Its release workflow uses trusted publishing.
Version 10.3.1 fixes a high-severity leading-zero parser advisory affecting 10.3.0 and older.
The advisory does not affect the evaluated version.
Evidence is the [registry record][ip-address-registry],
 [security advisory][ip-address-advisory],
 and clone
`~/temp/agent/ip-address-10.3.1`.
Status is provenance and security pass,
 followed by category-fit exit.

A GitHub Advisory Database query for each exact candidate and runtime dependency returned no advisory affecting
the evaluated version on 2026-07-28.
Repository advisory enumeration found only the four published `ip-address` advisories,
 all patched by 10.3.1.

### Production source and interface fit

`cidr-tools` directly parses inputs into separate IPv4 and IPv6 interval lists,
 coalesces each list,
 subtracts
coalesced exclusions,
 converts remaining intervals to minimal CIDRs,
 and emits IPv4 before IPv6.
The consumed boundary is one call:
 `excludeCidr(allowed, disallowed)`.
The adjacent source excerpt is `export function excludeCidr(base, excl)`,
 followed by
`subtractSorted4` and `subtractSorted6` over merged intervals.
Evidence is `index.ts:313-463` in clone `~/temp/agent/cidr-tools-2026-07-28` at commit
`a3b61d005c34b8eb91333ea5e78788ae24491d0b`.
Status is pass.

`fast-cidr-tools` directly provides `exclude(base, exclusions, sort)`.
Passing `true` opts into numeric ordering.
Its parser identifies a family merely by the presence of `:` or `.`,
 then converts the suffix with `BigInt`.
The adjacent source excerpts are `const version = fastIpVersion(cidr)` and
`const bitmask = ... BigInt(splitted[1])`.
Evidence is `src/parse.ts:7-29`,
 `src/merge.ts:42-148`,
 and `src/exclude.ts:8-137` in clone
`~/temp/agent/fast-cidr-tools-0.3.4` at release commit
`d37506e5fcacc7a04760bd9c1b8c924d877bbc39`.
Status is operation-fit pass and validation-quality concern pending execution.

`@h3mantd/ip-kit` normalizes one-family `RangeSet` instances,
 subtracts ranges,
 and converts each remaining
interval to a minimal CIDR cover.
A consumer must partition both input lists by family,
 create up to four sets,
 subtract twice,
 concatenate,
 and
stringify.
`CIDR.parse` uses `parseInt(parts[1], 10)`,
 so a suffix such as `24junk` is accepted as prefix 24 unless the
consumer performs an exact prefix check.
Evidence is `src/domain/rangeset.ts:21-168,227-280`,
 `src/domain/range.ts:92-137`,
 and
`src/domain/cidr.ts:21-45` in clone `~/temp/agent/ip-kit-2026-07-28` at commit
`cf077b0316ba484c5e357403e2aeb650b7b2695b`.
Status is operation-fit pass with orchestration and validation concerns pending execution.

`ip-address` exports only `Address4`,
 `Address6`,
 `AddressError`,
 and IPv6 formatting helpers.
An uncapped source search found no collection union,
 subtraction,
 or interval-to-CIDR operation.
Its adjacent complete export excerpt is the seven-line `src/ip-address.ts`.
Evidence is [that source file][ip-address-exports] in clone `~/temp/agent/ip-address-10.3.1` at signed commit
`be7e626c0d49fccb518899f520a3fb64ee189741`.
Status is category mismatch.
The existing-tools rule prevents treating a project-owned interval engine as a finalist while direct-operation
packages survive.

`ip-num` exposes ranges and pools,
 but list subtraction would require combining its mutable pool and per-range
`difference` operations.
Its `Pool.aggregate()` recursively calls itself until stable.
This broader interface does not rescue the install-lifecycle hard failure.
Evidence is `src/IPPool.ts:56-107` and `src/IPRange.ts:450-619` in clone
`~/temp/agent/ip-num-2026-07-28`.
Status is excluded.

### Auditability, tests, and CI

The measured non-test TypeScript surfaces are:

- `cidr-tools`:
   684 lines in one source file plus 307 lines in dependency `ip-bigint`;
- `fast-cidr-tools`:
   739 lines,
   plus 2,524 lines in the resolved `foxts` package and 183 lines in its two
  installed dependencies;
- `@h3mantd/ip-kit`:
   1,805 lines across 11 files with no runtime dependency;
- `ip-num`:
   4,122 lines across 13 source files;
- `ip-address`:
   2,525 lines across nine source files.

The first three candidates use strict TypeScript checking.
The inspected runtime source has no native or Wasm boundary.
An uncapped search found no fuzzing or mutation harness in any candidate.
Suppression and debt-marker counts,
 excluding locks and generated documentation,
 were zero for `cidr-tools`,
one for `fast-cidr-tools`,
 zero for `ip-kit`,
 19 for `ip-num`,
 and 22 for `ip-address`.
These counts describe audit surface and do not act as hard gates.

`cidr-tools` has 404 lines of direct tests and `ip-bigint` has 102.
The direct suite covers empty sets,
 full subtraction,
 partial subtraction,
 mixed families,
 minimization,
 ordering,
host inputs,
 overlapping inputs,
 and prior unsorted-IPv6 regressions.
CI runs lint and tests on Node 22,
 24,
 and 26 across Ubuntu,
 macOS,
 and Windows,
 plus Bun on all three systems.
Evidence is `index.test.ts:15-87,398-404` and `.github/workflows/ci.yaml:1-29`.
Status is strong relevant coverage with no fuzz or mutation evidence.

`fast-cidr-tools` has 159 lines of tests.
Its exclusion cases cover both families but are fewer,
 and its default expected output is deliberately unsorted.
The suite contains one regression for the prior 128-bit IPv6 bug.
CI runs only Node 22 on Ubuntu.
Evidence is `test/index.test.ts:1-159`,
 `.node-version`,
 and
`.github/workflows/publish.yml:1-45`.
Status is weaker platform and invariant coverage.

`@h3mantd/ip-kit` has 1,255 lines of tests.
The range-set tests include example-based set laws,
 but IPv6 range-set coverage exercises construction and
containment rather than IPv6 subtraction and conversion together.
CI runs lint,
 type checking,
 tests,
 and build on Node 18,
 20,
 and 22,
 only on Ubuntu.
Evidence is `tests/domain/rangeset.test.ts:1-188`,
 `tests/domain/range-toCIDRs.test.ts:1-31`,
 and
`.github/workflows/ci.yml:1-38`.
Status is broad unit coverage with a relevant dual-stack gap.

### Maintenance

The audit queried all issues updated since 2025-07-28 when the count was at most 20,
 otherwise the ten most
recently updated.
It queried the ten most recently updated pull requests under the same rule.
The GraphQL evidence,
 including author association,
 comments,
 reviews,
 and timeline actions,
 is stored under
`~/temp/agent/*-maintenance-2026-07-28.json`.

`cidr-tools` had four updated issues and three pull requests.
All were inspected.
The sole maintainer commented on every issue,
 closed three after fixes,
 and merged all three pull requests.
The last 12 months contain 124 maintainer-authored commits,
 three Copilot commits,
 and multiple releases,
 including
12.1.3 on 2026-07-28.
Issue 30 remains open with a maintainer response;
 its shorthand parsing concern is mitigated at this CLI boundary
by strict `node:net` validation.
Evidence includes [issue 30][cidr-issue-30] and [release 12.1.3][cidr-release].
Status is active releases and responsive maintenance,
 concentrated in one maintainer.

`fast-cidr-tools` had one updated issue and two pull requests.
All were inspected.
The maintainer responded to the issue and reviewed and merged both external fixes.
The last 12 months contain ten maintainer commits and two external commits.
Release 0.3.4 followed the duplicate-range fix on 2025-11-07.
The earlier pull request for 128-bit IPv6 calculation documents a relevant correctness regression now covered by
one test.
Evidence includes [pull request 7][fast-pr-7] and the [0.3.4 workflow run][fast-workflow-run].
Status is responsive but low-volume,
 single-maintainer maintenance.

`@h3mantd/ip-kit` had no issue and two owner-authored pull requests.
It had 22 owner-authored commits and releases 1.0.0,
 1.0.1,
 and 1.1.0 during the period.
No external maintenance sample exists.
Evidence includes [release 1.1.0][ip-kit-release].
Status is recent single-maintainer release activity with low public-tracker signal.

`ip-num` had two updated issues and 24 pull requests.
Both issues and the ten most recently updated pull requests were inspected.
The last 12 months contain 56 maintainer commits and three external commits,
 with releases 1.6.0 and 1.6.1 on
2026-07-19.
Three current external parser or range fixes remain open,
 including a correction to `RangedSet.isCidrAble()`.
Status is active,
 concentrated maintenance but excluded by the lifecycle gate.

`ip-address` had 31 updated issues and 33 pull requests.
The ten most recently updated of each were inspected.
The last 12 months contain 45 owner commits and four external commits.
Four releases on 2026-07-25 fixed four disclosed advisories and added GitHub Actions CI.
Status is active,
 concentrated maintenance but category-mismatched for this decision.

## Hard-gate outcomes

The validated finalists are `cidr-tools` 12.1.3 and `@h3mantd/ip-kit` 1.1.0.
Both passed license,
 source availability,
 provenance,
 pure JavaScript or TypeScript runtime,
 artifact,
 install-lifecycle,
 platform,
 upstream-suite,
 and published-consumer gates.
The hard constraints explicitly allow a candidate plus small project code.
A deterministic text guard can complete parser validation without changing set semantics,
so each finalist's parser limitations are scored under validation clarity rather than treated as arithmetic
correctness failures.
The published `fast-cidr-tools` set operation itself returns a wrong set and therefore fails the exact-subtraction
hard gate.

`fast-cidr-tools` 0.3.4 fails exact set subtraction at the published consumer boundary.
`ip-num` 1.6.1 fails the no-install-lifecycle hard constraint.
`ip-address` 10.3.1 exits for category mismatch because it supplies no set engine.
All parser-only,
 matcher-only,
 IPv4-only,
 and aggregate-only discovery exits remain excluded.

## Execution manifests

### Shared isolation boundary

Validation uses local image `docker.io/library/node:24-slim` at digest
`sha256:6f7b03f7c2c8e2e784dcf9295400527b9b1270fd37b7e9a7285cf83b6951452d`.
The measured container runtime is Node 24.18.0 and npm 11.16.0 on Linux x86-64.
Node 24 satisfies the Node 22-or-newer consumer requirement.
Upstream CI evidence supplies the separately recorded platform matrices.

Every container receives two CPUs,
 2 GiB memory,
 256 processes,
 4,096 file descriptors,
 no ambient credentials,
 a read-only root,
 a 256 MiB temporary filesystem,
 all Linux capabilities dropped,
 and `no-new-privileges`.
Candidate source is mounted read-only and copied into a private `~/temp/agent/wg-cidr-validation` directory.
A post-fetch size assertion enforces a 1 GiB scratch ceiling.
The process tool owns a ten-minute command ceiling without an external `timeout` wrapper.

Dependency fetch containers may contact the npm registry.
All later test,
 build,
 and consumer containers use `--network=none`.
Unexpected lifecycle execution,
 network access after fetch,
 a scratch-size breach,
 an undeclared subprocess,
 or a nonzero command stops that candidate.
Scratch trees and stopped containers are deleted after logs and checksums are recorded.

A pre-manifest environment probe ran `node --version`,
 `npm --version`,
 and `command -v` inside this image.
It loaded no candidate code,
 mounted no repository,
 used no network,
 and wrote no persistent state.
All candidate execution starts after these manifests.

### `cidr-tools` upstream manifest

Identity is `cidr-tools` 12.1.3 at commit
`a3b61d005c34b8eb91333ea5e78788ae24491d0b`,
 plus runtime dependency `ip-bigint` 9.0.7 at commit
`91be103d1e56c8ca49e7c1ad8ccb24e862d88154`.
The source locks pin every fetched artifact by integrity.

The fetch command copies each read-only source tree and runs
`corepack pnpm@11.17.0 install --frozen-lockfile`.
The first network-disabled lint attempt then failed with exact diagnostic `spawnSync pnpm ENOENT`.
Source inspection found `eslint-silverwind.js:1` invoking
`execFileSync('pnpm', ['exec', 'eslint', ...])`.
This was a validation-environment omission,
 not a candidate result.
A second bounded fetch step persisted Corepack's pnpm 11.17.0 cache under `/work/.corepack`.
A plain `pnpm` symlink to the Corepack binary did not dispatch by its symlink name.
The second test attempt therefore failed with exact Corepack diagnostic
`Unknown Syntax Error: Command not found` while receiving `exec eslint ...`.
No candidate test had started.
The next environment command used `corepack enable --install-directory /work/bin pnpm`,
 which created Corepack's actual pnpm shim.
That allowed `eslint-silverwind` to invoke pnpm,
 but placing the manager cache under `/work` made ESLint scan Corepack's own files.
The third attempt stopped on 26 lint errors under `/work/.corepack`;
 candidate source had not been linted.
The final environment keeps the same inspected cache and shims in a separate private `/manager` mount,
 removes the two environment directories from `/work`,
 adds `/manager/bin` to `PATH`,
 and keeps networking disabled.
This prevents manager implementation files from entering the upstream lint root.
`pnpm-workspace.yaml` sets `ignoreScripts: true`,
 so no package lifecycle can execute.
Corepack fetches and invokes pnpm 11.17.0;
 pnpm reads the lock,
 writes `node_modules`,
 and contacts the npm registry.
Post-fetch inspection found `unrs-resolver` 1.12.2 declaring `node postinstall.js`.
That script delegates native-package selection to `napi-postinstall`.
The workspace-wide `ignoreScripts` policy suppressed it,
 as the zero-stderr install transcript confirms.
The native resolver is a dev-only transitive of lint tooling and is not in the adopted runtime graph.

With networking disabled,
 the complete relevant command path is:

```text
# ~/temp/agent/cidr-tools-2026-07-28/Makefile
node_modules/.bin/eslint-silverwind --color .
node_modules/.bin/tsgo
node_modules/.bin/vitest run
node_modules/.bin/tsdown
```

The same four commands run for `ip-bigint`.
ESLint loads the checked-in config;
 `tsgo` loads the pinned TypeScript native-preview executable;
 Vitest may spawn bounded worker processes and load its pinned native transform packages;
 tsdown loads its checked-in config and pinned bundler.
These native packages are development-only and execute only inside the container.
The adopted runtime remains pure TypeScript-built JavaScript.
Success requires zero lint or type findings,
 passing tests,
 successful build,
 and no network attempt.

### `fast-cidr-tools` upstream manifest

Identity is `fast-cidr-tools` 0.3.4 at release commit
`d37506e5fcacc7a04760bd9c1b8c924d877bbc39`.
Its lock pins `foxts` 4.1.0 for upstream reproduction and all dev artifacts by integrity.
The real consumer manifest separately resolves the published `^4.1.0` range.

The fetch command is
`corepack pnpm@10.18.0 install --frozen-lockfile --ignore-scripts`.
The explicit flag prevents install lifecycle execution.
Post-fetch inspection found three suppressed dev-only commands:
 `@swc/core` 1.13.5 declares `node postinstall.js`,
 `oxc-resolver` 11.8.0 declares `napi-postinstall oxc-resolver 11.8.0 check`,
 and `unrs-resolver` 1.11.1 declares `napi-postinstall unrs-resolver 1.11.1 check`.
The SWC script can invoke npm to fetch a Wasm fallback when its native binding is absent;
 it did not run.
A `resolve` package test fixture also contains an inert `lerna bootstrap` field,
 but that nested fixture is not an installable package boundary.
Corepack and pnpm perform the same bounded registry reads and `node_modules` writes described in the shared
manifest.
The zero-stderr install transcript confirms all lifecycle commands remained suppressed.

With networking disabled,
 the command path is:

```text
# ~/temp/agent/fast-cidr-tools-0.3.4/.github/workflows/publish.yml
node_modules/.bin/bunchee --clean --target=es2021
node_modules/.bin/eslint --format=sukka .
node_modules/.bin/mocha --require @swc-node/register test/index.test.ts
```

An initial lint-before-build attempt failed because `benchmark.cts` imports the absent built artifact.
The exact diagnostic was `Unable to resolve path to module ./dist/es/index.mjs`.
The upstream workflow builds before lint,
 so the corrected command order matches `.github/workflows/publish.yml:31-35`.
Mocha loads `@swc-node/register`,
 which loads the lock-pinned `@swc/core` Linux x64 prebuilt.
ESLint loads the project config.
Bunchee loads Rollup,
 SWC,
 and its resolver plugins and writes `dist`.
These are development-only native and plugin boundaries.
Success requires zero lint findings,
 passing tests,
 successful build,
 and no network attempt.

### `@h3mantd/ip-kit` upstream manifest

Identity is `@h3mantd/ip-kit` 1.1.0 at commit
`cf077b0316ba484c5e357403e2aeb650b7b2695b`.
Its npm lock pins the development graph by integrity.

The fetch command is `npm ci --ignore-scripts`.
Post-fetch inspection found esbuild 0.18.20 and 0.21.5 each declaring `node install.js`;
 the explicit flag suppressed both lifecycle commands.
The installed dev graph also contains lock-pinned Linux x64 esbuild and Rollup native files,
 one source-map Wasm module,
 and unused vendored macOS and Windows `term-size` executables.
The selected lint,
 type,
 test,
 and build paths can load esbuild,
 Rollup,
 and source-map.
They do not call the `term-size` package.
Npm reads the lock,
 writes `node_modules`,
 and contacts the npm registry.
The install completed without lifecycle output,
 while npm reported 19 vulnerabilities in this development-only graph.
Runtime audit of the dependency-free published package remains separate.

With networking disabled,
 the command path is:

```text
# ~/temp/agent/ip-kit-2026-07-28/package.json
node_modules/.bin/eslint 'src/**/*.ts' 'tests/**/*.ts'
node_modules/.bin/tsc --noEmit
node_modules/.bin/vitest run
node_modules/.bin/tsup src/index.ts --dts --format esm,cjs --clean --tsconfig tsconfig.build.json
```

Vitest and tsup load their lock-pinned esbuild Linux x64 prebuilt;
 Vitest may spawn bounded workers;
 tsup writes `dist`.
The prebuilt is development-only.
Success requires zero lint or type findings,
 passing tests,
 successful build,
 and no network attempt.

### Published consumer manifest

A fresh disposable package installs exact `cidr-tools@12.1.3`,
 `fast-cidr-tools@0.3.4`,
 and `@h3mantd/ip-kit@1.1.0` with `npm install --ignore-scripts --save-exact`.
Post-fetch inspection resolved `cidr-tools` 12.1.3 with `ip-bigint` 9.0.7,
 `fast-cidr-tools` 0.3.4 with `foxts` 4.6.0 and its two dependencies,
 and `@h3mantd/ip-kit` 1.1.0.
Their lock integrities match the registry evidence.
The inspected runtime graphs contain no install lifecycle,
 native code,
 Wasm,
 or subprocess call.
The expected network endpoint is the npm registry;
 expected writes are a lock and `node_modules` under the private fixture.

After manifest and installed-tree inspection,
 a network-disabled container invokes:

```text
# ~/temp/agent/wg-cidr-consumer-validation.mjs
node wg-cidr-consumer-validation.mjs cidr-tools
node wg-cidr-consumer-validation.mjs fast-cidr-tools
node wg-cidr-consumer-validation.mjs @h3mantd/ip-kit
```

The planned `cidr-tools` validation boundary receives one additional network-disabled probe:

```text
# ~/temp/agent/wg-cidr-planned-validation-probe.mjs
node wg-cidr-planned-validation-probe.mjs
```

Its SHA-256 is
`359a04d1d32ad4838c359b00b5668109764be377000a9f03b4104587585ee6ef`.
It imports the already-inspected published `cidr-tools` graph and `node:net`,
then composes the specification's original-address `isIP` check,
`parseCidr`,
and explicit family bounds.
It must accept four valid dual-stack CIDRs and reject malformed address text,
nonstandard shorthand IPv4,
out-of-range prefixes,
trailing prefix junk,
stacked suffixes,
and a missing address.
The shared isolation,
resource,
read,
network,
write,
and stop conditions apply unchanged.
Success requires every assertion and exit status zero.

The first `ip-kit` consumer attempt exposed that `RangeSet.fromCIDRs` rejects host literals with exact
diagnostic `Invalid CIDR format: 192.0.2.1`.
The corrected consumer adapter converts hosts to `/32` or `/128` before family partitioning,
 which is additional production orchestration attributable to this candidate.
The corrected harness SHA-256 is
`b03a47741353be79c5fb92506a094e58f7c955e80b34fcf37cd02f93e896c3d6`.
It exercises five fixed vectors for dual-stack union,
 subtraction,
 minimization,
 complete subtraction,
 hosts,
 disjoint base intervals,
 and family ordering.
It also records six malformed-input parser outcomes.
Candidate modules read only their package files and inputs,
 write only JSON to stdout,
 and spawn nothing.
Each candidate succeeds only when every exact output equals the independently fixed expectation.

## Validation results

All local commands ran on Linux x86-64 in the recorded Node 24.18.0 container.
Full transcripts and checksums are under `~/temp/agent/wg-cidr-validation/logs`.
Dependency fetches used registry network access;
 every candidate test and consumer command used `--network=none`.
No command exceeded its resource or scratch ceiling.

### `cidr-tools` and `ip-bigint`

The two frozen installs completed in five seconds each,
 consumed 176,888 KiB and 176,716 KiB,
 and executed no lifecycle.
The final upstream commands each exited zero in eight seconds.
`cidr-tools` passed lint and type checking,
 nine Vitest tests,
 and its Node 22 build.
`ip-bigint` passed lint and type checking,
 one Vitest test,
 and its Node 22 build.
The only stderr was ESLint's performance-only `ESLintPoorConcurrencyWarning`.
Evidence is `proc_21-*.log` and `proc_22-*.log`.

The exact `cidr-tools` and `ip-bigint` commits also have successful upstream checks for Node 22,
 24,
 and
26 on Ubuntu,
 macOS,
 and Windows,
 plus Bun on all three systems.
Evidence is [the 12.1.3 CI run][cidr-ci] and [the ip-bigint 9.0.7 CI run][ip-bigint-ci].
No production source imports an operating-system,
 filesystem,
 process,
 worker,
 or network API.
Platform status is pass with high confidence.

The published `cidr-tools` package passed all five fixed consumer vectors.
Its parser rejected trailing-junk,
 stacked,
 and missing-address prefixes.
It accepted malformed IPv4 text and out-of-range prefixes,
 matching its documented rudimentary validation.
The planned `node:net` address check and explicit family prefix bound reject those accepted cases.
Evidence is `proc_10-stdout.log`;
 status is consumer pass.

The separately manifested specification-boundary probe composed `parseCidr` with `isIP` on the original address
text and explicit family bounds.
It accepted four valid dual-stack CIDRs and rejected all seven invalid cases,
including nonstandard shorthand IPv4,
out-of-range prefixes,
and every malformed case from the shared consumer catalog.
The network-disabled Node 24 process exited zero with `planned cidr-tools validation boundary passed`.
Evidence is `proc_28-stdout.log`,
SHA-256 `c7eb84794e0bd34a67d8d3dd590a283d97f1962666d39bff70ffb2bf4cb19241`.
Status is compensating validation boundary pass.

The local `ip-bigint` build is byte-identical to its published JavaScript.
The local `cidr-tools` build differs from the published JavaScript only in the bundler-generated local default
identifier,
 `work_default` instead of `cidr_tools_default`,
 because the build directory basename differs.
The declarations are byte-identical and the executable export graph and bodies are otherwise identical.
Source-to-artifact status is pass.

The non-default coverage command `pnpm exec vitest --coverage` was omitted.
It executes the same test files and changes only coverage accounting,
 so it cannot add a consumed code path.
Benchmarks and update or release commands were also omitted because they do not validate the consumed API.

### `@h3mantd/ip-kit`

The frozen install completed in 49 seconds,
 consumed 143,248 KiB,
 and executed no lifecycle.
Npm reported 19 advisories in the development graph;
 the published runtime package is dependency-free and its exact-version advisory query was empty.
The upstream command exited zero in five seconds:
 lint and type checking passed,
 all 160 tests in 13 files passed,
 and ESM,
 CommonJS,
 and declarations built.
Evidence is `proc_9-*.log`.

The first published consumer attempt failed before set arithmetic because `RangeSet.fromCIDRs` rejects a host
literal with `Invalid CIDR format: 192.0.2.1`.
After the candidate adapter converted hosts to `/32` or `/128` and partitioned inputs by family,
 all five
fixed vectors passed.
Parser probes rejected malformed addresses,
 stacked suffixes,
 and out-of-range prefixes,
 but accepted
`192.0.2.1/24junk` as prefix 24.
An exact-prefix consumer guard is therefore required.
Evidence is `proc_12-stderr.log` and `proc_16-stdout.log`.
Status is consumer pass with additional production orchestration.
The parser defect,
 verified consumer guard,
 minimal upstream patch,
 and 161-test prototype are recorded in
[the `ip-kit` troubleshooting report](../troubleshooting/ip-kit-trailing-prefix-text.md).

The local ESM build is byte-identical to the published artifact.
The exact commit's upstream Node 18,
 20,
 and 22 Linux checks passed.
No production source imports an operating-system,
 filesystem,
 process,
 worker,
 or network API;
 the consumed operations use language-defined BigInt arithmetic.
No macOS or Windows runner was locally available and upstream does not test those systems.
Platform status is pass with medium confidence from the pure runtime boundary rather than direct runner evidence.

The non-default `npm run test:coverage` command was omitted because it executes the same 13 test files and changes
only coverage accounting.
Watch mode,
 formatting,
 examples,
 changesets,
 and release commands cannot affect the consumed runtime operation and were omitted.

### `fast-cidr-tools`

The frozen install completed in five seconds,
 consumed 212,580 KiB,
 and executed no lifecycle.
After matching upstream's build-before-lint order,
 build and all seven tests passed in four seconds.
Lint exited zero but reported four naming-convention warnings.
The local ESM build is byte-identical to the published artifact.
Evidence is `proc_15-*.log`.

The published consumer failed the first set vector.
For allowed `10.0.0.0/30` and `10.0.0.8/30` with disallowed `10.0.0.0/28`,
 exact subtraction is empty.
Version 0.3.4 returned `10.0.0.8/30`.
Evidence is `proc_11-stderr.log`.

The source cause is `src/exclude.ts:111-127`.
When an exclusion changes one base,
 the loop appends remainders,
 splices the current base,
 then unconditionally increments `index`.
That increment skips the next original base shifted into the removed slot.
The defect is independent of input validation or output sorting and violates the exact union-minus-union hard
constraint.
Status is hard-gate fail;
 the candidate is not scored.
The root cause,
 minimal patch,
 upstream filing draft,
 and passing seven-group prototype are recorded in
[the `fast-cidr-tools` troubleshooting report](../troubleshooting/fast-cidr-tools-multiple-base-exclusion.md).

## Scoring and sensitivity

Hard-gate failures remain outside arithmetic.
The frozen weights total 18 and produce a maximum score of 72.

### `cidr-tools` 12.1.3

#### Interface fit and production-code footprint

Rating is `4 of 4`.
Confidence is high.
Weight is `5`.
Score effect is 20 points.

The consumer calls `excludeCidr(allowed, disallowed)` directly.
It needs no family partition,
host conversion,
or result reconstruction around set subtraction.

#### Validation clarity and error ergonomics

Rating is `3 of 4`.
Confidence is high.
Weight is `5`.
Score effect is 15 points.

`parseCidr` rejects malformed CIDR structure and trailing junk.
Its documented rudimentary address parsing means the planned `node:net` check and explicit family bounds remain
necessary,
but those checks have one clear ownership boundary.

#### Runtime dependency surface and human auditability

Rating is `4 of 4`.
Confidence is high.
Weight is `4`.
Score effect is 16 points.

The complete runtime graph is 684 direct lines plus 307 lines in one same-author dependency.
Both modules are strict TypeScript,
have zero suppressions or debt markers,
and contain no native,
Wasm,
operating-system,
network,
filesystem,
or process boundary.

#### Upstream test quality

Rating is `3 of 4`.
Confidence is high.
Weight is `3`.
Score effect is 9 points.

Tests cover both families,
empty and full subtraction,
overlaps,
hosts,
minimization,
ordering,
and prior unsorted IPv6 regressions.
No fuzzing or mutation harness exists.

#### Maintenance and release hygiene

Rating is `3 of 4`.
Confidence is high.
Weight is `1`.
Score effect is 3 points.

The maintainer responded to every sampled issue,
merged all sampled pull requests,
and released fixes,
including 12.1.3.
Maintenance remains concentrated in one person.

Arithmetic is `5 * 4 + 5 * 3 + 4 * 4 + 3 * 3 + 1 * 3 = 63`.
The score is `63 / 72 = 87.5%`.

### `@h3mantd/ip-kit` 1.1.0

#### Interface fit and production-code footprint

Rating is `2 of 4`.
Confidence is high.
Weight is `5`.
Score effect is 10 points.

The set engine passed after the adapter converted hosts to host CIDRs,
partitioned both inputs by family,
built two pairs of `RangeSet` objects,
subtracted each pair,
converted each result to CIDRs,
and joined the families.

#### Validation clarity and error ergonomics

Rating is `1 of 4`.
Confidence is high.
Weight is `5`.
Score effect is 5 points.

Errors are typed and most malformed inputs are rejected,
but `CIDR.parse` partially parses non-decimal prefixes.
Exact input rejection therefore requires a project-owned prefix-text parser before the package parser.

#### Runtime dependency surface and human auditability

Rating is `3 of 4`.
Confidence is high.
Weight is `4`.
Score effect is 12 points.

The runtime is dependency-free strict TypeScript with zero suppressions or debt markers and no native or
operating-system boundary.
Its 1,805 lines across 11 files are broader than the consumed task requires.

#### Upstream test quality

Rating is `2 of 4`.
Confidence is high.
Weight is `3`.
Score effect is 6 points.

The 1,255 test lines cover broad address operations and example-based set laws.
They miss non-decimal prefix text and do not combine IPv6 subtraction with minimal CIDR conversion.
No fuzzing or mutation harness exists.

#### Maintenance and release hygiene

The low-signal rating range is `1 through 3`.
Its provisional midpoint is `2`.
Confidence is low.
Weight is `1`.
The provisional score effect is 2 points,
with endpoints of 1 and 3 points.

Three recent releases and 22 owner commits support at least a weak rating.
No external maintenance sample exists,
and CI covers Ubuntu only,
so evidence does not distinguish weak from good response and release hygiene.

Provisional arithmetic is `5 * 2 + 5 * 1 + 4 * 3 + 3 * 2 + 1 * 2 = 35`.
The provisional score is `35 / 72 = 48.6%`.
The maintenance endpoints produce `34 / 72 = 47.2%` and `36 / 72 = 50.0%`.

### Sensitivity results

Maintenance and release hygiene is the only equal-default criterion.
Raising its weight one input at a time gives:

- weight 1:
  `cidr-tools` scores `63 / 72 = 87.5%`;
  `ip-kit` scores `35 / 72 = 48.6%`;
- weight 2:
  `cidr-tools` scores `66 / 76 = 86.8%`;
  `ip-kit` scores `37 / 76 = 48.7%`;
- weight 3:
  `cidr-tools` scores `69 / 80 = 86.3%`;
  `ip-kit` scores `39 / 80 = 48.8%`;
- weight 4:
  `cidr-tools` scores `72 / 84 = 85.7%`;
  `ip-kit` scores `41 / 84 = 48.8%`;
- weight 5:
  `cidr-tools` scores `75 / 88 = 85.2%`;
  `ip-kit` scores `43 / 88 = 48.9%`.

The `ip-kit` maintenance range endpoints at the frozen weight produce 47.2% and 50.0%.
They remain at the frozen weight because changing an endpoint and its weight together would change two inputs,
outside the required one-at-a-time method.
No exact rating has medium confidence,
so the one-step medium-confidence arm has no input.
Across every defined one-at-a-time test,
normalized scores range from 85.2% to 87.5% for `cidr-tools` and 47.2% to 50.0% for `ip-kit`.
The order never changes.
These ranges do not claim stability under simultaneous changes to several inputs.

## Finalist tradeoffs and ranking

### 1. `cidr-tools` 12.1.3

Pros:

- one direct subtraction call matches the required set model;
- smallest audited runtime source surface;
- every consumer vector passed without set-operation adapter machinery;
- exact-release CI passed on Node 22,
  24,
  and 26 across Ubuntu,
  macOS,
  and Windows;
- active,
  measured maintainer response and release flow.

Cons:

- rudimentary parsing requires the planned address and family-prefix guards;
- one same-author runtime dependency expands the provenance boundary;
- single-maintainer concentration and no fuzz or mutation suite remain.

### 2. `@h3mantd/ip-kit` 1.1.0

Pros:

- dependency-free,
  pure TypeScript runtime;
- exact subtraction and minimal CIDR conversion passed every adapted dual-stack consumer vector;
- custom error types and broad address-operation tests.

Cons:

- family partitioning,
  host conversion,
  object construction,
  and result conversion add production orchestration;
- `CIDR.parse` accepts non-decimal prefix text,
  requiring a second project-owned parser guard;
- more unused source surface and weaker relevant set-test coverage;
- no upstream macOS or Windows CI and no external maintenance sample.

The complete finalist ranking is `cidr-tools` > `@h3mantd/ip-kit`.

`cidr-tools` ranks ahead because both pass the hard gates,
while its direct API,
smaller audited boundary,
clearer validation ownership,
stronger relevant tests,
and measured cross-platform CI all match this decision better.
The score order survives every required one-at-a-time sensitivity test.

Hard-gate exits are not numeric runners-up.
`fast-cidr-tools` fails exact subtraction,
`ip-num` violates the install-lifecycle constraint,
and `ip-address` is a parser toolkit without a collection set engine.

## Recommendation

Retain `cidr-tools`.
The exact validated recommendation is `cidr-tools` 12.1.3 with `ip-bigint` 9.0.7.
Do not replace it with `beaugunderson/ip-address`,
because that package has no union,
subtraction,
or minimal CIDR-cover operation and would force a project-owned interval engine.
Do not replace it with `@h3mantd/ip-kit` for this CLI,
because its broader set API requires more consumer code and its parser requires an additional exact-prefix guard.

Confidence in the ordering is high.
The material retained risk is `cidr-tools`' permissive address and prefix-range parsing,
which makes the planned `node:net` and family-bound checks mandatory rather than optional.
Exact cross-platform CI,
source-to-artifact comparison,
and fixed consumer vectors support the recommendation.
Evidence limits are the absence of fuzz or mutation suites and the lack of local macOS or Windows runners.

This recommendation applies to the exact validated release.
It does not authorize a product dependency,
catalog range,
planning specification,
or decision-record change.
Adoption remains a separate action.

[cidr-ci]: https://github.com/silverwind/cidr-tools/actions/runs/30333360243
[cidr-issue-30]: https://github.com/silverwind/cidr-tools/issues/30
[cidr-registry]: https://registry.npmjs.org/cidr-tools/12.1.3
[cidr-release]: https://github.com/silverwind/cidr-tools/releases/tag/12.1.3
[fast-pr-7]: https://github.com/SukkaW/fast-cidr-tools/pull/7
[fast-registry]: https://registry.npmjs.org/fast-cidr-tools/0.3.4
[fast-release-commit]: https://github.com/SukkaW/fast-cidr-tools/commit/d37506e5fcacc7a04760bd9c1b8c924d877bbc39
[fast-workflow-run]: https://github.com/SukkaW/fast-cidr-tools/actions/runs/19178337000
[ip-address-advisory]: https://github.com/beaugunderson/ip-address/security/advisories/GHSA-mwp4-54f8-5fhr
[ip-address-exports]: https://github.com/beaugunderson/ip-address/blob/be7e626c0d49fccb518899f520a3fb64ee189741/src/ip-address.ts
[ip-address-registry]: https://registry.npmjs.org/ip-address/10.3.1
[ip-address-repo]: https://github.com/beaugunderson/ip-address
[ip-bigint-ci]: https://github.com/silverwind/ip-bigint/actions/runs/30332965290
[ip-kit-registry]: https://registry.npmjs.org/%40h3mantd%2Fip-kit/1.1.0
[ip-kit-release]: https://github.com/h3mantD/ip-kit/releases/tag/v1.1.0
[ip-num-registry]: https://registry.npmjs.org/ip-num/1.6.1
