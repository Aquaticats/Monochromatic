#!/usr/bin/env node
/**
 Start the native wrapper's parallel mutation campaigns together with the one run of the planted controls they share.

 This is the shared launcher (`package/mutation-test/cli-cargo-mutants/bin/mutation-parallel.ts`) plus the product's
 planted controls: for each image the units use, a proving record means no extra unit, and no record means one extra
 capped unit that runs only the controls, started at the same moment as the campaign units. See
 `bin/native-planted-parallel.ts` for the decision and `README.md` ("What a mutation campaign costs per mutant").
 */
import { mkdir } from 'node:fs/promises';
import { homedir } from 'node:os';
import { resolve } from 'node:path';

import {
  ensureUnitImages,
  formatWaitHint,
  imageReferenceOf,
  inspectionEnvironment,
  isContentAddressedId,
  parseLauncherOptions,
  planUnits,
  runCaptured,
  startUnit,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import {
  launcherImageArguments,
  nativeTestImageRepository,
} from './native-gate-image-tag.ts';
import { inspectUnit } from './native-planted-controls-unit.ts';
import { startPlantedControls } from './native-planted-parallel.ts';

/**
 Options of this launch, validated before anything starts.
 */
const options = parseLauncherOptions({
  argv: [
    ...launcherImageArguments(process.env),
    ...process.argv
      .slice(2),
  ],
  environment: process.env,
  home: homedir(),
  packageDirectory: resolve(
    import.meta.dirname,
    '..',
    '..',
    '..',
    'mutation-test',
    'cli-cargo-mutants',
  ),
});
/**
 Campaign units, planned before anything starts so a bad name stops the whole launch.
 */
const plans = planUnits(options);
/**
 Lines about the unit image tags, made from the gate image or checked, before anything starts: a unit whose image is
 missing dies at once, and the launcher would still exit 0.
 */
const imageLines = await ensureUnitImages({
  options,
  plans,
  ...(process.env
    .XDG_RUNTIME_DIR
    === undefined ? {} : { runtimeDirectory: process.env
      .XDG_RUNTIME_DIR, }),
});
for (const line of imageLines)
  console.log(line);
await mkdir(
  options.logDirectory,
  { recursive: true },
);
await mkdir(
  options.temporaryDirectory,
  { recursive: true },
);
/**
 What was decided about the planted controls, with the controls units already started.
 */
const planted = await startPlantedControls({
  options,
  plans,
  inspectImage: async function inspectTag(tag) {
    /**
     The Podman call, whose failure is an answer and not an error.
     */
    const inspected = await runCaptured({
      program: 'podman',
      environment: inspectionEnvironment({
        options,
        tag,
        ...(process.env
          .XDG_RUNTIME_DIR
          === undefined ? {} : { runtimeDirectory: process.env
            .XDG_RUNTIME_DIR, }),
      }),
      args: [
        'image',
        'inspect',
        imageReferenceOf({
          repository: nativeTestImageRepository,
          tag,
        }),
        '--format',
        '{{.Id}}',
      ],
      capture: true,
    });
    /**
     The image ID Podman printed.
     */
    const image = inspected.stdout
      .trim();
    return (inspected.status === 0) && isContentAddressedId(image)
      ? { image }
      : { failure: inspected.stderr
        .trim()
        || `status ${String(inspected.status)}` };
  },
  start: startUnit,
  inspectUnit,
});
for (const line of planted.lines)
  console.log(line);
/**
 Start lines of the campaign units, which start together; they carry the controls unit's name when one started.
 */
const startLines = (await Promise.all(planUnits({
  ...options,
  extraEnvironment: [
    ...options.extraEnvironment,
    ...planted.environment,
  ],
})
  .map(function startPlan(plan) {
    return startUnit({
      unit: plan.unit,
      systemdRunArguments: plan.systemdRunArguments,
    });
  })))
  .map(function lineOf(started) {
    return started.line;
  });
for (const line of startLines)
  console.log(line);
console.log(formatWaitHint([
  ...planted.units,
  ...plans.map(function unitOf(plan) {
    return plan.unit;
  }),
]));
