/**
 Model-id tokenization for retirement decisions.

 A model id is the only age evidence pi's bundled catalog carries, so this tokenizer
 decides what counts as a version, what counts as a calendar date, and what stays part
 of a model's name.
 Every scan is one forward pass over characters with no regular expressions, matching
 the style of `splitModelIdTokens` in
 `package/agent-harness-shared/model-selection/src/version.ts`.

 @module
 */

//region Constants

/**
 Digit count of a month-day or year-month snapshot token such as `0813` or `2512`.
 */
const SHORT_DATE_DIGIT_COUNT = 4;

/**
 Digit count of a `YYMMDD` snapshot token.
 */
const MEDIUM_DATE_DIGIT_COUNT = 6;

/**
 Digit count of a `YYYYMMDD` snapshot token.
 */
const LONG_DATE_DIGIT_COUNT = 8;

/**
 Digit count of the leading month or year in a short date token.
 */
const SHORT_FIRST_FIELD_DIGIT_COUNT = 2;

/**
 Index where the month field starts in a `YYMMDD` token.
 */
const MEDIUM_MONTH_START = 2;

/**
 Index where the day field starts in a `YYMMDD` token.
 */
const MEDIUM_DAY_START = 4;

/**
 Index where the month field starts in a `YYYYMMDD` token.
 */
const LONG_MONTH_START = 4;

/**
 Index where the day field starts in a `YYYYMMDD` token.
 */
const LONG_DAY_START = 6;

/**
 Smallest plausible calendar month or day in a date-shaped token.
 */
const SMALLEST_CALENDAR_FIELD = 1;

/**
 Largest plausible calendar month in a date-shaped token.
 */
const LARGEST_MONTH = 12;

/**
 Largest plausible calendar day in a date-shaped token.
 */
const LARGEST_DAY = 31;

/**
 Smallest plausible four-digit year in a date-shaped token.
 */
const SMALLEST_YEAR = 2_000;

/**
 Largest plausible four-digit year in a date-shaped token.
 */
const LARGEST_YEAR = 2_099;

/**
 Marker some router catalogs prefix onto rolling alias entries.
 */
const ALIAS_MARKER = '~';

/**
 Organization prefix that carries no version evidence and is dropped before tokenizing.
 */
const HUGGING_FACE_PREFIX = 'hf:';

/**
 Character closing an organization segment in a router model id.
 */
const ORGANIZATION_SEPARATOR = '/';

/**
 Character separating dotted version components inside one token.
 */
const COMPONENT_SEPARATOR = '.';

/**
 Prefix marking an explicit version token such as `v4`.
 */
const VERSION_MARKER = 'v';

/**
 Separator joining name-shape tokens so two ids with the same words in the same order
 share a family.
 */
const NAME_SHAPE_SEPARATOR = '-';

//endregion Constants

//region Types

/**
 Classification of one model-id token.

 `raws` stays empty for a token that carries no trailing version, which is the only
 signal the retirement rule needs to keep parameter sizes and glued names out of
 version comparisons.
 */
export type TokenClassification = {
  /**
   Text that joins the name shape, absent when the whole token is a version.
   */
  readonly word?: string;
  /**
   Raw text of each version component, leading zeros preserved because they decide
   date classification.
   */
  readonly raws: readonly string[];
};

/**
 Structured age evidence extracted from one model id.
 */
export type ModelIdParse = {
  /**
   Every non-version token in id order, joined by `-`.
   Tier words and parameter sizes stay here, so they separate families instead of
   ordering them.
   */
  readonly nameShape: string;
  /**
   Version components in id order as numbers.
   */
  readonly versionParts: readonly number[];
  /**
   Raw text of each version component, index-aligned with `versionParts`.
   */
  readonly versionRaws: readonly string[];
  /**
   Raw text of each version component that reads as a calendar date.
   */
  readonly dateRaws: readonly string[];
};

/**
 Sentinel for a token position where no numeric run starts.
 */
export const NO_NUMERIC_RUN: unique symbol = Symbol('token carries no numeric run',);

/**
 Outcome of reading one numeric run inside a token.
 */
type NumericRun = {
  /**
   Index just past the last character consumed by the run.
   */
  readonly end: number;
  /**
   Raw text of each dot-separated component.
   */
  readonly raws: readonly string[];
};

//endregion Types

//region Character predicates

