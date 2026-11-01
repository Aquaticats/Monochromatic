# Host integration

## Purpose and interface

`native/contract/integration/native-host/bootstrap.ts` binds one configured Pi session
to `bindNativePolicyRuntime` in `native/contract/integration/native-batch/runtime.mjs`.
The runtime installs complete-group review before tool execution,
connects original request observation,
and returns inspection and detachment operations to the session owner.

The host is needed because independent extension tool callbacks do not provide this package's
original complete-group preparation and execution lifetime.
The qualified upstream transformation adds that execution seam while retaining Pi's actual tools,
extensions,
resource loader,
and model stream.

## Ownership and invariants

The original session and agent remain the execution owners.
The runtime binds only while idle and refuses an already occupied complete-group executor.
`src/native-runtime-hooks.ts` owns slot installation,
rollback,
and removal.
`src/native-query-observation.ts` preserves the observed original request.
Neither hook installation nor observing a request grants permission.

`native/contract/lifecycle/root-lifecycle-retirement-entry` connects observation to session retirement.
`native/contract/lifecycle/runtime-factory-scopes` retains constructor and cleanup ownership.
These lifetime checks prevent an answer for a disposed or replaced session from authorizing a later operation.
Their decomposition remains a consolidation target;
the responsibility is needed even if the current internal arrangement changes.

## Request size and passive observation

Long primary-model requests remain observable without a default byte ceiling.
The provider already owns the serialized request string;
observation retains its unchanged bytes,
byte count and digest without replacing the provider's sender.
Explicit `maxBodyBytes` values remain positive safe-integer limits.
An explicitly exceeded limit withholds evidence while preserving primary transport.

The previous default and constructor ceiling were both 1,048,576 bytes.
The live MiMo source worker exceeded that default and logged
`RequestObservationProfileError: Serialized request exceeds observation byte bound`.
The retained failure then produced
`Tool execution requires usable request evidence; primary chat transport remains available`
when reviewing later tool calls.
This is an auto-mode observation limit,
separate from the earlier preparation catch-all and Git protocol incidents.

The built `test:native:request-observation-size` catalog reproduces both default-size and configured-size failures
before the fix,
while its exact UTF-8 boundary and explicit smaller-limit controls pass.
Frozen `ebe81e3364` passes all five named size cases,
fresh types,
complete units,
host isolation and the real receipt consumer.
Frozen `9d3dae29d8` also passes fresh types and the five-file scoped lint with zero findings.
Four private mutants reintroduce the default ceiling,
remove the explicit-limit guard,
remove configuration validation,
or reject equality at the byte boundary.
Each builds successfully,
fails its named maintained guard,
then passes after source restoration and rebuilding.
The root source and live worker artifact remain unchanged throughout these probes.

The source worker resumes its complete session history on private `native-release-IFS6IL`.
A captured live checkpoint completes 24 tool calls without approval dialogs
and records 13 non-Luna serialized requests larger than the former ceiling,
from 1,146,627 to 1,279,294 bytes.
Its 24 completed Luna requests return HTTP 200.
The audit checks 493 questions,
passes required-data and excluded-data controls,
and finds no excluded thinking or raw command fields.
This is a live coding checkpoint,
not completion of the package's remaining lint or configured-consumer verification.

## Dependencies and alternatives

Pi is a third-party dependency.
The build transforms a qualified source copy rather than editing the installed dependency.
Tests use the same built host with disposable sessions and controlled provider responses.

Keeping the superseded extension flagger would introduce another approval path with different semantics.
It is removed from the maintained entry and package extension metadata.
The native command remains the execution path.

## Verification

`test:native:constructor`,
`test:native:host-lifecycle`,
and `test:native:dependency-facade` exercise construction,
disposal,
and maintained-export identity.
`test:native:decision-fallback` crosses the actual group hook.
The final consumer replay also loads configured extensions,
skills,
and tool catalogs through the installed command.

Task names use `mise run //package/pi-plugin/auto-mode:<task>`.
