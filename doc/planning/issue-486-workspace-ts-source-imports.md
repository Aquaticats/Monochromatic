# Issue #486: do oxlint and tsc handle workspace `/ts` source imports?

Grilling session record,
 started 2026-09-06.
Proposal state only;
no decision is ratified until the user accepts one.
The decision,
 once made,
 lands in `doc/decision/workspace-ts-source-imports.md` and rewrites rule ST3 in `AGENTS.md`.

## The convention under question

Rule ST3 in `AGENTS.md`:
cross-package workspace imports use the package's `/ts` subpath,
which resolves to TypeScript source,
never built output.
The convention entered `AGENTS.md` in commit `e5bc11cbc` (2026-07-13).
Its rationale is recorded in `.out-of-scope/typescript-project-references.md` (issue #123,
 closed 2026-05-14):
tsgo reads sibling source through `./ts` exports,
so cross-package type checks stay fast without emitted declarations.

## Trigger (user, round 2)

The performance of the project-owned oxlint rule package `oxlint-plugin/prefer-readonly-parameter-type`.
The user's framing:
the performance case for `/ts` was made when the repo was smaller,
and it has since been seen to hurt.

## Measurements (2026-09-06, this machine)

- Package count at issue #123's close (2026-05-14): 95.
  Today: 151.
- Per-package `tsc --listFiles` over 148 packages with a `tsconfig.json`:
  median 95 sibling workspace source files per program;
  39 packages hold more than 100;
  122 hold more sibling files than their own.
  Largest:
   `config/oxlint` (394 sibling,
   14 own),
  `vm-builder/cli` (203 sibling,
   5 own),
  `mcp/mvm` (176 sibling,
   11 own).
- `mise run //package/monorepo-task/core-exec:lint:types`:
   0.46 s wall for a 126-file program (31 own).
- `mise run //package/monorepo-task/core-exec:lint:oxlint`:
   3 errors,
  all `typescript(no-useless-empty-export)`,
  unrelated to `/ts`.
- Cross-package `/ts` import sites: 1789.
  Packages exporting `./ts/*`: 61.
  Package tsconfigs declaring `references`: 0.
  Packages extending the non-dom base config: 8.

- Whole-repo warm `mise run lint:oxlint` (2026-09-06,
   single run,
   one oxlint invocation):
  114.4 s in oxlint over 3075 files,
  115.6 s wall,
  19477 warnings and 1340 errors.
  Run-to-run band recorded earlier for this sweep:
   about 4.6 s.
- Warm single-package `lint:oxlint`:
  `mcp/mvm` 1.1 s (11 files,
   176 sibling sources in its program);
  `pi-plugin/auto-mode` 4.8 s (66 files,
   148 sibling sources).
  Both under the 10 s package target;
  the cost that exceeds target is the sweep.
- Persistent rule cache `node_modules/.cache/prefer-readonly-parameter-type`:
   72 MB.

- Rule liveness (positive control,
   2026-09-06):
  the warm sweep output holds 23 `prefer-readonly-parameter-type(prefer-readonly-parameter-types)` findings,
  so the rule runs repo-wide and the repo is close to remediated.
  `file-enforcer` and `mcp/mvm` report none.
- `file-enforcer` warm `lint:oxlint` with the readonly rule allowed via `--allow`:
   1.3 s,
  against 1.4 s with it on;
  warm per-package cost of the rule is inside the noise here.

- Warm sweep with the readonly rule allowed (`--allow prefer-readonly-parameter-type/prefer-readonly-parameter-types`,
  wrapper run from the root the way the task runs it):
   65.3 s over the same 3075 files,
  against 114.4 s with the rule on.
  Confound:
  the `config-oxlint` bundle was rebuilt between the two runs and `stylistic(require-asterisk-prefix)` fell from
  15520 to 322 warnings,
  so a control run of the same wrapper command without `--allow` is being taken before the 49 s is attributed.

- Control sweep (same wrapper command,
   rule on,
   after the config rebuild):
   154.2 s on 3080 files,
  4273 warnings and 1327 errors.
  Both this and the 65.3 s rule-allowed run happened while the worktree spike was installing and building on
  the same machine,
  so the rule's share sits somewhere between 49 s and 89 s of the warm sweep until a back-to-back pair is
  measured on a quiet machine.

