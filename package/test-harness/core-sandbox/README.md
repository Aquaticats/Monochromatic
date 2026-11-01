# @monochromatic-dev/test-harness-core-sandbox

Disposable Sinon sandboxes and test-owned mocking implementation.
The [module-test runner](../core/README.md#stubs-and-spies) supplies attempt lifetime and execution context.

## Responsibilities

- `createSinon(config?)` creates an ordinary Sinon sandbox with synchronous and asynchronous disposal.
- `createOwnedSandbox({ owner, runtime })` wraps a sandbox with attempt guards and restoration ownership.
- `SandboxOwner` and `SandboxRuntime` describe the existing injected runner state.
- `SandboxOwnershipError` and `SandboxCleanupError` preserve ownership and combined-cleanup diagnostics.
- `NO_SANDBOX_OWNER` marks execution without a test-attempt owner.

The runner supplies contextual Node behavior;
this package does not create a test executor or install unhandled-rejection listeners.
Ordinary `createSinon` use does not acquire the runner's concurrent method isolation.

## Use

```ts
// consumer.ts
import { createSinon, } from '@monochromatic-dev/test-harness-core-sandbox/ts';

const target = { value: (): number => 1, };
{
  using sandbox = createSinon();
  sandbox.stub(target, 'value',).returns(2,);
}
// Scope exit restores target.value.
```

Workspace consumers use `/ts`.
The root entry resolves to the built neutral artifact.
Keep context-owned work inside its test attempt and await or cancel outstanding work before completion.

## Shared state

Method ownership retains the existing versioned global symbol
`@monochromatic-dev/test-harness-core/method-ownership/v1`.
It joins source and bundled copies in one realm.
The runner separately owns rejection observation and its async-context storage.

## Verification

```bash
# From the repository root.
mise run //package/test-harness/core-sandbox:buildAndTest
mise run //package/test-harness/core-sandbox:lint
mise run //package/test-harness/core:buildAndTest
mise run //package/test-harness/core:test:browser
```

Local tests exercise the built factories and disposal paths.
Runner tests retain concurrent ownership,
timeout,
cleanup-failure,
injection,
mixed-copy,
and browser integration coverage.
