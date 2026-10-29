# Split module-test into responsibility-focused packages

## Status

Implementation authorized by the user's explicit instruction to proceed.
The four-package layout is implemented.
All affected packages build,
pass package lint and type checks,
and pass their unit suites.
Runner browser acceptance,
public-export checks,
diagnostic negative controls,
and frozen-lockfile validation passed.

## Confirmed direction

- Maintainability and independent consumption are the goals.
  No ordering between those goals was specified.
  Dependency minimization is not a selected goal.
- Preserve observable test behavior during extraction.
  Matcher semantics,
  concurrency,
  timeout handling,
  cleanup,
  rejection attribution,
  and diagnostic content remain intact.
- Split into as many cohesive packages as make sense.
  Assertions should be extracted.
  Do not turn extraction into a questionnaire about enforcing new consumer contracts.
- Keep `module-test` as the runner with convenience re-exports,
  the user's Q6 A answer.
- This is not an `AGENTS.md` change request.
  No agent-policy or skill edit is proposed or authorized by this task.

## Interview correction

The second round over-framed physical decomposition as adoption of independently supported public contracts.
The user explained that the assertion answer changes which other questions should be asked at all.
That correction supersedes the earlier standalone-contract menus.
Q4,
Q5,
and Q7 are withdrawn rather than treated as unanswered blockers.

The new question is whether the concrete responsibility-based layout matches the intended split.
It does not ask the user to choose contract-enforcement infrastructure,
a new executor abstraction,
or an external publication policy.

## Proposed layout

Package names follow their paths under `package/`,
with the `@monochromatic-dev/` scope.
New siblings retain the `test-` prefix to show their relationship to the harness.

### package/test-harness/core-expect

Package name:
`@monochromatic-dev/test-harness-core-expect`.

Owns runtime assertions and assertion tracking:

- `expect.ts`.
- `expect-matchers.ts`.
- `expect-matchers-core.ts`.
- `expect-matchers-collection.ts`.
- The `expectTypeOf` re-export and assertion-related types.

Keep the complete current matcher behavior,
including asymmetric and Sinon-spy matchers.
Keep Chai,
`chai-as-promised`,
`sinon-chai`,
and Sinon where the current implementation uses them.
The package is a responsibility split,
not a dependency-purity redesign.

### package/test-harness/core-sandbox

Package name:
`@monochromatic-dev/test-harness-core-sandbox`.

Owns disposable Sinon construction,
guarded operations,
method leases,
restoration,
shared method-ownership state,
and the owner/runtime types consumed by that implementation.
Move `sinon.ts` and every `sandbox*.ts` file except `sandbox-runtime.ts`.

Retain `createSinon` and `createOwnedSandbox` with their existing meanings.
The latter already accepts an owner and a runtime adapter.
The runner supplies that adapter;
the sandbox package must not import the runner.

Do not invent a new standalone context executor or change ordinary Sinon behavior.
Do not move timeout and test-attempt orchestration into the sandbox package.

### package/test-harness/core-diagnostic

Package name:
`@monochromatic-dev/test-harness-core-diagnostic`.

Owns test-error formatting,
assertion-source inspection,
and shared stack-frame classification:

- `format-error.ts`.
- `assertion-source.ts`.
- `harness-frames.ts`.

Keep these together because source enrichment and formatting share frame policy.
This remains test-oriented diagnostics,
not a new generic application-error library.
The existing `module-caught-value` package continues to own basic caught-value text and stack extraction.

Update path recognition for every affected runner,
assertion,
sandbox,
and diagnostic source or artifact path.
Keep existing harness paths recognized where they still occur;
renaming only the diagnostic path fragments would expose other harness frames.

### package/test-harness/core

Existing package:
`@monochromatic-dev/test-harness-core`.

Retains execution and integration:

- `describe.ts`,
  `it.ts`,
  and `descriptor.ts`.
- `it-attempt.ts` and `sandbox-runtime.ts`.
- `execution.ts`,
  `execution-node.ts`,
  and `execution-types.ts`.
- `rejection-report.ts` and `verdict.ts`.
- `index.ts`,
  forwarding the current convenience exports from their new owners.

The runner owns test lifecycle,
async-context attribution,
and verdict emission.
Keep those responsibilities together rather than creating another package for each helper file.

Retain the current root export surface,
including assertion-source helpers,
formatting functions,
assertion types,
and `expectTypeOf`.
The current root exports `DisposableSandbox` as a type but does not export `createSinon`;
do not accidentally add that root export during forwarding.

