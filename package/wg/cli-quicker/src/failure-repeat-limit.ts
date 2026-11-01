/**
 Most distinct failures remembered for the rate limit before expired ones are dropped.
 */
const MAX_REMEMBERED_FAILURES = 256;

/**
 The identity of a failure for the rate limit:
 the line without its digits,
 so counts and durations do not make a repeat look new.

 @param line - The failure line.

 @returns The key.

 @example
 ```ts
 failureKey({ line: 'retry in 30 s' }); // 'retry in  s'
 ```
 */
export function failureKey({ line, }: { readonly line: string; },): string {
  /**
   Characters of the line that are not digits.
   */
  let key = '';
  for (const character of line)
    if ((character < '0') || (character > '9'))
      key += character;
  return key;
}

/**
 Decision of a repeat limit for one failure.
 */
export type RepeatDecision =
  | { readonly deliver: false; }
    | {
      readonly deliver: true;
      readonly held: number
    };

/**
 Limits how often the same failure reaches one channel,
 and counts the repeats it holds back.
 */
export type RepeatLimit = (call: {
  readonly key: string;
  readonly now: number
},) => RepeatDecision;

/**
 Creates the repeat limit of one channel.

 @param intervalMs - Least milliseconds between two deliveries of the same failure.

 @returns The limit.

 @example
 ```ts
 const limit = createRepeatLimit({ intervalMs: 60_000 });
 ```
 */
export function createRepeatLimit({ intervalMs, }: { readonly intervalMs: number; },): RepeatLimit {
  /**
   When each distinct failure last reached the channel,
   and how many repeats were held back since.
   */
  const remembered = new Map<string, {
    at: number;
    held: number
  }>();
  return function limit({
    key,
    now,
  },): RepeatDecision {
    /**
     Earlier delivery of the same failure.
     */
    const earlier = remembered.get(key,);
    if ((earlier !== undefined) && ((now - earlier.at) < intervalMs)) {
      earlier.held += 1;
      return { deliver: false, };
    }
    if (remembered.size >= MAX_REMEMBERED_FAILURES)
      for (const [text, entry,] of remembered)
        if ((now - entry.at) >= intervalMs)
          remembered.delete(text,);
    remembered.set(
      key,
      {
        at: now,
        held: 0,
      },
    );
    return {
      deliver: true,
      held: earlier?.held ?? 0,
    };
  };
}

/**
 Adds the count of held-back repeats to a line,
 so no occurrence is lost from the record.

 @param line - The failure line.

 @param held - Repeats held back since the last delivery.

 @returns The line to deliver.

 @example
 ```ts
 withHeldRepeats({ line: 'x', held: 2 }); // 'x (repeated 2 more time(s) since the last report)'
 ```
 */
export function withHeldRepeats({
  line,
  held,
}: {
  readonly line: string;
  readonly held: number
},): string {
  return held === 0 ? line : `${line} (repeated ${String(held,)} more time(s) since the last report)`;
}
