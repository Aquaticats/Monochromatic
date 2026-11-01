/**
 Most failure kinds remembered by one limit.
 When it is full the expired kinds go first and then the oldest,
 so the map never grows past this number.
 */
export const MAX_REMEMBERED_KINDS = 64;

/**
 Distinct normalized causes of one failure kind that one channel delivers per interval.
 */
export const MAX_CAUSES_PER_KIND = 4;

/**
 Distinct causes beyond the first of a kind that one channel delivers per interval,
 over all kinds together.
 The first cause of every kind is always delivered,
 so each kind is seen at least once;
 this bounds what a name chosen by a user can add on top.
 */
export const MAX_EXTRA_CAUSES_PER_INTERVAL = 32;

/**
 Characters of a normalized line that make up its cause.
 The whole line counts up to this bound,
 because every loud line starts with a fixed phrase of over a hundred characters
 that would use up a short bound before the reason begins.
 */
export const MAX_CAUSE_CHARS = 1_024;

/**
 What a name a user chose is replaced with.
 */
const NAME_PLACEHOLDER = '<name>';

/**
 Characters that may surround a word without being part of a name.
 */
const WORD_EDGE = '()[],;:\'"`.';

/**
 Endings of the names of systemd units,
 which are derived from names a user or an application chose.
 */
const UNIT_SUFFIXES = [
  '.service',
  '.scope',
  '.slice',
  '.socket',
  '.mount',
] as const;

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
 The kind decides what counts as a repeat,
 and within a kind the first few distinct causes are delivered.
 */
export type RepeatLimit = (call: {
  readonly key: string;
  readonly cause: string;
  readonly now: number
},) => RepeatDecision;

/**
 What one limit has seen of one failure kind within its interval.
 */
type KindWindow = {
  /**
   When the window began,
   in milliseconds.
   */
  readonly startedAt: number;

  /**
   Repeats held back since the last delivery.
   */
  held: number;

  /**
   Normalized causes delivered in this window.
   */
  readonly causes: string[];
};

/**
 The count of extra causes against the global cap of the interval.
 */
type ExtraCauses = {
  /**
   When the current count began,
   minus infinity before the first.
   */
  startedAt: number;

  /**
   Extra causes delivered since then.
   */
  admitted: number;
};

/**
 Whether a word,
 without its surrounding punctuation,
 is a name a user chose:
 an absolute path or the name of a unit.

 @param core - The word without surrounding punctuation.

 @returns Whether it is such a name.

 @example
 ```ts
 isUserChosenName({ core: '/user.slice/app.scope' }); // true
 ```
 */
function isUserChosenName({ core, }: { readonly core: string; },): boolean {
  return core.startsWith('/',)
    || UNIT_SUFFIXES.some(function endsWithSuffix(suffix,): boolean {
      return (core.length > suffix.length) && core.endsWith(suffix,);
    },);
}

/**
 A word with every run of digits made alike.

 @param word - One word of a line.

 @returns The word with each run of digits replaced by one `#`.

 @example
 ```ts
 withAlikeNumbers({ word: 'pid12' }); // 'pid#'
 ```
 */
function withAlikeNumbers({ word, }: { readonly word: string; },): string {
  /**
   Characters kept,
   and whether the last character read was a digit.
   */
  const scan = {
    pieces: [] as string[],
    inDigits: false,
  };
  for (const character of word) {
    if ((character >= '0') && (character <= '9')) {
      if (!scan.inDigits)
        scan.pieces
          .push('#',);
      scan.inDigits = true;
    }
    else {
      scan.inDigits = false;
      scan.pieces
        .push(character,);
    }
  }
  return scan.pieces
    .join('',);
}

/**
 Whether a character is not punctuation around a word.

 @param character - One character.

 @returns Whether it belongs to the word itself.

 @example
 ```ts
 isNotEdge('a'); // true
 ```
 */
function isNotEdge(character: string,): boolean {
  return !WORD_EDGE.includes(character,);
}

/**
 One word of a line in its normalized form:
 a name a user chose becomes a placeholder and keeps its surrounding punctuation,
 anything else keeps its text with the numbers made alike.

 @param word - One word of a line.

 @returns The normalized word.

 @example
 ```ts
 normalizeWord({ word: '(/a/b),' }); // '(<name>),'
 ```
 */