## Dependency direction and stopping point

The runner depends on the assertion,
sandbox,
and diagnostic packages.
None of those packages imports the runner in production.
Keep their existing shared utility and third-party dependencies with the implementation that consumes them.
Workspace imports continue to use `/ts`.

This yields four packages,
not a runner plus another facade package.
The existing runner remains the convenient assembly point.

A tracked-source inventory assigns all 35 non-test,
non-fixture TypeScript files under `package/test-harness/core/src`:

- Assertions:
  4 existing files.
- Sandbox:
  17 existing files.
- Diagnostics:
  3 existing files.
- Runner:
  11 existing files.

These are pre-extraction file counts,
not a size target or a count of future entry-point files.

No further package is proposed for:

- Individual matcher groups or assertion tracking,
  which implement the same assertion interface.
- Raw Sinon disposal separately from sandbox ownership,
  which would separate the factory from the subsystem that builds on it.
- Error formatting separately from source inspection,
  which share harness-frame classification.
- Verdict formatting or rejection reporting separately from execution,
  whose lifecycle supplies their meaning.
- `expectTypeOf` alone,
  which is an upstream re-export rather than our own subsystem.

An independent technical review found no additional cohesive package necessary
and no production dependency cycle in this proposed direction.
This is source-based design evidence,
not a claim that the unimplemented split has passed runtime verification.

## Consumers and tests

Keep existing ordinary consumers importing `module-test`.
No repository-wide import migration is required merely to prove the split happened.
Direct imports from the new packages are available when code needs that responsibility alone.
Do not change fixtures whose existing package string is itself part of the test subject.

Move responsibility-local tests with the implementation they exercise,
and use the extracted package's built interface as the test subject.
Keep cross-package runner acceptance for context-owned mocking,
cleanup,
timeouts,
repeats,
rejections,
and source/built-copy interoperability.
Browser consumer acceptance remains runner integration coverage.

Tests may use `module-test` as a development dependency of a leaf package,
as existing helper packages already do.
Distinguish that test-only back-edge from production imports.
Ensure build tasks do not acquire a cyclic prerequisite chain.
Do not introduce a new testing framework just to avoid a development-dependency cycle.

Each package gets its normal manifest,
README,
TypeScript configuration,
build configuration,
and `mise.toml` tasks.
Use the existing repository generators and build conventions rather than new enforcement machinery.

## Behavior-preservation checks

After confirmation and implementation:

- Run package lint,
  including type checks,
  for all affected packages.
- Build the extracted packages and runner,
  then exercise the built interfaces through their tests.
- Preserve both existing `Symbol.for` protocol keys:
  `@monochromatic-dev/test-harness-core/async-failure-runtime/v1`
  and `@monochromatic-dev/test-harness-core/method-ownership/v1`.
- Verify mixed source,
  neutral,
  and Node copies for attribution,
  cleanup,
  and rejection reporting.
  Unchanged keys alone do not prove this behavior.
- Exercise existing browser acceptance without broadening the runtime promises.
- Verify relocated source and built frames remain filtered
  while actual test frames and assertion expressions remain visible.
- Check root re-exports and representative existing consumers.
- Leave unrelated worktree changes untouched.

No publishing,
new contract-enforcement system,
behavior redesign,
agent-policy edit,
or library replacement is included.

## Supporting evidence

### Source seams

`package/test-harness/core/src/index.ts` provides the root export inventory.
`expect.ts` directly uses Sinon,
and `expect-matchers.ts` integrates the current Chai plugins.
`sinon.ts` provides ordinary disposable construction.
`sandbox.ts` already accepts `SandboxOwner` and `SandboxRuntime`.
`it-attempt.ts` owns timeout and disposal ordering.
`execution-node.ts` supplies contextual runtime integration using the rejection observer's storage.
`rejection-report.ts` consumes that execution metadata.
`format-error.ts` and `assertion-source.ts` share `harness-frames.ts`.

### Consumer census

A read-only scan enumerated `git ls-files -z -- package`,
parsed tracked package manifests,
and inspected tracked TS/TSX text outside `package/test-harness/core/` and `dist/`.

- 131 manifests declare `@monochromatic-dev/test-harness-core` in a dependency field.
- 976 source files contain that package-name string.
  This is a reference count,
  not an executable-import count;
  comments and fixture strings are included.
