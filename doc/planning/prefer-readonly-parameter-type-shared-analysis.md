# Proposal: share one file's effect analysis across TypeScript projects

## Purpose

The `prefer-readonly-parameter-type` lint plugin analyses every source file once per TypeScript project that reaches it.
After the translation-repair split (`doc/planning/translation-repair-package-split.md`),
each of 36 packages is its own project,
and a package's project reaches the source of every family package it imports,
so a fresh worktree lints the family in about 311 s against 72.6 s for the pre-split package.
Every gate runs in a fresh worktree.
Raising the cache bounds (owner's choice of 2026-10-18) fixes warm lints only.

The owner asked on 2026-10-18 for an investigation of whether projects can share the analysis,
with a proposal before any plugin change.
This document is that proposal.
Nothing here is decided.

## What was measured

All measurements are in the keyless scratch worktree `w165` at `8d09e8e092`,
with the cache bounds raised (uncommitted) so that no entry was evicted,
after linting every family package once;
scripts and outputs are under `/var/home/user/temp/agent/tr-fan/s245/split/lint/`.

- The cache held 8269 entries.
  992 source files were cached under more than one project,
  up to 34 projects each,
  in 7666 entries (`cache-surfaces.txt`).
  So the family's cold lint computes 1595 distinct analyses 8269 times.
- For all 992 files,
  the analysis result stored in the entry (`payload`)
  and the deliberately omitted callables were the same in every project.
- For all 992 files,
  the dependency closure (`dependencyDigests`,
  compared as sorted entries)
  was the same in every project,
  as were the augmentation surface and the lockfile digest,
  and no closure was unresolved.
- What differed for all 992 files are three whole-program inputs:
  the project's file list (`fileListDigest`),
  the digest of every declaration file in the program (`declarationSurfaceDigest`),
  and the compiler options (`compilerOptionsDigest`).
- The resolved options of `corpus-pass` and `probe` (`tsc --showConfig`) differ only in `paths`,
  whose `@/*` and `@_/*` aliases point into the linting package;
  no family source names either alias
  (searched with a pattern that matches one file elsewhere in `package/`).

Each entry is addressed by its project key (`effect-summary-persistent-cache.ts`,
`cacheIdentity`)
and validated against all five surface digests (`effect-cache-envelope.ts`,
`surfacesMatch`),
so no entry written under one project is ever read under another.
The TSDoc of `projectSourceText` in `effect-project-fingerprint.ts` says entries
"are keyed by path and shared across projects";
the code does not do that,
and the comment should be corrected whatever is decided here.

## Why the three inputs are in the key today

- The file list binds project membership:
  a file joining the program can change what an import resolves to.
- The declaration surface binds every `.d.ts` file in the program,
  because the closure walk "stays inside indexed sources:
  declaration and external files are excluded because whole-scope surface digests already bind them"
  (`effect-dependency-closure.ts`,
  `createDependencyClosureResolver`).
- The compiler options bind checking semantics
  (`strict`,
  `exactOptionalPropertyTypes`,
  `lib`,
  `target`)
  and module resolution (`paths`,
  `baseUrl`,
  `rootDirs`,
  `moduleResolution`).

All three are whole-program because the closure records only workspace sources.
Two projects whose programs differ anywhere,
even in files a given source can never reach,
cannot share that source's entry.

## Proposed design

Key and validate an entry by what can change its analysis,
so that two projects that agree on all of it share one entry:

1.  Extend the closure to record every file a module reference resolves to,
    declaration and external files included,
    with each file's content digest.
    A changed resolution then changes the closure,
    so the file list and the resolution-only options
    (`paths`,
    `baseUrl`,
    `rootDirs`)
    no longer need to be in the key.
2.  Keep a narrow ambient surface in the key:
    declaration files that apply without an import
    (global scripts,
    the `types` entries,
    `lib` files)
    and the augmentation surface the key already has.
3.  Keep in the key the compiler options that change checking,
    named by an explicit list,
    and refuse to share (keep today's per-project key) when an option outside both lists is set,
    so a new TypeScript option cannot be shared by accident.
4.  Address entries by file and key digest instead of by project,
    so the second package to reach a file reads the first one's entry within the same cold run.

## Risks and what would prove it safe

- Soundness rests on step 1 seeing every resolution the checker made,
  including type-only imports,
  `import()` types,
  triple-slash references and the declaration files an external package's types pull in.
  A file whose closure cannot be proven complete must fall back to today's per-project key,
  as unresolved closures already fall back to the whole scope.
- Before relying on it,
  a verification mode should analyse each shared hit afresh and compare,
  over the whole family and over the rest of the repository,
  and report every mismatch;
  the measured agreement of 992 of 992 payloads is evidence for the family as it stands,
  not proof for other projects.
- The plugin's existing cache tests (`effect-summary-invalidation.unit.test.ts` and siblings)
  must keep passing,
  plus new cases where two projects differ only in an unreachable file (share),
  in a reachable declaration file (do not share),
  in a checking option (do not share)
  and in `paths` that change one resolution (do not share).

## What it would buy

Measured only as counts:
the family's cold lint would compute 1595 analyses instead of 8269.
How much wall time that saves is not measured;
building each package's program and walking it remains per project.
A prototype in a scratch worktree would measure it before any merge.

## Owner's answer

On 2026-10-18 the owner chose to prototype this design in a scratch worktree,
with the verification mode,
and to see the measured cold time and mismatch count before any change lands.

Shown the results recorded in "Prototype results",
the owner chose on 2026-10-18 to harden the prototype and then land it.
The hardening owed before landing:
bump `EFFECT_CACHE_SCHEMA`;
correct the `projectSourceText` TSDoc;
show each fallback's test failing with its guard removed;
add fixtures for `export ... from`,
`import =` and an added `/// <reference types>` global;
and compare lint output over every repository package against the baseline bundle.
The hardening is done on `w177`'s detached head (`6cab24de4c` to `6899ef69af`,
notes `w177-notes/harden/`):
the schema is 9;
the TSDoc says each project pays the fingerprint once and shares only the source analysis;
every fallback's test fails with its guard removed (`guards.md`),
including a newly found third layer (whole-scope snapshots) that had masked the unresolved-specifier case;
the `export ... from`,
`import =` and `/// <reference types>` fixtures do not share when their targets differ;
`readonly-declaration-ownership.ts` now names a workspace path `workspace` whatever the library flag says;
the README documents the sharing.
A cold family lint with verification on gave the prototype's exact totals:
6856 hits verified,
0 mismatches,
110 unresolved-specifier fallbacks.
The comparison over every repository package against the baseline bundle (`w177-notes/fullrepo/`)
gave identical exit codes,
summary lines and sorted diagnostic headers in all 199 packages with a `lint:oxlint` task,
cold and warm (41 skipped,
reasons in `skipped.txt`);
109 packages have diagnostics,
4 have this rule's (10 diagnostics).
It left three items,
closed before landing:
`readonly-type-origin-location.ts` (`workspaceOrigin`) still reads the raw library flag,
so a workspace declaration reached through a `node_modules` symlink gets the guidance of an external one,
which the ownership fix no longer masks;
75 packages resolve none of their own imports in the scratch worktree (TS2307),
so their programs were exercised only weakly;
and `watch-restart/cli` logs an omission warning cold that its warm run drops.
All three are closed (`83e43a4604` to `f931308964`):
`workspaceOrigin` decides by path alone;
building the 72 packages the 75 import cleared every TS2307,
and those 75 lint identically under both bundles;
a consumer's entry now persists the omissions of the installed implementations it was built from
(cache schema 10),
so a warm run reports them as a cold one does.
Under that bundle all 199 packages again gave identical exit codes,
summary lines and diagnostic headers,
cold and warm,
and every differing body line is a paired "omitted" and "restored" report of the same file,
count and reasons,
or a Go panic address (`w177-notes/fullrepo/compare-base199-vs-h2*.txt`).
The 22 commits landed on 2026-10-18 as `1008a5999f` to `62ba3e10a6`,
with the plugin tree identical to the verified one.
Profiling the remaining cold time runs in `w177-notes/profile/`.
The owner also chose to profile the remaining cold time
(about 131 s against 11.5 s without the plugin)
after landing,
not before.

## Prototype results

The prototype is on the detached head of the keyless scratch worktree `w177`
(`0f2f380e59` to `1b0cdd8653`,
on top of `da4c8dcab9`,
unmerged);
scripts and per-package logs are under `/var/home/user/temp/agent/tr-fan/w177-notes/`.
I re-totalled the timings and lint results from its `out/<round>/results.json` files
(`s245/split/lint/proto-totals.ts`)
and the verification counters from its per-process files (`w177-notes/sum-verify.ts`).

### What it builds

- The key is a digest of the analyzer digest,
  file name,
  source digest,
  the closure of every file reached through module references
  (walked through declaration and external files,
  each with its content digest),
  an ambient surface,
  the augmentation surface,
  the lockfile digest,
  the compiler options by value,
  and the analysis root.
- The ambient surface holds every program file that applies without an import:
  scripts (including `lib` files),
  files with module or global augmentations or ambient module names,
  and declaration files with `export as namespace`.
- Left out of the key:
  the project's file list,
  its unreachable declaration files,
  and the resolution-only options
  (`paths`,
  `baseUrl`,
  `rootDirs`,
  `rootDir`,
  `outDir`,
  `declarationDir`,
  `tsBuildInfoFile`,
  `typeRoots`,
  `mapRoot`,
  `sourceRoot`,
  `project`,
  `configFilePath`,
  `pathsBasePath`).
- An option on neither list sends the whole project back to per-project entries.
  So does a source that is a global script,
  reaches a `.tsx`,
  `.jsx`,
  `.astro`,
  `.mdx`,
  `.vue` or `.svelte` file (implicit JSX import),
  names a specifier that does not resolve,
  or has an unresolved existing closure.
- Shared entries live at `shared/<hash(file)>/<keyDigest>.json` under the cache root;
  a read tries the shared entry first,
  then the per-project one.
- `PRPT_VERIFY_SHARED=1` analyses every shared hit afresh and compares callable summaries,
  omitted callables and omission reasons.

### Family lint times

Each round linted all 36 family packages in a fresh `systemd-run` unit,
cold (cache emptied) then warm.

- Baseline (`da4c8dcab9`):
  cold 325.6,
  323.9 and 318.2 s;
  warm 71.0,
  74.0 and 70.8 s.
- Prototype (`ce6354e62f`,
  verification off):
  cold 130.5 and 131.9 s;
  warm 73.3 and 73.2 s.
  Warm is inside the baseline's spread.
- Prototype with verification on:
  cold 410.5 s and more,
  so the mode is a probe,
  not a gate setting.
- The slowest packages cold went from 30 to 33 s to 15 s (`corpus-pass`),
  24 s to 7 s (`probe`) and 21 s to 10 s (`bench`).

The family computed 1611 analyses instead of 8467.
The remaining 131 s cold is not attributed;
without the plugin the family lints in 11.5 s,
so per-project program load and walks are the next thing to measure.

### Outside the family

48 other packages
(Astro,
DOM,
CLI,
config,
plugin and harness packages;
lists in `w177-notes/extra-list.txt` and `group2-list.txt`)
showed no speedup claim:
cold 56.2 against 57.7 s and 124.2 against 111.6 s in single paired runs.
Sharing there is thin:
166 shared hits,
123 across projects.
`pi-plugin/auto-mode` takes about 356 s cold and 374 s warm in both baseline and prototype,
so the cache does not help that package at all;
this is not investigated.

### Equality

- Verification over the family,
  cold:
  8467 keys,
  1611 shared writes,
  6856 shared hits (6628 across projects),
  6856 verified,
  0 mismatches,
  0 invalid entries;
  110 fallbacks,
  all unresolved specifiers
  (`node:*` names in `nano-spawn`'s JavaScript,
  one `undici-types` path).
- Outside the family:
  166 hits verified,
  0 mismatches,
  241 unresolved-specifier fallbacks.
- Positive control for the verification mode (`w177-notes/control/`):
  `shared-contract` linted on an empty cache,
  then one summary in its shared entry for `src/chunk-placement.ts` was changed
  (`mutated` from `[]` to `[0]`,
  envelope and digests untouched),
  then `artifact` linted with verification on:
  38 shared hits,
  1 mismatch naming that summary.
  With the entry restored and the cache emptied,
  the same two lints gave 38 hits and 0 mismatches.
  Only a payload change was tried,
  not omitted callables or omission reasons.
- Lint output (exit code,
  summary line,
  sorted diagnostic headers) was identical to baseline in every package run:
  144 family runs per comparison,
  all with no diagnostics,
  and 96 outside-family runs,
  74 of them with diagnostics and 6 with this rule's (10 diagnostics,
  cold and warm).
  This is thin evidence for the rule's output where sharing is heavy:
  the family has no diagnostics to lose.