function normalizeWord({ word, }: { readonly word: string; },): string {
  /**
   Characters of the word.
   */
  const characters = Array.from(word,);
  /**
   Index of the first character that is not leading punctuation.
   */
  const first = characters.findIndex(isNotEdge,);
  if (first === (-1))
    return withAlikeNumbers({ word, },);
  /**
   Index of the last character that is not trailing punctuation.
   */
  const last = characters.findLastIndex(isNotEdge,);
  /**
   The word without its surrounding punctuation.
   */
  const core = characters.slice(
    first,
    last + 1,
  )
    .join('',);
  if (!isUserChosenName({ core, },))
    return withAlikeNumbers({ word, },);
  return `${characters.slice(
    0,
    first,
  )
    .join('',)}${NAME_PLACEHOLDER}${characters.slice(last + 1,)
      .join('',)}`;
}

/**
 The cause of a failure line:
 its text with the numbers made alike and the names a user chose replaced,
 so the same reason for another process id,
 count,
 duration or cgroup name is the same cause,
 and a different reason is another.
 At most the first characters count.

 @param line - The failure line.

 @returns The normalized cause.

 @example
 ```ts
 normalizeCause({ line: 'pid 12 failed (attempt 3)' }); // 'pid # failed (attempt #)'
 ```
 */
export function normalizeCause({ line, }: { readonly line: string; },): string {
  /**
   Normalized words,
   read until the bound is reached.
   */
  const words: string[] = [];
  /**
   Characters kept so far.
   */
  const kept = { length: 0, };
  for (const word of line.split(' ',)) {
    if (kept.length >= MAX_CAUSE_CHARS)
      break;
    /**
     The word in its normalized form.
     */
    const normalized = normalizeWord({ word, },);
    words.push(normalized,);
    kept.length += Array.from(normalized,)
      .length
      + 1;
  }
  return Array.from(words.join(' ',),)
    .slice(
    0,
    MAX_CAUSE_CHARS,
    )
    .join('',);
}

/**
 Counts one extra cause against the global cap of the interval.

 @param extra - The count.

 @param now - Current time in milliseconds.

 @param intervalMs - The interval of the limit.

 @returns Whether the extra cause may be delivered.

 @example
 ```ts
 takeExtraCause({ extra: { startedAt: 0, admitted: 0 }, now: 1, intervalMs: 1000 });
 ```
 */
function takeExtraCause(
  {
    extra,
    now,
    intervalMs,
  }: {
    readonly extra: ExtraCauses;
    readonly now: number;
    readonly intervalMs: number;
  },
): boolean {
  if ((now - extra.startedAt) >= intervalMs) {
    extra.startedAt = now;
    extra.admitted = 0;
  }
  if (extra.admitted >= MAX_EXTRA_CAUSES_PER_INTERVAL)
    return false;
  extra.admitted += 1;
  return true;
}

/**
 Makes room for one more kind:
 drops the kinds whose interval is over,
 and the oldest one when none is.
 Insertion order is delivery order, because a kind is re-inserted when its window begins.

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
    readonly remembered: Map<string, KindWindow>;
    readonly now: number;
    readonly intervalMs: number;
  },
): void {
  for (const [kind, window,] of remembered)
    if ((now - window.startedAt) >= intervalMs)
      remembered.delete(kind,);
  if (remembered.size < MAX_REMEMBERED_KINDS)
    return;
  /**
   Kind whose window began longest ago.
   */
  const oldest = remembered.keys()
    .next();
  if (oldest.done !== true)
    remembered.delete(oldest.value,);
}

/**
 Creates the repeat limit of one channel.

 @param intervalMs - Least milliseconds between two windows of the same failure kind.

 @returns The limit.

 @example
 ```ts
 const limit = createRepeatLimit({ intervalMs: 60_000 });
 ```
 */
export function createRepeatLimit({ intervalMs, }: { readonly intervalMs: number; },): RepeatLimit {
  /**
   The window of each failure kind.
   */
  const remembered = new Map<string, KindWindow>();
  /**
   The global count of extra causes.
   */
  const extra: ExtraCauses = {
    startedAt: Number.NEGATIVE_INFINITY,
    admitted: 0,
  };
  return function limit({
    key,
    cause,
    now,
  },): RepeatDecision {
    /**
     Window of the same failure kind.
     */
    const earlier = remembered.get(key,);
    if ((earlier !== undefined) && ((now - earlier.startedAt) < intervalMs)) {
      if ((earlier.causes
        .length
        < MAX_CAUSES_PER_KIND)
        && (!earlier.causes
          .includes(cause,))
        && takeExtraCause({
          extra,
          now,
          intervalMs,
        },)) {
        earlier.causes
          .push(cause,);
        return {
          deliver: true,
          held: 0,
        };
      }
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
        startedAt: now,
        held: 0,
        causes: [cause,],
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