/**
 Test one character for decimal digit membership.

 @param character - single character under test

 @returns whether the character is `0` through `9`

 @example
 ```typescript
 isAsciiDigit('7'); // true
 ```
 */
export function isAsciiDigit(character: string,): boolean {
  return (character >= '0') && (character <= '9');
}

/**
 Test one character for lower-case ASCII letter membership.

 Callers lowercase the id first, so upper-case letters never reach this predicate and
 treating them as separators keeps the classifier total.

 @param character - single character under test

 @returns whether the character is `a` through `z`

 @example
 ```typescript
 isLowerAsciiLetter('q'); // true
 ```
 */
export function isLowerAsciiLetter(character: string,): boolean {
  return (character >= 'a') && (character <= 'z');
}

/**
 Test one character for membership in a model-id token.

 A dot stays inside a token so `3.8` reads as one dotted version instead of two
 components of the name shape.

 @param character - single character under test

 @returns whether the character continues a token

 @example
 ```typescript
 isTokenCharacter('.'); // true
 ```
 */
export function isTokenCharacter(character: string,): boolean {
  return isAsciiDigit(character,)
    || isLowerAsciiLetter(character,)
    || (character === COMPONENT_SEPARATOR);
}

//endregion Character predicates

//region Token scanning

/**
 Split an id into tokens on every character that cannot continue one.

 The scan is one pass with a mutable cursor held inside a named function, which keeps
 the tokenizer linear in id length.

 @param text - lowercased model id with organization prefixes removed

 @returns tokens in id order, empty runs dropped

 @example
 ```typescript
 splitOnNonTokenCharacters('glm-5.2'); // ['glm', '5.2']
 ```
 */
export function splitOnNonTokenCharacters(text: string,): string[] {
  return (function scanTokens(): string[] {
    /**
     Completed tokens in id order.
     */
    const tokens: string[] = [];
    /**
     Characters collected for the token being scanned.
     */
    let currentCharacters: string[] = [];
    for (const character of text) {
      if (!isTokenCharacter(character,)) {
        if (currentCharacters.length > 0)
          tokens.push(currentCharacters.join('',),);
        currentCharacters = [];
        continue;
      }
      currentCharacters.push(character,);
    }
    if (currentCharacters.length > 0)
      tokens.push(currentCharacters.join('',),);
    return tokens;
  })();
}

/**
 Drop the alias marker and any Hugging Face style organization segment.

 The organization of a router entry names a vendor rather than a model age, and keeping
 it would split `hf:moonshotai/Kimi-K3` from `kimi-k3` on the same provider. A router's
 own `vendor/model` segment stays, because there it does distinguish families.

 @param modelId - model id exactly as the catalog carries it

 @returns id ready for token splitting

 @example
 ```typescript
 stripOrganizationPrefix('hf:zai-org/GLM-5.3'); // 'glm-5.3'
 ```
 */
export function stripOrganizationPrefix(modelId: string,): string {
  /**
   Id lowercased for the classifier.
   */
  const lowered = modelId.toLowerCase();
  /**
   Id with the alias marker removed when present.
   */
  const unmarked = lowered.startsWith(ALIAS_MARKER,)
    ? lowered.slice(ALIAS_MARKER.length,)
    : lowered;
  if (!unmarked.startsWith(HUGGING_FACE_PREFIX,))
    return unmarked;
  /**
   Index of the separator closing the organization segment, or -1 when absent.
   */
  const separatorIndex = unmarked.indexOf(ORGANIZATION_SEPARATOR,);
  if (separatorIndex === (-1))
    return unmarked;
  return unmarked.slice(separatorIndex + ORGANIZATION_SEPARATOR.length,);
}

/**
 Read one `digits(.digits)*` run starting at a given index.

 @param text - token being scanned

 @param start - index the run must begin at

 @returns run extent and raw components, or {@link NO_NUMERIC_RUN} when no digit starts
 the run or a dot is not followed by a digit

 @example
 ```typescript
 readNumericRun({ text: '3.8', start: 0 }); // { end: 3, raws: ['3', '8'] }
 ```
 */
