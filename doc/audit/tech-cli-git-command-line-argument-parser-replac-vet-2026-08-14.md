# cli-git command-line argument parser replacement vet report

- Status:
   complete;
   no named alternative recommended
- Lifecycle phase:
   terminal no-validated-finalist result
- Subject:
   cli-git command-line argument parser replacement
- Decision scope:
   evaluate `jackspeak`,
   `type-flag`,
   and `argue-cli` as replacements for cli-git's handwritten
  Git argv region parser and management parser;
   evaluate no unnamed alternative
- Start date:
   2026-08-14
- Last updated:
   2026-08-14
- Completion repository revision before final report commit:
  `144f767ce9e659d4591ef0082e06f8953e4b95a9`
- Governing skill:
   `.agents/skills/choosing-technology/SKILL.md`
- Governing skill commit:
   `a05818ad70a40e5769a36de669697ba109891b31`
- Governing skill SHA-256:
   `393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`
- Compatibility fingerprint:
   `a184aef37ab9cd54b2c0c28a9c662b6957497ef1c031852a589555c05b0bd8bb`
- Active audit owner:
   Pi process `298127`
- Prior compatible report:
   none found under `doc/audit/`
- Related incompatible report:
   `doc/audit/tech-cli-git-parser-migration-to-cac-vet-2026-08-14-a48b54e2.md`

## Context

Cli-git is a published ESM package targeting Node `^22.18.0 || >=24.11.0`.
Its PATH-shadowing `git` executable parses partially declared Git option surfaces
before policy decisions that can block or transform destructive operations
(`package/git-policy/cli/package.json:1-45` and `package/git-policy/cli/README.md:1-22`).

The incumbent `parseArgv` implementation has eight production consumer modules and eleven production call sites,
measured with repository-wide import and call searches on 2026-08-14.
The implementation is 415 physical lines and its direct unit test is 333 physical lines.
Those physical counts include project-required TSDoc and formatting.
The relevant boundary is behavior rather than raw file length:

- exact aliases grouped under one result key;
- repeated flag counts and ordered repeated option values;
- options recognized after positionals;
- bare `-` retained as positional and `--` treated as the option terminator;
- declared separated values allowed to start with `-`;
- declared `--name=value` values recognized;
- partially declared Git grammars allowed to retain unknown plain and joined options;
- unknown joined options never reclassified as pathspecs;
- missing declared values refused;
- strict management commands able to reject unknown options and positionals.

These behaviors are implemented in `package/git-policy/cli/src/parser/argv.ts:27-412` and pinned in
`package/git-policy/cli/src/parser/argv.unit.test.ts:60-330`.
Commit parsing also normalizes attached short-option values and performs a separate Git-arity pathspec scan because no
partial option schema can know every unknown Git option's arity
(`package/git-policy/cli/src/parser/commit.ts:188-280`).

The handwritten parser replaced `@optique/core` after a measured wrapper case with one thousand pathspecs spent
2.56 seconds of 4.24 seconds building discarded no-match diagnostics.
The replacement reduced that complete wrapper case to 0.89 seconds.
This historical measurement is recorded in commit `4879a44e6a2460ab3c2531744e24a1f2aef27aeb`.
It is not yet a current candidate benchmark.

The related CAC-only audit is incompatible with this report's candidate set and is not reopened or ranked here.
Its incumbent inventory corroborates that the closed management grammar and partially declared forwarded-Git regions are
different parser roles.
No CAC evidence is applied to a named candidate.

Keeping the incumbent is a parity baseline required by the replacement overlay.
It is not added to the user-limited alternative set and will not be scored as a named alternative.

## Classification and overlays

Each named candidate is one inspectable open-source local technology.
The following overlays apply to each:

- incumbent dependency replacement;
- high-trust execution,
   because parse facts gate Git safety policies;
- human auditability;
- multi-platform Node use on Linux,
   macOS,
   and Windows.

The SaaS,
 sensitive-data,
 native,
 Wasm,
 browser,
 and managed-service gates are not applicable unless source inspection
finds such a boundary.

## Hard constraints

- Candidate is exactly `jackspeak`,
   `type-flag`,
   or `argue-cli`.
- Source and published-artifact provenance are inspectable.
- License is compatible with cli-git's LGPL-3.0-or-later distribution.
- ESM works on Node `^22.18.0 || >=24.11.0` without native,
   Wasm,
   or prebuilt runtime code.
- Supplied argv parsing does not mutate ambient environment or require ambient `process.argv`.
- Replacement removes the incumbent token scan rather than recreating an equivalent handwritten scan in an adapter.
- Declared value options accept separated dash-led values and joined equals values,
  and expose missing values as failure.
