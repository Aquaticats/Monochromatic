# Oxlint 1.81.0 reports a void-return mismatch for Node's overloaded promisify adapter

## Symptom

The dependency-free CI command used `promisify(execFile)` to await Git.
Oxlint emitted:

```text
! typescript(strict-void-return): Value-returning function used in a context where a void function is expected.
const executeFile = promisify(execFile,);
```

Replacing it with a callback wrapper removed that diagnostic but introduced
`promise(prefer-await-to-callbacks)`.
No suppression or rule relaxation was retained.

## Source trace

Inspected on 2026-09-09:

- Oxlint `1.81.0`.
- `@types/node` `26.4.1`.
- `oxc-project/tsgolint` revision `86cffadb8a7285f7bb4a59538fa7b160a2dabd04`.
- Read-only clone at `~/temp/agent/tsgolint-scope-20260909`.

`node_modules/@types/node/child_process.d.ts:1022-1025`
declares a synchronous child handle and a separate completion callback:

```ts
// node_modules/@types/node/child_process.d.ts
function execFile(
    file: string,
    callback?: (error: ExecException | null, stdout: string, stderr: string) => void,
): ChildProcess;
```

The same declaration's `execFile.__promisify__` namespace at line 1081
contains Node's async adapter signatures,
including stdout and stderr result fields.

`node_modules/@types/node/util.d.ts:1143-1154`
contains both a custom adapter overload and generic void-returning callback overloads:

```ts
// node_modules/@types/node/util.d.ts
export function promisify<TCustom extends Function>(fn: CustomPromisify<TCustom>): TCustom;
export function promisify<TResult>(
    fn: (callback: (err: any, result: TResult) => void) => void,
): () => Promise<TResult>;
```

The lint source considers function signatures across overloads.
`internal/rules/strict_void_return/strict_void_return.go:152-180`
collects expected callback return types,
then lines 187-189 can report the passed function:

```go
// internal/rules/strict_void_return/strict_void_return.go
if utils.Some(argExpectedReturnTypes, isVoid) && utils.Every(argExpectedReturnTypes, isNullishOrAny) {
    reportIfNonVoidFunction(argNode)
}
```

`internal/rules/strict_void_return/options.go:7-10`
provides only the `allowReturnAny` option:

```go
// internal/rules/strict_void_return/options.go
// AllowReturnAny corresponds to the JSON schema field "allowReturnAny".
AllowReturnAny bool `json:"allowReturnAny,omitempty"`
```

There is no function-name allowlist in that options type.
Allowing `any` does not describe `ChildProcess` and would not express the intended adapter contract.

## Verified resolution

Specialize the adapter with Node's own declared input and output types:

```ts
// package/toml-edit/core.fuzz/src/ci-scope-git.ts
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const executeFile = promisifyExecFile(execFile,);
```

This is an assignment checked by TypeScript,
not a cast or an implementation of a replacement process runner.
Runtime execution still calls Node's native `promisify` implementation.
The type boundary names the actual `execFile` contract rather than leaving generic callback overloads in play.

Tradeoff:
an additional typed binding depends on Node's public declaration shape.
The return value retains Node's stdout,
stderr,
and rejection behavior.
No custom callback bridge or lint disable remains.

## Verification

The unspecialized call produced the diagnostic in the scoped lint task,
providing a positive control that the rule examined this file.
The specialized binding passed the same task with no warnings or errors.

```sh
# From the repository root
mise run //package/toml-edit/core.fuzz:lint:scope
mise run //package/toml-edit/core.fuzz:test:scope
```

The command's subprocess tests cover successful output,
unavailable Git,
unavailable commits,
and Git failing after commit validation.
The complete scope suite also passed with the deployed command running on Node 22.18.0.
See [the CI scope decision](../decision/toml-edit-ci-scope.md).

Working catalog:
the specialized adapter and its actual Git consumer.
Diagnostic catalog:
the unspecialized adapter and the rejected callback bridge.

## What does not solve the design problem

- A callback wrapper solely to erase the returned child handle:
  it adds an implementation boundary and triggers the callback-pattern rule.
- Renaming the callback parameter:
  this does not remove the added callback boundary.
- Casting the adapter:
  unnecessary because the assignment is type-compatible.
- Loosening the global void-return rule:
  unnecessary and broader than this call site.

## Upstream filing decision

The `.out-of-scope/` search for Oxlint and callback exemptions found no matching entry.
No upstream filing is proposed:
the adopted solution makes the actual callable contract explicit without changing upstream behavior.

1.  **Upstream fault:**
    not established;
    the overload set does contain void-returning callback contracts.
2.  **Upstream fixability:**
    overload handling could be investigated separately,
    but no upstream change is required by this resolution.
3.  **Supported use:**
    Node declares the specialized adapter contract used by the fix.
4.  **Contribution welcome:**
    not evaluated because no contribution is proposed.
5.  **Likely upstream action:**
    not evaluated;
    no change is requested.
6.  **Fix prototype:**
    the verified change is at the consumer boundary,
    not an upstream patch.

Nothing to add upstream from this investigation.
