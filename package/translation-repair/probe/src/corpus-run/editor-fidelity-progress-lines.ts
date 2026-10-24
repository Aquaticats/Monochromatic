//region Editor fidelity progress lines
// THE LINES AN ARM WRITES ABOUT HOW FAR IT WENT: how many of the pool's trials it planned, where the spend guard capped it
// and how many trials it completed. The read step takes the number of trials the later arms walk from the first arm's
// `TRIALS` line, so these are the wording both sides agree on.

/**
 Says how many of the pool's trials an arm planned to walk.

 @param planned - trials the arm was asked to walk
 @param total - trials the pool holds

 @returns The line, without a newline

 @example
 ```ts
 log.info(plannedLine({ planned: 41, total: 88, },),);
 ```
 */
export function plannedLine({ planned, total, }: { readonly planned: number; readonly total: number; },): string {
  return `${String(planned,)}${String(total,)}`;
}

/**
 Says where the spend guard capped an arm.

 @param completed - trials completed before the cap
 @param planned - trials the arm was asked to walk
 @param said - the guard's own sentence

 @returns The line, without a newline

 @example
 ```ts
 log.warn(cappedLine({ completed: 41, planned: 88, said, },),);
 ```
 */
export function cappedLine(
  { completed, planned, said, }: {
    readonly completed: number;
    readonly planned: number;
    readonly said: string;
  },
): string {
  return `${String(completed,)}${String(planned,)}${said}`;
}

/**
 Says how many trials an arm completed.

 @param completed - trials completed
 @param planned - trials the arm was asked to walk

 @returns The line, without a newline

 @example
 ```ts
 log.info(trialsLine({ completed: 41, planned: 88, },),);
 ```
 */
export function trialsLine({ completed, planned, }: { readonly completed: number; readonly planned: number; },): string {
  return `${String(completed,)}${String(planned,)}`;
}

//endregion Editor fidelity progress lines
