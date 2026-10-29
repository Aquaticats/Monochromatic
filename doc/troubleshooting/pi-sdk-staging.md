# Pi 0.87.1 nominal dependency inventory rejects the configured workspace graph

## Pi 1.2.0 native migration loses the Shift+Tab bypass toggle

### Symptom

On 2026-10-15 the user reported that the Shift+Tab toggle was missing.
The previous statement that Pi was functional overstated the verified surface:
live provider and approval checks did not establish interactive bypass parity.
This is an owned migration regression,
not evidence of an upstream keyboard failure.

### Root cause

The legacy extension registers the shortcut in
`package/pi-plugin/auto-mode/src/index.ts:163`:

```ts
// package/pi-plugin/auto-mode/src/index.ts
pi.registerShortcut(
  BYPASS_SHORTCUT,
```

`package/pi-plugin/auto-mode/src/bypass.ts:26` defines its key:

```ts
// package/pi-plugin/auto-mode/src/bypass.ts
const BYPASS_SHORTCUT = 'shift+tab';
```

Native installation removes the legacy extension declaration in
`package/pi-plugin/auto-mode/mise.toml:567`:

```js
// package/pi-plugin/auto-mode/mise.toml, install:native
for (const declaration of legacy) {
  const source = typeof declaration === 'string' ? declaration : declaration.source
  await spawn(process.execPath, [join(packageRoot, 'dist/native/cli.mjs'), 'remove', source, '--no-approve'], {
```

Before the repair,
the replacement resource factory only captured the extension API in
`package/pi-plugin/auto-mode/native/contract/integration/native-host/resources.ts:97`:

```ts
// package/pi-plugin/auto-mode/native/contract/integration/native-host/resources.ts
nativeApi.current = pi;
```

Its registered built-in factory list at line 113 contains the caller's factories and `captureApi`.
The native resource-loader reproduction records no shortcut registration.
The installed settings contain no legacy auto-mode declaration.
The user's keybindings still assign `app.thinking.cycle` to `ctrl+e`.

### Verification

The built Pi 1.2.0 native host was probed at repository commit `c81e5308fe`.
The probe uses disposable agent and workspace directories,
synthetic provider configuration,
and blocked networking.
It calls the actual built bootstrap's resource loader and reads its extension shortcut maps.

```bash
# Run from ${HOME}/temp/agent/pi-tool-call-scope-20261014
mise run probe-native-bypass
```

Working control:
an injected extension registers `shift+tab`,
and the same loader probe observes it.
Failing case:
the native host without that control registers no shortcuts.

```json
{"legacyAutoModeConfigured":false,"thinkingCycle":["ctrl+e"]}
{"positiveControl":["shift+tab"],"currentNativeHost":[]}
```

The final assertion fails with Node's `AssertionError [ERR_ASSERTION]`:
`Native host must retain the user-facing Shift+Tab bypass toggle`.
This verifies missing registration;
it does not test physical keyboard input in the user's running terminal.

### Workarounds and rejected conclusions

At initial diagnosis no workaround or restoration had been applied or verified.
Changing the user's thinking-cycle binding is not supported by the evidence:
it is already on Ctrl+E.
Simply displaying a toggle would not restore its execution semantics.
Re-enabling the entire legacy extension has not been validated with the native runtime.
The existing bypass requirement in `doc/planning/pi-auto-mode-effect-contract.md`
retains explicit user control and the fixed virtual-input guard.
Those behaviors,
session restoration,
audit entries,
and visible status were required restoration checks.

### Repair and verification on 2026-10-15

The native resource factory now calls `registerNativeBypass(pi)` immediately after capturing its API.
`src/native-bypass.ts` restores the original shortcut,
branch-backed toggle entries,
session status,
allow audit entries,
and fixed virtual-input guard.
The native batch handler captures explicit bypass before dispatch,
retains prepared execution ownership,
and skips semantic assessment without issuing a policy verdict.
It rejects revocation at its freshness boundaries.

The first execution test exposed an additional integration failure:
`PolicySlotBindingError: Native extension event callbacks changed after source capture`.
The new audit callback changed the event observation during nested execution.
Commit `775efbc86c` uses the existing retained-source lifecycle for both resolved approval and explicit bypass,
renamed `retainExecutionSources` to describe its actual responsibility.
The collector still validates original dependencies and append-only receipt growth.
The bypass test asserts that `lastReview` remains undefined.

The maintained build,
native build,
package `lint:types`,
and `test:native:bypass` pass.
The built integration consumer runs a producer and parallel children in a disposable native session:

```json
{"complete":true,"shortcut":true,"bypassExecutions":3,"bypassAssessments":0,"disabledRestoresProhibition":true,"audit":true,"status":true,"reload":true,"fixedGuard":true}
```

The original built-bootstrap probe now reports:

```json
{"positiveControl":["shift+tab","shift+tab"],"currentNativeHost":["shift+tab"]}
```

The duplicate control registration is the injected control plus the restored built-in registration.
The installed command uses the rebuilt native artifact.
An existing Pi process retains its old runtime until restarted;
extension reload alone does not replace that runtime.
These checks exercise native shortcut registration and its actual handler,
not physical keyboard delivery in the user's running terminal.

### Upstream filing decision

No upstream filing or draft is appropriate.
The upstream-fault constraint fails because owned installation removes the registering extension.
The repair belongs to this repository's replacement host.
Upstream fixability,
support policy,
contribution policy,
acceptance likelihood,
and an upstream patch are therefore not applicable to this incident.
No upstream clone or tracker mutation was needed to establish the owned failure.

## Sequential native judgment retries in Oxlint 1.88.0

### Symptom

Oxlint reports `eslint(no-await-in-loop): Unexpected await inside a loop`
for the original judgment transport attempt in
`package/pi-plugin/auto-mode/native/contract/lifecycle/judgment-budget-error-occurrences/controls-private/candidate.mjs`.
The implementation moved to `candidate.ts` on 2026-10-15;
the sequential transport loop now lives in sibling `dispatch.ts`.
The complete diagnostic includes backticks around `await`.
The package denies warnings,
so this finding fails lint even when the runtime consumer passes.

### Rule and application boundaries

The upstream implementation in
`crates/oxc_linter/src/rules/eslint/no_await_in_loop.rs:94`
reports syntactic loop ancestry:

```rust
// oxc/crates/oxc_linter/src/rules/eslint/no_await_in_loop.rs
if is_in_loop {
    ctx.diagnostic(no_await_in_loop_diagnostic(span));
}
```

The [current rule source](https://github.com/oxc-project/oxc/blob/main/crates/oxc_linter/src/rules/eslint/no_await_in_loop.rs)
defines a unit rule with no option parser or sequential-retry allow-list.
The existing local source checkout at commit `288d8cc77984b0a3851c58c423ffe9e6edc79f2e`
also contains that unconditional diagnostic.
This is expected rule behavior,
not an upstream defect.

The linted operation belongs to the original judgment's dispatch loop.
Its catch branch records the failed attempt,
rechecks cancellation and deadline,
classifies the actual transport failure,
and consumes the judgment's single retry before continuing.
Starting attempts together would bypass those conditions and spend attempts that may never be allowed.
The attempt count remains capped at three across dispatches;
the original preparation-inclusive deadline remains five seconds.

### Verification and exception

The budget module originally reported 55 warnings.
Adding boundary and retained-state types removed its unsafe-value warnings;
subsequent statement and error-definition cleanup left the sequential-await diagnostic
and one return-shape warning.
The return branch now explicitly returns its never-returning failure helper.
A next-line `eslint/no-await-in-loop` exception covers only the original transport await.
The exception retains sequential behavior and does not change any configured rule or admission limit.
The JavaScript budget and its new consumer built and passed their runtime checks.
The overlap-guard omission control failed its expected rejection assertion without changing the built artifact.
The complete JavaScript subset still had 139 warnings,
including unchecked imports in the new consumer.
After migration to the existing TypeScript setup,
package type checking and lint of all eight budget implementation and consumer files pass.
The focused lint report contains no warnings or errors.
The rebuilt typed consumers pass;
removing the overlap guard in memory makes the typed consumer fail its expected rejection assertion.
The built candidate and dispatcher hashes remain unchanged.
`${HOME}/temp/agent/pi-native-budget-operator-lint-20261015.log`
and `pi-native-budget-checked-consumer-20261015.log` in the same directory retain those results.

Reproduce with the package's `lint:oxlint` task and the explicit budget path.
`${HOME}/temp/agent/pi-native-budget-shape-lint-20261014.log`
retains the pre-exception diagnostics.
Named error fields and explicit input types pass their corresponding rules;
the unannotated boundary and unsuppressed retry await form the failing controls.

### Rejected alternatives and upstream filing

Parallel attempts are incompatible with failure-dependent retry eligibility.
The rule provides no targeted configuration alternative;
disabling it for a file or package would weaken unrelated checks.
No recursive retry implementation or additional promise chain is introduced.

No upstream report or draft is warranted.
The rule operates as specified;
upstream could add an option,
but the application requires no tool change.
Supported syntax is already handled.
Contribution acceptance and likelihood of an upstream change are immaterial to this local exception,
and no upstream patch is proposed or prototyped.
The checked `.out-of-scope/` entries contain no applicable Oxlint exemption.
There is no upstream filing requiring a duplicate search.

## Owned native program builds depended on source formatting

### Symptom and cause

The October 14 formatting commit `2728f357f` changed maintained program source layout.
The native build then emitted `Native SDK edit does not have one matching location`
for the owned runner,
prepared dispatcher,
and judgment constructor.
The deciding helper in
`package/pi-plugin/auto-mode/native/contract/integration/native-batch/build-rebase.ts:63`
matches literal source text:

```ts
// native/contract/integration/native-batch/build-rebase.ts
const at = source.indexOf(from);
```

Those transformations patched repository-owned modules consumed only by this build.
Formatting changed the search text without changing the intended implementation.
This was an unnecessary owned-source transformation boundary,
not installed Pi source drift.

### Correction and verification

Commit `26994ff789` maintains the integrated runner,
dispatcher,
and judgment directly in `native/contract/collector/program-rule-batch-sdk-copy-v2/input/`.
`native/contract/integration/native-batch/prepare-program.ts:137` now emits an export bridge:

```ts
// native/contract/integration/native-batch/prepare-program.ts
const programRunner = `export {createReachedRunnerClass} from ${JSON.stringify(programRunnerUrl.href)};\n`;
```

The installed-Pi executor and converter adaptations remain separate.
Historical provenance hashes retain their original meaning.
The intermediate parsed-source runner and dispatcher replacements were removed:
they preserved an unnecessary transformation rather than removing its need.

The native build and package type check pass.
After correcting the separate JavaScript modifier recurrence documented in
"Native source autofixes emit invalid JavaScript",
the domain,
manual review,
approval context,
single call,
approved dependency,
event receipt,
host lifecycle,
and actual CLI process tasks pass.
The CLI fixture made zero provider requests and exited with clean shutdown.
The syntax task checks 227 maintained modules after explicit TypeScript erasure.
The retained log is `${HOME}/temp/agent/pi-native-direct-owned-regressions-syntax-fixed-20261014.log`.

### Upstream filing decision

- The failing transformation was repository-owned.
- No upstream fix is required for the direct module boundary.
- No unsupported Pi use case is inferred from this failure.
- External contribution policy is irrelevant to this local correction.
- No upstream maintainer position is inferred.
- The owned correction has passing build and consumer evidence;
  no upstream prototype or filing is proposed.

## Pi 1.2.0 confirmation fixtures must preserve occurrence identity and exact assessment input

### Symptoms and separate causes

The October 14 confirmation verification encountered separate local controller errors.
They did not establish an upstream Pi or Luna defect.

The first authored RPC assessment matched the policy tag `CNF` without its source occurrence.
It assigned `fresh-session-approval` both to the designated policy file
and to an extension-produced composite prompt containing the same text.
The compiler retained unresolved authority for that second occurrence,
so the then-current strict policy blocked the read.
`package/pi-plugin/auto-mode/native/contract/integration/action-policy/compiler.mjs:250`
gives the retained reason:

```js
// native/contract/integration/action-policy/compiler.mjs:250, authority diagnostic
'The original candidate has no established instruction authority and request scope; source registration does not resolve remaining unit-level delegation.'
```

The local fixture now selects the original occurrence as well as the tag.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/rpc-hook.mjs:44` contains:

```js
// Private configured-host fixture, rpc-hook.mjs:44
const original=instruction.selectedRule?.sourceOrdinal===0;
const rule=original&&instruction.selectedRule?.id==='RUL';
```

This corrects authored test inputs only.
It does not authenticate the composite source or change a live model's interpretation.
The later adopted manual-confirmation policy handles otherwise uncertain assessments separately.

The first live-score verifier passed response metadata into the strict categorical consumer.
It received `InstructionResolutionInputError: Policy record contains an unsupported field`
after the HTTP request had completed.
`package/pi-plugin/auto-mode/src/semantic-operating-choice.ts:122` selects only these fields:

```ts
// package/pi-plugin/auto-mode/src/semantic-operating-choice.ts, categorical input fields
fields: [
  'choice',
  'probabilities',
],
```

`package/pi-plugin/auto-mode/src/policy-data.ts:79` rejects every other own key:

```ts
// package/pi-plugin/auto-mode/src/policy-data.ts:79
for (const key of Reflect.ownKeys(input,)) {
  if (((typeof key) !== 'string') || (!fields.includes(key,)))
    throw new InstructionResolutionInputError('Policy record contains an unsupported field',);
```

The verifier was corrected to pass the original `choice` and `probabilities` fields.
It rescored the preserved response without repeating the completed provider call.

A later replay correctly failed
`AssertionError: Retained Luna answers require the unchanged complete assessment request`.
The new session's observed event timestamps,
durations,
and dependent source identities differed from the saved assessment.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/rpc-hook.mjs:37` enforces:

```js
// Private configured-host fixture, rpc-hook.mjs:37
assert.deepEqual(body,original,'Retained Luna answers require the unchanged complete assessment request');
```

Ignoring those differences would admit answers for another input.
The corrected live bridge instead sends each session's actual assessment to Luna within its original deadline.

### Verification and workarounds

All controls use Node 26.11.1 and the maintained Pi 1.2.0 host.
The failed composite-occurrence cases remain in `pi-hyper-rpc-allow-aIIepr`
and `pi-hyper-rpc-allow-rKcQFn` under `${HOME}/temp/agent/`.
The corrected authored controls in `pi-fresh-session-hyper-rpc-original-source-20261014.log`
pass Allow,
Stop,
dismissal,
and prohibition with zero provider requests.
Their tradeoff is authored interpretation rather than live model evidence.

`pi-luna-current-confirmation-20261014/failure.json` preserves the verifier failure.
Its retained responses and summary show three unchanged live calls with identical answer data
and 22 of 84 admitted meanings under the adopted `0.60` rule.
Projection into the strict consumer discards transport metadata only at that boundary;
the original provider responses remain intact.

`pi-hyper-rpc-allow-KN9yYC` retains the rejected replay.
The `compare-rpc` task reads its actual request and the earlier saved request without a provider call:

```sh
# Run from ${HOME}/temp/agent/pi-tool-call-scope-20261014
mise run compare-rpc
```

The subsequent live bridge cases in `pi-manual-review-hyper-live-luna-20261014.log`
pass Allow,
Stop,
and dismissal with one new assessment per current request.
The primary responses remain authored in those cases.
`pi-hyper-rpc-allow-Gt6QZx/verification.json` separately records an actual Hyper-generated read,
live Luna review,
scripted Allow,
and the original read result.
It used two Hyper requests and one Luna request.
Its receipt originally counted only the assessment;
the receipt was corrected from preserved primary-response records without rerunning the host.
Live verification costs new provider requests and is unsuitable for unchanged-response replay claims.

### What does not work

Matching a tag without source identity confuses distinct instruction occurrences.
Passing the whole decoded answer to a closed categorical consumer violates its input contract.
Replaying a response after timestamp or source-identity changes violates exact request attribution.
Repeating completed requests to repair a verifier discards usable evidence and changes the experiment.

### Upstream filing decision

1.  Fault: these failures belong to our authored inputs,
    verifier projection,
    and attempted cross-session replay.
2.  Fixability: corrections belong in those local controllers;
    the production identity and input checks remain intact.
3.  Supported use: current-input assessment works through the native host;
    applying old answers to different input is outside this verification contract.
4.  Contribution policy: no third-party source change or public report is proposed.
5.  Upstream disposition: no external defect is established,
    so there is no candidate upstream issue or duplicate search to pursue.
6.  Prototype: the occurrence-qualified fixture,
    preserved-response projection,
    and fresh-request bridge each have the passing controls named here.

Nothing is filed upstream.
These local incidents remain separate from model accuracy,
source-authority design,
and the package's broader lint work.

## Pi 1.2.0 Hyper selection succeeds while the configured fixture rejects startup diagnostics

### Symptom and cause

The October 14 Hyper default-selection fixture returned the requested
`hyper/gemma-4-26b-a4b-it` RPC state and exited zero,
then its controller failed `AssertionError [ERR_ASSERTION]` while comparing stderr with an empty string.
This was an overbroad local assertion,
not a failed model selection.
The controller at `${HOME}/temp/agent/pi-hyper-model-20261014/state.ts:80` contains:

```ts
// Private configured-host fixture, state.ts
assert.equal(state.isStreaming,false);assert.equal(state.pendingMessageCount,0);assert.equal(stderr,'');
```

The fixture deliberately rejects external fetches and copies only the Hyper model catalogue.
The retained startup diagnostics include the Synthetic extension's blocked model discovery,
Pi's `No models match pattern` warnings for other configured providers,
and ordinary extension lifecycle logging.
The related [Synthetic startup diagnosis](pi-synthetic-provider-startup-model-scope.md)
retains the separate historical scope-resolution incident;
this fixture's deliberate network boundary does not reproduce that historical root cause.

### Verification and bounded recovery

`${HOME}/temp/agent/pi-hyper-state-MD0Lvx/events.json` retains the original successful RPC response.
Its `stderr.txt` retains every diagnostic byte.
The controller's exit,
model identity,
idle state,
and pending-message assertions passed before the empty-stderr assertion.
The disposable host loaded all 17 configured packages under network isolation.
It made zero provider requests.

The separate `verify-state` task in `${HOME}/temp/agent/pi-hyper-model-20261014/mise.toml`
reads those original artifacts and the saved defaults without rerunning the host.
Its `state-verification.json` records successful selection and failed empty-stderr verification separately.
This recovers the selection result only;
it neither claims a live Gemma response nor qualifies an empty-stderr lifecycle.
Discarding the diagnostics or treating all startup output as a shutdown defect is rejected.

### Upstream filing decision

1.  Fault: the failing assertion belongs to our disposable controller.
2.  Fixability: our verifier can distinguish model selection from its separate diagnostic assertion.
3.  Supported use: ordinary model selection is supported;
    silence from every configured extension is our additional fixture expectation.
4.  Contribution policy: no external code change or report is proposed for this local assertion.
5.  Upstream disposition: this result establishes no new upstream defect to assess or search for duplicates.
6.  Prototype: the local artifact verifier passes while retaining the original failed assertion and all diagnostics.

Nothing is filed upstream.
The existing Synthetic troubleshooting record remains a separate incident.

## Pi 1.2.0 direct generated manager import reaches an uninitialized owned bootstrap constructor

### Symptom

On Node 26.11.1,
the first `native-tool-publication.unit.test.ts` entry imported the generated manager
before the maintained policy entry.
Node reported `ReferenceError: Cannot access 'SessionManager' before initialization`
at the owned `native-host/host.ts:160`,
called from `native-host/bootstrap.ts:236`.
Importing the generated SDK first produced the same error.
Neither run reached the publication assertions.
This is distinct from the earlier `ExtensionRunner` initialization failure.

### Root cause

The owned generator adds a dependency from the manager to the maintained policy entry.
`package/pi-plugin/auto-mode/native/contract/integration/native-batch/root-source.ts:263`
emits the query-observer import:

```ts
// package/pi-plugin/auto-mode/native/contract/integration/native-batch/root-source.ts:263
+`import {createLoadoutQueryObserver} from ${JSON.stringify(queryUrl)};\n`
```

`package/pi-plugin/auto-mode/native/contract/integration/native-batch/loadout-query-observer.mjs:1`
then imports the maintained implementation:

```js
// package/pi-plugin/auto-mode/native/contract/integration/native-batch/loadout-query-observer.mjs:1
import {createNativeQueryObservation} from '../maintained-policy.mjs';
```

Before commit `63be621a7d`,
the generated maintained-policy module re-exported the package's built extension entry.
That entry includes the upstream SDK import from
`package/pi-plugin/auto-mode/src/visible-context.ts:19`:

```ts
// package/pi-plugin/auto-mode/src/visible-context.ts:19
import {
  sessionEntryToContextMessages,
  type SessionEntry,
} from '@earendil-works/pi-coding-agent';
```

The resolver observations in `${HOME}/temp/agent/pi-tool-publication-controls-20261014/import-edges.json`
retain the generated host-runtime import of the owned bootstrap during this dependency traversal.
The bootstrap eagerly constructs the host in
`package/pi-plugin/auto-mode/native/contract/integration/native-host/bootstrap.ts:236`:

```ts
// package/pi-plugin/auto-mode/native/contract/integration/native-host/bootstrap.ts:236
export const nativeHost: NativeCliHost = createNativeCliHost({
  designatePolicy,
  bindSession
});
```

That constructor reads the still-initializing manager binding in
`package/pi-plugin/auto-mode/native/contract/integration/native-host/host.ts:158`:

```ts
// package/pi-plugin/auto-mode/native/contract/integration/native-host/host.ts:158
const factory = createRootSessionFactory({
  createAgentSession,
  SessionManager,
```

The read-only upstream checkout remains tag `v1.2.0`,
commit `06a466935536f8c9bcacb2d5efc0fc021be1bdff`.
Its `packages/coding-agent/src/core/session-manager.ts` has no owned query observer or CLI bootstrap import.
The demonstrated cycle crosses this repository's generated instrumentation and maintained entry.
No change to the new tool publication's identity checks remedies that import order.

### Verification and supported entry

The direct-manager failure is reproducible with the scratch `import-trace` task:

```sh
# Run from ${HOME}/temp/agent/pi-tool-publication-controls-20261014
mise run import-trace
```

The maintained consumer initializes its policy entry before inspecting manager exports:

```ts
// package/pi-plugin/auto-mode/src/native-tool-publication.unit.test.ts
await import(new URL('../dist/native/contract/integration/maintained-policy.mjs', import.meta.url).href);
```

With that entry order,
`mise run //package/pi-plugin/auto-mode:test:native:serializer-receipts` passes the new manager publication tests,
the original serializer emission tests,
and all copied-system/payload/transcript controls.
The separate normal SDK fixture still accounts for five contributions and executes three fixture children,
with zero provider requests.
That successful entry order verified the intended consumer;
it did not fix direct generated-manager or generated-SDK imports.
The dedicated policy entry described in this section later removed the coupling.

The 2026-10-15 domain-fixture split reproduced this same diagnostic at the same owned host constructor.
The first static dependency became the assertion module that imports the generated manager,
before the runtime's maintained policy dependency.
The failed built run is
`${HOME}/temp/agent/pi-native-domain-fixture-split-consumer-20261015.log`.
The fixture now explicitly imports `../maintained-policy.mjs` before its generated-manager consumers.
The rebuilt domain,
unconfigured-policy,
operating-policy,
single-call,
approved-dependency,
next-decision,
serializer,
and context contrast sets pass in
`${HOME}/temp/agent/pi-native-domain-fixture-entry-consumer-20261015.log`.
That verified the documented supported entry order after extraction;
it did not remove the underlying generated import cycle.

The manager publication tests cover original append and context copies,
separate equal-content owners,
cross-owner reuse,
array replacement,
tool replacement,
schema mutation,
and accessor rejection without invocation.
The process-local array identity omission makes the array-replacement assertion fail.
`verification.json` in the same scratch directory retains the unchanged built-artifact hash.

### Dedicated native policy entry

The owned graph fix preserves eager host construction and original constructor capture.
Instead,
the generator now resolves `./native-policy` at
`package/pi-plugin/auto-mode/native/contract/integration/native-batch/prepare.ts:137`:

```ts
// package/pi-plugin/auto-mode/native/contract/integration/native-batch/prepare.ts:137
const maintainedEntry=entry({
  root:maintainedRoot,
  subpath:'./native-policy'
});
```

The dedicated entry directly re-exports native policy dependencies,
including the observer at `package/pi-plugin/auto-mode/src/native-policy-api.ts:34`:

```ts
// package/pi-plugin/auto-mode/src/native-policy-api.ts:34
export { createNativeQueryObservation } from './native-query-observation.ts';
```

The same bundler invocation emits the extension and native entries with shared implementation chunks.
`package/pi-plugin/auto-mode/rolldown.node.config.ts:11` retains the named inputs:

```ts
// package/pi-plugin/auto-mode/rolldown.node.config.ts:11
const config: ReturnType<typeof nodeConfig> = nodeConfig({
  input: {
    index: './src/index.ts',
    'native-policy': './src/native-policy-api.ts',
  },
});
```

Shared chunks matter because policy functions,
error constructors,
and retained state must have the same identity for native and extension consumers.
Separately bundling the same implementation would not establish that property.
The generated policy entry and its shared chunk no longer import the SDK extension entry.
The runtime proof is `test:native:direct-imports`:
each fresh process imports either the generated manager or generated SDK first,
constructs an owned in-memory manager,
and compares every native policy export with the extension export by identity.

Commit `bdee84f697` added the regression before the fix.
Both cases failed with the exact original `ReferenceError` on Node 26.11.1 in
`${HOME}/temp/agent/pi-native-direct-import-regression-before-20261015.log`.
After the entry split,
the direct-import and identity checks pass in
`${HOME}/temp/agent/pi-native-policy-entry-consumers-20261015.log`.
That run also passes manager constructors,
host lifecycle replacement and disposal,
CLI RPC state and clean shutdown,
serializer controls,
domain contrasts,
and authored approval decisions without provider requests.
Its final type check exposed `TS9037: Default exports can't be inferred with --isolatedDeclarations.`
The config now explicitly annotates its value with `ReturnType<typeof nodeConfig>`.
Commit `6ddeaef3eb` also removes the publication and domain fixtures' policy-preload workarounds.
The rebuilt direct-import identity,
publication,
domain,
and package type checks pass without either preload in
`${HOME}/temp/agent/pi-native-policy-unordered-consumers-20261015.log`.
The original standalone manager reproduction also passes in
`${HOME}/temp/agent/pi-native-direct-manager-import-after-20261015.log`.
The entry,
generator,
declarations,
regression,
and build config have zero errors and one `tsdoc(multiline-blocks)` warning in
`${HOME}/temp/agent/pi-native-policy-entry-final-lint-20261015.log`.
The added config comment needs multiline formatting;
the earlier zero-warning statement in `12a4a8a00c` was incorrect.
Commit `6701c3937d` expands that comment.
The repeated focused lint now passes with zero warnings and errors in
`${HOME}/temp/agent/pi-native-policy-entry-clean-lint-20261015.log`.

Reproduce the committed entry checks after building:

```sh
# Run from the repository root.
mise run //package/pi-plugin/auto-mode:test:native:direct-imports
```

### What did not work before the entry split

- Importing the manager directly fails before the test body.
- Importing the generated SDK before the manager produces the same constructor initialization failure.
- Passing the native SDK scenario does not establish arbitrary submodule import order.

### Upstream filing decision

- Fault: the demonstrated cycle includes owned instrumentation and eager bootstrap construction.
  An upstream defect has not been established.
- Fixability: the owned graph can remove its eager dependency;
  no upstream architectural limitation was found.
- Supported use: upstream supports native SDK construction,
  but the failing generated graph is repository-owned.
- Contribution policy: the checked `CONTRIBUTING.md:43` requires the author's own issue voice;
  an autonomous generated issue would not meet that requirement.
- Maintainer direction: searches for `"SessionManager" "initialization"` returned no issues and
  [PR 10789](https://github.com/earendil-works/pi/pull/10789).
  Its complete body and comments concern JSONL write coalescing,
  not this import cycle.
- Prototype: the dedicated policy entry fixes the owned graph locally.
  No upstream patch is justified by this evidence.

The `.out-of-scope/` catalog was checked;
`pi-gpt55-long-context.md` concerns a different boundary.
Nothing to file or add upstream.

## Pi 1.2.0 transcript normalization invalidates a direct serializer identity comparison

### Symptom

The migration's new original-system emission check initially made
`test:native:sdk-stream` fail with `AssertionError [ERR_ASSERTION]`
at `domain-frontier-assertions.mjs:47`.
Its native request retained an unresolved system contribution although the original producer ran.
The consumer compared the system renderer's input array with the stream caller's input array.

### Root cause

Read-only checkout `${HOME}/temp/agent/auto-mode-pi-source-1.2.0`
is `earendil-works/pi` tag `v1.2.0`,
commit `06a466935536f8c9bcacb2d5efc0fc021be1bdff`.
The source disproves the assumption that the same transcript array crosses both boundaries.
`packages/ai/src/utils/transcript.ts:108` creates a new array when collapsing system messages:

```ts
// packages/ai/src/utils/transcript.ts:108
export function collapseSystemMessages(context: TranscriptContext): TranscriptContext {
  const head = getCurrentSystemMessage(context.messages);
  const messages = context.messages.filter((message) => message.role !== "system");
  return { messages: head ? [head, ...messages] : messages } as TranscriptContext;
}
```

`packages/ai/src/api/openai-completions.ts:310` selects that normalized context:

```ts
// packages/ai/src/api/openai-completions.ts:310
const normalizedContext = resolveTranscript(context, getCompat(model).supportsMidConvoSystemMessages);
```

The original call at `packages/ai/src/api/openai-completions.ts:358` passes it to `buildParams`:

```ts
// packages/ai/src/api/openai-completions.ts:358
let params = buildParams(
  model,
  normalizedContext,
  options,
  compat,
  cacheRetention,
  grammarToolInputProperties,
);
```

This is a consumer identity error,
not evidence of an upstream serialization defect.
The corrected owned transformation retains both original arrays in the existing producer receipt:

```js
// Generated by package/pi-plugin/auto-mode/native/contract/integration/native-batch/message-source.ts
nativeSerializedPayloads.set(params, Object.freeze({
  model,
  messages: context.messages,
  normalizedMessages: normalizedContext.messages,
}));
```

The system emission's `messages` reference is compared with `normalizedMessages`;
the request receipt still compares its original `messages` reference with the captured caller transcript.
No original request is parsed again or re-encoded to create that link.

### Verification and rejected approaches

The initial failing log is `${HOME}/temp/agent/pi-serialization-native-20261014.log`.
After commit `2510bc1e1d`,
`pi-serialization-native-final-20261014.log` records five accounted contributions,
three executed children,
one authored assessment,
and zero provider requests.
The built emission tests exercise updated system sections,
escaped text,
original tool-object identity,
absent tools,
and rejection of equal copied containers.
The copied-payload and copied-transcript native controls remain unresolved with zero executions.

```sh
# Repository root
mise run //package/pi-plugin/auto-mode:build:native:owned
mise run //package/pi-plugin/auto-mode:test:native:sdk-stream
mise run //package/pi-plugin/auto-mode:test:native:serializer-receipts
```

Do not replace the failed identity comparison with content equality or remove provenance validation.
The verified consumer correction follows the native normalization edge.
Its tradeoff is an additional original reference retained for the request lifetime;
it does not prove complete governing use of system text or tool declarations.
The copied-system control also passes:
an equal replacement system message leaves its contribution unresolved,
preserves the independent user contribution,
and executes no child.
Process-local omission of `&&originalSystem` makes that copied message incorrectly accounted
and fails the native contribution assertion.
`${HOME}/temp/agent/pi-serialization-controls-20261014/verification.json`
records the expected failure and unchanged artifact hash.

### Upstream filing decision

The `.out-of-scope/` inventory was checked.
`pi-gpt55-long-context.md` concerns model context limits and does not apply here.
Read-only `gh search issues` for `resolveTranscript identity`
and `gh search prs` for `collapseSystemMessages identity`
in `earendil-works/pi` both returned empty results on 2026-10-14.

- Upstream fault: no;
  the migration compared different original call boundaries.
- Fixability: the consumer owns and corrects this comparison without an upstream change.
- Supported use: the source explicitly implements collapsed system-message replay;
  it does not promise unchanged array identity.
- Contribution policy: `CONTRIBUTING.md:43` requires the reporter's own voice and restricts generated issue text;
  the README and bug template point to that policy.
  No filing is proposed.
- Maintainer disposition: the empty searches establish no position about this consumer integration.
- Prototype: the native consumer correction and built emission controls pass;
  no upstream patch is needed.

There is nothing to file upstream from this owned identity error.

## Pi 1.2.0 native RPC Stop exposes an overbroad fixture stderr assertion

### Symptom and corrected interpretation

The October 14 disposable original-RPC fixture initially required empty stderr for every response.
Allow and Deny passed.
Stop recorded `user-deny`,
executed no tool,
and exited zero,
but failed `AssertionError [ERR_ASSERTION]: Native child cleanup must not emit stderr`.
Its retained output contained `RequestObservationStaleError: Observed request was cancelled`
and `PreparedBatchExecutionError: Prepared batch retained lifecycle failures`.
The assertion incorrectly treated all execution diagnostics as cleanup diagnostics.

### Source trace

The maintained approval wrapper forwards the original Stop capability.
Abort forwarding now lives in
`package/pi-plugin/auto-mode/native/contract/lifecycle/incumbent-tui-group/approval-presentation.mjs`.
The original-source approval router was retired on 2026-10-17;
`prepared-group.mjs` now owns its scope checks and confirmation routing.
The historical source location remains in the captured excerpt:

```js
// package/pi-plugin/auto-mode/native/contract/lifecycle/incumbent-tui-group/approval.mjs:100
abort(){trace('tui-stop');return Reflect.apply(
  abort,
  ctx,
  []
);},
```

The completed semantic review still requires its original inputs to remain current:

```js
// package/pi-plugin/auto-mode/native/contract/integration/action-policy/evaluate.mjs:140
function assertCurrent(){inputs.assertCurrent();authorizationInputs.assertCurrent();result.assertCurrent();}
assertCurrent();
```

The request observer rejects a cancelled original request:

```js
// package/pi-plugin/auto-mode/native/contract/lifecycle/root-lifecycle-retirement-entry/observer.mjs:79
if(scope.signal
  ?.aborted)throw new RequestObservationStaleError(
  'Observed request was cancelled',
  {cause:scope.signal
    .reason}
);
```

The existing `native-batch/approval-domain.test.mjs` explicitly expects a handler failure for Stop.
No source guard was removed to satisfy the new fixture.

### Verification and rejected workaround

`${HOME}/temp/agent/pi-native-rpc-human-20261014/`
retains the child,
controller,
complete RPC transcripts,
original verdict entries,
and unfiltered stderr.
The corrected child forwards every original stderr write unchanged while recording its lifecycle phase.
The controller proves that concatenating captured writes reproduces every stderr byte.
Stop has 10,467 execution-diagnostic bytes and zero disposal-diagnostic bytes.
Cancellation,
Allow after 6,007 ms,
and an unrelated identifier followed by Deny also pass.
Each uses one assessment,
zero provider requests,
and exit zero.
Only Allow executes the inert tool.
No genuine human response is claimed.

Do not suppress cancellation logs or remove currentness checks to obtain empty stderr.
The corrected measurement distinguishes expected execution retirement from disposal failures;
it does not establish production authorization-owner integration.

The first scratch invocation also emitted Mise 2026.10.0 path-trust diagnostics
and `deprecated [tera_template_task_args]` before completing its Allow control.
Subsequent invocations use the existing repository `usage` convention,
non-login shell startup,
and the exact inspected config path through `MISE_TRUSTED_CONFIG_PATHS`.
This follows the [recorded temporary-config workaround](mise-untrusted-temporary-worktree-shell.md).

### Upstream filing decision

The failed assertion belongs to the owned fixture.
No upstream defect is established;
an upstream fix,
unsupported use-case claim,
contribution assessment,
or maintainer willingness claim is therefore inapplicable.
The local correction has the retained controls and changes no runtime behavior.
No upstream draft or message is warranted.

## Pi 1.1.0 dispatcher replaces a fetch override installed before module loading

The disposable offline native-host capture initially produced Pi's `Connection error.`
and no assessment request.
The fixture's network namespace had no network access,
so no provider request escaped.
Its fetch override was installed before the original HTTP-dispatcher module loaded.

At Pi source tag `v1.1.0`,
`packages/coding-agent/src/core/http-dispatcher.ts:16`
captures `const originalGlobalFetch = globalThis.fetch`.
The same file at line 105 decides whether to install Undici globals:

```ts
// packages/coding-agent/src/core/http-dispatcher.ts:105
const shouldInstallGlobals =
  installedGlobalFetch === undefined
    ? globalThis.fetch === originalGlobalFetch
    : globalThis.fetch === installedGlobalFetch;
if (shouldInstallGlobals) {
  undici.install?.();
  installedGlobalFetch = globalThis.fetch;
}
```

An override installed before module loading becomes the captured original and is replaced.
The verified fixture workaround imports this dispatcher first,
then installs its own fetch function before starting the native CLI.
This uses the source's explicit preservation of overrides installed after module loading.
It changes fixture setup order only;
installed Pi source and the production transport remain unchanged.

The failed catalog is `${HOME}/temp/agent/pi-native-installed-live-qjVj0h`.
The corrected capture at `${HOME}/temp/agent/pi-native-installed-live-UFBJsm`
uses synthetic credentials,
no network access,
the observed literal read program,
and the original configured extension graph.
It captures 84 questions and 73 source occurrences without executing the governed read.
The owning local fixture is `${HOME}/temp/agent/pi-capacity-offline.test.ts`.

No upstream filing is proposed.
The behavior follows the inspected dispatcher contract;
this is a fixture-order correction,
not an established upstream defect.
An upstream change is possible but unnecessary for supported post-load overrides.
Contribution policy and willingness were not investigated,
and no upstream patch prototype is needed for the verified caller-side correction.

### Maintained Pi 1.2.0 process fixture

The maintained CLI fixture had the same ordering defect independently of the offline capture.
After rebuilding the owned artifact,
`mise run //package/pi-plugin/auto-mode:test:native:process-network`
failed with Node's `AssertionError [ERR_ASSERTION]: Pi replaced the CLI fixture network blocker`.
The received function was Undici's `fetch`;
the expected function was the fixture's `rejectFixtureNetwork`.
The assertion runs before its data-URL probe,
so this regression makes no external request even in the failing state.

The emitted Pi 1.2.0 dispatcher at
`package/pi-plugin/auto-mode/dist/native/pi/coding-agent/dist/core/http-dispatcher.js:90`
uses the same original-function comparison:

```javascript
// Emitted Pi 1.2.0 core/http-dispatcher.js:90
const shouldInstallGlobals = installedGlobalFetch === undefined
    ? globalThis.fetch === originalGlobalFetch
    : globalThis.fetch === installedGlobalFetch;
```

The owned guard now loads that original module before installing the blocker.
Its source is `package/pi-plugin/auto-mode/native/contract/integration/native-host/process-network-guard.ts`.
The regression covers initial setup,
reconfiguration,
actual fetch rejection,
and rejection of use outside the explicitly marked fixture child.
After `build:native:owned`,
both `test:native:process-network` cases pass.
The real `test:native:cli-process` also passes with the expected Qwen model,
idle RPC state,
empty stderr,
and clean shutdown.
This verifies fetch blocking through the native dispatcher;
it is not a general network namespace isolation claim.

## Pi 1.1.0 retained callback options shift an owned source fixture

### Symptom and cause

Node 26.10.0 emitted `AssertionError [ERR_ASSERTION]: undefined !== 1`
in private `contract/integration/native-batch/original-judgment.test.mjs:124`
during the maintained runtime-hook cutover (`proc_f31e`).
The fixture interpreted a callback-options object as the constructor's tool array.
Its later missing-review assertion was a consequence of that earlier failed wire check.

Before private `0a69355c`,
the fixture placed constructor metadata immediately after handler messages:

```javascript
// Private contract/integration/native-batch/original-judgment.test.mjs:26 before 0a69355c
const constructorSourceOffset=messageSourceOffset+2;
```

The original run-input collector already retained callback options after those messages.
Private `contract/collector/run-prompt-custody-v3/message-inputs.mjs:27` includes this sequence:

```javascript
// Excerpt of the original active-input array in message-inputs.mjs:27
...publication.handlerMessageInputs,...(publication.handlerOptionInputs??[])
```

The hypothesis that extracting hook bookkeeping caused this source mismatch was rejected.
`proc_9b32` loaded the pre-cutover runtime from `990e177c^` into a disposable module,
kept current source owners and fixture assertions,
and reached the same `undefined !== 1` assertion.
No installed SDK source or maintained runtime artifact was changed by that comparison.

### Correction and verification

Private `0a69355c` derives later offsets from the original callback observation count.
It also checks every newly represented option source,
rather than merely changing the constructor index:

```javascript
// Private contract/integration/native-batch/original-judgment.test.mjs:72 and :129
assert.equal(run.handlerOptionInputs.length,run.handlerObservations.length);
assert.equal(policy.content,JSON.stringify(publications[0].run.handlerObservations[ordinal].before.options));
```

`proc_6fa5` passed both the migrated and pre-cutover runtimes with these corrected assertions.
The migrated path also passed exact maintained-export identity,
disposal gating,
partial installation rollback,
retryable native cleanup,
and original native start-error preservation.
These were local fixture exchanges with zero provider requests and zero protected tool executions.

Owning commands in private `contract/integration/native-batch` are
`mise --no-env --no-hooks run --skip-tools test:runtime-hooks`
and `mise --no-env --no-hooks run --skip-tools test:runtime-hooks:baseline`.
Use the restricted launcher in the current frontier.
The failing catalog is the old constructor offset with either runtime;
the passing catalog is original callback-option accounting with either runtime.
This corrects a fixture contract,
not source membership,
authority,
or native behavior.

### Rejected remedies and upstream filing decision

Reverting hook extraction did not resolve the assertion.
Dropping the callback inputs or ignoring failed wire checks would discard required evidence,
not fix the fixture.
There is nothing to file upstream:

- The fault was an owned fixture's outdated source offset,
  not Pi or Node behavior.
- The consumer correction passed without an upstream change.
- No upstream SDK support boundary was changed.
- Contribution policy was not investigated because no upstream defect is proposed.
- Maintainer willingness is not asserted and cannot override the failed fault condition.
- The verified patch is consumer-side;
  no upstream patch prototype or issue draft is warranted.

## Pi 1.1.0 extension receipt fixtures and main-only imports

### Symptoms

The first extension receipt fixture failed with Node's
`TypeError: Cannot read properties of undefined (reading 'description')`
in `extension-tool-input.test.mjs` (`proc_af3f` and `proc_f39c`).
It queried `session.getToolDefinition()` while configuring `tools: []`.
Binding extensions did not resolve the lookup.
The earlier explanation that binding caused the missing definition was incorrect.

The expanded stager also failed with
`TypeError: Cannot read properties of undefined (reading '.')`
at `metadata.exports[subpath]` (`proc_5a5e`).
That helper assumed every native dependency declared an export map.

### Source findings

In installed Pi 1.1.0,
`package/pi-plugin/auto-mode/node_modules/@earendil-works/pi-coding-agent/dist/core/sdk.js:153`
selects an allowlist from the explicit tool option:

```javascript
// Installed Pi 1.1.0 dist/core/sdk.js:153
const allowedToolNames = toolModifiers
    ? options.noTools === "all"
        ? selectedToolNames
        : undefined
    : (options.tools ?? (options.noTools === "all" ? [] : undefined));
```

`package/pi-plugin/auto-mode/node_modules/@earendil-works/pi-coding-agent/dist/core/agent-session.js:2821`
filters extension definitions before placing them in the session definition registry:

```text
// Installed Pi 1.1.0 dist/core/agent-session.js:2821
.filter((tool) => this._isAllowedTool(tool.definition.name));
```

The native registration map and the session's allowed definition registry are different evidence surfaces.
Installed `dist/core/extensions/loader.js:231` receives and validates `registerTool(tool)`,
then replaces `extension.tools` by name before calling `runtime.refreshTools()`.
An excluded session definition does not prove registration never received the input.

The installed `@earendil-works/pi-tui` 1.1.0 `package.json` has
`"type": "module"` and `"main": "dist/index.js"`,
without `exports`.
The owned stager's unconditional export-map lookup caused the import failure.

### Correction and verification

The fixture now checks the native extension registration map directly.
An additional `--allow-hidden` case names the tool in the allowlist
and verifies the session definition while its hidden exposure still prevents activation.
This distinguishes received input,
registration,
allowed definitions,
and active declarations without assigning policy authority to any of them.

The stager uses the package's declared `main` only for a root import when `exports` is absent.
It preserves native package-relative loader resolution
and stages resource and extension loader observations without changing installed files.
`proc_1073` passed the original receipt test;
`proc_6749` passed resource bounds,
inline excluded/allowed registration,
and actual file loading with a native Pi AI compatibility import.
The stager reported SDK 1.1.0 and 13 copied native modules.
All those fixtures made zero provider requests.

The owning inner command is
`mise --no-env --no-hooks run --skip-tools test:extension-tool-input`
in private `contract/integration/native-batch`.
Use the restricted worker wrapper recorded in the current frontier.
The default and `--allow-hidden` cases are separate controls;
changing the allowlist is not required to retain a rejected or excluded original receipt.

### Rejected remedies and upstream filing decision

Do not infer original-input absence from a filtered session registry.
Do not weaken the source-domain requirement,
downgrade Pi,
or guess a package entry filename to avoid these fixture and stager errors.

No upstream filing is warranted:

- Upstream fault is not established;
  the failures were owned test and staging assumptions.
- Fixability is demonstrated by the consumer changes and native invocations.
- Supported SDK allowlisting and package entry metadata remain unchanged.
- Contribution policy was not investigated because no upstream defect is being proposed.
- Maintainer willingness is not asserted and does not override the failed fault condition.
- No upstream patch prototype is needed for these consumer-side corrections.

There is no upstream issue or comment draft to add.

## Private document verification still imported the deleted Markdown CLI

### Symptom and owner change

`proc_e70c` failed before linting or rendering:
Node reported `MODULE_NOT_FOUND` for
`@monochromatic-dev/cli-markdown-lint/ts/rule/index.ts`.
Private `contract/native-docs/native-lint.mjs:17` still imported the former rule API.

Main commit `5bc05682ec` deleted that package.
The replacement is the installed `monochromatic-lint` 0.1.1 executable,
not another location for the old TypeScript exports.
The [unified-linter cutover](../handover/unified-linter-cutover.md) records the landing on 2026-10-10.

### Correction and verification boundary

The maintained `native-guidance-resumption-2026-10-07/documents.mjs` runner
now invokes the current executable in stdin mode,
with the actual document filename,
current repository configuration,
and concurrency one:

```javascript
// Private document runner, current CLI arguments
['--config', join(main, 'monochromatic-lint.config.jsonc'),
 '--stdin', '--stdin-filename', path, '--concurrency', '1']
```

Every document path first receives a clean heading control
and a deliberately punctuated heading that must report `markdown/no-trailing-punctuation`.
Actual source must produce zero findings and remain unchanged.
Micromark still renders every checked document and verifies the written HTML.
The qualified runtime policy parser is unchanged;
this is only document verification.

First current-linter run `proc_9685` reached the troubleshooting document
and found `rust/require-rustdoc` warnings on its Mise source excerpts.
The current executable also checks embedded Rust.
Each excerpt now has a file-documentation sentence and its original authored file citation;
the quoted executable code is unchanged.
An intermediate run `proc_8c6e` reported `markdown/semantic-line-breaks`
on commas in the citations when they were used directly as Rust documentation.
The documentation sentence avoids that punctuation while retaining the exact citations.
No warning filtering,
rule suppression,
or configuration weakening was added.
Corrected `proc_4b01` passed all 14 documents with zero diagnostics,
including per-path clean/error controls and written Micromark output.
The command remains `mise --no-env --no-hooks run documents`
in private `contract/collector/native-guidance-resumption-2026-10-07`.

### Rejected remedies and filing decision

Do not restore the deleted package,
silently omit linting,
or rewrite the runtime parser to solve this document-runner failure.
The old adapter is retained for historical consumers,
not represented as working against the current repository.

Nothing is proposed upstream.
This was an owned stale consumer after an accepted repository cutover,
so the upstream-fault condition fails.
Upstream fixability,
supported-use,
contribution,
maintainer-intent,
and upstream-prototype conditions are not claimed.

## Private native build rejects retained imports after a filesystem move

### Symptom and source trace

Restoration `b6943a8` / `proc_931a` failed before tests in private `native-batch/prepare.mjs`:
`NativeBatchBuildError: Native SDK edit does not have one matching location`.
The missing literal named `native-script-value.mjs` under `/var/mnt/encrypted/temp/agent/`;
the retained generated import used `/var/home/user/temp/agent/`.

Both spellings still resolve to the same file,
as checked with `realpath`.
This is a build-time path-comparison failure,
not a result from the restored prompt observation or the earlier full-policy incident.

Private `contract/integration/native-batch/prepare.mjs:32` required exactly one textual match:

```javascript
// Private prepare.mjs, replaceOnce
if(at<0||source.indexOf(from,at+from.length)!==-1)
 throw new NativeBatchBuildError('Native SDK edit does not have one matching location: '+from);
```

The historical `native-return-consumer.mjs:2` import was generated from its original location.
The fresh expected URL came from `import.meta.url` at the new canonical location.
No native SDK source change was needed to reproduce the mismatch.

### Consumer correction and verification

Private `staged-import.mjs:7` selects exactly one generated named-import line.
It decodes only that build-time module specifier,
rejects query/fragment variants,
and requires the existing file to resolve to the expected incumbent build input:

```javascript
// Private staged-import.mjs, expected target check
if(realpathSync(fileURLToPath(observed))!==realpathSync(fileURLToPath(expected)))
 throw new StagedImportError('Generated import differs from its expected local build input');
```

Only the fresh output buffer is rewritten.
Historical artifacts,
installed modules,
request parsing,
source capture,
and runtime authority are unchanged.
This check does not authenticate module contents or establish policy authority.

`b60415d` / `proc_0064` passed `test:staged-import`:
original and symlinked paths work;
equal bytes in another file,
duplicate declarations,
missing files,
query/fragment URLs,
non-file URLs,
and malformed declarations reject.
The subsequent actual `prepare` passed with SDK 1.1.0 and unchanged installed sources.
The task runs through `mise --no-env --no-hooks run test:staged-import`
in private `contract/integration/native-batch`.

### Tradeoffs and rejected approaches

The retained alias must remain accessible for its untouched historical imports.
This is not migration of the historical artifact graph.
Fresh preparation adds filesystem checks and fails if either target cannot be resolved.
Disposable aliases exercise both accepted spellings without replacing the real mount.

Hardcoded alternatives for the old and new roots were not adopted:
they would encode environment-specific path spelling without verifying the current target.
Regenerating old hash-locked chains,
rewriting historical files in place,
or enabling symlink-preserving module loading was unnecessary.
Target-guard omission `e38b148` / `proc_cfc3` failed `Missing expected exception (StagedImportError)`
on the equal-byte wrong-file case.
The guard was restored before complete native regression `proc_6645`,
which passed including the import controls and complete runtime suffix.
This repairs the measured preparation boundary without relocating the rest of the historical graph.

### Upstream filing decision

Nothing to file.
The failure belongs to this private build generator,
not Pi or Node.
The upstream-fault condition fails;
upstream fixability,
supported-use,
contribution,
maintainer-intent,
and upstream-prototype conditions are not claimed or investigated.
The consumer correction remains private.

## Empty Git index lock interrupted setter documentation

On 2026-10-11,
main documentation process `proc_8254` stopped before `git add`.
Cli-git emitted `index-lock-unproven-owner` after its 1000 ms unproven-owner budget.
The existing user-owned lock was empty,
device 55,
inode 237840036,
with modification time `2026-10-11T15:09:26.069Z`.

Current-user process inspection found Git filesystem monitors,
not an active Git writer.
A first shell descriptor probe did not expose a holder and was not treated as a valid null control.
Managed `proc_64ca` kept the parent shell alive after `fuser`;
the positive control reported its PID 2340389.
After that process exited,
`fuser` reported no holder.

The recovery rechecked the exact device,
inode,
size,
user ID,
and modification time before a non-overwriting move.
The empty record was preserved as `~/temp/agent/auto-mode-main-index-lock-237840036.empty`.
The source lock was then absent,
and the preserved file retained zero bytes and its modification time.
No index content,
ref,
or journal was restored.
The scoped retry `proc_7a6b` completed as `b9f38eb1a4`.
The creator and reason for the abandoned lock remain unknown.
This follows the evidence-preserving procedure in
[the separate empty-lock recovery](cli-git-superseded-index-recovery.md#separate-empty-index-lock-on-2026-10-09),
not a generic permission to remove live locks.

An earlier read-only status invocation emitted the logger's
`sink verification failed for entry 3: Timed out after 5000ms: sink 3 verify`
diagnostic before the command runner's ten-second timeout.
A subsequent managed status completed.
These observations do not prove the logger caused the lock or that its delay was fixed;
see [the logger diagnostic](cli-git-logger-sink-verify-timeout.md).
Neither incident is a failure of native setter capture.

## Manager append capture invalidates emission-time policy sources

### Original input loss before construction

Pi 1.1.0 `dist/core/sdk.js:84` reads the manager's resolved context:

```javascript
// Pi 1.1.0 dist/core/sdk.js:84
const existingSession = sessionManager.buildSessionContext();
```

The same function passes `existingSession.messages` into a literal Agent `initialState`.
An original message can already be absent from that context.
Required red `4c74fc7` / `proc_5d44` appended user text to an owned in-memory manager,
then hid it with `appendContextEdit(id,null)`.
Native delivery succeeded,
but review failed
`The original manager append input must survive a pre-construction context edit`.

The original append API has a pre-body observation point:

```javascript
// Pi 1.1.0 dist/core/session-manager.js:827 to 837
appendMessage(message) {
    const entry = {
        type: "message",
        id: generateId(this.byId),
        parentId: this.leafId,
        timestamp: new Date().toISOString(),
        message,
    };
    this._appendEntry(entry);
    return entry.id;
}
```

The selected fields must be captured before the incumbent private copy can reject the input.
Installed `_appendEntry` does not perform that clone;
private `contract/integration/native-batch/root-source.mjs` adds it.
A private clone-rejection control is not installed-SDK clone parity.

### Unrestricted capture conflicts with current source bindings

Private `bf5aff8` observed every append invocation.
The first consumer `proc_0975` stopped before review with
`Observed request is no longer current`.
Diagnostic `0f05a19` / `proc_323e` retained the original subscribed judgment-start error.
It had cause
`Native run prompt was replaced after this source snapshot`.
There was one request,
zero assessments,
and no latched content-profile failure.

Native Session persistence appended the assistant response before judgment start.
The manager's new constructed input caused the run owner to publish a new retained-input view.
The existing resource collector requires the original view:

```javascript
// Private contract/collector/rule-relevance-sdk-copy/stage-private/resource-owner.mjs:204
if(readSessionRun(session)!==saved.run)
  throw new SourceCollectionError('Native run prompt was replaced after this source snapshot');
```

Changing that check alone is not a remedy.
Private `governing-source.mjs`,
`registered-authority.mjs`,
`run-messages.mjs`,
and the judgment's rule index all use `receipt.sources`.
Allowing new membership without changing those bindings would silently omit it from policy review.

### Bounded implementation and remaining gap

The completed initial checkpoint observed appends only before the first original Session context-copy attachment.
It reuses the existing one-way `contextCopyAttached` state and manager input pool.
It adds neither a temporary mute window nor authentication by native callsite.
The source origin is `native-manager-initial-input`;
the `managerInput` location has method metadata and no fabricated ordinal.

Narrowed `740ffae` / `proc_22d7` passed the canonical pre-attachment consumer.
Run private `test:manager-append-source` through `mise --no-env --no-hooks run`.
The original hidden input remained available without appearing in the original model-request body
or gaining authority or payload eligibility.
`ffa4e19` / `proc_f1e3` also passed private clone rejection:
no entry was inserted,
but the original input still reached review.

Owner `9c9885b` / `proc_0d67` passed profile and shared-bound controls.
Its negative-zero red exposed argument normalization by JSON serialization;
the profile now rejects negative zero and undefined argument values,
while a positive-zero control passes.
Native `3e4dfd2` / `proc_7543` passed actual strict frames,
private-copy/reentry/getter/proxy parity,
original unexpected errors,
and attachment/reset lifetime controls.
Its comparison removes only the new hook from a disposable private generated manager.
It does not claim installed-SDK cloning parity.
Integrated `proc_4882` passed 128 constructed-input outputs with empty child stderr.

All four deliberate omissions failed their intended controls:
`proc_d58c` lost original input,
`proc_1c79` lost clone-rejected input,
`proc_2dbc` reproduced the pre-review snapshot failure,
and `proc_a769` admitted an unsupported content block.
All guards are restored,
with empty runtime diff against `9c9885b`.
Complete one-shot native `proc_9e75` passed in 624 seconds,
with the 128-output constructed-input matrix and all six full-policy profiles.
Complete action-policy and policy-source suites passed `proc_3bea`.
Only documentation changed during those runs.
Outcome document checkpoint `proc_a64d` passed all 14 documents with zero diagnostics.

This does not account for post-attachment manager appends.
Those potentially governing inputs remain unresolved.
Do not infer that native persistence is nongoverning,
drop it by role,
defer it silently to another request,
or relax a retired receipt.
Broader support needs coherent original-request and judgment source bindings,
not just a freshness exception.
That checkpoint implemented no such contract change.
The subsequent handoff is described in "Current judgment sources without changing emitted material".

This is private integration behavior,
not an upstream SDK fault.
Upstream fault is not established;
upstream fixability,
supported-use,
contribution acceptance,
maintainer intent,
and prototype conditions are not claimed.
There is no external filing or additive upstream report.

### Current judgment sources without changing emitted material

Post-attachment required red `b650661` / `proc_72ea` completed a native request and assessment,
but failed to retain an append hidden by a null context edit after Session attachment.
Connected `6f71097` / `proc_dc02` passed that consumer using a separate judgment-source binding.
The original receipt,
request body,
emission snapshot,
and material contributions remain historical original observations.

The private collector's existing `snapshots` ownership map authenticates both snapshots.
The handoff requires unchanged policy/resource/base identities and copied material,
unchanged non-retained run fields,
and an exact original-reference prefix extended only by unregistered manager inputs.
Judgment indexing,
wire content,
and manager input views use the bound current snapshot.
The resource collector's strict check is not weakened.
After binding,
any further source change retires the original judgment.
The emission-only `system-view.mjs` profile intentionally refuses the expanded source set.

Run `mise --no-env --no-hooks run test:manager-live-paths`
from private `contract/integration/native-batch`.
`a4a7b43f` / `proc_9b65` passed hidden inputs,
a host-called public `message_end`,
post-emission text and system-tool inputs,
and post-binding retirement.
The checks require exact `originalRequestBody`,
new sources in judgment/wire but not emission sources,
and native assistant persistence in the current source set.
All cases made no protected execution or provider requests.
`proc_4412` passed owner and handoff controls,
including irreversible failure,
reentry,
root/cancellation/generation changes,
and the narrow profile's refusal.
Its non-native reader and producer adapters are explicit;
those controls are not additional native Session executions.

### Handoff lifecycle and wiring sensitivity

Native lifecycle `12ea892b` / `proc_83f6` passed the handoff clock positive control,
expiry during source comparison before judgment publication,
unchanged-view capture failure,
shared occurrence exhaustion,
and late reset,
disposal,
reload,
and oversized input.
The clock is an explicit fixture;
the Session and judgment paths are native.
The positive control retains the original 5000-millisecond deadline.
Expired handoff work sends no assessment and publishes no judgment.
`7e7edd94` / `proc_2872` also retained a post-emission append rejected by the private clone.
No entry insertion,
authority,
or successful persistence is inferred from capturing that input.

Constructed-input regression `e32496ec` / `proc_a9e3` passed 133 outputs with empty child stderr.
The newly integrated lifecycle and clone cases still need the full native parent.
Indexed-question positive control `fb444692` / `proc_4150` passed.
Omitting observation or current message-source binding lost the original input.
Omitting the handoff or current governing-source binding caused stale-snapshot failure.
Omitting current indexing failed wire source/ordinal coherence.
Omitting copied-material equality admitted changed adapter material.
All omissions are restored in `54575a9a`,
with an empty scoped runtime diff against `7e7edd94`.
The complete action-policy and policy-source suites passed `proc_417a`,
with 29 successful result records.
Full native `proc_a3df` stopped in `default-payload-copy` because no review was returned.
Its original exception was not retained,
so no cause is established.
The isolated native-default/copy pair passed `96a0c52e` / `proc_e195` without a runtime change;
that later pass is not a diagnosis or fix.
The second parent `proc_b1f3` stopped in the distinct `carrier-bound` case,
whose original exception was also not retained.
Both fixtures now retain batch errors and report total judgment-start to reviewed time.
Repeated configurations passed `bef690b2` / `proc_a7bd`:
carrier-bound totals were 4984.015445,
5402.251908,
and 5789.265434 milliseconds;
copy-case totals were 971.362350,
914.014512,
and 1024.266656 milliseconds.
These include work after evidence closure and are not deadline-failure measurements.
Neither failed parent has an established cause or a claimed runtime fix.
Parent `proc_05da` passed 35 complete prefix outputs,
including 142 constructed-input cases and 69 section cases,
then stopped at an older original-judgment source-list expectation.
`adfb8fd9` / `proc_a45e` passed the corrected source-list assertions in all four variants.
The assertions retain exact manager source references,
roles,
content,
and emission-versus-judgment membership.
Prompt and handler fixtures also needed their newly captured persistence inputs counted.
Those corrections passed `proc_5415` and `proc_7fa1` before the next boundary was reached.
Diagnostic `9f8e832f` / `proc_b4f5` measured 69 frozen sources in a shared-session carrier case,
exceeding the unchanged 64-source limit.
Independent carrier cases now use disposable sessions;
the original shared-session rejection is retained as a separate bound control.
Runtime suffix `eaa78d4a` / `proc_5185` passed all 39 result records,
including the retained bound rejection and all six full-policy profiles.
The final one-shot native parent `3ea75e62` / `proc_d937` then passed:
82 complete result records,
142 constructed-input cases,
69 section cases,
and all six full-policy profiles.
The parent had no nonroutine stderr;
child matrices reported empty stderr.
Complete action-policy and policy-source verification passed `proc_417a`.
All deliberate omissions remain restored.
Only documentation and standalone,
non-wired intake tests changed during the final parent.
The earlier unattributed parent failures remain historical failures,
not explained by this later pass.

The governing-source omission first exposed no original reviewer error in judgment finalization.
An `evidence-closed` record is not a completed policy review.
A fixture-only batch wrapper retained the propagated exception:
`PreparedBatchExecutionError` contains
`SourceCollectionError: Native run prompt was replaced after this source snapshot`.
The indexing omission separately retained
`RuleAssessmentWireError: Selected clause differs from its complete source`.
Do not merge these with capture-profile failures or infer causes from an empty failure field.

### Preserve received system tool records

Source review identified a separate omission:
the manager system selector kept `content` and `sections`
but dropped `toolsAdded` and `toolsRemoved`.
Owner red `d8175179` / `proc_3b4d` reported `AssertionError [ERR_ASSERTION]`
with the missing declaration description,
schema prose,
constrained-sampling configuration,
and removal record in the expected source.
This is a private selector error,
not a Pi producer defect.

Pi AI 1.1.0 already normalizes parameters before publishing its declaration:

```javascript
// Pi AI 1.1.0 dist/utils/transcript.js:106 to 113
export function toToolDeclaration(tool) {
    return {
        name: tool.name,
        description: tool.description,
        parameters: JSON.parse(JSON.stringify(tool.parameters)),
        ...(tool.constrainedSampling === undefined ? {} : { constrainedSampling: tool.constrainedSampling }),
    };
}
```

Private `1c6985cf` retains the received arrays under the existing manager copier budget,
without calling that normalization again.
The incumbent `native-content-json` encoding carries their complete supported JSON records.
Accessors,
proxies,
negative zero,
and undefined nested values fail the selected profile without changing native operation behavior.
Manager initial/current phases share this selector;
loaded,
Agent prompt,
and setter profiles are unchanged.
`proc_88a2` passed the owner red after correction,
and the real post-emission declaration/removal consumer passed `proc_9b65`.

The complete native parent,
action/source regressions,
and wiring sensitivity in "Handoff lifecycle and wiring sensitivity" passed.
Additional original fields can exhaust the unchanged source and aggregate bounds sooner.
Upstream tool-definition inputs before declaration production,
images,
complete governing scope,
and semantic qualification remain independent gaps.
Do not replace the handoff with a freshness exception,
role-based exclusion,
source deduplication,
or a new request encoding.
Installed sources remain unchanged.
The private-ownership filing decision in "Bounded implementation and remaining gap" still applies;
there is no new upstream fault or filing artifact.

## Direct Agent state assignments bypass prompt and queue capture

### Symptom and native setter

Required red `cd5fd32` / `proc_896e` assigned text through the native setter,
cleared it before a request,
and failed `The original native input must survive handling or expansion`.
The request itself completed.
No installed code was changed.

Pi 1.1.0 core `dist/agent.js:51` copies the incoming array:

```javascript
// Pi agent-core 1.1.0 dist/agent.js:51 to 53
set messages(nextMessages) {
    messages = nextMessages.slice();
}
```

The closure owns the state,
so the setter receiver does not establish a different message owner.
Native Agent reset and Session `_refreshFinalizedContext` also assign through it.
Observing every internal projection as a fresh text-only host input would misclassify derived content
and add unsupported accumulated histories to that profile.

### Private implementation and verification

The private generator observes actual public setter arguments before native `slice`.
A per-state private token identifies only original native setter invocations from maintained internal sites.
A target or descriptor mismatch falls back to ordinary assignment.
Replacement setters never receive the token,
and normal reentrant calls have no token.
The implementation does not mute observation dynamically.

The internal function is obtained from the existing one-time Agent attachment
and validated against that Agent before storage in a private Session field.
It is passed as a separate constructor argument,
not a new getter on borrowed options.

Post-construction Session attachment was rejected.
The constructor reaches `_applyToolLoadout`,
which calls user code at `dist/core/agent-session.js:1178`:

```javascript
// Pi 1.1.0 dist/core/agent-session.js:1178
const changes = definition.prepareLoadout?.(loadout);
```

That callback can use the already-bound extension interface to append a custom message,
which refreshes projected state.
`proc_91c7` exercised this path through real `pi.sendMessage`,
retained its original custom input,
and asserted no new state-input publication for the native projection.

Initial canonical `proc_1f5b`,
native `proc_67c2`,
constructor `proc_91c7`,
and live lifetime/bound `proc_7faa` controls passed.
Native tests cover shallow copies,
strict/unmapped frames,
reentry,
forged extra arguments,
descriptor replacement/flags,
foreign targets,
native falsy slice failures,
proxy read parity,
and unexpected observer errors.
Run private `test:agent-state-native` and `test:agent-state-paths`
as separate `mise --no-env --no-hooks run` invocations.

### Retained-projection fixture reconciliation

Restored `8cb4a62` / `proc_d6c9` exposed a legacy fixture that used public assignment
to imitate native projection refresh.
Its expected request count was four,
but only two requests completed.
Diagnostic `d36500e` / `proc_8730` checked the capture latch immediately after assignment
and exposed `Native Agent input requires a supported text-message role`.
This was profile rejection,
not merely a changed occurrence count.

The selected public-input profile rejects unsupported roles in `loaded-inputs.mjs:33`.
The fixture instead needs native projection behavior.
Pi 1.1.0 `dist/core/agent-session.js:1016` provides it:

```javascript
// Pi 1.1.0 dist/core/agent-session.js:1016 to 1018
refreshContext() {
    this._refreshFinalizedContext();
}
```

The declaration in `dist/core/agent-session.d.ts:388` is synchronous:

```typescript
// Pi 1.1.0 dist/core/agent-session.d.ts:388
refreshContext(): void;
```

Private `37124ab` uses that real method for compaction/edit/fork projection fixtures.
The explicit stale-path clear remains an ordinary observed assignment,
with its new empty-array source asserted.
`proc_eff0` passed all 43 expanded/retained modes with empty child stderr.
The separate direct-equal-projection negative remains in `agent-state-native.test.mjs`.
No text profile,
source count,
or byte bound was relaxed.

The tradeoff is intentional:
public assignment of a derived projection is still a fresh host input,
whereas native refresh identifies a maintained internal derivation.
After adding an observation point,
inspect maintained fixture writes to distinguish those operations before broad regression runs.

### Setter sensitivity and fixed owner

All five omissions failed their intended assertions:
`proc_0ef3` lost erased input,
`proc_b838` accepted a forged token,
`proc_4a25` bypassed a replacement setter,
`proc_8953` reminted constructor projection,
and `proc_c53f` reached a config getter before capability rejection.
All guards were restored.
Both scoped runtime diff and exhaustive changed-path inventory were checked against `7d42311`.

Extended native `69deaca` / `proc_be71` passed shared count and serialized-byte bounds,
native assignment despite expected profile failure,
direct constructor-callback writes,
and real direct Sessions sharing one attached Agent.
An unqualified second Session does not receive or replace the first Agent's attachment.
Its ordinary writes are observed by that Agent's fixed original input owner.
This does not authenticate the calling Session or transfer governing scope.
Multi-Session sharing is not claimed as qualified execution.

### Limits and filing decision

The internal route identifies its operation only.
Unknown manager/model content is not authenticated or silently declared covered.
In-place array mutation,
constructor state,
and replacing state objects or descriptors remain separate gaps.
All existing source limits and admission holds remain unchanged.
Integrated and restored sensitivity checks passed.
Complete one-shot native parent `proc_428d` passed in 719 seconds,
including the current constructed-input matrix and all six full-policy profiles.
Complete action-policy and policy-source suites passed `proc_b3aa`.
Only documentation changed during regression execution.
Outcome document checkpoint `proc_99f8` passed all 14 documents with zero diagnostics.

Nothing is proposed upstream.
The SDK setter behaved as written;
this is private source observation.
The upstream-fault condition fails;
upstream fixability,
supported-use,
contribution,
maintainer-intent,
and upstream-prototype conditions are not claimed.

## Native prompt arrays include a system-section delta

### Symptom and deciding source

The first direct message/array capture connected isolated consumers,
but `proc_d774` never created the late review.
Diagnostic `proc_8744` showed zero requests and assessments.
`proc_bc9b` exposed the manager's `SessionInputCaptureError`,
whose cause was `Native Agent input requires a supported text-message role`
inside `agentPromptArray`.

The owned mistake was reusing the queue role subset for every prompt-array member.
Pi 1.1.0 `dist/core/agent-session.js:1307` builds a system-section delta:

```javascript
// Pi 1.1.0 dist/core/agent-session.js:1307 to 1308
const sections = diffSystemPromptSections(getCurrentSystemMessage(messages)?.sections ?? {}, buildSystemPromptSections(options));
return sections ? { role: "system", content: "", sections, timestamp: Date.now() } : undefined;
```

The same file at line 1628 prepends it before calling the Agent:

```javascript
// Pi 1.1.0 dist/core/agent-session.js:1628 to 1631
if (updateMessage)
    messages.unshift(updateMessage);
preflightResult?.("started");
await this._runAgentPrompt(messages);
```

Single user-message direct calls did not exercise that Session construction.
Their earlier pass did not establish Session-array compatibility.

### Correction and remaining verification

Prompt capture now reuses the loaded-system selection and validation for content and sections.
Null deletion remains original input data,
not evidence of revoked instructions.
The prompt role adds no authority,
and tool definitions remain outside this selected prose representation.
The queue profile is not expanded.

The existing native review assertion stays intact.
Owner system-field controls include absent,
null,
and named sections,
plus zero reads of tool getters.
An intermediate correction failed `proc_8936` because its loop-local `message` shadowed the array parameter.
Renaming the selected element to `entry` preserved the receiver.
Corrected `e1fbda6` / `proc_e97e` passed system owner controls
and all eight real late/oversize/reset modes.
Direct system-message and system-array canonical consumers passed `proc_f007`.
Integrated `proc_a44a` passed 116 outputs with empty child stderr.
Observation/section/proxy omissions `proc_0e00`,
`proc_883a`,
and `proc_7d1b` failed their intended controls and were restored.
The full native case set is covered by the unchanged source prefix of `proc_5868`
and corrected complete runtime suffix `proc_45e7`,
not a passing one-shot parent invocation.
Complete action-policy `proc_4f7e` and policy-source `proc_0091` passed.
Run private `test:agent-message-prompt-owner` and `test:agent-message-prompt-paths`
as separate `mise --no-env --no-hooks run` invocations.

The separate `custom-trigger-bound-allowed` fixture initially checked its pre-run trigger
only after another Session prompt forwarded its pending large messages as one array.
`proc_3b98` then reported a terminal error instead of the expected stop.
Correction `3f7d81f` / `proc_2853` preserves the original base-bound success and rejection checks,
then explicitly verifies the later aggregate array exceeds the separate source bound before transport.
Neither limit was enlarged.

### Conversion hints do not imply image omission

Full regression `proc_45f1` reached `retained-image`,
which now fails the text-only prompt-array profile before policy review.
The first fixture correction still expected `retained-image-hint` to reach review;
`proc_21bd` disproved that expectation.

Pi 1.1.0 `dist/utils/image-process.js` normalizes the valid GIF fixture
from `image/x-gif` to PNG and returns both image data and a conversion hint.
`dist/core/agent-session.js:1503` retains both:

```javascript
// Pi 1.1.0 dist/core/agent-session.js:1503 to 1504
normalizedImages.push({ type: "image", data: processed.data, mimeType: processed.mimeType });
hints.push(...processed.hints);
```

The fixture is converted-image-plus-text,
not omitted-image hint-only text.
Corrected `766f4d4` / `proc_a122` passed all four image/conversion modes:
native image construction,
actual conversion hint where applicable,
text-profile failure,
zero transport/assessments/executions,
no text payload capability,
and no revival after a later plain prompt.
No runtime guard or limit changed.
A genuinely omitted-image hint-only route is not claimed as exercised by these fixtures.

### Keep full-policy gates independently reachable

Complete native `proc_5868` reached full-policy verification.
The single-policy case indexed 207 clauses and dispatched 414 legacy questions.
The visible independent copy stopped before judgment construction.
`proc_2bba` exposed the original capture cause:
`Loaded session message content exceeds the original source byte bound`.
The rendered prompt measured 64,559 bytes,
but the encoded selected array crossed the separate 65,536-byte source limit.

The fixture retains that original rejection.
A second independent-copy case replaces only rendered project context,
retaining both unchanged source inputs and requiring all 414 clauses at the existing clause gate.
The first attempted empty override did not hide context:
Pi's `dist/core/system-prompt.js:111` only applies truthy custom sections:

```javascript
// Pi 1.1.0 dist/core/system-prompt.js:111 to 114
for (const [name, content] of Object.entries(customSections)) {
    if (content)
        promptSections[name] = content;
}
```

The corrected case uses a nonempty rendering placeholder,
not altered policy text.
It checks the actual prompt during `originalStream`;
checking `session.systemPrompt` after settlement instead sees the base prompt,
as the getter selects `_runSystemPromptOptions ?? _baseSystemPromptOptions`
at `dist/core/agent-session.js:1058`.

The default-system cases also retain a separate direct Agent string call
that reaches the original rendered-prompt byte guard without an aggregate array at API entry.
`proc_b0c7` passed all six legacy/clause/content-use profiles:
207-clause positives,
visible aggregate-array rejections,
414-clause rejections despite replaced rendering,
and the default rendered-prompt rejection at 66,533 bytes.
No source,
question,
wire,
or time limit was enlarged.
This is a diagnosed earlier capture-bound failure,
not a cause or fix for the separate historical `proc_5939` incident.

### Upstream filing decision

Nothing is proposed upstream:
native construction is intentional and the mismatch was in the private observer.
The upstream-fault condition fails;
upstream fixability,
supported-use,
contribution,
maintainer-intent,
and upstream-prototype conditions are not claimed.

## Pi 1.1.0 direct Agent string prompts bypass session text input capture

### Symptom and native boundary

Direct `Agent.prompt` does not pass through `AgentSession.prompt`.
Required red `proc_b58b` completed the actual direct request,
then failed the original-source assertion during canonical review.

Installed `pi-agent-core@1.1.0/dist/agent.js` checks the active run before normalization:

```javascript
// Installed pi-agent-core dist/agent.js, Agent.prompt
async prompt(input, images) {
    if (this.activeRun) {
        throw new Error("Agent is already processing a prompt. Use steer() or followUp() to queue messages, or wait for completion.");
    }
    const messages = this.normalizePromptInput(input, images);
    await this.runPromptMessages(messages);
}
```

The native normalizer preserves array input,
wraps a non-string single input,
and constructs a user message only for string input.
This private increment observes only that original string argument,
not the later user-message identity.

### Private correction and verification

The existing fixed observer runs synchronously at method entry only for string input.
The existing manager owner selects the original frame's first data descriptor
and retains text with method `prompt`.
Images are not inspected by the observer.
Input-only membership supplies neither role authority nor payload association.
Non-string forms remain unobserved by this branch.

Run `test:agent-prompt-source` and `test:agent-prompt-native` separately
through `mise --no-env --no-hooks run` in private `contract/integration/native-batch`.

Canonical `proc_960b` passed both the real direct request and active-run rejection/source-retirement cases.
Native/owner `proc_f1aa` passed exact frames,
normalization output,
unchanged image references,
zero observer image reads,
unmapped frame mutation,
untouched non-string forms,
source limits,
failure latching,
and original unexpected observer failure.
The normalization delegate is a double,
not a provider or qualification trial.

### Remaining scope and upstream filing decision

Integrated `proc_0117` passed 103 outputs with empty child stderr.
Predicate omission `proc_b636` poisoned non-string capture;
observation omission `proc_58f8` lost the required original string source.
Both were restored before complete native `proc_6645`,
which passed after the separate build-path correction.
Complete action-policy `proc_379b` and policy-source `proc_580e` also passed.
Message/array prompt inputs,
image governance,
other state producers,
and governing-domain closure remain unfinished.
Nothing is proposed upstream:
native prompt behavior remains unchanged,
and this is a private observation extension.
No installed activation or production policy admission changed.

## Pi 1.1.0 direct Agent queues bypass session input construction

### Symptom and native boundary

Direct core-Agent queue calls do not pass through `AgentSession` text entry or user-message construction.
Required red `proc_d56b` enqueued a caller-owned message,
cleared it through native `clearAllQueues`,
then failed with zero original sources instead of one.

Installed `pi-agent-core@1.1.0/dist/agent.js:60`
stores the message itself,
not a new queue-entry wrapper:

```javascript
// Installed pi-agent-core dist/agent.js
enqueue(message) {
    this.messages.push(message);
}
```

At lines 184 and 189,
`Agent.steer` and `followUp` synchronously call their queue's `enqueue`.
These methods have no separate processing-state or message-validation branch.

Installed `pi-coding-agent@1.1.0/dist/core/sdk.js:77`
constructs/selects the session manager before `new Agent` at line 253.
That existing factory boundary supplies the fixed observer's manager without a new registry.

### Private correction and verified controls

The existing generator attaches the observer immediately after native Agent construction,
without reading any additional borrowed constructor option.
A once-assignable private Agent field holds it.
Actual queue-call arguments frames carry the new input-only publication;
the message itself keeps any earlier publication.

The manager captures only supported content and companion metadata,
using the existing bounded copier,
source record,
constructed-input reference view,
and shared caps.
It publishes no payload capability or execution fact.
Native enqueue still runs after expected source-profile failure;
unexpected observer failure retains its original thrown value.

Run `test:agent-input-source`,
`test:agent-input-native`,
`test:agent-input-owner`,
and `test:agent-input-paths` separately
through `mise --no-env --no-hooks run` in private `contract/integration/native-batch`.

Canonical `proc_afcc` passed.
Native `proc_0d29` verified queue references,
attachment rejection cases,
constructor-getter parity,
actual SDK manager isolation,
same-message mutation,
prior publication preservation,
capture failures,
queue failures,
and disposed-session source refusal.
The owner phase of `proc_c501` passed profile and shared-bound checks.
Consumer `proc_7763` passed delivery,
clear,
late,
oversize,
reset,
and disposal paths.

### Profile tradeoff and corrected fixture evidence

The supported direct-Agent source profile is text-only.
System,
mixed image,
and other non-text representations latch capture failure,
including internal forwarding through an already-observed `AgentSession` API.
Native queuing still retains the exact original message.
Image fields are not emitted as semantic source material,
and borrowed messages/images are not frozen.
This is not a complete image-origin claim.

`proc_c501` later failed the old queue fixture:
its initial image call latched source failure,
so the next expected source was absent.
The fixture now separates supported-text success from mixed-image source failure.

The first separated run `proc_77d0` also failed:
an earlier fixture had injected an enqueue failure only into the private SDK,
which retained steering text before the throw.
Its later queue-clear result therefore differed from the installed SDK's result.
The progress record's premature pass claim was retracted.

Corrected `proc_54e8` injects the earlier failure into both SDKs,
asserts identical retained steering text,
clears it,
then passes the image/reference comparison with the expected source-profile failure.
No native state behavior or source guard was changed.

### Remaining scope and upstream filing decision

Integrated `proc_b210` passed 97 constructed-input outputs with empty child stderr,
including actual settled-event queueing before clear.
Rethrow omission `92fe738` / `proc_7cf0` failed with `Missing expected exception.`
SDK attachment omission `b3c1431` / `proc_7234` and queue-observation omission `d34d75c` / `proc_cd4a`
each lost the original cleared input.
All guards and observations are restored.
Complete native `proc_c273` passed in 428 seconds.
Complete action-policy `proc_43c5` and policy-source `proc_89b0` also passed.
These local checks establish the bounded mechanism,
not empirical qualification or a production admission policy.
Other direct Agent producers,
image governance,
and full governing-domain closure remain unfinished.
Nothing is proposed upstream:
this is a private observation extension and fixture correction,
not a native queue defect.
No installed activation or production policy admission changed.

## Pi 1.1.0 public input calls can reject before input construction

### Symptom and native boundary

Known commands rejected by queue APIs do not reach either command-context construction or input events.
A throwing prompt-option getter can likewise stop before input handling.
Required red `proc_30b3` preserved the native queue rejection,
then failed the original-source assertion during later canonical review.

Installed `pi-coding-agent@1.1.0/dist/core/agent-session.js:1517`
starts prompt deferral before input construction:

```javascript
// Installed pi-coding-agent dist/core/agent-session.js
async prompt(text, options) {
    if (this._isEmittingAgentSettled) {
        this._deferredSettledActions.push(async () => await this.prompt(text, options));
        return;
    }
```

At lines 1716 and 1727,
the public queue methods receive text first and evaluate option source before queue handling:

```javascript
// Installed pi-coding-agent dist/core/agent-session.js
async steer(text, images, options) {
    return this._queueUserInput(text, images, "steer", options?.source ?? "interactive");
}
async followUp(text, images, options) {
    return this._queueUserInput(text, images, "followUp", options?.source ?? "interactive");
}
```

These are not the direct `Agent.steer(message)` and `Agent.followUp(message)` producers.
Options contain metadata,
not established authority.

### Private observation and verification

The maintained generator observes the actual engine-created entry `arguments` object.
The incumbent manager owner reads only its own index-zero data descriptor,
validates an actual arguments object and supported method,
and retains an input-only publication.
It does not enumerate arguments or read options/images.
Capture is synchronous and precedes native branching.

Engine-only `proc_dd75` first checked compatibility with the existing alias owner.
It did not establish native SDK coverage.
Actual native `proc_2d41` then verified strict callee descriptors,
original frame identity,
unchanged native formal arguments after frame-only mutation,
and installed/private behavior for each public method.
The option-getter counters have native positive reads and zero added observer reads.

Run `test:call-frame-profile`,
`test:call-input-source`,
`test:call-input-native`,
`test:call-input-owner`,
and `test:call-input-deferred` separately
through `mise --no-env --no-hooks run` in private `contract/integration/native-batch`.

Canonical `proc_0ff4` passed both queue-method rejections and original prompt-option getter failure.
Owner and queue `proc_48a4` passed source-profile,
shared-bound,
collision,
and failure controls.
Expected source-profile failures latch without replacing native behavior;
unexpected failures latch and rethrow their original value.

### Rejected readings and remaining verification

The synthetic class probe alone was insufficient evidence for native frames.
Direct `Agent` message-shaped APIs are not these `AgentSession` text APIs.
Input-only aliases retain historical membership,
not current payload signatures.
Neither original invocation nor deferred replay proves a human author.

`proc_118a` exposed stale queue-fixture ordering,
not a runtime source error:
the first source was now `native-call-input`,
rather than `native-input-event`.
Explicit assertions preserve both sources and the original native queue result.

The public-entry observation adds genuine call occurrences even on successful paths.
Equal text is not deduplication authority.
All existing caps remain unchanged;
earlier occupancy exhaustion is possible.
Deferred replay is a separate physical invocation,
not a separately authenticated human instruction.

Native settled deferral/replay and reset/disposal checks passed `proc_1842`.
The deferred entry reads no options;
replay has its own physical invocation,
and prior root eligibility is not restored.
Late rejected-call and unchanged-view failure controls passed `proc_d2ba`.
Original-judgment profiles passed within `proc_0abb`;
subsequent prompt/handler fixtures now verify API-call and return sources separately.

Runtime `proc_5939` passed those custody checks,
then the full-policy fixture failed to reach review during independent-copy preparation.
Diagnostic-only retry `proc_9a1e` passed with unchanged runtime and the same policy input.
That non-reproduction proves neither cause nor fix.
The five-second budget remains unchanged,
and nested failure diagnostics are now preserved.
Integrated `proc_4a74` passed 85 constructed-input outputs with empty child stderr.
Rethrow omission `0d7076e` / `proc_82cb` failed with `Missing expected rejection.`
Entry-observation omission `5a3020f` / `proc_c64f` lost the original rejected-call source.
Both guards are restored;
no deliberate omission remains.
Complete native `proc_834c` passed in 428 seconds,
including the full-policy fixture.
Complete action-policy `proc_fcb7` and policy-source `proc_e74d` also passed.
The prior full-policy incident remains unattributed;
these later passes are not evidence of a runtime fix.
All guards and original bounds remain intact.
Direct-agent producers and complete governing-domain closure remain unfinished.

### Upstream filing decision

Nothing to report upstream:
native early-return and rejection behavior is preserved.
This is a private observer-coverage change,
with no installed activation or production policy admission.

## Pi 1.1.0 known commands bypass native input events

### Symptom and native source boundary

A known extension command can complete without passing through `_runInputHandlers`.
Private red `proc_d845` recorded its original arguments and actual handler entry,
with no input event or initial transport,
then failed `The original native input must survive handling or expansion` in later review.

Installed `pi-coding-agent@1.1.0/dist/core/agent-session.js:1636`
parses the command and returns before context construction when no command matches.
At line 1645,
the known-command branch constructs context before its handler catch boundary:

```javascript
// Installed pi-coding-agent dist/core/agent-session.js
const ctx = this._extensionRunner.createCommandContext();
try {
    await command.handler(args, ctx);
    return true;
}
```

Installed `dist/core/extensions/runner.js:724`
preserves lazy context getters through property descriptors:

```javascript
// Installed pi-coding-agent dist/core/extensions/runner.js
const context = Object.defineProperties({}, Object.getOwnPropertyDescriptors(this.createContext()));
```

The context authenticates this observed construction/handoff occurrence,
not text-bearing context fields or successful dispatch.
The native handler receives parsed arguments;
the private source retains the original full text independently.

### Private correction and verification

The existing manager owner records `native-command-input` at `commandInput`,
using the actual context as an input-only alias.
It does not copy context fields,
call getters,
freeze borrowed values,
or assign authority.
Capture precedes the native handler `try`.
Expected `SourceCollectionError` profiles latch and permit native handler behavior;
unexpected failures also latch,
then rethrow their original value outside the handler catch.

Run `test:command-input-source`,
`test:command-input-owner`,
`test:command-input-native`,
and `test:command-input-paths` separately
through `mise --no-env --no-hooks run` in private `contract/integration/native-batch`.

Canonical `proc_e670` and corrected owner `proc_24cc` passed.
Paired native `proc_cfac` exercised argument boundaries,
equal calls,
lazy getters with an intentional read control,
unknown commands,
falsy synchronous/awaited handler failures,
reporter failure,
context-factory failure,
oversize capture combined with a native handler error,
and unexpected observer failure before handler entry.
`proc_797b` exercised pre-run delivery,
overlap,
reentrancy,
late capture,
oversize failure,
and reset before consumption.

### Rejected readings and remaining scope

Owner control `proc_d918` wrongly treated a `toJSON` accessor as supported.
The incumbent `contract/collector/run-prompt-custody-v3/handler-publication.mjs:8`
excludes that property by presence:

```javascript
// Private contract/collector/run-prompt-custody-v3/handler-publication.mjs
return value!==null&&typeof value==='object'&&!types.isProxy(value)
 &&Object.getPrototypeOf(value)===Object.prototype&&!('toJSON' in value);
```

The corrected positive control uses ordinary lazy context getters;
the negative control requires hook rejection without invoking it.
No guard was loosened.

A native `true` result also follows reported handler failure.
Capture therefore proves neither successful execution nor permission.
Unknown commands and context-factory failures do not reach this capture edge.
The unchanged source-content and aggregate caps can withhold guarded continuation,
even when native command handling completes.

Integrated `proc_375b` passed 71 constructed-input outputs with empty child stderr,
including awaited-handler reset/disposal.
Rethrow omission `b30b5b0` / `proc_1854` failed with `Missing expected rejection.`
Observation omission `9bebebc` / `proc_e106` retained zero original command sources instead of one.
Both guards are restored.
Complete native `proc_1736` passed in 411 seconds;
complete action-policy `proc_4dfe` and policy-source `proc_666d` also passed.
Earlier deferred/rejected calls and full governing-domain closure are not covered.

### Upstream filing decision

Nothing to report upstream:
native command dispatch is preserved,
and this is a private observation-coverage change.
The installed plugin and production admission remain unchanged.

## Pi 1.1.0 final input transformations can disappear during template expansion

### Symptom and native boundary

Per-handler input observations cannot capture text first created by the last handler.
Required red `proc_2bf2` passed native terminal-success checks,
then failed `The original native input must survive handling or expansion` in canonical review.

Installed `pi-coding-agent@1.1.0/dist/core/agent-session.js:1484`
constructs the aggregate transformed result before template expansion:

```javascript
// Installed pi-coding-agent dist/core/agent-session.js
if (inputResult.action === "transform") {
    return { text: inputResult.text, images: inputResult.images ?? images };
}
```

Installed `dist/core/extensions/runner.js:1234` compares final values,
not whether a handler ever returned a transform action:

```javascript
// Installed pi-coding-agent dist/core/extensions/runner.js
return currentText !== text || currentImages !== images
    ? { action: "transform", text: currentText, images: currentImages }
    : { action: "continue" };
```

Consequently,
equal or restored text with the original image reference takes the continue branch.
Changed image references can select transform even when text bytes are equal.

### Private correction and verified controls

The incumbent generator observes the fresh transformed-return object and returns that exact object.
The existing manager source owner captures its text as `native-transformed-input`,
at `transformedInput`,
with no image inspection,
authority,
invented run,
or entry ordinal.
Observation is outside the native handler catch boundary.
This does not observe each intermediate handler result or grant material contribution.

Run `test:transformed-input-source` and `test:transformed-input-native` separately
through `mise --no-env --no-hooks run` in private `contract/integration/native-batch`.

Canonical `proc_03ab` passed.
Parity and owner controls `proc_83f6` passed empty/changed/equal/restored text,
handled outcomes,
falsy handler errors,
omitted/null/empty/replacement images,
chained transformations,
UTF-8 limits,
original failure latching,
foreign-manager rejection,
and reset/disposal during the awaited handler.
Observer image reads remained zero.

The unchanged ordinary source cap now also applies to the final transformed text.
An oversized final transformation is unsupported even if native expansion would shorten it.
Expected capture failure preserves native returns,
but guarded source use fails.
Reset can replace capture state,
not revive the original root lease.

### Rejected readings and remaining verification

An earlier input event is not the final transformed return.
The handler's result object is not the native aggregate return object.
Byte equality does not merge those occurrences,
and source/streaming labels do not authenticate authorship.

Integrated `proc_1851` passed 60 constructed-input outputs with empty child stderr.
The matrix includes queued/cleared,
chained,
equal-byte,
oversized-before-expansion,
and rejected-streaming transformations.
Observation-only omission `c0e74fc` / `proc_166f` failed at the committed original-source assertion.
`6e373a9` restores the observation.
Standalone source-profile preparation and complete native regression passed in `proc_3e8e`
(426 seconds for the combined command).
Complete action-policy `proc_db21` and policy-source `proc_8595` passed.
No installed source or production admission was changed.

### Upstream filing decision

Nothing to report upstream:
this is a private observer coverage gap,
not a native template or handler defect.
The native behavior is preserved.

## Pi 1.1.0 template expansion can discard unintercepted raw input

### Symptom and native source boundary

A native template without argument placeholders can replace the entire original invocation.
The private expanded-prompt capture alone therefore misses unused original argument text.
`proc_785c` demonstrated the missing source in canonical review.

Installed `pi-coding-agent@1.1.0/dist/core/agent-session.js:1476`
returns a fresh native value when no input handler exists:

```javascript
// Installed pi-coding-agent dist/core/agent-session.js
if (!this._extensionRunner.hasHandlers("input")) {
    return { text, images };
}
```

Installed `dist/core/prompt-templates.js:247` passes only template content and parsed arguments to substitution:

```javascript
// Installed pi-coding-agent dist/core/prompt-templates.js
if (template) {
    const args = parseCommandArgs(argsString);
    return substituteArgs(template.content, args);
}
```

At `prompt-templates.js:56`,
`substituteArgs` returns a replacement over template content,
not content followed by unused arguments.
A template with no matching placeholders consequently preserves none of those unused arguments.

### Private correction and verification

The maintained generator observes the original native return object before expansion.
The existing manager owner captures text and source/streaming metadata,
then the observer returns the identical object.
Image data is not read or frozen.
This creates no synthetic input event or run.

Run `test:unintercepted-input-source`,
`test:unintercepted-input-native`,
`test:unintercepted-input-paths`,
and `test:queued-input-controls` separately through `mise --no-env --no-hooks run`
in private `contract/integration/native-batch`.

`proc_bd69` passed the original raw template consumer.
`proc_af25` matched installed/private returns and borrowed image references.
`proc_a904` preserved raw sources through template queuing,
clearing,
and a native rejected streaming prompt.
`proc_753b` passed the integrated controls with empty child stderr,
including equal raw/expanded data with no raw payload upgrade,
original failure latching,
and reset before source consumption.
The source stays unregistered.

The same ordinary 65536-byte source-content cap now applies before expansion.
A raw input exceeding it is unsupported even when the native template would shorten its output.
Native return values remain unchanged,
but guarded continuation rejects the captured-source failure.
A later valid input cannot clear that failure.
Fresh native owner creation can clear capture state,
while original root retirement remains independent.
No budget was enlarged.

### Metadata proxy correction

The existing copier already rejects accessor descriptors before cloning.
The proxy case differed:
it inspected a proxy's prototype and keys before rejecting it.
Private commit `bfb344f`,
in process `input-metadata-proxy-read-required-red`,
failed `2 !== 0` for observer trap calls.
That process's short ID `proc_f207` is distinct from the historical append-persistence control with the same ID.

The original `contract/collector/run-prompt-custody/messages.mjs` now rejects proxies with
`types.isProxy` before reflective reads.
`1583671` / `proc_5258` verified zero observer getter/proxy calls.
No parallel copier was added.

### Rejected readings and remaining scope

`proc_801f` was a fixture error:
its loaded-source selector also included the new current native input.
The corrected selector requires `native-load`;
loaded `proc_e82c` and expanded `proc_3db1` regressions passed.

Complete run `proc_5485` exposed the same category of stale fixture assumption in
`original-judgment.test.mjs`:
its built-in source slice now also contained the original raw prompt.
Node reported `ERR_ASSERTION` from the fixture's wire validator,
then the original failure remained in the closed judgment with both tool members unentered.
Narrow `test:original` reproduction `proc_28b6` failed the same assertion.
`cf238c5` explicitly verifies the raw wire/source publication before the generated-fragment slice;
no source was dropped and no runtime guard changed.
`proc_5f2e` and every original-judgment source profile in `proc_1be7` passed.
Complete retry `proc_efa9` passed the source profiles but exposed another stale assumption:
settled prompt/handler snapshots expected raw inputs to disappear with the run.
`proc_18dd` also exposed stream assertions converted into native terminal error messages.
The fixtures now require successful native termination and compare original run/raw source references separately.
The existing task sequence is split at `test:runtime`,
without dropping cases,
so fixture retries do not repeat the source phase.
Runtime stage `proc_1cea` passed in 39 seconds.
Complete staged parent run `proc_075f` passed in 431 seconds.
Complete action-policy `proc_843e` and policy-source `proc_87c4` also passed.
All deliberate omissions are restored;
these are local mechanism checks,
not semantic qualification or production policy admission.

Omitting the native return observation failed `proc_8292`;
the observation is restored.
Equal bytes do not merge raw and expanded source identities or grant a payload capability.
Metadata such as `source:'interactive'` does not establish a human author.

Earlier commands,
deferred/rejected calls before this branch,
last-handler outputs before additional expansion,
direct-agent inputs,
image origin,
and complete governing scope remain separate gaps.
Skills expansion is not separately exercised by this increment.

### Upstream filing decision

No upstream report is proposed:
the template's placeholder behavior is native behavior,
and the missing observation and proxy reads were private adapter issues.
The restored native,
action-policy,
and policy-source regressions passed.
The installed plugin,
production qualification,
and automatic policy admission remain unchanged.

## Pi 1.1.0 input handlers can remove original text before prompt construction

### Symptom and native cause

The private consumer initially lost original input when an extension handled it or rewrote it.
`proc_dae2` reached canonical review after a handled input and found no original event source.
Installed `pi-coding-agent@1.1.0/dist/core/extensions/runner.js:1202`
creates an event for each snapshotted handler and then applies its result:

```javascript
// Installed pi-coding-agent dist/core/extensions/runner.js
const event = {
    type: "input",
    text: currentText,
    images: currentImages,
    source,
    streamingBehavior,
};
const result = (await handler(event, ctx));
if (result?.action === "handled")
    return result;
if (result?.action === "transform") {
    currentText = result.text;
    currentImages = result.images ?? currentImages;
}
```

Installed `dist/core/agent-session.js:1476` returns no processed input for `handled`.
The later prompt/queue constructor therefore cannot recover the original event from final text.
Byte equality with another message would not authenticate it.

### Private correction and error boundary

The existing copied emitter observes the actual event before invoking its handler.
The manager's existing source owner captures text and source/streaming metadata,
with input-only identity and no image reads.
No callback input is frozen.
Original events remain distinct even when text is equal.

The first placement put observation inside the native handler try/catch.
That could misattribute a bridge defect to the extension.
Structural control `proc_c956` supplied an authored observer throwing `false`
to the copied emitter with a real native runner.
It failed `Missing expected rejection`;
the native catch had swallowed the observer failure.
This was a private instrumentation error,
not an upstream handler defect.

The generator now constructs/observes the event before the handler try.
The constructor contains only fixed data fields from existing local values.
Handler invocation and native error reporting remain inside their original catch boundary.
Normal capture failures still latch in the manager owner;
an unexpected observer failure is not relabeled as a handler failure.

### Verification and rejected readings

Run `test:input-event-source`,
`test:input-event-chain`,
`test:queued-input-native`,
and `test:queued-input-controls` separately through
`mise --no-env --no-hooks run`
in the private `contract/integration/native-batch` directory.

`proc_7ee7` passed handled/transformed source retention.
`proc_64d3` passed chained/equal/chained-handled events and late handled/oversized input.
`proc_6bb9` passed original-owner bounds,
data-property validation,
and no-extra-image-read controls.
An intercepted queue API call has an input event plus queue construction;
`proc_5822` exposed the old single-occurrence fixture assumption,
and corrected `proc_7530` passed.
The observer-error correction's initial native consumers passed `proc_be20`.

Native source labels such as `interactive` do not establish a human author.
The no-handler fast path and commands handled before `emitInput` are not represented by fabricated events.
On 2026-10-09 the no-handler return gained a separate original-input capture;
commands handled before that boundary remain a gap.
Image origin and governing-domain completeness remain unestablished.

### Upstream filing decision

No upstream filing is proposed.
The private adapter needed an earlier observation boundary and correct observer-error attribution.
Native input-handler behavior itself is unchanged for supported capture.
Integrated `proc_a059` passed with empty child stderr.
Observation-edge omission `proc_e954` lost the original handled input and failed;
the edge is restored.
Complete native `proc_a1d5`,
action `proc_d1a1`,
source-policy `proc_62bd`,
and 14-document rendering `proc_e9ec` passed.
The installed plugin is untouched.

## Pi 1.1.0 custom message construction can precede any prompt run

### Symptom and cause

Private control `proc_2d96` found no original input source for a current custom message.
After construction capture landed,
`proc_fb7c` exposed a lost input-only alias at custom-entry construction.
`proc_07f8` then found that first-turn `triggerTurn` has no earlier native prompt-run carrier.

Installed `pi-coding-agent@1.1.0/dist/core/agent-session.js:1782`
constructs the message before choosing its dispatch route:

```javascript
// Installed pi-coding-agent dist/core/agent-session.js
const appMessage = {
    role: "custom",
    customType: message.customType,
    content: message.content ?? [],
    display: message.display,
    details: message.details,
    timestamp: Date.now(),
};
```

At `agent-session.js:1803`,
an idle triggering call goes directly to the agent prompt,
or defers that same call while settled callbacks are being emitted:

```javascript
// Installed pi-coding-agent dist/core/agent-session.js
else if (options?.triggerTurn) {
    if (this._isEmittingAgentSettled) {
        this._deferredSettledActions.push(async () => await this._runAgentPrompt(appMessage));
        return;
    }
    await this._runAgentPrompt(appMessage);
}
```

That route does not construct a user-prompt emitter occurrence.
The private source collector must not invent one merely to retain the actual custom input.
The later `proc_3e24` control exercised the actual `agent_settled` callback,
observed one deferred action while not streaming,
and verified its later original custom-message delivery.

### Private correction

The existing manager source owner captures normalized custom content and selected metadata,
excluding opaque details,
at the actual constructor.
The existing input-only alias owner now follows the custom-entry edge.
Private `contract/collector/run-prompt-custody-v3/handler-publication.mjs:126` uses:

```javascript
// Private contract/collector/run-prompt-custody-v3/handler-publication.mjs
const known=aliases.get(before);
if(known?.owner===owner&&known.kind==='input-only'){retainInput(after);return;}
```

The direct append helper passes its original `appMessage` to the same native entry observer.
This neither creates text-part eligibility nor authenticates a serialized contribution.

Private `contract/integration/policy-source/system-slot.mjs:152`
reads the original manager reference view only when no run is present:

```javascript
// Private contract/integration/policy-source/system-slot.mjs
function constructedInputs(run){
    return run===undefined&&session?readSessionConstructedInputs?.(session):undefined;
}
```

The fixed session reader checks disposal and deferred capture failure.
The incumbent collector keeps the returned original references in `nativeConstructedInputs`,
checks their identity during capture and finalization,
and retains the original one-MiB base-linked snapshot limit.
The 64-source/four-MiB owner limits are separate.
An absent reader never establishes governing-domain completeness.

### Verification and rejected remedies

Run the private `contract/integration/native-batch` tasks separately:
`test:custom-message-input`,
`test:custom-message-before-run`,
`test:custom-message-bounds`,
and `test:queued-input-controls`.
They use native APIs with authored local responses,
not provider requests.

`proc_f9b4` passed immediate,
next-turn,
hidden,
initial-trigger,
and streaming construction cases.
`proc_d5cb` rejected late pre-run additions,
capture failure,
and root reset.
`proc_fb31` matched installed/private normalization,
including null and missing content.
`proc_bbe9` passed the integrated matrix with empty child stderr.
It verifies actual continuation transmission and original entry aliases for streaming custom messages.

The pre-run budget control accepts eight pending 63000-byte inputs.
It rejects 18 before any original transport.
The first omission probe `proc_ef01` survived because a later check emitted the same error after one transport.
That is not sensitivity evidence.
The corrected assertion failed `proc_0fdc` with `1 !== 0`;
the guard is restored.

Entry-edge omission `proc_2dde` lost the saved custom-entry alias.
Pre-run freshness omission `proc_c28d` wrongly let the earlier review finish.
Both are restored.

A copied-resource-owner remedy failed `proc_15b4` with
`Native base composition lacks its exact owned resource binding`.
Copying that module separated its binding map from the incumbent owner.
The extra route and its unused generated artifact were removed.
The implementation now extends the incumbent policy-input collector.

The disposal fixture initially expected a disposed-reader error to win.
`proc_685d` showed `JudgmentCancelledError` with the original `AbortError` cause instead.
The corrected control preserves cancellation precedence and separately checks the disposed reader.
No native error was replaced to satisfy the fixture.

### Upstream filing decision and remaining scope

No upstream report or patch is proposed:
these were private adapter gaps.
Direct agent producers,
pre-handler content,
mixed representations,
applicable governing scope,
and actual source-domain closure remain unfinished.
This source subset has no authority,
delegation,
semantic qualification,
or production policy grant.
Complete native `proc_d3b5`,
action `proc_f8cd`,
source-policy `proc_21a7`,
and 14-document render `proc_3378` passed.
The settled-callback addition changes tests only;
runtime and generator source is unchanged after the full-native checkpoint.
The installed plugin is unchanged.

## Pi 1.1.0 queued text has no session entry at construction

### Symptom and native source boundary

The private provenance consumer initially omitted queued user text:
`proc_10e8` reached canonical review but found zero original queue sources instead of one.
This is a private adapter gap,
not an upstream queue failure.

Installed `pi-coding-agent@1.1.0/dist/core/agent-session.js:1690`
runs input handlers and expansions before calling the queue constructors.
At lines 1733 and 1749,
the constructors build native content and pass a new user message to the agent:

```javascript
// Installed pi-coding-agent dist/core/agent-session.js
const content = [{ type: "text", text }];
if (images) {
    content.push(...images);
}
this.agent.followUp({ role: "user", content, timestamp: Date.now() });
```

Installed `pi-agent-core@1.1.0/dist/agent.js:60`
retains those original object references in `PendingMessageQueue`:

```javascript
// Installed pi-agent-core dist/agent.js
enqueue(message) {
    this.messages.push(message);
}
drain() {
    const drained = this.peek();
    this.messages = this.messages.slice(drained.length);
    return drained;
}
clear() {
    this.messages = [];
}
```

The queue therefore exists before any session-entry position.
Queue removal alone proves no policy revocation or historical human authorship.
The private source location uses delivery metadata without inventing an entry ordinal.

### Private correction and verification

The existing manager input owner captures only the actual constructor's original text.
Its frozen queued-input view contains source/publication references,
not copied messages.
The shared source caps,
native queue results,
and deferred capture-error boundary remain.
Source reads refresh changed nonempty views and recheck failures even when the view is unchanged.

`proc_43ef` and `proc_83d8` passed delivered and cleared steering/follow-up cases.
The first request for a pending follow-up omits its text while source accounting retains the original input.
Delivered objects and copied `entry.message` values retain the original input-only publication.
They never become payload capabilities.

`proc_14cd` rejected late arrivals during semantic assessment,
including streaming prompt and extension routes.
Oversized captured text preserved native enqueue completion,
then failed guarded source use.
Native reset before the new input merged raised `Root lease has been retired`;
no member executed.
This contradicts a reviewer hypothesis that this tested reset window silently continued.

`proc_3fab` showed why the first empty queue view must not trigger context retention.
The corrected owner passed `proc_a0bc`.
A new empty owner view also reuses already-published unchanged membership.
A changed owner cannot validate previously retained queued origins.

Run `mise --no-env --no-hooks run test:queued-input-source`,
`test:queued-input-late`,
`test:queued-input-lifetime`,
`test:queued-input-owner`,
and `test:selected-path-owner` separately in the private
`contract/integration/native-batch` directory.
The fixtures use authored local responses,
not provider requests or genuine approvals.

### Rejected readings and remaining scope

The private generator initially referenced `_sessionManager`,
which was not the actual private field.
`proc_1693` raised Node's
`TypeError: Cannot read properties of undefined (reading '#loadedInputs')`.
The generator now uses `#sessionManager`;
installed source is unchanged.

The ordinary-entry alias assertion in `proc_60ef` was also wrong:
the maintained copy observer follows `entry.message`,
not its wrapper.
No new wrapper alias was added to satisfy the fixture.

A source review suggested all queued images would poison capture.
The implementation captures the separate text argument,
not image data;
independent image origin remains unresolved.
Direct agent queue APIs,
custom queue producers,
and text removed by preceding input handlers are also outside this increment.
The governing-domain obligation remains open,
so these gaps do not produce automatic permission.

### Upstream filing decision

No upstream filing or upstream patch is proposed.
The missing custody edge belongs to the private adapter.
Native queue support does not promise this application's instruction authority or domain closure.
The production qualification and installed-cutover gates remain closed;
sensitivity and full-regression work remain local.
Installed/private API parity passed `proc_6162`.
Omissions `proc_d12b`,
`proc_2f0b`,
and `proc_88a1` failed at their intended source obligations;
all were restored before integrated `proc_2ae8` passed.
Complete native `proc_33e4`,
action `proc_593c`,
source-policy `proc_188a`,
and 14-document rendering `proc_b538` passed.

## Private SDK 1.1.0 source association stops at defensive copies

### Symptom and cause

The private integration's actual-serializer control reached canonical review but reported `unresolved`
where an original source association was expected.
This was a private adapter gap,
not an upstream SDK defect.
`proc_6ab2` first exposed the absent consumer;
`proc_66a8` and `proc_d722` then isolated lost sections-object identity.

The private `contract/integration/native-batch/root-source.mjs` already copies entries on append
and copies public projections to protect native state.
Those copies intentionally replace objects.
Installed `@earendil-works/pi-coding-agent@1.1.0/dist/core/extensions/runner.js:1006`
then makes another request-context copy:

```javascript
// Installed pi-coding-agent dist/core/extensions/runner.js
async emitContext(messages) {
    const ctx = this.createContext();
    let currentMessages = structuredClone(messages);
```

Comparing bytes after these operations would not authenticate an original source.
The private owner now republishes its existing capability at the actual copy sites,
before extension transforms.
No persisted-session provenance is inferred.

Installed `@earendil-works/pi-ai@1.1.0/dist/api/openai-completions.js:188`
awaits the payload hook before handing the selected object to the client:

```javascript
// Installed pi-ai dist/api/openai-completions.js
const nextParams = await options?.onPayload?.(params, model);
if (nextParams !== undefined) {
    params = nextParams;
}
```

Installed `openai@7.19.0/client.mjs:1156` also awaits authentication before body construction:

```javascript
// Installed openai client.mjs
const authenticationHeaders = this._provider || x509Authentication
    ? undefined
    : await this.authHeaders(inputOptions, inputOptions.__security ?? { bearerAuth: true });
const { bodyHeaders, body, isStreamingBody } = this.buildBody({ options });
```

A payload-hook check alone therefore leaves an alias-mutation interval.
The private adapter copies and freezes only newly owned payload containers for the associated branch.
It does not freeze callback-owned objects or require discarded aliases to remain current after serialization.

### Verification and supported workaround

From the private repository's `contract/integration/native-batch` directory,
run `mise --no-env --no-hooks run test:serializer-association`.
It regenerates the adapter from the current installed SDK and uses local SSE responses through the actual serializer.
`proc_6043` passed its 11-mode matrix on SDK 1.1.0.
Provider requests and native tool executions were zero.

Original publication,
top-level payload replacement retaining the original message,
and discarded-alias mutation after materialization remain associated.
Equal-byte message copies,
independent sections copies,
changed text,
accessors,
serialization hooks,
extra system content,
and opaque stream wrappers acquire no association.
An observed sections mutation permanently retires the original publication.
Retirement must not block native recording of the failure itself;
`proc_1b18` exposed that private cleanup error before `7890f0c` repaired it.

The workaround is partial source association,
not complete composite-origin accounting.
Request-domain and delegation obligations still withhold canonical execution.
It adds neither a JSON parser nor another request representation.

Expanded matrix `proc_2731` passed 15 modes.
`proc_0731` first reproduced a sections accessor swapping its result between authentication and serialization.
`proc_98b2` separately reproduced the copy observer invoking a nonenumerable getter that throws `false`.
Source-bearing envelopes now require ordinary data fields;
copy observation uses descriptors and does not invoke getters.
The enumerable-getter control counts only the native clone's invocations.

`proc_3ece` compared installed and private context processing in 10 SDK sessions:
unchanged conversation,
changed conversation,
system-message replacement,
in-place mutation,
and a handler throwing `false`.
The ordered handler observations and serializer inputs matched.

Equality omission `34a0355` failed `proc_92e5`.
Owned-payload omission `8bc2eb4` failed `proc_7738` during real client body construction,
before terminal fetch.
Both omissions are restored.
Full native `proc_c4d5` and action `proc_2fa4` passed.
These results remain scoped to the explicit fixed serializer and current-run publication,
not the default model-runtime wrapper or general history provenance.

### Retained forced and custom-section inputs

The private consumer separately lacked forced-prompt and custom-section links.
Actual-serializer checks `proc_84e0` and `proc_ff08` failed with `AssertionError [ERR_ASSERTION]`
at the required original-source association assertions.
The captured source text already existed;
adding another source inventory or JSON parser would not supply the missing publication lineage.

The existing run owner now records the actual forced-message projection.
Its source does not inherit a displaced host source's authority,
even when their text is equal.
Identity omission `c316f17` failed `proc_305b` by associating an independent equal-byte message copy.
After restoration,
full native `proc_38d6` and action `proc_3eaf` passed.

Installed Pi SDK 1.1.0 `dist/core/system-prompt.js:97` first assigns appended and project-context text.
Its custom-section override at `dist/core/system-prompt.js:111` then replaces only truthy values:

```javascript
// Installed pi-coding-agent dist/core/system-prompt.js:111
for (const [name, content] of Object.entries(customSections)) {
    if (content)
        promptSections[name] = content;
}
```

Consequently,
a retained base or run-replacement input does not contribute when a nonempty custom section displaces it.
An empty custom value does not remove the original section.
The private adapter consumes existing `mappedPromptInputs` only through their original run,
source membership,
and full sections publication.
`contract/collector/run-prompt-custody-v3/owner.mjs:211` binds the published options to the captured render:

```javascript
// Private contract/collector/run-prompt-custody-v3/owner.mjs:211
if(scope.rendered&&(scope.rendered.options!==(scope.delivery??scope.working)||!isDeepStrictEqual(scope.rendered.input,options)))throw new SourceCollectionError('Native run publication differs from its original prompt construction inputs');
```

From private `contract/integration/native-batch`,
`mise --no-env --no-hooks run test:section-source-association` passed `proc_49c5` with 18 modes.
These cover additions,
base and run-replacement shadowing,
empty overrides,
forced precedence,
independent copies,
malformed Unicode,
native-default rendering,
and invalid section names.
From private `contract/integration/action-policy`,
`mise --no-env --no-hooks run test:source-association` passed `proc_73fc` for original capabilities and map bounds.

Removing native shadowing failed `proc_30bf` by crediting displaced project-context text.
Removing source membership initially failed on an unrelated source shape in `proc_d146`;
that result did not demonstrate false admission.
Refined omission `fa5432b` failed `proc_cb23` with `nonmember`,
reporting `accounted` instead of `unresolved`.
Both guards are restored.
Full native `proc_744b`,
action `proc_09ee`,
and ten-document render `proc_fed3` passed.

This remains partial lineage at a containing serialized field,
not complete request-origin accounting or instruction authority.
Non-transmission does not revoke separately established governing instructions.
The private adapter still withholds execution on independent missing premises.
The existing filing disposition applies:
these are private-consumer gaps,
not defects in native copying or native override behavior.

### Native skill-guidance whitespace is transformed before publication

SDK 1.1.0 `dist/core/skills.js:283` constructs the guidance with leading blank lines:

```javascript
// Installed pi-coding-agent dist/core/skills.js:283
const lines = [
    "\n\nThe following skills provide specialized instructions for specific tasks.",
```

Its caller in `dist/core/system-prompt.js:106` trims the complete skills section:

```javascript
// Installed pi-coding-agent dist/core/system-prompt.js:106
const skillsPrompt = formatSkillsForPrompt(skills, skillFileReadTool).trim();
```

The retained guidance source therefore differs from the rendered prefix's leading whitespace.
The private association follows this known native transformation through the original full sections publication;
it does not rewrite the retained source or authenticate it by substring equality.
Test-only terminal parity checks the trimmed prefix separately from original-capability checks.

Private `mise --no-env --no-hooks run test:context-skill-source`
in `contract/integration/native-batch` failed `proc_0f4b` on the absent project-context introduction association,
then passed `proc_0500` after both introduction and skill-guidance links were added.
The expanded `test:section-source-association` matrix passed `proc_edd5` with 32 modes,
including read/bash guidance,
disabled skills,
no reader,
section overrides,
forced output,
and equal payload copies.
No provider request or native tool execution occurred.

This does not associate arbitrary skill-file contents or establish their authority.
Default-rule fragments use separate normalization/deduplication emission evidence,
not this whitespace-transformation profile.
Native trimming is expected behavior;
only private source consumption needed extension,
so there is no upstream defect or filing artifact.

### Default-rule construction can precede deduplication

SDK 1.1.0 `dist/core/system-prompt.js:34` normalizes and deduplicates each rule before appending it:

```javascript
// Installed pi-coding-agent dist/core/system-prompt.js:34
const addRule = (rule) => {
    const normalized = rule.trim();
    if (!normalized || seen.has(normalized))
        return;
    seen.add(normalized);
    rules.push(normalized);
};
```

The private recorder previously retained each constructed default literal before calling `addRule`.
That established construction,
not contribution to the serialized rules section.
An independent guideline could already have supplied identical text.

The fixed private recorder now observes the original local array length around the untouched native call:

```javascript
// Private builder-source.mjs: generated addBuiltInRule instrumentation.
const before = rules.length;
addRule(rule);
recordGenerated({
    slot: "default-rule",
    content: rule,
    nativeRuleInsertion: rules.length > before ? "inserted" : "not-inserted",
});
```

The existing render-input owner accepts this optional enum only as an own string data field on a default rule.
It freezes the fact on the existing original source.
No source is dropped from instruction assessment when its addition was suppressed.
Producer and consumer both require `inserted` for a contribution,
with the original run/render/source/publication checks and later override handling unchanged.

Private native task `test:native-rule-source` failed `proc_ed24` before the consumer existed,
then passed `proc_bfe3`.
The expanded rendering matrix passed `proc_a236` with 38 modes.
Its extra task names were forwarded as arguments rather than running builder/owner checks;
corrected separate invocations passed `proc_4b3c`.
See [mise task argument forwarding](mise-usage-args-inline-node.md#additional-task-names-can-be-forwarded-as-ordinary-arguments).

Producer omission `2885c7d` failed `proc_3a5c` with `1 !== 0`
for a fabricated suppressed-source contribution.
Consumer omission `39c6c44` failed `proc_1fd3` by reporting `accounted` instead of `unresolved`
for `rule-not-inserted`.
Both guards are restored.
Full native `proc_a618`,
action `proc_412e`,
and eleven-document render `proc_d1a5` passed.
The underlying native deduplication was correct;
this was a private source-consumption gap,
not an upstream defect or a reason to infer authority from equal bytes.

### Tool snippet rendering follows the actual prepared loadout

SDK 1.1.0 `dist/core/agent-session.js:1306` replaces the prompt's hidden-tool list during native preparation:

```javascript
// Installed pi-coding-agent dist/core/agent-session.js:1306
options.hiddenTools = [...this._hiddenDeclarations];
```

The list comes from actual `prepareLoadout` hook results collected at `dist/core/agent-session.js:1182`.
Editing only the earlier handler's `systemPromptOptions.hiddenTools` would not exercise that native outcome.
The private fixture uses the hook and checks the original final options,
without changing native loadout behavior.

In `dist/core/system-prompt.js:76`,
declared tools exclude hidden names.
Inside native-default rendering,
the tools section then uses only declarations with nonempty snippets:

```javascript
// Installed pi-coding-agent dist/core/system-prompt.js:84
const visibleTools = declaredTools.filter((name) => !!toolSnippets[name]);
```

The source consumer now accounts for the existing original mapped snippet occurrence only on that route.
Custom prompts,
inactive or hidden tools,
empty snippets,
a tools-section override,
or forced output do not receive that snippet association.
A hidden read tool can still produce indirect skill guidance;
that distinct native behavior is preserved rather than treating hidden declarations as unavailable callables.

Actual-serializer red `proc_9121` preceded green `proc_04cd`.
Expanded snippet/hidden-reader matrix `proc_4f4e` and consumer-owner controls `proc_43a1` passed.
The controls include independent equal active/inactive snippets,
whitespace,
empty overrides,
and payload copies.
Hidden-declaration omission `2fdd453` failed `proc_882a`;
selected-tool omission `5943e0f` failed `proc_d76d`.
Both incorrectly associated a non-rendered source and both are restored.
Full native `proc_18cf`,
action `proc_7c12`,
and eleven-document render `proc_3594` passed.

Tool names and native declaration placement still supply no source authority,
implementation-effect proof,
or governing-domain completeness.
The native loadout and rendering behavior was not defective;
only the private consumer lacked these source relationships.
There is no upstream filing artifact.

### Skill descriptions retain original sources through XML escaping

SDK 1.1.0 selects a native read/bash or indirect reader before rendering skills
in `dist/core/system-prompt.js:103`.
Its formatter excludes disabled entries at `dist/core/skills.js:279`:

```javascript
// Installed pi-coding-agent dist/core/skills.js:279
const visibleSkills = skills.filter((s) => !s.disableModelInvocation);
```

The emitted description uses the native XML encoder:

```javascript
// Installed pi-coding-agent dist/core/skills.js:297
lines.push(`    <description>${escapeXml(skill.description)}</description>`);
```

The private consumer now links existing original skill-description sources through that known rendering.
It does not parse metadata into a second inventory,
rewrite the retained description,
or infer the referenced file's identity or instruction authority.
An enabled empty description still creates native XML markup,
but no contribution from description text.
Equal enabled descriptions remain separate original occurrences.

Actual-serializer red `proc_f07f` preceded green `proc_cd76`.
Expanded matrix `proc_1cc5` passed 55 modes,
including native read/bash/indirect readers,
disabled and empty descriptions,
section and forced overrides,
independent copies,
and equal enabled descriptions.
The adversarial description contains closing-tag delimiters,
quotes,
an ampersand,
a backslash,
and newlines;
its expected escaped text is authored separately from the native formatter.
Original source/run/publication rejection controls passed `proc_848f`.

Construction and byte appearance alone are insufficient:
source membership and actual native visibility remain independent requirements.
The installed formatter required no changes;
this is private source accounting rather than an upstream defect or filing.

### Prompt and tool guideline insertion needs the original render window

SDK 1.1.0 `dist/core/system-prompt.js:57` visits tool guideline arrays before prompt guideline entries.
The same native `addRule` normalization and deduplication determines which original occurrence contributes.
An equal later entry must not borrow the first entry's insertion.

```javascript
// Installed pi-coding-agent dist/core/system-prompt.js:57
for (const name of selectedTools) {
    for (const rule of toolGuidelines[name] ?? [])
        addRule(rule);
}
for (const rule of promptGuidelines)
    addRule(rule);
```

The private adapter now constructs the existing guideline source records as private drafts
from the original frozen render input.
The fixed native loops report their field,
tool name where applicable,
loop-local ordinal,
raw value,
and observed local insertion result directly to that owner.
The owner seals and publishes the same records rather than creating a second inventory.
The recorder never supplies an instruction tier or a decision.

The existing run owner's render-finally path closes the observation window on success and failure.
Late or foreign-render calls reject before recording a new failure on an old run.
Failures during an accepted observation enter the original failure ledger,
so catching one in the builder cannot revive eligibility.
Legacy owner fixtures without native rendering retain their inputs without an insertion fact.

Required red `proc_99ae` preceded actual-serializer green `proc_f2dc`.
Original-record,
lifetime,
and unchanged-builder-output controls passed `proc_8829` and `proc_ab19`.
The latter includes a caught liveness failure that throws `false`.
Expanded visibility matrix `proc_b1b1` passed 69 modes;
consumer-owner controls passed `proc_c84f`.
The matrix includes duplicate entries within one array,
equal prompt/tool entries,
whitespace-equivalent entries,
hidden/inactive tool guidelines,
empty inputs,
custom prompts,
section overrides,
and forced output.

Failure-recording omission `77e34e4` failed `proc_acd7` with `Missing expected exception.`:
a builder caught the observation failure and incorrectly completed its render.
Render-window omission `a143431` failed `proc_3d7a` by recording
`Guideline insertion observation is already sealed` on the old scope after a late call.
Producer omission `6cfcc6c` failed `proc_95dc` with `1 !== 0` for a suppressed contribution.
Consumer omission `d3ab546` failed `proc_5308` with `accounted` instead of `unresolved`
for `Guideline not-inserted`.
Every guard is restored.
Full native `proc_f733`,
action `proc_8b6b`,
and eleven-document render `proc_1cf2` passed.
Native guideline handling itself was correct;
this remains a private source-consumption and ownership change,
not an upstream filing.

### SDK 1.1.0 handler messages need original construction and conversion links

The private required control failed `proc_51d1` with Node's `AssertionError [ERR_ASSERTION]`:
`The original retained handler message needs its own non-system field association`.
The native body contained the handler text;
the private source consumer had no lineage across the native object constructions.

SDK 1.1.0 `dist/core/agent-session.js:1615` creates a new native custom message
from each original emitter-result member:

```js
// Pi SDK 1.1.0, dist/core/agent-session.js:1615 to 1625
for (const msg of result.messages) {
    messages.push({
        role: "custom",
        customType: msg.customType,
        // Untyped extensions can pass null/missing content; normalize at ingestion.
        content: msg.content ?? [],
        display: msg.display,
        details: msg.details,
        timestamp: Date.now(),
    });
}
```

SDK `dist/core/messages.js:89` creates another object during LLM conversion:

```js
// Pi SDK 1.1.0, dist/core/messages.js:89 to 97
case "custom": {
    const content = typeof m.content === "string" ? [{ type: "text", text: m.content }] : m.content;
    return {
        role: "user",
        content,
        timestamp: m.timestamp,
    };
}
```

Pi AI 1.1.0 `dist/api/openai-completions.js:933` filters empty text
and constructs fresh provider text blocks.
Its text branch is:

```js
// Pi AI 1.1.0, dist/api/openai-completions.js:936 to 941
if (item.type === "text") {
    return {
        type: "text",
        text: sanitizeSurrogates(item.text),
    };
}
```

The private patch observes those actual constructions,
the existing native manager copy sites,
and the existing projection's `sourceEntry`/message relationships.
It does not match origins by text,
entry ID,
message ordinal,
or `customType`.
Context-edit targets do not republish the displaced source,
including when replacement text is equal.
Other custom-message append sites serve boundary drafts and `sendCustomMessage` deliveries;
they have no retained original handler-input publication merely because their text matches.

The verified consumer preserves original string or JSON source text,
owns fresh nested payload containers,
and seals through the existing terminal JSON validation.
There is no extra request parse or request re-encoding.
The installed/private conversion check passed `proc_1bd1`:
15 SDK cases and 60 provider cases,
including image downgrading and synthetic transcript messages.
Canonical handler accounting passed `proc_e424`;
original-source/run consumer controls passed `proc_9621`;
the restored 24-mode native matrix passed `proc_7137`.
Run `mise --no-env --no-hooks run test:handler-source-controls`
inside private `contract/integration/native-batch`.

The passing catalog includes original structured text,
empty/nullish inputs without invented transmission,
equal independent inputs,
blocked-image settings with text,
native context edits,
and mutation of discarded payload aliases before and after materialization.
Unsupported image input still rejects at the incumbent text-only snapshot boundary.
Malformed Unicode remains retained but unassociated under this original-text profile.

Two private guard defects were reproduced and fixed.
`proc_2adc` reported `Equal independent nested replacement must retire content-array`:
equal text had concealed replacement of an original nested object.
Alias signatures now compare the original content and part references,
not only values and descriptor flags.
`proc_9061` reported
`Observed accessor substitution cannot revive after descriptor restoration`
with `true !== false`.
Known handler currentness now runs before structural rejection can skip retirement.

The first matrix attempt `proc_2a7c` also caught a fixture error:
its callback changed both requests while its getter assertion expected only the first.
The fixture now explicitly targets the first request.
The first restoration attempt `proc_71c1` threw its assertion inside the native stream catch;
the explicit `proc_9061` control records the observation and asserts outside that catch.
These failures are not upstream SDK defects,
and no installed source was changed.

The expanded 26-mode matrix passed `proc_2471`,
including real later-review ancestry and new-prompt isolation from persisted equal text.
Deliberate omissions failed for nested part identity (`proc_cecc`),
context-edit exclusion (`proc_dd37`),
nested payload custody (`proc_6256`),
and original emitting-run membership (`proc_6486`).
Every guard is restored.
Full native `proc_41bf` and action `proc_de99` passed;
eleven-document rendering passed `proc_c7e4` before this evidence update.

### Original root scheduling must be captured at the deciding dispatcher

The private scheduling control `proc_c205` reached canonical review but found
`groupExecutionMode` absent from original prepared members.
Node reported `AssertionError [ERR_ASSERTION]`:
`Original prepared members must retain the deciding dispatcher group mode, not later configuration`.

The incumbent private dispatcher in
`contract/diagnostic/parallel-tool-batch/prepared-dispatch.mjs:40`
computes the effective group mode once:

```js
// Private contract/diagnostic/parallel-tool-batch/prepared-dispatch.mjs:38 to 40
const scheduled=Object.freeze(calls.map(call=>Object.freeze(structuredClone(call))));
if(executionMode!=='parallel' && executionMode!=='sequential')throw new PreparedBatchContractError('Unknown batch execution mode');
const serial=executionMode==='sequential' || scheduled.some(call=>tools.find(tool=>tool.name===call.name)?.executionMode==='sequential');
```

The same `serial` value controls entry-turn waiting.
The maintained private `contract/integration/native-batch/program-source.mjs` now extends
the existing dispatcher copy's member-publication expression with that value:

```js
// Private native dispatcher publication generated by program-source.mjs
groupExecutionMode: serial ? "sequential" : "parallel"
```

It creates a new frozen member shape at the existing publication,
rather than mutating a frozen record or recomputing from later configuration.
The original judgment already serializes the prepared manifest;
the semantic wire forwards that serialization unchanged.
The generator reuses the original dispatcher error classes.
Installed files and historical generated dispatcher artifacts are untouched.

The first latching fixture mistakenly changed authenticated live `agent.toolExecution`
and expected policy review to continue.
`proc_1a87` exposed the existing `ExecutionContextOwnershipError` from
private `contract/integration/native-batch/tools.mjs:69`:
`Native tool "codemode" or its context owner changed after request preparation`.
That guard is correct and remains unchanged.
The corrected controls distinguish a discarded batch-argument alias from the live configuration:
the alias cannot rewrite the already captured mode,
while a live change retires the original context before semantic assessment.

The scheduling/configuration controls passed `proc_5337`;
actual current-dispatcher execution,
queue,
abort,
and disposal passed `proc_c317`.
The complete-group consumer control then failed `proc_e3d5`
because admitted whole-operation estimates were still restricted to one original member.
Parallel and sequential group consumption passed `proc_5a03`,
and the 12-mode native matrix passed `proc_bce2`.
Unknown,
unsupported,
abstained,
and independently fact-owned operation uses retain their unresolved obligations.
Run `mise --no-env --no-hooks run test:group-scheduling`
inside private `contract/integration/native-batch`.

These are private consumer and fixture changes,
not an upstream defect or a reason to relax configuration freshness.
No model was asked to decide scheduling or the final action outcome.

### SDK 1.1.0 expanded prompt identity differs from transport user role

The original body already retained prompt bytes,
but the private source set had no original expanded-prompt occurrence.
Required control `proc_2f9f` failed with Node's `AssertionError [ERR_ASSERTION]`:
`The native expanded prompt needs its original run-bound source capability, not attribution from serialized user role`.
After adding capture to the existing run owner,
`proc_6dd4` reached the missing serialized-contribution assertion.
This separates absent input ownership from absent publication lineage.

SDK `dist/core/agent-session.js:1591` passes the post-input/skill/template text
into its actual before-agent-start emitter:

```js
// Pi SDK 1.1.0, dist/core/agent-session.js:1591
const result = await this._extensionRunner.emitBeforeAgentStart(expandedText, currentImages, this._baseSystemPromptOptions);
```

The native image normalizer can then change the text
at `dist/core/agent-session.js:1600`:

```js
// Pi SDK 1.1.0, dist/core/agent-session.js:1600
const userText = normalized.hints.length > 0 ? `${expandedText}\n\n${normalized.hints.join("\n")}` : expandedText;
```

The private capture identifies `expandedText`,
not an unmodified human utterance or every upstream transformation.
Its text-only publication observes the actual native user-message construction.
Image-containing and hint-modified constructions keep their original input source
without receiving an unsupported contribution association.
Native output and errors remain native.

The original copy/publication index is reused for ordinary user identity pass-through,
manager/context copies,
and the fixed native serializer.
The collector's combined field is `runMessageInputs`
(renamed from snapshot `handlerMessageInputs` on 2026-10-09);
per-run `handlerMessageInputs` remains emitter-only.
The original expanded input is unregistered.
No second JSON parse,
request re-encoding,
parallel message inventory,
or role-derived authority was introduced.

Initial full consumer `proc_3e36` passed.
Owner regressions passed `proc_1190`,
conversion parity `proc_f69b`,
source-policy `proc_898f`,
and action `proc_dd50`.
The 23-mode native matrix passed `proc_7109`,
including actual native image and normalization-hint routes.
The passing catalog distinguishes independent deep material copies from wrappers
that retain original part capabilities,
and covers context edits,
input transformation,
empty/whitespace/malformed text,
payload races,
active ancestry,
and separate new prompts.

The additional retirement control passed `proc_eebb`.
Removing pass-through currentness observation failed `proc_334e`:
an equal replaced part regained an `accounted` contribution on a later request after restoration.
Run `mise --no-env --no-hooks run test:expanded-prompt-controls`
inside private `contract/integration/native-batch`.
Scalar-publication omission `proc_b17c` also admitted a copied publication;
hint-profile omission `proc_7ea3` changed unsupported hint handling into an exception.
All guards are restored.
Full native `proc_5d3b`,
full action `proc_15a2`,
and 12-document rendering `proc_2db5` passed.
This proves a bounded origin edge,
not complete request-domain coverage,
source authority,
or permission.

### SDK 1.1.0 failed migration writes leave a partial in-place load

A disposable read-only version-2 session caused native `setSessionFile`
to throw filesystem `EACCES` from `open` during migration rewriting.
The installed and private managers returned equal surviving projections,
but those projections still selected the previous indexed entries.
Their entry arrays already contained the attempted new load.

The deciding order is in SDK `dist/core/session-manager.js:717`:

```js
// Pi SDK 1.1.0, dist/core/session-manager.js:717
_loadEntries(entries, options) {
    const header = entries.find((e) => e.type === "session");
    if (header) {
        this.fileEntries = entries;
        this.sessionId = header.id;
        if (migrateToCurrentVersion(this.fileEntries)) {
            this._rewriteFile();
        }
    }
    else {
        this.newSession(options);
        this.fileEntries = this.fileEntries.concat(entries);
    }
    this._buildIndex();
}
```

The rewrite can fail after replacing `fileEntries` but before rebuilding `byId`.
This does not establish a rollback guarantee.
The original fixture `proc_12a4` wrongly expected the attempted new value
to be the value returned by the surviving projection.
Corrected `proc_84c8` checks both the new entry-array value and the old projected value,
native output/error parity,
and unchanged read-only file bytes.

The private origin reader withholds the prior indexed value's origin
rather than relabeling it as the attempted new load.
This remains an unresolved provenance case,
not a qualified session continuation.
A separate invalid-file switch that fails before entry replacement preserves the original loaded values
and their existing original load occurrence.
Native root retirement remains independent.

The checks use only disposable files and restore their writable modes afterward.
No native rollback,
file repair,
or upstream SDK modification was implemented.
The successful loading and migration controls are distinct:
`proc_3dd2` covers original manager identity and value output;
`proc_68f4` covers version-1/version-2 migration and invalid-file opening;
`proc_84c8` covers failed in-place loading.

Provenance capture failures also have a separate boundary.
Unsupported loaded content or an exceeded source bound leaves native file loading intact,
but stops the private source-use path before transport.
That is intentional fail-closed behavior,
not transparent compatibility with every unguarded prompt.

### SDK 1.1.0 compaction projection has distinct system and summary outputs

A private summary observer must not assign one summary input's identity to every message from its entry.
Native `appendCompaction` stores the current system message inside the new compaction entry.
SDK `dist/core/session-manager.js:880` contains:

```js
// Pi SDK 1.1.0, dist/core/session-manager.js:880
const systemMessage = getCurrentSystemMessage(this.buildSessionProjection().messages);
// Entry construction includes this separate companion:
...(systemMessage ? { systemMessage: { ...systemMessage, timestamp: new Date(timestamp).getTime() } } : {}),
```

The same file's `sessionEntryToContextMessages`,
at line 188,
constructs the summary separately:

```js
// Pi SDK 1.1.0, dist/core/session-manager.js:188
const summary = createCompactionSummaryMessage(entry.summary, entry.tokensBefore, entry.timestamp);
return entry.systemMessage ? [entry.systemMessage, summary] : [summary];
```

The original system entry is not duplicated in the retained range:
`buildContextEntries`,
at line 223,
excludes system-message entries there:

```js
// Pi SDK 1.1.0, dist/core/session-manager.js:223
if (foundFirstKept && !(entry.type === "message" && entry.message.role === "system")) {
    contextEntries.push(entry);
}
```

A reviewer hypothesis that one projected system message disproved a same-entry companion was incorrect.
Native row-cardinality assertions in `proc_047a` verify the actual two-output compaction row,
including its stored `systemMessage`.

The private `contract/integration/native-batch/root-source.mjs` generator observes the actual summary constructor.
The shared publication owner records only that summary output's input-only identity.
Its generic entry projection still requires one output,
so it cannot assign summary provenance to the system companion.
The summary-to-user conversion observer preserves historical input identity without payload eligibility.
No additional file parser,
request parser,
or transcript inventory was added.

Required missing-source control `proc_640e` failed;
connected compaction and branch-summary consumers passed `proc_1efc`.
Empty-summary and other loaded-input modes passed `proc_503b`.
Native output parity,
same-entry companion isolation,
version-1/version-2 migration,
failed migration-rewrite withholding,
shared limits,
and distinct equal-value occurrences passed `proc_047a`.
Compaction-constructor omission `proc_de3c` lost the summary alias;
the constructor observation is restored.
Companion-attribution omission `proc_9769` failed because the stored system message acquired summary provenance.
The single-output guard is restored.
Complete native `proc_18d1`,
action `proc_06a7`,
source-policy `proc_62b3`,
and 14-document rendering `proc_7bd6` passed.

Run the private controls from `contract/integration/native-batch`:

```sh
# Private consumer-contract repository, contract/integration/native-batch
mise --no-env --no-hooks run test:loaded-summary-manager
mise --no-env --no-hooks run test:loaded-input-controls
```

The boundary remains partial:
stored system companions and current-run generated summaries do not gain an original instruction-source capability
from capturing a loaded summary.
Saved summaries establish neither historical human intent nor governing authority.
This is private adapter propagation,
not an upstream defect;
the filing disposition in this section remains unchanged.

### SDK 1.1.0 stored system deltas differ from compaction checkpoints

A system source fixture incorrectly expected a checkpoint to preserve a null section deletion verbatim.
`proc_3666` instead reported a source containing only the surviving text section.
The original loaded delta still contained the null value.

Native `appendCompaction` stores the result of system-message replay,
not a byte copy of every prior message.
Pi AI `dist/utils/transcript.js:69` applies section updates:

```js
// Pi AI 1.1.0, dist/utils/transcript.js:69
for (const [name, value] of Object.entries(message.sections ?? {})) {
    if (value === null)
        sections.delete(name);
    else
        sections.set(name, value);
}
```

The private fixture now checks original system entries and stored checkpoint companions independently.
The source observer retains their original values rather than rewriting either to match the other.
It captures content and sections only,
without extracting tool definitions or rebuilding the request body.
Native section deletion does not establish revocation of a governing instruction.

Private owner and native companion checks passed `proc_950c`.
The corrected source matrix and complete regression remain tracked in the private progress record.
This is a fixture correction and private provenance extension,
not an SDK defect or an upstream filing.

### SDK 1.1.0 bash record formatting creates a new user message

A loaded `bashExecution` record is data about a command,
not authenticated proof that the command ran.
Its native formatting also creates a different object.
In `dist/core/messages.js:79`:

```js
// Pi SDK 1.1.0, dist/core/messages.js:79
case "bashExecution":
    if (m.excludeFromContext) {
        return undefined;
    }
    return {
        role: "user",
        content: [{ type: "text", text: bashExecutionToText(m) }],
        timestamp: m.timestamp,
    };
```

The private observer follows that actual construction edge.
It preserves input-only identity without authenticating payload text or historical execution facts.
Excluded and compacted inputs remain separately retained through original source membership.

Required missing-input red `proc_f853` became native consumer green `proc_5aac`.
Native formatting and exclusion variants,
owner getter/profile controls,
and the 31-mode loaded-input matrix passed `proc_589c`.
Conversion omission `proc_30e1` lost the resulting user message's original publication.
The conversion observation is restored.
Complete native `proc_aa93`,
action `proc_25cc`,
source-policy `proc_7e06`,
and 14-document rendering `proc_e36e` / `proc_0f9f` passed.

Run `mise --no-env --no-hooks run test:loaded-bash-manager`
inside private `contract/integration/native-batch`.
The fixtures write authored session records,
but do not execute their recorded commands.
This is a private propagation repair,
not an upstream defect or a filing proposal.

### SDK 1.1.0 context edits create replacement messages without transferring authority

Native context edits retain message metadata while replacing content.
In `dist/core/session-manager.js:235`,
`projectContextEntry` returns no messages for a null replacement.
For supported editable roles,
line 252 constructs a fresh output:

```js
// Pi SDK 1.1.0, dist/core/session-manager.js:249
const content = (message.role === "assistant" || message.role === "toolResult") && typeof replacement.content === "string"
    ? [{ type: "text", text: replacement.content }]
    : replacement.content;
return { ...message, content };
```

The private observer attaches the original edit input only at that construction.
It does not reuse the displaced target's publication.
System messages and summaries that the native non-null edit leaves unchanged do not acquire edit provenance.
Missing targets likewise produce no replacement output.

The canonical equal-edit fixture initially searched only array-shaped user content.
Pi AI `dist/api/openai-completions.js:926` preserves scalar user text:

```js
// Pi AI 1.1.0, dist/api/openai-completions.js:926
if (typeof msg.content === "string") {
    params.push({
        role: "user",
        content: sanitizeSurrogates(msg.content),
    });
}
```

`proc_5125` and the matrix phase of `proc_2061` exposed that fixture omission.
The fixture now checks both native representations.
A separate fixture error,
`proc_871e`,
used a family-unspecific hidden-mode match and unintentionally created a compaction for `edit-hidden`.
That flag now requires the compaction-system family.
Neither correction changes runtime provenance or native output.

Corrected `proc_da68` passed native editable-role variants,
unapplied system/missing/compaction targets,
owner shape/getter/shared-cap checks,
and the 36-mode loaded-input matrix.
Construction omission `ddc6f13` / `proc_e0e8` lost the edited message's original publication.
The observer is restored.
Complete native `proc_36c9`,
action `proc_fafc`,
source-policy `proc_80b6`,
and 14-document rendering `proc_18b1` passed.

Run `mise --no-env --no-hooks run test:loaded-edit-manager`
inside private `contract/integration/native-batch`.
Omissions and shadowed edits remain original inputs,
not proof that an earlier governing instruction was revoked.
The source profile remains input-only and unregistered.
This is private consumer propagation and fixture repair;
no upstream defect or filing is proposed.

### SDK 1.1.0 context handlers can create inputs after initial capture

The native context emitter first clones messages,
then awaits extension handlers,
then returns the final message list.
Installed `dist/core/extensions/runner.js:1006-1065` was read in full:
there is one final return after both handler loops.
The relevant operations include:

```js
// Pi SDK 1.1.0, dist/core/extensions/runner.js:1006-1065, excerpts
let currentMessages = structuredClone(messages);
const handlerResult = (await handler(event, ctx));
currentMessages = handlerResult?.messages ?? currentMessages;
return currentMessages;
```

Capturing only at entry misses a context edit appended during those awaited handlers.
Required red `proc_cc5e` demonstrated that missing same-request input.
The private emitter now returns the original run handle from entry observation
and revalidates it at final observation.
The same manager input owner captures each native edit append with a fresh occurrence,
not a fabricated file-load or earlier-run identity.

Final observation rechecks the capture-failure latch before considering view reuse.
It preserves previous original inputs,
validates their current manager membership,
and adds newly observed inputs.
No-op reuse requires an already-published view;
the first empty observation must still publish.
The initial optimization broke that distinction in `proc_5206`.

The native image-failure fixture first asserted the nested capture-error message,
then tried reading a failed run after prompt cleanup (`proc_ef57` / `proc_6aad`).
The actual native surface was `RunPromptCustodyError`.
The corrected fixture intercepts and rethrows the original context error before cleanup,
then checks the exact nested `SessionInputCaptureError`.
`proc_b11b` passed that control,
the 41-mode native matrix,
and original-owner controls.
A superseded completion was independently rejected after required red `proc_db41`.

Append omission `proc_7ed6`,
final-observation omission `proc_0903`,
and prior-input-union omission `proc_a4c1` each failed their control.
All omitted code is restored.
Complete native `proc_8bde`,
action `proc_9601`,
source-policy `proc_f730`,
and 14-document rendering `proc_a31c` / `proc_c9e9` passed.

Run `mise --no-env --no-hooks run test:created-edit-context`
and `mise --no-env --no-hooks run test:selected-path-owner`
inside private `contract/integration/native-batch`.
These are private observation and fixture corrections,
not SDK defects.
They establish neither full governing-domain coverage nor authority from native role labels.

### SDK 1.1.0 publishes appended entries in memory before persistence

Installed `dist/core/session-manager.js:815` orders append operations as follows:

```js
// Pi SDK 1.1.0, dist/core/session-manager.js:815
_appendEntry(entry) {
    this.fileEntries.push(entry);
    this.byId.set(entry.id, entry);
    this.leafId = entry.id;
    this._persist(entry);
}
```

A context handler can append an instruction input without rebuilding or returning the native projection.
That input can therefore be absent from the transmitted request and the most recent observed path prefix.
Required red `proc_6f5a` exposed this missing original input.

The private observer now records known original input membership after the native leaf assignment
and before persistence.
It extends the incumbent source-reference set without another path walk.
Same-root overflow is deferred to source use and latched;
a later shorter observation cannot clear it.
Root replacement starts a separate observation scope,
not revival of the old runtime.

Current summary arguments receive original append occurrences,
but their derived system checkpoints are not minted as new independent instruction roots.
Loading a saved checkpoint is a separate original load observation.
Neither route proves complete composite origin or governing authority.

`proc_7838` passed untransmitted edit/summary consumers.
`proc_eecb` passed current-summary/derived-checkpoint and expanded native controls.
`proc_1bea` passed root,
duplicate,
foreign,
and deferred-bound controls.
`proc_f207` passed native write-error parity after this hook,
including simultaneous capture overflow and filesystem `EACCES`.
The native error surfaced immediately,
while the capture failure remained available at source use.
Original in-memory membership is not proof of successful persistence or qualified continuation.

Leaf-observation omission `proc_19e4`
and same-root latch omission `proc_40b7` failed their required controls.
Both guards are restored.
Complete native `proc_7a67`,
action `proc_a32c`,
source-policy `proc_42e6`,
and 14-document rendering `proc_62a0` / `proc_28b9` passed.

Run `mise --no-env --no-hooks run test:created-edit-owner`
and `mise --no-env --no-hooks run test:selected-path-owner`
inside private `contract/integration/native-batch`.
This is private consumer observation,
not an upstream append defect;
no upstream filing is proposed.

### Rejected approaches and filing disposition

Byte equality and transport role cannot replace source custody.
A check of mutable payload aliases after serialization is also the wrong lifetime:
the immutable historical request has already been produced.
Neither approach is used as source authentication.

No upstream filing is warranted:
the lost association arose from private adapter copies and missing private propagation.
Upstream capability propagation has not been established as a supported API requirement.
Fixability does not imply an upstream defect,
contribution welcome and maintainer intent were not evaluated,
and no upstream patch is proposed.
The verified patch belongs to the private consumer.
There is no upstream filing artifact to add.

## Private SDK 1.0.4 adapter misses runner and schema mutation

### Symptoms

These failures belonged to the private integration,
not the installed SDK.
Paths in this section are relative to the private consumer-contract repository,
unless an installed package is named.

`proc_305f` and `proc_6072` reproduced public runner shadowing and prototype-method replacement.
The inert program returned a guarded root error instead of completing.
The test emitted `AssertionError [ERR_ASSERTION]: true !== false`.
After private runner storage alone passed,
prototype replacement still redirected the method lookup.

`proc_c467` reproduced a different failure.
Changing an input schema in place after fixture assessment still permitted its three inert child executions.
The test emitted `AssertionError [ERR_ASSERTION]: false !== true`.
This was independently authored fixture execution,
not a demonstrated production policy release.

### Cause and repair

The private adapter had protected object references without fixing every consumed method or nested data dependency.
Current `contract/integration/native-batch/program-source.mjs:10,17`
captures the original method before later prototype mutation
and invokes it through the private session runner:

```javascript
// contract/integration/native-batch/program-source.mjs, generated source excerpts
const originalCreateDeclaredProgram=NestedToolCallRunner.prototype.createDeclaredProgram;
Reflect.apply(originalCreateDeclaredProgram,session.#getOwnedProgramRunner(),[input])
```

Schema reference identity was insufficient.
Installed `@earendil-works/pi-ai@1.0.4/dist/utils/validation.js:280`
uses the schema during argument preparation:

```javascript
// Installed pi-ai dist/utils/validation.js
normalizeOptionalNulls(args, tool.parameters);
Value.Convert(tool.parameters, args);
const validator = getValidator(tool.parameters);
```

The existing execution-tool owner now retains data descriptors
and validates both schema graphs at its currentness boundary.
Relevant code is `contract/integration/native-batch/tools.mjs:29,45,61,62,68`:

```javascript
// contract/integration/native-batch/tools.mjs, currentness excerpt
assertParameters();assertOutputSchema();
```

The comparisons include symbol keys,
descriptor attributes,
prototypes,
and original values.
They do not freeze borrowed objects or extract and encode a second request representation.
Callable values remain identity dependencies,
not proof of implementation semantics.
Ordinary schema records and arrays are supported;
accessors and nonordinary object prototypes are rejected.
The request-wide bound is 10,000 property and object entries.
This boundary is not a guarantee about arbitrary host callbacks or complete operation effects.

### Verification and tradeoffs

Private fix `87b8e7e` passed public-shadow control `proc_5adc`.
Fix `3100219` passed prototype and input/output schema controls in `proc_9d00`.
Expanded native matrix `proc_5caf` passed 18 cases,
including original failure-cause and terminal-state assertions.
Unit control `proc_a52b` passed nested,
array,
symbol,
hidden-property,
accessor,
prototype,
descriptor,
callable-reference,
bound,
and permanent-retirement cases.
These unit controls use execution-position facades;
they are not native authentication or policy-admission evidence.
Full native suite `proc_c1b1`,
action compatibility `proc_edaa`,
and source compatibility `proc_bdbb` passed.

From `contract/integration/native-batch`,
the runnable controls are:

```sh
# Private consumer-contract repository, contract/integration/native-batch
mise --no-env --no-hooks run test:program
mise --no-env --no-hooks run test:tools
```

The earlier `proc_3948` failure was a test mistake:
inspection includes declared `unreached` groups.
Counting only reached states repaired that assertion without changing an execution guard.
A successful inert fixture does not qualify semantic estimates,
authority interpretation,
or production permission.

### Constructor-description capture follows native loadout metadata

This is the active private increment;
the completed manager checkpoint predates this runtime change.
Native `prepareLoadout` can replace declared descriptions,
and hidden tools need not appear in a model request.
The original definition text remains a separate potentially governing input.

Declared-description control `7414bed9` / `proc_2d95` passed.
Hidden `proc_fc39` and replaced `proc_acbd` completed native request and review,
then failed the original-description retention assertion.
The first setup attempt `proc_8d89` instead expected an unaugmented declaration;
the native Codemode call/output hint is preserved,
not removed.

Private `67a2a405` observes the actual Session constructor frame after its original manager assignment,
before reading `customTools` or running loadout callbacks.
It selects ordinary `name` and `description` data fields under one aggregate copier budget.
The existing manager source owner retains an input-only publication on that frame,
not on the borrowed config,
list,
or definition objects.
The distinct `native-tool-description-input` kind and `customTools` location add no message role,
ordinal,
authority,
or payload capability.
Absent input and an explicit empty list remain different.
The original occurrence,
content,
and serialized-record bounds are unchanged.

Owner controls and declared/hidden/replaced native consumers passed
`d1e67271` / `proc_1456`.
They require the constructor-specific view,
original source/publication references,
unchanged request bytes,
and indexed assessment questions.
Schema,
executable,
and other opaque fields are not inspected.
Native constructor parity passed `93c75a07` / `proc_9cc4`.
The getter counters matched the baseline,
including zero construction-time reads of `execute`,
and separate positive reads verified the counters.
Lifecycle and declared/hidden/replaced/raw-mutated description consumers passed `proc_a37a`.
Action/source parents passed `proc_50d1`;
all 154 integrated input cases passed `proc_a226`;
original-judgment variants passed `proc_d5fb`;
the runtime suffix passed 39 complete outputs in `proc_f646`.
Complete native parent `89a504de` / `proc_7042` passed in 860 seconds:
82 complete outputs,
157 integrated input cases,
and 69 section-source cases.
This is the restored implementation with the accepted processing-receipt controls.
Required pre-implementation description-loss reds are retained;
no duplicate omission campaign is opened.
The first parity baseline `proc_0081` stopped at an unresolved relative module import before execution;
its fresh copy now keeps the original staged-module targets.

This covers selected constructor descriptions,
not earlier SDK option mutations,
extension registrations,
schemas and other metadata,
deep mutation,
or applicability across manager-root resets.
The manager-only post-emission growth predicate is unchanged.
Constructor-source additions after emission cannot silently gain that handoff.
No installed activation,
semantic qualification,
or governing-domain closure follows.


### Native loadout description preservation

A later control,
`proc_3145`,
found that wrapping the registered definition discarded the native projected description.
It specifically lost
`Codemode: tools.fixture_value(args) resolves to a string.`
(the native text places the call expression in backticks).
The fixture rejected the group with
`Preserve native loadout descriptions for fixture_value`.

Installed `@earendil-works/pi-coding-agent@1.0.4/dist/core/agent-session.js:1190`
creates that description projection without replacing the registered executable definition:

```javascript
// Installed pi-coding-agent dist/core/agent-session.js
declared = tools.map((tool) => {
    const description = descriptions.get(tool.name);
    return description === undefined ? tool : { ...tool, description };
});
```

Current `contract/integration/native-batch/tools.mjs:84`
preserves the projected description with the existing owned callbacks:

```javascript
// contract/integration/native-batch/tools.mjs
const wrapped=wrapTool(tool.name),projected=Object.freeze({...wrapped,description:tool.description});
```

`proc_8ebd` passed the real native description and callable-snapshot controls.
This repairs a consumer-side loss of native metadata;
it does not make descriptions permission or qualified effect evidence.

### Rejected approaches and upstream filing decision

Private storage alone did not protect prototype method lookup.
Schema-reference comparison alone did not detect in-place mutation.
Adding another JSON tools source would not solve either consumed-runtime dependency.

- Upstream fault:
   no.
  The reproduced omissions were in our private adapter.
- Upstream fixability:
   not needed for these fixes.
  They are implemented at the existing consumer boundary.
- Supported upstream use case:
   the private ownership contract is not asserted to be an SDK guarantee.
- Contribution policy:
   not investigated because no upstream defect or filing is proposed.
- Maintainer intent:
   not investigated for the same reason.
- Prototype:
   private fixes and controls exist;
  no upstream patch is claimed.

There is nothing to file upstream from this evidence.
No SDK issue,
provider request,
or installed-source modification occurred.

## Typed-relation verification: Git inspection timeout and overlapping index access

### Observations

On 2026-10-08,
the Bash tool timed out a private-repository `git status --short` and `git log` inspection after 20 seconds.
It reported the existing logger diagnostic:

```text
# Reported during the owned Git inspection; emitting process was not captured.
logger internal error: sink verification failed for entry 3: Timed out after 5000ms: sink 3 verify
```

This was not a policy test or a state-changing Git invocation.
It does not establish the cause described in another logger incident.
The existing [logger investigation](cli-git-logger-sink-verify-timeout.md) retains the unresolved cause limits.

The recovery inspection `proc_0b64` completed.
The assistant incorrectly overlapped it with sensitivity commit `proc_e866` in the same private repository.
That commit command failed at `git add` with `index.lock: File exists`;
its test did not run.
The actual lock holder was not captured,
so overlapping dispatch is an observed scheduling error,
not proof of a particular Git implementation path.

### Recovery and verification

After both processes exited,
`ls --full-time .git/index.lock` reported no lock,
and the scoped process search found no matching Git process.
No lock was removed.
The inspection showed the intended source modification still uncommitted and no new omission commit.
Serialized `proc_a4ae` then committed omission `1c41a06` and reached the expected immutability assertion failure.
Restoration `88f3635` passed separately as `proc_c971`.
All Git calls for this repository are now serialized,
including inspection calls.
A successful later invocation is not evidence that the logger delay was fixed.

### Rejected actions and upstream scope

Do not classify a rejected Git command as a failed test,
blindly replay an ambiguous commit,
or remove an unassigned index lock.
No upstream SDK or Git defect was established,
and no issue was filed or reopened.

## SDK 1.0.4 default prompt with an independent policy copy exceeds the private source cap

### Symptom and cause

The expanded native control `proc_83e5` retained this synthetic terminal diagnostic:

```text
# contract/collector/collector.mjs
Current rendered prompt exceeds source byte cap
```

The emitting `SourceCollectionError` belongs to our private collector,
not the SDK or provider.
Paths in this section are relative to the private consumer-contract repository.
`contract/collector/collector.mjs:10,16,73` checks the complete rendered string:

```javascript
// contract/collector/collector.mjs
const limits = Object.freeze({ sourceBytes: 65_536, totalBytes: 524_288, files: 64, appendTexts: 32 });
if (Buffer.byteLength(value) > limits.sourceBytes)
  throw new SourceCollectionError(`${label} exceeds source byte cap`);
const rendered = text(adapter.render(), 'Current rendered prompt');
```

This cap is distinct from the one-MiB base-linked snapshot bound described in
`Owned SDK 1.0.4 hook masked terminal errors`.
It also differs from the generated-fragment text cap.
The measured default preamble was 169 bytes;
documentation guidance was 1517 bytes in the empty-system fixture.
Those fragments did not individually exceed their source bound.

### Verification and supported boundary

`proc_a631` ran `mise --no-env --no-hooks run test:full-policy:default`
in `contract/integration/native-batch/`.
The native terminal listener measured the actual current rendered prompt before run settlement.

- Linked current policy:
   34,332 rendered bytes,
  207 canonical clauses,
  414 questions,
  one local relation response.
- Additional independent equal-text policy copy:
   66,405 rendered bytes,
  source-cap rejection before judgment creation,
  zero relation requests.

Both cases used the unmodified current policy as disposable fixture text.
No provider request occurred.
The default prompt with the linked policy is supported by this control;
the failure must not be generalized to that case.

### Limits and rejected remedies

No bound was raised and no policy text was trimmed.
An independent copy cannot be merged merely because its bytes match.
The negative control now asserts the actual earlier source-cap boundary,
rather than demanding the later clause-batch rejection.
The accepted original-source carrier remains the supported representation for genuine linked copies,
but it is not a workaround that authenticates an independent source.
The oversized independent default case remains unsupported.

### Upstream filing decision

No upstream defect or filing artifact is established:

- Fault:
   the bound belongs to our private collector;
  the test expected to reach a later boundary.
- Feasibility:
   an explicit negative control preserves the existing profile;
  no SDK change is requested.
- Support:
   the default SDK rendering and linked-source path were exercised successfully.
- Contribution policy:
   not investigated because no upstream contribution is proposed.
- Maintainer willingness:
   not investigated because no upstream change is requested.
- Prototype:
   the consumer-side control passed;
  installed SDK sources remain unchanged.

## SDK 1.0.4 empty append configuration is not an empty published entry

### Owned fixture symptom and cause

The private `run-input-carriers.test.mjs` fixture in `proc_90fe` configured
`appendSystemPrompt: ['', 'Retained appendix', '']`
and incorrectly expected the resource loader to publish all those positions.
Node's `assert.strictEqual` raised `AssertionError [ERR_ASSERTION]` with `1 !== 3`.
The native agent retained that assertion text in its synthetic terminal error;
the fixture then failed its expected assistant text check.

Installed SDK 1.0.4
`package/pi-plugin/auto-mode/node_modules/@earendil-works/pi-coding-agent/dist/core/resource-loader.js:100-103`
treats a falsy configured prompt input as absent:

```javascript
// package/pi-plugin/auto-mode/node_modules/@earendil-works/pi-coding-agent/dist/core/resource-loader.js
function resolvePromptInput(input, description) {
    if (!input) {
        return undefined;
    }
```

The same file at lines 481 to 486 resolves and filters configured inputs before invoking the append override:

```javascript
// package/pi-plugin/auto-mode/node_modules/@earendil-works/pi-coding-agent/dist/core/resource-loader.js
const baseAppend = appendSources
    .map((s) => resolvePromptInput(s, "append system prompt"))
    .filter((s) => s !== undefined);
this.appendSystemPrompt = this.appendSystemPromptOverride
    ? this.appendSystemPromptOverride(baseAppend)
    : baseAppend;
```

The source-custody implementation had retained the actual native output correctly.
The configuration-to-output count assumption belonged to the fixture.

### Verification and corrected fixture

`proc_26ba` passed `mise --no-env --no-hooks run test:run-inputs`
in the private `contract/integration/native-batch/` directory.
The corrected fixture asserts the resolved input first,
then constructs the empty output positions at the intended native boundary:

```javascript
// contract/integration/native-batch/run-input-carriers.test.mjs
appendSystemPromptOverride(previous) {
  assert.deepEqual(previous, ['Retained appendix']);
  return ['', 'Retained appendix', ''];
}
```

The installed type declaration exposes that callback in
`package/pi-plugin/auto-mode/node_modules/@earendil-works/pi-coding-agent/dist/core/resource-loader.d.ts:123`.
Both present-system and absent-system session variants passed.
They retain the complete ordered append sequence,
including empty outputs,
without selecting a subset by text matching.
Other passing cases cover equal clones,
empty or removed custom text,
changed append text,
added/reversed/removed/empty context entries,
late aliases,
and original snapshot ownership.
No provider requests occurred.

The callback deliberately constructs replacement outputs.
This tests their publication and downstream custody;
it does not claim that empty configuration strings survive native resolution.
Do not weaken the carrier assertion or change SDK resolution to repair this fixture.

### Upstream filing decision

There is no upstream defect or filing artifact:

- Fault:
   the fixture confused configured inputs with published outputs.
- Feasibility:
   the fixture boundary correction is implemented;
  no SDK change is requested.
- Support:
   the installed callback type accepts an output string array,
  and the actual native invocation passed.
- Contribution policy:
   not investigated because no upstream contribution is proposed.
- Maintainer willingness:
   not investigated because no upstream behavior change is requested.
- Prototype:
   the consumer-side correction passed the native session matrix;
  installed SDK sources remain unchanged.

## Owned SDK 1.0.4 hook masked terminal errors

### Symptom and cause

The private judgment-start listener failed in `proc_edda` with:

```text
# contract/integration/action-policy/judgment-start.mjs
RequestProducerError: Response has no producer observation for this exact consumer
```

The listener authenticated every assistant message before checking whether the native loop could execute it.
Pi's installed `pi-agent-core/dist/agent.js:365-383` constructs an error response in `handleRunFailure`.
That message reports a host-side failure;
it is not a model response associated by our request producer.
Our callback therefore replaced the native diagnostic with an ownership error.

The corrected private `contract/integration/action-policy/judgment-start.mjs:11-14` checks non-executing
terminal reasons before producer authentication:

```javascript
// contract/integration/action-policy/judgment-start.mjs
if (response.stopReason === 'error' || response.stopReason === 'aborted') return;
producerOwner.assertMainAgentResponse({response, stream});
```

Executable responses still require the original producer observation.
No new judgment or budget is created for the native terminal error.

### Verification and distinct size boundary

`proc_f814` passed `mise --no-env --no-hooks run test:start-failure` in the private
`contract/integration/native-batch/` directory.
A real SDK session retained its original local-stream error,
started no judgment,
and made no provider request.
`proc_a46f` passed the action controls,
including rejection of an unowned executable response.

The full-policy diagnostic `proc_052e` then exposed a separate limit:

```text
# contract/collector/rule-relevance-sdk-copy/stage-private/resource-owner.mjs:197
Base-linked source collection exceeds total byte bound
```

That private collector checks:

```javascript
// contract/collector/rule-relevance-sdk-copy/stage-private/resource-owner.mjs
if (Buffer.byteLength(JSON.stringify(snapshot)) > 1048576)
  throw new SourceCollectionError('Base-linked source collection exceeds total byte bound');
```

The oversized case included an independent copy of the full policy and a later run-ancestry snapshot.
The bound was not raised.
The negative mixed-batch fixture now ends explicitly after its batch-limit check;
it does not claim support for the later oversized snapshot.
The linked-source case completed its normal follow-up and passed in `proc_9e95`,
as did the complete native suite in `proc_690f`.

### Rejected remedies and filing decision

Do not authenticate a native terminal diagnostic as a model response.
Do not weaken authentication for executable tool proposals.
Do not discard independent equal-text sources or lift collection limits merely to make this fixture pass.

No upstream filing is warranted:

- Fault:
   our listener applied the wrong precondition;
  the collection limit belongs to our private consumer.
- Feasibility:
   the listener ordering correction is implemented and verified.
- Support:
   no SDK promise of producer identity for synthesized terminal diagnostics was relied on legitimately.
- Contribution policy:
   not investigated because no upstream change is proposed.
- Maintainer willingness:
   not investigated because no upstream fix is requested.
- Prototype:
   the local correction passed;
  there is no upstream defect or filing artifact.

## SDK 1.0.4 run publication resets at settlement

### Owned fixture failure

The private native prompt-custody test in `proc_8791` asserted that a request's run publication remained current
after `session.prompt()` returned.
The original collector rejected it with:

```text
# contract/integration/native-batch/prompt-custody.test.mjs
SourceCollectionError: Native run prompt was replaced after this source snapshot
```

The follow-up diagnostic `proc_0d2a` accessed `ordinal` on the now-absent publication and failed with
`TypeError: Cannot read properties of undefined (reading 'ordinal')`.
Neither failure demonstrates a native SDK defect.

### Deciding source and correction

Installed SDK 1.0.4 `dist/core/agent-session.js:1401-1408`,
inside `_runAgentPrompt`,
clears the run-specific prompt in its settlement cleanup:

```javascript
// Installed Pi SDK 1.0.4: dist/core/agent-session.js
finally {
    if (this._agentRunAbortRequested)
        this._finishCancelledRetry();
    this._failedResponse = undefined;
    this._runSystemPromptOptions = undefined;
    this._flushPendingBashMessages();
    this._flushPendingCustomMessages();
    await this._emitAgentSettled();
}
```

The private custody transform mirrors that reset through the incumbent run owner.
Request-time freshness and handler-alias independence must therefore be checked while the original run is active.
After settlement,
retain the historical snapshot but expect its freshness check to reject.
The current native reader reports an absent run,
not a replacement instruction authority or permission.

The fixture now checks active-run freshness inside its local stream,
then checks reset and stale-snapshot rejection after settlement.
`proc_a6fe` passed `mise --no-env --no-hooks run test` in the private `contract/integration/native-batch/` directory.
Active-run freshness and late-alias independence passed;
post-settlement,
copied-snapshot,
reload,
and disposal rejection passed.
The native handler and all source owners remain real;
only the model stream is local test data.

### Rejected reading and upstream decision

An absent run after settlement is not evidence that the native capture failed.
Keeping a completed run artificially current would contradict the SDK's reset boundary.
No upstream change is proposed:

- Fault:
   the fixture asserted freshness after native reset.
- Feasibility:
   correct the consumer's assertion phase.
- Support:
   no SDK promise of post-settlement run-publication currency was found.
- Contribution policy:
   not investigated because no upstream fix is requested.
- Maintainer willingness:
   not investigated because no upstream fix is requested.
- Prototype:
   the local correction passed;
  there is no upstream defect or filing artifact.

## SDK 1.0.4 private-copy license path assumption

### Symptom and cause

The owned current-SDK copy task failed in `proc_516a` at `copyFileSync`:

```text
# Private native-batch preparation
Error: ENOENT: no such file or directory, copyfile '.../pi-coding-agent/LICENSE' -> '.../.sdk-private/LICENSE'
```

The initial private `contract/integration/native-batch/prepare.mjs:50`
assumed the installed package shipped a file named `LICENSE`.
`ls --all` showed no such file in either installed 1.0.4 package directory.
This was an owned setup assumption,
not evidence of an SDK runtime defect.
The source copies had been written,
but the task had not completed and no SDK test had run.

### Verified recovery

The upstream [license](https://github.com/earendil-works/pi/blob/main/LICENSE)
matched the retained MIT notice.
The consumer now keeps that notice in `PI-LICENSE.txt`.
Current `contract/integration/native-batch/prepare.mjs:50` copies it explicitly:

```javascript
// contract/integration/native-batch/prepare.mjs
copyFileSync(join(import.meta.dirname, 'PI-LICENSE.txt'), join(output, 'LICENSE'));
```

`mise --no-env --no-hooks run test` in the private `contract/integration/native-batch/` directory
completed preparation and the native cases in `proc_3e64`.
The later termination and repeated-group cases passed in `proc_7fcb`.
The installed SDK files were not edited,
and no provider request was made.
The retained notice is independent of package layout;
future upstream licensing changes still require review rather than silently reusing it.

### Rejected approach and filing decision

Do not assume an npm package includes the repository-root license filename.
Do not remove the notice merely to make preparation succeed.

No upstream filing is warranted:

- Fault:
   the failing path was chosen by our consumer.
- Feasibility:
   the consumer correction is implemented and exercised.
- Support:
   no SDK promise of that installed filename was relied on legitimately.
- Contribution policy:
   not investigated because no upstream change is proposed.
- Maintainer willingness:
   not investigated because no upstream fix is requested.
- Prototype:
   the consumer fix is verified;
  there is no upstream defect prototype or public filing artifact to add.

## Owned prospective-input planner omitted rendered working-directory context

### Symptom and cause

The private prospective-input preparation,
`proc_6d51`,
stopped before publishing its manifest or launching an SDK worker:

```text
# Private prospective fixture preparation
AssertionError [ERR_ASSERTION]: assert(!body.includes(oldRoot))
```

The owned planner only rebased exact path values inside JSON.
Read-only `proc_cd8e` found that the complete captured main request also contained
the fixture directory inside a rendered system-message suffix:

```text
# Captured main-request message content
<cwd>
fixture-directory
</cwd>
```

The source was the private contract's
`contract/collector/program-rule-prospective-sdk/prospective.mjs`.
Its string branch left non-JSON text unchanged,
and its final old-root check correctly rejected that incomplete prediction.
This was a planner coverage defect,
not an established Pi or provider fault.

### Verified correction and guard witness

The fresh `program-rule-prospective-sdk-v2` sibling rebases only the inspected read-path fields
and the exact terminal working-directory suffix.
It preserves the policy prefix,
other messages,
and all question content.
Reverse rebasing must deep-equal the complete seed body.

`proc_4a49` verified that the predicted 256,710-byte body exactly matched
the body produced by a fresh native judgment:
one SDK session,
two local main requests,
one fake guard request,
205 original captured records,
and no tool execution.
This is a finite fixture profile,
not arbitrary text rewriting or source authority.

The mismatch control `proc_6418` added a final newline to the expected body only.
The original assessment recorded one attempt,
but the body gate stopped before any fake network fetch.
Both tool members remained `unentered`,
and the expected worker failure and persisted tool errors were retained.

The paired `proc_8812` removed only the exact authored body-equality guard.
Its fake fetch count became one;
the zero-fetch assertion failed with
`AssertionError`,
`ERR_ASSERTION`,
actual one and expected zero.
A later body verifier could detect the mismatch,
but cannot replace pre-dispatch enforcement.
Neither control contacted a provider or executed a tool.

### Rejected remedies and filing decision

Do not strip the working-directory context,
globally substitute arbitrary prompt text,
or reopen the failed preparation namespace.
All source,
stream,
persistence,
and cleanup checks remain separate from semantic qualification.

No upstream filing:
the fault was in the owned planner;
no upstream repair is established or proposed.
Native SDK support was exercised by the corrected fixture.
Contribution policy and maintainer disposition were not assessed because no upstream change is needed.
The demonstrated repair is consumer-side,
not an upstream patch.

## Full-rule binding loops and genuine dependency retirement

### Measured failure and bounded remedy

The first 205-rule native action fixture,
`proc_e496`,
failed during capture with `GoverningRuleBindingError: Governing-rule binding is no longer current`.
Its serializer omitted the cause,
so the original underlying failure remains unestablished.
Fresh instrumented attempt `proc_7e9b` recorded:

```text
# program-rule-sdk-diagnostic/controls-private/native-text/hook-failure.json
DependencyDeadlineError: Original dependency assessment deadline expired
lastCaptureIndex: 45
dependencyTraceCalls: 26485
remainingMs: -0.2679229999994277
```

The original signal was not aborted,
and the protected policy digest and mode still matched.
That observation identifies the diagnostic attempt's deadline failure,
not every possible source of the earlier wrapped error.

The old resolver repeated complete original dependency validation per binding:

```javascript
// Private program-rule-sdk-copy/stage-private/child-constructor.mjs:39
ready();const instruction=instructionBindings.get(bindingHandle);
```

Its rule reader also invoked the original source callback,
which performed another `ready()` check.
The private owner now batches immutable binding work between complete entry and exit checks.
It keeps the original signal and budget checks per rule,
and uses separate synchronous segments before and after the transport await.
No deadline extension or omitted rule is involved.

`proc_ca71` completed all 205 fake estimates and canonical captures,
recording 628 dependency-trace callbacks and no native tool execution.
`proc_b707` checked source changes at the exit boundary and across transport await,
clock expiry,
reentry,
and an exact omission of the postcheck.
The omission produced the authored `Missing expected exception.` witness.
These are finite Node 26.10.0/SDK 1.0.2 profiles,
not filesystem atomicity or a representative latency benchmark.

### Keep phase misuse separate from genuine source failure

The follow-up diagnostic `proc_e02b` exposed distinct paths:
a direct dependency check could fail without retiring captured batch evidence;
a late attempt to capture more evidence correctly rejected the closed phase
but incorrectly retired evidence already captured successfully.
Its closed-phase fixture then failed a stale expectation that the SDK would record no rejection.
That fixture error does not negate its completed in-hook checks.

The source check now retains its first genuine failure centrally:

```javascript
// Private program-rule-batch-sdk-copy-v2/stage-private/child-constructor.mjs:36
function dependencyCheck(read){
  try{return read();}
  catch(error){evidenceEligibilityFailure??=Object.freeze({cause:error});throw error;}
}
```

Caller phase checks remain outside this catch.
Canonical evidence authentication still precedes dependency validation.
`proc_8424` verified direct,
per-claim,
and batch retirement after source restoration,
stable first causes,
and harmless refusal of late collection calls.
No model score or retained record became permission.

### Cancellation diagnostics retain their layers

The postcheck-cancellation prefix in `proc_75d4` completed.
Its await suffix expected cancellation words in the top-level message,
but the original budget retains distinct failures:

```javascript
// Private contract/lifecycle/judgment-budget-error-occurrences/controls-private/candidate.mjs:34
throw new AggregateError([error,terminal],'Transport failure and judgment boundary failure');
```

The repaired expectation checked `ProgramRuleTransportError` and `JudgmentCancelledError` separately.
That suffix then hit another narrow fixture expectation:
the request observer wraps cancellation in its original-currentness error:

```javascript
// Private contract/lifecycle/root-lifecycle-retirement-entry/observer.mjs:35
if(scope.signal?.aborted)throw new RequestObservationStaleError('Observed request was cancelled',{cause:scope.signal.reason});
```

The surrounding catch emits `Observed request is no longer current` with that cause.
`proc_7d8a` verified the exact aggregate,
original native signal,
observer cause,
permanent retirement,
empty worker stderr,
persisted error outcomes,
and disposal on the corrected graph.
The completed postcheck prefix was not replayed.

### Rejected remedies and filing decision

Do not extend or restart the five-second budget,
drop indexed rules to make the fixture finish,
cache freshness across an await,
flatten distinct cancellation errors,
or treat caller phase misuse as a new source failure.
The batching remedy changes validation granularity within synchronous owned operations;
it does not qualify semantic answers or grant permission.

The observed defects and assertion mismatches belong to owned private integration code.
No Pi or Gateway defect was established.
Upstream fault,
fixability,
supported-use-case claims,
contribution acceptance,
and willingness therefore do not justify filing.
The local changes have the named native controls;
no upstream issue or comment is warranted.

## Original claim evidence must not regain eligibility after a dependency failure

### Symptom and distinct fixture failure

The private candidate-aware evidence handoff first failed in `proc_b5b2` with:

```text
# Private candidate-claim-evidence-sdk-consumer, after the next request generation
RequestObservationStaleError: Observed request is no longer current
```

Its cause was `Request generation changed`.
The fixture placed a retained-evidence assertion after `session.prompt()` had completed another main request.
This was correct original-request retirement,
not a restoration defect.
The intended changed/restored dependency suffix had not run.

Fresh active-request control `proc_f221` placed that suffix inside the original `beforeToolBatch` callback,
after evidence closure.
Changing disposable `b.txt` correctly rejected evidence validation.
Restoring its original bytes then incorrectly passed validation.
The authored assertion recorded:

```text
# Private candidate-evidence-restoration-sdk/controls-private/native-text/restoration-witness.json
AssertionError [ERR_ASSERTION]: Missing expected exception.
```

The SDK converted the thrown callback assertion into admission-blocked tool results.
A separate completion assertion ensured this error handback could not masquerade as a passing test.
No fixture tool executed in this failing restoration case.

### Root cause and verified repair

The original handoff authenticated the evidence object,
then performed only a current dependency check:

```javascript
// Private candidate-claim-evidence-sdk-copy/stage-private/child-constructor.mjs:107
function assertClaimEvidence(input){knownSelection().assertCapturedIdentity(input);assertDependenciesCurrent();}
```

Retained file comparison in `contract/lifecycle/external-dependency-controls/file-observer.mjs:91`
compares original path,
device,
inode,
mode,
and bytes.
It is a current comparison,
not a record that a caller's earlier eligibility assertion failed:

```javascript
// Private contract/lifecycle/native-text-read-contract/dependencies.mjs:83
function checkRetained(handle) { return revalidate({handle,verify:checkSignal}); }
```

The enclosing judgment now retains a failure sentinel for its captured evidence:

```javascript
// Private candidate-claim-evidence-sdk-copy-v2/stage-private/child-constructor.mjs:107
function assertClaimEvidence(input){
  knownSelection().assertCapturedIdentity(input);
  if(evidenceEligibilityFailure)throw new JudgmentConstructionError('Original relation evidence lost dependency eligibility',{cause:evidenceEligibilityFailure.cause});
  try{assertDependenciesCurrent();}
  catch(error){
    evidenceEligibilityFailure=Object.freeze({cause:error});
    throw error;
  }
}
```

Checking canonical identity first keeps copied or cross-claim records from poisoning genuine evidence.
A genuine shared-dependency failure retires all captured evidence in that original judgment.
The object sentinel also retains falsy thrown values rather than treating them as absence of failure.
This is evidence eligibility,
not a new registry or a policy verdict.

`proc_d959` checked the canonical-capture owner and its exact identity-guard omission.
`proc_7bc6` verified the repaired native suffix on SDK 1.0.2 and Node 26.10.0:
one session,
two local main requests,
one fake guard request,
and zero fixture executions.
It checked successful validation after inference-clock expiry,
changed/restored dependency refusal,
shared-claim retirement,
copy rejection before retirement checking,
persisted outcomes,
source hashes,
empty worker stderr,
and disposal.
`proc_f9b4` independently reconciled both failed namespaces without SDK replay.

### Rejected remedies and filing decision

Do not refresh the old judgment,
reset its budget,
use retained JSON as live evidence,
or waive a stale original request after its successor begins.
Moving the fixture check into the original callback repaired the placement error only;
it did not fix restoration eligibility.
The failure sentinel intentionally requires a new original judgment after a genuine eligibility failure.
It does not provide filesystem atomicity or detect every unobserved transient source change.

This is owned private integration code.
No Pi or Gateway fault was established,
so upstream fixability,
supported-use-case claims,
contribution acceptance,
and willingness do not justify an upstream filing.
The local repair is prototyped and verified;
no upstream issue or comment is warranted.

## Relevance canary and native hook diagnostics

### Symptoms and separate failures

The Pi 1.0.2 private relevance-hook worker `proc_6478` exited successfully.
Its verifier rejected the copied stdout constants of five sessions and ten requests.
The actual persisted result contained one session and four local requests.
The source discrepancy is explicit:

```javascript
// Private contract/collector/relevance-before-request-sdk-v3/probe.mjs:139, retained faulty summary
console.log(JSON.stringify({complete:true,SDKAgentSessions:5,localWireRequests:10,semanticAttempts:0,nativeReads:[4]}));
```

Read-only reconciliation `proc_e53e` checked the incorrect stdout exactly,
then verified actual counts from the completed result and persistence artifacts.
It also verified source hashes,
empty worker stderr,
disposal,
and question counts `[205, 0, 1, 0]`.
No successful SDK worker was replayed to fix its summary.

Do not merge this reporting error with the prior fixture failures:
`proc_f2a7` retained an unrelated 10,000ms synthetic clock offset into its second run;
`proc_e353` reported the cached view's creation-time question count on warm operations.
The corrected preprocessor reads the current operation's sender instead:

```javascript
// Private contract/lifecycle/rule-relevance-preprocessor-v2/boundary.mjs:35
const requestedRuleIds=transport?.record?.originalQuestionIDs??Object.freeze([]);
```

The later real relevance canary succeeded,
including warm and restarted-owner reuse,
as verified by `proc_f465`.
Its separate outer metadata checker printed:

```text
# Outer source/namespace checker, after its completion record
logger internal error: sink verification failed for entry 3: Timed out after 5000ms: sink 3 verify
```

The worker and immediate controller had empty stderr.
This diagnostic does not describe a Jev response failure or a tool-judgment deadline.

### Logger source path and verified workaround

`package/git-executable/core/src/resolve-real-git.ts:10` imports the incumbent tagged logger.
Its default Node sink order places the asynchronous file sink at index 3:

```typescript
// package/logger/core/src/default-sinks.node.ts:38
return [
  createConsoleSink(),
  createSessionStorageSink(),
  createLocalStorageSink(),
  createFileSink(),
];
```

`package/logger/core/src/create-logger.ts:351` starts each verification under a deadline:

```typescript
// package/logger/core/src/create-logger.ts:358
available: await withHostTimeout({
  label: `sink ${entryIndex} verify`,
  ms: verifyTimeoutMs,
  promise: entry.sink
    .verify(),
},),
```

The file sink awaits filesystem operations in `package/logger/core/src/sink/file.ts:169`.
The outer checker then synchronously waited for its child while logger startup could still be pending.
`proc_6308` isolated this mechanism on Node 26.10.0 with disposable homes and log directories:
blocking for 5,200ms after initiating the default logger reproduced the exact diagnostic;
awaiting `logger.flush()` before the same blocking child produced empty stderr.
Neither control contacted a provider or started an SDK session.

```javascript
// Private contract/collector/relevance-launcher-logger-controls/probe.mjs:10, consumer-side ordering remedy
if(mode==='drained')await logger.flush();
```

The workaround waits for incumbent sink verification and queued logging before blocking.
It does not filter stderr or remove logging.
Its tradeoff is waiting for the logger's bounded flush lifecycle;
it does not guarantee a failing filesystem sink becomes available.
The paid canary and its consumed launcher remain historical artifacts,
not targets for replay.

### Rejected remedies and upstream filing decision

Do not rewrite retained stdout,
reset consumed namespaces,
suppress the logger diagnostic,
or repeat the paid relevance request to repair launcher reporting.
These are owned fixture,
accounting,
and consumer lifecycle issues,
not demonstrated Pi or Gateway bugs.
Upstream fault is not established;
upstream fixability,
support,
contribution acceptance,
and willingness are therefore not grounds for a filing.
The consumer-side lifecycle remedy is prototyped and checked locally.
No upstream issue or comment is warranted by these observations.

## SDK 1.0.2 update retires installed 1.0.0 paths

A background Pi update removed the installed 1.0.0 package paths used by the completed private fixtures.
During that transition,
the live Codemode tool reported `Cannot find module 'quickjs-wasi/quickjs.wasm'`
with a 1.0.0 bundled-chunk require stack.
After the update,
a fresh live Codemode invocation succeeded and the installed Pi package directories were 1.0.2.
This records the interruption and recovery,
not a diagnosed upstream Wasm defect.

The old qualification namespaces and hashes remain unchanged.
`proc_fb7e` compared the latest listed-source manifest with current installed paths:
307 unique entries,
297 matching byte digests,
nine changed entries,
and one missing parser path.
The missing `yuku-parser@0.14.0` entry was replaced by an explicitly selected 0.17.0 entry
in fresh staging,
not by altering the old manifest.
These counts do not describe the full transitive dependency graph.

### Account for source changes before relocation

The SDK 1.0.2 session-source change was isolated to the HTML tool-renderer callback.
The fresh staging controller reverses the new callback to the old form
and requires the entire resulting file to match the historical digest:

```javascript
// Private contract/collector/sdk-1-0-2-paired-copy/stage.mjs:26, session delta check
assert.equal(sha(currentSession.replace(newLine,oldLine)),sessionChange.expected);
```

The old and new callback fields are:

```javascript
// SDK 1.0.0: dist/core/agent-session.js:3445, historical HTML renderer field
getToolDefinition: (name) => this.getToolDefinition(name),
```

```javascript
// SDK 1.0.2: dist/core/agent-session.js:3445
getToolRenderers: (name) => this._extensionRunner.resolveToolRenderers(name, () => this.getToolDefinition(name)),
```

The same delta is applied to the private session copy.
Other inspected changes include the extension renderer resolver,
AI sampling-parameter resolution,
and Codemode output limits.
Those changes are not qualified merely by an import-path replacement.
The upstream comparison is
[Pi 1.0.0 to 1.0.2](https://github.com/earendil-works/pi/compare/v1.0.0...v1.0.2);
the 1.0.2 release commit is `cd32f7725fdbddbaecdff5b1e68491563394e0ca`.

### Preserve separate failed staging attempts

The 26-artifact copy in `proc_59df` staged successfully,
but an import inspection found its separately shared private manager still referenced old installed paths.
No SDK worker had used that staging.

The manager relocation initially used the wrong manifest:
`proc_2f8f` failed its assertion that the manager pin was present in the parent graph inputs.
That pin belongs to the manager's own manifest.
The next attempt,
`proc_0681`,
used the right manifest but called `realpathSync` on every historical entry.
It reached a removed installed path and failed with `ENOENT` before finding the retained manager.

The corrected lookup selects the exact retained artifact path before reading its bytes:

```javascript
// Private contract/collector/sdk-1-0-2-closure-copy-v3/stage.mjs:9, manager lookup
managerPin=managerManifest.sources.find(entry=>entry.path===oldManager);
```

Its unchanged native manager source is checked separately against the owning historical digest.
`proc_c2ea` staged 27 artifacts and checked 45 reachable private literal-import modules
and 46 installed literal edges.
This is a bounded literal-import check,
not complete transitive attestation or runtime qualification.

Two dependent checks were started before the first manager staging had succeeded.
`proc_0801` failed with `ERR_MODULE_NOT_FOUND` before parsing;
`proc_e8a6` failed with `ENOENT` for the missing staging manifest before SDK import.
Those were orchestration errors,
not parser or SDK behavior failures.
Their original namespaces remain preserved.
Replacement checks started only after `proc_c2ea` had exited successfully
and its receipt was reconciled.

### Verified continuation and limits

The new parser profile,
`proc_7e8e`,
passed the existing bounded result-program assertions on parser 0.17.0:
six positive source forms,
four producer values,
17 source rejections,
five value rejections,
and two ownership rejections.
It created no SDK session.

The new SDK consumer,
`proc_89f9`,
passed one actual 1.0.2 session,
two injected requests,
one original canned assessment,
and two root definitions.
The intrinsic resource binding remained current after the decision deadline closed.
This does not replay or requalify every historical SDK matrix.

The updated SDK 1.0.2 path then passed `proc_35e7`:
three actual sessions,
six injected requests,
three original canned assessments,
child-entry counts `[3, 1, 1]`,
and sibling-root counts `[1, 0, 1]`.
This rechecks the result-bound successor and guard-versus-ordinary-error behavior
with parser 0.17.0 and the intrinsic resource collector.
It is targeted changed-dependency qualification,
not a claim that every historical SDK profile was rerun.

These phases use `mise --no-env --no-hooks run stage`
or `mise --no-env --no-hooks run check`
from their named directories under the private repository's `contract/collector/`.
Existing output namespaces are not replayable.
No installed package,
dependency lockfile,
or protected policy file was edited by this recovery.

The workaround is fresh source-checked relocation plus targeted changed-dependency verification.
Repointing old receipts,
recreating old installed directories,
assuming the staging manifest contains every shared dependency,
and treating process start as prerequisite completion are not accepted alternatives.
The staging failures were owned harness errors;
no upstream issue or fix is proposed.
The existing upstream-filing limits continue to apply.

## Owned fork consumer omitted a persisted system entry

### Symptom and evidence

The separate private fork mechanics controller `proc_90f8` failed after one second.
Node emitted `AssertionError [ERR_ASSERTION]` at the parent acceptance gate:
`Historical fork consumer failed; detailed diagnostics remain private; stop without replay`.
The child exited one without a signal or bounded stop,
with empty stdout and 818 bytes of private stderr.

Fixed-coordinate inspection `proc_5f79` identified the owned assertion at
`contract/lifecycle/fork-mechanical-controls/attachment.mjs:128`.
Strict role metadata reader `proc_2412` failed without exporting its rejected token.
Separately declared enum-only correction `proc_ef03` admitted:
`system/user/assistant/toolResult/assistant`.
The reference expected `user/assistant/toolResult/assistant`.
The consumed runtime reached the completed origin-session reference,
but rejected before actual fork construction.
Its child/reset suffix remains unqualified.

### Source cause and rejected premise

The error was in the owned consumer's four-role expectation,
not demonstrated upstream behavior failure.
The earlier source-only clearance missed the persisted-prefix premise.

Pinned Pi source `packages/coding-agent/src/core/agent-session.ts:1420`
constructs a structured system message when prompt sections change:

```ts
// packages/coding-agent/src/core/agent-session.ts:1420
return sections ? { role: "system", content: "", sections, timestamp: Date.now() } : undefined;
```

The same file at line 934 includes system messages in ordinary persistence:

```ts
// packages/coding-agent/src/core/agent-session.ts:934
event.message.role === "system" ||
event.message.role === "user" ||
event.message.role === "assistant" ||
event.message.role === "toolResult"
```

This source admits persisted system entries;
the private enum receipt establishes the actual fixture's leading role.
Those facts do not establish every section's producer,
configuration binding,
or authority.
Read-only source remains commit `f07218c4d4bbc12bef056a7058c3dd49dfe41abe`.

### Verification and next correction

The consumed command was `mise --no-env --no-hooks run probe`
from the private `contract/lifecycle/fork-mechanical-controls/` directory.
It must not be launched again.
The separate full closure freeze `proc_2cae` passed ten syntax checks,
not runtime correctness.
The completed root sensitivity controls remain a different passing catalog,
not evidence for added fork guards.

At the consumed failure frontier,
no verified fork correction had run.
A separately declared correction must preserve the exact SDK prefix,
ordered call/result/stop,
selected records,
complete branch JSON,
and original immutable evidence linkage.
Accepting a system role cannot admit instruction authority or a human grant.
Any configuration-bound prefix assertion needs its deciding getter/projection source,
not a guessed section schema.
Do not fix the consumed source,
filter away every system message,
or relabel an intact test as guard sensitivity.

### Separate corrected consumer result

The consumed source and runtime remain unchanged.
Separate `contract/lifecycle/fork-system-prefix-controls/` captures an opaque immutable origin prefix
and full branch JSON after completed SDK origin execution,
then validates exact message order with named positions.
Before child publication,
a private helper follows actual persisted parent links and checks selected path/prefix JSON.
Configuration-derived sections equality is not needed for copying consistency;
configuration fidelity and instruction authority remain unqualified.

Pure freeze `proc_ba0c` and eight-case synthetic acceptance/rejection `proc_16e6` passed.
New full digest intake `proc_bd33` and twelve-module freeze `proc_7264` preceded one protected runtime.
`proc_72e8` passed two actual SDK sessions,
four scripted responses,
two inert callbacks,
and one reserved fork:
exit zero,
stdout 1,058 bytes,
empty stderr,
no signal or bounded stop.
Both sessions had their own completed non-error persisted result;
origin reset and independent child reset preserved the described evidence/snapshot boundaries.
No external fetch/models,
fixture action,
or grant write occurred;
current human eligibility remained unestablished.
This fresh pass is not a replay or retroactive pass for `proc_90f8`.

Input rejection controls do not prove guard necessity or omission sensitivity.
Wrong-root/cross-owner capture,
ledger recapture,
and persisted-path rejection branches remain untested.
Full cache/target/source/deadline finalization,
raw-byte/complete-property freshness,
current human grants/directives,
and five-second preparation-inclusive handback remain open.

### Upstream filing decision

- Fault:
   the observed mismatch is the owned transcript expectation;
  no upstream defect is established.
- Fixability:
  intact consumer correction passed separately;
  sensitivity and complete finalization remain open;
  no upstream change is required by this evidence.
- Supported use:
   the inspected SDK persistence path explicitly includes system messages.
- Contribution policy:
   no external contribution is proposed.
- Maintainer disposition:
   not assessed because no defect or contribution is established.
- Prototype:
   no upstream patch is justified;
  the separate consumer epoch passed only its finite intact mechanical reference.

Nothing is filed or drafted upstream.
This incident does not establish current human-grant eligibility,
human-authorized transfer,
complete lifecycle coverage,
or five-second handback.

## Symptom

The private auto-mode SDK preparation controller stopped before evaluating package code:

```text
AssertionError [ERR_ASSERTION]: Dependency staging prerequisites unresolved; retain inventory
3 !== 0
```

The emitter was the owned `contract/sdk/inventory.mjs:89`,
not Pi,
pnpm,
or a provider.
Process `proc_1af5` retained its output before throwing.
It recorded missing declared `@aws-sdk/client-bedrock-runtime` and `@google/genai` dependencies,
and rejected `proper-lockfile` because its resolved directory was outside the external package store.
This was not an SDK startup failure.

This preparation incident is separate from the
[instruction-view observations](pi-instruction-snapshots.md)
and the unresolved historical preparation stall.
No production package,
lockfile,
policy,
or provider configuration was changed.

## Source identity

The private repository is `~/temp/agent/auto-mode-consumer-contract.mDLkyNoP`.
Private paths in this report are relative to it.
The nominal inventory is retained at `contract/sdk/sdk-dependency-inventory.json`,
SHA-256 `a4db9a8556e7dc3e272a58cea164cc16e749b121b9d922d06a05c95445429f05`.

The installed SDK and Pi AI versions are `0.87.1`.
Read-only upstream source is `~/temp/agent/pi-input-provenance-2026-09-26`,
commit `f07218c4d4bbc12bef056a7058c3dd49dfe41abe`.
`packages/ai/` paths refer to that checkout.
The host metadata controllers reported Node `v26.10.0`;
no SDK runtime was started by these inventories.

## Root cause trace

### The owned admission rule excluded the existing shim

The initial controller admitted only directories under `node_modules/.pnpm`:

```javascript
// Private contract/sdk/inventory.mjs:42
const fromStore = relative(store, path);
if (fromStore === '..' || fromStore.startsWith(`..${sep}`)) {
  failures.push({ path, reason: 'dependency resolves outside the installed external package store' });
  continue;
}
```

Repository configuration intentionally selects a different owner:

```yaml
# pnpm-workspace.yaml:358
'proper-lockfile': 'link:package/pnpm-override/shim-proper-lockfile'
```

The [existing removal decision](../decision/proper-lockfile-removal.md)
and [dependency audit](dependencies.md)
already document this substitution.
The current shim's `package.json:24` declares no runtime dependencies.
Its current `index.cjs:3` imports Node filesystem,
timer,
and path builtins:

```javascript
// package/pnpm-override/shim-proper-lockfile/index.cjs:3
const { mkdirSync, rmdirSync, } = require('node:fs',);
const { setTimeout: sleep, } = require('node:timers/promises',);
const { dirname, basename, resolve, } = require('node:path',);
```

The remedy is to preserve and bind this exact owner,
not silently omit it or restore the upstream dependency.
No general claim of shim parity is established by reading its metadata.

### Published declarations are not the effective workspace selection

The initial controller combined package dependency declarations
and treated every unresolved non-optional declaration as a staging failure:

```javascript
// Private contract/sdk/inventory.mjs:56 and 63, selected statements
const declared = { ...metadata.dependencies, ...metadata.peerDependencies, ...metadata.optionalDependencies };
else failures.push({ ...record, reason: 'required installed dependency missing' });
```

Current repository configuration explicitly removes the reported cloud SDK edges:

```yaml
# pnpm-workspace.yaml:234
'@earendil-works/pi-ai>@aws-sdk/client-bedrock-runtime': '-'
'@earendil-works/pi-ai>@google/genai': '-'
```

The reviewed configuration hash is
`4a6a413b76bc2d6cea134bffd552f7eb77668aea9710bba29b70ef48a191b55c`.
The original controller's store-only and all-declarations-installed assumptions
were not valid for this configured graph.
The inventory did not demonstrate that these packages were needed by the planned scripted provider.

### Lazy provider entry points are narrower evidence than startup qualification

Current source defers the Google implementation imports:

```typescript
// packages/ai/src/api/google-generative-ai.lazy.ts:4
export const googleGenerativeAIApi = (): ProviderStreams => lazyApi(() => import("./google-generative-ai.ts"));
```

```typescript
// packages/ai/src/api/google-vertex.lazy.ts:4
export const googleVertexApi = (): ProviderStreams => lazyApi(() => import("./google-vertex.ts"));
```

`packages/ai/src/api/bedrock-converse-stream.lazy.ts:26` similarly supplies
an asynchronous implementation loader to `lazyApi`.
The loader is called from the streaming paths:

```typescript
// packages/ai/src/api/lazy.ts:73, selected statements
stream: (model, context, options) =>
  lazyStream(model, async () => (await load()).stream(model, context, options)),
streamSimple: (model, context, options) =>
  lazyStream(model, async () => (await load()).streamSimple(model, context, options)),
```

The installed compiled lazy-route files were also inspected and hashed.
This supports retaining the configured removals for the proposed scripted-provider path.
It does not prove complete static reachability,
actual package import success,
or availability of the removed providers.
An unexpected attempt to use a removed route must stop the probe,
not install dependencies or fall back to another provider.

## Verification

These are recorded private invocations,
not replay instructions:

```bash
# Private contract/sdk
mise --no-env --no-hooks run inventory
```

```bash
# Private contract/sdk/effective
mise --no-env --no-hooks run compose
mise --no-env --no-hooks run files
```

```bash
# Private contract/sdk/topology
mise --no-env --no-hooks run collect
```

The original inventory stopped with 54 external package records and 26 missing optional declarations.
The separate effective composition passed with 55 package roots,
including the measured shim,
while retaining every original failure and its disposition.
It verified the retained inventory hash,
reviewed override configuration,
current package metadata,
and exact shim code.

The file walk recorded 11,850 candidate files totaling 119,118,689 bytes,
no package-content symlinks,
and 11 skipped package-internal `node_modules` directories.
These are candidate filesystem entries,
not an admitted package-byte snapshot or hermetic runtime closure.

The separate topology collector recorded 96 declared edges:
68 existing symlink lookups,
2 configured cloud SDK removals,
22 absent non-Linux-x64 esbuild platform packages,
and 4 absent optional peers.
The peers are Anthropic's `zod`,
OpenAI's `ws` and `zod`,
and `proxy-agent-negotiate`'s `kerberos`.
The optional entries were not additional Pi removal overrides.

### Clean observations

- Current resolved targets still matched the retained package identities.
- Previously absent edges remained absent.
- The existing shim was admitted explicitly without broadening access to arbitrary workspace packages.
- The follow-on walk inspected the skipped directories within its fixed entry and byte caps.
- No SDK code,
  external model,
  native addon,
  session,
  or represented operation was executed.

### Rejected assumptions and artifact exclusions

- The nominal inventory failed its external-store-only and required-declaration gates.
- The skipped directories were not interchangeable with the adjacent dependency-link directories.
  They contained generated executable wrappers and local logger artifacts.
  The collector retained names,
  sizes,
  and hashes,
  not raw log contents.
  Neither category should be copied into the SDK probe automatically.
- The candidate shim subtree included a TypeScript build cache and workspace build configuration.
  Staging must select the shim's runtime files,
  declarations,
  documentation,
  and licenses rather than copying the cache or build configuration.
- Staged resolution,
  native/Wasm compatibility,
  and actual import/startup remain unverified.

### Selected artifact bytes

The separate `contract/sdk/artifacts/freeze.mjs` phase completed once in `proc_490f`.
It selected and hashed 11,847 files totaling 119,045,169 bytes
and retained 68 dependency lookup placements.
It excluded the shim's `mise.toml`,
`tsconfig.json`,
and TypeScript build cache;
none of the skipped logger artifacts or executable wrappers was admitted.

The create-new manifest is `contract/sdk/artifacts/manifest.json`,
SHA-256 `d9df0286368eecc8a54f826c80b2524f5eb22344085ed5cd9cdd84d2ec86e1e3`.
This establishes listed artifact-byte identity before staging,
not publisher authenticity,
a copied image,
or actual SDK execution.
A later policy freshness check found a changed `AGENTS.md`;
the [separate policy-epoch intake](../handover/pi-auto-mode-axiom-evaluation.md#policy-freshness-checkpoint)
must precede the SDK probe.
The dependency-byte inventory does not refresh the policy evidence.

## Verified workaround and remaining gates

`contract/sdk/effective/compose.mjs` is the verified metadata-level workaround.
It consumes the original result in a new output namespace,
binds the reviewed configuration and shim,
and gives the original failures explicit dispositions.
Its tradeoff is a deliberately restricted scripted-provider profile,
not general SDK dependency completeness.
No completed constructor or stopped original inventory was replayed.

Before SDK execution,
copy the admitted artifact bytes with hash checks,
preserve measured lookup topology,
verify staged resolution and required-dependency omission controls,
and bind the actual runtime and image identity.
Use a read-only image with declared disposable tmpfs for writable session state,
not host state mounts.
Seal the invocation,
transcript,
provider,
resource,
and lifetime caps before session construction.

## Module-preflight environment admission

The first actual-module preflight `proc_0e8c` stopped at the owned probe's assertion:

```text
Unexpected environment variable names
```

This occurred before SDK import.
A separate names-only diagnostic `proc_bf98` found `HOSTNAME`
besides the explicitly configured probe variables,
`HOME`,
and `PATH`.
No environment values were printed.
The installed tool reported Podman `5.8.7`.

The read-only source checkout is `~/temp/agent/podman-sdk-env-5.8.7`,
commit `c593b672bf3db1173aebea565ebf1a724ea196dc`.
Its default-environment processing clears defaults:

```go
// Podman pkg/specgen/generate/container.go:222
if s.UnsetEnvAll != nil && *s.UnsetEnvAll {
    defaultEnvs = make(map[string]string)
}
```

At line 240 it combines those defaults with explicit environment values.
Later hostname handling checks whether an explicit value already exists:

```go
// Podman libpod/container_internal_linux.go:537
needEnv := true
for _, checkEnv := range g.Config.Process.Env {
    if strings.SplitN(checkEnv, "=", 2)[0] == "HOSTNAME" {
        needEnv = false
        break
    }
}
if needEnv {
    g.AddProcessEnv("HOSTNAME", hostname)
}
```

The owned assumption that `--unsetenv-all` left only the enumerated explicit names was wrong.
The correction is not to admit arbitrary environment variables.
The new `contract/sdk/module-preflight-hostname` epoch supplies
`--env=HOSTNAME=sdk-preflight`,
requires that exact synthetic value,
and still rejects every other unexpected name.
Its diagnostic names unexpected keys without exposing values.

`proc_4708` passed both fixed cases:

- The intact image verified recorded bytes,
  links,
  package lookup/absence edges,
  and the resource envelope,
  then imported the real SDK barrel and observed the expected exported APIs.
- The preserved throwaway image with only TypeBox's package metadata removed
  failed the same SDK import with `ERR_MODULE_NOT_FOUND` naming TypeBox.
  This control intentionally was not admitted as an intact artifact.

Both cases observed Node `v26.10.0`,
2 GiB memory,
zero extra swap,
2 CPUs,
64 PIDs,
read-only root,
and loopback-only networking.
They used declared disposable tmpfs and a synthetic home.
Stderr was empty and the fetch counter stayed zero.
No `AgentSession` was constructed and no external model call ran.
The original failed epoch and unused original control schedule remain preserved;
the existing omission image was reused rather than rebuilt.
This qualifies the measured module-import boundary,
not actual session lifecycle,
human-input authority,
or every native/API path.

No Podman source or host configuration was modified.
No upstream contribution is proposed for the owned allowlist correction;
no claim about absence of an upstream issue or contribution policy is made.

## Fixture import-condition correction

The first actual-session attempt `proc_5d43` stopped before constructing any session:

```text
Error [ERR_PACKAGE_PATH_NOT_EXPORTED]: No "exports" main defined in .../@earendil-works/pi-ai/package.json
```

Node's stack identifies `require.resolve` at the owned `contract/sdk/session/probe.mjs:41`.
The fixture first imported the SDK barrel successfully,
then incorrectly used a CommonJS resolver to select Pi AI's helper entry:

```javascript
// Private contract/sdk/session/probe.mjs:41
const ai = await import(pathToFileURL(require.resolve('@earendil-works/pi-ai')).href);
```

The staged Pi AI `0.87.1` package metadata declares an import-only root condition:

```jsonc
// Staged @earendil-works/pi-ai/package.json:13, selected root export.
{
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "import": "./dist/index.js"
    }
  }
}
```

This is the owned resolver choosing the wrong public condition,
not missing SDK files or an instruction-view finding.
It does not establish a general inability to use ESM from CommonJS.
The separate `contract/sdk/session-esm` correction follows the already verified dependency edge,
checks the published `exports['.'].import` target,
and imports that file URL.
The original failed epoch remains unchanged.
The case-reference hashes remain identical:
`9acfe7b38a559b9044dceeed88b24c4caf5886d46093d0087f2743de9fa83020`.
No expectation was relabeled to pass.
The corrected phase passed all 4 actual-session cases in `proc_da53`,
with 8 scripted responses,
4 inert tool executions,
empty stderr,
and zero fetch or external model calls.
`proc_a79f` reconciled the saved raw outputs and unchanged references without replay.
Result SHA-256:
`18ae734862240c7c28d7fb235cfce2972f6de015841311fcf0c45762f8a17e98`.
The [SDK observations](pi-instruction-snapshots.md#actual-sdk-session-observations)
record the measured instruction-view differences and remaining authority limits.

### Current requester repeats the import-condition mistake

The private current-requester stage `proc_c9c9` failed under Node `26.10.0`
before SDK imports,
terminal launches,
or human responses.
Node emitted `ERR_PACKAGE_PATH_NOT_EXPORTED`
for the installed Pi coding-agent `1.0.2` root export.
The failing call was again a CommonJS resolver:

```javascript
// Private contract/human-origin/current-requester-copy/stage.mjs:18
const resolved=require.resolve(specifier);
```

The installed coding-agent `package.json` has the same root shape shown for Pi AI:
an `import` target,
but no `require` or `default` target.
The current recovery inspects that metadata instead of interpreting resolver failure as a missing package:

```javascript
// Private contract/human-origin/current-requester-copy-v2/stage.mjs:20
const target=specifier==='@earendil-works/pi-tui'
  ? (assert.equal(metadata.exports,undefined),metadata.main)
  : metadata.exports['.'].import;
```

This selector is restricted to the inspected SDK,
TUI,
and TypeBox imports,
not a general Node resolution algorithm.
The fresh stage passed as `proc_aed8`;
the subsequent pre-spawn instrumentation stage passed as `proc_4363`.
`proc_3e9a` executed the current requester with its actual default launcher and request-owned helper
using disposable scripted terminal/editor inputs.
Its approved,
denied,
and blank-response paths completed with empty helper stderr and removed answer workspaces.
That result verifies this current consumer profile,
not a genuine human origin or permission.

The failed stage remains unchanged.
No dependency installation,
installed package edit,
or upstream fix was needed.
The upstream-filing decision remains unchanged:
this was an owned resolver mistake,
not a demonstrated Node or Pi defect.

## Owned documentation renderer dependency-path drift

The `proc_2b5e` handoff renderer failed under Node `v26.10.0` with `ERR_MODULE_NOT_FOUND`:
its absolute `micromark@4.0.2` import no longer existed.
The failing owned import is retained:

```javascript
// Private contract/sdk/docs/render-confirmation-handoff.mjs:5
import { micromark } from '/var/home/user/Monochromatic/node_modules/.pnpm/micromark@4.0.2_supports-color@10.2.2/node_modules/micromark/index.js';
```

An uncapped `find node_modules -type d -name micromark` found the existing `4.0.3` package.
The installed declarations were read before invoking it.
A separate `render-confirmation-handoff-current.mjs` changed only the consumer's import coordinate
and passed the complete handoff render and heading/key/emphasis assertions through
`mise --no-env --no-hooks run confirmation-handoff-current:render`.
The old failed renderer and diagnostic are preserved.
No installation,
lockfile edit,
production linter patch,
SDK restaging,
or frozen genuine-input change was made.

This is an owned stale import coordinate,
not an established micromark or package-manager defect.
The concurrent lockfile change does not identify the actor or mechanism that removed the old package directory.
The tradeoff is renderer version `4.0.3` for this separate documentation check;
old renders and frozen runtime references are not silently refreshed.
Do not use the obsolete absolute coordinate or claim the new render proves semantic equivalence.
No upstream filing or upstream-fix prototype is justified by this consumer mistake.

## What does not work

Treating every nominal dependency declaration as a mandatory installation
would undo intentional repository selection.
Treating every workspace-resolved package as inadmissible would discard the incumbent shim.
Neither assumption is repaired by installing another provider,
rewriting the lockfile,
or editing upstream source.
Those changes were not attempted.

A package name,
manifest hash,
or successful metadata walk does not prove actual SDK startup,
complete instruction coverage,
or genuine human authority.

## Upstream filing artifact

Nothing is filed or drafted.
The diagnostic came from the owned inventory's admission assumptions,
not an established upstream defect.
The existing dependency decision and audit already cover the configured removals and shim.

### Upstream filing decision

1.  Upstream fault is not established;
    the rejecting controller is owned private preparation code.
2.  The measured remedy belongs in consumer staging,
    not an upstream package modification.
3.  Pi's SDK and custom-provider surfaces are supported;
    this exact staged profile still needs actual execution verification.
4.  The previously inspected pinned `CONTRIBUTING.md` requires understood contributions
    and appropriate human-voice or disclosed AI participation.
    No contribution is proposed here.
5.  No maintainer refusal or willingness is inferred from a local inventory failure.
6.  No upstream patch was prototyped because there is no upstream defect target.
    The tested artifact is a separate owned metadata composition.

No new upstream tracker search was used to claim absence of an issue.
A future upstream filing would require its own scope,
duplicate,
contribution,
and demonstrated-fix checks.

## Separate lifecycle API intake stop and staged diagnostic

### Symptom and established boundary

The new synthetic SDK API intake `proc_256c` stopped at its 15-second child bound.
Its outer Node driver emitted
`AssertionError [ERR_ASSERTION]: SDK API intake failed; synthetic diagnostics retained privately; no replay`.
Retained outcome metadata reports child status null,
`SIGTERM`,
`ETIMEDOUT`-based bounded stop,
and empty private stdout/stderr.
No synthetic fixture-cwd or session-files directory was created.
No actual SDK session-manager instance,
provider,
model,
genuine original,
or GUI interaction was reached.
The failed epoch is preserved without replay.

### Source trace and cause limit

The owned `contract/lifecycle/api-intake/run.mjs:21` launches Node with
`timeout: plan.childDeadlineMs` and private file-backed stdout/stderr.
Its `run.mjs:23` records `child.error?.code === 'ETIMEDOUT'`.
The worker's `probe.mjs:17` verifies every retained dependency file before importing the SDK.
`probe.mjs:25` then awaits the SDK barrel;
`probe.mjs:27` creates the synthetic fixture directory before
`probe.mjs:28` constructs the first `SessionManager`.
These are private qualification-repository paths,
not production changes or upstream patches.

```javascript
// Private contract/lifecycle/api-intake/probe.mjs:25 to 28
const { SessionManager } = await import(pathToFileURL(join(staging, 'repository', manifest.sdkRoot, 'dist/index.js')).href);
const fixtureCwd = join(privateRoot, 'fixture-cwd');
mkdirSync(fixtureCwd, { mode: 0o700 });
const manager = SessionManager.inMemory(fixtureCwd);
```

This places the recorded stop before API construction,
not at a reset,
fork,
or permission decision.
The original worker had no stage markers;
dependency validation versus import versus filesystem setup remains unassigned.
Empty output alone does not identify a cause.

### Verification and non-workaround result

A distinct staged diagnostic `proc_0a95` used the same retained SDK graph and a 60-second bound
within the historical SDK consumer envelope.
It constructed no session manager and did not replay any original lifecycle API check.
It checked 11,847 dependency files,
completed the barrel import and synthetic directory creation,
and exited zero with empty private stderr.
Its observed cumulative dependency/import completion times were
`5299.591325` ms and `5777.045059` ms.
These are this diagnostic's observations,
not a timing comparison or proof of the original stage/cause.

The clean catalog is the separately staged graph/import/setup diagnostic.
The failing catalog is the preserved original bounded stop with no fixture directory.
A larger bound is not established as a fix for that original incident.
No SDK API,
permission lifecycle,
producer coverage,
or five-second handback claim follows from the diagnostic success.
Unopened API checks require a separately declared namespace and their own outcome.

### What does not work

- Replaying the consumed original probe would destroy its once-only history.
- Inferring import failure from absent stdout skips dependency checking and setup.
- Calling the later successful diagnostic a reproduced fix would conflate distinct runs.
- Treating copied session headers or IDs as authority would bypass original-source admission.

### Upstream filing decision

No upstream defect or fileable draft is established.
The existing pinned source clone remains read-only.
The failure is at an owned diagnostic boundary,
not an identified SDK or Node implementation path.
Upstream fault,
a supported failing API use case,
a deciding source cause,
contribution/maintainer acceptance,
a compatible tested fix,
and duplicate-tracker applicability remain unestablished for this incident.
No vendor contact,
account change,
or upstream filing was made.

## Scoped Git source-frontier commit rejection

### Symptom

During the pure prefix source handoff,
`proc_7306` failed with exit code 128 at `git add`.
Its diagnostic was:

```text
fatal: Unable to create '<private repository>/.git/index.lock': File exists.
```

The repository path is redacted;
the original diagnostic remains in retained process logs.
`git --version` subsequently reported Git `2.55.0` with the configured `cli-git` wrapper.
This is separate from earlier renderer receipt lock incidents.
The source smoke `proc_29bd` passed;
a rejected documentation commit is not a failed source test.

### Owned dispatch and cause limit

The assistant dispatched receipt commit `proc_28ae` and frontier commit `proc_7306`
concurrently against the same private repository.
The receipt writer completed successfully.
The rejected command identifies the index-lock path,
not its actual owner or whether it was stale.
No lock owner,
upstream defect,
or source-level Git cause is established.
Future owned writers are serialized rather than treating concurrency as harmless.

### Verification and recovery

Scoped `git status --short` showed the frontier `README.md` modified and the source-check `README.md` untracked.
Scoped `git log` had no commit for the new source-check document.
A subsequent `git rev-parse --verify HEAD` returned `73f68daf332837a08b21b2f35d0a46adb4ab7f88`;
`test ! -e .git/index.lock` passed.
After observing the receipt writer's completion,
`proc_9aca` committed the already existing documents and incident note with explicit scoped paths.
No lock was removed and no source smoke or renderer was replayed.
The failed catalog is `proc_7306`;
the clean catalog is the inspected existing-document recovery `proc_9aca`.
Recovery establishes this scoped retention,
not general Git process-tree or lock-owner guarantees.

### What does not work

Blindly repeating a commit can duplicate an ambiguous success.
Removing an unassigned lock can interfere with another writer.
Rerunning a consumed test or renderer cannot repair a rejected Git operation.
Scoped document retention is not global worktree cleanliness.

### Upstream filing decision

Nothing to add or file:
upstream fault,
a deciding implementation cause,
a supported failing Git use case,
contribution acceptance,
maintainer response,
and a compatible tested upstream patch are unestablished.
No source clone,
account change,
vendor contact,
or upstream mutation is needed for this owned command-serialization correction.

## Owned command-result admission after a root-cwd rejection

### Symptom and source

The persisted source-smoke preflight ran `git diff` from `contract/lifecycle`.
The configured `cli-git` wrapper returned `require-root/not-at-root`,
exit code one,
before Git diff or the chained namespace-absence test ran.
The owned boundary is `package/git-policy/cli/src/rule/require-root.ts:181`:

```typescript
// package/git-policy/cli/src/rule/require-root.ts:181 to 187
if (repoRoot !== effectiveCwd) {
  throw new RequireRootViolationError(
    `cli-git: not at the root of the git repository. `
      + `Repo root is ${repoRoot} but effective cwd is ${effectiveCwd}. `
      + `Tip: cd to ${repoRoot} or pass -C ${repoRoot} before the subcommand.`,
  );
}
```

The orchestration awaited the returned tool object but did not gate on its exit status.
It then falsely described the diff and namespace checks as successful and dispatched the pure source smoke.
This is an owned orchestration failure,
not a broken Git root guard.

### Verification and authoritative correction

The failing catalog is the recorded preflight exit one.
The clean catalog is the subsequent scoped root-cwd `git diff` returning zero.
That later measurement is post-dispatch,
not retroactive evidence that the skipped checks ran.
The pure new-baseline smoke itself passed once as `proc_7dd8`:
all ordered aggregate movements,
four direct domain rejections,
exit zero,
no signal,
stdout 427 bytes,
and stderr zero.
Complete prior body review,
successful predispatch byte hashes,
validated fixed worker projection,
and matching post-run sources support that finite source result.
They do not turn the failed admission procedure into success.
`preflight-git-scope-correction.json` and `result-disposition.json` retain the authoritative scope;
the original false metadata and consumed code remain preserved.

### Prospective correction and rejected approaches

Require each prerequisite command's successful exit status before dependent claims or dispatch.
Tool-response fulfillment is not command success.
For `bash`,
inspect `exit_code` explicitly;
an intended failure needs its own predeclared accepted result.
Use repository-root cwd for scoped Git operations.
Later measurements must retain their actual observation time.
Do not erase false historical metadata,
repeat consumed source tests,
or treat create-new markers as complete historical namespace proof.
The [instruction-tightening proposal](../planning/pi-command-result-admission.md) is unaccepted;
`AGENTS.md` remains untouched.

### Upstream filing decision

Nothing to add or file:
the deciding guard is owned source and rejected the observed non-root cwd.
An upstream failing use case,
upstream root cause,
maintainer/contribution acceptance,
upstream fix necessity,
and compatible upstream prototype are not established.
No clone,
account change,
vendor contact,
or upstream mutation follows from the owned exit-status correction.

## Owned read-only checkpoint rejected native normalization

### Symptom and source

The consumed read-only document checkpoint,
`proc_78cc`,
stopped with Node 26.10.0 exit code 1:

```text
// Owned private contract: contract/sdk/docs/render-combined-source-policy-correction.mjs:22
AssertionError [ERR_ASSERTION]: Read-only checkpoint must not rewrite audit or any documentation
```

Its owned assertion compared the native formatter result with unchanged input:

```javascript
// Owned private contract: contract/sdk/docs/render-combined-source-policy-correction.mjs:22
assert.equal(fixed.source, bytes.toString('utf8'), 'Read-only checkpoint must not rewrite audit or any documentation');
```

The audit HTML was created before the handover comparison failed.
No completed checkpoint receipt existed for that run.
This is a consumer assumption failure,
not evidence of a Node or Sätteri defect.

### Verification and recovery

A separately named renderer,
`render-combined-source-policy-correction-normalized.mjs`,
used the already qualified native-coordinate formatter.
It kept the historical audit read-only,
allowed native formatting of other task documents,
checked source freshness before replacement,
and created new source-path-hashed HTML artifacts.

`proc_95b6` exited successfully with 31 rendered documents and zero native diagnostics.
The retained receipt is
`contract/sdk/docs/combined-source-policy-correction-normalized-result.json`.
It explicitly leaves audit-context compatibility unvalidated.
No genuine witness inputs were read;
`AGENTS.md` and the retiring Markdown linter were unchanged.

### Rejected approaches and upstream filing decision

Do not rerun the consumed renderer,
delete its audit HTML,
weaken the native linter,
or infer a complete pass from its partial artifact.
The new renderer namespace preserves the failed attempt.

No upstream filing:
the failed assertion was owned consumer code.
There is no established upstream defect,
supported upstream change request,
or upstream patch to evaluate.

## Owned fork source-accounting and encoded import boundary

### Symptom and source

Independent draft review found that counters called constructor invocations actually counted owner-delegate entries.
It also found a replacement-string boundary after URL/JSON encoding and a masked extra-system test.
These were owned prototype defects,
corrected before the new source smoke was consumed.
They do not establish an SDK defect or previously executed allocation failure.

In the private qualification repository,
`contract/lifecycle/fork-sensitivity-controls/fork-owner-wrapper.mjs:50` records a boundary request before admission.
Its delegate counter at line 56 records entering the owned factory,
not observing an SDK-internal constructor:

```javascript
// Private contract repository: contract/lifecycle/fork-sensitivity-controls/fork-owner-wrapper.mjs:50 to 57
childRequests += 1;
if (!configured || returnedRoot === undefined) throw new SyntheticForkFixtureError('Declared synthetic origin construction is unavailable');
if (originManager !== returnedRoot) throw new SyntheticForkFixtureError('Declared synthetic fork source is not the returned origin manager');
assertBaselineOwnedManager(originManager);
if (childReserved) throw new SyntheticForkBudgetError();
childReserved = true;
childOwnerDelegateCalls += 1;
const manager = forkBaselineOwnedManager(originManager);
```

The controlled fault follows the owned returned child,
not an SDK-internal allocation-then-throw event.
The recovered child cannot be relabeled as successful ledger publication.
The import fix is at `contract/lifecycle/fork-sensitivity-controls/encoded-owner-import.mjs:7`:

```javascript
// Private contract repository: contract/lifecycle/fork-sensitivity-controls/encoded-owner-import.mjs:7
return source.replace(selector, () => JSON.stringify(pathToFileURL(ownerPath).href));
```

The callback preserves encoded path text as data rather than replacement directives.
The corrected extra-system reference test supplies a valid frozen return and checks capture was never called;
a malformed return can no longer mask that rejection.

### Verification and limits

One separately declared source-only `proc_d55f` passed on the pinned Node v26.10.0 runtime:
exit zero,
stdout 494 bytes,
stderr zero,
and no signal.
Its working catalog includes eight parsed ledger variants,
four dedicated mock wrapper copies,
six fork classifier rows,
two recapture rows,
and thirteen source/JSON import-literal cases.
The unsafe string-replacement positive control differs for a dollar replacement token.
The rejection catalog includes unknown errors,
sink failure,
pair-inappropriate domain errors,
invalid ledger permission metadata,
malformed recapture output,
and extra system entries.

The actual SDK owner is only read/pinned;
all twelve loaded wrapper modules are mock-bound.
Ledger guards were not behaviorally exercised.
The new decoder originals are synthetic,
not a configured-host human witness.
The real body-identity gate compared complete reviewed tool-read sources against dispatch bytes.
Successful root-cwd Git and checked creation boundaries were explicitly admitted before dispatch.
The consumed task was `mise --no-env --no-hooks run check` from `fork-source-check/`;
do not rerun it.
SDK imports,
genuine originals,
models,
grants,
and represented actions are zero.

### Rejected interpretations and next verification

Do not equate delegate entries with constructor or complete internal-allocation counts,
return events with unique identities,
fault recovery with published inheritance,
or source literal round trips with filesystem confinement.
Mock rows and imported ledger variants do not establish actual SDK guard sensitivity.
Helper recovery and documented invalid configuration/delegate/return branches remain unexercised.
The shared actual SDK closure needs a new review,
freeze,
and bounded phase with own callback/result/persistence/stop/idle observations.
The worker old-space,
post-exit stream-size,
and timeout controls do not establish total memory,
live-write caps,
or parent-stall resistance.

### Upstream filing decision

Nothing to add or file.
The deciding counters,
interpolation,
and reference-test isolation are owned code.
An upstream defect,
upstream fix necessity,
supported upstream failing use case,
contribution acceptance,
maintainer response,
and compatible upstream patch are not established.
No external source edit,
account change,
vendor contact,
or upstream mutation follows from this source-only correction.

## Owned Mise 2026.9.12 descriptor contrast assumed non-inheritance

### Symptom

The normal-task negative `proc_fddd` exited one.
Node 26.10.0 emitted `AssertionError [ERR_ASSERTION]` at the owned
`sdk-startup/fd-pair-control/check.mjs:20:8`:

```text
// Private contract: combined-sdk-phase/sdk-startup/fd-pair-control/check.mjs:20
Expected values to be strictly equal:
true !== false
```

Mise also printed `[normal] ERROR task failed`.
The control expected both descriptors not to match their private files merely because the task lacked `raw = true`.
The stdout comparison was true.
The stderr assertion was not reached,
and no accepted negative projection was produced.

### Root cause and source trace

The failed assumption was owned:
absence of `raw = true` does not imply piped output.
The installed `mise --version` reported `2026.9.12 linux-x64 (2026-09-20)`.
The deciding published `v2026.9.12` sources were inspected read-only.

`src/cli/run.rs:1284` obtains the dependency graph's linearity:

```rust
//! Source excerpt from Mise v2026.9.12.
// Mise v2026.9.12, src/cli/run.rs:1284
self.is_linear = tasks.is_linear();
```

`src/task/task_output_handler.rs:554` permits interleaved output for a linear graph without raw mode:

```rust
//! Source excerpt from Mise v2026.9.12.
// Mise v2026.9.12, src/task/task_output_handler.rs:554 to 558
if self.raw(task) || self.jobs() == 1 || self.is_linear {
    TaskOutput::Interleave
} else {
    TaskOutput::Prefix
}
```

`src/task/task_executor.rs:1790` permits inherited descriptors when raw mode is false but redactions are empty:

```rust
//! Source excerpt from Mise v2026.9.12.
// Mise v2026.9.12, src/task/task_executor.rs:1790 to 1797
} else if raw || redactions.is_empty() {
    if !task.silent.suppresses_stdout() {
        cmd = cmd.stdout(Stdio::inherit());
    } else {
        cmd = cmd.stdout(Stdio::null());
    }
    if !task.silent.suppresses_stderr() {
        cmd = cmd.stderr(Stdio::inherit());
```

The one-task control defines no dependencies.
Its measured global `raw` setting was false and `jobs` was eight;
`settings get task.output` returned `Setting [task.output] is not set`.
The failed control is consistent with the source's linear-graph inheritance path,
not evidence that Mise ignored an output guarantee.

### Verification catalog

The consumed raw controls `proc_2662` and `proc_f108` exited zero.
The former checked only stdout.
The latter checked both actual descriptors against their named files and returned
`stdoutBound: true` and `stderrBound: true`.
Both retained Mise command echoes privately.

The normal-task negative `proc_fddd` is the failing catalog.
Its frozen expectation and original output remain unchanged.
It is not replayed or relabeled.
These controls contain no SDK imports,
genuine input reads,
models,
grants,
or represented actions.

The fresh `sdk-startup/fd-identity-control/plan.json` declares a different contrast:
matched destinations first,
then distinct existing decoy destinations,
with independent expected pairs `[true, true]` and `[false, false]`.
The fresh positive `proc_2a24` and negative `proc_1506` each exited zero with their exact expected projection.
That independently declared contrast adds prospective evidence;
it does not repair or relabel `proc_fddd`.

### Verified boundary and tradeoffs

The consumed raw positive establishes descriptor identity for its own invocation.
The prospective caller redirects the entire Mise startup into precreated private files.
The builtin startup gate additionally compares its actual descriptors using
`fstatSync()`,
device,
and inode before importing the controller.

This preserves parsing and loader diagnostics privately.
It does not establish total memory,
live disk-write bounds,
startup timing,
or descriptor identity for an invocation that has not run.
A false decoy comparison alone does not identify the actual destination or prove it is regular.

### What does not work

- Treating `raw = false` as the complement of descriptor inheritance.
- Checking named file modes without comparing inherited descriptor identities.
- Counting an assertion failure as an accepted negative result.
- Replaying a consumed control or replacing its frozen expectation.
- Running the startup gate as a harmless preflight:
  it imports the controller and launches the actual phase.

### Upstream filing decision

Nothing to add or file.

- Upstream fault:
  not established;
  the failed expectation was owned and published source permits the observed behavior.
- Upstream fix:
  unnecessary for this caller-owned admission boundary.
- Supported failing use case:
  no violated output guarantee was established.
- Contribution acceptance:
  not investigated because no upstream change is proposed.
- Maintainer willingness:
  not investigated because no upstream defect is claimed.
- Compatible upstream patch:
  none;
  the separately declared correction belongs in the owned control.

No upstream source edit,
issue,
comment,
or vendor contact follows.

## Owned GNU install 9.10 argument grouping stopped preparation

### Symptom and evidence boundary

The prospective SDK-ID caller created an empty `0700` preparation directory,
then rejected its destination-file command before source-data admission or SDK startup.
The original command's stderr was not retained in the model-visible transcript.
Do not substitute a later control's diagnostic for that historical receipt.

The owned command passed `/dev/null`,
`stdout`,
and `stderr` as operands to one `install` invocation.
Installed GNU coreutils 9.10 documents `SOURCE DEST` and `SOURCE... DIRECTORY` in `install --help`.
That operand grouping is not two destinations.

### Verification and correction

Fresh disposable controls ran independently of the consumed namespace.
Two separate `SOURCE DEST` invocations exited zero and produced distinct empty `0600` regular files.
The failing control supplied an absent final destination:
it exited one with `install: target 'missing-destination': No such file or directory`.

```sh
# Fresh disposable GNU install grammar fixture, not an existing consumed namespace.
fixture=$(mktemp --directory)
install --mode=600 /dev/null "$fixture/stdout"
install --mode=600 /dev/null "$fixture/stderr"
stat --format='%a:%F' -- "$fixture" "$fixture/stdout" "$fixture/stderr"
install --mode=600 /dev/null "$fixture/missing-second-source" "$fixture/missing-destination"
```

Individual destination commands preserve stdout/stderr separation.
Their success is not inherited descriptor identity:
the invoked Node gate must still compare actual descriptors to the named files.
The old directory remains untouched;
only separately named future preparation namespaces may be created.

### Upstream filing decision

Nothing to file or draft.

- Upstream fault:
  none established;
  the caller contradicted documented operand grammar.
- Fixability:
  the fix belongs in the owned caller.
- Supported use case:
  the documented individual source/destination form passed.
- Contribution acceptance:
  not investigated because no upstream change is proposed.
- Maintainer willingness:
  not investigated because no upstream defect is claimed.
- Compatible upstream patch:
  unnecessary;
  no public issue or vendor contact follows.

## Installed pi-processes 0.12.0 write receipts are not consumer EOF evidence

### Symptom and deciding source

An owned orchestration asserted `EOFDelivered: true` after awaiting the write tool.
That assertion was withdrawn before the synthetic dependency run.
A tool receipt is not evidence that the child consumed all bytes or observed EOF.

Read-only source clone `pi-processes-write-source.nHfffTGG` is pinned at
`510e9bc6e5d01b42e8175c2b664295fe1645fec0`.
Its decisive files match installed source:
`extensions/processes/tools/write/index.ts` SHA-256
`cf1c0bf2f6b4158657521eae08ce578d40f03bed476e61fe1f2501a17d2a1d78`;
`src/manager/process-runtime-controller.ts` SHA-256
`ad66b76a9920c5a148ea5839eaa3b648fd5cc0c5a2fd473974f5c9979a5904aa`.

`src/manager/process-runtime-controller.ts:361` calls the stream synchronously:

```typescript
// pi-processes/src/manager/process-runtime-controller.ts
managed.stdin.write(data);
if (opts?.end) {
  managed.stdin.end();
  managed.stdinClosed = true;
}
return { ok: true };
```

`extensions/processes/tools/write/index.ts:49` reports the encoded input length,
not independently measured child consumption:

```typescript
// pi-processes/extensions/processes/tools/write/index.ts
bytes: Buffer.byteLength(input, "utf-8"),
end,
ok: true,
```

`extensions/processes/tools/write/index.ts:74` formats successful output as
`Wrote <bytes> bytes to "<name>" (<id>) and closed stdin.`
The failed variant at line 70 begins `Failed to write to stdin for`.
Neither formatter waits for the child's input parser.

### Verification and consumer boundary

Historical positive `proc_53ff` exited zero and saved a byte-identical 17,974-byte original record.
Historical negative `proc_1e6f` received empty input and exited one with
`SyntaxError: Unexpected end of JSON input`.
Their consumed namespaces remain preserved;
neither is replayed.

The new dependency caller compares and retains the installed tool's exact receipt.
It reports only queued-input length and end invocation.
Consumer admission still requires the child's complete saved original,
literal body equality,
applicable complete output checks,
and terminal zero.
The version-bound receipt comparison deliberately stops on changed or ambiguous formatting.
At that source-review epoch,
these caller checks were not a completed-runtime claim.
Later consumed `proc_f674` exited zero after retaining its exact 126,984-byte queued-input receipt.
The child's saved original matched the archived v4 strings byte for byte;
complete output/native outcome checks accepted 31 synthetic outcomes.
This establishes that finite consumer boundary,
not a stronger promise from the write receipt itself.
No SDK,
genuine original,
current permission,
grant,
or represented action was established.

Ignoring the receipt or treating fulfilled tool delivery as successful EOF does not establish that boundary.

### Upstream filing decision

Nothing to file or draft:
this was an owned overclaim,
not a demonstrated upstream defect.

- Upstream fault:
  not established.
- Fixability:
  correct the consumer assertion and admission checks.
- Supported use case:
  stdin writing is documented;
  child-consumption attestation was not established.
- Contribution acceptance:
  not investigated because no upstream change is proposed.
- Maintainer willingness:
  not investigated because no upstream defect is claimed.
- Compatible upstream patch:
  unnecessary for the owned correction.

## Owned empty stdin stopped Node 26.10.0 source-data admission

### Symptom and root cause

Consumed `proc_1e6f` exited one.
Node's `JSON.parse` emitted `SyntaxError: Unexpected end of JSON input` at
`contract/lifecycle/prospective-sdk-id-controls/prepare-reviewed-source-v2.mjs:45:34`
in the private prototype repository.
The caller sent zero stdin bytes and EOF instead of its retained original source record.
No SDK startup or manager construction occurred.

The data helper correctly rejected the missing input.
Changing its parser,
accepting empty input,
or reopening the consumed process would undermine admission rather than fix delivery.

### Verification and corrected boundary

Fresh `proc_53ff` used a separately named v3 helper and namespace.
One orchestration loaded the retained records,
verified their nonempty UTF-8 size,
started the managed process,
and sent all 17,974 bytes plus EOF to that returned process ID.
It exited zero with exact stdout/result projections and a byte-identical `0600` saved record.
Actual private stdout/stderr identity checks ran before record creation.
The original v2 helper,
diagnostics,
and failed namespace remain preserved.

The correction qualifies only this source-data transaction:
zero SDK imports,
zero helper imports,
and no human authority.
It does not qualify arbitrary stdin producers,
general process cleanup,
or an SDK invocation that has not run.

### Upstream filing decision

Nothing to file or draft.

- Upstream fault:
  no;
  the owned caller sent no document.
- Fixability:
  the correction belongs in orchestration.
- Supported use case:
  nonempty JSON passed in the fresh source-data transaction.
- Contribution acceptance:
  not investigated because no upstream change is proposed.
- Maintainer willingness:
  not investigated because no upstream defect is claimed.
- Compatible upstream patch:
  unnecessary;
  the parser's rejection remains required.

## Node 26.10.0 cleared-home parser import stopped the Pi 1.0.0 fixture

### Symptom and deciding source

`proc_7252` failed before SDK session construction.
Node's module resolver emitted `MODULE_NOT_FOUND`:
`Cannot find module 'yuku-parser'`.
The require base was inside the disposable fixture's
`controls-private/home/Monochromatic/package/git-policy/cli/package.json`,
which was not a repository dependency tree.

The owned loader selected its package base from `homedir()`:
private `contract/research/composed-literal-facts/load-parser.mjs:7-9`.

```javascript
// Private contract/research/composed-literal-facts/load-parser.mjs:7-9
const require = createRequire(join(homedir(), 'Monochromatic/package/git-policy/cli/package.json'));
export const parserPath = require.resolve('yuku-parser');
export const { parse } = await import(pathToFileURL(parserPath).href);
```

The new child-judgment constructor imported the source profile,
which reached that loader in a worker whose `HOME` had already been cleared.
This is an owned dependency-resolution assumption,
not an upstream SDK inability or a missing parser installation.

### Corrected source boundary and verification

The fresh v3 generator binds the parser entry before launching the cleared-home worker.
Private `contract/collector/native-program-sdk-copy-v3/stage.mjs:20-28`
emits an explicit module and relocates the profile,
collector,
and constructor imports.

```javascript
// Private contract/collector/native-program-sdk-copy-v3/stage.mjs:21, formatted across lines
save('parser',
  'export {parse} from ' + JSON.stringify(url(parserPath))
  + ';\nexport const parserPath=' + JSON.stringify(parserPath) + ';\n');
```

`proc_de32` staged 21 artifacts with no SDK imports.
`proc_5927` then passed two actual SDK sessions,
four injected requests,
and one canned original assessment per session.
The valid native literal program entered both children;
changed final child inputs entered neither.
The worker kept its cleared home,
private descriptors,
source checks,
and network tripwire.
No whole-home symlink or installed-source edit was used.

The consumed verification command was `mise --no-env --no-hooks run check`
in private `contract/collector/native-program-sdk-controls-v2/`.
Its retained manifest and result are the reproducibility record;
do not replay the consumed namespace unchanged.

### Separate generator failure and rejected approaches

`proc_d0d3` separately failed while parsing the owned v2 generator.
Node emitted `SyntaxError: Invalid regular expression`
with `Unterminated group` at `stage.mjs:19`.
An extra escaping layer corrupted the regex literal;
the generated parser module's newline spelling also needed correction.
No SDK import occurred.
Fresh v3 source corrected those syntax boundaries;
both failed namespaces remain unchanged.

Passing standalone parser tests under the real home was not evidence for a cleared-home SDK worker.
Changing `HOME` back,
exposing the whole repository through a home symlink,
or editing consumed evidence was not the adopted remedy.
Explicit source paths still depend on the pinned local dependency layout;
the result is a private host qualification,
not a portable production deployment.

### Upstream filing decision

Nothing is filed or drafted upstream.

- Fault:
  the owned loader and generator supplied the failing assumptions.
- Fixability:
  explicit staging passed the actual SDK consumer.
- Supported use:
  installed parser imports and the SDK's existing tool pipeline were exercised.
- Contribution policy:
  no upstream change is proposed.
- Maintainer disposition:
  no rejection or intent is inferred.
- Prototype:
  the correction lives in the owned staging boundary;
  no upstream patch is justified.

## Owned admission verifier confused extension callbacks with persisted results

### Symptom and source

The private Pi SDK 1.0.2 clause-reuse omission worker exited with status zero.
The separate verifier in `proc_60d2` failed Node's `assert.deepStrictEqual`:
actual `[]`,
expected `[true, true]`.
This was an owned verifier error,
not a new SDK failure or an absent tool outcome.

Paths in this section are relative to the private consumer-contract repository.
`contract/collector/instruction-meaning-reuse-omission/verify.mjs` checked the extension callback trace:

```js
// contract/collector/instruction-meaning-reuse-omission/verify.mjs
assert.deepEqual(record.trace.map(value=>value.isError),[true,true]);
```

The private dispatcher catches assessment failure before native execution.
In `contract/collector/instruction-meaning-sdk-copy/stage-private/prepared-dispatch.mjs:105`,
the deciding branch is:

```js
// contract/collector/instruction-meaning-sdk-copy/stage-private/prepared-dispatch.mjs
}catch(error){failures.push(error);block('Complete group preparation or assessment failed');return {block:true,reason:'Complete group preparation or assessment failed'};}
```

The staged agent loop separately creates and emits persisted tool-result messages
at `contract/collector/instruction-meaning-sdk-copy/stage-private/agent-loop.mjs:481`:

```js
// contract/collector/instruction-meaning-sdk-copy/stage-private/agent-loop.mjs
const message = createToolResultMessage(finalized);
await emitToolResultMessage(message, emit);
```

The callback trace was therefore the wrong evidence surface for this admission failure.
Do not infer absent outcomes from that empty trace.

### Verification and remedy

The intact `proc_0eb5` case executed both native reads and observed successful extension callbacks.
The omission replaced shared `instructionMeaningPairs` reuse with a fresh `WeakMap`.
It retained the exact intended assertion failure:
`AssertionError`,
code `ERR_ASSERTION`,
operator `throws`,
message `Missing expected exception.`
Neither native read executed.

The recovery task is `mise --no-env --no-hooks run check`
from `contract/collector/instruction-meaning-reuse-reconciliation/`.
It reads the retained worker exit,
source hashes,
assertion witness,
streams,
outcome,
and session JSONL.
It checks native result identities and error messages rather than manufacturing extension callbacks.
Read-only reconciliation `proc_cc9f` passed,
including both persisted error results and the exact omission witness.
It added no SDK sessions or provider calls.
This is not broader event qualification.
The failed verifier and original namespace remain unchanged.

Replaying the SDK worker to repair the verifier,
treating every exception as an omission witness,
or weakening native admission would not correct this evidence-boundary error.

### Upstream filing decision

Nothing is filed or drafted upstream.

- Fault:
  the owned verifier chose the wrong event surface.
- Fixability:
  the correction belongs in the owned result verifier.
- Supported use:
  this private staged admission profile does not establish a new upstream event contract.
- Contribution policy:
  no upstream contribution is proposed.
- Maintainer disposition:
  no upstream response or intent is inferred.
- Prototype:
  retained-artifact reconciliation tests the local remedy;
  no upstream patch is justified by this incident.

## Pi 1.2.0 preparation and retired fixture dependency paths

### Symptom

The resumed native acceptance on 2026-10-13 stopped before policy review.
Node emitted `ERR_MODULE_NOT_FOUND` for the historical `yuku-parser@0.17.0` path.
The installed parser was 0.19.0 and Pi was 1.2.0.
The previous unbundled coding-agent JavaScript paths were also absent from the installed package.

After separate source emission,
the owned prompt transform emitted Node `AssertionError [ERR_ASSERTION]`
for its obsolete `emitBeforeAgentStart(expandedText, currentImages, ...)` anchor.
Subsequent fixture failures identified a missing wildcard export,
missing generated `amazon-bedrock.json`,
and `MODULE_NOT_FOUND` for `satteri` through the retired `package/cli/markdown-lint` owner.
These were distinct preparation failures,
not evidence that the policy consumer had run.

### Source trace

The read-only Pi checkout is tag `v1.2.0`,
commit `06a466935536f8c9bcacb2d5efc0fc021be1bdff`.
Its `packages/coding-agent/src/core/agent-session.ts:1885` prepares a run from a trigger,
and line 1894 passes that trigger to the extension runner:

```typescript
// packages/coding-agent/src/core/agent-session.ts, Pi v1.2.0
private async _prepareRun(trigger: BeforeAgentStartTrigger): Promise<AgentMessage[]> {
// Selected call within this method:
const result = await this._extensionRunner.emitBeforeAgentStart(trigger, this._baseSystemPromptOptions);
```

The private transform now retains that trigger,
including its custom-message branch,
and closes its failure handling at `_prepareRun`'s return.
The existing run owner still observes the original handler traversal and message construction.

Pi's `packages/ai/package.json` declares wildcard exports such as `./utils/*`
and separately copies generated model catalogs during its build:

```json
// packages/ai/package.json, selected fields from Pi v1.2.0
"./utils/*": { "types": "./dist/utils/*.d.ts", "import": "./dist/utils/*.js" }
```

The owned source emitter copies the matching installed release's `dist/providers/data` assets.
The loader delegates export selection to Node from each staged package's own package boundary.
It does not interpret only the exact export keys.

Repository commit `5bc05682ec` removed `package/cli/markdown-lint`.
The private `contract/native-index/native-parser.mjs:19` previously resolved from that retired owner:

```javascript
// Historical private contract/native-index/native-parser.mjs
const ownerRequire = createRequire(join(repositoryRoot, 'package/cli/markdown-lint/package.json'));
const entry = ownerRequire.resolve('satteri');
```

Private `f411f9da` now resolves the declared fixture dependency from `import.meta.url`.
The qualified version remains 0.10.5;
the version and astral-coordinate checks remain in force.
Private `ba4ecadd` records the generated lockfile.

### Verification and workaround

From private `contract/integration/current-sdk`,
`mise run stage` emits the current Agent,
AI,
and coding-agent sources without editing installed packages or the upstream checkout.
`mise run native:prepare` passes and reports 13 instrumented modules.
`mise run lint:types` passes for the staging helper.

The private root's `mise run dependencies:install` restored `satteri@0.10.5`
with `pnpm install --ignore-scripts`.
It changed only that fixture workspace's manifest,
catalog,
lockfile,
and installed dependency tree.
The native acceptance subsequently passed the parser's original version and coordinate checks
and reached the actual policy reviewer.
It retained the unresolved domain hold;
preparation success is not domain closure or production qualification.

Working cases are current native preparation,
wildcard package imports,
generated model-catalog loading,
and the restored parser's original coordinate self-check.
Rejected cases are the historical absolute dependency paths,
the exact-key-only export mapper,
source emission without generated catalogs,
and resolution through a removed linter package.
The workaround remains a private current-version test build,
with maintained host delivery still required.

### Current tool reachability metadata

The first reviewer result also reported `tool-metadata-observation-gap`
for field `reachesTools`.
Pi's `packages/coding-agent/src/core/extensions/types.ts:621` declares it as an executable capability:

```typescript
// packages/coding-agent/src/core/extensions/types.ts:621, Pi v1.2.0
reachesTools?: (input: Record<string, unknown>) => "all" | readonly string[];
```

Private `2fe40907` added a control whose callback throws if receipt capture invokes it.
The control first failed because the collector reported unsupported metadata gaps.
Private `27d713a7` classifies this declared callback alongside the existing executable hooks.
`mise run test:tool-metadata` then passed,
preserving data-field capture,
unread accessor gaps,
cycle rejection,
and shared-value accounting.
This recognizes a callback field;
it does not account for material returned by invoking callbacks or close the host domain.

### Isolated codemode worker dependency root

After the owned domain consumer first returned approval,
Pi's codemode result reported `Script sandbox failed` with `ENOENT` for
`fixture-private-.../home/Monochromatic`.
The native result also reported `No tool calls were made.`
The fixture deliberately changes `HOME` to a disposable directory.
The inherited test loader then derived its dependency root from that changed home in the worker.

Pi `packages/codemode/src/runtime/host.ts:34` creates the worker:

```typescript
// packages/codemode/src/runtime/host.ts:34, Pi v1.2.0
worker = new Worker(workerUrl, { workerData });
```

The owned `contract/integration/current-sdk/paths.mjs` had recomputed the dependency root:

```javascript
// Historical private contract/integration/current-sdk/paths.mjs
const repoRoot = join(homedir(), 'Monochromatic');
```

Private `f3d40d82` records the original measured dependency root in the staging manifest
and reads it when the loader runs in a worker.
It preserves the disposable home and native worker execution.
After rebuilding through `mise run stage`,
`mise run test:domain-resolution` passed permission with three native child calls,
prohibition with zero calls,
and unspecified policy with zero calls.
The failing case had reached approval but never entered a child call;
the corrected case exercised the actual native sandbox.
This is an owned fixture-loader correction,
not an upstream sandbox defect.

### Help loads the trust store before printing

Pi 1.2.0 `pi --help` failed in the read-only agent sandbox with
`EROFS: read-only file system` for `$HOME/.pi/agent/trust.json.lock`.
The same command outside that filesystem restriction printed help and exited zero.
The native session RPC probe also succeeded outside that restriction.

`packages/coding-agent/src/main.ts:719` constructs `ProjectTrustStore`.
At line 749,
runtime setup can read project trust before the help branch at line 877:

```typescript
// packages/coding-agent/src/main.ts:749, Pi v1.2.0
(!hasTrustRequiringResources || trustStore.get(cwd) === true)
```

`packages/coding-agent/src/core/trust-manager.ts:221` reads entries under the trust lock:

```typescript
// packages/coding-agent/src/core/trust-manager.ts:221, Pi v1.2.0
getEntry(cwd: string): ProjectTrustStoreEntry | null {
  return withTrustFileLock(this.trustPath, () => {
    const data = readTrustFile(this.trustPath);
    return findNearestTrustEntry(data, cwd);
  });
}
```

The lock implementation at line 147 uses a lock directory beside the trust file:

```typescript
// packages/coding-agent/src/core/trust-manager.ts:147, Pi v1.2.0
return lockfile.lockSync(trustDir, { realpath: false, lockfilePath: `${path}.lock` });
```

The verification command is `pi --help` from this repository.
The failing case is the existing user trust directory mounted read-only;
the working case is the same command with its normal writable trust directory.
The verified workaround changes the command's filesystem access,
not its trust decision or project configuration.
Its tradeoff is that Pi performs normal startup work before returning help.
The help flag alone does not bypass that startup path.
No upstream fault or universal sandbox failure follows from this local mount restriction.

### Upstream filing decision

Nothing is filed or drafted upstream.

- Fault:
  the failures are stale owned integration assumptions and a retired local dependency owner.
- Fixability:
  the current source and private dependency boundary support the local adaptation.
- Supported use:
  these private source transforms are not established upstream integration contracts.
- Contribution policy:
  no upstream contribution is proposed.
- Maintainer disposition:
  no upstream intent is inferred.
- Prototype:
  the owned staging and receipt controls exercise the local corrections;
  they do not justify an upstream patch.

## Maintained native build after the installed release changes

### Failure and scope

The installed Pi command later reported `1.1.0`,
while the integration's compiled source and dependency links still targeted `1.2.0`.
Node also reported `ERR_MODULE_NOT_FOUND` for an earlier parser dependency path.
These failures concern owned staging paths;
the earlier Pi 1.2.0 results retain their original version boundary.

The maintained build is `package/pi-plugin/auto-mode/native/build.ts`.
Its source emitter in `native/build-source.ts` verifies Pi 1.1.0 and source commit
`abe508e1b89912adde45528136c3221eb69acdd7`
before compiling the upstream Agent,
AI,
and coding-agent module trees.
It copies generated model catalogs from the matching installed AI package.
Installed Pi sources are never patched.

### Verified correction

`mise run //package/pi-plugin/auto-mode:build:native` emits the upstream modules,
copies maintained owned inputs,
and applies the native transformations inside the package's `dist/native` directory.
The optional `PI_AUTO_MODE_SOURCE` points at a source checkout only during this build;
the emitted host does not read that checkout.
The build rejects a mismatched source revision or installed Pi release.
`build:native:owned` rebuilds owned transformations against the existing emitted upstream source.

The first maintained build,
package type check,
and permission/prohibition/unspecified fixtures passed.
The disposable fixture was then moved outside the repository after its original location inherited
the repository's ancestor `AGENTS.md`.
The normal SDK provider case now passes with 33 captured sources and five accounted contribution uses.
It invokes Pi's SDK and model runtime,
while a local fixture supplies the transport response and no external provider is called.

### Source normalization and lifecycle boundaries

Pi's `resolveTranscript` creates another message array through `collapseSystemMessages`.
Comparing that array with the original request input failed the SDK contribution check.
The maintained serializer now retains the input occurrence before normalization,
with a private payload receipt tying it to the actual native serializer call.
This preserves normal SDK authentication and provider dispatch.

The CLI also reuses an in-memory manager when replacing a forked session.
The original observer attachment outlived its disposed session,
producing `Original manager already has a context-copy owner`.
The maintained source transformation releases that attachment only after disposal completes.
Native tests cover reuse,
foreign-manager rejection,
retired leases,
incomplete disposal,
and repeated completed disposal.
These are owned integration corrections,
not evidence of upstream Pi defects.

The candidate CLI bootstrap now exists in the maintained native graph.
Its first relocated import emitted Node's `ERR_MODULE_NOT_FOUND` for `chalk`.
The original `main.js` imports `chalk` from its package dependency root;
the relocated `host-main.mjs` had no corresponding dependency location.
The host source transformation now resolves that import from the original emitted Pi package.
The next import exposed a separate owned staging omission:
Node reported that `./core/sdk.js` did not export `AgentSessionRuntime`.
The earlier isolated SDK transform had removed its runtime re-export.
The complete host graph retains that native export and routes it through the owned runtime module.
Neither failure changes installed Pi source or demonstrates an upstream defect.

`mise run //package/pi-plugin/auto-mode:test:native:host-lifecycle` passed actual CLI manager selection,
service creation,
resource reload,
cancelled replacement,
new session,
in-memory fork,
persisted resume,
and completed cleanup retry.
Its initial resource reload calls the original trust resolver through the policy owner.
The fixture makes no external provider request.
The separate-process `test:native:cli-process` check also passes:
the candidate returns the expected Qwen RPC state and exits without stderr output.
It uses disposable state,
synthetic credentials,
offline mode,
and a rejecting fetch implementation.
No model prompt is sent.

The native built-in Codemode extension exposed another ownership loss at registration.
The upstream extension spreads its tool definition to add `defaultActive: false`.
The maintained program factory authenticates its original returned definition by identity,
so that extra object lost the owned-program identity.
The owned factory now returns the inactive definition directly,
and the host transformation registers that same object.
The committed rejection control failed before this change and passes afterward.

The first package type checks excluded the new native helpers through the inherited include list.
They therefore did not verify those files.
The package now includes `native/**/*.ts` explicitly.
An intentional invalid native assignment failed with `TS2322` in a disposable worktree;
removing it restored a passing check with the same source scope.
This corrects owned verification configuration,
not TypeScript behavior.

Complete package lint,
production semantic admission,
and genuine human approval remain separate delivery checks.
No upstream issue or patch is warranted by the verified local staging corrections.

## Pi 1.1.0 native approval contexts change on resource reload

### Symptoms and source findings

The native context fixture first stopped with owned `PolicyApprovalError`:
`Native approval host requires an ordinary bootstrap configuration`.
The original evaluator captured a presentation object once during runtime construction.
The candidate host needs a fixed bootstrap function that captures current native presentation capabilities
for each original review.
That function receives no judgment,
request,
authority,
or semantic evidence.

Pi v1.1.0,
commit `abe508e1b89912adde45528136c3221eb69acdd7`,
creates extension contexts in `packages/coding-agent/src/core/extensions/runner.ts:877`.
Their UI getter retains the original runner and checks it before returning its UI:

```typescript
// packages/coding-agent/src/core/extensions/runner.ts:878, Pi v1.1.0
const runner = this;
// The returned context contains this getter.
get ui() {
  runner.assertActive();
  return runner.uiContext;
}
```

Native reload at `packages/coding-agent/src/core/agent-session.ts:3661` invalidates that runner:

```typescript
// packages/coding-agent/src/core/agent-session.ts:3662, Pi v1.1.0
const oldRunner = this._extensionRunner;
const previousFlagValues = oldRunner.getFlagValues();
await emitSessionShutdownEvent(oldRunner, { type: "session_shutdown", reason: "reload" });
oldRunner.invalidate();
```

The replacement comes from the `new ExtensionRunner(...)` assignment in the same file at line 3637.
Its native stale-context diagnostic begins
`This extension ctx is stale after session replacement or reload.`
The first test assertion expected different wording and was corrected to match this actual diagnostic.

A separate direct-SDK fixture omission then prevented all provider and assessment calls.
The original assistant error was `Original resource input frame changed`.
The maintained guard in
`package/pi-plugin/auto-mode/native/contract/integration/policy-source/system-slot.mjs:102`
compares the currently owned resource frame with the retained policy binding:

```javascript
// package/pi-plugin/auto-mode/native/contract/integration/policy-source/system-slot.mjs:102
if(readBoundResourceFrame({binding:nativeBinding,loader})!==binding.frame)
  throw new PolicySlotBindingError('Original resource input frame changed');
```

The direct-SDK fixture had not registered its original policy input with the maintained host reload route.
It now registers that original pair before native reload.
The CLI host's private resource membership and session-construction ownership remain independent.

### Correction and verification

The maintained evaluator captures the bootstrap capability source once,
then captures its current Pi API and extension context once per original review.
The existing approval binder still retains the actual selected capabilities and rejects stale evidence.
The candidate host supplies Pi's current extension context and its original resource-loader API.
This adds presentation routing,
not semantic admission or authorization facts.

`mise run //package/pi-plugin/auto-mode:test:native:approval-context`
checks native Allow,
Deny,
Stop,
cancellation during selection,
and reload before selection.
The selections are fixture responses;
they are not genuine human approvals.
All these cases pass through the native context.
Cancellation executes no prepared child.
Approval after reload records the original native verdict and executes all three prepared children.
The fixture also verifies that reload replaces the actual Pi API,
the old context rejects access,
and each review captures its current capabilities once.

### Rejected remedies and upstream filing decision

Retaining the old native context after reload is rejected by Pi's own active-runner check.
Bypassing the original resource-frame comparison would discard required policy-currentness evidence.
The remedy connects the fixture to the existing host reload route.

An eager bootstrap read of `maintainedPolicyEntry` also failed with Node's
`ReferenceError: Cannot access 'maintainedPolicyEntry' before initialization`.
The maintained artifact and the SDK host participate in an import cycle.
Live named re-exports defer use of `askUser` and `VERDICT_ENTRY_TYPE` until the session is bound.
The normal native approval cases pass after this correction.

There is nothing to file upstream:

- Fault belongs to owned bootstrap and fixture assumptions.
- The consumer can capture the current native context through Pi's existing API.
- No upstream support change is needed for these controls.
- No upstream contribution is proposed or contribution policy claimed.
- No maintainer disposition is inferred.
- The maintained bridge and native fixture form the local correction;
  an upstream patch prototype is unnecessary.

## Native lifecycle logger timeout on 2026-10-14

The maintained Pi 1.1.0 `test:native:host-lifecycle` command passed its session assertions
but emitted this `@monochromatic-dev/logger-core` diagnostic:

```text
logger internal error: sink verification failed for entry 3: Timed out after 5000ms: sink 3 verify
```

The original invocation did not capture the emitting process or pending verification operation.
The [logger source trace](cli-git-logger-sink-verify-timeout.md#emitter)
identifies the reporting path;
matching diagnostic text does not connect this incident to the earlier Git or synchronous-launcher incidents.
Its cause remains unknown.

### Verification and limits

The local probe is `${HOME}/temp/agent/pi-native-logger-probe-20261014.ts`,
with preload `${HOME}/temp/agent/pi-native-logger-trace-20261014.mjs`.
It preserves warnings and records process identity,
pending file operations,
their call stacks,
and event-loop tick gaps.

The positive control uses a disposable directory with its own `node_modules`
and delays only the file sink's `mkdir` by 5100 ms.
The detector captured the exact timeout and pending `mkdir`.
The subsequent unchanged native lifecycle command passed all assertions without that diagnostic.
The raw records are `pi-native-logger-control-20261014.json`
and `pi-native-lifecycle-logger-20261014.json` in the same private scratch directory.
They contain local paths and need inspection before external sharing.

No workaround is established.
The successful traced run does not demonstrate a fix,
and the injected delay proves detection rather than the original cause.
Suppressing stderr,
raising the sink deadline,
or attributing the delay to load without an emitting-process trace would not resolve the incident.

### Upstream filing decision

- Upstream fault is not established; the emitter belongs to this repository.
- Fixability depends on identifying the delayed operation.
- No unsupported Pi API use is established by this diagnostic.
- No external contribution or contribution-policy claim is proposed.
- No maintainer disposition is inferred.
- The disposable positive control validates instrumentation,
  not a candidate fix.

There is nothing to file upstream from this observation.

## Overlapping native verification and Oxlint 1.87.0 stop before completing checks

### Session-construction port verification on 2026-10-14

Final repository inspection encountered a separate Git LFS bookkeeping failure in the default sandbox:
`Error cleaning Git LFS object`,
`read-only file system`,
and `clean filter 'lfs' failed` for a music-player README image.
The command returned status 128 before printing repository status.
A subsequent escalated `git status --short` restricted to this task's package and documentation paths succeeded
and reported those paths clean.
That result establishes task-path cleanliness only;
it neither diagnoses the unrelated asset nor supplies evidence about the compiler failure.

The subsequent factory-scope port type-check attempt stopped in TypeScript Go with:

```text
# pi-factory-scopes-types-20261014.log
runtime: failed to create new OS thread (have 140 already; errno=11)
runtime: may need to increase max user processes (ulimit -u)
fatal error: newosproc
```

The source formatter was also running.
No immediately preceding task-counter sample attributes the failure to that overlap.
The wrapper returned exit zero despite retaining this panic;
this is not a successful type check.
The separate log is `${HOME}/temp/agent/pi-factory-scopes-types-20261014.log`.
The retry ran serially with `TSC_SINGLE_THREADED=1` and `GOMAXPROCS=2`.
These controls reduce compiler concurrency without changing TypeScript checks or resource ceilings.
The complete log at `${HOME}/temp/agent/pi-factory-scopes-types-final-20261014.log`
contains the successful build-info emission and no fatal compiler diagnostic.
The final scoped lint separately reports zero warnings and zero errors across ten changed files.
The owned rebuild,
factory-scope tests,
actual SDK host lifecycle,
manager constructors,
and isolated Qwen CLI process pass.
This establishes bounded verification recovery,
not a diagnosis of the original thread allocation failure.

The incorrect wrapper exit result has a separate owned source explanation:
`package/monorepo-task/core-exec/src/tsc-filter.ts:203`
sets a failure code only for recognized retained TypeScript errors:

```ts
// package/monorepo-task/core-exec/src/tsc-filter.ts
if (stdoutResult.hasRemainingErrors || stderrResult.hasRemainingErrors)
  process.exitCode = subprocessError.exitCode ?? 1;
```

The Go panic is retained as ordinary output without a recognized TypeScript error header.
The independent signal branch does not cover a compiler that exits with code two.
This wrapper behavior was subsequently repaired and verified separately.
The [owned wrapper incident](typescript.md#task-tsc-002-converted-compiler-process-failures-into-successful-checks)
retains failing-before process fixtures and passing-after evidence.
The successful serial type-check retry alone did not fix it.

After the typed session-construction port,
the approval-context `allow` case failed with
`JudgmentDeadlineError: Judgment preparation and assessment deadline expired`.
The final assertion was `ERR_ASSERTION: 0 !== 1` for original selector reach.
The CLI process check and result-test formatter were also running.
That overlap is an observed condition,
not an established cause.

The unchanged isolated approval sequence then passed Allow,
Deny,
Stop,
cancellation,
and Allow after reload.
Its complete log is `${HOME}/temp/agent/pi-session-root-approval-isolated-20261014.log`.
Every completed mode reports zero provider calls and zero genuine human approvals.
The isolated repeat did not change the five-second deadline or prove a permanent timing remedy.
This is distinct from the earlier SIGABRT and Go thread-creation failures in this incident record.

During the later owned-runtime cleanup,
the domain run and manual uncertainty Allow separately exhausted the same original judgment deadline.
The unchanged manual uncertainty case passed in isolation.
After an unrelated bounded CPU stress process exited,
the domain and manual uncertainty suites passed,
but ordinary native-context Allow still exhausted its deadline.
The stress overlap therefore does not establish the cause or a sufficient remedy.

Three unchanged isolated ordinary Allow runs then passed.
Total process durations were 7259.69,
7205.50,
and 7954.08 milliseconds;
these include setup and shutdown rather than measuring only judgment time.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/native-allow-repeat.json`
retains individual outcomes and enclosing CPU counters.
The subsequent complete remaining consumer sequence passed,
including approval context,
single calls,
approved dependencies,
event receipts,
host lifecycle,
CLI shutdown,
queue release,
resource reload and copies,
and context receipts.
`${HOME}/temp/agent/pi-native-resource-runtime-remaining-20261014.log` retains that sequence.
This is verified recovery without a causal diagnosis or permanent timing fix.

The later profile-cleanup build passed its domain and manual-uncertainty suites,
then ordinary native-context Allow again reached
`JudgmentDeadlineError: Judgment preparation and assessment deadline expired`.
The native selector assertion was `ERR_ASSERTION: 0 !== 1`.
`${HOME}/temp/agent/pi-native-profile-cleanup-regressions-20261014.log` retains the failure.
The original five-second deadline remains unchanged;
the isolated passes did not establish a permanent timing fix.

After the run-owner split,
domain cases and manual uncertainty Allow,
Deny,
Stop,
and cancellation passed.
The prohibited case then exhausted the judgment deadline before producing its policy outcome.
Its final assertion reported `undefined` rather than the expected outcome.
`${HOME}/temp/agent/pi-native-run-owner-regressions-20261014.log` retains that separate occurrence.

The unchanged loaded-input build then failed ordinary Allow twice and passed once.
`${HOME}/temp/agent/pi-tool-call-scope-20261014/binding-set-before.json`
retains complete process durations and enclosing CPU counters.
CPU profiles of separate Allow and prohibition cases both passed;
they show repeated source-freshness validation and filesystem observation costs.
An instrumented Allow case failed with
`DependencyDeadlineError: Original dependency assessment deadline expired`
after 23740 opens of the disposable `workspace/AGENTS.md`.
It also recorded 47478 `lstatSync` calls each for the workspace and its policy file.
The fixture-specific count log is
`${HOME}/temp/agent/pi-tool-call-scope-20261014/loaded-allow-fs.json`.
The instrumentation preserves native filesystem results;
its timing overhead means it is not an uninstrumented latency estimate.

The committed `test:native:source-freshness-work` regression uses the actual built Allow consumer.
Before the change,
the native scenario completed and its work-bound assertion failed at 45637 policy opens.
After complete-binding-set validation,
the native scenario again completed,
and the same assertion failed at 15653 opens.
Both exceed its 10000-open limit.
The change checks every local binding identity between fresh common-parent checks;
individual public source assertions retain their own parent freshness check.
This reduces measured repeated filesystem work without yet completing the performance remedy.
`${HOME}/temp/agent/pi-native-source-work-red-20261014.log`
and `${HOME}/temp/agent/pi-native-source-work-green-20261014.log` retain the failing assertions.
The `green` filename is a stage label,
not a passing result.

The subsequent unchanged repetitions of the first remedy all passed;
`binding-set-after.json` records complete process durations of 8224.54,
6064.67,
and 6179.07 milliseconds.
The remaining duplicate validation is being tested through `readCurrentSourceBinding()`:
the original judgment validates dependencies before returning its original request and source identities.
This lets the governing-source owner avoid separately repeating the same observer and collector reads.
The unchanged 10000-open regression and the approved-dependency rejection controls remain required.
Commit `2ea0d1c836` builds and completes the Allow scenario,
but still fails the work bound at 14185 opens.
`${HOME}/temp/agent/pi-native-current-source-work-20261014.log` records that incomplete remedy.
Subsequent built Allow work counts were 13233 after batching nested rule reads,
12169 after collecting policy selections under one synchronous boundary,
and 11224 after removing the duplicate request read before `active()`.
All still failed the unchanged work bound after completing their native scenarios.
The logs are `pi-native-rule-read-work-20261014.log`,
`pi-native-policy-selection-work-20261014.log`,
and `pi-native-active-check-work-20261014.log` under `${HOME}/temp/agent/`.
The constructor retains the original request read through `active()` while the judgment is open,
and its explicit request read plus retained-dependency validation after evidence closes.
Reading each prepared input and program inspection together measured 10812 opens;
using validated judgment identities during approval measured 10634.
Selecting the matching source before validation did not change that count in this fixture.
Skipping the initial dependency check when complete source validation already performs it measured 10236 opens.
These runs completed Allow but still failed the unchanged 10000-open assertion.
The broader regression sequence is retained in
`${HOME}/temp/agent/pi-native-source-boundary-regressions-20261014.log`.

The separate whole-request claim extraction error in `b3f980b0f8` now has direct built-consumer coverage.
`test:native:request-claim` passes original request identity,
selector reuse,
foreign-handle rejection,
and dependency retirement.
A process-local omission reproduced `ReferenceError: requestMaterialBinding is not defined`
without changing the built module;
the receipt is `${HOME}/temp/agent/pi-tool-call-scope-20261014/request-claim-omission.json`.

The broader regression sequence passed on `ac35a7c2ac`,
including approved policy changes,
cancellation,
prepared argument mutation,
native event receipts,
host lifecycle,
and the isolated CLI process.
Batch selection consolidation measured 10208 opens;
the original-approval validation consolidation measured 10086.
Commit `f79ac0ea45` then passed the unchanged work bound with 9046 opens:
the native tool wrapper reads its original execution origin through the required entry validation,
instead of performing entry validation and then repeating dependency validation to read the origin.
`${HOME}/temp/agent/pi-native-entry-origin-work-20261014.log` retains the passing output.
This is work-count evidence for the fixed fixture,
not a universal latency guarantee.
The expanded final runtime regression sequence passed in two parts.
The first part reached the CLI successfully,
then the scratch runner's next command failed with Mise's
``Monorepo task paths (like `//path:task` or `:task`) require a monorepo root configuration.``
The task lacked a repository working directory.
Pinning its `dir` to the repository allowed all remaining queue,
resource reload,
resource input,
context receipt,
and whole-request claim checks to pass.
No production configuration change was required.
The evidence is `pi-native-source-boundaries-final-20261014.log`
and `pi-native-source-boundaries-remaining-20261014.log` under `${HOME}/temp/agent/`.

The typed file-observer verification reproduced the same scratch-task directory mistake on 2026-10-15.
Its build,
consumer checks,
and type check had completed before the next scratch-owned lint command emitted the diagnostic.
`${HOME}/temp/agent/pi-native-typed-observer-consumer-20261015.log` retains that boundary.
The recovered `lint-typed-observer` task pins `dir` to the repository;
only the failed lint step is resumed.
Its first recovered pass produced ordinary lint diagnostics,
confirming recovery of task discovery without rerunning the successful consumer sequence.

A separate full lint attempt during this cleanup failed before producing diagnostics with
`Error running tsgolint: "exit status: exit status: 2"`
and `runtime: failed to create new OS thread (have 2 already; errno=11)`.
Its log is `${HOME}/temp/agent/pi-maintained-resource-lint-20261014.log`.
A later parent-slice sample showed 50 tasks,
`pids.max=256`,
and cumulative `pids.events: max 340`.
Without a pre-failure counter sample,
these values do not attribute that particular failure.
The current lint retry uses the previously probed `GOMAXPROCS=2` worker setting
with no concurrent native consumer verification.
That retry completed the full 438-file sweep with 203 errors and 12801 warnings.
It verifies recovery of diagnostic collection,
not a passing lint gate or a permanent thread-allocation remedy.
The completed report is `${HOME}/temp/agent/pi-maintained-resource-lint-retry-20261014.log`.
After the source-boundary changes and resumed state cleanup,
the same bounded full lint invocation completed across 455 files with 175 errors and 13188 warnings.
The report is `${HOME}/temp/agent/pi-maintained-source-boundaries-lint-20261014.log`.
This remains a failing quality gate;
the higher warning count exposes additional untyped module boundaries rather than qualifying those extractions.

On 2026-10-15,
overlapping event-fixture verification,
lint,
type checking,
and an explicit-path commit produced separate failures.
The Git commit reported `cannot fork()` for its transaction-local `pre-commit` hook,
followed by `Resource temporarily unavailable`.
Oxlint's TypeScript worker reported
`runtime: failed to create new OS thread (have 33 already; errno=11)` and `fatal error: newosproc`.
The native build's `nano-spawn` wrapper reported its preparation child terminated with `SIGABRT`;
the child supplied no captured stdout or stderr.
That missing child diagnostic does not establish the same cause as the thread-allocation failure.
The logs are `pi-native-event-receipt-split-lint-20261015.log` and
`pi-native-event-receipt-split-consumer-20261015.log` under `${HOME}/temp/agent/`.
The type check passed in `pi-native-conversion-fixture-types-20261015.log`.
The failing lint run produced no structured diagnostic report and cannot supply a source lint result.

After the jobs ended,
the parent slice reported 12 tasks,
`pids.max=256`,
and cumulative `pids.events: max 362`.
There is no immediate pre-failure counter sample tying that counter to a particular allocation.
The original event-fixture edits remained staged and the explicit-path commit retry succeeded.
Recovery follows the existing sequential verification workaround with `GOMAXPROCS=2`.
This serializes the jobs and bounds Go execution parallelism;
it does not raise the cgroup limits or claim a bound on all operating-system threads.
The original failed logs remain intact;
the rebuilt event and converter retry is recorded separately in
`${HOME}/temp/agent/pi-native-event-receipt-split-retry-consumer-20261015.log`.
The standalone retry passes the rebuilt event suite and generated converter tests.
The package's existing lint resource settings now also retain `GOMAXPROCS=2`,
so subsequent lint invocations do not depend on a manually supplied shell prefix.
No new upstream defect is established,
so the existing upstream filing decision remains unchanged.

The follow-up callback shape check first reported
`no-restricted-syntax(no-variable-function-expression)` for direct variable-assigned callbacks.
The existing rule source permits callback function expressions where external APIs dictate positional arguments.
The request callback now comes from a named factory;
the filesystem counter is assigned directly to its native hook.
The focused callback/counter/approval report has zero errors and 300 warnings,
so the warning gate remains red.

### Symptoms and separate incident boundaries

On 2026-10-14,
the native approval task passed through `bare`,
then Mise reported its Node 26.11.1 `prohibited` child as `killed by SIGABRT`.
No fatal Node diagnostic was retained for this occurrence.
The subsequent approval-context and CLI tasks were not reached in that command.
This is not established as the resource-copy heap incident documented separately in this file.

An overlapping full-package Oxlint 1.87.0 run used `oxlint-tsgolint` 7.0.2003
and stopped with:

```text
Error running tsgolint: "exit status: exit status: 2"
runtime: failed to create new OS thread (have 13 already; errno=11)
runtime: may need to increase max user processes (ulimit -u)
fatal error: newosproc
```

Its full log is `${HOME}/temp/agent/pi-auto-mode-full-lint-after-host-20261014.log`.
The retained stack places the failure at Go `runtime/os_linux.go:199`,
through `runtime.newosproc`,
`runtime.newm1`,
and `runtime.newm`.
The installed Go 1.27.2 source at
`${HOME}/.local/share/mise/installs/go/1.27.2/src/runtime/os_linux.go:194`
contains the matching diagnostic path:

```go
// runtime/os_linux.go
if ret != 0 {
    print("runtime: failed to create new OS thread (have ", mcount(), " already; errno=", ret, ")\n")
    if ret == _EAGAIN {
        println("runtime: may need to increase max user processes (ulimit -u)")
    }
    throw("newosproc")
}
```

This installed source explains the message;
its compiler version has not been equated with the linter binary's build version.
The observed worker parent retained its existing 4 GiB memory,
zero swap,
200% CPU,
and 256-task limits.
After the failures,
`pids.events` reported `max 89`,
while memory OOM and OOM-kill counters remained zero.
There was no immediately preceding task-counter sample,
so that cumulative count cannot attribute either process's failure to this particular overlap.
No causal connection between the native abort and the Go thread failure is established.

### Verification and bounded recovery

Process inspection found neither affected verification process still running before retry.
Running the native tasks separately from full lint passed every original approval mode,
the native-context Allow,
Deny,
Stop,
cancellation,
reload controls,
and the separate Qwen CLI's RPC state and empty-stderr shutdown:

```bash
# package/pi-plugin/auto-mode/mise.toml tasks
mise run //package/pi-plugin/auto-mode:test:native:approval
mise run //package/pi-plugin/auto-mode:test:native:approval-context
mise run //package/pi-plugin/auto-mode:test:native:cli-process
```

Then the unchanged full lint task completed its diagnostic sweep on its own:

```bash
# package/pi-plugin/auto-mode/mise.toml task
mise run //package/pi-plugin/auto-mode:lint:oxlint
```

It reported 13,432 warnings and 454 errors across 368 files.
Those are outstanding source findings,
not a passing lint result.
The complete sequential log is `${HOME}/temp/agent/pi-auto-mode-full-lint-sequential-20261014.log`.
The parent task-denial count remained `89` after that sweep.
No memory limit,
task limit,
heap bound,
lint rule,
or linter configuration changed.
Separating these verification batches preserves their assertions and removes their execution overlap;
it does not diagnose the original native abort or prove the overlap caused it.

### Rejected conclusions and upstream filing decision

- The missing native fatal diagnostic prevents attributing `SIGABRT` to the historical heap incident.
- The Go runtime diagnostic establishes failed OS-thread creation,
  not an upstream defect or the responsible resource owner.
- Current evidence does not identify an upstream fix to prototype.
- Supported-use and contribution-policy claims were not made;
  no external filing is proposed.
- No maintainer disposition is inferred from these local failures.
- The successful sequential checks establish a usable verification arrangement,
  not a prototype of an upstream correction.

The checked `.out-of-scope/codex-harness.md` applies to harness integration work;
it is not evidence that either failure originated in the harness.
There is nothing to file upstream from these observations.

## Pi 1.1.0 resource-copy diagnostics exhaust the Node 26.11.1 heap

### Symptom

The typed resource-input port's unit assertions passed,
but the native permission consumer twice aborted at its existing 512 MiB heap limit:

```text
FATAL ERROR: Ineffective mark-compacts near heap limit Allocation failed - JavaScript heap out of memory
```

Node emitted the fatal diagnostic;
Mise reported that the child was killed by `SIGABRT`.
This is separate from the native lifecycle sink-verification timeout.

### Owned source trace

`package/pi-plugin/auto-mode/native/contract/collector/rule-relevance-sdk-copy/input/resource-owner.mjs:85`
rechecks current resource outputs whenever the retained native observation is used:

```javascript
// package/pi-plugin/auto-mode/native/contract/collector/rule-relevance-sdk-copy/input/resource-owner.mjs
try{compareOutputs({scope,outputs:readOutputs(record)});}
```

Its `readOutputs` at line 71 calls `snapshotInput` separately for context,
system,
and appended text.
The resource-copy port added a synchronous logger emission to every `snapshotInput` entry.
The counted permission consumer makes 169,893 snapshot calls and two constructor-option copies.

`package/logger/core/src/create-logger.ts:285`
retains each available sink write until its asynchronous completion:

```typescript
// package/logger/core/src/create-logger.ts
const trackedWrite = trackWrite({
  writePromise: entry.sink
    .write(record,),
},);
pendingWrites.add(trackedWrite,);
void removePendingWriteWhenSettled({ trackedWrite, },);
```

The new emission rate therefore creates asynchronous sink work inside the synchronous freshness checks.
The disposable comparisons isolate those emissions as the trigger.
They do not identify every retained heap object or demonstrate an external runtime defect.
The alternative explanation that merely importing the logger caused the failure was rejected:
the instrumented passing copy retains both the imports and tagged logger construction.

### Verification and correction

The disposable comparison lives at
`${HOME}/temp/agent/pi-resource-input-native-diff-KBHKYZ/run.ts`.
It copies the built owned artifact,
rebases its native absolute imports,
and runs the same permission consumer with the original heap limit.
Its retained logs show:

- The resource-copy implementation preceding `7e337381aa` passes.
- The typed implementation with only its added logging removed passes.
- The complete typed implementation aborts with the fatal heap diagnostic.
- A counted variant retaining logger imports and tagged wrappers passes with 169,893 snapshots.

The comparison JSON records the abort signal separately;
its fallback numeric `exitCode: 0` for signal termination is not a successful result.
Read `signal: SIGABRT` and the retained fatal diagnostic.

Commits `838f5aa849` and `c1d4fe7ecd` retain attempt counts in fixed categories
and emit one diagnostic per microtask batch.
`package/pi-plugin/auto-mode/native/contract/collector/resource-input-custody/diagnostics.ts:63`
uses this bounded scheduling path:

```typescript
// package/pi-plugin/auto-mode/native/contract/collector/resource-input-custody/diagnostics.ts
counts[category]++;
if (state.pending) return;
state.pending = true;
queueMicrotask(flushResourceInputDiagnostics);
```

The diagnostic reports attempted snapshots,
including unsupported kinds,
without retaining input text.
It moves emission to the next microtask and loses individual per-copy timestamps;
complete category counts remain available.
Resource validation,
copy ownership,
freshness checks,
and source budgets stay unchanged.

```sh
# package/pi-plugin/auto-mode
mise run //package/pi-plugin/auto-mode:lint:types
mise run //package/pi-plugin/auto-mode:build:native:owned
mise run //package/pi-plugin/auto-mode:test:native:domain
```

The corrected source passes scoped lint and package types.
The rebuilt native permission,
prohibition,
and unspecified-policy cases pass at the original 512 MiB limit.
Permission executes three prepared child calls;
prohibition and unspecified policy execute none.
These fixtures make no provider requests and adopt no production semantic policy.

The disposable `pi-resource-diagnostic-counts-PaxlX6` consumer exercises the built snapshot function
and real file sink with a 128 MiB child heap.
Its `consumer.ts` sends 100 context,
100 system,
100 append,
and two unsupported-kind attempts in one synchronous batch,
then one system attempt after yielding and flushing.
The retained sink records contain exactly those category counts in two diagnostic writes.
The initial record reader incorrectly assumed that the sink-verification record had a message;
reading the actual JSONL identified its separate `test: true` shape.
The corrected reader validates the original captured records without rerunning the consumer.

### Rejected approaches and limits

Passing isolated copy tests did not exercise the repeated native freshness checks.
Increasing the heap limit was neither attempted nor needed.
Omitting all new logging was used only as a disposable causal probe;
the committed correction retains counted diagnostics.
This change does not reduce the number of freshness checks or establish general performance bounds.

### Upstream filing decision

- The triggering emissions and corrected consumer belong to this repository.
- The owned consumer correction passes the original failing command.
- No unsupported Pi API or Node behavior is established.
- No external contribution or contribution-policy claim is proposed.
- No upstream maintainer disposition is inferred.
- The owned correction is verified;
  no external runtime patch is proposed.

There is nothing to file upstream from this consumer regression.

## TypeScript 7.0.2 rejects copied native source imports from consumer tests

### Symptom and source trace

During the native return consumer migration,
`mise run //package/pi-plugin/auto-mode:lint:types` reported `TS6307`:

```text
File '.../dist/native/contract/collector/program-rule-batch-sdk-copy-v2/input/native-return-types.ts'
is not listed within the file list of project '.../package/pi-plugin/auto-mode/tsconfig.json'.
Projects must list all files or use an 'include' pattern.
```

The same diagnostic named copied return,
access,
error,
and script-guard modules.
Type-only imports still brought those implementation files into the project.
The package source configuration intentionally includes maintained `native/**/*.ts`,
while its shared composite configuration excludes generated `dist` input.
The relevant owned configuration is `package/pi-plugin/auto-mode/tsconfig.json:3`
and `package/config/typescript/tsconfig.options.json:144`:

```jsonc
// package/pi-plugin/auto-mode/tsconfig.json, relevant source selection
{
  "include": [
    "src/**/*.ts",
    "src/**/*.astro",
    "src/**/*.mdx",
    "*.config.ts",
    ".astro/types.d.ts",
    "native/**/*.ts"
  ]
}
```

```jsonc
// package/config/typescript/tsconfig.options.json, relevant inherited option
{
  "composite": true
}
```

The read-only TypeScript `v7.0.2` source checkout is commit
`1e4744d68260a7cb91b62b12edc3f6a2187faaf1`.
Its `src/compiler/program.ts:4124` checks composite project membership:

```ts
// microsoft/TypeScript, src/compiler/program.ts:4124
if (options.composite) {
    const rootPaths = new Set(rootNames.map(toPath));
    for (const file of files) {
        if (sourceFileMayBeEmitted(file, program) && !rootPaths.has(file.path)) {
            programDiagnostics.addLazyConfigDiagnostic(
                file,
                Diagnostics.File_0_is_not_listed_within_the_file_list_of_project_1_Projects_must_list_all_files_or_use_an_include_pattern,
                file.fileName,
                options.configFilePath || "",
            );
        }
    }
}
```

### Verification and correction

The installed compiler reports `7.0.2`.
The failing copied imports are retained in commit `c5bf44db19`;
the correction is in `d6a2d41555` and `9e666dcdcf`.
The initial source-type import also failed the owned `test-import/require-eventual-artifact` rule.
That rule explicitly checks type-only imports in
`package/oxlint-plugin/test-import/src/require-eventual-artifact.ts:225`:

```ts
// package/oxlint-plugin/test-import/src/require-eventual-artifact.ts:225
/*
 Checks every static `import` declaration, including `import type`. A relative
 specifier is allowed when it lands inside a directory the package ships, or
 when it names a test-only fixture or helper.
 */
```

The verified correction declares the tested consumer interface in
`package/pi-plugin/auto-mode/src/native-return-fixture-types.ts`.
The fixture dynamically loads the actual generated `native-return-consumer.mjs` as `unknown`
and checks its callable exports before executing its operations.
It imports no source implementation or copied implementation declaration.
The tradeoff is a separately maintained consumer interface;
runtime behavior assertions must verify that interface against the artifact.

The corrected source type check and initial generated native return suite pass:

```sh
# Repository root, after rebuilding the owned native artifact
mise run //package/pi-plugin/auto-mode:lint:types
mise run //package/pi-plugin/auto-mode:test:native:return
```

The owned build also passes with Node `26.11.1`.
Node prints `ExperimentalWarning: stripTypeScriptTypes is an experimental feature and might change at any time`
when the native preparation stage first erases TypeScript annotations.
That warning is separate from `TS6307` and did not prevent generation.

### Rejected approaches and upstream filing decision

Importing maintained source types failed the artifact-import rule;
importing copied implementation types failed the composite project boundary.
Neither failure justifies disabling a rule or adding generated implementation files to the source project.
The fixture-only type filename must also match the existing fixture convention:
`**/*-fixture*.ts` in `package/oxlint-plugin/test-import/src/checked-file.ts:48`.
No linter or compiler setting changed.

The upstream-filing checks yield no filing:

- Fault belongs to the owned test import choice; no TypeScript defect is established.
- An upstream change is unnecessary for the corrected consumer.
- The compiler source explicitly implements composite source membership.
- Contribution policy was not investigated because no upstream change is proposed.
- Maintainer willingness is untested and irrelevant to this owned correction.
- No upstream prototype is needed; the local consumer correction passed its type and runtime checks.

There is nothing to add upstream from this incident.

## Native source autofixes emit invalid JavaScript

The 2026-10-14 maintained-source `format:oxlint` pass produced these parser diagnostics:

```text
Expected `(` but found `=>`
TS(8009): The 'readonly' modifier can only be used in TypeScript files.
```

### Owned arrow rule source trace

`package/oxlint-plugin/no-restricted-syntax/src/rule/arrow-function-params.ts:77`
implements `extractParamsText` by scanning for balanced parentheses.
It does not supply parentheses for a lone identifier parameter.
`package/oxlint-plugin/no-restricted-syntax/src/rule/no-arrow-function.ts:173`
then concatenates the function name and extracted text:

```typescript
// package/oxlint-plugin/no-restricted-syntax/src/rule/no-arrow-function.ts
const replacement =
  `${exportPrefix}${asyncPrefix}function ${name}${typeParamsText}${returnTypeText} ${bodyText}`;
```

The native input `const sha=value=>createHash('sha256').update(value).digest('hex');`
therefore became an invalid declaration beginning `function shavalue=>createHash('sha256')`.
The same failure affected `digest`,
`source`,
`helperUrl`,
`project`,
and the separate `policy` helpers in the native approval and domain fixtures.
This is a repository-owned autofix defect.

The separate `typescript/prefer-readonly` warning originally targeted `SystemPolicyInput.#state`.
The formatter inserted `readonly #state;` into `system-slot.mjs`,
which Node cannot parse as JavaScript.
The generated diff establishes the invalid output;
this investigation does not establish that external rule's deciding implementation.

### Verified correction and limits

Commit `e6acf4ff53` retains the formatter output,
including the broken declarations.
Commit `26aaecf2dd` supplies the intended named functions with explicit parameter parentheses
and removes the TypeScript-only field modifier.
No rule was disabled or weakened.

Run the consumer parser and native domain cases after rebuilding:

```sh
# package/pi-plugin/auto-mode
mise run //package/pi-plugin/auto-mode:lint:native:syntax
mise run //package/pi-plugin/auto-mode:build:native:owned
mise run //package/pi-plugin/auto-mode:test:native:domain
```

Node parsed all 157 maintained native modules after correction.
The rebuild passed,
and the permission,
prohibition,
and unspecified-policy cases retained their respective execution behavior.
The workaround changes only the declaration syntax;
it does not fix either formatter.
Reapplying the complete fixer to the JavaScript class can reintroduce the `readonly` error.
The failed formatter's remaining diagnostic count cannot establish full lint coverage
because syntax errors prevented complete analysis.

Commit `2728f357f` repeated that known modifier insertion during the later formatting pass.
The built domain test stopped at `SyntaxError: Unexpected identifier '#state'`
before any test body ran.
Commit `bbb1aa2f64` removed the modifier again.
The rebuilt domain and subsequent native regression tasks passed.
Do not treat a successful build or package type check as validation of copied JavaScript syntax.

The expanded syntax task also passed `.ts` files directly to `node --check`.
Node `26.11.1` rejected valid `override readonly` syntax in `process-network-guard.ts`.
Commit `13ec2b134a` calls Node's `stripTypeScriptTypes` for TypeScript inputs before syntax checking;
JavaScript remains unchanged.
The corrected task passes all 227 maintained modules.

### Upstream filing decision

- The arrow extraction fault belongs to this repository;
  external responsibility for the separate JavaScript modifier case is not traced.
- The explicit declaration correction is available at the consumer boundary.
- No Pi support limitation follows from these parser errors.
- No external contribution or contribution-policy claim is proposed.
- No maintainer disposition is inferred.
- The consumer correction passes the actual parser and native domain checks;
  an upstream formatter patch has not been proposed.

There is nothing to file upstream from this bounded consumer repair.

## Node 26.11.1 package build fanout reports `spawnSync mise EPERM`

### Symptom and observed boundary

On 2026-10-14,
`mise run //package/pi-plugin/auto-mode:build:js`
stopped before Rolldown with this Node diagnostic:

```text
Error: spawnSync mise EPERM
spawnargs: [ 'tasks', '--all', '--hidden', '--json' ]
status: 0
signal: null
```

The exception also retained task-list stdout and mise cache-write warnings.
It therefore does not establish that the subprocess never ran.
The command was inside the existing sandbox and worker cgroup.
No inference request or native host activation occurred.

### Source trace and unresolved cause

`mise.toml:335` reads the complete task catalog before selecting package children:

```javascript
// mise.toml, fanout template
const MISE_TASKS_JSON_MAX_BUFFER = 16 * 1024 * 1024
const tasks = JSON.parse(execFileSync('mise', ['tasks', '--all', '--hidden', '--json'], {
  encoding: 'utf8',
  maxBuffer: MISE_TASKS_JSON_MAX_BUFFER
}))
```

The installed Node 26.11.1 `node:child_process` source at lines 1017 and 1024 to 1027
passes the synchronous spawn result through `checkExecSyncError` and throws that error.
The source was read from the running executable's built-in module text.
This identifies the reporting path,
not the cause of `EPERM`.
No deciding kernel or sandbox failure was established.
The [historical task-buffer incident](mise-task-fanout-buffer.md) reported `ENOBUFS`;
that diagnosis does not explain this different diagnostic.

### Verified workaround

The explicit child task bypasses the task-catalog subprocess and builds the same configured Node artifact:

```sh
# package/pi-plugin/auto-mode
mise run //package/pi-plugin/auto-mode:build:js:node
mise run //package/pi-plugin/auto-mode:build:native:owned
```

`mise.toml:561` defines the child as
`rolldown --configLoader native --config rolldown.node.config.ts`.
That task passed with Rolldown 1.2.13,
and the subsequent owned native build passed.
The built sender passed isolated,
loopback HTTP,
and canonical native policy-review checks.
This workaround selects the existing Node child explicitly;
it will not discover future sibling build targets automatically.

Repeating the failed fanout command was unnecessary for delivery.
No buffer setting,
task template,
tool version,
or sandbox rule was changed.

### Upstream filing decision

- Upstream fault is not established by the returned error.
- No upstream fix target is established.
- The repository already defines the successful explicit child task.
- Contribution policy was not investigated because no external change is proposed.
- Maintainer disposition is unknown and is not inferred from this local failure.
- The consumer workaround passes;
  no upstream prototype is justified by the available cause evidence.

`.out-of-scope/codex-harness.md` excludes Codex-specific harness investigation and filing.
This record does not assign the failure to that harness.
There is nothing to file upstream from the observed boundary and working task alternative.

## Pi 1.2.0 dependency refresh invalidates an emitted 1.1.0 host

### Symptom and cause

After workspace commit `755685da03` refreshed installed Pi packages,
the installed-extension resolver failed before loading extensions.
Node `26.11.1` reported:

```text
Error [ERR_MODULE_NOT_FOUND]: Cannot find package 'partial-json' imported from
package/pi-plugin/auto-mode/dist/native/pi/ai/dist/utils/json-parse.js
```

The generated AI package's `node_modules` link still named the removed
`@earendil-works+pi-ai@1.1.0_undici@8.11.2/node_modules` directory.
The workspace now contained the corresponding `1.2.0` directory.
This is stale owned build output after a dependency refresh,
not evidence of a missing dependency in the upstream release.

`package/pi-plugin/auto-mode/native/build-contract.ts:16` now selects the exact installed release:

```ts
// package/pi-plugin/auto-mode/native/build-contract.ts
export const qualifiedRelease = {
  version: '1.2.0',
  commit: '06a466935536f8c9bcacb2d5efc0fc021be1bdff',
} as const;
```

`package/pi-plugin/auto-mode/native/build-emission.ts:156` replaces the owned generated package tree:

```ts
// package/pi-plugin/auto-mode/native/build-emission.ts
const target = join(output, 'pi', specification.folder);
await rm(target, { recursive: true, force: true });
```

The normal emission then copies the qualified source and creates its matching dependency link.
Replacing the tree also prevents removed upstream source files from surviving a release change.

### Verification and recovery

The unchanged read-only resolver failed against the stale output.
After the source update and full build,
that resolver completed with 18 configured package entries and 22 enabled extension entries
after excluding the legacy auto-mode entry that the native host replaces.

```sh
# Repository root
mise run //package/pi-plugin/auto-mode:build:native
mise run //package/pi-plugin/auto-mode:lint:types
mise run //package/pi-plugin/auto-mode:test:native:host-lifecycle
mise run //package/pi-plugin/auto-mode:test:native:cli-process
```

These commands passed against source tag `v1.2.0` and its recorded commit.
The lifecycle check retained original construction,
reload,
cancelled replacement,
fork,
resume,
and disposal-retry behavior.
The actual CLI returned the selected Qwen model through RPC and shut down with empty stderr.
These fixtures made no external provider requests.

The recovery rebuilds generated output and requires the native acceptance checks on the new release.
It does not edit installed dependencies or qualify the configured extensions merely by resolving their paths.
The owned-only rebuild cannot repair stale upstream dependency links because it preserves upstream emission.
The source checkout remains a build input only.

### Upstream filing decision

- The stale dependency link belongs to this repository's generated output.
- The consumer build can recreate its own source and dependency boundary.
- No unsupported upstream use case is established by this failure.
- No external contribution is proposed,
  so contribution-policy research is unnecessary.
- No maintainer disposition is inferred.
- The consumer rebuild and original host checks pass;
  an upstream patch is neither needed nor proposed.

There is nothing to file upstream from this owned build recovery.

## Pi 1.2.0 progress delivery invalidates the migration's frozen source snapshot

### Symptom

The maintained `test:native:progress-events` fixture approves a complete Codemode operation,
then enters no child tool.
One `tool_execution_update` callback returns no value and leaves its event unchanged.
The migration emits `PolicySlotBindingError: Native extension event callbacks changed after source capture`.
The ordinary trace has a pending callback;
a diagnostic that awaits it reaches the same error after the callback completes.

### Root cause

Upstream source tag `v1.2.0`,
commit `06a466935536f8c9bcacb2d5efc0fc021be1bdff`,
constructs its progress callback in `packages/coding-agent/src/extensions/codemode/execute.ts:460`:

```ts
// packages/coding-agent/src/extensions/codemode/execute.ts
const publish = () => onUpdate?.({ content: [], details: snapshot() });
publish();
```

The migration's native group boundary returns asynchronous event delivery in
`package/pi-plugin/auto-mode/native/contract/integration/native-batch/boundary.ts:38`:

```ts
// package/pi-plugin/auto-mode/native/contract/integration/native-batch/boundary.ts
onUpdate:async (partial,call) => emit({
  type:"tool_execution_update",toolCallId:call.id,
  toolName:call.name,args:call.arguments,partialResult:partial
})
```

The migration's observer changes its publication when an emission begins and completes.
The update helper in `package/pi-plugin/auto-mode/native/contract/integration/native-batch/event-observer.ts:112`
invalidates the original cached object:

```ts
// package/pi-plugin/auto-mode/native/contract/integration/native-batch/event-observer.ts
state.revision++;
delete state.view;
```

The source collector rejects any later publication in
`package/pi-plugin/auto-mode/native/contract/integration/policy-source/system-slot.mjs:418`:

```js
// package/pi-plugin/auto-mode/native/contract/integration/policy-source/system-slot.mjs
if(nativeDomain?.observation!==snapshot.nativeDomainEventObservation)
  throw new PolicySlotBindingError(
    `Native extension event callbacks changed after source capture: ${JSON.stringify({
      before:snapshot.nativeDomainEventObservation,after:nativeDomain?.observation
    })}`
  );
```

The initial progress delivery therefore changes the judgment's source observation.
Its complete input and settled input remain retained by the native manager.
This is a consumer integration failure;
the source trace alone does not establish an upstream defect.

### Verification

The normal permission consumer without a progress callback passes with three child executions.
The maintained reduced callback fixture fails with one authored assessment,
one callback invocation,
and zero child executions:

```sh
# Repository root
mise run //package/pi-plugin/auto-mode:build:native:owned
mise run //package/pi-plugin/auto-mode:test:native:progress-events
```

The ordinary failure moves the native observation from revision zero,
pending zero,
to revision one,
pending one.
A private Node load hook changed only the first `publish()` to `await publish()` in memory.
Its application assertion confirmed the selected callsite.
That run moved to revision two,
pending zero,
and still produced zero child executions with the same source-identity diagnostic.
Neither diagnostic used external provider inference or changed an artifact file.

The first diagnostic-loader attempt failed before the callback because it expected Node's module source to be text.
Installed Node `ModuleSource` also permits bytes.
Decoding that representation allowed the intended await contrast to run;
the failed loader attempt is not evidence about event settlement.

### Workarounds and rejected approaches

No production workaround is established.
Waiting for callback completion alone does not preserve the selected source identity.
Removing the callback would alter the user's configured extension behavior.
Ignoring source changes without an adopted timing rule would silently change the governing-domain contract.
The user adopted decision-scoped timing on 2026-10-14:
later received inputs govern the next decision.
The native collector now preserves the assessed source set during approved execution
while checking original policy,
resource,
run,
and receipt identities.
Its dependency facade uses that same collection;
an independently captured facade snapshot initially continued to reject progress.
The reduced progress consumer now completes three child calls through seven callback invocations
with one authored assessment.
The next-decision consumer retains an input mutation and unused callback result,
then blocks the following operation after its own assessment.
Selected-policy changes and cancellation still stop the remaining child calls.
The adopted timing and its measured contrasts are in
`doc/planning/pi-auto-mode-governing-domain-closure.md`.

### Upstream filing decision

- The failing source-identity guard belongs to this migration.
- Upstream could change progress scheduling,
  but the measured await contrast does not solve this consumer failure.
- Native extension progress delivery is present;
  no upstream promise about this migration's frozen snapshots is established.
- No external contribution is proposed,
  so contribution-policy research is unnecessary.
- No maintainer disposition is inferred.
- No upstream fix is claimed or proposed;
  the adopted receipt-timing implementation belongs to the consumer.

There is nothing to file upstream from this diagnosis.

## Pi 1.2.0 configured operation exceeds migration receipt capacity

### Symptom

After the timing correction,
the isolated fixture loads all 22 configured extensions and executes three child calls.
Cleanup then fails with
`InputReceiptBudgetError: Combined original resource and session receipts exceed the shared 64-input or four-MiB budget`.

### Root cause

This failure is separate from the frozen-source timing failure.
The shared arithmetic in `package/pi-plugin/auto-mode/src/input-receipt-budget.ts:170`
checks both source count and serialized bytes:

```ts
// package/pi-plugin/auto-mode/src/input-receipt-budget.ts
if (((resourceUsage.count + sessionUsage.count) > INPUT_COUNT_LIMIT)
  || ((resourceUsage.bytes + sessionUsage.bytes) > INPUT_BYTE_LIMIT)) {
```

The committed artifact has a count limit of 64 and a byte limit of 4,194,304.
The proposed working-tree change sets the count limit to 128;
it has not been activated or committed.

### Verification

The fixture uses the qualified Pi `v1.2.0` source and the same 22 configured extension paths
as the frozen-source investigation.
The original 64-count artifact completes the reduced progress fixture,
but fails the configured fixture during execution cleanup.
Run the isolated configured fixture with:

```sh
# Repository root, previously verified private isolated fixture
node "${HOME}/temp/agent/pi-extension-isolation-20261014.test.ts" extensions
```

The count boundary is independently isolated by a private Node load hook that changes count limits to 128 in memory,
keeps byte limits unchanged,
and records application markers for every transformed module.
The corrected experiment completes the configured consumer,
reaching 81 receipts and 347,913 bytes at the last observed check.
No artifact file changes or provider requests occur in that experiment.

```sh
# Repository root, private count-only diagnostic retaining the isolated fixture
node "${HOME}/temp/agent/pi-receipt-limit-isolation-20261014.test.ts" extensions
```

### What does not work

The first hook matched every package ending in `dist/final/node/index.mjs`.
Its asserted replacement failed on unrelated extension bundles,
and the resulting fallback reported a missing `compat.js/api/openai-codex-responses` module.
Scoping the hook to `package/pi-plugin/auto-mode/dist/final/node/index.mjs` fixes that diagnostic mistake.
That failed import does not establish a configured extension defect.

### Proposed remedy and tradeoff

The prepared 128-receipt capacity patch retains the four-MiB combined byte limit,
the resource-local count and byte limits,
and the original assessment budget.
Automatic approval review rejected the commit pending explicit user authorization for the limit change.
The user approved that increase on 2026-10-14.
Commit `b45783436f` implements it while preserving the resource-local 64-receipt limit.
The rebuilt artifact passes the original isolated configured-extension runner without a diagnostic hook.
The 128/129 native receipt boundary and combined count/byte unit controls also pass.
The approved cap permits more source records to be processed within the same byte bounds.
It does not provide unlimited session history or establish permission for later instructions.

### Upstream filing decision

- The failing count guard belongs to the migration,
  so this is not an established upstream defect.
- Pi does not own the migration's count limit;
  an upstream patch is not the remedy evaluated here.
- The fixture exercises Pi's existing extension dispatch,
  without claiming an upstream promise about this consumer's capacity.
- No external contribution is proposed,
  so contribution-policy research is unnecessary.
- No maintainer disposition is inferred.
- The prototype changes consumer limits only,
  retains all byte bounds,
  and does not modify upstream source or installed artifacts.

There is nothing to file upstream from this capacity measurement.

## Pi 1.2.0 native CLI fails during owned runner initialization

### Symptom

The real CLI process fixture failed before returning RPC state on 2026-10-14.
Node 26.11.1 reported `ReferenceError: Cannot access 'ExtensionRunner' before initialization`
from the generated `run-emitter.mjs`.
The configured SDK-session fixture passed because it entered the module graph through another path.
Its success did not establish CLI startup.

### Root cause

The owned prompt-emitter generator read the runner prototype during module evaluation.
Before commit `7c210e871b`,
`package/pi-plugin/auto-mode/native/contract/integration/native-batch/prompt-emitter-source.ts:355`
emitted this eager access:

```js
// Generated run-emitter.mjs, before the correction
const nativeCreateContext = ExtensionRunner.prototype.createContext;
```

The event runner now participates in the manager/session import graph.
The generated session imports the prompt emitter in
`package/pi-plugin/auto-mode/native/contract/integration/native-batch/prepare-session.ts:211`:

```js
// Generated session import emitted by prepare-session.ts
import { emitOwnedBeforeAgentStart, emitOwnedContext, emitOwnedInput } from './run-emitter.mjs';
```

The CLI entry reaches the prompt emitter before the runner class finishes initialization.
Moving prototype capture to the defining runner module avoids that cross-module eager read.
`package/pi-plugin/auto-mode/native/contract/integration/native-batch/event-runner-source.ts:334`
now emits the captures after the class:

```js
// Generated event-runner.mjs, after the class declaration
export const nativeCreateContext = ExtensionRunner.prototype.createContext;
export const nativeAssertActive = ExtensionRunner.prototype.assertActive;
export const nativeEmitError = ExtensionRunner.prototype.emitError;
```

The prompt emitter imports those live bindings and uses them only when invoked.
Original methods remain captured before callbacks run;
the correction does not introduce late prototype lookup during callback dispatch.

### Verification and rejected approaches

The existing process fixture is the regression control:

```bash
# Run from repository root after rebuilding the owned native artifact.
mise run //package/pi-plugin/auto-mode:test:native:cli-process
```

It failed with the initialization error before the correction.
After the correction it returned the requested Qwen model and idle RPC state,
then exited successfully with empty stderr and zero provider requests.
Next-decision receipt verification,
package types,
and scoped generator lint also pass.
All maintained unit tests passed before this generator-only correction.
Logs are `${HOME}/temp/agent/pi-approved-capacity-cli-20261014.log`
and `${HOME}/temp/agent/pi-native-live-runner-cli-20261014.log`.

The passing configured-session fixture alone is rejected as proof of CLI startup.
No import identity bypass,
SDK version change,
or late mutable prototype lookup was needed.
The tradeoff is an explicit generated-module dependency on the runner's captured exports.

### Upstream filing decision

- The eager prototype access belongs to the owned generator.
- The consumer can correct its module initialization order without upstream changes.
- No upstream support promise for this transformed graph is asserted.
- No external contribution is proposed or contribution policy claimed.
- No maintainer disposition is inferred.
- The maintained generator correction and actual CLI regression provide local verification.

There is nothing to file upstream from this consumer initialization failure.

## Pi 1.2.0 installed policy capture calls an absent optional effect owner

### Symptom

The installed Qwen check emits the supported program `return await tools.read({path:"proof.txt"});`.
The owned dispatcher returns `Complete group preparation or assessment failed`.
The retained lifecycle cause is `TypeError: effectOwnerFor is not a function`.
No file content is returned.
This is separate from the preceding rejection of unsupported single-call syntax.

### Root cause

The runtime contract permits omission of the implementation-fact callback:
`package/pi-plugin/auto-mode/native/contract/integration/native-batch/runtime-types.ts:47` declares:

```ts
// package/pi-plugin/auto-mode/native/contract/integration/native-batch/runtime-types.ts
readonly effectOwnerFor?: NativeForwardedFunction;
```

The installed bootstrap omits that callback.
The original collector nevertheless called it unconditionally.
`package/pi-plugin/auto-mode/native/contract/integration/action-policy/inputs.mjs:77`
now retains the intended absence:

```js
// package/pi-plugin/auto-mode/native/contract/integration/action-policy/inputs.mjs
const owner = effectOwnerFor?.({ judgment, member, prepared });
```

The existing absent-owner branch retains `unresolved-effect` evidence.
The correction supplies no implementation fact or execution permission.
The native domain fixture previously supplied a callback returning `undefined`,
so it did not exercise the installed host's omitted property.

### Verification and correction

Pi 1.2.0 uses the qualified source revision `06a466935536f8c9bcacb2d5efc0fc021be1bdff`.
The failed live log is `${HOME}/temp/agent/pi-single-call-installed-live-20261014.log`.
Removing the fixture's dummy callback reproduces the exact TypeError locally,
without provider requests,
in `${HOME}/temp/agent/pi-missing-effect-owner-red-test-20261014.log`.

```bash
# Run from repository root after rebuilding maintained and native artifacts.
mise run //package/pi-plugin/auto-mode:test:native:single-call
```

With the optional call in commit `b3581b0be0`,
both single-call forms execute one child and return its original result.
Prohibition executes no child.
Copied declarations and unsupported syntax still reject.
The passing log is `${HOME}/temp/agent/pi-optional-effect-native-test-20261014.log`.
The provider-backed installed check must still be repeated.

Supplying a dummy callback only in the fixture is rejected as verification of omitted configuration.
Changing the semantic threshold,
inventing effect facts,
or attributing this error to Qwen is unnecessary and unsupported.
The tradeoff is explicit absence handling at the collector;
all existing unresolved-fact obligations remain.

### Upstream filing decision

- The unconditional call belongs to the owned policy collector.
- The consumer fixes it without changing Pi or provider code.
- No upstream contract for this owned callback is asserted.
- No external contribution or contribution-policy claim is proposed.
- No maintainer disposition is inferred.
- The committed correction passes the actual omitted-property regression locally.

There is nothing to file upstream for this owned callback mismatch.

## Pi 1.2.0 MiMo inspection loses prompt evidence and masks Stop

### Symptom and original evidence

The 2026-10-17 MiMo lint session proposes parallel Bash calls
`call_cec448f1353443bd88da096e` and `call_8cacc98bebf7433a9945f1d3`.
Both return `Complete group preparation or assessment failed`.
One command searches for disposal syntax;
the other reads an existing lint log with a Python heredoc.
Earlier Python heredocs in this same session succeed.

Session `2026-10-17T15-47-34-073Z_01a14a8c-0a38-760f-aca3-aa17ebd2a725`
and original logger file `2026-10-17T15-47-33.217Z.log.jsonl` establish the sequence.
The original retained cause is `RangeError: Invalid string length` at prompt capture,
followed by `RequestObservationStaleError: Observed request is no longer current`.
The native selector opens at `2026-10-17T16:14:03.781Z`.
It returns at `2026-10-17T17:18:03.286Z`,
records the response,
and takes the native Stop path.
The dispatcher then reports `AbortError: This operation was aborted` and publishes the generic message.
The interval is retained selector wait time;
it does not establish a model service timeout.

### Root cause and maintained boundaries

Before commit `2f90217589`,
`package/pi-plugin/auto-mode/native/contract/collector/run-prompt-custody-v3/owner.mjs:137`
defines an unused run byte counter:

```js
// Historical owner.mjs, removed on 2026-10-18.
function charge({scope,value}) {
  scope.bytes += Buffer.byteLength(JSON.stringify(value));
  return value;
}
```

The counter's only references initialize or increment it;
the run view and diagnostics never read it.
Historical `context.mjs:55` passes every retained origin to this function together.
Separate supported inputs therefore become one aggregate string solely for unused accounting.
The original failure stack names this `JSON.stringify` call before command assessment.
The maintained context keeps the original records directly:
`package/pi-plugin/auto-mode/native/contract/collector/run-prompt-custody-v3/context.mjs:55`.

```js
// Maintained context.mjs.
const retainedMessageOrigins = Object.freeze(origins);
```

The shared text store belongs to the existing manager pool,
not to another source owner.
`package/pi-plugin/auto-mode/native/contract/collector/run-prompt-custody-v3/loaded-pool.mjs:42`
reuses immutable content and its digest:

```js
// Maintained loaded-pool.mjs.
let data = sourceDataByContent.get(serialized);
if (data === undefined) {
  data = Object.freeze({content:serialized,sha256:createHash('sha256').update(serialized).digest('hex')});
  sourceDataByContent.set(serialized,data);
}
```

The same `retain()` call still creates a fresh source occurrence and publication for every receipt.
It still charges each source separately to the manager's consumed usage counters.
Equal text and hashes do not merge authority or capability identity.
This addresses repeated serialized data retained during long native event sequences.

Stop is a separate error-masking boundary.
`package/pi-plugin/auto-mode/native/contract/lifecycle/incumbent-tui-group/approval-presentation.mjs:84`
calls the original native abort capability:

```js
// Maintained approval-presentation.mjs.
abort(){trace('tui-stop');return Reflect.apply(abort,ctx,[]);}
```

The dispatcher formerly checked cancellation before retaining the returned blocking decision.
That replaced the user's Stop result with a preparation failure.
The maintained dispatcher validates the decision first and checks execution eligibility only for release:
`package/pi-plugin/auto-mode/native/contract/collector/program-rule-batch-sdk-copy-v2/input/prepared-dispatch.mjs:147`.

```js
// Maintained prepared-dispatch.mjs.
if (!decision.block) active('assessment-returned');
state.decision = Object.freeze(decision.block ? {block:true,reason:decision.reason} : {block:false});
```

No blocked member enters execution.
Approval retains its asynchronous currentness check and the native execution guards.

### Verification catalogs

The original failing output is `native-release-qWSxFz`.
Source fixes are `2f90217589`,
`daf6b8e179`,
and `77620b6c23`.
The current shared-text output is `native-release-iQw5fd`.
Private captured sessions,
replays,
and logs live in `${HOME}/temp/agent/pi-mimo-preparation-failure-20261018`.

Failing controls:

- The aggregate capacity test fails on the original build with the exact `RangeError` and `charge()` stack.
- Native `stop` and `cancel` tests originally return the generic preparation error instead of the stopped message.
- Configured-host replay `pi-mimo-failure-rpc-xWj1Z0` reproduces the generic result after original Stop.
- Full character-fragmented replay still exhausts the 2 GiB container after removing only the dead counter.
  It does not count as successful consumer verification.

Passing measured boundaries:

- `test:native:retained-context-capacity` keeps separate authenticated records larger than one engine string.
  The revised ASCII JSON fixture accounts for 713,092,714 bytes,
  retains distinct source occurrences and publications,
  observes a later distinct input,
  and rejects a changed native root.
- Three unchanged baseline runs retain 688,995,568 to 689,029,488 heap bytes.
  Three shared-text runs retain 3,609,280 to 3,610,472 heap bytes.
  Both sets use the same fixture and per-occurrence receipt accounting.
- Actual native `stop` and `cancel` modes retain
  `The user stopped execution. Wait for their next instructions.`
  in every withheld result and execute no member.
- The counter/Stop build passes decision fallback,
  prompt capacity,
  run history,
  run-message freshness,
  native phases,
  single-call catalogs,
  and type checking with a disposable writable package cache.

The configured full-conversation replay of `native-release-iQw5fd` subsequently exhausts the heap.
The later callback-sharing build `native-release-6z61x7` also exits 134 in both full replays.
Those results supersede the pending status recorded on 2026-10-18.
The lint handoff remains reopened until complete replay and negative controls pass on the final output.

Reproduction tasks:

```sh
# Repository package tasks, each run in a disposable resource-bounded fixture during this investigation.
mise run //package/pi-plugin/auto-mode:test:native:retained-context-capacity
mise run //package/pi-plugin/auto-mode:test:native:decision-fallback:case stop
mise run //package/pi-plugin/auto-mode:test:native:decision-fallback:case cancel
```

### Rejected alternatives and workaround limits

Changing the Bash search or Python syntax cannot repair a failure that occurs before command assessment.
An oversized individual message is not required:
the regression uses individually supported records and fails only when the old counter combines them.
No custom serializer is required.
The source records and publication capabilities remain original immutable objects.

Restoring only persisted history loses the original request-evidence construction sequence.
That fixture falls back to manual review for a different reason,
so it cannot establish the capture repair.
The full replay rebuilds original proposals from a fresh configured host instead.
Dropping history,
disabling freshness,
loosening approval thresholds,
or granting a second generic Allow after an established stale decision are not repairs.

The verified local corrections remove unused aggregate serialization,
share stored primitive data within the existing pool,
and preserve blocking decisions through native cancellation.
The tradeoff is a pool-lifetime text index in addition to the original per-occurrence records.
There is no source-capacity ceiling,
receipt deletion,
or shared publication capability.

The first read-only type-check container fails with
`EROFS: read-only file system, unlink '.cache/typescript/tsconfig.tsbuildinfo'`.
The existing type-check task removes its own build-info cache before checking.
Mounting a disposable writable package cache makes that same task pass.
This harness filesystem error is distinct from the product's capture and Stop failures.

### Full-session source-binding allocation

The original capture failure and later heap failures are distinct observed boundaries.
Removing dead aggregate serialization repairs the original `RangeError`,
but complete configured-host replays still expose retained allocation growth.
The callback-sharing build preserves all ordered callback phases with per-emission value sharing;
it still exhausts the heap after 28 fragmented-stream or 34 faithful-stream live assessments.

Private pool profiling reaches 103,200 original occurrences and 61,899,464 unique content characters.
The sampled heap reaches 1,396,560,032 bytes.
An inspector allocation sample at 100,000 receipts retains its explicit 2,097,168-byte positive control.
It attributes 476,526,736 sampled bytes to `bindOriginalRunMessage`
and 153,547,688 to `createMessagePublication`.
Its total sampled live allocation is 1,150,318,896 bytes.
These are allocation samples, not exact object counts or exclusive proof of a complete-session cause.
The private source is `allocation-summary.json` for fixture `pi-mimo-failure-rpc-e8GtWu`
in the incident scratch.

`package/pi-plugin/auto-mode/native/contract/integration/policy-source/run-messages.mjs:33`
now owns shared checking methods within each governing snapshot.
A private weak set authenticates the original frozen plain binding before its view is read.
The source view, request, occurrence, ordering, and parent freshness remain independently checked.
This removes checking closures per source while preserving separate records and original publications.

Three unchanged 120,000-binding probes retain 48,742,376 to 48,742,952 bytes.
Three shared-method probes retain 30,762,600 to 30,763,960 bytes.
The committed capacity regression fails on the old output at 48,724,128 bytes,
against a 36 MiB budget.
The rebuilt `native-release-2vxGY8` passes at 30,747,064 bytes,
including foreign receivers, cross-snapshot borrowing, substituted sources, and retired parents.
A private receiver-guard mutation makes the forged-receiver assertion fail;
the production owner hash remains `0ad7cf1e86d17bcde704bb0d610b986eef81d1b7132bb948a965bca8e43724e7`.
Complete configured-host replay remains required after this isolated improvement.

The earlier 20,000-receipt allocation sample emphasizes queued log writes.
A synchronous-append counterfactual still exits 134.
The conclusion that this excluded logger append retention is withdrawn on 2026-10-18:
the hook changed the bundled logger,
while configured extensions also loaded a separate TypeScript logger instance.
The hook receipt proves bundle loading,
not replacement of that source instance's append path.
The private input-only publication probe reduces its separate capacity fixture to 2,753,896 bytes;
it is not applied to production and does not establish a full-session repair.
The class-based binding probe is also rejected:
it changes the existing plain-record prototype and constructor surface.

### Disposable build and logging controls

The first binding probes discover a default file-log destination outside their writable mounts
and emit `logger internal error: file sink verification failed: EROFS: read-only file system`.
Setting the new logger directory environment variable cannot repair an older bundled policy
that lacks that implementation.
The corrected probe mounts its discovered diagnostic directory writable and preserves the records.
Its repeated measurements contain no corresponding file-sink error.

The owned native rebuild copies existing JavaScript policy output.
A fresh `build:js` precedes the `native-release-2vxGY8` rebuild;
the emitted policy now contains `MONOCHROMATIC_LOG_DIRECTORY` support.
The disposable JavaScript build initially fails with `Error: spawnSync mise ENOENT`
because the parent task invokes child tasks by executable name.
Adding the mounted `/fixture-bin` to the container's `PATH` makes the same task complete.
Neither harness filesystem error nor missing executable establishes a product preparation failure.

### Upstream filing decision

- Ownership: unused aggregate accounting and retained input storage belong to this repository.
- Fixability: local owner and dispatcher changes repair the traced boundaries.
- Supported use: this package promises configured native callbacks and repeated session turns.
- Contribution policy: no external repository contribution is proposed.
- Maintainer disposition: no external maintainer decision is inferred.
- Prototype: original failing controls and genuine native regressions exist;
  final full-consumer verification is still pending.

There is nothing to file upstream for these owned implementation defects.

## Logger completion retention and file ownership on 2026-10-18

### Separate measured failures

A completed faithful configured-host replay exercises all 52 saved calls and captures 62 actual Luna requests.
Its fragmented-stream stress counterpart still exits 134 before the logger repairs.
The active artifact for that checkpoint is `native-release-XGbh0H`.
Successful original-boundary replay and unfinished streaming-memory repair remain separate conclusions.

A collected heap snapshot exposes a large logger `pendingWrites` set through the module-local default instance.
The original retained graph is not a computed dominator proof.
An isolated consumer supplies the stronger capacity control:
100,000 sink deliveries returning one shared fulfilled completion retain 93,863,816 heap bytes
in the committed built-artifact regression before the repair.
The subsequent repaired verification retains 48,304 bytes and still delivers all records.

The logger originally created tracking and cleanup continuations for each record.
Its producer can synchronously emit records faster than the JavaScript microtask queue can drain those continuations.
`src/pending-writes.ts` now observes each active sink completion once,
without skipping sink deliveries.
Synchronous backends reuse a fulfilled completion,
and the file sink shares completion across bounded append batches.
Flush deadlines retain their observation-abandonment behavior;
the same unfinished operation can be observed again after a deadline.

Independent file regressions reveal additional ownership defects:

- A record's `toJSON` hook reenters the same sink.
  Capturing the predecessor before serialization lets the inner and outer append operations overlap.
  The original output fails JSON parsing at position 349536.
- Separate sinks initialized in one frozen millisecond choose the same timestamp-only filename.
  Their separate serial queues still overlap on the shared file.
  The unchanged artifact fails JSON parsing at position 349544.
- A valid record object whose `toJSON` returns `undefined` inserts the literal `undefined` into JSONL.
  The unchanged artifact emits `SyntaxError: "undefined" is not valid JSON` during read-back.

The repairs finish serialization before queue admission,
reject serialization without a JSON value,
use a UUID in each sink filename,
create the probe file exclusively,
and publish an append queue only after successful probe read-back.
Batch appends remain serial and preserve whole records.
All corrected file cases pass the runtime suite;
final types and lint verification remain recorded in the canonical frontier.
These file controls do not establish the cause of the original auto-mode preparation failure.

### Consumer and environment verification

The host imports logger source through configured extensions and also loads a bundled logger.
Those module-local defaults have independent state.
Both consumer outputs need rebuilding before replay can establish the memory repair at the Pi boundary.
The logger package's
[write completion and file ownership explanation](../../package/logger/core/doc/write-completion.md)
records the roles,
alternatives,
completion limits,
and maintained regressions.

The direct MiMo RPC pilot selects `llmgateway-devpass/mimo-v2.6-pro`,
receives `READY` without tool execution,
and exits normally under a 2-GiB memory and 2-CPU limit.
Its disposable home copies configured resources and credentials privately.
The controller resolves the repository's Mise environment without printing its values.
An initial pilot lacking that environment emits
`AssessmentTransportError: AUTO_MODE_LLMGATEWAY_API_KEY is unavailable for the assessment request`.
The corrected environment resolves that startup failure.

The package `native:cli` task changes cwd to the package directory.
The current policy bootstrap then attempts that directory's absent `AGENTS.md` and emits `ENOENT` before RPC starts.
Launching the maintained Pi command from the repository root exercises the intended reported consumer.
The disposable mount setup also preserves both home aliases and disables automatic Mise installation.
These controller corrections do not change the package's tool-review policy.

No Node,
JavaScript,
Pi,
or Linux defect is established by these controls.
The failures belong to our completion accounting,
serialization ordering,
file ownership,
and invocation setup.

## Native cancellation controls are not ordinary event data

The extra `test:native:session-actions` run against `native-release-mmaT8k` fails during tree navigation.
The retained error is `SessionInputCaptureError` with cause
`SourceCollectionError: Run message data has an unsupported prototype`.
A diagnostic-only private copy identifies `field=signal`,
`[object AbortSignal]`,
constructor `AbortSignal`.
The qualified Pi source declares that top-level field on both `SessionBeforeTreeEvent`
and `SessionBeforeCompactEvent` in `packages/coding-agent/src/core/extensions/types.ts`.
Its `AgentSession.navigateTree` constructs the controller and passes its signal to the original event emission.

`native/contract/collector/run-prompt-custody/messages.mjs` intentionally accepts ordinary data only.
`loaded-event-inputs.mjs` applies that same copier to every native event field,
so a valid host control object is rejected at our capture boundary.
The passive observer preserves actual extension callbacks and stores the capture failure for assessment readers.
Consequently,
successful native navigation alone does not verify retained event evidence.
The committed `test:native:event-controls` first checks the original callbacks and signal identity,
then fails when reading that retained evidence.

### Actual Node signal behavior

The Node `v26.12.0` probe uses the installed runtime's own `internal/abort_controller` source.
Its `validateThisAbortSignal` checks a reflectable symbol property.
A shallow copy preserving the prototype and all own descriptors passes the intrinsic `aborted` getter.
That getter therefore does not prove constructor ownership.
The same probe confirms that calling the captured intrinsic getter bypasses a user-defined own `aborted` accessor,
and that default cancellation creates a `DOMException` reason rather than ordinary data.

The provenance boundary remains the original native event emission and callback positions,
not a signal's name or prototype.
The repair needs a narrowly scoped native-control representation while leaving ordinary message/tool copying strict.
`node26-abort-signal-probe.log` and `node26-abort-controller-source.js` in
`${HOME}/temp/agent/pi-mimo-preparation-failure-20261018` preserve the probe and deciding source.
The opaque-control repair passes the dedicated consumer,
all 14 real session-action modes with manager-evidence assertions,
the affected native matrix and six separately rebuilt guard mutations.
The canonical frontier retains the remaining lint and final configured-consumer queue.

## CLI startup classification and complete stderr

The extra native CLI process test initially reports
`AssessmentTransportError: AUTO_MODE_LLMGATEWAY_API_KEY is unavailable for the assessment request`
under the `reload-policy-relevance` logger tag.
Its selected primary model already has a synthetic credential,
but startup policy classification uses a separate credential and transport.
The fixture rejects every fetch,
so supplying the missing credential alone would still leave a fixture transport failure.
This is separate from the native cancellation-control defect.

The isolated child now has a synthetic classifier credential.
Its preload answers only the actual startup decisions route locally,
binds predicate answers to the received question names,
and rejects every other fetch.
A private exit receipt proves one local classification request in the passing run.
The actual CLI returns its requested RPC state,
makes zero external provider requests and closes with empty captured stderr.
`native-cli-local-classification.log` records that result on `native-release-dXedQJ`.

The parent also waits for child `close` before asserting stderr,
replacing its earlier wait for `exit`.
Node process termination and inherited stream closure are different boundaries.
The installed Node `v26.12.0` source emits `close` after its expected stream-close count is reached.
The positive-control probe gives a descendant the child's stderr pipe,
observes the child exit,
then releases the descendant's write.
Captured stderr is empty at `exit` and contains `late-fixture-stderr` at `close`.
The earlier wait could therefore miss a shutdown diagnostic.
The test retains the complete stream rather than filtering diagnostic lines.

`node-stdio-close-receipt.json`,
`node-stdio-close-probe.log` and `node26-child-process-source.js`
in `${HOME}/temp/agent/pi-mimo-preparation-failure-20261018` retain the causal probe and deciding source.

## Pi 1.2.0 streaming reveals full-prefix retention in the owned collector

### Symptom and scope

Node `v26.12.0` reports `ERR_ASSERTION` from the maintained native capacity consumer:
`Original native streaming snapshots retain 342565888 bytes, exceeding 33554432`.
The case emits 1,600 growing `message_update` inputs,
retaining complete original before/after snapshots while the final text has 203,200 UTF-16 code units.
This reproduces a consumer storage defect.
It does not establish the cause of a separate actual MiMo worker's
`FATAL ERROR: Ineffective mark-compacts near heap limit Allocation failed - JavaScript heap out of memory`.

### Source trace

Qualified Pi source commit `06a466935536f8c9bcacb2d5efc0fc021be1bdff`,
`packages/agent/src/agent-loop.ts:433`,
forwards each partial through a shallow outer message copy:

```ts
// Qualified Pi source: packages/agent/src/agent-loop.ts:433
partialMessage = event.partial;
context.messages[context.messages.length - 1] = partialMessage;
await emit({
  type: "message_update",
  assistantMessageEvent: event,
  message: { ...partialMessage },
});
```

The top-level objects differ;
the content still belongs to the same streaming partial.
`packages/coding-agent/src/core/agent-session.ts:1311` forwards those fields to extension callbacks:

```ts
// Qualified Pi source: packages/coding-agent/src/core/agent-session.ts:1311
const extensionEvent: MessageUpdateEvent = {
  type: "message_update",
  message: event.message,
  assistantMessageEvent: event.assistantMessageEvent,
};
await this._extensionRunner.emit(extensionEvent);
```

Our `package/pi-plugin/auto-mode/native/contract/collector/run-prompt-custody-v3/loaded-event-inputs.mjs:40`
copies each original callback phase before the borrowed input changes:

```js
// package/pi-plugin/auto-mode/native/contract/collector/run-prompt-custody-v3/loaded-event-inputs.mjs:40
const value=Object.freeze({value:copyToolMetadata({value:projectEventInput(input)})});
```

Before repair commit `0666bd3dcb`,
`loaded-pool.mjs:42` retains each distinct complete serialization:

```js
// package/pi-plugin/auto-mode/native/contract/collector/run-prompt-custody-v3/loaded-pool.mjs:42 at 9b4bb6258d
const serialized=contentFormat==='text'?value:JSON.stringify(value??[]);
let data=sourceDataByContent.get(serialized);
if(data===undefined){
  data=Object.freeze({content:serialized,sha256:createHash('sha256').update(serialized).digest('hex')});
  sourceDataByContent.set(serialized,data);
}
```

Identical content shares storage,
but every growing prefix is a distinct key and complete string.
The native event layout can contain the same growing content in both message fields.
This storage belongs to the owned collector;
Pi's supported streaming behavior is the trigger.

### Verification and working catalogs

Run the maintained consumers from a built package:

```sh
# Repository root; executed here inside a disposable 2-GiB, 2-CPU verification container.
mise run //package/pi-plugin/auto-mode:test:native:streaming-context-capacity
mise run //package/pi-plugin/auto-mode:test:native:input-content
```

The task limits the probe Node heap to 512 MiB and enables explicit collection.
The updated same-test comparison uses a private copy of `native-release-q83LlN`
and repaired `native-release-EWv4wy` without changing production publication during the checks.
The prior artifact retains 342,565,888 heap bytes and fails;
the repaired artifact retains 10,544,896 bytes and passes.
Original callback inputs,
distinct publications,
sampled earlier/later content and source hashes remain checked.
The logical serialized receipt count remains 366,034,224 bytes.

Working catalogs include repeated identical inputs,
repeated tool queries,
plain and JSON text,
escaped delimiters,
newlines,
Unicode and unpaired surrogate code units.
Failing catalogs include growing-prefix storage on the prior artifact,
copied content accessors on a different source,
and foreign getters,
which are rejected without invocation.

### Verified consumer repair and limits

`input-content.ts:77` interns exact chunk-identity sequences;
`input-content-chunks.ts:21` resets bounded chunk alignment at generated JSON string tokens.
The package-local [storage explanation](../../package/pi-plugin/auto-mode/doc/instruction-collection.md#retained-native-input-storage)
records the component,
tradeoffs and maintained tests.
The source still returns the complete original string through a pure producer-owned accessor.
`manager-source-growth.mjs:20` accepts only the exact registered source/accessor pair,
after the caller's original-owner authentication.
Other fields and foreign accessors retain their strict rejection.
Reads reconstruct strings as needed;
the store does not retain a flat string for every distinct prefix.

Private rebuilt mutations detect loss of JSON token alignment,
exact accessor identity and UTF-16 preservation.
Removing explicit chunk copies does not fail this runtime's capacity case;
that observation is not claimed as a detecting guard proof.
Digest-only equality is not used:
the exact-content test proves different unpaired surrogates can encode to identical UTF-8 bytes.
This is an encoding equivalence,
not a SHA-256 collision.

Deduplicating only identical complete strings does not address growing prefixes.
Removing transient RPC recorder output does not remove the host's actual callback inputs.
Dropping observations or merging original publications would change the required source contract.
The actual repaired MiMo session still needs its own completed editing and final replay verification.

### Upstream filing decision

1.  Upstream fault: no demonstrated Pi or Node defect;
    our complete-prefix cache owns the reproduced retention.
2.  Upstream fixability: no upstream change is required for the verified consumer repair.
3.  Supported use: Pi explicitly forwards streamed message updates to extension callbacks.
4.  Contribution policy: not assessed because no external contribution is proposed.
5.  Upstream movement: not assessed because no upstream defect is established.
6.  Prototype: the owned consumer repair is built and tested;
    an upstream patch is neither needed nor drafted.

The `.out-of-scope/` catalog was checked.
Its `pi-gpt55-long-context.md` concerns model-window changes,
not this MiMo collector-storage case.
There is no upstream filing artifact or duplicate-tracker claim:
no external issue is proposed for this owned defect.

## Pi native release retirement after missing cross-spawn, 2026-10-19

### Symptom and scope

[Issue #1786](https://github.com/Aquaticats/Monochromatic/issues/1786) reports Node's
`ERR_MODULE_NOT_FOUND: Cannot find package 'cross-spawn'` from the copied Pi distribution.
The active release was `native-release-kFkBoY`, with Pi 1.2.0 recorded in its source manifest.
The ordinary installed Pi package inspected for comparison was 1.3.0.
The user subsequently removed auto-mode's `dist`, `bin`, and `native` directories.
This section records the evidence captured before removal, not the state of a working replacement.

### Owned modification and dependency failure

The old launcher imported `../dist/native/cli.mjs` at `native/command.ts:13`.
That generated CLI loaded `./loader.mjs`, then `./pi/coding-agent/dist/cli.js`.
The source is preserved in the pre-rebuild worktree at
`~/temp/agent/pi-auto-mode-before-rebuild-20261019/package/pi-plugin/auto-mode/`.

`native/contract/integration/native-batch/prepare-agent.ts:113` replaced Pi's tool-dispatch implementation;
`native/loader.ts:51` mapped original Pi modules to generated replacements and line 161 installed the resolver hook:

```ts
// Historical native/loader.ts:161, in the preserved reference worktree.
export const nativeLoaderHooks: ModuleHooks = registerHooks(hooks);
```

The generated agent loop invoked the added `config.executeToolBatch` at line 390.
Thus `installedSourcesModified: false` described absence of in-place package writes only.
It did not mean Pi's implementation was unchanged.
The agents introduced this architecture without the user's approval.

The active release's `pi/coding-agent/node_modules` symlink targeted the absent directory
`node_modules/.pnpm/@earendil-works+pi-coding-agent@1.2.0/node_modules`.
Its child-process module imported the dependency unconditionally:

```js
// Historical dist/native/pi/coding-agent/dist/utils/child-process.js:2.
import crossSpawn from "cross-spawn";
```

The dangling link is observed evidence. The issue's theory about a particular install changing hoisting remains unproven.

### Verification and retirement

Before deletion, `mise run probe-pi-1786` in the private visual-review scratch directory
imported that child-process module directly without launching Pi or contacting a model.
It reproduced `ERR_MODULE_NOT_FOUND` naming `cross-spawn` and the active copied-release path.
The positive control imported the installed Pi child-process module and checked its `spawnProcess` export successfully.
These are import checks, not a complete configured CLI session test.
The probe requires the now-removed release and must not be rerun by rebuilding that release.

The remedy selected by the user is retirement of the copied/transformed Pi architecture.
An ordinary TypeScript plugin must use public Pi APIs.
Normal plugin output may use `dist`; private loading hooks and rebuilding Pi remain prohibited.
Adding another dependency link would not satisfy that requirement.
There is no claimed replacement-plugin runtime verification yet.

### Upstream filing decision

No upstream filing is appropriate: this is an owned relocated build with a broken dependency link.
The first filing condition, an upstream defect, is not met.
Upstream fixability, supported use case, contribution policy, willingness, and patch prototyping
do not change that conclusion and were not investigated for a proposed upstream report.
The existing repository issue is the incident record; no duplicate or external comment was sent.

## Pi 1.3.0 selective blocking misunderstood as atomic batch execution, 2026-10-19

### Symptom and corrected cause

The rebuild proposal incorrectly required rejecting every member of a parallel batch
when one member was denied.
This was an agent design error, not a Pi diagnostic or runtime failure.
The earlier prototype's whole-run abort was mistaken for a host constraint.

Pi's installed public `ToolCallEventResult` at
`package/pi-plugin/auto-mode/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/types.d.ts:1089`
offers per-event blocking:

```typescript
// Pi 1.3.0 public result, excerpt; the separate terminate hint is omitted.
export interface ToolCallEventResult {
    block?: boolean;
    reason?: string;
}
```

The same installed package's `dist/core/agent-session.js:333` forwards the current call identity:

```javascript
// Pi 1.3.0: dist/core/agent-session.js:333
return await runner.emitToolCall({
    type: "tool_call",
    toolName: toolCall.name,
    toolCallId: toolCall.id,
    ...(parentToolCallId ? { parentToolCallId } : {}),
    input: args,
});
```

Its `dist/core/extensions/runner.js:965` returns the blocking result for that event:

```javascript
// Pi 1.3.0: dist/core/extensions/runner.js:965
if (result.block) {
    return result;
}
```

### Consumer verification and limits

Run `mise run lint:types` and `mise run probe` in
`~/temp/agent/pi-public-plugin-probe-20261019/`.
The disposable public-only consumer uses a scripted provider and inert tools.
Its `results.json` records 20 asserted cases;
`results-18-cases.json` preserves the previous results.

The working mixed-outcome catalog contains `member-allow-a-block-b` and `member-block-a-allow-b`.
Both record the whole pending batch, return a blocking result for the denied ID,
execute only the allowed sibling, and reach the scripted final response without `ctx.abort()`.
The reversed order is the control for accidentally granting only the first member.

The rejected design catalog contains the assertion that a shared review group must execute atomically,
and the requirement to repropose allowed siblings as a new group.
Neither follows from the public contract or these consumer traces.
The previous prototype's abort after already returning approval to earlier hooks cannot establish that restriction.

No upstream workaround is required for selective pre-execution blocking.
The consumer must preserve call identity in its own decisions while retaining shared batch context.
The evidence does not establish arbitrary cancellation of already-running calls,
the complete replacement plugin's prepared-input capture,
or a chosen rule-answer attribution mechanism.
Those limits are recorded in the [working understanding](../../package/pi-plugin/auto-mode/doc/planning/rebuild-understanding.md).

### Upstream filing decision

1.  Upstream fault: none demonstrated; the incorrect atomic-batch assumption was ours.
2.  Fixability: supported per-event blocking already satisfies the exercised case.
3.  Supported use: the public event result explicitly exposes blocking.
4.  Contribution policy: not investigated because no external contribution is proposed.
5.  Upstream movement: no defect claim or duplicate-tracker claim is made.
6.  Prototype: the owned public consumer demonstrates both mixed-outcome orders;
    no Pi source was modified and no upstream patch is required.

There is nothing to file upstream for this correction.