- Back-to-back warm sweep pair on a quiet machine (wrapper from the root,
   16 threads,
   3086 files):
  rule on 63.6 s and 63.4 s;
  rule allowed 64.8 s and 65.6 s;
  identical 4273 warnings and 1327 errors in all four.
  The rule's warm share of the sweep is inside the noise,
  and the earlier 114.4 s and 154.2 s runs were inflated by the research and spike agents running alongside.
  The warm sweep sits at the 60 s target within the recorded 4.6 s band.

## Evidence already in the repo

- `doc/planning/oxlint-warm-sweep-attribution.md` (issue #374,
   target:
   warm whole-repo `lint:oxlint` under 60 s):
  warm sweep 184 s,
  171 s of it inside `prefer-readonly-parameter-types`;
  cold 645 s;
  per-file `Program` visitor 82.8 ms warm across 2080 visits;
  85 percent of the rule's warm time sits inside `Program` outside every instrumented span
  (opening the semantic file session and verifying each callable).
  The document does not attribute that time to sibling source size;
  that attribution is a hypothesis,
   not a measurement.
- `doc/troubleshooting/oxlint-prefer-readonly-incremental-cache.md`:
  the rule's semantic project held 834 source files for a 13-file lint target;
  the effect-summary index retains every non-declaration workspace source in the project,
  so the rule's workload is bounded by the transitive `/ts` closure,
  not by oxlint's target count.
- `package/oxlint-plugin/prefer-readonly-parameter-type/README.md`:
  "Workspace calls resolve through repository source and are analyzed live";
  locked package calls resolve through package exports to shipped implementation on demand.
- `doc/troubleshooting/typescript.md`:
  a consumer's tsconfig type-checks sibling source under the consumer's `lib`,
  which forced every package onto the `/dom` config.
- `doc/troubleshooting/rolldown-plugin-dts-typescript-7-generator.md`:
  bundling sibling `/ts` source broke declaration generation under TypeScript 7;
  worked around with rolldown-plugin-dts 0.27.4 plus explicit Oxc.
- `package/oxlint-plugin/test-import/README.md`:
  oxlint's native import restriction cannot tell own `/ts` from sibling `/ts`,
  hence the custom rule.
- `doc/decision/npm-publishing.md`:
  `./ts` is stripped at publish.

## Precedent (research, 2026-09-06)

`doc/research/typescript-monorepo-cross-package-imports.md` surveys Vue core,
Vite,
Vitest,
typescript-eslint,
Effect,
Babel,
Sentry,
Nx,
Turborepo,
and JSR at pinned commits.
Patterns found:

- Single root program with `paths` to source (Vue core,
   Vitest):
   sibling source checked once;
   lint not type-aware.
- Custom source condition plus compiled default (Vitest,
   Babel,
   Nx):
   editors,
   bundlers,
   and tests opt into source;
  checking is one root program or references.
- Source exports in-repo,
   compiled on publish (Effect):
   checking by references.
- Per-package programs over sibling declarations (typescript-eslint,
   Effect,
   Babel,
   Sentry,
   Nx):
  type-aware lint reads declarations;
  Sentry runs oxlint `--type-aware` per package after `^build:types`.
- Compiled-only with build ordering (Vite `plugin-legacy`,
   Sentry runtime).
- Per-package programs over source:
   only Vite's intra-package overlap and this repo's `/ts` subpath;
  no surveyed project documents choosing it for cross-package imports.

Every surveyed repo with type-aware lint (Sentry,
 typescript-eslint,
 Babel) has that lint read sibling declarations
and orders lint after a declaration build.

## Rejected

- TypeScript project references plus `emitDeclarationOnly`.
  Rejected by the user again on 2026-09-06 without investigation;
  recorded in `.out-of-scope/typescript-project-references.md`.

## Answers so far

- Trigger:
   prefer-readonly performance (round 2).
- Isolation costs (sibling source checked under the consumer's config,
   sibling errors in every consumer):
   not decided yet.
- Deliverable:
   decision doc,
   ST3 rewrite,
   and a real body on #486.
- New `AGENTS.md` rule to check `.out-of-scope/` and `doc/decision/` before proposing a direction:
  not added yet;
  tracked as issue #487.

## Round 3 answers

- Lever:
  the user asked how other large TypeScript monorepos handle cross-package imports before choosing;
  research delegated to a background agent on 2026-09-06.