- Parsing accepts declared options after positionals and preserves repeated flag counts and option-value order.
- Parsing preserves exact `--` termination and bare `-` positional behavior.
- Partial Git schemas retain unknown plain and joined options without rejecting valid forwarded Git syntax or treating a
  joined option as a pathspec.
- Unknown plain options preserve cli-git's tentative consumption of one following non-option token.
- Management parsing can reject unknown options and unexpected positionals.
- Parser behavior avoids per-pathspec diagnostic work that recreates the measured Optique slowdown.

A candidate that needs a small result-shaping adapter can pass.
A candidate that needs a second argv scanner to restore these semantics has not replaced the incumbent parser and fails.

## Frozen scoring rubric

Every criterion has default weight 1 because the user specified no outcome-changing preference:

- semantic coverage beyond the hard minimum;
- replacement completeness and adapter simplicity;
- runtime performance on representative short and long argv;
- human auditability and runtime dependency surface;
- TypeScript and package integration fit;
- maintenance,
   release,
   and provenance confidence.

Each validated finalist will receive a 0 to 4 rating with evidence and confidence.
Hard-gate failures remain outside arithmetic.
Sensitivity will raise each default weight independently from 1 through 5,
vary every medium-confidence and low-confidence exact rating by one point,
and test every evidence range endpoint.

Unresolved preferences:
 none at context freeze.
If sensitivity exposes a controlling non-measurable preference,
the report will ask only for that preference and will not force a winner.

## Query schedule

The user fixed the candidate set.
Searches may discover evidence or comparison vocabulary but cannot add or evaluate another package.
The supplied npmx pages were fetched first to confirm package identity and exact stable-version leads.
The evidence-discovery schedule then froze as follows.

### Package registry and ecosystem index

- `https://npmx.dev/package/jackspeak`
- `https://npmx.dev/package/type-flag`
- `https://npmx.dev/package/argue-cli`
- exact npm registry metadata and tarball integrity for `jackspeak@4.2.3`
- exact npm registry metadata and tarball integrity for `type-flag@4.5.0`
- exact npm registry metadata and tarball integrity for `argue-cli@3.1.0`

Filters:
 exact package and stable version only.
Sort:
 not applicable.
Pagination:
 exact records have no cursor.

### Repository host

- exact release tag and source revision for each package;
- repository metadata,
   releases,
   security policy,
   workflows,
   and default branch;
- issues created or updated from 2025-08-14 through 2026-08-14;
- ten most recently updated pull requests when at least ten exist,
  otherwise every available pull request in that period;
- maintenance publication path from tag to npm artifact.

Filters:
 exact repository and access-date window.
Sort:
 updated descending for issues and pull requests.
Pagination:
 continue to exhaustion for samples of at most twenty issues and ten pull requests.

### Broader web evidence

Initial literal queries:

- `jackspeak 4.2.3 unknown options pass through`
- `type-flag 4.5.0 unknown flags forwarding`
- `argue-cli 3.1.0 readOptions unknown options`
- `jackspeak type-flag argue-cli comparison`

One expansion round from vocabulary in the supplied documentation:

- `jackspeak stopAtPositional util.parseArgs strict unknown options`
- `type-flag ignore unknown-flag option terminator missing value`
- `argue-cli setArgs readOptions internal state option terminator`

Filters:
 no package exclusion filters.
Sort:
 provider relevance.
Pagination:
 provider result set,
 with result count recorded after execution.
The expansion round is frozen;
 later vocabulary will not add searches.

### This repository

- exact imports and calls of `parseArgv` and `tryParseArgv`;
- parser tests and command-specific adapters;
- package runtime and bundling constraints;
- parser replacement history and measured regression evidence;
- prior audit,
   decision,
   planning,
   and troubleshooting references.

Filters:
 `package/git-policy/cli` plus repository documentation.
Sort and pagination:
 not applicable to repository text and history searches.

## Query ledger and saturation

### Package registry and ecosystem index

The three exact npmx pages each resolved one user-named package and its stable version.
No page added a candidate because the user fixed the set.
Exact npm registry records then resolved source repositories,
license identifiers,
runtime metadata,
published tarball integrity,
and package signatures.

- `jackspeak@4.2.3` identifies source commit
  `a58b42f39e2fb04b28b8169005a5ddbc3302730e` and a seventeen-file tarball.
- `type-flag@4.5.0` identifies source commit
  `6e0c46911ea64c829459a27bfaf1b45e8e335869`,
   a seven-file tarball,
  trusted GitHub publisher,
  and SLSA provenance.
- `argue-cli@3.1.0` identifies a nineteen-file tarball but publishes neither `gitHead` nor a provenance attestation.

Each exact registry record is exhausted without pagination.
The downloaded tarballs match their registry SHA-512 integrity values.

### Repository host

All three exact repositories were cloned at the stable-version source revision.
Source,
tests,
workflows,
release metadata,
security-policy presence,
twelve-month issue activity,
twelve-month commit activity,
and the required pull-request sample were inspected.

