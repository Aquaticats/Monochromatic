/**
 Only recognized suppressed diagnostics can account for a failed compiler process.

 @module
 */

/**
 Recognize emitted-file metadata observed from the configured compiler's build output.

 @param line - retained stdout line after diagnostic filtering
 
 @returns whether the line is blank or names an emitted file
 */
function isEmissionMetadata(line: string): boolean {
  if (line.trim()
    .length
    === 0) return true;
  return line.startsWith('TSFILE: ') && (line.slice('TSFILE: '.length)
    .trim()
    .length
    > 0);
}

/**
 Permit failure suppression only when known diagnostic removal explains every retained stream.

 @param stdout - original compiler stdout
 
 @param stderr - original compiler stderr
 
 @param filteredStdout - stdout after known diagnostic removal
 
 @param filteredStderr - stderr after known diagnostic removal
 
 @returns whether known suppression fully accounts for the failed process
 
 @example
 ```ts
 isFullySuppressedCompilerFailure({ stdout: diagnostic, stderr: '', filteredStdout: '', filteredStderr: '' });
 ```
 */
export function isFullySuppressedCompilerFailure({
  stdout,
  stderr,
  filteredStdout,
  filteredStderr
}: {
  readonly stdout: string;
  readonly stderr: string;
  readonly filteredStdout: string;
  readonly filteredStderr: string;
}): boolean {
  if ((stdout === filteredStdout) && (stderr === filteredStderr)) return false;
  if (filteredStderr.trim()
    .length
    > 0) return false;
  return filteredStdout.split('\n')
    .every(isEmissionMetadata);
}