- The plugin's unit suite passes (27 suites),
  including the existing invalidation suite
  and 11 new sharing cases.

### Not proven

- The non-literal `import()` type case is the only fallback shown to fail with its guard removed.
- No fixture covers `export ... from` or `import =` specifiers,
  a `/// <reference types>` global added to one project,
  or the implicit JSX import refusal (never hit).
- The option lists beyond TypeScript's option interface rest on one probe.
- Program file order and call edges that are not module references are not in the key;
  the existing workspace-only dependency check is the only guard for the latter.
- Verification compares only hits;
  it cannot see a source that should have shared and did not.
- At the prototype:
  `EFFECT_CACHE_SCHEMA` was not bumped,
  `readonly-declaration-ownership.ts` classified a symlink-reached workspace file as external,
  and the `projectSourceText` TSDoc was wrong.
  The hardening closed all three before landing (landed as `a6a42f8416`,
  `b88c34eec0`,
  `c501702670`).

The prototype agent's summary said "6751 cross-project shared hits";
its own counter files total 6628,
and its detailed section said so.

## Cold time after landing

Profiled on 2026-10-18 at `f931308964` with the landed bundle,
on a shared host (load 5 to 47 on 28 cores);
the profiles,
logs and scripts are in `w177-notes/profile/`.