Maintenance samples:

- Jackspeak had two issues and two pull requests updated in the twelve-month window,
  so all were inspected.
- Type-flag had four issues and fifty-three pull requests updated in the window;
  every issue and the ten most recently updated pull requests were inspected.
- Argue had one issue and thirty-nine pull requests updated in the window;
  that issue and the ten most recently updated pull requests were inspected.

Exact release-commit check runs were inspected for each candidate.
No repository contains a security policy,
fuzz harness,
or mutation harness.
The broader source and test searches completed without a negative filter.

### Broader web evidence

Each of the seven frozen searches returned ten results through the configured provider.
The results consistently led back to official npm metadata,
source repositories,
release pages,
and exact source files.
The three-way comparison query produced no direct comparison of all named candidates.
Several Argue results referred to a different repository with the same ordinary word;
those were category mismatches and were not evaluated.
The provider exposes no page cursor through this harness.
Exact registry and repository enumeration independently completed the named-candidate evidence class.

### This repository

Uncapped source and history searches found eight production consumer modules and eleven production call sites for the
shared parser.
They also found the completed CAC-only report and handover.
Those incompatible-scope documents added incumbent parity evidence but no candidate.
No current manifest or lockfile declares any of the three named alternatives for cli-git.

### Terminal discovery result

Discovery is saturated with the three user-named candidates.
All three pass initial category,
license,
source-availability,
and runtime-platform screening and advance to targeted semantic evidence.
No unnamed technology was discovered,
screened,
or ranked.

## Candidate ledger

### jackspeak

- Discovery source:
   user-supplied <https://npmx.dev/package/jackspeak>,
   accessed 2026-08-14
- Version lead:
   `4.2.3`
- Repository: <https://github.com/isaacs/jackspeak>
- Base category:
   inspectable open-source local technology
- Overlays:
   replacement,
   high-trust,
   human-auditability,
   multi-platform
- Clone:
   `~/temp/agent/jackspeak-2026-08-14`
- Pinned revision:
   `a58b42f39e2fb04b28b8169005a5ddbc3302730e`,
   tag `v4.2.3`
- Screening state:
   hard-gate exit after published-artifact semantic confirmation

### type-flag

- Discovery source:
   user-supplied <https://npmx.dev/package/type-flag>,
   accessed 2026-08-14
- Version lead:
   `4.5.0`;
   the `5.0.0-beta.18` prerelease is outside the stable-version scope
- Repository: <https://github.com/privatenumber/type-flag>
- Base category:
   inspectable open-source local technology
- Overlays:
   replacement,
   high-trust,
   human-auditability,
   multi-platform
- Clone:
   `~/temp/agent/type-flag-2026-08-14`
- Pinned revision:
   `6e0c46911ea64c829459a27bfaf1b45e8e335869`,
   tag `v4.5.0`
- Screening state:
   hard-gate exit after published-artifact semantic confirmation

### argue-cli

- Discovery source:
   user-supplied <https://npmx.dev/package/argue-cli>,
   accessed 2026-08-14
- Version lead:
   `3.1.0`
- Repository: <https://github.com/TrigenSoftware/Argue>
- Base category:
   inspectable open-source local technology
- Overlays:
   replacement,
   high-trust,
   human-auditability,
   multi-platform
- Clone:
   `~/temp/agent/argue-cli-2026-08-14`
- Pinned revision:
   `45db68f4acce979d0ba725ae83e320a0e906165a`,
   tag `v3.1.0`
- Screening state:
   hard-gate exit after published-artifact semantic confirmation;
  reproducible build is no longer decision-relevant

## Evidence records

### Repository context record R1

- Candidate:
   incumbent parity baseline at repository revision
  `b92e07dfde48ee7793d464d9ab4866e09e97bde1`
- Claim:
   replacement must preserve partial Git grammar behavior rather than only parse a conventional closed CLI schema
- Relevance:
   parse facts gate policy enforcement and command transformation
- Gate:
   replacement parity and high-trust overlay
- Status:
   hard constraint
- Primary source:
   `package/git-policy/cli/src/parser/argv.ts:215-412`,
   accessed 2026-08-14
- Tests:
   `package/git-policy/cli/src/parser/argv.unit.test.ts:60-330`
- Consumer source:
   `package/git-policy/cli/src/parser/commit.ts:188-280` and
  `package/git-policy/cli/src/management-parser.ts:175-265`
- Outcome:
   requirements frozen before candidate ratings

### Repository performance record R2

- Candidate:
   incumbent parity baseline at commit `4879a44e6a2460ab3c2531744e24a1f2aef27aeb`
- Claim:
   discarded per-nonmatch diagnostics caused a measured long-pathspec regression in `@optique/core`
