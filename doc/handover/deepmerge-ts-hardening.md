# deepmerge-ts hardening grilling handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/mutation-test` is now `package/mutation-test/cli`.

## Status

Shared understanding confirmed 2026-09-23;
first implementation pass done 2026-09-24 (see `Outcomes`).
The user judged the defect count too low for the search effort;
the widening pass that followed is integrated into both drafts as of 2026-09-24 (see `Widening pass` and `Outcomes`),
so the drafts are ready for the user's review.

Next actions, in order:

1.   Mutation-testing fork: done 2026-09-24
     (report `doc/audit/deepmerge-ts-mutation-2026-09-24.md`;
     upstream suite 800 of 1098 Stryker mutants, sidecar now 1017, the other 81 argued equivalent).
     Its four public findings were reproduced independently on 8.0.2 and added to issue sections 2, 8, 10, 11, and 13;
     `fix/into-first-value-typing` was built and leaves the `filterValues` empty-seed case unchanged.
     No new security candidates.
     User decision 2026-09-24: make the run reproducible as a Stryker image task in this package
     (chosen over extending `package/cli/mutation-test`, which cannot run upstream's Vitest suite,
     and over leaving it documented only).
     Ported as the `mutation`, `mutation:sweep`, and `mutation:differential` tasks (commits `d1a3cd6d1`, `50348c435`):
     the rerun from a `v8.0.2` checkout reproduces 800 of 1098 with matching mutant ids,
     the sweep baseline passes all sidecar files, four known mutants reproduce their verdicts,
     and the differential controls hold.
     Generator-widening fork (the report's generator gaps): done, commits `1d2cc683f` to `035805aa8`;
     its new divergence (into never passes a target-only Map entry to a custom function)
     and a slot-written `actions.defaultMerge` ignored by into array, Set, and Map functions
     are pinned in `known-defect-options.unit.test.ts` and in issue section 8.
     Campaign failure at 05:58 (seed 297866779) came from its half-edited files: the seed passes on the committed files.
2.   Integrate each widening fork's report per `Widening pass`.
     Done for the exotic, options, scale, and aliasing forks
     (findings reproduced independently and added to the local drafts;
     model and snapshot made realm-independent in `src/realm.ts`, commit `fa548bf15`).
     New fork branch `fix/cross-realm-collections` (worktree `~/temp/agent/deepmerge-ts-fix-cross-realm`):
     291 upstream tests pass, its new test fails 5 of 6 cases on unfixed `src`,
     and the sidecar run against its build turns only the cross-realm known-defect test red.
     The other new defects wait for an upstream design choice (throw or document), so they have no branch.
     Declared-type fork integrated too
     (7 new type classes; issue snippet type-checked verbatim in a local file;
     capped `fuzz:declared-types`, `fuzz:declared-types:control`, and `generate:declared-type-cases` tasks,
     the control catching 42 of 45 tampered cases and the corpus regenerating byte-identical).
     The restarted campaign found a new defect on its own
     (one `deepmergeInto` call rewrites a cyclic source node it stored by reference, seed 1395258848);
     pinned in `known-defect-alias.unit.test.ts`,
     with the source-mutation property narrowed by node identity (`src/alias-rewire.ts`)
     and a positive control failing at the same test number with the exclusion removed.
     The next campaign run (seed 1831879258) found the same rewrite one node further in,
     on a source node the target reaches only through the stored one;
     pinned beside it (commit `07f01a556`),
     with the property now skipping every source node the target reaches after the call
     and a helper test showing writes into unreached source nodes are still reported.
     Issue section 6 carries both repros, each checked on 8.0.2.
     `lint:oxlint` passes but prints plugin warnings (`effect-summary-omission`, `readonly-source-evidence`)
     for `src/declared-type-sample.ts`:
     the `prefer-readonly-parameter-type` plugin's deliberate omission record for a TypeScript tuple-serialization panic
     (`doc/handover/prefer-readonly-parameter-types-issue-review.md`, "Verified controls"),
     so that one callable goes unanalyzed by that rule, not a finding in this package.
     `format:oxlint`, `lint:oxlint` (0 warnings, 0 errors), `lint:types`, and `test:unit` pass;
     coverage baseline ratcheted to 1765 of 1772 lines.
3.   Campaign with the widened generators stopped on the user's call after about 2 hours:
     580 rounds of 10000 runs per property file passed with no counterexample.
     Rerun it after any generator widening, upstream release, or fork fix,
     and triage any counterexample per `Campaign`.
4.   Method audit (see `Method audit`), started 2026-09-24 after the user rejected the drafts as too thin:
     "This can't be it. I expected many more findings. Maybe we're not looking for findings correctly?"
     All three reports integrated (see `Integration after the three reports`);
     The campaign on the widened generators was stopped on the user's call after about a day:
     4326 rounds of 10000 runs per property file passed with no new counterexample
     (the only failure record is the earlier, already-pinned seed 1831879258).
     User decision 2026-09-24: the user reviews and posts both drafts personally.
5.   Open PRs from the fork branches if the maintainer wants them,
     and follow `After upstream responds`.

User constraints from confirmation:
all security findings go into one combined private advisory draft,
all other upstream items go into one combined issue draft,
and both stay local until the user reviews and posts them personally.
Only those two drafts are held (user correction the same day):
the GitHub fork is created and non-embargoed fix branches are pushed to it.
Embargoed repros, properties, and fixes belong to the advisory and stay in `*.local.*` files (Q8).

Findings under disclosure embargo live only in gitignored `*.local.*` files
(see `Embargo`);
this public document never describes them.

## Objective

The user reported that <https://github.com/RebeccaStevens/deepmerge-ts>
"doesn't seem well-tested or fuzzed" and asked to resolve it.
The repo recommends the library in `doc/decision/cli-git-policies-platform.md`
and uses its semantics as the reference for rule-settings merging in `doc/handover/unified-linter.md`;
no workspace package depends on it at runtime.

Purpose stated by the user 2026-09-24:
settle deepmerge-ts so it can serve as the oracle for the user's upcoming Rust fork of it.
So every behaviour the fork will be compared against must be characterized,
each open intent question in the issue draft is an unsettled oracle region until upstream answers,
and the JSON-serializable corpus in `src/json-case.ts` (Q13) is the likely hand-off format.

## Evidence

Upstream audit run 2026-09-23 against `main` at `17fc99cb` (v8.0.2),
in a capped container (`podman run --memory=2g --cpus=2 --rm`, `node:24-slim`):

- Maintenance is active:
   v8.0.0 to v8.0.2 released in 2026-08,
   last commit 2026-09-03,
   zero open issues or PRs,
   private vulnerability reporting enabled.
- Vitest 4 suite:
   285 runtime tests pass,
   plus 109 `tsd` type assertions.
- Coverage is informational only:
   `vitest.config.ts` sets watermarks, not thresholds,
   and `.github/codecov.yml` marks project and patch status `informational: true`.
   Measured totals:
   statements 95.97%,
   branches 90.04%,
   functions 100%,
   lines 95.9%.
- No fuzzing or property-based testing:
   no `fast-check`, `jsfuzz`, or similar in manifests or the lockfile.
- The npm tarball of 8.0.2 ships `dist/index.mjs` and `dist/index.cjs` without source maps
   (`npm pack --dry-run deepmerge-ts@8.0.2`),
   so black-box coverage maps only onto built output.
- Public, non-embargoed defects reproduced by probe:
  - `getCyclicReferenceDepth` in `src/utils.ts` checks `parents.includes(object)` across all inputs' ancestors;
     `deepmerge({ k: s }, { k: { k2: s } })` returns `r.k.k2 === r.k`,
     a cycle absent from every input.
- Untested behaviour:
   getters (invoked and flattened, throwing getters propagate),
   frozen targets and sources,
   `constructor.prototype` and nested `__proto__` payloads,
   `deepmergeInto` with `filterValues`,
   `rootMetaData`,
   sparse arrays (holes dropped by `Array.prototype.flat`),
   merges past the default `maxDepth` of 1000 (silently fall back to last-value-wins).
- Monochromatic is a public repo with auto-push,
   and no fork `Aquaticat/deepmerge-ts` existed on 2026-09-23.

## Decisions

- Q1 and Q6:
   both an in-repo sidecar package `package/module/deepmerge-ts.fuzz`
   and a GitHub fork under the user's account.
   The sidecar is the durable test home using `module-test`, `mise`, and `fast-check`;
   the fork stays on upstream's toolchain and exists only to carry focused fix branches upstream.
- Q2:
   properties check both a reference model of documented semantics
   and invariants (no input mutation, no prototype pollution outside documented `FastUnsafe` variants,
   no crash, identity laws).
   The model is intended to double as the reference for the Rust unified-linter merge.
- Q3:
   upstream issues, PR descriptions, and advisories are drafted locally;
   the user files them.
- Q4 and Q5:
   two modes over the same properties:
   a fixed-seed bounded run inside `test:unit`,
   and an unbounded `fuzz` campaign that runs until a counterexample or interruption,
   persists the replayable seed and path,
   and runs under container resource caps per `RXI` and `BOX`.
- Q7:
   Vitest stays in the fork only;
   upstream PRs add Vitest regression tests in upstream style,
   and this repo never adopts Vitest.
- Q8:
   embargoed findings, their repro, their property,
   and advisory drafts live only in files matching `*.local.*`
   (ignored by `.gitignore:11`);
   they land publicly after upstream publishes an advisory,
   or after 90 days without response.
- Q9:
   the sidecar tests the npm release by default
   and can target a build of a fork branch for source-mapped coverage and pre-release fix verification.

## Settled without asking (veto open)

- The sidecar targets the npm package `deepmerge-ts`,
   not `jsr:@rebeccastevens/deepmerge`,
   because the workspace installs from npm.
- Every exported function is covered,
   including the four `FastUnsafe` variants,
   per `PKG` and `TCV`;
   pollution invariants exempt `FastUnsafe` variants where upstream documents them as unsafe.
- Non-embargoed fix branches may be pushed to the user's fork (`PX3`);
   opening PRs waits for the user (Q3).
- Any further finding with plausible security impact joins the embargo (Q8).

## Embargo

Details:
`doc/handover/deepmerge-ts-hardening.local.md` (gitignored, local machine only).

## Open questions

None.
Frontier empty on 2026-09-23;
confirmed by the user the same day.

## Answered 2026-09-23 (round 5)

- Q14:
   a local draft issue proposes contributing the property suite upstream
   (fast-check under their Vitest);
   contribute only if the maintainer agrees.
- Q15:
   do not propose enforced coverage thresholds upstream.
- Q16:
   the sidecar keeps a V8 coverage-reachability gate with a committed baseline,
   mirroring `package/module/logger.fuzz` `fuzz:coverage`;
   the baseline refreezes when the pinned deepmerge-ts version changes.
- Settled (veto open):
   known unfixed defects never leave `test:unit` red (`PKG`).
   Model-equality properties exclude each known-defect region,
   and a paired known-defect test asserts the defect still reproduces,
   citing its draft issue,
   so an upstream fix flips it and forces re-inclusion.
- Settled (veto open):
   public generators stay inside bounds that avoid embargoed findings;
   the excluded regions are exercised only by `*.local.*` properties until the embargo lifts.

## Planned sequence

1. One combined local advisory draft for every embargoed finding, for the user to file privately.
2. Sidecar `package/module/deepmerge-ts.fuzz`:
   reference model,
   invariants,
   fixed-seed `test:unit`,
   unbounded containerized `fuzz`,
   `fuzz:coverage` gate,
   `README.md`.
3. GitHub fork `Aquaticat/deepmerge-ts`,
   local checkout under `~/temp/agent` pushing only to the fork,
   and sidecar support for targeting a local branch build.
4. Fix branches with Vitest regression tests for non-embargoed defects,
   pushed to the fork;
   no upstream PRs until the user posts the combined issue.
5. One combined local issue draft:
   non-embargoed defects,
   `maxDepth` fallback docs request,
   sparse-hole intent question,
   property-suite proposal.

## Answered 2026-09-23 (round 4)

- Q10:
   the model accepts the silent last-value-wins fallback past `maxDepth`;
   a local draft issue asks upstream to document it.
- Q11:
   sparse-array holes are neither defect nor accepted yet;
   a local draft issue asks upstream for the intended behaviour.
   Until answered,
   model-equality properties generate dense arrays only,
   and one characterization test pins current hole-dropping so an upstream change is noticed.
- Q12:
   getters are accepted as read-and-flatten,
   matching object spread;
   a throwing getter propagating is expected behaviour.
- Q13:
   the Rust fixture export waits for the merge-crate vet named in `doc/handover/unified-linter.md`;
   model cases are written JSON-serializable now.
- Settled (veto open):
   the embargoed advisory draft is written first,
   because its disclosure clock starts only once the user files it.

## Rejected ideas

- Replacing deepmerge-ts with an in-house merge module (Q1 option C):
   duplicates an actively maintained library.
- Dropping the recommendation (Q1 option D):
   pushes untested ad hoc merges onto every consumer.
- Porting upstream's Vitest suite into `module-test` (Q7 V2):
   duplicates passing tests and drifts every release.
- Committing embargoed repro publicly (Q8 C).
- Associativity as an invariant (`deepmerge(a, b, c)` equals `deepmerge(deepmerge(a, b), c)`):
   false under the documented n-ary rule;
   `deepmerge(null, [x], [])` is `[]` while the pairwise nesting is `[x]`.
- Fixing the type-level defects in fix branches:
   they touch upstream's HKT design, so the issue draft asks for a direction first.

## Outcomes

Sidecar package `package/module/deepmerge-ts.fuzz` (README there lists every layer):

- Model properties for all eight entry points,
   invariant properties,
   known-defect and accepted-behaviour tests,
   helper unit tests,
   about 60 hand-written `expectTypeOf` assertions,
   and a generated type-level soundness corpus of 1000 cases
   (600 widened literals, 400 `as const`),
   each asserting that the runtime result fits the static result type.
- Coverage-reachability baseline over the npm `dist`:
   1765 of 1772 lines for `deepmerge-ts@8.0.2` after the widening pass,
   up from 1676 after the first pass and 1569 when first frozen;
   `defaultMetaDataUpdaterFast` is still never called.
- Widening-pass layers (README lists each):
   realm-independent model and snapshot,
   exotic inputs, generated option plans, alias graphs with a bisimulation oracle and per-node rewrite detection,
   declared-type fuzzing (439-case corpus, capped campaign, positive control),
   scale probes (embargoed parts machine-local),
   and mutation-derived tests.
- Mutation testing, reproducible via the `mutation`, `mutation:sweep`, and `mutation:differential` tasks
   (`doc/audit/deepmerge-ts-mutation-2026-09-24.md`):
   upstream's suite detects 800 of 1098 Stryker mutants;
   with the sidecar, 1017, and the remaining 81 are argued equivalent.
- Unbounded campaign (`fuzz` task) in a 2 GiB / 2 CPU podman container;
   the run started 2026-09-24 had passed 363 rounds of 10000 runs per property
   with no counterexample at the time of writing.
- `fork:build` builds a checkout with source maps;
   `fuzz:coverage` with `DEEPMERGE_FUZZ_TARGET` reports per `src/*.ts` file.

Fork <https://github.com/Aquaticat/deepmerge-ts>
(local checkouts under `~/temp/agent/deepmerge-ts-fork` plus one worktree per branch;
`upstream` push URL is `DISABLED`;
cli-git auto-push sends commits to the fork,
so embargoed work must never be committed there):

- `fix/false-cycle-detection`:
   cycle detection only counts an ancestor on the value's own path.
   Upstream suite 288/288;
   new guards fail without the fix;
   against its build the sidecar's false-cycle known-defect test turns red and nothing else does.
- `fix/into-first-value-typing`:
   `deepmergeInto` and `deepmergeIntoFastUnsafe` type each merge by the first filtered value.
   Upstream suite 293/293;
   8 new guards fail without the fix;
   the sidecar's two into known-defect tests turn red;
   5000 runs of the into model properties with object leaves and `undefined` restored pass on the fix and fail on 8.0.2.
   It does not cover the `filterValues` empty-seed case (built and checked).
- `fix/cross-realm-collections`:
   Sets and Maps from another realm merge as collections.

Public runtime findings (in the issue draft):
false cycle detection,
`deepmergeInto` first-value typing (three symptoms),
two intent questions (array holes, deeper single-input cycles passed through),
and a docs request for the `maxDepth` fallback.
Type-level findings (in the issue draft):
index signatures absorbing known keys,
union-typed mergeable values,
optional-then-required records,
custom `maxDepth` / `mergeRecords: false` / `filterValues: false` / `mergeArrays: false`,
`deepmergeInto`'s `Target & Merged` intersection,
`deepmergeIntoCustom` leaving the target type unchanged,
`any` inputs,
plus two sound imprecisions.

Added by the widening pass, each reproduced independently on 8.0.2 before entering the drafts:
the issue draft now has 13 sections
(runtime defects, intent questions, a docs request, result types including 7 declared-type classes,
and the property-test proposal with the mutation score),
and the advisory draft 7 findings (4 new).
The campaign found two further runtime defects on its own
(single-call `deepmergeInto` source rewrites, seeds 1395258848 and 1831879258).

### Unexercised surfaces after the widening pass

Each fork's own list, plus gaps measured here:

- Options (first options fork): custom functions returning a container of another kind;
   custom functions calling `utils.mergeFunctions` or `utils.defaultMergeFunctions` directly;
   the documented `skipme` and `keyPath` examples
   (the fork's list is truncated after this item in the recovered transcript).
- Options (generator-widening fork): into array, Set, Map, or records functions writing `actions.defaultMerge`
   into the slot (probed by hand here and pinned, not generated);
   into `mergeRecords` returning a marker at the root;
   a `filterValues` that reads `meta`;
   leaf-kind record-likes as a root into source;
   cycle functions returning `undefined` without implicit merging, and `first`-value cycle functions;
   record-likes on the into model paths.
- Exotic inputs: `deepmergeInto` with several exotic sources;
   the `*Custom` variants on exotic inputs;
   Proxies that lie about descriptors as into targets or sources;
   exotic Map keys beyond `-0`, `NaN`, and objects;
   `WeakRef` and module namespace objects;
   foreign-realm collections nested in into targets beyond one case.
- Aliasing: no oracle for custom `maxDepth` with cycles or for into with cyclic or shared targets (no-throw only);
   FastUnsafe variants on DAGs;
   graph nodes with getters, hidden keys, null prototypes, symbol keys, or containers as Map keys;
   deep graphs (embargo).
   The mutation differential's six categories never draw a drop-all filter at a key the target lacks (mutant 511).
- Declared types: the AST has no generic, conditional, or function types (only `Partial` and `Required` wrappers),
   and calls cover `deepmerge`, `deepmergeCustom`, `deepmergeFastUnsafe`, and `deepmergeInto` only.
- Scale: cyclic inputs (embargo), width with many inputs, custom functions at scale,
   the CJS build, Bun and Deno, and non-default stack or heap limits.
- Mutation: TypeScript types, higher-order mutants, mutators outside Stryker's defaults,
   the sidecar's own oracles, the 17 or 18 Stryker timeouts, and upstream's rollup build.

## After upstream responds

- When a fix ships in a release:
   bump the catalog entry,
   let the matching known-defect test fail,
   delete it,
   re-include its excluded generator region,
   regenerate the type corpus,
   and refreeze the coverage baseline.
- When the advisory is published (or 90 days pass without response):
   move the `*.local.*` embargo test into a committed known-defect test
   and lift the depth caps on public cycle generators.
- Rust unified-linter merge (Q13):
   once the merge-crate vet lands,
   export `src/json-case.ts` as a fixture file.

## Widening pass

### Why

The first pass reported 3 security findings, 2 runtime defect root causes (one with 3 symptoms),
2 intent questions, 1 docs request, and 9 unsound plus 2 imprecise result types.
The user called that absurdly low.
Assessment:
the fuzzer itself found only the `deepmergeInto` first-value defects;
everything else came from source reading, hand probes, and hand-written type probes.
After each known-defect region was excluded,
the campaign's quiet rounds only measured narrow generators,
so they were never evidence of absence.
Correction recorded in `AGENTS.md` rule `QIV`
(validate generator reach before trusting a null or count; list unexercised surfaces),
commit `0d39b4b7a`.

### Unexercised surfaces at the end of the first pass

- Customization:
   custom merge functions,
   `actions.skip` and `actions.defaultMerge`,
   `enableImplicitDefaultMerging`,
   custom `filterValues`,
   `metaDataUpdater` and `rootMetaData`,
   custom `mergeCircularReferences`,
   and all four `*Custom` entry points.
- Input kinds:
   `isRecord` fallback branches (still uncovered in the fork source report),
   cross-realm objects,
   Proxies,
   typed arrays,
   boxed primitives,
   subclasses of Array, Set, and Map,
   non-writable, sealed, and setter-only `deepmergeInto` targets.
- Aliasing and cycles:
   shared containers (excluded after the first finding),
   target and source aliasing in `deepmergeInto`,
   cycles through Maps, Sets, and arrays.
- Declared types:
   the type corpus uses inferred literals only,
   while every type defect came from declared unions, optionals, and index signatures.
- Scale:
   record width, array length, argument count, per-level copying, superlinear time.
- Test strength:
   no mutation testing of upstream's suite or the sidecar.

### Forks launched 2026-09-24

Five parallel forks,
each owning new files only under a prefix in `package/module/deepmerge-ts.fuzz/src/`
and forbidden from editing shared files or running `format:oxlint` (package-wide rewrite);
shared-file changes come back as requests in their reports.
Security candidates go only to `doc/handover/deepmerge-ts-embargo-<area>.local.md`.

- Custom options:
   prefix `options-`,
   known defects in `known-defect-options.unit.test.ts`.
- Exotic input kinds:
   prefix `exotic-`,
   known defects in `known-defect-exotic.unit.test.ts`.
- Aliasing and cycles (cycles kept at depth 20 or less):
   prefix `alias-`,
   known defects in `known-defect-alias.unit.test.ts`,
   own fork builds under `dist/alias-fork-build/`.
- Declared-type generation:
   prefix `declared-type-`,
   new type classes in `type-known-defect-declared.unit.test.ts`.
- Scale and limits (all runs in `podman --memory=2g --cpus=2`):
   prefix `scale-`,
   known defects in `known-defect-scale.unit.test.ts`.

Pending, blocked by the concurrency cap:
mutation testing.
StrykerJS with the Vitest runner over upstream `src` against upstream's suite,
inside the capped container (never installed in this repo or the fork);
then each survivor applied to a build and run against the sidecar via `DEEPMERGE_FUZZ_TARGET`,
classified killed-by-sidecar, survives-both, or equivalent;
new tests under prefix `mutation-`;
report `doc/audit/deepmerge-ts-mutation-2026-09-24.md`.

### Integration checklist per fork report

- Reproduce each claimed finding against 8.0.2 before accepting it.
- Apply requested shared-file changes
   (generator options in `src/arbitraries.ts`, tasks in `mise.toml`, `README.md`).
- Run `format:oxlint`, `lint:oxlint`, `lint:types`, `test:unit`,
   then ratchet `fuzz:coverage --write`.
- Public findings into `doc/handover/deepmerge-ts-issue.local.md`;
   security candidates into `doc/handover/deepmerge-ts-advisory.local.md`
   (one combined advisory, one combined issue, per the user's constraint).
- Record new findings and counts in `Outcomes` with the surfaces still unexercised.

## Method audit

### Why

Nothing so far measured whether the search method can find defects at all.
Mutation testing measured upstream's suite and the sidecar's example tests against single-token mutants;
the campaign's 580 quiet rounds measure only its generators,
which exclude every known-defect region.
Three structural doubts remain:

- Oracle independence:
   `src/model.ts` and the accepted-behaviour tests were written after reading upstream source and probing it,
   so wherever the model copies observed behaviour, a defect there reads as the specification.
- Surface choice:
   every layer checks runtime values or result types under one tsconfig;
   docs conformance, compiler-flag and TypeScript-version variation, type-checker cost,
   packaging (exports map, CJS, declaration files), and other runtimes were never searched.
- Recall:
   no positive control for the method as a whole,
   so a quiet campaign cannot be told apart from a blind one.

### Workstreams launched 2026-09-24

- Historical recall (prefix `recall-`, report `doc/audit/deepmerge-ts-recall-2026-09-24.md`):
   bundle upstream `src` at the parent of each past bug-fix commit that fits the current API,
   run the sidecar and a bounded campaign against it through `DEEPMERGE_FUZZ_TARGET`,
   and classify each miss (generator reach, excluded region, oracle copies the bug, surface never checked).
- Unsearched surfaces (prefix `surface-`, report `doc/audit/deepmerge-ts-surface-2026-09-24.md`):
   every upstream docs example run literally against its stated result,
   result types under strictness flags and the supported TypeScript range,
   type-checker cost for wide, deep, and many-argument merges,
   packaging checks, and CJS consumption.
- Clean-room oracle (prefix `cleanroom-`, report `doc/audit/deepmerge-ts-cleanroom-2026-09-24.md`):
   a fresh agent without this session's context writes expected results from upstream docs alone,
   then runs them against 8.0.2,
   so expectations cannot inherit the model's reading of the implementation.

### Clean-room result

Report `doc/audit/deepmerge-ts-cleanroom-2026-09-24.md` (commit `c86573cfd`):
1329 runtime cases, 35 type expectations, and runtime, harness, mutation, and type controls that all failed as required.
Its 3 defects are known:
M1 and M2 are issue section 2
(a class instance's own fields are copied like the `Date` symptom, added there;
a plain `new Error("m")` copies nothing here because `message` is not enumerable),
M3 is issue section 12.
Its embargoed item is issue section 6, already public;
its `deepmerge`-then-`deepmergeInto` variant and the 8.0.0 changelog claim were added there.
Its 2 docs errors are new, reproduced on 8.0.2, and added to issue section 11:
the `deepmergeCustom` TSDoc example customizes the default,
and `ObjectType` is documented as a utility but ships only as a `const enum`.
Its ambiguities (null-prototype records, root `actions.skip`,
into root kind change) are issue sections 4 and 5 or accepted.
Reading:
an oracle written from the docs alone found no runtime defect the sidecar had missed,
which is recall evidence for runtime values against documented behaviour,
not for the surfaces the other two workstreams search.

### Recall result

Report `doc/audit/deepmerge-ts-recall-2026-09-24.md` (commits `bb05f7b58` to `95f6fc976`):
of 22 historical bugs whose control reproduced them, the sidecar at `e4237dc8f` detects 16:
runtime 8 of 8 (all in `test:unit`, none needed the campaign),
result types 8 of 10,
packaging and metadata 0 of 4.
Every miss is on a surface the sidecar never loads
(exported utility types, `utils.defaultMergeFunctions` in custom functions, `node10` resolution, `engines`);
none came from generator reach or a model copying the bug.
Upstream shipped only 8 runtime fixes ever, so this bounds recall for the fault classes upstream has shipped,
not the count of unknown faults.
New 8.0.2 finding, reproduced independently here (pin with the directive removed reports TS2589,
and the issue snippet type-checks verbatim in `src/issue-snippet-union-depth.local.ts`):
with `DeepMergeNoFilteringURI`, a 50-member leaf union exceeds instantiation depth;
added to issue section 12.
The same edit dropped a work-inviting offer from section 12 (`XCM`).

Its proposals, split by owner:
generator changes (zero-argument calls, `undefined` between 3+ records, 50+ member unions, more declared-type seeds)
are applied from this session because no other workstream touches those files;
exported-type, documented-example, packaging, and `engines` checks wait for the surfaces report,
which covers the same ground, so both land in one integration pass.

### Surface result

Report `doc/audit/deepmerge-ts-surface-2026-09-24.md` (commits `4d8ca3694` to `02c6e8c62`):
every documented runtime result held,
and packaging (publint, attw 0.18.5 with a control), Node 16.9 to 26, CJS, Bun, and Deno found nothing.
Reproduced independently here before entering the drafts
(runtime and types in `src/verify-surface.local.ts`, the embargoed repro in a capped container,
200-argument TS2589 in a scratch file;
the docs-snippet type errors rest on `src/surface-docs-type.unit.test.ts`,
whose `@ts-expect-error` pins pass `lint:types`):

- Issue section 11 (docs):
   `ObjectType` consequences per toolchain,
   four documented TypeScript snippets the declarations reject,
   `objectHasProperty`, the FastUnsafe circular statement, and FastUnsafe custom `meta`.
- Issue section 12 (types):
   spread after a fixed argument, interface-typed inputs, `ReadonlySet`/`ReadonlyMap` views,
   and two questions (`strictNullChecks: false` filters `null`; argument-count cost about N^2.6 with TS2589 at 200).
- Advisory finding 2:
   the documented `metaDataUpdater`s also disable cycle handling (two cyclic inputs exhaust the stack).

### Integration after the three reports

- Generators:
   zero-argument and `undefined`-between calls (`src/argument-arbitraries.ts`, commit `9ff6f5c3f`);
   both rebuilt recall bugs now fail the model property directly.
   The shifted draws dropped coverage to 1763 (`src/deepmerge.ts` 302 to 303, the custom array fallback);
   parallel arrays and an `implicitArray` reach tally restored 1765 by construction (commit `204f9f08b`).
- Exported types:
   `src/type-export.unit.test.ts` references all 31 exported types,
   and `src/type-export-name.unit.test.ts` fails when the declaration file exports one it does not (commit `2e0b605bf`).
- Default-function delegation (`src/type-default-function.unit.test.ts`) found a new declaration defect:
   `utils.defaultMergeFunctions.mergeArrays`, `mergeSets`, and `mergeOthers`
   reject the `utils` their optional parameter declares (TS2345);
   pinned there and added to issue section 12.
- Literal-union cost families (`src/surface-type-cost-family.ts`, commit `5db24c179`):
   default, FastUnsafe, into, and three-input paths stay near-linear to 400 members;
   no-filter and custom filter URIs grow about quadratically
   (1420365 instantiations and 22 s at 400) with TS2589 from 50;
   added to issue section 12.
- Tasks `recall:*` and `surface:*` with README entries (commit `26ac6cedf`, each run for real);
   `recall-setup.ts` moved to async APIs; the recall report counts each id once (commit `f4db75cc9`).
- Checks after integration:
   `format:oxlint`, `lint:oxlint` (0 warnings, 0 errors),
   `lint:types`, `test:unit`, and `fuzz:coverage` (1765 of 1772) pass.

Method verdict:
runtime search has measured recall (8 of 8 historical runtime bugs,
and a docs-only oracle found no runtime defect the sidecar missed),
so the low runtime count reflects the library, within the fault classes measured.
The misses and most new findings were on surfaces never searched before: result types under real consumer shapes,
declarations of the customization API, documentation, and type-checker cost.

### Campaign

The run started 2026-09-24 had passed 363 rounds without a counterexample before the widening pass;
it re-reads property files each round,
so forks' new property files join it automatically,
and a failure caused by a half-written file is triage noise, not a finding.
