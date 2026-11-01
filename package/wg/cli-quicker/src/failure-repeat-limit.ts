/**
 Most failure kinds remembered by one limit.
 When it is full the expired kinds go first and then the oldest,
 so the map never grows past this number.
 */
export const MAX_REMEMBERED_KINDS = 64;

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
 Makes room for one more kind:
 drops the kinds whose interval is over,
 and the oldest one when none is.
 Insertion order is delivery order, because a kind is re-inserted at every delivery.

 @param remembered - The remembered kinds.

 @param now - Current time in milliseconds.

 @param intervalMs - The interval of the limit.

 @example
 ```ts
 forgetOldest({ remembered: new Map(), now: 0, intervalMs: 1000 });
 ```
 */
function forgetOldest(
  {
    remembered,
    now,
    intervalMs,
  }: {
    readonly remembered: Map<string, {
      at: number;
      held: number
    }>;
    readonly now: number;
    readonly intervalMs: number;
  },
): void {
  for (const [kind, entry,] of remembered)
    if ((now - entry.at) >= intervalMs)
      remembered.delete(kind,);
  if (remembered.size < MAX_REMEMBERED_KINDS)
    return;
  /**
   Kind delivered longest ago.
   */
  const oldest = remembered.keys()
    .next();
  if (oldest.done !== true)
    remembered.delete(oldest.value,);
}

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
    remembered.delete(key,);
    if (remembered.size >= MAX_REMEMBERED_KINDS)
      forgetOldest({
        remembered,
        now,
        intervalMs,
      },);
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