- Relevance:
   candidate parsing must not repeat that failure shape
- Gate:
   replacement parity and scored performance concern
- Status:
   historical evidence only;
   not repeated after candidate hard-gate exits
- Primary source:
   commit message and diff for `4879a44e6a2460ab3c2531744e24a1f2aef27aeb`,
   accessed 2026-08-14
- Counterevidence limit:
   the recorded 0.89 second result measures the complete wrapper,
  not parser-only cost or current hardware
- Outcome:
   benchmark shape retained;
   historical timings will not be used as candidate scores

### Jackspeak source record J1

- Candidate:
   `jackspeak@4.2.3`,
   commit `a58b42f39e2fb04b28b8169005a5ddbc3302730e`
- Claim:
   Jackspeak is a closed-schema parser that rejects every undeclared option even though it invokes
  `util.parseArgs` with `strict: false`
- Relevance:
   cli-git intentionally declares only guard-relevant subsets of Git's evolving grammar
- Gate:
   replacement parity and high-trust semantic fit
- Status:
   hard-gate fail,
   confirmed by published artifact
- Primary source:
   clone `~/temp/agent/jackspeak-2026-08-14`,
  `src/index.ts:701-816`,
   accessed 2026-08-14
- Adjacent excerpt:
   `parseRaw` iterates option tokens and throws `Unknown option` whenever no config field owns the name
- Counterevidence:
   required-value failure,
  repeat collection,
  options after positionals,
  and `--` handling are implemented by Node's parser
- Outcome:
   retaining valid unknown Git options would require declaring the complete Git grammar or reparsing raw argv

### Jackspeak package and auditability record J2

- Candidate:
   `jackspeak@4.2.3`
- Claim:
   runtime is inspectable and platform-compatible but has the largest audited surface in this set
- Gate:
   license,
   provenance,
   dependency surface,
   human auditability,
   and multi-platform
- Status:
   pass for screening;
   scored concern if semantics survive
- Primary metadata: <https://registry.npmjs.org/jackspeak/4.2.3>,
   accessed 2026-08-14
- Artifact integrity:
   registry and measured integrity
  `sha512-ykkVRwrYvFm1nb2AJfKKYPr0emF6IiXDYUaFx4Zn9ZuIH7MrzEZ3sD5RlqGXNRpHtvUHJyOnCEFxOlNDtGo7wg==`
- Source mapping:
   published ESM source map contains byte-identical `src/index.ts`
- License:
   BlueOak-1.0.0;
   distribution must retain license text or link
- Runtime graph:
   one same-author dependency,
  `@isaacs/cliui@9.0.0`,
   with no further runtime dependencies and no native,
  Wasm,
  downloaded,
  filesystem,
  network,
  or process-execution boundary
- Measured production source:
   1,066 code lines in Jackspeak plus 1,014 code lines in eight cliui files
- Platform evidence:
   release commit passed Node 20 and 22 on Ubuntu,
  macOS,
  and Windows under Bash and PowerShell
- Test evidence:
   strict TypeScript and direct parsing tests;
   no fuzzing or mutation harness
- Outcome:
   screening pass

### Type-flag source record T1

- Candidate:
   `type-flag@4.5.0`,
   commit `6e0c46911ea64c829459a27bfaf1b45e8e335869`
- Claim:
   stable type-flag does not implement cli-git's declared-value or unknown-option token roles
- Relevance:
   dash-led values and unknown option arity directly affect policy facts
- Gate:
   replacement parity and high-trust semantic fit
- Status:
   hard-gate fail,
   confirmed by published artifact
- Primary source:
   clone `~/temp/agent/type-flag-2026-08-14`,
  `src/argv-iterator.ts:46-122` and `src/type-flag.ts:40-140`,
   accessed 2026-08-14
- Adjacent excerpt:
   seeing a new flag first invokes any pending value callback with `undefined`;
  string parsing then applies the parser to `''`
- Counterevidence:
   `--` termination,
  options after positionals,
  short groups,
  joined values,
  repeated typed values,
  and an unknown-flags result are implemented
- Outcome:
   `-m -a` cannot preserve `-a` as the declared message value,
  and `--message` at end cannot be distinguished from explicit empty value;
  unknown plain options and following positionals are returned in separate normalized collections
  rather than one ordered token sequence

### Type-flag package and auditability record T2

- Candidate:
   `type-flag@4.5.0`
- Claim:
   package has the strongest publication provenance and no runtime dependency
- Gate:
   license,
   provenance,
   dependency surface,
   human auditability,
   and multi-platform
- Status:
   pass for screening
- Primary metadata: <https://registry.npmjs.org/type-flag/4.5.0>,
   accessed 2026-08-14
- Artifact integrity:
   registry and measured integrity
  `sha512-1aLzxcL6u1O9XHieAJBBX9U4QzwzDTWN0ER9M7QQSvS24NBmGM+N8FcghlgHAzOvDlEEpOx4hEml9CVcDnflcw==`
