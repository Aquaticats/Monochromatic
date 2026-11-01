import { CommandError, } from './errors.ts';
import { runAllowingFailure, } from './runner.ts';

/**
 Exact idempotent-delete diagnostics emitted by `ip`.
 */
const ABSENT_DELETE_DIAGNOSTICS: ReadonlySet<string> = new Set([
  'Error: FIB table does not exist.',
  'RTNETLINK answers: No such file or directory',
  'RTNETLINK answers: No such process',
],);

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
  if ((result.exitCode === 2)
    && ABSENT_DELETE_DIAGNOSTICS.has(result.stderr
      .trimEnd(),)) {
    return;
  }
  throw new CommandError({
    command: 'ip',
    args: commandArgs,
    exitCode: result.exitCode,
    stderr: result.stderr,
  },);
}
