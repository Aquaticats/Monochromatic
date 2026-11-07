/**
 The planted controls of a `native:mutation:parallel` launch: run beside the campaign units, never before them.

 Every campaign unit would plant the same controls over the same image, and the launcher starts the units together, so
 the controls would either repeat in every unit or hold the units back. Instead, for each distinct image the units use,
 the launcher looks for a record that proves the controls passed. With one, no controls unit starts and every campaign
 unit finds the record itself. Without one, the launcher starts one extra capped unit that runs only the planted controls
 and writes the record, and starts the campaign units at the same time without waiting. Those units skip the controls
 without proof and say which unit holds it, so their results hold only once that unit ends with a record.

 Nothing here waits and nothing locks. When the controls unit cannot be started, the campaign units get no skip and
 plant the controls themselves, as `native:mutation:scoped` does alone.
 */
import { randomUUID } from 'node:crypto';
import { join } from 'node:path';

import {
  buildLogBase,
  buildShellBody,
  buildSystemdRunArguments,
  buildUnitEnvironment,
  buildUnitName,
  CargoMutantsToolingError,
  isContentAddressedId,
  type LauncherOptions,
  type UnitPlan,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import { parseContainerLimitOptions } from './container-limits.ts';
import { campaignPrefix } from './native-gate-proof.ts';
import { passingPlantedControls } from './native-planted-controls-proof.ts';
import {
  isInvocationRunning,
  type UnitState,
} from './native-planted-controls-unit.ts';
import {
  type PlantedUnit,
  plantedUnitsEnvironmentVariable,
  plantedUnitsText,
} from './native-planted-controls-units.ts';

/**
 Image repository of the tested images the campaign units resolve their tags in.
 */
export const nativeTestImageRepository = 'localhost/git-policy-native-test';

/**
 Mise task that runs only the planted controls over one image and records their pass.
 */
export const plantedTask = '//package/git-policy/cli:native:mutation:planted';

/**
 Environment entry a person may pass through `--setenv` to skip the planted controls without proof.
 */
export const skipPlantedEntry = 'NATIVE_MUTATION_SKIP_PLANTED=1';

/**
 Environment variables the launcher sets for the units itself, which a `--setenv` must not replace: the image tag decides
 which image a unit runs and so which image the launcher inspected for it, and the coordination variable says which
 controls unit holds the proof.

 @param options - the launcher options of this launch

 @returns the variable names

 @example
 ```ts
 authoritativeVariables(options);
 ```
 */
export function authoritativeVariables(options: LauncherOptions): readonly string[] {
  return [
    options.tagEnvironmentVariable,
    plantedUnitsEnvironmentVariable,
  ];
}

/**
 The environment the launcher inspects images with: the assignments a unit with that tag gets (the tag, the person's
 `--setenv` entries, and the launcher's own `TMPDIR`, `PATH` and `HOME` last), and nothing inherited from the launcher's
 shell, so it looks in the container storage the units will use. Only `XDG_RUNTIME_DIR`, which rootless Podman needs and
 the user service manager sets to the same value for a unit, is taken over.

 @param options - the launcher options of this launch

 @param tag - the image tag being inspected

 @param runtimeDirectory - the launcher's `XDG_RUNTIME_DIR`, when it has one

 @returns the variables, later assignments replacing earlier ones of the same name

 @example
 ```ts
 inspectionEnvironment({ options, tag: 'w50', runtimeDirectory: '/run/user/1000' });
 ```
 */
export function inspectionEnvironment({
  options,
  tag,
  runtimeDirectory,
}: {
  readonly options: LauncherOptions;
  readonly tag: string;
  readonly runtimeDirectory?: string;
}): Readonly<Record<string, string>> {
  return Object.fromEntries([
    ...(runtimeDirectory === undefined ? [] : [`XDG_RUNTIME_DIR=${runtimeDirectory}`]),
    ...buildUnitEnvironment({
      options,
      tag,
    }),
  ].map(function pairOf(entry) {
    /**
     Position of the first equals sign, which ends the name.
     */
    const separator = entry.indexOf('=');
    return [
      entry.slice(
        0,
        separator,
      ),
      entry.slice(separator + 1),
    ];
  }));
}

/**
 Evidence directory of the campaigns, relative to the working tree the units start in.
 */
const evidenceSegments = [
  'package',
  'git-policy',
  'cli',
  'target',
  'verification',
] as const;

/**
 How the launcher learns the full ID of the image a tag names.
 */
export type ImageInspector = (tag: string) => Promise<
  | { readonly image: string }
    | { readonly failure: string }
  >;

/**
 How the launcher starts one unit; `started` is false when `systemd-run` refused it.
 */
export type UnitStarter = (call: {
  readonly unit: string;
  readonly systemdRunArguments: readonly string[];
}) => Promise<{
  readonly started: boolean;
  readonly line: string
}>;

/**
 What was decided about the planted controls of one image: the line to print, and the controls unit when one started.
 */
type ImageDecision = {
  readonly line: string;
  readonly started?: PlantedUnit;
};

/**
 An image and the first tag that names it.
 */
type ImageTag = {
  readonly image: string;
  readonly tag: string;
};

/**
 Characters of the image ID that name its controls unit.
 */
const imageNameLength = 12;

/**
 What the launcher decided about the planted controls of a launch.
 */
export type PlantedLaunch = {
  /**
   Lines to print, one per image (or per unresolved tag), saying what was decided and why.
   */
  readonly lines: readonly string[];
  /**
   Names of the controls units that started, for the wait hint.
   */
  readonly units: readonly string[];
  /**
   `NAME=value` entry for the campaign units, absent when no controls unit started.
   */
  readonly environment: readonly string[];
};

/**
 Read the `--image <full image ID>` option the launcher gives the controls-only task.

 @param options - the task's arguments after the container limit options were taken out

 @returns the image ID as the only element, or no element when the option is absent

 @throws CargoMutantsToolingError when the option has no value or a value that is not a full content-addressed image ID,
 or when another argument is left

 @example
 ```ts
 imageOptionOf(['--image', 'a'.repeat(64)]);
 ```
 */
export function imageOptionOf(options: readonly string[]): readonly string[] {
  if (options.length === 0)
    return [];
  /**
   The option and its value.
   */
  const [name, image, ...surplus] = options;
  if ((name !== '--image') || (image === undefined)
    || (!isContentAddressedId(image))
    || (surplus.length > 0))
    throw new CargoMutantsToolingError(`The controls-only task takes --memory, --cpus, --pids-limit and one --image <full image ID>, got ${JSON.stringify(options)}.`);
  return [image];
}

/**
 Plan the unit that runs the planted controls over one image.

 @param options - the launcher options of this launch

 @param image - full ID of the image

 @param tag - a tag that names the image, kept in the unit's environment for its log; the unit runs `image`

 @param taken - unit names of the campaign units, which the controls unit's name must not equal

 @param launchToken - a token unique to this launch, which makes the unit name unique to it: a unit another launch started,
 or one of an earlier launch that ended, can never be taken for this launch's

 @returns the unit, its first log file, and the `systemd-run` arguments

 @throws CargoMutantsToolingError when the name equals a campaign unit's or is invalid

 @example
 ```ts
 planControlsUnit({ options, image, tag: 'w50', taken: [], launchToken: 'a1b2c3d4' });
 ```
 */
export function planControlsUnit({
  options,
  image,
  tag,
  taken,
  launchToken,
}: {
  readonly options: LauncherOptions;
  readonly image: string;
  readonly tag: string;
  readonly taken: readonly string[];
  readonly launchToken: string;
}): {
  readonly unit: string;
  readonly log: string;
  readonly systemdRunArguments: readonly string[];
} {
  /**
   Stand-in for the file the name is built from: the controls are per image, not per file.
   */
  const name = `planted-${image.slice(
    0,
    imageNameLength,
  )}-${launchToken}`;
  /**
   Systemd unit name of the controls unit.
   */
  const unit = buildUnitName({
    prefix: options.unitPrefix,
    label: options.label,
    file: name,
  });
  if (taken.includes(unit))
    throw new CargoMutantsToolingError(`The planted-controls unit name ${JSON.stringify(unit)} equals a campaign unit's; rename the label or the prefix.`);
  /**
   Log path prefix; the unit runs once, so its log is the first attempt's.
   */
  const logBase = buildLogBase({
    directory: options.logDirectory,
    prefix: options.logPrefix,
    label: options.label,
    file: name,
  });
  return {
    unit,
    log: `${logBase}.1`,
    systemdRunArguments: buildSystemdRunArguments({
      unit,
      memoryMax: options.unitMemoryMax,
      cpuQuota: options.unitCpuQuota,
      tasksMax: options.unitTasksMax,
      workingDirectory: options.root,
      environment: buildUnitEnvironment({
        options,
        tag,
      }),
      body: buildShellBody({
        logBase,
        attempts: 1,
        retryMarker: options.retryMarker,
        gateSlotPath: options.gateSlotPath,
        gateSlotArguments: options.gateSlotArguments,
        task: plantedTask,
        taskArguments: [
          // The unit runs this image ID, never the tag, which may be moved while the unit waits for a slot.
          '--image',
          image,
          ...(options.memory === undefined ? [] : [
            '--memory',
            options.memory,
          ]),
          ...(options.cpus === undefined ? [] : [
            '--cpus',
            options.cpus,
          ]),
        ],
      }),
    }),
  };
}

/**
 Decide, for every image the campaign units use, whether the planted controls are proved, run beside the units, or left
 to each unit, and start the controls units that are needed.

 @param options - the launcher options of this launch

 @param plans - the campaign units, whose tags name the images

 @param inspectImage - resolves a tag to the full image ID

 @param start - starts one unit

 @param inspectUnit - asks systemd about a started unit, so the launcher can name the invocation it started

 @param launchToken - a token unique to this launch for the controls units' names: 122 random bits (a UUID without hyphens)
 unless a test gives one. Two launches drawing the same token is not a realistic event, so the name is the launch's own and the
 lookup by name right after `systemd-run` returns can only find the unit this launch started; no reservation is needed.

 @returns the lines to print, the controls units that started, and the environment entry for the campaign units

 @throws CargoMutantsToolingError when a controls unit's name equals a campaign unit's

 @example
 ```ts
 await startPlantedControls({ options, plans, inspectImage, start, inspectUnit });
 ```
 */
export async function startPlantedControls({
  options,
  plans,
  inspectImage,
  start,
  inspectUnit,
  launchToken = randomUUID()
    .replaceAll(
    '-',
    '',
    ),
}: {
  readonly options: LauncherOptions;
  readonly plans: readonly UnitPlan[];
  readonly inspectImage: ImageInspector;
  readonly start: UnitStarter;
  readonly inspectUnit: (unit: string) => Promise<UnitState>;
  readonly launchToken?: string;
}): Promise<PlantedLaunch> {
  /**
   The variables the launcher owns that a `--setenv` names.
   */
  const overridden = authoritativeVariables(options)
    .filter(function isNamed(name) {
      return options.extraEnvironment
        .some(function setsIt(entry) {
          return entry.startsWith(`${name}=`);
        });
    });
  if (overridden.length > 0)
    throw new CargoMutantsToolingError(`--setenv cannot set ${overridden.join(', ')}: the launcher sets ${overridden.length === 1 ? 'it' : 'them'} for each unit, from the image it inspected, so an override would run another image than the one the planted controls are proved for.`);
  if (options.extraEnvironment
    .includes(skipPlantedEntry))
    return {
      lines: [`Planted controls: ${skipPlantedEntry} was given, so no controls unit started and the campaign units skip the controls without proof.`],
      units: [],
      environment: [],
    };
  /**
   Memory and process limit the campaign units run with, which a record must have been made at or under.
   */
  const { limits } = parseContainerLimitOptions({
    options: [
      ...(options.memory === undefined ? [] : [
        '--memory',
        options.memory,
      ]),
      ...(options.cpus === undefined ? [] : [
        '--cpus',
        options.cpus,
      ]),
    ],
    acceptPidsLimit: true,
  });
  /**
   Each unit's tag with what resolving it gave.
   */
  const resolved = await Promise.all(plans.map(async function resolveTag(plan) {
    return {
      tag: plan.tag,
      answer: await inspectImage(plan.tag),
    };
  }));
  /**
   First tag of every distinct image ID, in unit order.
   */
  const images = new Map<string, string>();
  /**
   Lines about tags whose image did not resolve; those units plant the controls themselves.
   */
  const unresolved: string[] = [];
  for (const {
    tag,
    answer
  } of resolved) {
    if ('failure' in answer)
      unresolved.push(`Planted controls: the image of tag ${tag} did not resolve (${answer.failure}); that campaign unit plants the controls itself.`);
    else if (!images.has(answer.image))
      images.set(
        answer.image,
        tag,
      );
  }
  /**
   Decision and started unit of every image.
   */
  const decided = await Promise.all(Array.from(
    images,
    function tagOf(entry: readonly [
      string,
      string
    ]): ImageTag {
      return {
        image: entry[0],
        tag: entry[1],
      };
    },
  )
    .map(async function decideImage({
      image,
      tag,
    }: ImageTag): Promise<ImageDecision> {
    /**
     Path of a record that proves the controls for this image, as the only element, or none.
     */
      const [record] = await passingPlantedControls({
        evidenceRoot: join(
          options.root,
          ...evidenceSegments,
        ),
        evidencePrefix: campaignPrefix,
        image,
        limits,
      });
      if (record !== undefined)
        return {
          line: `Planted controls for image ${image}: proved by ${record}; no controls unit started, and each campaign unit logs that record.`,
        };
    /**
     The controls unit for this image.
     */
      const controls = planControlsUnit({
        options,
        image,
        tag,
        taken: plans.map(function unitOf(plan) {
          return plan.unit;
        }),
        launchToken,
      });
    /**
     Whether `systemd-run` took the unit, and its start line.
     */
      const started = await start({
        unit: controls.unit,
        systemdRunArguments: controls.systemdRunArguments,
      });
      if (!started.started)
        return {
          line: `Planted controls for image ${image}: no record proves them and unit ${controls.unit} did not start (${started.line}); the campaign units plant the controls themselves.`,
        };
      /**
       The invocation systemd started under that name. The campaign units must defer to this one and no other, so a unit that
       ended at once, or a name another launch took, is not offered to them.
       */
      const unitState = await inspectUnit(controls.unit);
      return isInvocationRunning({
        state: unitState,
        invocation: unitState.invocation,
      })
        ? {
          line: `Planted controls for image ${image}: no record proves them, so unit ${controls.unit} (invocation ${unitState.invocation}) runs them beside the campaign units (log ${controls.log}). The campaign units skip them without proof; their results hold only once that unit ends with a planted-controls-passed.json, and its log ends with the record path or with the control that was not noticed.`,
          started: {
            image,
            unit: controls.unit,
            log: controls.log,
            invocation: unitState.invocation,
          },
        }
        : {
          line: `Planted controls for image ${image}: unit ${controls.unit} started but is already ${unitState.state === '' ? 'gone' : unitState.state}, so the campaign units plant the controls themselves; see ${controls.log}.`,
        };
    }));
  /**
   The controls units that started, with the image each runs.
   */
  const running = decided.flatMap(function startedOf(decision) {
    return decision.started === undefined ? [] : [decision.started];
  });
  return {
    lines: [
      ...unresolved,
      ...decided.map(function lineOf(decision) {
        return decision.line;
      }),
    ],
    units: running.map(function unitOf(entry) {
      return entry.unit;
    }),
    environment: running.length === 0 ? [] : [`${plantedUnitsEnvironmentVariable}=${plantedUnitsText(running)}`],
  };
}
