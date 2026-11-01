# `nano-spawn` 2.1.0 inherits a value omitted by an absent-variable fixture

## Symptom

On Node `26.12.0`,
the auto-mode network unit passes with a clean parent environment
but fails when its parent supplies a nonempty `PI_AUTO_MODE_CLI_FIXTURE_REQUESTS`.
The test expects guard installation to reject an absent receipt path.
Instead,
the child reaches its deliberately forbidden probe:

```text
# package/pi-plugin/auto-mode/src/native-process-network.unit.test.ts: inherited-parent failure
Error: Unexpected fixture entry
```

The module-test assertion reports that stderr does not contain
`Native CLI fixture requires its disposable request receipt path`.
This is an owned fixture defect,
not an observed `nano-spawn` or Node defect.

## Source identity and root cause

The installed package is `nano-spawn` `2.1.0`.
The read-only upstream checkout at
`${HOME}/temp/agent/nano-spawn-inheritance-20261018`
uses release tag `v2.1.0`,
resolved commit `cc231e2c7b1e434a96f25f907ca2cb2f7c596e90`.
The tag reference itself is an annotated tag object,
not that commit.
Installed and cloned `source/options.js` have SHA-256
`b8b0e1c4cf652c88bdec37fe61f950cfbda8304a13f8e200e6e465148a6cdd34`.

At owned source commit `6c1dc11ea3`,
`package/pi-plugin/auto-mode/src/native-process-network.unit.test.ts:116`
passes the supplied object directly:

```typescript
// package/pi-plugin/auto-mode/src/native-process-network.unit.test.ts:116 at 6c1dc11ea3
env: environment,
```

The invocation intended to exercise absence supplies only the fixture-child marker.
Omission from this object does not replace the whole child environment.
Upstream `source/options.js:16` merges parent and supplied values:

```javascript
// nano-spawn/source/options.js:16 at v2.1.0
const env = envOption ? {...process.env, ...envOption} : undefined;
```

`source/options.js:22` passes that merged object onward;
`source/spawn.js:20` supplies the options to Node's process creation:

```javascript
// nano-spawn/source/options.js:22 and source/spawn.js:20 at v2.1.0
env: preferLocal ? addLocalPath(env ?? process.env, cwd) : env,
const instance = spawn(file, commandArguments, options);
```

The type documentation in `source/index.d.ts:76` explicitly says other variables are inherited.
Its relevant excerpt is:

```text
# nano-spawn/source/index.d.ts:76 at v2.1.0, prose excerpt
Other environment variables are inherited from the current process
```

The nonempty parent value therefore reaches the child,
the guard correctly accepts its setup path,
and the forbidden probe throws.
The owned guard's setup condition at
`package/pi-plugin/auto-mode/native/contract/integration/native-host/process-network-guard.ts:32`
in `6c1dc11ea3` distinguishes the genuinely absent and empty cases:

```typescript
// package/pi-plugin/auto-mode/native/contract/integration/native-host/process-network-guard.ts:32 at 6c1dc11ea3
if ((receiptPath === undefined) || (receiptPath === ''))
  throw new NativeCliFixtureNetworkError('Native CLI fixture requires its disposable request receipt path');
```

## Verification catalogs

The private harness freezes all tracked auto-mode package files at `68e7ce538a`
and runs against corrected native artifact `native-release-LytMAt`.
Both invocations use disposable homes,
private logs,
two CPUs and 2 GiB memory.

```shell
# doc/troubleshooting/nano-spawn-environment-inheritance.md: retained reproduction harness
cd "${HOME}/temp/agent/pi-mimo-preparation-failure-20261018"
mise run snapshot-verification 68e7ce538a test:native:process-network
mise run snapshot-verification 68e7ce538a test:native:process-network inherited-receipt
```

The working catalog includes the clean-parent absent path,
the explicitly empty path,
the valid private receipt path and dispatcher override retention.
The complete network unit exits zero for the clean-parent invocation.

The failing catalog adds only a nonempty parent receipt variable.
The absent-path assertion fails with the forbidden probe's actual error.
`cli-corrected-unit-normal.log` and `cli-corrected-unit-inherited-receipt.log`
retain the paired output.

## Verified repair and workaround

The controlled clean-parent invocation passes without changing the library.
Its limitation is that it leaves the test dependent on caller state.
Commit `a6417c7303` repairs the owned child-environment declaration:

```typescript
// package/pi-plugin/auto-mode/src/native-process-network.unit.test.ts:119 at a6417c7303
env: {
  PI_AUTO_MODE_CLI_FIXTURE_REQUESTS: undefined,
  ...specificEnvironment
},
```

The explicit undefined value masks the inherited variable.
Spreading the case-specific environment afterward preserves the separate empty-string case.
The fix neither mutates the parent environment nor changes the library.
Its tradeoff is that each negative fixture must explicitly mask any parent variable it requires absent.

Independent frozen-source runs repeat both catalog invocations with commit `a6417c7303`.
Both exit zero against `native-release-LytMAt`:
`environment-mask-independent-clean.log` and `environment-mask-independent-inherited.log`.
The original contaminated-parent failure remains the positive control for the harness.

## What does not work

Passing an object without the key does not remove an inherited value.
Treating a successful clean-parent run as proof of environment isolation also fails:
the paired contaminated-parent run disproves that conclusion.

## Upstream filing decision

- Fault: the owned test assumes replacement while the documented option augments its parent environment.
- Fixability: the consumer controls its child environment;
  no upstream change is needed to address this failure.
- Supported use: variable inheritance is explicitly documented and requested in
  [issue 11](https://github.com/sindresorhus/nano-spawn/issues/11).
  Its full body and owner comment confirm the chosen default.
- Contribution policy: not assessed for a patch because no upstream defect or contribution is proposed.
- Maintainer disposition: issue 11 explicitly approves the observed augmentation behavior.
- Prototype: no upstream prototype is justified;
  the owned fix passes clean and contaminated-parent verification.

The `.out-of-scope` inventory was checked;
no Nano-specific filing exemption determines this decision.
Tracker searches for `environment` include closed issue 11 and merged platform-related pull requests.
There is nothing additive to report about the library's intended contract.
No issue,
comment or upstream patch is filed or drafted.