### Spread

Three sequential cold family lints took 151.2,
155.1 and 161.7 s,
and three warm ones 96.9,
82.9 and 84.5 s,
so a difference under about 10 s cold or 14 s warm is noise.
The prototype's 131 s was measured on a quieter host;
how much of the gap is load was not separated.
With no JavaScript plugin the family lints in about 20 s,
with the other four JavaScript plugins in about 40 s,
so this rule's own work is about 110 to 120 s of the cold time.

### Where the time goes

- Each package runs its own oxlint process,
  which starts two `tsc --api` children and one `tsgolint`;
  every program is built from scratch,
  and a family source such as `shared-contract/src/index.ts`
  is read once per engine per package (96 reads in one family lint).
- 2.86 million requests to the TypeScript children cost 73.8 s as seen by Node:
  30.0 s of server work and about 49 s of pipe round trip,
  wake-up and JSON codec.
- 67.1 percent of those requests repeat a request already made on the same immutable snapshot
  (`resolveName` 86.6 percent repeats,
  `getSymbolAtLocation` 65.6 percent),
  35.9 s of the 73.4 s of request time.
- Project fingerprint,
  program open and index scope together cost 21.6 s cold;
  the shared-key walk 4.8 s;
  cache reads and writes 3.7 s;
  plugin load 1.8 s;
  garbage collection 12.3 s;
  per-process start and tail about 30 s across the 36 processes.

