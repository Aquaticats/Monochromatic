# @monochromatic-dev/agent-harness-shared-neuralwatt-listing

Harness-neutral NeuralWatt listing core shared by the DSH and pi NeuralWatt
plugins:
 listing-to-facts mapping,
 thinking-level expansion,
 the checked-in listing snapshot,
 and the bounded listing fetch.
It lives under `package/agent-harness-shared/` because two agent harnesses
consume it
(the DSH plugin `package/dsh-plugin/neuralwatt`
and the pi plugin `package/pi-plugin/neuralwatt`),
and it carries no Pi types and no DSH types:
 each harness projects the same facts
onto its own model profile shape.
The mapping moved here from `package/dsh-plugin/neuralwatt` on
2026-11-07,
keeping its behavior and its test cases.

## Pi dependency trust

PJM:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

This package takes the rule at its strongest reading:
 it has zero runtime dependencies,
zero Pi peers,
and zero DSH peers.

## Exports

The package root exports the built core;
workspace consumers use `/ts` for their source
(`doc/decision/workspace-ts-source-imports.md`).

- `mapNeuralWattListing`:
 listing entries to `NeuralWattModelFacts`,
 keeping every mapping rule the DSH plugin served
 (skip rules,
 display-name fallback,
 context and max-output fallback chains,
 input modalities,
 the offered efforts dict and its `off` rule).
- `expandThinkingLevelMap`:
 one offered efforts dict to the fully decided
`NeuralWattThinkingLevelMap`,
 declared levels carrying their wire spelling,
 declared `null` staying absent ("send nothing"),
 undeclared levels pinning `null`.
- `fetchNeuralWattListing`:
 the bounded `GET` of the public listing endpoint,
 with `NeuralWattListingError` as its one failure shape.
- `neuralWattListingSnapshot`:
 the checked-in listing snapshot,
 the offline fallback the mount fetch keeps serving.
- Constants:
 `NEURALWATT_ROUTE`,
 `NEURALWATT_API`,
 `NEURALWATT_BASE_URL`,
 `NEURALWATT_LISTING_URL`,
 `NEURALWATT_API_KEY_ENV`,
 `NEURALWATT_LISTING_TIMEOUT_MS`,
 `NEURALWATT_THINKING_LEVELS`,
 and `NEURALWATT_EFFORT_LEVEL_BY_SPELLING`.

## Snapshot

The snapshot is a reduced listing fetched from
`https://api.neuralwatt.com/v1/models`
on 2026-10-22,
keeping the fields the mapping reads.
Its `metadata.pricing` block was merged from the live endpoint on
2026-11-07T22:41:21-05:00
so a harness that prices models can,
and entries the endpoint has since dropped carry no pricing block.
The `refresh-snapshot` task regenerates the chunks from the live listing
and prints the drift it would change;
update the fetch-date header in `src/snapshot.ts` with it.

## Tasks

[`mise.toml`](mise.toml) defines this package's tasks.
Run one from this directory with `mise run <task>`.

Unit tests import package behavior from the built `dist` entry,
so run `build`
before `test:unit` after source changes,
or run `buildAndTest` for both.
