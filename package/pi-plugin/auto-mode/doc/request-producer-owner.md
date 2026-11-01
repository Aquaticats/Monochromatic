# Request and judgment ownership

The native host must distinguish a response from its actual producing request,
keep an active operation usable while another request starts,
and release completed evidence while historical SDK messages remain reachable.
An object claiming a role or carrying matching text cannot establish those relationships.
Producer ownership supplies identity evidence;
it grants no execution permission or human authority.

## Callers and interface

The native session binds its main-agent and auxiliary-summary stream consumers through
`native/contract/collector/request-producer-judgment-consumers/owner.ts`.
The route wrappers select the request kind internally.
The prepared-group handler reserves a judgment for the original main-agent response
and completes it after group cleanup.
Consumers read the latest retained diagnostic through the same owner.

The owner retains request,
stream,
response and consumer identities in private weak tables.
Response or stream reuse leaves an ambiguity tombstone instead of authenticating a new occurrence.
Public object fields cannot substitute for those original table entries.
Completion reads the already-owned response record even when later reuse invalidates its authority:
cleanup must preserve the primary operation failure.

## Internal responsibilities

- `owner.ts` owns bound consumers,
  request claims and exact response registration.
- `owner-judgments.ts` owns construction reservation,
  reentry protection,
  completion and retirement.
- `owner-types.ts` derives stream contracts from the installed SDK
  and construction inputs from the maintained judgment constructor.
- `owner-errors.ts` shares the original error constructor and reference guard.
  Separate error constructors would break identity checks across the internal modules.

Each response slot contains one discriminated state:
constructing,
active with its value,
failed with its original caught value,
or retired without either retained value.
Retirement replaces the whole state record.
The latest completed diagnostic stays readable until superseded;
an older active operation remains usable until its own completion.
Independent consumers retain independent latest slots.

Moving these relationships into callers would duplicate reservation,
cleanup and reuse handling across execution paths.
An indefinitely retained judgment per historical response preserves identity but retains completed evidence.
Deleting all historical records instead loses reuse detection.
Weak ownership records plus explicit evidence retirement preserve both requirements.

## Error-name lint exception

`RequestProducerError` preserves its existing standard message and cause
and derives `name` from the constructing subclass.
The typed split preserves that runtime behavior and the same exported constructor.

The installed Oxlint version is `1.88.0`.
The audited upstream source is the Oxc repository at tag `oxlint_v1.88.0`,
`crates/oxc_linter/src/rules/unicorn/custom_error_definition.rs`.
`check_class` accepts only the expected class-name string literal for the assignment or class field.
The rule explicitly rejects `this.name = this.constructor.name` in its fixtures.
Its unit rule type exposes no configuration fields,
configuration parser or allow-list.
Severity configuration cannot preserve this dynamic value without disabling the diagnostic.

Removing only the suppression in a private fixture produces exactly one warning:
`unicorn(custom-error-definition): The name property should be set to RequestProducerError`.
The receipt is `${HOME}/temp/agent/pi-mimo-preparation-failure-20261018/error-name-audit.log`.
The scoped disable surrounds only the constructor and its documentation.
Replacing the value with a literal changes inherited error-name behavior;
the current typing repair retains the existing behavior rather than making that separate design change.

## Verification and limits

At `3a21255dde`,
the nine semantic producer-lifetime cases and the actual native producer-retirement capacity guard pass.
Five separately rebuilt source-removal variants fail their intended assertions;
restoring the implementation passes both consumers again.
Those controls separately exercise handler completion,
retained-state clearing,
active-operation preservation,
consumer independence and failure-preserving cleanup.

Fresh combined JS/native builds and full types pass at that checkpoint.
Native stream routing and constructor binding still need explicit typed-consumer compatibility checks:
untyped production call sites do not establish those assignments through an ordinary type-check pass.
The complete lint backlog and configured full-history retention qualification remain tracked in
the [current handover](../../../../doc/handover/pi-auto-mode-current-frontier.md).