### Levers, ranked

1. Memoize TypeScript requests per snapshot:
   at most 35.9 s,
   low risk because a snapshot is immutable,
   provided the memo is dropped with the snapshot and bounded.
2. One oxlint invocation for the family:
   15 to 55 s on a sequential run,
   depending on load,
   but the repository already lints packages in parallel,
   which hides most per-process cost,
   and it costs 5.5 to 6.8 GB of peak memory and merges 36 reports into one.
3. Cheaper project setup:
   at most about 6.6 s of fingerprint transport plus part of index scope,
   medium risk because disk text can disagree with the analysed snapshot.
4. Skip files the rule cannot report on:
   at most 2.8 to 15.2 s,
   and it changes cache population order.
5. Share one program across packages:
   at most about 2 s.

The first lever landed on 2026-10-19 (UTC) as `a47fa17b02` and the three commits before it;
the second is recorded and not built,
because parallel linting already takes most of its gain.

### The first lever after landing

The memo wraps each TypeScript API instance and answers ten read-only requests from memory
(`resolveName`,
`getSymbolAtLocation`,
`getTypeAtLocation`,
`getResolvedSignature`,
`getSignaturesOfType`,
`getTypeOfSymbolAtLocation`,
`getPropertiesOfType`,
`getShorthandAssignmentValueSymbol`,
`getTypesOfType`,
`getTypeArguments`),
keyed by snapshot id,
then by method and parameters.
Each was chosen by reading its handler in the native server's `internal/api/session.go`;
a request a later TypeScript release adds goes to the server every time until its handler is read.
A snapshot's store is dropped when the snapshot is released,
failures are never stored,
and a store holds at most 131,072 answers,
oldest out first:
replaying the recorded request streams of the 36 family processes,
only `corpus-pass` (136,050 at its peak) evicts at that bound,
and the hit rate stays at 70.99 percent.

