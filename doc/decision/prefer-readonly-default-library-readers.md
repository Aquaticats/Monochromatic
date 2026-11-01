# Deriving the default-library readers that take their value as an argument

`Object.entries`,
 `Object.values`,
 `Object.keys` and `Object.hasOwn` read a caller-owned
value and return a fresh container.
 Until now every value handed to one took an opaque
boundary,
 because the collection authority answers `receiver.member(...)` and these put the
caller-owned value in argument position with a global as the receiver.
 They are now derived,
under a structural gate,
 and this records what was measured and what was refused.

## Why they were unreachable

The collection path is driven by the call's receiver.
 For `Object.entries(value)` that is
`Object`,
 which carries no caller origin,
 so nothing in that path ever looked at `value`.
The call then fell through to the opaque boundary like any unresolved callee.

Measured before the change,
 across the repository:
 73 findings named one of these and
nothing else,
 so they would clear entirely,
 and 25 more named one alongside another
unresolved call,
 so they would shrink.

## Where the channel claim lives, and where it was refused

The obvious home was `effect-member-channel-authority.ts`,
 the single permitted authority on
which user code a default-library member reaches.
 That was tried and its own probe refused
it,
 reporting `ObjectConstructor.hasOwn is not callable on the probe receiver`.

The probe was right,
 and the reason is worth keeping.
 That table is keyed by interface owner
and its evidence comes from calling each listed member on a receiver of that interface.
 A
static function reached through an argument cannot be probed that way,
 and merging the two
would have made one table's evidence mean two different things.
 The claim therefore lives
with the reader authority,
 which carries its own probe.

The claim itself is the one the member authority already makes for own-index access,
 in its
property-keyed form.
 Each reader performs `Get(value, key)` for own enumerable string keys,
which runs an accessor if the caller installed one.
 That is the channel `value.key` already
opens,
 and this rule treats a plain property read as a pure read,
 so admitting these widens
nothing that was not already assumed.

`effect-default-library-reader.unit.test.ts` establishes two things against a fully trapped
object rather than restating them:
 no reader reaches `set`,
 `deleteProperty`,
`defineProperty` or `setPrototypeOf`,
 and the key list is taken before any value is read,
 so
an accessor firing mid-walk cannot add or remove entries from the result.

## The operand must hold only data

`readonly-plain-data-invalid.ts` already required that enumerating a `ReadonlyMap` stays
fail-closed,
 and the audit that removed the plain-data catalog is why.
 Deriving
unconditionally would have reversed that decision,
 so the derivation is gated.

The gate is structural,
 and deliberately not a list of admitted types,
 because a list is the
thing the audit removed.
 An operand is refused when its type has a call signature,
 or when
any property it exposes is declared by an accessor,
 a method,
 or a declaration that cannot be
resolved.
 A plain record derives;
 a `ReadonlyMap`,
 whose properties are methods,
 does not.
That is exactly the split the fixture asserts,
 and the fixture now passes for a stated reason
rather than by coincidence.

## Results that carry the operand, and results that cannot

`keys` returns freshly built strings and `hasOwn` a boolean,
 so nothing of the operand comes
back and no later use can reach it.
 Those answer unconditionally.

`entries` and `values` hand back the operand's own property values inside a fresh array.
 The
array is fresh;
 the values in it are not.
 Those answer only when the result does not escape
the callable,
 reusing the escape discipline the collection discharge already uses,
 and
provenance runs from the result to the operand so a write through it is attributed.

## What it cost and gained

Repository-wide sweeps,
 compared by offer identity rather than count:

- Before:
   1859 findings and 23 offers.
- After:
   1837 findings and 27 offers.

So 22 findings cleared rather than the 73 the pre-gate estimate suggested,
 and the gap is the
data-only gate doing its job:
 most operands in this repository are not plain records.

Four offers are new,
 and each was checked rather than assumed.
 `navigate` and
`findRegressions` in `toml-edit/core`,
 `collectIgnoredKeys` in `pi-plugin/search-fetch`,
and `parseCodexRateLimitSnapshots` in `pi-plugin/statusline` all read their parameter and
never write through it,
 directly or through a helper:
 the bodies contain no property
assignment at all,
 and every other call they make still goes through ordinary analysis,
 since
the derivation discharges only the reader call itself.

No offer was withdrawn.