export function readNumericRun(
  {
    text,
    start,
  }: {
    readonly text: string;
    readonly start: number;
  },
): NumericRun | typeof NO_NUMERIC_RUN {
  return (function scanRun(): NumericRun | typeof NO_NUMERIC_RUN {
    /**
     Cursor walking the run.
     */
    let cursor = start;
    while ((cursor < text.length) && isAsciiDigit(text.charAt(cursor,),))
      cursor += 1;
    if (cursor === start)
      return NO_NUMERIC_RUN;
    /**
     Raw components collected so far.
     */
    const raws: string[] = [text.slice(
      start,
      cursor,
    ),];
    for (;;) {
      if ((cursor >= text.length) || (text.charAt(cursor,) !== COMPONENT_SEPARATOR))
        break;
      /**
       Index just past the component separator.
       */
      const componentStart = cursor + COMPONENT_SEPARATOR.length;
      cursor = componentStart;
      while ((cursor < text.length) && isAsciiDigit(text.charAt(cursor,),))
        cursor += 1;
      if (cursor === componentStart)
        return NO_NUMERIC_RUN;
      raws.push(text.slice(
        componentStart,
        cursor,
      ),);
    }
    return {
      end: cursor,
      raws,
    };
  })();
}

/**
 Count the leading lower-case letters of one token.

 @param token - token being classified

 @returns how many characters form the alphabetic prefix

 @example
 ```typescript
 alphaPrefixLength('qwen3.8'); // 4
 ```
 */
function alphaPrefixLength(token: string,): number {
  return (function countPrefix(): number {
    for (let index = 0; index < token.length; index += 1) {
      if (!isLowerAsciiLetter(token.charAt(index,),))
        return index;
    }
    return token.length;
  })();
}

//endregion Token scanning

//region Date classification

/**
 Test whether one raw version component reads as a calendar date.

 Widths are four digits for `MMDD` or `YYMM`, six for `YYMMDD`, and eight for
 `YYYYMMDD`. A component that fits none of them stays a version number, which is what
 keeps `2024` in `gpt-4o-2024-05-13` comparable against another snapshot's year.

 @param raw - raw text of one version component, leading zeros intact

 @returns whether the component is a calendar date

 @example
 ```typescript
 isDateShapedRaw('0813'); // true
 isDateShapedRaw('813'); // false
 ```
 */
export function isDateShapedRaw(raw: string,): boolean {
  for (const character of raw) {
    if (!isAsciiDigit(character,))
      return false;
  }
  if (raw.length === SHORT_DATE_DIGIT_COUNT) {
    /**
     First two digits, read as a month by one convention and as a year by the other.
     */
    const first = Math.trunc(
      Number(raw.slice(
        0,
        SHORT_FIRST_FIELD_DIGIT_COUNT,
      ),),
    );
    /**
     Last two digits, read as a day by one convention and as a month by the other.
     */
    const second = Math.trunc(
      Number(raw.slice(SHORT_FIRST_FIELD_DIGIT_COUNT,),),
    );
    /**
     Whether the token reads as `MMDD`.
     */
    const readsAsMonthDay = (first >= SMALLEST_CALENDAR_FIELD)
      && (first <= LARGEST_MONTH)
      && (second >= SMALLEST_CALENDAR_FIELD)
      && (second <= LARGEST_DAY);
    /**
     Whether the token reads as `YYMM`.
     */
    const readsAsYearMonth = (first <= LARGEST_DAY)
      && (second >= SMALLEST_CALENDAR_FIELD)
      && (second <= LARGEST_MONTH);
    return readsAsMonthDay || readsAsYearMonth;
  }
  if (raw.length === MEDIUM_DATE_DIGIT_COUNT) {
    /**
     Month field of a `YYMMDD` token.
     */
    const month = Math.trunc(
      Number(raw.slice(
        MEDIUM_MONTH_START,
        MEDIUM_DAY_START,
      ),),
    );
    /**
     Day field of a `YYMMDD` token.
     */
    const day = Math.trunc(
      Number(raw.slice(MEDIUM_DAY_START,),),
    );
    return (month >= SMALLEST_CALENDAR_FIELD) && (month <= LARGEST_MONTH)
      && (day >= SMALLEST_CALENDAR_FIELD)
      && (day <= LARGEST_DAY);
  }
  if (raw.length === LONG_DATE_DIGIT_COUNT) {
    /**
     Year field of a `YYYYMMDD` token.
     */
    const year = Math.trunc(
      Number(raw.slice(
        0,
        LONG_MONTH_START,
      ),),
    );
    /**
     Month field of a `YYYYMMDD` token.
     */
    const month = Math.trunc(
      Number(raw.slice(
        LONG_MONTH_START,
        LONG_DAY_START,
      ),),
    );
    /**
     Day field of a `YYYYMMDD` token.
     */
    const day = Math.trunc(
      Number(raw.slice(LONG_DAY_START,),),
    );
    return (year >= SMALLEST_YEAR) && (year <= LARGEST_YEAR)
      && (month >= SMALLEST_CALENDAR_FIELD)
      && (month <= LARGEST_MONTH)
      && (day >= SMALLEST_CALENDAR_FIELD)
      && (day <= LARGEST_DAY);
  }
  return false;
}

