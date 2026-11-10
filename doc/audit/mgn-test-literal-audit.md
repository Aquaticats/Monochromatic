# MGN test literal audit

Audit of the test suites against rule `MGN`,
from GitHub issue #3371:
tests never assert magic numbers or magic lists,
and derive expectations from the named constant,
the input,
or the relation under test.

The audit ran over every package under `package/` in nine scopes,
fixed the violations it found,
mutation-checked every rewrite,
and left the remaining candidates in place with the reason they are not violations.

## Method

A pattern search finds candidates,
not violations,
so the audit ran in three passes over each package.

1.  Search the test and sidecar files for the four violation shapes named in the issue:
    expected values that repeat a production constant,
    default,
    limit,
    timeout or count;
    expected lists that repeat a production list;
    sums or products of literals that mirror production arithmetic;
    counts of a collection the code defines.
2.  Pair each candidate literal mechanically with the production constants in the same package
    that carry the same value,
    so a hit pointed at the constant it might copy.
    The pairing is a reading aid,
    not a verdict.
3.  Read every hit against the four classes and the issue's not-a-violation list,
    fix the violations,
    and record why each remaining candidate is not one.

Each fix was verified by changing the guarded production value or list,
running the narrow test that covers the rewritten assertion,
and confirming it fails before restoring the value,
so no fix silently weakened a test.
Each touched package's own task was then run through `mise run`,
the exact commands and results sit in the area reports and commit bodies.

## Coverage

The in-scope inventory is about 4,000 test files across 211 package directories:
2,868 TypeScript test files,
985 Rust test files,
142 Kotlin test files,
and the `.fuzz`,
`.bench` and `.conformance` sidecar packages.

The scopes were:

- `package/git-policy/`:
  the native wrapper and its fuzz sidecar,
  where deadlines,
  lock waits,
  budgets and registry tables live.
- `package/linter/` and `package/forbidden-strings/`:
  the linter and the scanner.
- `package/crate-release/`,
  `package/mpm-shadow/` and `package/monorepo-task/`:
  toolchain and release tooling.
- `package/translation-repair/`:
  the largest suite,
  subpackage by subpackage.
- `package/music-player/`:
  the desktop app,
  its end-to-end package and the truepeak crates.
- The agent and plugin tooling packages:
  `pi-plugin`,
  `agent-harness-shared`,
  `test-harness`,
  `llm`,
  `dsh-plugin`,
  `mcp`,
  `config`,
  `logger`,
  `const`,
  `css`,
  `terminal` and their siblings.
- The remaining dev tooling and app packages:
  `wg`,
  `nested-wayland-session`,
  `dependency-catalog`,
  `jsonc-edit`,
  `toml-edit`,
  `upstream-fork`,
  the webapps,
  the editor backends,
  the shims and the release helpers.
- `package/claude-code-plugin/` and `package/ide/app/`,
  the last two package directories.
- The Kotlin tests under `package/music-player/android-app` and `package/linter/kotlin`,
  and the evidence scripts under `package/music-player/design`,
  which the TypeScript and Rust
  sweeps cannot reach.

Every TypeScript and Rust test file and sidecar was inside exactly one scope,
and every candidate hit from the issue's patterns and the constant cross-check in that scope
was read and classified.
The largest long-tail candidate lists were triaged by class rather than read line by line,
and each area report names the lists where that happened.

Excluded,
with reason:

- `package-paused/`:
  78 test files in paused apps that have no `mise.toml` task root
  and are excluded from the root test glob,
  so no task can run them.
- `package-deprecated/mcp/nvim`:
  4 test files,
  excluded from the task graph by
  `config_roots` in the root `mise.toml`.

## Violations fixed

The audit landed 60 commits touching 339 files,
each with `Part of #3371` in the body,
grouped by area:

