import {
  buildTimeoutFlag,
  cpusFlag,
  examineFlag,
  memoryFlag,
  passesFlag,
  timeoutFlag,
} from './constants.ts';

/**
 Build the arguments that follow `--` on the campaign task's command line for one file.

 The build timeout and the container limits are forwarded only when the caller names them, so the task's own defaults
 apply otherwise; every mutant-name pattern is forwarded, in order, because the patterns apply to every file.
 The values are forwarded unchanged; the task validates them.

 @param scopeFlag - flag the task takes once per source-file scope
 
 @param scope - file scope, with the scope prefix already applied
 
 @param timeoutSeconds - per-mutant test timeout
 
 @param buildTimeoutSeconds - per-mutant build timeout, or absent for the task's default
 
 @param examine - mutant-name patterns, forwarded one flag each, or absent for none
 
 @param memory - container memory limit, or absent for the task's default
 
 @param cpus - container CPU count, or absent for the task's default

 @param passes - number of passes the campaign runs, or absent for the task's default
 
 @returns argument list in the order the task receives it

 @example
 ```ts
 buildTaskArguments({ scopeFlag: '--file', scope: 'src/a.rs', timeoutSeconds: 900 });
 ```
 */
export function buildTaskArguments({
  scopeFlag,
  scope,
  timeoutSeconds,
  buildTimeoutSeconds,
  examine = [],
  memory,
  cpus,
  passes,
}: {
  readonly scopeFlag: string;
  readonly scope: string;
  readonly timeoutSeconds: number;
  readonly buildTimeoutSeconds?: number;
  readonly examine?: readonly string[];
  readonly memory?: string;
  readonly cpus?: string;
  readonly passes?: 1 | 2;
}): readonly string[] {
  return [
    timeoutFlag,
    String(timeoutSeconds),
    ...(buildTimeoutSeconds === undefined ? [] : [
      buildTimeoutFlag,
      String(buildTimeoutSeconds)
    ]),
    ...examine.flatMap(function examineArguments(pattern) {
      return [
        examineFlag,
        pattern
      ];
    }),
    scopeFlag,
    scope,
    ...(memory === undefined ? [] : [
      memoryFlag,
      memory
    ]),
    ...(cpus === undefined ? [] : [
      cpusFlag,
      cpus
    ]),
    ...(passes === undefined ? [] : [
      passesFlag,
      String(passes)
    ]),
  ];
}
