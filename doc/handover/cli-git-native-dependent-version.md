# cli-git native dependent-version planner

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`.

Note,
 2026-10-14:
 the retirement of the TypeScript wrapper deleted the driver `bin/dependent-version-differential.mjs`,
 its task `native:differential:dependent-version` and the package `git-policy-repository` that this record names.
The record is `doc/handover/cli-git-native-retire.md`;
the incumbent that this record compares against is the commit `f4fc1fa213ece65672e4314983a75266837892f9`.

## Purpose and how to respond

This records the native Rust planning core of `mono/dependent-version-bump`
and the proof that it plans what the incumbent TypeScript planner plans.
The owner decided on 2026-10-05 that the dependent-version ripple ends with one implementation,
in the native wrapper:
the release task will call `git cli-git fix`,
and the TypeScript planning code under `package/git-policy/repository/src` is deleted at cutover
(`doc/planning/cli-git-rust-open-decisions.md`, section "TypeScript utilities beside the wrapper").
This branch builds the planner and the equivalence evidence,
then wires it into the native executable for `git add`, `git cli-git check` and `git cli-git fix`
(section "Wiring into the native executable").
Section "What the wiring step needed" is the list the wiring was written against.

What to inspect:

- the wiring, the removed refusal and its guard proof,
  and the release-task differential,
  in section "Wiring into the native executable";
- the mutation evidence,
  where two timeouts were caught on their one rerun
  and the one mutant that the first run missed is equivalent after the candidate fix,
  in section "Mutation of the wired code";
- the intentional differences, each with its reason,
  in section "Intentional differences";
- the choices open to veto,
  in section "Choices open to veto";
- the differential results and positive controls,
  in section "Proof of equivalence".

How to respond:
a veto of a choice or of an intentional difference names its heading;
the planner then changes to match the incumbent at that point,
and the differential harness proves it.

## What changed

### Planner modules

All files are in `package/git-policy/cli/src/native/`,
each with a sibling `*_tests.rs`,
registered in `lib.rs`.
Nothing outside them calls them yet.

- `dependent_version_content.rs`:
  the content seam (`WorkspaceContent`, `TrackedPath`, `TrackedMode`)
  and the failure types (`ContentUnavailable`, `PolicyIncomplete`, `PlanError`) with their complete diagnostics.
- `dependent_version_paths.rs`:
  which tracked paths are workspace manifests,
  the registry configuration,
  and a package's non-test source.
- `dependent_version_manifest.rs`:
  strict UTF-8 and strict JSON reading of a manifest's name,
  version and dependency names.
- `dependent_version_release.rs`:
  the patch bump of a plain release and `JSON.stringify` quoting.
- `dependent_version_text.rs`:
  the byte-preserving rewrite of the top-level version.
- `dependent_version_publishable.rs`:
  publishable names from `package/config/pnpr/config.yaml`.
- `dependent_version_imports.rs`:
  the module-specifier scan.
- `dependent_version_graph.rs`:
  the dependency walk and the bump plan.
- `dependent_version_states.rs`:
  every manifest at both states and the raised names.
- `dependent_version_bundled.rs`:
  which development dependencies each dependent bundles.
- `dependent_version_plan.rs`:
  `plan_workspace_bumps`,
  the planner behind both entry points,
  with exact bytes before and after each bump.
- `dependent_version_policy.rs`:
  `find_dependent_bumps`,
  the trigger rule,
  the candidate rule and the findings.

Test-only modules:
`dependent_version_test_support.rs` (an in-memory workspace that records reads),
`dependent_version_fixture_json.rs`,
`dependent_version_fixture_output.rs` and `dependent_version_fixture_cases.rs`
(the native side of the differential harness),
and `dependent_version_differential_tests.rs`.

### Differential harness

The driver is `package/git-policy/cli/bin/dependent-version-differential.mjs` (deleted 2026-10-13, no replacement),
run as `mise run //package/git-policy/cli:native:differential:dependent-version -- <record | check | corpus>`.
Its modules in the same directory, all named `dependent-version-<role>.mjs`:

- `incumbent` and `incumbent-reader`:
  the incumbent side and its fake policy context.
- `unit-scenarios`,
  `unit-functions`,
  `unit-graph`,
  `unit-workspaces`
  and `unit-workspace-parts`:
  the transcribed unit-test scenarios.
- `generator`,
  `workspace`,
  `generator-parts`,
  `manifest-text`,
  `malformed`
  and `random`:
  the seeded generator.
- `repository`,
  `repository-samples`
  and `repository-cases`:
  the real repository.
- `native`:
  the native side,
  in place or in a planted copy.
- `probes` and `report`:
  predictions,
  plants and the per-class comparison.
- `types`:
  the shapes both sides read and write.

Each lints with 0 findings under the package's Oxlint configuration.
The committed shared fixture is
`package/git-policy/cli/src/native/dependent_version_fixtures/unit_cases.json`.

### Fuzz target

`package/git-policy/cli.fuzz` has a target
`dependent_version`,
described in that package's `README.md` under "`dependent_version`".

### Dependencies

No dependency was added.
Manifests are parsed with `monochromatic-jsonc-edit`,
which the crate already links.
No `Cargo.lock` changed.

## The planner

### Content seam