- `package/git-policy/cli` (34 files):
  the set-deadline lock wait now derives from the deadline plus `STOP_GRACE` and `LOCK_WAIT_MARGIN`,
  retry pauses and the budget assert their doubling relation and sum,
  the wait defaults and scratch bound assert relations,
  forwarding gained `SIGNAL_EXIT_BASE` for its exit code and the LFS signal test,
  and the pins for push output,
  script inspection,
  alias expansion,
  control spellings,
  conclusion state files,
  recovery sizes,
  SHA lengths,
  exit codes and `MAXIMUM_CHANGED_PASSES`
  derive from their constants.
  A review round made the conclusion state table bidirectional and restored a name guard
  a first rewrite had dropped.
- `package/linter/monochromatic-lint` (33 files):
  diagnostic codes,
  the Rust default line limit,
  shell defaults,
  `CONSTRUCTS` and `RULE_IDS`
  lists,
  rule capability relations,
  `MAX_PASSES`,
  worker stack size,
  closure rules and the
  documented directive codes.
  A review round made the capability pins bidirectional per `RULE_IDS` row.
- `package/forbidden-strings/`:
  the Sheng state and class ceilings,
  the group-unroll limit,
  the repetition cap,
  the cache path layout and the fuzz token bound.
- `package/crate-release/cli` (21 files before and after the concurrent redesign merge):
  exit statuses,
  program names,
  commit type,
  manifest and lock names,
  the crates.io base,
  poll arithmetic,
  wait bound,
  workflow names and inputs,
  runner host,
  registry file names,
  git argument tables and the commit message shape.
- `package/mpm-shadow/cli` (15 files):
  usage and not-found exit codes,
  the signal offset,
  lock schema,
  digest prefix,
  platform list and count,
  GitHub page size and cap,
  scope file names,
  cargo layout paths,
  plan and selection counts and the valid-id list.
- `package/monorepo-task/core-exec`:
  the Playwright tag length and image names,
  the fake podman platform and the stale-leftover age.
- `package/translation-repair/`:
  provider and provider-stream deadlines,
  pace windows,
  daily allowance and energy margin;
  shared-contract card rows,
  provider order,
  nesting bounds and the roster check;
  corpus-pass,
  page-check and consolidate schema names;
  markup and document-structure bounds;
  the run subpackage deadline.
- `package/music-player/desktop-app` (45 files,
  about 130 assertions):
  indicator timing and percent caps,
  the double-Shift window,
  key-hint hold,
  taskbar gap,
  position interval,
  layout pixels,
  text-size limits,
  feedback durations and messages,
  first-run labels,
  search tiers and limit,
  template nesting,
  job titles and terminate codes,
  and notification urgency and expiry.
  The truepeak crates and the end-to-end package derive their shipped policies,
  WAV header
  fields,
  report columns and fallback names the same way.
  One audit commit captured 12 production sources while a mutation check had them edited;
  the corrective commit restored them and the test fixes and their coverage are unaffected.
- The agent and plugin tooling packages:
  thresholds,
  severity bands,
  byte-unit sizes,
  exit statuses,
  lookback windows and exported registry lists.
- The dev tooling and app packages:
  removal thresholds,
  resolver flag bits composed from the
  named bits,
  generated cause lists,
  retry backoff,
  render limits and release floors.
- `package/claude-code-plugin/source` and `package/ide/app`:
  message tail,
  act probability,
  collapse threshold,
  block hash length,
  container bounds,
  launch arguments,
  find diagnostics,
  crate wrap width,
  minimap seams,
  raster limits,
  row and wrap geometry,
  tab stops and the double-Shift gap.
- `package/music-player/android-app` Kotlin tests (13 test files,
  about 130 assertions):
  settings,
  focus-ring,
  folder-rail,
  track-template,
  search-layout and search-page values now
  derive from their named sources,
  with six production constants widened from private to
  internal and no value or behavior change.
  The Kotlin sweep covered the app module's numeric and string constants,
  its size and list
  assertions and its embedded prefixes;
  `package/linter/kotlin` and the
  `package/music-player/design` evidence scripts needed no change,
  the first because its
  counts are of test-built fixtures and the second because the scripts test disposable
  evidence generators rather than app production code.