- Provenance:
   decoded npm SLSA statement names subject
  `pkg:npm/type-flag@4.5.0` with the measured SHA-512 from the artifact-integrity record;
  it names repository `privatenumber/type-flag`,
  workflow `.github/workflows/release.yml`,
  resolved commit `6e0c46911ea64c829459a27bfaf1b45e8e335869`,
  Actions run `27501307853`,
  and Rekor log index `1817329468`
- License and runtime:
   MIT,
  no runtime dependency,
  and no native,
  Wasm,
  downloaded,
  filesystem,
  network,
  or process-execution boundary
- Measured production source:
   626 code lines in eight files
- Platform evidence:
   release commit passed its Ubuntu workflow on the repository Node version and Node 18;
  no upstream macOS or Windows job exists
- Test evidence:
   extensive parsing and type tests plus an upstream benchmark;
  no fuzzing or mutation harness
- Outcome:
   screening pass

### Argue source record A1

- Candidate:
   `argue-cli@3.1.0`,
   commit `45db68f4acce979d0ba725ae83e320a0e906165a`
- Claim:
   Argue scans known options out of process-global mutable argv but does not model option termination
  or return role-tagged unknown-option facts;
  built-in readers overwrite scalar repeats,
  while custom readers can express counts and ordered known values
- Relevance:
   cli-git calls the shared parser repeatedly as a pure function and must preserve exact token roles
- Gate:
   replacement parity,
  process integration,
  and high-trust semantic fit
- Status:
   hard-gate fail,
   confirmed by published artifact
- Primary source:
   clone `~/temp/agent/argue-cli-2026-08-14`,
  `src/argv.ts:1-24` and `src/options.ts:1-109`,
   accessed 2026-08-14
- Adjacent excerpt:
   module initialization copies `process.argv` into one exported array;
  `setArgs` and every reader mutate that shared array;
  the `--` token is merely not an option,
  so later options continue to parse;
  `removePrefix` strips either one or two leading dashes before a custom reader sees the name
- Counterevidence:
   arbitrary aliases,
  joined equals values,
  dash-led separated values,
  unknown-token preservation in leftover argv,
  and options after positionals work
- Outcome:
   an adapter would need pre-splitting for termination,
  original-token inspection for exact prefixes,
  and a leftover classifier for unknown-option versus positional roles;
  custom readers can supply counts and ordered known values

### Argue package and auditability record A2

- Candidate:
   `argue-cli@3.1.0`
- Claim:
   package has the smallest source surface but weaker release provenance and platform coverage
- Gate:
   license,
   provenance,
   dependency surface,
   human auditability,
   and multi-platform
- Status:
   pass for source mapping;
   build provenance remains lower confidence
- Primary metadata: <https://registry.npmjs.org/argue-cli/3.1.0>,
   accessed 2026-08-14
- Artifact integrity:
   registry and measured integrity
  `sha512-DhBpBfXL4SS2uC0N922MMajKR3CdrTG0u2or1PNYgXMsrSzViJrbtvT0nCLlLGUI0plam/ZZCs7aAauHtW9thw==`
- Source mapping:
   published source map embeds byte-identical copies of all five runtime implementation files from the
  release tag
- Provenance limit:
   registry metadata has no `gitHead` and no SLSA attestation;
  release workflow uses a mutable `@latest` action
- License and runtime:
   MIT,
  Node `>=22`,
  no runtime dependency,
  and no native,
  Wasm,
  downloaded,
  filesystem,
  network,
  or process-execution boundary
- Measured production source:
   298 code lines in seven files
- Platform evidence:
   release commit passed type and unit jobs on Ubuntu and Node 24;
  no upstream macOS or Windows job exists
- Test evidence:
   direct unit and type tests with reported 98.34 percent line coverage;
  no `--` terminator case,
  fuzzing,
  or mutation harness
- Outcome:
   source screening pass;
  reproducible build was not run after the semantic hard-gate exit

### Maintenance record M1

- Candidates:
   all three stable releases,
   activity window 2025-08-14 through 2026-08-14
- Gate:
   maintenance and release confidence
- Status:
   scored concern,
   not a hard gate
- Primary evidence:
   GitHub issue,
  pull-request,
  review,
  event,
  commit,
  release,
  and check-run APIs,
   accessed 2026-08-14
- Jackspeak:
   eleven commits,
  all maintainer-authored during 2026-02-05 to 2026-02-07;
  two sampled issues received one maintainer comment total;
  two sampled pull requests received three maintainer comments and neither merged
- Type-flag:
   fifty-four commits,
  thirty-two maintainer-authored,
  twenty-one Renovate-authored,
  and one externally authored;
  sampled issues received three maintainer comments during the window plus closure and release-label actions;
  eight of ten sampled pull requests were maintainer-authored and self-merged in 1,259 to 217,650 seconds with no review
