# Sandbox restorer scope for issue 582

## Status

Resolved [issue 582](https://github.com/Aquaticat/Monochromatic/issues/582).
The user confirmed the scope-only change after the grilling round.
The implementation moves the callback declaration without changing its body or execution order.
Final package lint, type checking, and rebuilt-artifact unit tests passed.
GitHub confirms the issue is closed.
No design questions or follow-up actions remain open.

## Evidence

Before the fix, `mise run //package/test-harness/core:lint` exited with status 1.
Oxlint reported one warning and no errors across 68 files:
`unicorn(consistent-function-scoping)` at `package/test-harness/core/src/sandbox.ts:89:16`.
The file moved on 2026-10-04 to `package/test-harness/core-sandbox/src/sandbox.ts`.
The named callback `restoreOrdinarySandbox` contains only `raw.restore()`.
Its captured `raw` belongs to `createOwnedSandbox`, not to `restore`.

`package/test-harness/core/src/sandbox-cleanup.ts` invokes each supplied callback
and aggregates restoration failures.
The file moved on 2026-10-04 to `package/test-harness/core-sandbox/src/sandbox-cleanup.ts`.
`package/test-harness/core/src/sinon-cleanup.unit.test.ts` exercises automatic restoration,
manual restoration and restubbing, completed-attempt guards, and combined body/cleanup failures
through the public artifact.

## Confirmed change

Move the named function declaration into `createOwnedSandbox`, immediately before `restore`.
Pass `restoreOrdinarySandbox` as the final callback after the lease callbacks.
Keep the `raw.restore()` method call, cleanup authority window, error aggregation,
and public interface unchanged.
Do not change `restoreSandboxSteps` or its signature.

The recommendation is this scope-only change rather than a broader lifecycle refactor:
it directly addresses the diagnostic without introducing a new cleanup design.
Lint suppression is excluded because the declaration can inhabit its captured binding's scope.
The review risk is accidentally changing the receiver or callback execution order;
the patch must preserve both explicitly.

## Verification and completion

- `7a1df4dce` moved the declaration and referenced issue 582 for closure.
- The first build/unit run passed, but lint reported `stylistic(max-statements-per-line)`
  on the moved one-line declaration.
  `fd6dbba98` placed its body on a separate line without changing the method call.
- Final `mise run //package/test-harness/core:lint` exited successfully,
  including `lint:types`, with zero warnings and zero errors across 68 files.
- Final `mise run //package/test-harness/core:buildAndTest` exited successfully.
  It rebuilt the Node and neutral artifacts and browser fixture before running the package unit suite.
  Cleanup boundaries, attempt lifetime, ordinary timers, injected capabilities,
  installation failure handling, and mixed harness copies passed.
- Existing behavioral tests cover the unchanged restoration paths.
  The lint baseline and final run verify the declaration-scope correction without adding a source-layout test.
- The final source diff contains only the documented callback declaration move and reference replacement.
  Unrelated concurrent changes were not included in task commits.
- `gh issue view 582 --repo Aquaticat/Monochromatic --json state,closedAt,url` confirmed closure.

Browser execution was excluded by the confirmed scope and was not run.
Building the browser fixture is not evidence of browser execution.
