# Issue #481 implementation handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/logger` is now `package/logger/core`.

## Authority and state

The user accepted implementation with
"Okay,
do it."
Implement and resolve Aquaticat/Monochromatic#481 through existing `ctx.sinon` syntax,
not suite restructuring or an exclusivity option.
Continue the tracked queue without asking the user to say continue.

All tracked tasks are complete.
Implementation,
documentation,
consumer acceptance,
and GitHub closure are verified.
No further #481 work is queued.

The [planning record](../planning/issue-481-context-owned-stubs.md) holds the accepted scope.
The [troubleshooting record](../troubleshooting/sinon-context-owned-stubs.md) holds source traces,
rejected hypotheses,
and reproducible controls.

## Implemented contract

- `it-attempt.ts` creates a fresh context,
  sandbox,
  and owner for every body attempt,
  including repeats.
  Ownership closes before cleanup and after timeout.
  This does not cancel application work.
- `execution-node.ts` reuses the realm-shared rejection observer's async storage.
  `sandbox-runtime.ts` selects it lazily.
  Browsers retain ordinary Sinon plus lifetime guards,
  not claimed browser async-context isolation.
- Own configurable writable data methods use owner-private Sinon facades.
  Property reads select by current context.
  Captured fakes keep identity;
  unowned and completed readers get the original.
- The registry and method-slot modules coordinate source and built copies in one realm.
  Assignment rejects during ownership.
  Final release restores the exact original descriptor.
  Foreign deletion or redefinition is preserved and reported.
- Contextual getters preserve their actual receiver.
  Converting a contextual data method into a setter rejects before mutation.
  Existing accessors retain ordinary stubbing behavior.
- `sandbox-fake.ts` shares owner and restoration generation with call-sequence behavior objects.
  Descriptor-changing authority retires after completion or successful restoration.
  Local fake history and ordinary behavior remain available.
- `sandbox-member.ts` discovers introduced data-method and accessor fakes,
  including function-object members.
  `sandbox-install.ts` rolls back only partial installations from the failed operation,
  retaining independent cleanup errors.
- `sandbox-result.ts` retires mock-controller generations and guards injected factories before setters receive them.
  Partial injection cannot expose an unguarded factory through an application setter.
- Ordinary fake timers,
  replacements,
  mocks,
  inherited/nonconfigurable/accessor/proxy targets,
  and whole-object operations do not gain context isolation.
  Collision preflight and lifetime safeguards still apply.
- This is not a security membrane against arbitrary reflection or direct mutation.
  Explicitly borrowed active contexts retain their own factory authority;
  do not add a current-reader equality requirement.

## Verified evidence

### Core acceptance

- `proc_bced` passed full rebuilt module-test unit tests and real Chromium,
  Firefox,
  and WebKit acceptance.
- `proc_e1c1` passed full module-test Oxlint and types.
- `proc_e422` passed the added rollback-failure fixture,
  its scoped formatting/lint,
  and package types.
- `proc_c6c6` passed full rebuilt unit tests,
  scoped lint,
  and types after the final detached-spy correction.
- Browser checks exercise a consumer of built neutral output,
  absent/partial process globals,
  repeat identities,
  completed factories,
  restoration,
  error formatting,
  and absence of the Node observer.
- Earlier `proc_a844` and `proc_9107` also passed actual browser acceptance.
  They are historical evidence,
  not replacements for the final runs.

### Routing mutation control

A throwaway worktree at `94996e25a` resolved its package self-import to its own built Node artifact.
The committed `sinon-context.unit.test.ts` passed in `proc_8267`.
Only contextual dispatch was then disabled and the artifact rebuilt.
`proc_6a49` failed with double wrapping,
foreign reader values,
and nested ownership contamination.
Restoring the condition and rebuilding passed in `proc_fc0e`.

The worktree used Node 26.7.0 from its tracked lock.
It was removed after restoring source,
checking root ignored sentinels,
and unlinking the read-only dependency links.
The worktree path was `/home/user/temp/agent/issue-481-routing.thixd4dI`.

### Logger consumer

`42de970c5` removes only breadcrumb-wrapper serialization in
`package/module/logger/src/create-logger.unit.test.ts`.
`proc_511c` passed rebuilt logger unit tests and types.
`proc_81bb` passed logger Oxlint and the combined real-browser process-shim cases.
The console-sink tests retain serialization because they also change `process.env` and `process.argv`.

### Review findings and corrections

- Raw Sinon 22.1.0 disproved the proposed `withArgs` descriptor-authority escape:
  child `get`/`set`/`value` throw `Object.defineProperty called on non-object` without changing the target.