- Argue:
   twenty-two commits,
  twenty maintainer-authored and two automation-authored;
  the only sampled issue was Renovate's dashboard;
  the ten most recently updated pull requests were all open Renovate updates with no maintainer comment,
  review,
  or merge
- Release context:
   Jackspeak published 4.2.3 in February 2026;
  type-flag published multiple stable releases through 4.5.0 and continued 5.0 prereleases;
  Argue published 3.0.0 and 3.1.0 in July 2026 after its previous 2022 release
- Security evidence:
   exact GitHub Advisory Database queries returned no advisory affecting any exact package version;
  no candidate repository publishes a security policy
- Outcome:
   type-flag has the strongest current release activity,
  but every candidate has single-maintainer concentration

## Execution manifests

### Published-artifact semantic matrix

Candidates and pinned artifacts:

- `jackspeak@4.2.3`,
   measured tarball SHA-512 from record J2;
- `type-flag@4.5.0`,
   measured tarball SHA-512 from record T2;
- `argue-cli@3.1.0`,
   measured tarball SHA-512 from record A2.

Top-level command:

```text
podman run \
  --memory=2g \
  --cpus=2 \
  --pids-limit=128 \
  --ulimit nofile=1024:1024 \
  --rm \
  --network none \
  --read-only \
  ... \
  node /probe/parser-probe.mjs
```

Reachable candidate command tree:

- import Jackspeak's documented self-contained ESM `min` artifact and call `jack`,
  field-definition methods,
  and `parseRaw`;
- import type-flag's ESM artifact and call `typeFlag` with a copied explicit argv;
- import Argue's ESM artifact and call `setArgs`,
  custom or built-in option readers,
  `readOptions`,
  and `rest`.

Static source inspection found no candidate filesystem,
network,
subprocess,
plugin,
native,
Wasm,
or downloaded-code path.
Jackspeak's min artifact statically bundles the inspected cliui source.
The harness imports no lifecycle,
build,
test,
or release command.

Expected reads:
read-only Node image,
read-only extracted package artifacts,
and read-only probe.

Expected writes:
bounded anonymous container state only,
with a 64 MiB `/tmp` tmpfs.

Subprocesses:
Podman runtime and one Node process.
Candidate code spawns none.

Network:
disabled.

Image:
local `docker.io/library/node:24-slim`,
Node 24 on Linux x64,
image ID `2f35c3d18013b7d65e31c40f0602e4c0a65a18efc65c16e2b98497f13f4da921`,
digest `sha256:d45d78e7929b46875bbd4e29bea672d5bc48186c6c3588306521c815e78352d6`.

Credentials and environment:
no home mount,
no repository mount,
no ambient credential environment,
and no network.
Jackspeak uses `parseRaw` and no `envPrefix`,
so its optional environment read-write path is not reached.

Cases:
positive control for an ordinary flag and string value;
repeated flags and values;
option after positional;
lone dash;
`--` termination;
joined equals value;
exact short-versus-long prefix distinction;
dash-led declared value;
missing declared value;
unknown plain option followed by a plain token;
unknown joined option;
and exact leftover or error evidence.

Success condition:
process exits zero,
every catalog case yields a captured result or captured error,
and positive controls distinguish parsed values from positionals.
Semantic parity is an observed result,
not a command success condition.

Failure condition:
import failure,
uncaptured throw,
missing case,
unexpected effect,
resource breach,
or nonzero process exit.

Stop condition:
any undeclared read,
write,
subprocess,
network,
native,
Wasm,
or generated-code boundary requires manifest revision before continuing.

No candidate package was installed.
The recorded published artifacts ran only through this manifest.
No undeclared effect or command boundary appeared.

## Hard-gate outcomes

### jackspeak

Outcome:
 fail.

The published parser throws on both `-q path` and `--unknown=value path`.
Those are ordinary partial-schema cases for cli-git because real Git accepts many options
not declared by a particular policy parser.
Declaring the complete Git grammar would abandon the current subset design,
while rescanning raw argv would retain the parser responsibility the dependency was meant to replace.

Jackspeak passes the tested positive,
repetition,
option-after-positional,
lone-dash,
terminator,
joined-value,
dash-led-value,
and missing-value cases.
Its failure is specifically the required partial-Git unknown-option contract.

### type-flag

Outcome:
 fail.

The published parser reads `-m -a` as message `''` plus flag `-a`,
rather than message value `-a`.
It also accepts terminal `-m` as message `''` rather than exposing a missing required value.
For `-q path`,
it returns unknown flag `q` separately from positional `path` instead of retaining the current tentative unknown-option
pair.
The result normalizes unknown spelling and order.