- Requests in one cold family lint fell from 2,879,340 to 975,205,
  and Node-side request time from 72.8 to 45.3 s.
  What remains is mostly requests left out on purpose (`getSourceFile`,
  `updateSnapshot`,
  `getSymbolsAtLocations`).
- Over ten alternating cold passes,
  the per-package medians sum to 191.5 s before and 159.2 s after,
  a saving of 32.3 s (17 percent),
  which agrees with the request time removed.
  The pass totals alone spread by about 50 s within each variant on this host and cannot show the saving.
- Peak memory per package unit is unchanged within noise (median change of the maxima minus 9 MiB).
- All 199 packages give the same exit code,
  summary line and diagnostic headers cold and warm.
  The one package whose full output differs,
  `translation-repair/consistency`,
  prints the same error with its
  stack window shifted by the memo's wrapper frame.
- Gate g107 at `abab8e541a`:
  the plugin's build,
  type check,
  lint,
  unit tests and three host tests,
  the config bundle rebuilt and carrying the memo,
  and the translation-repair family gate (145 steps),
  all exit 0.

### The spread rule when the bridge cannot start

The spread rule fell back without a word when the bridge could not start:
it dropped `Array.from` from its report and called slices and spreads ambiguous.
In production the readonly plugin starts the bridge at load,
so this needs a configuration without that plugin
or a host where the late spawn fails.
The first fix,
which stops the lint and names the cause,
was held:
the fixture runner hid a plugin's throw,
so two valid-fixture cases passed only because the plugin threw,
and one ambiguous-spread case newly failed.

The redo landed on 2026-10-19 (UTC),
ending at `249c3d2d0b`.
The cause of the seven spread-rule cases that failed on a 28-thread host is the fork itself:
before the first file needs a type,
oxlint has reserved about 192 GiB of address space for its workers,
and a probe forks at 132 GiB reserved and fails at 144 GiB on this host (62 GiB of memory and 79 GiB of swap),
so the parent's `clone` returns `ENOMEM` whatever the child's size.
Pinning `--threads` would test a setting that bare `oxlint` and editors do not use,
and Node's `spawn` cannot fork more cheaply,
so the plugin now starts its bridge when it loads,
before the workers exist.
A failed start is kept:
the other rules keep working,
and the first spread rule that needs a type throws the stored error and names the way out.
`runOxlintFixture` now fails a run when oxlint reports a diagnostic with no rule code,
which is how a plugin's throw shows.
In the production bundle both plugins share one bridge,
so the extra start is a no-op there;
the standalone `no-restricted-syntax` build carries its own copy,
which is where the failures came from.
Gate g108 ran the six plugin packages that use the fixture runner,
the config bundle and the translation-repair family gate,
and every step exited 0 except `lint:oxlint` of `tsdoc` (14 errors) and `stylistic` (1 error,
2 warnings);
on the branch without the redo `tsdoc` reports the same 14 errors,
and `stylistic` the same three findings plus type-aware warnings from dependencies that worktree had not built,
so they are existing findings of those packages,
not of this change.
Fix-mode fixture runs that read the file back without looking at diagnostics still pass through a crash;
that gap is being closed separately.

