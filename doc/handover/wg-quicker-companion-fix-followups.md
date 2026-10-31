# Handover: wg-quicker companion fix follow-ups

## Purpose

`wg-quicker up` failed with `BypassRouteError: wg-quicker-exempt is unavailable`
although the companion was built.
The cause is fixed,
tested,
and recorded in
`doc/troubleshooting/wg-quicker-exempt-companion-pre-move-paths.md`.
This handover lists what landed,
what to inspect,
and the follow-ups that are deliberately not done.

## What landed

- `34df734099` fixes the three pre-move layout literals in
  `package/wg/cli-quicker/src/application-exemption-command.ts`.
- `03ca61953c` adds one test per discovery branch of
  `resolveApplicationExemptionCommand`.
- `aaa3c76bf0` clears six pre-existing lint findings in
  `package/wg/cli-quicker/src/scratch-parent.ts`.
- `db02fa4c92` moves the changed-config down test mark off netavark's reserved
  `0x2000` bit (`ExemptMark = 9999` to `200`).
- `1384f4e850` adds the two troubleshooting docs.

## What to inspect

- `mise run //package/wg/cli-quicker:buildAndTest` (green,
  expected suite names).
- `mise run //package/wg/cli-quicker:lint` (`Found 0 warnings and 0 errors.`).
- `mise run //package/wg/cli-quicker:test:integration:bypass`
  (`wg-quicker bypass integration passed`).
- `node src/application-exemption-command.unit.test.ts` inside
  `package/wg/cli-quicker` (ten case names,
  all `PASS`).

Not run:
`mise run //package/wg/cli-quicker:test:integration:route`
(endpoint routing suite,
unrelated to the resolver change).

## Follow-ups left open

1. Logger retention hygiene.
   Root-owned debris in `node_modules/.monochromatic` makes every later sweep
   report `log retention could not stamp ... EACCES` and skip pruning.
   Two first-party changes are recorded in
   `doc/troubleshooting/logger-retention-sweep-root-owned-debris.md`
   under "Durable fix not landed":
   privileged runs should log to a root-owned destination,
   and the sweep should prune around foreign-owned entries.
   Owning package:
   `package/logger/core`.
1. AGENTS.md rename tightening,
   pending the user's accept or veto in this session's question.
   Proposed new rule next to `SGD`:

   ```text
   RNL: Rename moves runtime path literals and layout constants with the
   directories, then runs each touched package's `buildAndTest` before
   committing; a red suite in a moved package is a stale consumer.
   ```

   Evidence it would have caught this incident:
   `113633b259` renamed the directories,
   left the literals stale,
   and its promised follow-up verification never ran
   `//package/wg/cli-quicker:buildAndTest`,
   so the suite sat red from 2026-10-29 to `34df734099`.
   Tag `RNL` is unique across `AGENTS.md`,
   skills,
   and package docs,
   and absent from `forbidden-strings.append.txt` and
   `forbidden-strings.append.local.txt`.
   Applying it needs the `writing-for-agents` skill loaded.
1. Re-running `wg-quicker up fr-par-fr1` against the real config,
   pending the user's choice in this session's question.
   Nothing on the host was mutated by the fix work;
   the tunnel state is whatever the user last left.

## How to respond

If the user accepts `RNL`,
edit `AGENTS.md`,
run the forbidden-strings check,
and commit as `docs(AGENTS.md): ...`.
If the user asks for the tunnel,
run `wg-quicker up fr-par-fr1` from the repository root and report its output,
with `wg-quicker down fr-par-fr1` as the recovery if activation fails midway.
