import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import { parseModelJson, } from './model-content.ts';

//region Json leading value
// LEDGER P8 (the whole-package audit, 2026-09-27): a complete JSON value
// followed by more text failed the whole-answer parse and was lost as a
// schema mismatch, 760 times across the run logs and about four voices on
// each TianqiChen666 run, mostly Mercury's. The stored TianqiChen666 replies
// of that shape trail a hyphen line (31), a sentence (21) or a stray fence
// (8), and every one ended with the provider's stop reason, so the value the
// answer opens with is the whole answer and the text after it is commentary.
//
// ONE LINEAR PASS over code points, aware of strings and their escapes,
// finds where the opening object or array closes; a brace inside a string
// never counts, and a character outside the Basic Multilingual Plane is one
// step, never two. The
// slice is then parsed as JSON, so a scan confused by malformed input can
// return nothing, never a wrong value.

/**
 Logger root for the leading-value reader.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Characters that open a JSON container.
 */
const OPENERS: ReadonlySet<string> = new Set([
  '{',
  '[',
],);

/**
 Characters that close a JSON container.
 */
const CLOSERS: ReadonlySet<string> = new Set([
  '}',
  ']',
],);

/**
 Where a scan over an answer's characters stands.
 */
type ScanState = {
  /**
   Containers open at the cursor.
   */
  readonly depth: number;

  /**
   Whether the cursor stands inside a string.
   */
  readonly inString: boolean;

  /**
   Whether the previous character inside a string was an unescaped backslash.
   */
  readonly escaped: boolean;

  /**
   Code point count of the opening container once it has closed, -1 before.
   */
  readonly end: number;
};

/**
 Scan state before the first character.
 */
const SCAN_START: ScanState = {
  depth: 0,
  inString: false,
  escaped: false,
  end: -1,
};

/**
 Advances the scan over one character.

 @param state - scan so far

 @param character - next code point

 @param at - its code point index

 @returns Scan after it, unchanged once the container has closed

 @example
 ```ts
 const next = scanStep({ state: SCAN_START, character: '{', at: 0, },);
 ```
 */
function scanStep(
  {
    state,
    character,
    at,
  }: {
    readonly state: ScanState;
    readonly character: string;
    readonly at: number;
  },
): ScanState {
  if (state.end !== (-1))
    return state;
  if (state.inString) {
    if (state.escaped)
      return {
        ...state,
        escaped: false,
      };
    if (character === '\\')
      return {
        ...state,
        escaped: true,
      };
    return (character === '"')
      ? {
        ...state,
        inString: false,
      }
      : state;
  }
  if (character === '"')
    return {
      ...state,
      inString: true,
    };
  if (OPENERS.has(character,))
    return {
      ...state,
      depth: state.depth + 1,
    };
  if (!CLOSERS.has(character,))
    return state;
  /**
   Containers still open after this one closes.
   */
  const depth = state.depth - 1;
  return {
    ...state,
    depth,
    end: (depth === 0) ? (at + 1) : (-1),
  };
}

/**
 Code point count of the object or array a text opens with, where its
 brackets balance outside strings.

 @param characters - answer's code points, leading whitespace already dropped

 @returns Count of the container's code points, or -1 when the answer opens
 with no container or it never closes

 @example
 ```ts
 leadingJsonValueEnd({ characters: Array.from('{"a": 1} tail',), },); // => 8
 ```
 */
export function leadingJsonValueEnd(
  { characters, }: { readonly characters: readonly string[]; },
): number {
  if (!OPENERS.has(characters[0] ?? '',))
    return -1;
  /**
   Scan over every character.
   */
  const scanned = characters.reduce(
    function step(
      state: ScanState,
      character: string,
      at: number,
    ): ScanState {
      return scanStep({
        state,
        character,
        at,
      },);
    },
    SCAN_START,
  );
  return scanned.end;
}

/**
 Reads the JSON value an answer opens with when more text follows it.

 @param text - fence-stripped answer channel that failed to parse as a whole

 @returns Parsed value with the count of trailing code points, or nothing to
 read

 @example
 ```ts
 readJsonBeforeTrailingText({ text: '{"best": 2}\nThat is my pick.', },);
 // => { parsed: true, value: { best: 2 }, trailing: 17 }
 ```
 */
export function readJsonBeforeTrailingText({ text, }: { readonly text: string; },):
  | {
    readonly parsed: true;
    readonly value: unknown;
    readonly trailing: number;
  }
    | { readonly parsed: false; }
  {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: readJsonBeforeTrailingText.name,
    l,
  },);
  /**
   Answer's code points from its first non-blank character.
   */
  const characters = Array.from(text.trimStart(),);
  /**
   Code points of the opening value.
   */
  const end = leadingJsonValueEnd({ characters, },);
  if (end === (-1)) {
    rl.debug('answer opens with no container that closes, so there is no leading value to read',);
    return { parsed: false, };
  }
  /**
   Parse of the opening container, whose refusal is data that names its class
   and quotes none of the text V8 refused.
   */
  const attempt = parseModelJson({
    text: characters
      .slice(
      0,
      end,
      )
      .join('',),
  },);
  if (!attempt.parsed) {
    rl.debug(`the opening container is not JSON: ${attempt.detail}`,);
    return { parsed: false, };
  }
  return {
    parsed: true,
    value: attempt.value,
    trailing: characters.length - end,
  };
}

//endregion Json leading value