A caller can clone the supplied array to contain type-flag's documented mutation,
but restoring dash-led values and distinguishing missing from explicit empty input
requires reading declared option arity before type-flag does.
The package's `getFlag` API uses the same `argvIterator`,
so extracting one option at a time does not change these results.
A fixed-point pass that first discovers unknowns and then redeclares them
would add repeated reinterpretation rather than remove parser ownership.

Type-flag also expands `-all` as short group `-a -l -l`,
setting the declared `all` flag even though cli-git's generic exact-name parser treats the complete token as unknown.
Type-flag passes the tested positive,
repetition,
option-after-positional,
lone-dash,
terminator,
joined-value,
and unknown-joined recognition cases.

### argue-cli

Outcome:
 fail.

The published parser continues parsing after `--`:
`['--', '-a']` produces count one and leaves only the separator.
It retains unknown options and positionals together in mutable global leftovers,
without enough role information to produce cli-git's `unknownOptions` and `positionals` collections.
Its built-in readers also overwrite repeated scalar results;
the probe used public custom readers to prove counts and ordered values are expressible.

Argue strips one or two leading dashes before a reader sees an option.
The probe therefore accepted both undeclared `--a` and undeclared `-all` as declared aliases for `-a` and `--all`.
A custom reader cannot restore that distinction because the raw prefix is already gone.

Splitting at `--` can repair termination.
A linear leftover classifier can implement cli-git's current unknown-consumption heuristic without an arity table.
Exact prefix handling still requires inspecting the original token stream before Argue normalizes it.
That pre-scan,
the leftover classifier,
and custom count or collection readers leave cli-git owning the distinctive token-role logic.
Argue therefore comes closest to parity but fails the frozen requirement that the dependency replace that logic rather
than surround it with another handwritten parse layer.

Argue passes the tested positive,
repetition with custom readers,
option-after-positional,
lone-dash,
joined-value,
dash-led-value,
missing-value,
and exact unknown-leftover cases.
Its process-global mutable argv is an additional integration concern,
not the hard-gate reason.

## Validation results

### Published-artifact semantic matrix

Command:
the Podman invocation recorded in the execution manifest.

Environment:
Node 24.18.0 on Linux x64,
network disabled,
read-only root and candidate mounts,
2 GiB memory,
2 CPUs,
128-process limit,
and 1,024-file-descriptor limit.

Exit:
zero.

Positive controls:
all candidates parsed one ordinary boolean flag,
one string option,
and one positional;
all also preserved repeated known values when configured through their available APIs.
The controls prove the harness could distinguish values,
counts,
and positionals before the divergent cases.

Evidence:

- probe:
   `~/temp/agent/cli-git-parser-candidate-probe-2026-08-14.mjs`;
- probe SHA-256:
   `553f70d7a13202a6506e9d3b767a94d164c805368f49ca4e70e9a533ccbab89b`;
- output:
   `~/temp/agent/cli-git-parser-candidate-probe-output-2026-08-14.json`;
- output SHA-256:
   `64c1c61440ff1490beb6bd367b1400542e44f1a899a5b28cce97a1cc5a4c7889`.

The exact observed failures are recorded under each hard-gate outcome.
They confirm the source traces rather than relying on API documentation.

### Omitted finalist execution

No candidate became a finalist.
Upstream installs,
builds,
default test commands,
macOS and Windows consumer probes,
cli-git integration worktrees,
and performance benchmarks were therefore not run.
Those operations cannot repair the observed stable-artifact semantic failures,
and the governing workflow excludes hard-failed candidates before finalist execution.

Repository check-run evidence remains relevant only as screening corroboration:
Jackspeak's release commit passed its Linux,
macOS,
and Windows matrix;
type-flag and Argue passed their Linux release-commit checks.
No candidate timing is claimed.

The durable behavior diagnosis,
minimal reproduction,
workaround record,
and upstream-filing checks are in
`doc/troubleshooting/npm-cli-parsers-partial-git-argv.md`.

## Management-only applicability

The closed management grammar is a narrower role than forwarded Git parsing.
None of the following narrower uses replaces the shared parser:

- Jackspeak can parse strict declared management options and correctly reject missing values,
  but cli-git still owns command dispatch,
  authored help,
  action shaping,
  and every Git-region parse.
- Type-flag retains the same dash-led and missing-value behavior for management value options.
  A policy ID such as `--all` after `--policy` has the same role conflict as the tested `-m -a` case.
- Argue can express counts and ordered policy values through custom readers,
  but management use still needs explicit `--` splitting,
  exact one-dash versus two-dash validation,
  leftover validation,
  and process-global state settlement.

A management-only dependency therefore leaves the requested handwritten shared parser in place
and adds another parse model.
No management-only shape advances to recommendation.

## Score arithmetic and sensitivity