The readonly rule's own catch,
which warns `semantic evidence unavailable` and skips the file,
stays:
`doc/troubleshooting/typescript-go-tuple-type-panic.md` records it as the handling of an upstream panic.

### Coupling found on the way

`no-restricted-syntax/src/rule/spread-evidence/expression-kind.ts` imports the semantic bridge.
Without this plugin loaded,
the bridge starts lazily mid-lint,
and on `corpus-pass` it failed with `spawn ENOMEM`;
what that rule loses then is in "The spread rule when the bridge cannot start".

### Remaining repeated requests, read on 2026-10-19

Issue 1780 asked for the requests the memo still sends to be read and,
where the rule allows,
added.
It estimated about 130,000 repeats,
and named `getSymbolsAtLocations`,
`getIndexInfosOfType`,
`isArrayType`,
`getSymbolOfType`
and `getAliasedSymbol`.
This section records what was measured,
what was read,
and what changed.
The scripts,
logs and per-process files are in `/var/home/user/temp/agent/w209-notes/`.

#### Source read

- The installed `typescript@7.0.2` records `gitHead` `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`,
  and `pnpm-lock.yaml` pins 7.0.2 (the catalog entry is `>=7.0.2`).
- That commit exists in `microsoft/typescript-go`,
  as "Merge branch 'main' into ts7-release"
  (author date 2026-07-08 04:22:40 UTC),
  and not in `microsoft/TypeScript`,
  whose `v7.0.2` tag (`1e4744d68260`) is a merge commit with the same subject and date.
- I cloned `microsoft/typescript-go` at that commit and read `internal/api/session.go`
  and the checker functions the handlers call in `internal/checker`.
- The ten memoized requests were read at the same commit,
  so one standard covers all sixteen.

#### How repeats were counted

- The measurement preload of `w177-notes/profile/` and `w190-notes/profile/` (`PRPT_DEDUPE=1`) wraps
  `Client.prototype.apiRequest` of the installed synchronous client,
  keys each request by method plus its JSON parameters (which include the snapshot id),
  and totals requests and distinct keys per process.
- The memo sits on the client instance and forwards to that prototype method,
  so the preload counts only requests that reach the server.
  A repeat it reports is a request the memo did not answer.
- Each run was one cold lint of the 36 `translation-repair` packages in its own capped systemd unit,
  with the cache emptied first.
- The same preload gave 975,205 requests after the memo in the earlier measurement,
  and 976,608 and 976,463 here in two runs,
  so the setup reproduces the earlier numbers.

#### What repeats

- Two baseline runs,
  before this change:
  976,608 requests with 27,475 repeated (2.8 percent),
  and 976,463 with 28,697 repeated.
  Together they use about 36 s of Node-side request time,
  and the repeated requests cost about 0.7 s of it.
- The issue's 130,000 is not reproduced.
  The five methods it names sent 98,283 requests,
  of which 21,985 were repeats,
  and two of them repeated none:
  `getSymbolsAtLocations` (26,437 requests,
  all distinct)
  and `getSymbolOfType` (12,092,
  all distinct).
- Repeats by method in the first baseline run (the second run in parentheses):
  - `getIndexInfosOfType`:
    28,175 requests,
    10,017 repeated (28,732 and 10,574).
  - `isArrayType`:
    24,879 and 6,770 (25,436 and 7,327).
  - `getAliasedSymbol`:
    6,700 and 5,198 (the same).
  - `typeToString`:
    4,338 and 4,266 (4,448 and 4,376).
  - `getBaseConstraintOfType`:
    699 and 531 (the same).
  - `getNonNullableType`:
    1,710 and 402 (the same).
  - `getSourceFileNames`:
    176 and 119 (the same).
  - `isTypeAssignableTo`:
    92 and 64 (the same).
  - `getExportsOfModule`:
    153 and 57 (the same).
  - `getSignatureUsages`:
    97 and 12 (the same).
  - Every other kind repeated 12 times or fewer
    (`getSignaturesOfType` 11 and 12,
    `initialize` 9,
    the rest fewer),
    and the kinds that repeated none are named in the verdicts.
