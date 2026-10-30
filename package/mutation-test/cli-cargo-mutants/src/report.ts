/**
 Describe the outcome of starting one unit.

 @param unit - unit name
 
 @param status - exit status of `systemd-run`, which `runProgram` reports as 1 when a signal ended it
 
 @param stderr - its standard error text
 
 @returns `<unit>: started` or `<unit>: <error text>`

 @example
 ```ts
 formatStartLine({ unit: 'u', status: 0, stderr: '' });
 ```
 */
export function formatStartLine({
  unit,
  status,
  stderr,
}: {
  readonly unit: string;
  readonly status: number;
  readonly stderr: string;
}): string {
  return `${unit}: ${status === 0 ? 'started' : stderr.trim() || `systemd-run ended with status ${String(status)}`}`;
}

/**
 Build the command that waits for the units, printed so the caller can poll them.

 @param units - unit names
 
 @returns the `systemctl` command line

 @example
 ```ts
 formatWaitHint(['a', 'b']);
 ```
 */
export function formatWaitHint(units: readonly string[]): string {
  return `wait with: systemctl --user is-active ${units.join(' ')}`;
}