Not applicable.
All three candidates failed at least one hard constraint before finalist validation.
The governing method does not assign soft points to offset a hard failure.

Sensitivity is also not applicable.
Changing equal-default weights cannot make unknown-option rejection,
dash-led value loss,
prefix loss,
or missing terminator state satisfy the frozen parser contract.

## Pros and cons

### argue-cli

Pros:

- smallest measured production source in the named set at 298 code lines across seven files;
- no runtime dependency;
- arbitrary aliases and custom readers can express occurrence counts and ordered known values;
- correctly consumes tested dash-led declared values and fails on tested missing values;
- leaves unknown input lexemes intact and in order.

Cons:

- one process-global mutable argv array rather than an argument-local pure parser;
- does not terminate option parsing at `--`;
- erases one-dash versus two-dash prefix information before custom readers run;
- mixes unknown options and positionals in one leftover stream;
- requires handwritten pre-processing,
  leftover classification,
  and custom readers to recover the incumbent result;
- no SLSA attestation or registry `gitHead`,
  and upstream CI at the release commit covers Linux only.

### type-flag

Pros:

- explicit argv input with an argument-local result;
- no runtime dependency;
- returns known flags,
  unknown flags,
  positionals,
  and post-`--` positionals separately;
- preserves repeated known values and supports options after positionals;
- SLSA provenance binds the exact npm artifact to the source commit;
- strongest measured current release activity among the named packages.

Cons:

- mutates the supplied argv array unless the caller passes a copy;
- reads arbitrary dash-led declared values as new flags;
- turns missing string values into empty strings;
- expands every short token as an alias group even where cli-git requires exact generic matching;
- normalizes unknown option names and separates them from following plain tokens,
  losing the incumbent ordering and tentative-consumption facts;
- one-character alias model cannot directly group every exact long abbreviation under one result key;
- upstream release checks cover Linux only.

### jackspeak

Pros:

- correctly handles the tested known flags,
  repeated values,
  option terminator,
  options after positionals,
  dash-led declared values,
  and missing-value errors;
- typed config and help generation are available for conventional closed CLIs;
- release commit passed Linux,
  macOS,
  and Windows CI on Node 20 and 22;
- source-to-artifact mapping is present in the published source map.

Cons:

- unconditionally rejects undeclared plain and joined options,
  which is a direct category mismatch for cli-git's partial Git grammar;
- catching the error yields no successful token facts;
- complete audited runtime source includes 1,066 Jackspeak code lines plus 1,014 cliui code lines;
- adds one same-author runtime dependency;
- arbitrary long aliases cannot share one config field;
- publication has no SLSA attestation.

## Ranking and recommendation

Practical proximity ranking:

1.  `argue-cli`
2.  `type-flag`
3.  `jackspeak`

Ranking:
`argue-cli` > `type-flag` > `jackspeak`.

Argue ranks ahead of type-flag because it preserves dash-led values,
missing-value failure,
and exact unknown leftovers,
while type-flag loses those declared-value roles before an adapter can recover them.
Argue still fails exact prefix and termination requirements and is not recommendable.

Type-flag ranks ahead of Jackspeak because it accepts partial schemas and returns unknown-flag facts,
whereas Jackspeak terminates every parse on an undeclared option.
Type-flag still requires an owned arity scan to repair dash-led and missing values and is not recommendable.

Recommendation:
none of the three named alternatives should replace cli-git's handwritten parser.
Retain the repository-owned parser.
It is narrower than Jackspeak's audited runtime graph,
already models the required partial Git grammar,
and avoids adapters that reintroduce the same token-role decisions around another API.

This is a recommendation,
not adoption authority.
No dependency,
product code,
configuration,
lockfile,
or decision record changed.

## Confidence and evidence limits

Confidence is high for the terminal result because source traces and the actual stable npm artifacts
agree on every hard-gate failure.
The positive controls show that the harness distinguishes ordinary successful parsing from the failing shapes.

The bounded behavior matrix ran on Linux x64 and Node 24.18.0.
Platform-specific finalist runs were not warranted after platform-independent semantic exits.
Upstream check runs provide corroborating platform evidence,
not consumer parity.

Only current stable versions were candidates.
Type-flag 5.0.0 beta was not rated;
its negative-number change does not cover arbitrary dash-led strings,
missing values,
or exact unknown-token order.

No candidate performance comparison is reported.
Each candidate exited before a semantically comparable integration existed,
so a timing result would compare different behavior.

Argue's non-recommendation depends materially on the frozen replacement requirement.
Under the tested versions and constraints,
it cannot replace the handwritten parser.
If the goal changes to delegating only known-option removal while intentionally retaining handwritten prefix,
terminator,
and leftover classification logic,
Argue is the closest named package.
A later Argue release that exposes raw prefixes and explicit termination would require a fresh evaluation.