- Repeat counts move a little between identical runs (10,017 against 10,574),
  because the order in which 36 processes share cached analysis is not fixed.

#### Verdicts

Added,
after reading each handler and its checker call:

- `getIndexInfosOfType`:
  `setupChecker`,
  handle lookup,
  `GetIndexInfosOfType` on the persistent API checker,
  and responses made of type handles and a node handle.
- `isArrayType`:
  a pure test of the type's flags and target against the checker's global array types.
  A `false` answer is replayed as an answer.
- `getAliasedSymbol`:
  resolves an alias symbol to its target with the checker's own alias resolution.
  The plugin already checks the alias flag before asking,
  because the checker panics on a symbol that is not an alias.
- `getBaseConstraintOfType`:
  reads the resolved base constraint,
  which the checker stores on the type (`resolvedBaseConstraint`) once computed.
- `getNonNullableType`:
  asks the checker for the type with `null` and `undefined` removed,
  which it builds as a union looked up by its constituents in the checker's union table.
- `getExportsOfModule`:
  resolved exports of a module symbol,
  sorted by the checker's own symbol order.

Read and not added:

- `getSymbolsAtLocations`,
  `getSymbolOfType`,
  `getAliasSymbolOfType`,
  `getParametersOfSignature`,
  `getAliasTypeArgumentsOfType` and `isTupleType` meet the rule,
  but none of their 26,437,
  12,092,
  4,431,
  2,615,
  17 and 3 requests repeated on a snapshot.
  Adding them would store answers (2.5 KB per `getSymbolsAtLocations` answer)
  and save no request.
- `typeToString` returns a rendering and not a handle.
  The printer resolves type members lazily while it prints (a comment in `internal/checker/printer.go` says so),
  and nothing here shows the text independent of earlier queries.
  It repeats 4,266 times for 0.07 s.
- `isTypeAssignableTo` is a relation check against the checker's relation caches,
  and nothing here shows its answer independent of earlier queries.
  It repeats 64 times.
- `getSignatureUsages` is answered by a language service that acquires its own checker
  and asks for references across every source file,
  so it does not follow the persistent API checker the other handlers use.
  It repeats 12 times.
- `getSourceFileNames` hands the client's raw array to the caller,
  so a hit would give every caller the same array,
  and `getDefaultProjectForFile` likewise.
  `getSourceFileNames` repeats 119 times for 0.05 s.

The registration the handlers do is idempotent and keyed by the checker object,
in registries that live until the snapshot is released,
the same as for the ten already memoized.
The synchronous client returns the object already registered for an id and ignores later data for it
(`getOrCreateSymbol`,
`getOrCreateType` and `getOrCreateSignature` in `dist/api/sync/api.js`),
and the client's wrappers for the six read the answer without changing it.

#### Replay check

One more cold lint with the baseline bundle (`PRPT_BYTES=1`) compared every repeated answer
with the first answer to the same request.

- No answer differed in anything the client keeps,
  in any repeat of the six kinds:
  `getIndexInfosOfType` 9,788 repeats,
  `isArrayType` 6,541,
  `getAliasedSymbol` 5,198,
  `getNonNullableType` 402,
  `getBaseConstraintOfType` 531,
  `getExportsOfModule` 57.
- 299 `getIndexInfosOfType` and 18 `getBaseConstraintOfType` answers differed in raw text,
  and each difference was an `objectFlags` value on a type that had the same id,
  which the checker sets lazily.
  One `getTypesOfType` answer of the already memoized kinds differed the same way.
  The client ignores that data for an id it has registered,
  which is why replaying the first answer changes nothing a caller can see.
- The comparison reduces every nested object that carries an id to the id.

#### Size of a stored answer

- Serialized,
  one distinct request key plus its answer is 561 bytes on average over the sixteen kinds
  (823,446 entries,
  440.7 MiB across the 36 processes).
  The six added kinds average 212 bytes (39,341 entries,
  7.95 MiB).