//endregion Date classification

//region Token classification

/**
 Classify one token as a version, a name-shape word, or a word plus a version.

 A token counts as version evidence only when its digits run to the end, optionally
 behind a `v` and optionally behind an alphabetic prefix. That splits `qwen3.8` into
 the word `qwen` and the version `3.8` while leaving `70b`, `a12b`, and `4o` as single
 words, which is what keeps parameter sizes and glued names out of version ordering.

 @param token - one token from {@link splitOnNonTokenCharacters}

 @returns word contribution and raw version components

 @example
 ```typescript
 classifyToken('qwen3.8'); // { word: 'qwen', raws: ['3', '8'] }
 classifyToken('70b'); // { word: '70b', raws: [] }
 ```
 */
export function classifyToken(token: string,): TokenClassification {
  if (token.length === 0)
    return { raws: [], };
  /**
   Numeric run covering the whole token, when the token is purely numeric.
   */
  const bareRun = readNumericRun({
    text: token,
    start: 0,
  },);
  if ((bareRun !== NO_NUMERIC_RUN) && (bareRun.end === token.length))
    return { raws: bareRun.raws, };
  if (token.startsWith(VERSION_MARKER)) {
    /**
     Numeric run behind the version marker.
     */
    const markedRun = readNumericRun({
      text: token,
      start: VERSION_MARKER.length,
    },);
    if ((markedRun !== NO_NUMERIC_RUN) && (markedRun.end === token.length))
      return { raws: markedRun.raws, };
  }
  /**
   Length of the leading alphabetic run.
   */
  const prefixLength = alphaPrefixLength(token,);
  if (prefixLength === 0)
    return {
      word: token,
      raws: [],
    };
  /**
   Index where digits may start after the alphabetic prefix and an optional marker.
   */
  const digitStart = token.charAt(prefixLength,) === VERSION_MARKER
    ? prefixLength + VERSION_MARKER.length
    : prefixLength;
  /**
   Numeric run trailing the alphabetic prefix.
   */
  const trailingRun = readNumericRun({
    text: token,
    start: digitStart,
  },);
  if ((trailingRun !== NO_NUMERIC_RUN) && (trailingRun.end === token.length)) {
    return {
      word: token.slice(
        0,
        prefixLength,
      ),
      raws: trailingRun.raws,
    };
  }
  return {
    word: token,
    raws: [],
  };
}

/**
 Extract the age evidence from one model id.

 @param modelId - model id exactly as the catalog carries it

 @returns name shape, version components, their raw text, and the date-shaped subset

 @example
 ```typescript
 parseModelId('deepseek-v4-pro-0813');
 // { nameShape: 'deepseek-pro', versionParts: [4, 813], versionRaws: ['4', '0813'], dateRaws: ['0813'] }
 ```
 */
export function parseModelId(modelId: string,): ModelIdParse {
  /**
   Name-shape tokens in id order.
   */
  const words: string[] = [];
  /**
   Version components in id order.
   */
  const versionParts: number[] = [];
  /**
   Raw text of each version component.
   */
  const versionRaws: string[] = [];
  /**
   Raw text of each date-shaped version component.
   */
  const dateRaws: string[] = [];
  /**
   Id text with the alias marker and any organization segment removed.
   */
  const normalizedId = stripOrganizationPrefix(modelId,);
  for (const token of splitOnNonTokenCharacters(normalizedId,)) {
    /**
     Classification of the current token.
     */
    const classification = classifyToken(token,);
    if (classification.word !== undefined)
      words.push(classification.word,);
    for (const raw of classification.raws) {
      versionParts.push(
        Math.trunc(Number(raw,),),
      );
      versionRaws.push(raw,);
      if (isDateShapedRaw(raw,))
        dateRaws.push(raw,);
    }
  }
  return {
    nameShape: words.join(NAME_SHAPE_SEPARATOR,),
    versionParts,
    versionRaws,
    dateRaws,
  };
}

//endregion Token classification
