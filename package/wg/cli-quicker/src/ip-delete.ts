import { CommandError, } from './errors.ts';
import {
  runAllowingFailure,
  type CommandResult,
} from './runner.ts';

/**
 Exact idempotent-delete diagnostics emitted by `ip`.
 */
const ABSENT_DELETE_DIAGNOSTICS: ReadonlySet<string> = new Set([
  'Error: FIB table does not exist.',
  'RTNETLINK answers: No such file or directory',
  'RTNETLINK answers: No such process',
],);

/**
 Reports whether a delete found nothing to delete,
 which a caller needs when absence means something else than success.

 @param result - What the delete ended with.

 @returns Whether `ip` answered with its exact absence diagnostic.

 @example
 ```ts
 isAbsentDelete({ result: { stdout: '', stderr: 'RTNETLINK answers: No such file or directory\n', exitCode: 2 } });
 ```
 */
export function isAbsentDelete(
  { result, }: { readonly result: CommandResult & { readonly exitCode: number; }; },
): boolean {
  return (result.exitCode === 2) && ABSENT_DELETE_DIAGNOSTICS.has(result.stderr
    .trimEnd(),);
}

/**
 Deletes one route or rule while suppressing only exact absence.
 
 @param args - Arguments after `ip` describing delete operation.
 
 @throws {@link CommandError} for every nonzero result other than exact absence.
 
 @example
 ```ts
 await runIpDelete({ args: ['-4', 'rule', 'delete', 'pref', '50'] });
 ```
 */
export async function runIpDelete(
  { args, }: { readonly args: readonly string[]; },
): Promise<void> {
  /**
   Fresh arguments separated from caller-owned container.
   */
  const commandArgs = [...args,];
  /**
   Delete result translated only for idempotent absence.
   */
  const result = await runAllowingFailure({
    command: 'ip',
    args: commandArgs,
  },);
  if (result.exitCode === 0)
    return;
  if (isAbsentDelete({ result, },))
    return;
  throw new CommandError({
    command: 'ip',
    args: commandArgs,
    exitCode: result.exitCode,
    stderr: result.stderr,
  },);
}
