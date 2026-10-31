import { shellProgram } from './constants.ts';

/**
 Build the `systemd-run` arguments that start one capped transient user unit.

 The arguments go to the program as an array, never through a shell; the only text a shell reads is `body`,
 which the caller has already quoted.

 @param unit - unit name without the `.service` suffix
 
 @param memoryMax - systemd `MemoryMax` value for the whole unit
 
 @param cpuQuota - systemd `CPUQuota` value for the whole unit
 
 @param workingDirectory - directory the unit starts in
 
 @param environment - `NAME=value` entries set inside the unit, in the order given
 
 @param body - shell program the unit runs
 
 @returns the argument list after the `systemd-run` program name

 @example
 ```ts
 buildSystemdRunArguments({ unit: 'u', memoryMax: '12G', cpuQuota: '500%', workingDirectory: '/w', environment: [], body: 'true' });
 ```
 */
export function buildSystemdRunArguments({
  unit,
  memoryMax,
  cpuQuota,
  workingDirectory,
  environment,
  body,
}: {
  readonly unit: string;
  readonly memoryMax: string;
  readonly cpuQuota: string;
  readonly workingDirectory: string;
  readonly environment: readonly string[];
  readonly body: string;
}): readonly string[] {
  return [
    '--user',
    `--unit=${unit}`,
    '--collect',
    `--property=MemoryMax=${memoryMax}`,
    `--property=CPUQuota=${cpuQuota}`,
    `--working-directory=${workingDirectory}`,
    ...environment.map(function setenvArgument(entry) {
      return `--setenv=${entry}`;
    }),
    shellProgram,
    '-c',
    body,
  ];
}