- Acceptance target:
  both;
  warm whole-repo `lint:oxlint` under 60 s (issue #374) and a single-package `lint:oxlint` under 10 s warm.

- Issue disposition (round 3):
  edit #486 in place once the decision doc lands.

- Lever (round 4):
  measure first.
  Swap one package's sibling `/ts` imports for built declarations and dist in a throwaway worktree,
  run the rule warm twice both ways,
  and compare before choosing between the compiled boundary and a rule-only boundary.
- Coverage trade (round 4):
  once sibling calls are not analysed live through source,
  the rule infers their effects from the sibling's built dist,
  not by treating them as opaque.

- Performance premise (round 5):
  the user chose to extend the measurement before ruling on the `/ts` performance case:
  two more large-closure packages,
  and a cold whole-repo sweep with the rule on versus allowed.
- Isolation costs (round 5):
  accept and record;
  the `/dom`-everywhere convention stays the remedy,
  and sibling type errors keep surfacing in consumers.

- Round 6:
  prototype content-keyed,
  cross-project sharing of sibling effect summaries in a worktree and measure the cold sweep (yes);
  no repo-wide flip to built declarations;
  the convention stays:
  keep `/ts`,
  and the decision doc must make the positive case with prior art.

## Open questions

- Which lever:
   change the import convention,
  change only the rule's boundary for sibling packages,
  or measure first by swapping one package's sibling `/ts` imports for built declarations and comparing rule time.
- Acceptance target:
   adopt issue #374's 60 s warm sweep,
   a per-package latency target,
   or both.
- Coverage trade if sibling calls stop being analyzed live through source:
  infer from the sibling's built dist (build before lint) or treat them as opaque.

## Measurement design (round 4, in progress)

Subject:
 `package/file-enforcer/core` (138 own files,
151 sibling source files in its program;
main-worktree warm `lint:oxlint` 1.4 s on 136 files with 84 pre-existing `test-import` errors).
Throwaway worktree at `~/temp/agent/ts-boundary-spike-2026-09-06`.
Variant A keeps the `/ts` specifiers.
Variant B rewrites the five buildable sibling specifiers
(`module-test`,
 `module-logger`,
 `module-toml-edit`,
 `module-caught-value`,
 `module-matrix`)
to bare package names,
so tsc and the rule read the bundled `dist/final/*/index.d.mts` declarations and the dist runtime;
the bundled declarations were checked to import no sibling package.
Per variant:
 one populate run (cold,
 cache deleted first),
then three warm runs at `OXLINT_THREADS=1`,
`tsc --listFiles` composition,
three `lint:types` timings,
and finding counts by rule.
Known confound:
variant B also stops live analysis of sibling callables,
so its saving is an upper bound on the program-size effect alone.

## Spike result (2026-09-06, worktree `~/temp/agent/ts-boundary-spike-2026-09-06` at `30713b5ba`)

- Install needed `pnpm install --trust-lockfile`;
  plain and `--prefer-offline` installs failed in supply-chain verification with
  `ERR_PNPM_META_FETCH_FAIL` fetching `@jsr/std__internal` metadata from `registry.npmjs.org` (404).
- Positive control:
  with a known-invalid fixture copied into `file-enforcer/src`,
  the rule reported 13 `prefer-readonly` lines cold and warm,
  so the rule was live in the worktree.
  Without the fixture `file-enforcer` reports none in either variant;
  the rule is off for test files (`package/config/oxlint/src/overrides.ts:192`).
- Variant A (`/ts` source),
   `OXLINT_THREADS=1`:
  cold 8.1,
   7.9,
   8.1 s;
  warm 1.3,
   1.4,
   1.4 s;
  program holds 151 sibling source files;
  rule cache after a cold run 2,519,172 bytes in 122 files.
- Variant B (five siblings through built `index.d.mts` plus dist,
   73 files rewritten):
  cold 5.2,
   5.1,
   5.2 s;
  warm 1.4,
   1.4,
   1.3 s;
  program holds 11 sibling files (5 declarations,
   6 source from the types-only marker and `config/rolldown`);
  rule cache after a cold run 1,417,755 bytes in 89 files.
- `lint:types`:
   0.9 s in both variants,
  with or without a prior buildinfo.
- Findings:
   byte-identical between variants (217 warnings,
   101 errors,
   none from the readonly rules).
- Reading:
  the `/ts` closure changes only the cold path,
  by 2.9 s of 8.1 s for this package (spread 0.2 s and 0.1 s);
  warm cost is unchanged.
  Because variant B also stops live analysis of sibling callables,
  2.9 s is an upper bound on the program-size effect.

## Spike 2 result (2026-09-06, worktree `~/temp/agent/ts-boundary-spike-2-2026-09-06` at `c1abe05d3`, removed after)

- Retraction:
  `--allow prefer-readonly-parameter-type/prefer-readonly-parameter-types` (and `--allow=`,
   `-A`,
   rule-name-only)
  does not disable this JS plugin rule.
  A fixture probe kept its 11 findings under every spelling,
  and a cold sweep under `--allow` still wrote the rule cache and reported 22 findings.
  Every rule-allowed timing recorded earlier in this document
  (file-enforcer 1.3 s,
   sweeps 65.3 s,
   the quiet-machine pair) measured the rule on and is void.
  Setting the rule to `off` in the root `oxlint.config.ts` does disable it (484 rules,
   0 findings).
- Cold whole-repo sweeps in the worktree (16 threads,
   3086 files):
  rule on 261.4 s and 305.4 s (the first overlapped another session's package lint);
  rule off by config 14.8 s and 13.2 s;
  warm reference with the rule on 105.3 s.
  The rule is roughly 95 percent of a cold sweep and most of a warm one;
  the August attribution still holds.
- `pi-plugin/auto-mode` (`OXLINT_THREADS=1`,
   66 files,
   333 warnings and 38 errors every run):
  cold `/ts` 21.96 s median (spread 1.43) against built 5.43 s (spread 0.72);
  warm 3.06 s against 3.16 s;
  program sibling sources 147 against 6 plus 8 declarations;
  finding sets identical;
  positive control 11 rule findings with the fixture.
- `mcp/mvm` (11 files,
   zero findings):
  cold 7.43 s against 7.36 s;
  warm 1.21 s against 1.14 s;
  sibling sources 175 against 5 plus 5 declarations;
  no difference in either direction beyond spread.
- Reading:
  warm cost never moves with the closure.
  Cold cost moves only where the subject reaches many sibling callables
  (auto-mode 4 times faster,
   file-enforcer a third faster,
   mvm unchanged),
  which matches live analysis of reached sibling source rather than program size.

- Mechanism (corrected by the prototype below):
  `effect-summary-persistent-cache.ts:79-84,140-160` keys each entry by consuming project,
  but the demand-driven analysis only summarizes reached callables,
  so redundancy across projects is about a quarter of entries (2454 entries,
   1834 distinct files,
   620 duplicates),
  not the sum of closures.
  The per-package cold term that grows with sibling source (auto-mode 22 s against 5 s) is real;
  its mechanism inside the rule is not isolated yet.

## Prototype result: environment-keyed sibling summaries (2026-09-06)

Worktree removed after;
patch at `issue-486-summary-share.patch` beside this file (4 files changed plus one new module,
 30 insertions,
 18 deletions).

- Change:
  the persistent address segment `digest(projectKey)` became `digest(analysisEnvironmentKey)`,
  a digest of the analysis root plus compiler options with the config directory relocated to `${configDir}`;
  every envelope validation kept.
  Project directories in the cache fell from 166 to 53.
- Unit tests pass (including the incremental invalidation suite);
  `lint:types` clean;
  all 23 rule findings and every non-test finding preserved;
  a sibling edit still re-summarized the edited file and rewrote its 30 dependents.
- No speedup:
  zero cross-project hits,
  because `fileListDigest` and `compilerOptionsDigest` differed for 247 of 247 shared files
  and `declarationSurfaceDigest` for 228,
  while payloads were identical for 247 of 247 and dependency digests for 246.
  Same-environment projects overwrote each other instead (188 rewrites during one warm sweep).
- Timings were not comparable:
  the sandbox cgroup caps memory at 8 GB,
  cold sweeps peaked at the cap and `tsc` children were OOM-killed in baseline and prototype runs alike
  (baseline cold 260 s and 399 s;
   prototype 299 s).
- Cache census (baseline):
   2454 entries,
  166 project directories,
  1834 distinct files,
  247 files reached from more than one project,
  620 redundant entries.
- Next step if pursued:
  replace whole-program surface validation for sibling entries with per-file dependency-closure digests,
  which were identical across projects,
  and prove soundness for augmentations and declaration surfaces first.

## Next action

None open in this record.
The decision landed (`9509d2ef1`,
 closes #486),
ST3 links it (`a5e3fb4dc`),
the `--allow` troubleshooting doc landed (`5a1624dcf`),
and the cold-term follow-up is recorded under issue #374.