## Candidates that are not violations

These candidate classes stay as written,
because the literal is the input or the specification of the case,
or the number is the behavior under test:

- A literal that is the input of the case,
  such as the seconds passed to a duration conversion
  whose result the test then asserts,
  a manifest fixture,
  or a command-line argument under
  construction.
- A specification literal with no named constant,
  such as an RFC test vector,
  a documented wire format,
  or an expected rendered string.
- A boundary case built from the named constant plus or minus one.
- A count that is the behavior under test,
  where the collection is built in the test and its length is what the case checks.
- A fixture name or a hostile-input string chosen to exercise escaping,
  which no production constant owns.
- A documented external name or value,
  such as a published image name,
  an environment
  variable name,
  a wire header name,
  or a command name.
  This was ruled a non-violation:
  the literal is the specification of the external contract and deriving it from the
  constant would make the test tautological.
- A decision or registry pin whose subject is the accepted value itself,
  such as a provider kept off or a set of writer phases.
  Changing that value is a decision
  that should break a test,
  so the literal stays and the area reports record the reason.
  Where the same decision is also duplicated in a human-owned decision doc or README,
  deriving the guard from that declaration belongs to issue #3175 and was left there.
- A completeness proof that enumerates a registry variant list to prove the built table holds
  every variant;
  deriving it from the table would weaken the proof.
- Value coincidences from the mechanical pairing,
  such as a date fixture whose digits match
  an unrelated limit.

## Verification

Every area recorded mutation checks (GFP):
the guarded production constant or list entry was
changed,
the narrow test failed,
the value was restored,
and the restored run passed.
The per-area reports record each mutation with its command,
the observed failure and the
restore confirmation.

Package test tasks were run through `mise run` for every touched package.
As an independent check the Lead re-ran five package tasks on the final tree:
`crate-release/cli:test:unit` after the merge,
`translation-repair/provider:test:unit`,
`claude-code-plugin/source:test:unit`,
`forbidden-strings/core:test` (235 passed,
1 skipped),
and `linter/monochromatic-lint:test` (721 passed,
2 skipped).

The Kotlin audit ran the Android JVM unit suite headless in 136 classes with 1385 tests and no
failures,
after pointing `java.io.tmpdir` at a writable directory (the package task alone fails
154 tests while extracting the native library into the default temp dir),
and ran
`package/linter/kotlin:test` with the RequireKDoc suite green.

Surfaces no task could run here,
so those guards carry static evidence only:

- The 7 instrumented tests under `package/music-player/android-app`:
  they need an Android
  emulator,
  and headless emulator launches are banned.
- `package/music-player/desktop-app:test:e2e`:
  needs a release build driven in a nested
  Wayland compositor with a private PipeWire sink.
- `package/llm/image-diff`:
  expensive suite,
  and `package/logger/core` IndexedDB browser suite.
- The `.github/workflows/` guards reached from `crate-release`:
  those files sit outside the
  package scope,
  so their guards were not mutated;
  the constants they derive from were.

## Known exceptions

- `package/dsh-plugin/cost-dismissal` holds the one unfixed violation:
  its implementation `index.js` is ignored by the root `*.js` rule and is not tracked,
  so the named constant that the fix needs cannot be committed.
  The package was reverted to HEAD and needs a repository-owner decision
  (a gitignore negation and a forced add,
  or moving the implementation to a tracked source).
- Pre-existing failures unrelated to this audit,
  proven present at the parent commit:
  the `GIT_POLICY_NATIVE_TEST_LINTER` environment contract test in `package/git-policy/cli`;
  `package/ide/app` rustfmt drift on 14 files and one clippy finding in another session's file;
  `wg/cli-quicker` tests that reference the TypeScript `file-enforcer` path a concurrent
  change deleted.

## Follow-up

Review keeps rejecting a literal copy of a production value at review time (rule `MGN`),
and a new test derives its expectation from the start.
Deriving guards from a human-owned canonical declaration remains issue #3175's scope.
