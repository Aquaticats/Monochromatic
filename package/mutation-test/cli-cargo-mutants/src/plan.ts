import {
  buildImageTag,
  buildLogBase,
  buildUnitName,
} from './names.ts';
import type { LauncherOptions } from './options.ts';
import { buildShellBody } from './shell-body.ts';
import { buildSystemdRunArguments } from './systemd-run-arguments.ts';
import { buildTaskArguments } from './task-arguments.ts';
import { CargoMutantsToolingError } from './error.ts';

/**
 One file's campaign, ready to start.
 */
export type UnitPlan = {
  readonly file: string;
  readonly unit: string;
  readonly logBase: string;
  readonly tag: string;
  readonly systemdRunArguments: readonly string[];
};

/**
 Reject a second file whose derived unit name equals an earlier one's, because the two campaigns would share a unit.
 The log name keeps more characters than the unit name, so files with distinct unit names never share a log.

 @param seen - names already taken, mapped to the file that took them
 
 @param kind - what the name is, for the message
 
 @param name - derived name under test
 
 @param file - file that derived it
 
 @throws CargoMutantsToolingError when another file already holds the name
 */
function claimName({
  seen,
  kind,
  name,
  file,
}: {
  readonly seen: Map<string, string>;
  readonly kind: string;
  readonly name: string;
  readonly file: string;
}): void {
  /**
   File that took the name first, if any.
   */
  const holder = seen.get(name);
  if (holder !== undefined)
    throw new CargoMutantsToolingError(
      `${JSON.stringify(holder)} and ${JSON.stringify(file)} give the same ${kind} ${JSON.stringify(name)}; rename one or run them in separate launches.`,
    );
  seen.set(
    name,
    file
  );
}

/**
 Plan the unit of every file without starting anything.

 Each unit gets its own image tag, log base and unit name, so the campaigns never share an image, a log or a unit.

 @param options - validated launcher options
 
 @returns one plan per file, in argument order
 
 @throws CargoMutantsToolingError when two files derive the same unit name, log base or image tag, or a name is invalid

 @example
 ```ts
 planUnits(options);
 ```
 */
export function planUnits(options: LauncherOptions): readonly UnitPlan[] {
  /**
   Unit names taken so far.
   */
  const units = new Map<string, string>();
  return options.files
    .map(function planUnit(
      file,
      index
    ) {
    /**
     Systemd unit name of this file's campaign.
     */
      const unit = buildUnitName({
        prefix: options.unitPrefix,
        label: options.label,
        file,
      });
    /**
     Log path prefix of this file's attempts.
     */
      const logBase = buildLogBase({
        directory: options.logDirectory,
        prefix: options.logPrefix,
        label: options.label,
        file,
      });
      claimName({
        seen: units,
        kind: 'unit name',
        name: unit,
        file
      });
    /**
     Image tag that keeps this campaign's image apart from the others'.
     */
      const tag = buildImageTag({
        template: options.tagTemplate,
        label: options.label,
        index,
      });
    /**
     Shell program the unit runs.
     */
      const body = buildShellBody({
        logBase,
        attempts: options.attempts,
        retryMarker: options.retryMarker,
        gateSlotPath: options.gateSlotPath,
        gateSlotArguments: options.gateSlotArguments,
        task: options.task,
        taskArguments: buildTaskArguments({
          scopeFlag: options.scopeFlag,
          scope: `${options.scopePrefix}${file}`,
          timeoutSeconds: options.timeoutSeconds,
          ...(options.memory === undefined ? {} : { memory: options.memory }),
          ...(options.cpus === undefined ? {} : { cpus: options.cpus }),
        }),
      });
      return {
        file,
        unit,
        logBase,
        tag,
        systemdRunArguments: buildSystemdRunArguments({
          unit,
          memoryMax: options.unitMemoryMax,
          cpuQuota: options.unitCpuQuota,
          workingDirectory: options.root,
          environment: [
            `${options.tagEnvironmentVariable}=${tag}`,
            ...options.extraEnvironment,
            `TMPDIR=${options.temporaryDirectory}`,
          // The units start with a bare environment, so the tools the campaign needs come from the launcher's.
            ...(options.path === undefined ? [] : [`PATH=${options.path}`]),
            `HOME=${options.home}`,
          ],
          body,
        }),
      };
    });
}