- The corresponding `onCall` selector paths really mutated descriptors.
  Committed tests failed in `proc_46f5` before guards were propagated.
- The same red run exposed failed whole-object construction leaving unregistered partial fakes.
  The adapter now rolls them back without restoring unrelated sandbox work.
- The first rollback-failure fixture used `isSinonProxy`,
  which did not trigger wrapping failure.
  Reading `wrap-method.js` identified `restore` as the actual preflight read.
  The corrected fixture passed in `proc_e422`.
- Focused independent review found no normal public-Sinon reproduction violating the accepted ownership contract.
  A subsequent ordinary-overload check nevertheless found detached function spying was over-preflighted.
  `proc_a541` proved the failure;
  `2fe8e6311` narrows the check,
  and `proc_c6c6` passed the rebuilt regression and full unit suite.
- Ordinary Sinon cleanup stopping after a throwing restorer remains a baseline limitation.
  Do not expand this task into a reflective security membrane or complete global-state isolation.

## Integration fixes and commits

- `e658d1bbd`:
   red attempt/repeat lifecycle tests.
- `473407d38`,
  `1bcf62d56`,
  `10bb13e69`:
   attempt lifecycle and context-owned routing.
- `53013f383`:
   ordinary fake and clock restoration generations.
- `e2a77c23a`:
   lazy filesystem imports for browser diagnostics.
- `4c3c805c8`:
   logger handles a partial process shim.
- `8daa2901b`:
   mock verification/restoration generations.
- `c63fd9ac8`,
  `1f821673c`:
   contextual getter receiver and setter rejection.
- `b6f1a813c`:
   one build dependency graph prevents duplicate neutral builds racing the client consumer.
- `28793f3a5`:
   red returned-controller and partial-install tests.
- `8abaff575`:
   returned behavior generations,
  partial installation rollback,
  function-object members,
  and injection before setter exposure.
- `5e033ebc6`:
   function-object fixture satisfies declaration rules.
- `54faa77f6`,
  `6b98e7f27`:
   independent partial rollback failure coverage.
- `42de970c5`:
   actual logger breadcrumb concurrency.
- `b4e78ed32`:
   verified contract and canonical testing guidance.

## Documentation and closure verification

File-enforcer regenerated the canonical testing skill's mirrors and manifests.
Canonical and generated copies share SHA-256 digest
`0706fcd84c0c96ac8df583eaf62a25fd48479f340768e7eb2a6cbdb37102db17`.
Both manifests record it.
Mirrors are ignored local outputs,
confirmed with `git check-ignore` and `git ls-files`;
only canonical guidance is committed.

Scoped Markdown lint passed in `proc_11df`.
The historical missing-`isMdxPath` startup error is no longer present.
The final descriptor-field annotation passed scoped lint and types in `proc_c502`.

Post-correction acceptance passed:

- `proc_c6c6`:
  rebuilt full module-test unit tests,
  scoped lint,
  and types.
- `proc_fbfd`:
  full module-test Oxlint,
  actual logger unit tests,
  and logger Oxlint.
- `proc_df13`:
  combined real Chromium,
  Firefox,
  and WebKit consumer checks.

Closing commit `f7da78f6314dce62065cb4b4a8413a2816274ab0` carries `Closes #481`.
`git ls-remote origin refs/heads/main` confirmed that commit on the remote.
`gh issue view 481 --repo Aquaticat/Monochromatic --json state,closedAt,url`
confirmed `CLOSED` at `2026-09-07T09:34:59Z`.
The final scoped Markdown gate passed in `proc_2054` before closure.

## Tooling and concurrent work

Repository:
`/var/home/user/Monochromatic`.
Commits auto-push.
Use explicit owned pathspecs;
concurrent music-player changes may already be staged.
Preserve `mise.lock` and `doc/troubleshooting/module-test-unhandled-rejection.md`.

All builds and tests run through `mise run`.
The module-test browser task uses a disposable Podman container limited to 2 GiB RAM,
2 CPUs,
and one Playwright worker.
The neutral consumer is served at `/dist/module-test/` on port 3005.

The module-test scoped formatter accepts package-relative paths or normalized absolute paths outside the package.
It rejects `..`.
Root `format:oxlint` templates do not forward positional file paths.

Process logs from the initial session under `/tmp/pi-processes-Htuiik` disappeared during the pause.
Current logs are under `/tmp/pi-processes-CFxuQr`.
Durable verification details belong in these documents,
not only process logs.
