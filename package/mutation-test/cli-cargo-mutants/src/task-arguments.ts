import {
  cpusFlag,
  memoryFlag,
  timeoutFlag,
} from './constants.ts';

/**
 Build the arguments that follow `--` on the campaign task's command line for one file.

 The container limits are forwarded only when the caller names them, so the task's own defaults apply otherwise.
 The values are forwarded unchanged; the task validates them.

 @param scopeFlag - flag the task takes once per source-file scope
 
 @param scope - file scope, with the scope prefix already applied
 
 @param timeoutSeconds - per-mutant test timeout
 
 @param memory - container memory limit, or absent for the task's default
 
 @param cpus - container CPU count, or absent for the task's default
 
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
  memory,
  cpus,
}: {
  readonly scopeFlag: string;
  readonly scope: string;
  readonly timeoutSeconds: number;
  readonly memory?: string;
  readonly cpus?: string;
}): readonly string[] {
  return [
    timeoutFlag,
    String(timeoutSeconds),
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
  ];
}
