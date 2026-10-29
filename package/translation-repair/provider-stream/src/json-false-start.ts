import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { readJsonBeforeTrailingText, } from './json-leading-value.ts';
import { parseModelJson, } from './model-content.ts';

//region Json false start
// THE SEVENTEENTH CLASS, found by the tenth hakureico pass on 2026-09-08 and
// read back through the ninth: a reasoning stream sometimes opens its answer,
// abandons the opening, and then writes the whole object after it, so the
// answer channel reads `{"best": 1{"best": 1, "reason": ...}`,
// `{"{"resolution": ...}`, `{ {   "choice": ...}` or
// `{   "issues":{     "issues": [...]}`. Bedrock's gpt-oss-120b did it on six
// of the ninth pass's eight mismatches; OpenRouter's Makora route for
// deepseek-v4-flash-0731 did it on every mismatch of the tenth; every one of
// those streams carried reasoning characters, and no stream without them did.
// The object after the fragment is the model's answer, so it is read and the
// fragment's length is logged, where before the voice was lost.

/**
 Logger root for the false-start reader.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Characters of the answer within which an abandoned opening may sit.
 The longest observed was 13 (`{   "issues":`); a fragment longer than
 this window is not an opening but some other defect.
 */
export const FALSE_START_WINDOW = 256;

/**
 Reads a JSON object that follows an abandoned opening fragment.

 Each `{` inside the window, after the first character, is tried as the
 object's start until one parses to the end of the text; the first that
 does is the answer. Bounded by the window, so a long reply costs at most
 as many parses as it has braces in its first characters, and each parse
 is one linear pass.

 @param text - answer channel that failed to parse as a whole

 @returns Parsed value with the abandoned length, or nothing to read past

 @example
 ```ts
 const past = readJsonPastFalseStart({ text: '{"best": 1{"best": 1}', },);
 ```
 */
export function readJsonPastFalseStart({ text, }: { readonly text: string; },):
  | {
    readonly parsed: true;
    readonly value: unknown;
    readonly abandoned: number;
  }
    | { readonly parsed: false; }
  {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: readJsonPastFalseStart.name,
    l,
  },);

  /**
   Answer with its leading whitespace dropped, to see what it opens with.
   */
  const opening = text.trimStart();

  if (!opening.startsWith('{',)) {
    rl.debug('answer does not open an object, so there is no false start to read past',);
    return { parsed: false, };
  }

  /**
   Index past which no object start is tried.
   */
  const last = Math.min(
    text.length,
    FALSE_START_WINDOW,
  );

  // One counter pass over the window; only a brace is tried as a start.
  for (let at = 1; at < last; at += 1) {
    if (text[at] !== '{')
      continue;
    /**
     Parse of the remainder, whose refusal is data that names its class and
     quotes none of the text V8 refused.
     */
    const attempt = parseModelJson({ text: text.slice(at,), },);
    if (!attempt.parsed) {
      rl.debug(`no object starts at ${String(at,)}: ${attempt.detail}`,);
      continue;
    }
    rl.debug(`read an object past an abandoned opening of ${String(at,)} ${
      wordForCount({
        count: at,
        one: 'char',
        many: 'chars',
      },)
    }`,);
    return {
      parsed: true,
      value: attempt.value,
      abandoned: at,
    };
  }

  return { parsed: false, };
}

/**
 Parses an answer as a whole; where the whole fails, past a false start; and
 where that fails too, as the value it opens with when more text follows
 (ledger P8).

 THE FALSE START BEFORE THE LEADING VALUE: a reply of two whole objects is
 the model revising its answer, and the replay of 587,102 stored replies on
 2026-09-28 found 18 such, the first empty or missing a field the second
 carries, so the later object is the answer and the false-start reader, which
 reads to the end of the text, already takes it. Trailing prose defeats that
 reader, since no brace inside the window starts JSON that runs to the end,
 and the leading value then recovers the answer (919 replies in that replay).

 @param text - fence-stripped answer channel

 @returns Parsed value with the abandoned and trailing lengths (zero for a
 whole parse), or failure detail

 @example
 ```ts
 const attempt = parseAnswerJson({ text: unwrapped, },);
 ```
 */
export function parseAnswerJson({ text, }: { readonly text: string; },):
  | {
    readonly parsed: true;
    readonly value: unknown;
    readonly abandoned: number;
    readonly trailing: number;
  }
    | {
      readonly parsed: false;
      readonly detail: string;
    }
  {
  /**
   Parse of the whole answer, the ordinary case.
   */
  const whole = parseModelJson({ text, },);

  if (whole.parsed) {
    return {
      parsed: true,
      value: whole.value,
      abandoned: 0,
      trailing: 0,
    };
  }

  /**
   Reading past an abandoned opening, when there is one.
   */
  const past = readJsonPastFalseStart({ text, },);
  if (past.parsed) {
    return {
      ...past,
      trailing: 0,
    };
  }

  /**
   The value the answer opens with, when text follows it.
   */
  const leading = readJsonBeforeTrailingText({ text, },);
  return leading.parsed
    ? {
      parsed: true,
      value: leading.value,
      abandoned: 0,
      trailing: leading.trailing,
    }
    : whole;
}

//endregion Json false start
