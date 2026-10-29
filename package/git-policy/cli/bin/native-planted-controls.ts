/**
 Phase 1 of the native wrapper's mutation campaign: plant known guard removals and require the named control to fail.

 A campaign whose harness cannot observe a missing guard proves nothing, so these controls run before cargo-mutants
 and any unnoticed removal stops the campaign.
 */
// The bin scripts sit outside the package tsconfig's include list, so type-aware lint checks them in an inferred
// program that loads no ambient Node types; this reference loads them, so `process` and `node:` imports stay typed
// in this file on its own instead of depending on a sibling module that carries the same reference.
/// <reference types="node" />
import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { plantedCommitControls } from './native-planted-controls-commit.mjs';
import { plantedEntryControls } from './native-planted-controls-entry.mjs';
import { plantedLandingControls } from './native-planted-controls-landing.mjs';
import {
  createContainer,
  NativeVerificationError,
  runCommand,
} from './native-verification-process.mjs';

/** Source directory inside the tested image. */
const containerSource = '/work/package/git-policy/cli';

/**
 Guard removal paired with the test that must notice it.

 @typedef {{ name: string, file: string, from: string, to: string, failing: string }} PlantedControl
 */

/**
 Planted control plus what its container observed.

 @typedef {PlantedControl & { status: number | null, noticed: boolean }} PlantedControlResult
 */

/**
 Inputs every planted control shares.

 @typedef {{ base: string, limits: readonly string[], context: string, evidence: string }} PlantedControlSetting
 */

/**
 Guard removals with the control that must notice each one, in the order the campaign runs them.
 The tables sit in sibling modules by area.

 @type {readonly PlantedControl[]}
 */
const plantedControls = [
  ...plantedEntryControls,
  ...plantedCommitControls,
  ...plantedLandingControls,
];

/**
 Plant one guard removal in its own container and record whether the named control failed.

 @param {PlantedControlSetting & { control: PlantedControl, index: number }} request -
   shared inputs, the control, and its position, which names its planted source and log files
 @returns {Promise<PlantedControlResult>} control with its container's exit status and whether the named test failed
 */
async function runPlantedControl({
  base,
  limits,
  context,
  evidence,
  control,
  index,
}) {
  const original = await readFile(
    resolve(control.file),
    'utf8',
  );
  const occurrences = original.split(control.from)
    .length
    - 1;
  if (occurrences !== 1)
    throw new NativeVerificationError(
      `Planted control "${control.name}" matches ${String(occurrences)} times in ${control.file}; it must match exactly once.`,
    );
  const planted = join(
    context,
    `planted-${String(index)}.rs`,
  );
  await writeFile(
    planted,
    original.replace(
      control.from,
      control.to,
    ),
  );
  // Unit tests notice these removals first; `--no-fail-fast` keeps Cargo going so the named control also runs.
  await using container = await createContainer({
    args: [
      'create',
      ...limits,
      base,
      'cargo',
      'nextest',
      'run',
      '--offline',
      '--locked',
      '--all-targets',
      '--no-fail-fast',
    ],
  });
  await runCommand({
    command: 'podman',
    args: [
      'cp',
      planted,
      `${container.id}:${containerSource}/${control.file}`,
    ],
    capture: true,
  });
  const run = await runCommand({
    command: 'podman',
    args: [
      'start',
      '--attach',
      container.id,
    ],
    capture: true,
    allowFailure: true,
  });
  const output = `${run.stdout}\n${run.stderr}`;
  // nextest prints `FAIL [ 0.012s] <binary id> <test name>`, or `FAIL + LEAK [` when the failed test left a child
  // running; the name is the last token of the line.
  const noticed = (run.status !== 0)
    && output.split('\n')
      .some(function failedLine(line) {
        return line.trimStart()
          .startsWith('FAIL ')
          && line.includes('[')
          && line.trimEnd()
            .endsWith(` ${control.failing}`);
      });
  await writeFile(
    join(
      evidence,
      `planted-${String(index)}.log`,
    ),
    output,
  );
  console.log(`Planted control "${control.name}": ${noticed ? 'noticed' : 'NOT noticed'} by ${control.failing}`);
  const result = {
    ...control,
    status: run.status,
    noticed,
  };
  return result;
}

/**
 Planted-control results in table order. Each control starts only after the previous one returned and removed its
 container, so at most one bounded container runs at a time.

 @param {PlantedControlSetting} setting - inputs every planted control shares
 @returns {AsyncGenerator<PlantedControlResult>} one result per planted control
 */
async function* plantedControlResults(setting) {
  for (const [
    index,
    control,
  ] of plantedControls.entries()) {
    yield runPlantedControl({
      ...setting,
      control,
      index,
    });
  }
}

/**
 Plant each guard removal in its own container and require the named control to fail.
 The results file is written before any unnoticed removal throws, so a failing campaign keeps its evidence.

 @param {PlantedControlSetting} setting - tested image ID, container bounds, scratch and evidence directories
 */
export async function runPlantedControls(setting) {
  /** @type {PlantedControlResult[]} */
  const results = [];
  for await (const result of plantedControlResults(setting))
    results.push(result);
  await writeFile(
    join(
      setting.evidence,
      'planted-controls.json',
    ),
    `${JSON.stringify(
      results,
      null,
      2,
    )}\n`,
  );
  const unnoticed = results.filter(function isUnnoticed(result) {
    return !result.noticed;
  });
  if (unnoticed.length > 0)
    throw new NativeVerificationError(
      `Planted guard removals were not noticed: ${unnoticed.map(function controlName(result) {
        return result.name;
      })
        .join('; ')}. Mutation results cannot be trusted; inspect ${setting.evidence}.`,
    );
}