`WorkspaceContent` has three methods:
`tracked_paths` (every tracked path with its mode, in the provider's order),
`candidate_bytes(path)`,
and `base_bytes(path)`,
which answers nothing when the base lacks the path.
The planner selects manifests,
the configuration and source files from the listing itself,
so the provider needs no pathspec support.
It reads every manifest at both states,
the configuration only once a version was raised,
and source files only of dependents whose development edges could carry a raised version.

### Failure kinds

`PlanError::ContentUnavailable` is a read the provider could not answer;
the engine reports it as `content-unavailable`.
`PlanError::PolicyIncomplete` is content the planner read but cannot use,
reported as `policy-incomplete`:
a manifest or the configuration that is not UTF-8,
a manifest that is not JSON,
a manifest without a string name or with a non-string version,
a version the rewrite cannot find as parsed,
or two manifests with one package name.
A dependent whose version is not a plain release is not a failure:
the plan ends with `PlanOutcome::Unsupported`,
which the policy reports as one `dependent-version-unsupported` finding,
as the incumbent does.

### Retained behavior

Every rule the ledger section "Dependent-version propagation" names is kept:
the trigger rule
(a forwarded command other than `commit` plans nothing;
`git cli-git check` and `git cli-git fix` plan whatever the command word),
the candidate rule
(only a modified `package/<category>/<name>/package.json` starts planning),
the raised rule
(a version that differs from the base,
including one removed,
counts as raised;
a manifest new at this state,
or without a version at the base,
raises nothing),
the walk through unpublished packages and cycles,
the stale rule
(publishable,
versioned,
reached and not itself raised),
the bundled-edge rule,
the publishable-name line scan,
the name order of findings,
and the finding codes and messages verbatim.
The incumbent's order of failures is kept:
every manifest is decoded before any is parsed,
and a manifest's base text is parsed before its current text.

## Proof of equivalence

### How the comparison works

Both planners read the same cases and write one canonical JSON text per case:
keys in a fixed order,
no spaces,
strings quoted as `JSON.stringify` quotes them,
and every byte string (paths, original and patched manifest bytes) as lower-case hexadecimal.
Equal texts are equal results.
A workspace case is evaluated twice by each side over the same content:

- the plan,
  through the incumbent's `planWorkspaceBumps` with the release task's reader semantics
  (`bump-dependents-worktree.ts`: lenient UTF-8,
  a byte-order mark kept),
  and through the native `plan_workspace_bumps`;
- the policy,
  through the incumbent's `findDependentBumps` over a fake policy context shaped like the unit tests' `contextOf`,
  whose reader is the policy's own (strict UTF-8, a byte-order mark stripped),
  and through the native `find_dependent_bumps`.

The plan result holds the raised names and,
per bump,
the name,
directory,
both versions,
the manifest path,
and the exact bytes before and after.
The policy result holds every finding's code,
message,
path and patch,
whose before and after bytes are read back from the incumbent's unified diff.
A failure compares by class
(`syntax`, `shape`, `decode`, `unavailable`, `graph`, `other`)
and,
for shape problems,
by the incumbent's `ManifestShapeError` message verbatim;
the parsers' own syntax messages differ by design and are not compared.

The incumbent is the TypeScript source,
imported unchanged through `@monochromatic-dev/git-policy-repository/ts` and run by `node`.

### Inputs

#### Unit-test scenarios

Every scenario of the incumbent's five dependent-version unit-test files is transcribed in
the `unit-*` modules:
61 cases,
from function-level ones (`patchBumpVersion`, `planDependentBumps`, `readManifestDependencyFacts`,
`replaceManifestVersion`, `importsPackage`, `isNonTestSourcePath`, `readPublishableNames`)
to workspace ones (`findDependentBumps` and `bumpWorktreeDependents`).
Each keeps its unit test's own assertion,
which the driver applies to the incumbent's result before recording it,
so a transcription mistake fails before any comparison.
The `record` subcommand writes the results to the committed fixture
`src/native/dependent_version_fixtures/unit_cases.json`;
`check` requires the incumbent to reproduce that file;
the native gate test `dependent_version_differential_tests::unit_fixtures_match_the_incumbent` reads the same file.
Two incumbent tests are not planner scenarios and are not transcribed:
the plugin registration test
(the native trigger table in `policy_trigger.rs` pins the triggers)
and the constructor of `WorktreeBumpConflictError`
(the native release path is `git cli-git fix`,
whose conflict handling is the engine's `patch-conflict`).

#### Real repository

The driver reads this worktree at `HEAD` with `/usr/bin/git ls-tree` and `cat-file`,
read-only and bypassing the wrapper:
every tracked path with its mode,
and the bytes of every workspace manifest,
the registry configuration and every file under a package's `src/`.
It samples packages from the actual graph,
four per category:
leaf packages (no manifest names them),
the most depended-on packages (by runtime fields),
private packages that something depends on,
and packages reached only through bundled development imports whose importer is published.
Each case raises one sample's minor version as a pre-forward commit of that manifest;
one more raises one sample of each category together,
and one more runs that raise as the release workflow's `direct-fix`.

#### Generated workspaces

The `generator` module and its parts,
seeded with Mulberry32.
Each workspace has 2 to 10 packages with random edges of every field kind
(so cycles,
self edges and edges to names without a manifest occur),
workspace protocol variants,
unpublished packages,
names that order differently in UTF-16 and UTF-8,
source files of every extension with every specifier form and decoys
(comments,
plain strings,
prefix look-alikes,
`reimport`,
`myrequire`,
test files,
files that are not source,
bytes that are not UTF-8),
unusual manifest formatting
(indentation of 0, 2 or 4 spaces or tabs,
CRLF,
no final newline,
spaces before the colon,
shuffled keys,
a nested `publishConfig.version`,
`"version"` as a value),
new,
renamed and reformatted manifests,
raised,
removed and already bumped versions,
every lifecycle point,
deleted candidates,
manifests that are executable,
symbolic links or submodules,
and at most one malformed manifest per workspace
(a comment,
a trailing comma,
single quotes,
a missing comma,
truncation,
a scalar or array root,
a numeric name,
a non-string version,
a duplicated version key whose first value differs).

#### Probes

One feature per workspace whose result is meant to differ,
25 workspaces per feature,
each difference checked against its prediction in the `probes` module:
`bom`,
`huge-patch`,
`duplicate-name`,
`unpaired-surrogate-name`,
`deep-nesting`,
`non-utf8-manifest`,
`non-utf8-config`.

### Results

Evidence directory
`package/git-policy/cli/target/verification/dependent-version-DufJ5h`
(`cases.jsonl`, `ts-results.jsonl`, `rust-results.jsonl`, `report.json`),
seed `20261006`,
on the tree of commit `0d7318ff2`.
The same corpus on the final gated commit `fee63c86e`
(evidence `dependent-version-dgeD2D`)
gave the same counts and the same outcome distribution in every class:

- Unit-test scenarios:
  61 cases,
  61 identical.
- Real repository at `0d7318ff2`:
  17 cases
  (the import-only category had three qualifying packages),
  17 identical;
  10 plan bumps
  (between 1 and 80 dependents each;
  the three import-only samples are reached only through bundled development imports and plan 1, 2 and 8),
  7 raise a package nothing published reaches.
- Generated:
  2,000 cases,
  2,000 identical.
  The incumbent's outcomes:
  744 plans with bumps,
  650 raised without a bump,
  216 nothing raised,
  213 unsupported versions,
  91 syntax failures and 86 shape failures of the plan;
  646 policy runs with stale findings,
  192 with an unsupported finding,
  1,003 without a finding,
  81 syntax and 78 shape failures.
- Probes:
  175 cases,
  23 identical
  (the feature did not reach the plan),
  152 differences,
  all matching their prediction,
  0 unexplained.
  Differences per feature:
  `bom` 25,
  `huge-patch` 13,
  `duplicate-name` 19,
  `unpaired-surrogate-name` 25,
  `deep-nesting` 25,
  `non-utf8-manifest` 25,
  `non-utf8-config` 20.

The harness was rewritten to the package's lint rules after a first run
(evidence `dependent-version-avWftA`, commit `926a93a40`);
that run was also free of unexplained differences,
with a generated corpus that differed in detail.

No difference was found outside the probes.

### Positive controls

Each planted defect was built in a disposable copy of the crate
(`--plant`),
and the same corpus was compared again,
on commits `7d9808522` and `ac49900f2`,
which differ only in this document;
each run exited 1:

- `wrong-component`
  (the minor component bumped instead of the patch):
  10 unit,
  10 repository,
  744 generated
  (every generated case whose plan has a bump)
  and 22 probe cases reported
  (evidence `dependent-version-LgLt1u`).
- `skip-peer-dependencies`
  (`peerDependencies` dropped from the runtime fields):
  1 unit,
  258 generated and 5 probe cases reported;
  no repository case,
  since no workspace manifest here names a workspace package under `peerDependencies`
  (evidence `dependent-version-QUy0hL`).
- `skip-bundled-edges`
  (confirmed bundled edges dropped from the walk):
  2 unit,
  9 repository,
  193 generated and 9 probe cases reported
  (evidence `dependent-version-qBOw94`).

The same three plants on commit `c2772ecbb`,
after the merge of `main`,
reported exactly the same counts
(evidence `dependent-version-heMoFx`, `dependent-version-tKmUwV` and `dependent-version-KV5JAu`).
That needed the planted copy to include the scanner crates the merged crate depends on;
before `c2772ecbb` the copy failed to build
(`failed to read .../package/cli/forbidden-strings/Cargo.toml`, evidence `dependent-version-5rHjCA`).
The first harness found the same three plants
(evidence `dependent-version-ZZ6zuR`, `dependent-version-ZYkRJh` and `dependent-version-H7Ux9B`).

The driver exits 1 whenever a difference is unexplained.

### Intentional differences

Each is a defect or an unbounded case of the incumbent that the ledger does not mark as retained behavior.
Each has a probe,
or a direct measurement where the incumbent cannot run.

#### A byte-order mark is kept

The policy's strict `TextDecoder` strips a leading byte-order mark,
so the incumbent's patch rewrites the manifest without it,
and its patch's original side does not match the tracked bytes.
The release task keeps the mark and `JSON.parse` then refuses the manifest.
The native planner parses after one mark and keeps it in both the original and the patched bytes.
Probe `bom`.

#### The patch increment is exact

`String(Number(component) + 1)` rounds a patch component at or above 2^53 through a double.
Measured with `node` on the incumbent's `patchBumpVersion`:
`1.0.9007199254740991` becomes `1.0.9007199254740992` (correct),
`1.0.9007199254740992` stays `1.0.9007199254740992`,
`1.0.9007199254740993` becomes `1.0.9007199254740992`, a lower version,
and `2.3.999999999999999999999` becomes `2.3.1e+21`.
The native increment adds one to the decimal digits at any size.
Probe `huge-patch`.

#### Two manifests with one name are refused

The incumbent maps names to manifests last-wins,
so two bumps of one name share the later manifest's text:
equal versions give two findings for one path,
different ones a `ManifestShapeError`,
and the earlier manifest's bundled edges are replaced by the later one's.
The native planner refuses with `policy-incomplete`,
naming both manifests,
once a version was raised and the configuration exists,
which is where the incumbent starts using names.
Probe `duplicate-name`.

#### A package name with an unpaired surrogate is refused

`"name": "a\ud800"` is a JavaScript string but not Rust text,
and no npm package can carry it.
The native planner refuses such a name as a shape problem.
A dependency key with an unpaired surrogate is dropped:
it can name no workspace package,
since such a name is refused.
A version with an unpaired surrogate is kept as UTF-16 code units and quoted as the incumbent quotes it.
Probe `unpaired-surrogate-name`.

#### Manifests nest at most 511 containers

`monochromatic-jsonc-edit` refuses a 513th open container,
and the parser wraps the text in one array.
`JSON.parse` has no such bound.
Probe `deep-nesting`.

#### An empty package name terminates

`importsPackage` with an empty name never advances its `indexOf` cursor:
`timeout 10 node` on `importsPackage({ sourceText: "import 'x';", packageName: '' })` exited 124,
while the same call with `'x'` printed `true` and exited 0.
The incumbent reaches it when a workspace package is named `""`,
another package lists it under `devDependencies`,
and that package has source files.
The native scan applies its stated rule at every position:
`import ''` imports the empty name.
The generator never produces an empty name,
because the incumbent would not return.

#### Source files are selected by a literal prefix

The incumbent lists a package's source files with the pathspec `:(glob)<directory>/src/**`,
so a directory name holding glob magic selects other paths.
Measured with Git 2.55.0 in a disposable repository holding `package/module/[ab]/src/index.ts`
and `package/module/a/src/index.ts` (an illustrative fixture path; no package has existed there):
`git ls-files ':(glob)package/module/[ab]/src/**'` printed only `package/module/a/src/index.ts`
(an illustrative fixture path; no package has existed there),
which the incumbent's own prefix check then discards,
so the package's own source is never scanned.
The native planner selects `<directory>/src/` by literal prefix.
The fake context in the harness matches paths literally,
so no generated case shows this difference;
the generator uses no glob magic in directory names.

#### The order of dependency names

`Object.keys` lists integer-like keys first;
the native reader keeps source order.
No result of the plan depends on that order,
and no unit scenario uses such a key.

### Differences between the incumbent's two entry points

These are differences inside the incumbent,
which the single native planner resolves the way the policy does.
The release task (`bump:dependents`) reads manifests leniently:
a manifest whose bytes are not UTF-8 is decoded with replacement characters,
can parse,
and is then written back with those characters,
while the policy refuses it;
probe `non-utf8-manifest` shows the policy results identical and the release plan differing.
The same holds for the registry configuration
(probe `non-utf8-config`)
and for a byte-order mark
(section "A byte-order mark is kept").
Once the release task runs `git cli-git fix`,
it refuses those inputs instead of rewriting them.

## Gates

### Container gate

`GIT_POLICY_NATIVE_IMAGE_TAG=dependent-version mise run //package/git-policy/cli:native:test:container`:

- On commit `b8afd3d74`,
  before the merge of `main`:
  524 unit tests passed
  (442 before this branch, 82 added),
  1 ignored
  (`corpus_from_environment`, which only the driver runs),
  37 binary-level tests passed,
  and Clippy with warnings denied passed.
  Image `7b08f389846d`,
  evidence `package/git-policy/cli/target/verification/native-bva1QD`.
  The mutation campaigns ran against this image.
- On the merge commit `343a2696c`:
  605 unit tests passed,
  including the 82 planner tests,
  1 ignored,
  37 binary-level tests passed,
  and Clippy with warnings denied passed
  (`passed.json`: tests and Clippy true).
  Image `d0189d1c2c33`,
  evidence `native-dJWeV6`.
- On commit `fee63c86e`,
  after the fixture modules were split for the Rust linter
  and the fuzz dictionary was fixed:
  605 unit tests passed,
  1 ignored,
  37 binary-level tests passed,
  and Clippy with warnings denied passed.
  Image `1782030bb936`,
  evidence `native-CpbtDP`.
  This is the final gated commit;
  the third mutation campaign ran against this image.

### Package lint

Every harness file lints with 0 findings
(`mise run //package/git-policy/cli:lint:oxlint:paths` per file).
The package-wide `mise run //package/git-policy/cli:lint:oxlint` exits 1
with 171 errors and 1,809 warnings,
all in 81 files that this branch does not change:
no file in `git diff --name-only main...HEAD` has a finding.
`mise run //package/git-policy/cli:native:lint:rust` exits 1 with 273 findings in 58 files,
none of them a file this branch changes.
At the merge commit it also reported 15 findings in two of this branch's test-only modules
(`dependent_version_fixture_cases.rs` over the 300-code-line limit,
and `use` lines without rustdoc there and in `dependent_version_fixture_json.rs`);
commit `90300ff70` split the canonical renderers into `dependent_version_fixture_output.rs`
and documented each `use` line.

### Mutation

Every viable mutant of the twelve planner modules is caught,
on the gate image of the final gated commit `fee63c86e`
(image `1782030bb936`):
the full campaign left one timeout,
and the one rerun of its file caught every mutant with no timeout.
No campaign reported a missed mutant.
Neither the 90-second test limit nor the exclusion list was changed.

#### Final campaign

`GIT_POLICY_NATIVE_IMAGE_TAG=dependent-version mise run //package/git-policy/cli:native:mutation:scoped -- --file 'src/native/dependent_version_*.rs'`,
evidence `package/git-policy/cli/target/verification/native-mutation-EJyi9m`:
206 mutants,
187 caught,
18 unviable,
1 timeout
(`replace is_specifier_position -> bool with true`,
`dependent_version_imports.rs:86:5`),
0 missed.
The unmutated baseline took 10 seconds to test.
The runner's five planted guard removals were all noticed.
The runner process ended before its container did and wrote no `exit.json`;
`exit-collected-manually.json` records the container's exit status (3, from `podman inspect`),
and the report was copied out with `podman cp`.

The timed-out mutant,
run alone in the same image and limits with a 600-second test limit,
was caught with a 6-second test phase against a 10-second baseline
(`native-mutation-EJyi9m/timeout-diagnostic`).
Its neighbours in the campaign took 17 to 64 seconds,
while the sampled load average was between 27 and 55 on 16 CPUs from work outside this branch.

#### Rerun of the file with the timeout

The same task with `--file src/native/dependent_version_imports.rs`,
against the same image,
evidence `native-mutation-fmJr79`
(`exit.json` with status 0):
29 mutants,
29 caught,
0 timeouts,
0 missed;
the baseline took 13 seconds to test.

#### Earlier campaign before the merge

`GIT_POLICY_NATIVE_IMAGE_TAG=dependent-version mise run //package/git-policy/cli:native:mutation:scoped -- --file 'src/native/dependent_version_*.rs'`
against the gate image of commit `b8afd3d74`,
evidence `package/git-policy/cli/target/verification/native-mutation-6RzaA3`
(`mutants.out/outcomes.json`, `exit.json` with status 3):
206 mutants over the twelve planner modules,
179 caught,
18 unviable,
9 timeouts,
0 missed.
The runner's five planted guard removals were all noticed.
The unmutated baseline took 8 seconds to build and 37 seconds to test;
the test limit is 90 seconds.
The 9 timeouts are in `dependent_version_text.rs` (6),
`dependent_version_imports.rs` (2)
and `dependent_version_policy.rs` (1).

#### Its rerun of the three files with timeouts

The same task with `--file` for those three files,
against the same image,
evidence `native-mutation-nA947L`:
78 mutants,
61 caught,
3 unviable,
14 timeouts,
0 missed;
the baseline took 52 seconds to test.
The runner process ended before its container did and wrote no `exit.json`;
`exit-collected-manually.json` records the container's exit status (3, from `podman inspect`),
and the report was copied out with `podman cp`.

#### Why those timeouts are attributed to host load

- The sets differ between runs:
  of the 78 mutants in both runs,
  11 caught in the first timed out in the second,
  6 that timed out in the first were caught in the second,
  and 3 timed out in both.
- The 3 that timed out in both
  (`replace && with ||` at `dependent_version_text.rs:122:21`,
  `replace + with *` at `dependent_version_text.rs:127:30`,
  `replace is_specifier_position -> bool with true`)
  were applied one at a time on the host,
  and `cargo test --lib dependent_version` failed in 0.02 seconds each,
  with 15, 6 and 5 failing tests
  (`native-mutation-nA947L/twice-timed-out-mutants.json`).
  Only `lib.rs` names the planner modules,
  so no other test reaches the mutated code;
  the rest of the 90 seconds is the crate's other tests,
  the same in every mutant.
- Caught mutants' test phases ranged from 5 to 89 seconds in the rerun
  (median 34),
  against the 37 and 52 second baselines.
- Sampled load averages during both campaigns were between 33 and 96 on a host with 16 CPUs,
  from work outside this branch.
- No mutated function steps a loop by hand:
  every loop in the planner modules is a `for` over an iterator or a slice,
  or the graph walk's `while let Some(..) = pending.pop()`,
  which none of the timed-out mutants touches.

Neither the timeout nor the exclusion list was changed.

### Fuzzing

On the tree of `fee63c86e`:

- `mise run //package/git-policy/cli.fuzz:test`:
  21 generator and invariant controls passed,
  4 of them for `dependent_version`.
- `mise run //package/git-policy/cli.fuzz:test:planted`:
  all 14 planted defects were noticed,
  the three of `bin/planted-dependent-version.mjs` among them
  (a nested `version` key rewritten,
  a subpath import missed,
  an escaped quote that ends a string literal);
  evidence `package/git-policy/cli.fuzz/target/verification/planted-YDYB0d`.
- `GIT_POLICY_NATIVE_IMAGE_TAG=dependent-version mise run //package/git-policy/cli.fuzz:smoke`
  (30 seconds per target, AddressSanitizer, 2 GiB and 2 CPUs, no mounts):
  `dependent_version` ran 276,737 executions and exited 0 with no crash artifact,
  its dictionary loaded with 32 entries;
  evidence `package/git-policy/cli.fuzz/target/verification/campaign-4XyQRH`.
  The other targets in the same run:
  `global_arguments` 1,477,892,
  `config_loading` 500,219,
  `config_schema` 63,229,
  `wrapper_controls` 93,647,
  `batch_reply` 304,992,
  each exiting 0.

Two defects of this branch's target were found on the way and fixed in `fee63c86e`:

- The first smoke run stopped before any execution with `ParseDictionaryFile: error in line 30`,
  because the dictionary spelled whitespace as `\n`, `\r\n` and `\t`,
  which libFuzzer does not accept
  (evidence `campaign-8fWevh/dependent_version`;
  `doc/troubleshooting/libfuzzer-dictionary-escapes.md`).
  The entries are now `\x0a`, `\x0d\x0a` and `\x09`.
- The first planted run did not notice "an escaped quote ends a string literal"
  (evidence `planted-CIAYk2`):
  the only escaped quotes in the generated manifests came in pairs inside one string,
  which restores the parity the defect flips.
  A decoy member with one escaped quote,
  `"q":"say \"hi"`,
  now breaks every layout that puts it before the version.

## Choices open to veto

### Two manifests with one name

Refused with `policy-incomplete` once a version was raised and the configuration exists.
The alternative is the incumbent's last-wins mapping,
which yields duplicate findings or a shape failure.

### The exact patch increment

The alternative is to report a patch component at or above 2^53 as unsupported,
as npm's `semver` rejects components above `Number.MAX_SAFE_INTEGER`.

### The empty package name

The scan applies its rule at every position.
The alternative is to treat an empty name as never imported.

### Undecodable manifests and configuration are `policy-incomplete`

The bytes were read,
so the input is present;
the planner's machinery cannot use it.
The alternative is `content-unavailable`.

### One unsupported dependent stops the plan

Retained from the incumbent:
the policy then reports only that dependent,
and the stale findings of the others appear once it is bumped by hand.

### A manifest whose base text does not parse stops the plan

Retained from the incumbent:
a commit that repairs a broken manifest is refused by this policy
until the policy is skipped for that commit.

### The provider lists every tracked path

The seam has no pathspec;
the planner selects paths itself.
In this repository that is 11,443 paths per planning run,
and planning starts only for a commit that modifies a workspace manifest.

### How the surviving mutant was closed

The gate accepts a mutant that is caught or justified as impossible to catch.
The survivor is justified by the equivalence argument in section "The surviving mutant is equivalent after the
candidate fix".
The literal-pathspec controls are regression tests for the defect that the same section records.

## Wiring into the native executable

The planner is wired into `policy_checks::ShippedChecks` for `git add`, `git cli-git check` and `git cli-git fix`.
Commits `0186be5bc`, `c49201e47` and `cd344723a` added the arm and its first controls.
Merge `f41098875` brought in the current `main`, with its content policies and candidate layers, without conflicts.
Commits `8c82803a1` and `ff6162190` added the binary-level controls for `git add` and for a staged deletion.

### The refusal is gone

`ShippedChecks::check` sends `PolicyId::DependentVersionBump` to `check_dependent_version`.
The only policy still answered by `unported_content` is `markdown/autofix`.
These binary-level controls in `binary_dependent_version_tests.rs` show the executable no longer refuses:

- `check_reports_and_fix_bumps_every_dependent`:
  `check --all` exits 1 with the two stale findings,
  and `fix --all` exits 0 with one `fix-summary`, with nothing on standard error.
- `add_of_a_raised_manifest_is_not_refused`:
  `git add` of the raised manifest exits 0, prints nothing, and stages the manifest.
- `a_staged_deletion_does_not_block_the_check`:
  a staged deletion of an unrelated file does not stop `check --all` from reporting both dependents.

Guard proof:
a refusal line placed before the `check_dependent_version` call made the three controls present at the time
(`check_reports_and_fix_bumps_every_dependent`, `a_narrow_fix_changes_unselected_dependents_only_when_they_match_head`
and `add_of_a_raised_manifest_is_not_refused`) fail with exit 2.
After `policy_checks.rs` was restored from `HEAD`, all three passed again.

### What each entry point does

- Direct `check` and `fix` run the planner over the candidate state.
  `fix` installs corrections through the engine's atomic install, as the other content policies do.
- `git add` plans nothing.
  `check_dependent_version` returns no findings for the pre-forward trigger unless the forwarded command is `commit`,
  the same condition as the incumbent at
  `package/git-policy/cli/src/optional/repository-policy/dependent-version-bump-policy.ts:182`.
- `git commit` cannot reach the planner yet.
  The native executable refuses it with exit 2 and the message that the commit transaction
  "is not implemented in this native development executable".
  `check_dependent_version` also returns early for every pre-forward trigger
  and passes `forwards_commit: false`.
  Commit planning changes both together, and belongs to the commit transaction port.

### Release task differential

The release task (`bump:dependents`, called by `changeset:version` in the root `mise.toml`)
and the native `git cli-git fix --all` must write the same bytes.
A scratch driver outside the repository runs both over the same disposable repository, so the driver is not committed.
Each scenario builds two copies of a repository committed as five publishable packages and one private package.
In the worktree, `@s/a` is raised from `1.0.0` to `1.1.0`, the way `changeset version` leaves it.
One copy runs `node package/git-policy/repository/src/bump-dependents.ts` with its working directory in the copy.
The other runs `git cli-git fix --all` through a `git`-named link to the native executable,
with `mono/dependent-version-bump` set to `error`.
The driver then compares every file byte for byte.

- `standard-lf`: both exit 0.
  The release task changes `@s/b`, `@s/d` and `@s/e`, and the native fix changes the same files with identical bytes.
- `standard-crlf`: the same result with CRLF manifests.
- `two-raised`: `@s/a` and `@s/c` are raised.
  Both sides change `@s/b`, `@s/d` and `@s/e` with identical bytes.
- `only-private-dependent`: the private `@s/p` is raised.
  Neither side changes a file.

The driver's summary was `{"scenarios":4,"identical":4,"failed":[]}`,
in scratch run `run-KbE5dZ`.
Positive control: the same driver with `/bin/true` standing in for the native executable
reported differences in `standard-lf`, `standard-crlf` and `two-raised` (summary `identical` 1 of 4),
so a difference in bytes would be seen.

### Gate on the final commit

`GIT_POLICY_NATIVE_IMAGE_TAG=dependent-version mise run //package/git-policy/cli:native:test:container`
on `1899e041ef` (the candidate fix; the gated commit), image `localhost/git-policy-native-test:dependent-version`,
evidence `package/git-policy/cli/target/verification/native-ExrjxD` (`passed.json`: tests and Clippy true):

- 680 unit tests passed, 1 ignored (`corpus_from_environment`, which only the driver runs);
- 45 binary-level tests passed, including the two literal-pathspec controls;
- 1 consumer test passed;
- Clippy with warnings denied passed.

Earlier gates: `d0a65058b6` (evidence `native-vBmFHL`, 44 binary) and `ff6162190` (evidence `native-wqRmpp`, 43 binary).

### Mutation of the wired code

The scoped campaign was
`mise run //package/git-policy/cli:native:mutation:scoped -- --file 'src/native/policy_checks.rs' --file 'src/native/policy_dependent_version.rs' --file 'src/native/dependent_version_*.rs'`
on the gate image of `8c82803a1`, evidence `package/git-policy/cli/target/verification/native-mutation-edbNIJ`:
260 mutants, 208 caught, 49 unviable, 2 timeouts, 1 missed.

- Timeouts:
  `dependent_version_paths.rs:77:5` (`package_directory` replaced by `Vec::leak(vec![0])`)
  and `dependent_version_manifest.rs:198:5` (`dependency_names` replaced by `()`).
- Missed: `policy_dependent_version.rs:255:66`, `!=` replaced by `==` in the filter that builds `listed`.

Rerun of the files with the timeouts, and of the file with the missed mutant, on the gate image of `ff6162190`
(evidence `native-mutation-GkjgYs`):
47 mutants, 33 caught, 13 unviable, 0 timeouts, 1 missed.
Both timeouts were caught on the one rerun; the missed mutant survived both runs and is equivalent after the candidate
fix
(section "The surviving mutant is equivalent after the candidate fix").

#### The surviving mutant is equivalent after the candidate fix

Campaign on the final candidate fix `1899e041ef` (gate image of that commit, evidence `native-mutation-jmgEl0`):
`candidate_prediction.rs` and `policy_dependent_version.rs`, 27 mutants: 8 caught, 15 unviable, 3 timeouts, 1 missed.
The missed mutant is `policy_dependent_version.rs:255:66`,
analysed in the section The surviving mutant is equivalent after the candidate fix.
The three timeouts were rerun alone, as the rule for timeouts requires:

- `policy_dependent_version.rs` alone (evidence `native-mutation-dZ9eEY`): 13 mutants, 4 caught, 8 unviable, 1 missed,
  and no timeout; the `correction_of` mutant that timed out in the first run was caught on its rerun.
- `candidate_prediction.rs` alone (evidence `native-mutation-dZgsWV`): 14 mutants, 6 caught, 7 unviable, 1 timeout.
  The `predicted_add` mutant that timed out in the first run was caught on its rerun.
  The `projected_scope` mutant (`Ok(vec![])`) timed out again at the 90-second test limit.
  Applied by hand on the host, it fails 28 lib tests in 10.6 seconds and 12 binary tests in 12.6 seconds,
  so the suite detects it; the timeout is the container's test phase.
  Rerun with a 300-second per-mutant test limit, on the same gate image (evidence `native-mutation-ClAzBS`):
  the runner was edited in the working tree for this one run to pass `--timeout 300` and `--re projected_scope`
  (the committed runner still passes `--timeout 90` and no `--re`), and it was restored with `git checkout` afterwards.
  The run ran through `node ~/temp/agent/gate-slot.ts` with the runner command
  `mise run //package/git-policy/cli:native:mutation:scoped -- --file 'src/native/candidate_prediction.rs'`.
  Result: 2 mutants, 1 caught (`replace projected_scope -> ... with Ok(vec![])`, listed in `mutants.out/caught.txt`),
  1 unviable,
  0 timeouts, 0 missed; the baseline took 47 seconds to test.
  Its verdict is caught by cargo-mutants itself, so the timeout is closed.
  The hand evidence above (28 lib tests and 12 binary tests failing, in 10.6 and 12.6 seconds) shows the same detection.

The mutant is `policy_dependent_version.rs:255:66`: `!=` replaced by `==` in the filter that builds `listed`.
`listed` reaches the tracked listing only through `LifecycleWorkspace` (`dependent_version_lifecycle.rs:87`),
whose only reader is the loop in `index()` (`dependent_version_lifecycle.rs:181`).
That loop returns `UNLISTED` (`:185`) when a listed path is missing from `ls-files --stage -z --full-name -- :/`
(`:166`).
The original filter lists every non-deleted candidate; the mutant lists the deleted candidates only.
The two differ in outcome only if the loop can refuse, so the question is whether any route can produce an unlisted
candidate.

Every route that gives candidates to the planner, traced in the code:

1. Direct check or fix. `management.rs:445` builds `CandidateRequest::Direct(scope)`.
   `prepare_candidates` (`candidate_prediction.rs:480`) builds one store whose prefix is the global arguments
   plus `--no-literal-pathspecs` (`:481-483`).
   `projected_scope` (`candidate_prediction.rs:385`) stages the selection with that same prefix (`:486-491`),
   and `scope_candidates` (`candidate_stage.rs:482`) turns the entries of `ls-files --stage -- <pathspecs>`
   (`candidate_prediction.rs:405`, `candidate_store.rs:407-420`) into records that are `Added` or `Modified`.
   The listing reads `ls-files --stage -- :/` from the same store, prefix and private index.
   The staging is the only index write, and it finishes before either read.
   An entry matched by the pathspecs is an entry of `:/`, so every candidate is listed.
   A non-zero stage is refused on both sides (`candidate_stage.rs` `UnmergedPath`,
   `dependent_version_lifecycle.rs:172`).
   Direct fix passes reuse the same prepared store (`policy_content.rs:308`), so they follow this route.
2. Forwarded `git add`. `wrapped_command.rs:297` builds `CandidateRequest::Add`, and the forwarded stage runs with
   `Trigger::PreForward` (`wrapped_command.rs:352`).
   `check_dependent_version` returns clean for that trigger before it reads any candidate
   (`policy_dependent_version.rs:228-230`).
   Add records can contain `Deleted` from `delta_candidates` (`candidate_stage.rs:416`), but they never reach the
   filter.
3. Forwarded `git commit`. Only the `add` branch sets candidates (`wrapped_command.rs:296-299`),
   and the trigger is `PreForward`, so the planner returns clean.

On route 1 every candidate is listed by construction, and on routes 2 and 3 the filter is never reached.
So the listing loop cannot refuse in any reachable run, and the mutant's changed set changes no outcome.
The mutant is equivalent for every reachable input, and it is justified as impossible to catch on that basis.

Before the fix in `1899e041ef`, route 1 did not hold under `GIT_LITERAL_PATHSPECS=1`:
the listing read `:/` with literal semantics and found no entry, so the loop refused and the mutant was detectable.
That defect is the one the controls below pin; it is fixed, and the controls stay as regression tests.

Defect and fix. A direct check of one named manifest, and `check --all`, under `GIT_LITERAL_PATHSPECS=1`
refused with `content-unavailable` (exit 2) and `transaction-failed` respectively.
Cause: the direct prediction's listing and staging used the caller's literal pathspec mode, so `:/` matched nothing,
and `git add --all -- :/` failed. Commit `1899e041ef` gives the direct prediction the add prediction's magic-enabled
prefix.
Controls (binary, `binary_dependent_version_tests.rs`):
`literal_pathspecs_check_a_named_manifest_like_plain_mode` and `literal_pathspecs_check_all_like_plain_mode`
were committed first in `da49a81e62`, where both failed against the unfixed code with exit 2, then fixed in
`1899e041ef`.
They replace the pinning control of `d0a65058b6`, which asserted the refusal.

Choice open to veto. Under `GIT_LITERAL_PATHSPECS=1`, user pathspecs in the direct prediction now have glob meaning,
so a literal name containing `*`, `?` or `[` can match more files than literal mode would.
Checks then see a superset of the literal candidates, and a fix could correct more files.
The alternative is to wrap each user pathspec as `:(literal)` in literal mode only, which needs the mode detected from
the environment.

### Fuzz smoke on the final commit

`GIT_POLICY_NATIVE_IMAGE_TAG=dependent-version mise run //package/git-policy/cli.fuzz:smoke`
on `ff6162190`, 30 seconds per target, evidence `package/git-policy/cli.fuzz/target/verification/campaign-8DFFYG`:
all nine targets exited 0 with no crash artifact.
`dependent_version` ran 59,530 executions.

## What the wiring step needed

Every item below landed as described in section "Wiring into the native executable".
The list stays as the reference for what the wiring covers.

- A second failing `policy_engine::PolicyOutcome` variant for `policy-incomplete`
  and one more arm in `run_policy_stage`,
  as recorded in `cli-git-native-policy-engine.md`,
  section "The failure code of a policy that could not finish follows its cause".
- An arm for `PolicyId::DependentVersionBump` in `policy_checks::ShippedChecks` that:
  builds a `DependentRequest` from the trigger,
  whether the forwarded subcommand is `commit`,
  and the lifecycle's candidates with their change kinds;
  supplies a `WorkspaceContent` that answers from the candidate layer
  (the tracked paths of the candidate state with their modes,
  candidate-state bytes,
  and base bytes: `HEAD`,
  or the preparation base inside a commit transaction);
  and maps `PlanError::ContentUnavailable` to `content-unavailable`
  and `PlanError::PolicyIncomplete` to `policy-incomplete`,
  with `plan_error_message` as the text.
- A conversion of each `DependentFinding` to a `PolicyFinding`
  (`code`, `message`, `path` as text, `fix_available` when `patch` is present)
  and of its `ManifestBump` to a full-content patch of a tracked file that may not be a candidate,
  at that file's exact revision and mode,
  from `original` to `replacement`;
  the engine's added-path rules then decide whether it applies
  (`SPEC.md`, the paragraphs on patches that name a tracked file that is not a candidate,
  and section "Added paths").
- `cli-git.config.jsonc` must list `mono/dependent-version-bump`:
  the native registry gives it `Off` unless listed.
- The release task:
  `changeset:version` runs `git cli-git fix --all --policy mono/dependent-version-bump`
  instead of `bump:dependents`.
  The direct fix's candidates must include the manifests `changeset version` left modified in the worktree.
  The task's optional base-revision argument has no equivalent in the policy path;
  the base is `HEAD`.
- The differential harness can then compare the wired policy:
  a provider over a disposable repository would replace the in-memory workspace.

## Shared lines touched in files this branch does not own

- `package/git-policy/cli/src/native/lib.rs`:
  module declarations inserted after `pub mod rule_add_explicit;`
  (twelve modules,
  and five test-only modules after `pub mod dependent_version_policy;`).
- `package/git-policy/cli/mise.toml`:
  one task block,
  `native:differential:dependent-version`,
  before `test:git-image`.
- `package/git-policy/cli.fuzz/Cargo.toml`:
  one `[[bin]]` block for `dependent_version`,
  after the `batch_reply` block.
  File-enforcer manages this file;
  `mise run sync:files` left it unchanged before and after the merge of `main`.
  Each run also rewrote root `mise.toml` and `package/config/pnpr/config.yaml`
  from the local package set;
  that drift is unrelated and was not committed.
- `package/git-policy/cli.fuzz/src/lib.rs`:
  one module declaration at the end.
- `package/git-policy/cli.fuzz/bin/container.mjs`:
  one entry at the end of `targets`.
- `package/git-policy/cli.fuzz/bin/planted-controls.mjs`:
  one `import` line and one spread line at the end of `plants`.
- `package/git-policy/cli.fuzz/README.md`:
  a section "`dependent_version`" after "`batch_reply`" and before "Controls".

The merge of `main` (commit `343a2696c`) conflicted in the five `cli.fuzz` files,
where `main` had appended its `batch_reply` entry at the same place;
each keeps both entries,
`main`'s first.
The planner modules and their tests are byte for byte those of the gated commit `b8afd3d74`
(`git diff b8afd3d74 HEAD -- 'package/git-policy/cli/src/native/dependent_version_*.rs'` is empty).

## Not verified

- The ledger's consumer-level test (three manifests in the standard fixture, a commit of one raised manifest)
  has not run, because `git commit` is refused by the native executable until the commit transaction is ported
  (section "Wiring into the native executable").
- The release task itself was not switched to `git cli-git fix`;
  the differential runs the task and the native fix side by side in disposable repositories.
- The release workflow end to end (`changeset version`, then the fix in CI) was not run;
  the differential simulates the `changeset version` result by writing the raised manifest.
- The incumbent ran over a fake policy context,
  not the wrapper's candidate layer,
  and its pathspecs were matched literally;
  section "Source files are selected by a literal prefix" is the one place real pathspec semantics differ.
- Only Linux was exercised.
- Superseded by merge `f41098875`, which took the `main` of that time (`c3a4fd284`).
  Recorded as it stood then: `main` moved again after the final gate:
  at `a2c7b8ec0` it has eight commits under `package/git-policy/` that this branch does not merge
  (`git log 439672ddf..a2c7b8ec0 -- package/git-policy/`).
  `git merge-tree` shows conflicts in the same five `cli.fuzz` files as the first merge,
  where both sides append one entry;
  none of the eight commits touches a `dependent_version_*` file;
  and the one `jsonc-edit` change among them
  (`f313a8933`, release 0.1.1)
  renames local variables in `parse.rs` and `scan.rs` without changing behavior.
  The gate, campaigns and fuzz runs here are on `fee63c86e`, which does not include them.
- Processes started from the agent's shell ended early several times,
  for a cause not identified:
  the mutation runner of `native-mutation-nA947L` and of `native-mutation-EJyi9m`
  while their containers kept running
  (their reports were collected from the containers,
  as each `exit-collected-manually.json` states),
  and the runner of `native-mutation-LaSpZU` during its planted guard controls,
  before any campaign;
  that rerun was started again and is `native-mutation-fmJr79`.
  The last sequence
  (the rerun of `dependent_version_imports.rs`, the fuzz planted controls and the smoke)
  ran as a user unit started with `systemd-run --user`
  so that it would outlive the agent's shell;
  each step is the package task named in its section.
- The Oxlint 1.86.0 default formatter aborted earlier in this work
  (a panic at `apps/oxlint/src/output_formatter/default.rs:181:14`)
  while linting a harness file with many findings;
  `--format=unix` worked.
  It did not recur in the package-wide run,
  and no reproducer was kept.

## Commits

On `feat/cli-git-native-dependent-version`, oldest first, first parent only:

- `f463f055c` the planning core.
- `c246ef070`, `926a93a40` the differential harness and its outcome counts.
- `b8afd3d74` the test that kills the callee-position mutant; the first gated commit.
- `cabd397b7` the `dependent_version` fuzz target.
- `84d38472e` this document.
- `0d7318ff2`, `3a0ed3610` the harness rewritten to the bin-script lint rules.
- `7d9808522`, `ac49900f2`, `583aaf6f7`, `de22245c4` this document: rerun corpus, mutation, planted controls.
- `343a2696c` the merge of `main`.
- `ee68b8895`, `4a5ddd23f` this document: merge, lint state, gate on the merge.
- `90300ff70` the fixture renderers split out for the Rust linter.
- `fee63c86e` the fuzz dictionary escapes and the odd escaped-quote decoy; the final gated commit.
- `bba8c3100` `doc/troubleshooting/libfuzzer-dictionary-escapes.md`.
- `fe9cbc5fe`, `ae70c7c8f` this document: final gate, mutation, fuzzing.
- `c2772ecbb` the planted differential copy includes the scanner crates.
- `0186be5bc`, `c49201e47`, `cd344723a` the wiring into `ShippedChecks` and its first controls.
- `f41098875` the merge of `main` at `c3a4fd284`, without conflicts.
- `8c82803a1` the `git add` binary control.
- `ff6162190` the staged-deletion binary control.
- `e5a7b608e7` the merge of `main` at `9ece20bd66`, with six conflicts resolved
  (doc-comment reflow, the wiring replacing the dependent-version refusal, and the candidate-tree declaration kept).
- `5e3a2f7158` the handover cleanup: the dependent-version refusal is gone from two handover documents.
- `d0a65058b6` the literal-pathspec control that asserted the refusal (replaced by `da49a81e62`).
- `da49a81e62` the two literal-pathspec controls, committed failing against the unfixed code.
- `1899e041ef` the candidate fix: the direct prediction reads pathspecs with the add prediction's magic-enabled
prefix. Gated commit.
- The commit that adds the section "The surviving mutant is equivalent after the candidate fix" and this list.
- The commit that adds the section "Wiring into the native executable" and this paragraph.

## Merge of main at 1b6677a9af, gated at 39ba42039c

The merge commit is `0b9e4e3eda` (main brought the Markdown autofix branch, `1b6677a9af`).
Conflicts and their resolution:

- `package/git-policy/cli.fuzz/Cargo.toml`, `bin/container.mjs`, `bin/planted-controls.mjs`, `src/lib.rs`:
  both sides kept, the dependent-version target and the Markdown targets (`linter_output`, `event_path`,
  `linter_config`).
- `package/git-policy/cli/src/native/policy_checks.rs`:
  `MarkdownAutofix` runs `check_markdown` (main's policy), `DependentVersionBump` runs `check_dependent_version` (this
  branch).
  The `unported_content` helper and `MARKDOWN_AUTOFIX_NEEDS` / `DEPENDENT_VERSION_BUMP_NEEDS` were removed,
  since no content policy is unported now.
- Tests that asserted a refusal, rewritten to the wired behavior: `policy_checks_tests.rs`, `policy_pass_tests.rs`,
  `wrapped_command_tests.rs`, `direct_fix_tests.rs` (`a_fix_with_the_wired_policy_listed_completes_and_installs`),
  `binary_frontier_tests.rs` (`add_runs_the_content_policies_over_what_it_stages`), `binary_policy_tests.rs`
  (`direct_fix_corrects_only_worktree_files`, its refusal step removed).
- `policy_dependent_version.rs` and its tests: `PolicyOutcome::Failed` and `PolicyFinding.path` now carry `EventPath`
  (`EventPath::from_git_bytes` for findings, `path: None` for failures).

Doc-comment regeneration (`mise run format:monochromatic-lint -- package/git-policy/cli/src/native`) was run.
It also rewrote files this merge did not touch; those changes were reverted, and only the files named above keep
the regenerated comments.

Gate on `39ba42039c` (the gated commit; container, through `~/temp/agent/gate-slot.ts`), evidence
`package/git-policy/cli/target/verification/native-Wde3ai` (`passed.json`: tests and Clippy true):
736 unit passed, 1 ignored; 49 binary passed; 1 consumer passed.

Host runs show the same environment-bound failures as before (Git 2.55 and wrapper resolution), plus two tests that need
`GIT_POLICY_NATIVE_TEST_LINTER`, which only the gate image sets.

Gated paths: `package/git-policy/cli`, `package/git-policy/cli.fuzz`, `package/cli/forbidden-strings`,
`package/rust-module`, `package/linter/monochromatic-lint`, `clippy.toml`.
`git diff --quiet 39ba42039c HEAD -- <those paths>` is empty at the time of writing.

### Drained state

Stopped for a `~/temp` move. Nothing is left running in the repository.

In flight when drained:

- Fuzz smoke on `39ba42039c`:
  `GIT_POLICY_NATIVE_IMAGE_TAG=dependent-version node ~/temp/agent/gate-slot.ts mise run //package/git-policy/cli.fuzz:smoke`,
  log `~/temp/agent/dv-gate/fuzz-smoke2.log`, campaign directory
  `package/git-policy/cli.fuzz/target/verification/campaign-aq6uCe`.
  Finished before the drain report: all twelve targets exited 0 (`dependent_version` 221,829 executions;
  `linter_output`, `event_path`, `linter_config` included), `FUZZ-EXIT=0`.
  The fuzz step is therefore complete and is not in the list of work left.

Not done yet:

- Scoped mutation over the changed planner and candidate files, with its timeout rerun:
  `GIT_POLICY_NATIVE_IMAGE_TAG=dependent-version node ~/temp/agent/gate-slot.ts mise run //package/git-policy/cli:native:mutation:scoped -- --file 'src/native/policy_checks.rs' --file 'src/native/policy_dependent_version.rs'`.
  Any timed-out mutant is rerun alone with `--timeout 300` (temporary runner edit, restored afterwards, as before).
- Lint: `format:monochromatic-lint` reports `rust/no-anonymous-functions` warnings in `policy_dependent_version.rs`
  (lines 226, 311, 312) and `policy_dependent_version_tests.rs` (lines 50, 289). Not addressed.

## Lint refactor, gate and scoped mutation on 3186bb316c

The Rust linter (`format:monochromatic-lint`) reported `rust/no-anonymous-functions` for five closures in
`policy_dependent_version.rs` and its tests. Commit `3186bb316c` names them: `unreadable_failure`,
`is_listed_change` and `candidate_path` in the planner, `dependency_block` and `b_patch_conflict` in the tests.
The linter reports nothing for those two files afterwards.

Gate on `3186bb316c`, evidence `package/git-policy/cli/target/verification/native-5aXXmC`
(`passed.json`: tests and Clippy true; run through `~/temp/agent/gate-slot.ts`, log `~/temp/agent/dv-gate/gate6.log`):
736 unit passed, 1 ignored; 49 binary passed; 1 consumer passed.

Scoped mutation over `src/native/policy_checks.rs` and `src/native/policy_dependent_version.rs` on the same gate image
(evidence `native-mutation-oI9441`, log `~/temp/agent/dv-gate/mutation-8.log`):
`cargo mutants` ran 34 mutants: 10 caught, 21 unviable, 3 missed, 0 timeouts.
No timeout occurred, so no solo rerun was needed.

The 3 missed mutants are in `is_listed_change` (`policy_dependent_version.rs:204`), the helper that the lint refactor
created from the filter that section "The surviving mutant is equivalent after the candidate fix" justifies:
`is_listed_change -> true`, `is_listed_change -> false` and `!= -> ==` in it.
The same proof applies unchanged, because the helper is the same filter
and has the same single reader (the `UNLISTED` loop).
On every route that reaches the planner the listing check cannot refuse, so no mutant of the filter changes an outcome.
The earlier campaign on `1899e041ef` recorded the same survivor at the old line 255.

Rerun commands, for a later rerun after a land or a merge:

- Gate:
`GIT_POLICY_NATIVE_IMAGE_TAG=dependent-version node ~/temp/agent/gate-slot.ts mise run //package/git-policy/cli:native:test:container`
- Mutation:
`GIT_POLICY_NATIVE_IMAGE_TAG=dependent-version node ~/temp/agent/gate-slot.ts mise run //package/git-policy/cli:native:mutation:scoped -- --file 'src/native/policy_checks.rs' --file 'src/native/policy_dependent_version.rs'`