- Direct reads confirmed assertion-only imports in
  `package/pi-plugin/openai-fast/src/host-fixture-abort.ts`
  and `package/test-fixture/oxlint-test-import/case/standard/src/allowed.test.ts`.

The scan does not establish absence of namespace imports,
dynamic imports,
untracked consumers,
or consumers outside the repository.
No published-compatibility obligation is inferred from `private: false`.

### Ecosystem precedent

The inspected [Jest expect README][jest-expect] describes a separate package exporting Jest's `expect`.
The inspected [Vitest expect manifest][vitest-expect] separates assertions while depending on
`@vitest/spy` and Chai.
These are precedents for responsibility separation,
not proposals to replace our implementation or evidence of equivalent behavior.

[jest-expect]: https://github.com/jestjs/jest/blob/main/packages/expect/README.md
[vitest-expect]: https://github.com/vitest-dev/vitest/blob/main/packages/expect/package.json

## Work areas

- [x] Confirm the proposed package layout and names.
- [x] Extract implementations,
  metadata,
  and tests while preserving runner re-exports.
- [x] Verify package tasks,
  mixed-copy state,
  browser integration,
  and diagnostic relocation.
- [x] Record implementation commits and verification outcomes.

## Final verification

- All four package builds and unit suites passed.
  `module-test` now includes a root-export inventory check for both Node and neutral entries.
- All four package lint tasks passed with zero warnings and errors,
  including their type checks.
  The moved Oxlint override's package also passed `lint:types`.
- The existing bounded browser task passed all 6 tests across Chromium,
  Firefox,
  and WebKit.
- In a detached throwaway worktree,
  the relocation test passed with the new frame fragments,
  failed in all 6 extracted-package cases when those fragments were removed,
  and passed after restoration.
  The 2 original-runner controls continued passing during the negative run.
- A neutral-only diagnostic entry selected the neutral filesystem adapter under Node.
  The new Node-prefix test reproduced the resulting unstripped workspace prefix in the throwaway worktree.
  A conditional Node artifact restores filesystem-backed root discovery and passes that test.
  Neutral artifacts remain available for browser consumers.
- `mise run prepare:pnpm:install -- --offline --ignore-scripts --frozen-lockfile --dry-run` passed.
  The initial generated task delta added only the new importers and moved runner dependency edges.
  Concurrent commit `f1bca0541` included that generated lockfile alongside its unrelated work;
  this session did not stage unrelated lockfile changes.
- Independent final review found no remaining semantic,
  bundling,
  or test blocker.
  Historical source references receive dated relocation notes rather than being silently rewritten.

Implementation commits include `79a3565fc`,
`2aea88562`,
`feff926c0`,
`8a02c2724`,
and `d5a267175`.
The detached verification worktree was removed after its temporary CLI trust was revoked.
One `config-oxlint` type-check encountered concurrent Rolldown edits;
the later complete check passed without this task modifying that concurrent source.

## Document verification

The new package READMEs,
this plan,
and the commit-policy incident record pass scoped Markdown lint.
The initial `semantic-line-breaks` findings after label colons were corrected.
All 16 changed Markdown documents were rendered in full through the installed Satteri renderer.
The verification checked rendered package links and relocation code spans,
including the historical documents too large for convenient inline inspection.
Earlier interview revisions also passed GitHub Markdown rendering checks.
Implementation began after explicit approval.
Commit `79a3565fc` extracts the packages.
The first commit attempt hit a `git-policy-cli` engine failure in `mono/dependent-version-bump`;
the documented one-invocation policy escape succeeded,
and the following test-correction commit succeeded without an escape.
No policy source or trust setting was changed.

The first sandbox factory test used an incomplete `SinonSandboxConfig` and incorrectly expected
local detached-fake configuration to be forbidden after completion.
Installed Sinon types require all configuration fields;
`sandbox-fake.ts` explicitly permits local behavior/history changes while guarding descriptor changes.
The test now supplies the full configuration,
checks rejection of `fake.value`,
and verifies `fake.returns` cannot reattach the restored target.
No production behavior changed for this correction.

The first extracted-assertion lint identified the existing variadic-matcher allowlist still naming the old paths.
Move those exact existing path entries to `test-expect`,
without widening their rule scope.
The runner lint already passed with zero warnings or errors.
Browser acceptance passed through the existing bounded Playwright task.

## Next action

No implementation work remains from the accepted split.
Report the package layout,
preserved imports,
and verification results.
No package was published and no `AGENTS.md` file was edited.
