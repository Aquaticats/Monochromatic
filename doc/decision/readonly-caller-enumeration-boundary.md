# Readonly caller-enumeration boundary

Decision record for what counts as "reachable from outside the program" in
`callersAreEnumerable`,
 the shared completeness predicate behind `prefer-readonly-parameter-types`.

Status:
 decided.
Date:
 2026-08-07.

## Context

The returned-result discharge removes a receiver-opacity charge when a call's result is
returned outright and every caller substitutes for it.
`completeForeignBorrowedGraph` asks a related question when building its inbound closure.

Both used to read "every usage `getSignatureUsage` can enumerate resolves" as completeness.
That is satisfied by an exported callable with one in-program caller,
 while consumers outside
the program go unenumerated:

```ts
// package-a/src/copy-rows.ts
export function copyRows(rows: Row[],): Row[] {
  return rows.slice();
}

void copyRows([],);
```

An external consumer can then write `copyRows(rows,)[0].value = 1` with nothing attributing it.

The two mechanisms have opposite failure directions.
The ownership graph over-approximates and adds charges;
 the discharge under-approximates and
removes one.
An enumeration that may be missing callers is safe to trust only in the first direction,
 which
is why the shared predicate was strengthened rather than forked.

The guarded failure for this rule is minting a wrong read-only offer,
 so removing a charge is
the dangerous direction and adding one is the safe direction.

## Decision

A callable counts as reachable from outside when its source file's module surface exports it,
tested through `getExportsOfModule` and compared by declaration span.
Anything a module exports is refused;
 anything it does not is admitted.

This is deliberately an over-approximation.
A callable exported only to a sibling file inside this same program has fully enumerable
callers and is refused anyway.

## Measurement

Taken 2026-08-07 by instrumenting the predicate across a cold-cache workspace sweep,
 then
classifying each declaring file as public or internal.
A file counts as public when a `package.json` `exports` entry names it,
 when a public file
re-exports it,
 or when a wildcard subpath such as `"./ts/*": "./src/*"` publishes its tree.

- 377 verdicts:
 302 admitted,
 75 refused,
 across 40 distinct files.
- 28 of those files are reachable from a package export,
 so refusing them is correct.
- 12 are internal and carry 27 of the 75 refusals.

Twenty-seven refusals is the entire precision cost of the over-approximation.
Half sits in two files:
 `package/rolldown-plugin/import-attributes/src/ast-extract.ts` with five,
 and the eight in `package/pi-plugin/auto-mode/src/signals.ts`,
 deleted with the auto-mode extension pipeline on 2026-10-17 (`a3955898f1`).

The predicate admits four out of five callables it is asked about,
 so this did not disable the
discharge at workspace scale.
The provenance fixture suggested otherwise only because every callable in it is exported.

## Rejected alternatives

### Resolve package entry points and narrow to what they reach

The precise notion,
 and rejected on the measurement rather than on principle.

Twenty-seven refusals is the *upper* bound on what it could recover,
 and the real figure is
lower:
 a refusal becomes an offer only when no other path charges the parameter,
 and this
rule charges most of these shapes several ways.
The same redundancy leaves four of the discharge's seven guards unable to change any
diagnostic.

Against that,
 entry resolution has to follow conditional exports,
 wildcard subpaths and
re-export chains,
 and a bug in it fails **open**,
 which is the direction that mints wrong
offers.
Buying an unmeasured fraction of 27 refusals with a fail-open dependency is the wrong trade.

Reopen this if the internal share grows materially,
 or if a package stops publishing
wildcard subpaths and its internal helpers become a large refused population.

### Keep the old completeness notion for agreement

Rejected while making this change.
The argument was that the two mechanisms must agree about identical callables,
 which is
nearly right:
 they do have to share a notion,
 but the one they shared was sound for only the
charge-adding consumer.
Preserving agreement by letting both fail open is the wrong direction when one of them removes
a charge.

## Notes for whoever revisits this

The wildcard handling decides the answer.
A first classifier ignored `*` specifiers and reported 377 of 3054 workspace sources as
public;
 expanding wildcards gives 1483.
Read from the first pass,
 the same probe would have said most refusals were internal and
argued for building the machinery it argues against.

Any re-measurement must expand wildcard subpath exports before classifying,
 and should be
sanity-checked against a package known to declare one:
 `prefer-readonly-parameter-type` itself
declares `"./ts/*": "./src/*"`,
 so it has no internal files at all.

Full working record,
 including two claims made and withdrawn along the way:
`doc/planning/prefer-readonly-return-substitution.md`.