- `corpus-pass`,
  the largest process,
  stored 141,367 entries and 75.1 MiB serialized.
- The heap held by parsed answers was not measured and is larger.
- The bound stays at 131,072.
  Replaying the recorded request stream with the six added,
  `corpus-pass` peaks at 141,220 entries in one store and evicts 10,167 at this bound
  (4,896 with the ten alone,
  on the same stream),
  and no other process evicts.
  The eviction costs 22 of the 2,060 repeats that process could still answer.
- These figures are in the TSDoc of `MAXIMUM_MEMOIZED_RESPONSES_PER_SNAPSHOT`.

#### Before and after

Two cold family lints each,
same flags,
same host,
load average between 4 and 13 when each started.

- Requests that reached the server:
  976,608 and 976,463 before,
  951,964 and 953,148 after.
  That is 24,644 and 23,315 fewer,
  2.4 to 2.5 percent.
- Per added method,
  before in the two baseline runs and after in the two later runs:
  - `getIndexInfosOfType`:
    28,175 and 28,732,
    then 18,160 and 18,160.
  - `isArrayType`:
    24,879 and 25,436,
    then 18,109 and 18,109.
  - `getAliasedSymbol`:
    6,700 and 6,700,
    then 1,502 and 1,502.
  - `getNonNullableType`:
    1,710 and 1,710,
    then 1,308 and 1,308.
  - `getBaseConstraintOfType`:
    699 and 699,
    then 168 and 168.
  - `getExportsOfModule`:
    153 and 153,
    then 96 and 96.
  After each run an added method's count equals its distinct keys (two `getIndexInfosOfType` repeats remain).
- Repeats left in a run:
  4,588 and 4,585,
  of which 4,330 are `typeToString` and 119 `getSourceFileNames`.
- Node-side request time:
  35.76 and 36.02 s before,
  38.81 and 35.72 s after.
  The baseline runs differ by 0.26 s and the later two by 3.09 s,
  so the saving cannot be seen in this measurement.
  The repeats removed cost about 0.35 s in the first baseline run (1 percent),
  well inside that spread.
- Wall time of the 36 lints in sequence:
  111.6 and 110.8 s before,
  121.5 and 112.9 s after;
  the 121.5 s run started at a load average of 12.6.
  The section "Spread" puts noise at about 10 s cold.

The change removes about 2.5 percent of the requests and stores about 8 MiB of serialized answers more,
and its effect on cold time is below this measurement's spread.
What is left of the 950,000 requests is almost all distinct requests,
so another memo kind cannot buy more than the 4,600 repeats listed.

#### Findings are unchanged

- All 36 family packages gave the same exit code,
  summary line and sorted diagnostic headers
  in the first baseline run,
  the second,
  and both later runs.
  The family has no diagnostics in this tree (every package says 0 warnings and 0 errors),
  so this shows the rule did not start reporting,
  and cannot show a finding preserved.
- To cover that,
  four packages outside the family were linted before and after:
  `electron-infra/core`,
  `pi-plugin/morph-compact`,
  `rolldown-plugin/import-attributes`
  and `claude-code-plugin/statusline`.
  They carry 105 diagnostic headers,
  9 of them this rule's,
  and every one is identical after the change,
  as is the whole normalized log text.
- Normalized log text means the output with timestamps,
  Go panic addresses,
  and bundle chunk names with their columns replaced,
  sorted by line.
  Between the first two baseline runs the same normalization shows no difference,
  so it does not hide run-to-run noise that would also hide a change.
- `translation-repair/consistency` prints the same native panic (the tuple type panic recorded in
  `doc/troubleshooting/typescript-go-tuple-type-panic.md`) before and after.

#### Gate

- The plugin's `build`,
  `lint:types`,
  `lint` and `test:unit` exit 0 on the final source:
  34 suite lines passed and none failed,
  0 warnings and 0 errors from `lint`.
- The new cases fail without the change:
  with the six names removed from the allow-list,
  seven stub cases and the native case failed.
- The native case runs the real server on a two-file project,
  asks each added request twice through the checker,
  and sees it reach the server once with equal answers;
  a request left off the list (`typeToString`) reaches it twice.
