/**
 Reading characters and whole numbers out of a failure text: the digits a status code is read from, the separators a number written
 after a key may be surrounded by, and the one pass that reads such a number.

 Every text is folded to lower case before it reaches this module, so each test here is one comparison per character and nothing
 can backtrack.

 @module
 */

/**
 What may stand between a wait's key and its number: whitespace of any kind, quotes, a colon and an equals sign. A body
 pretty-printed over several lines puts a line break between the key and its number, which is still the wait it asks for.
 */
const WAIT_SEPARATORS: ReadonlySet<string> = new Set([
  '"',
  '\'',
  ' ',
  '\t',
  '\n',
  '\r',
  ':',
  '=',
]);

/**
 Tells whether a character is an ASCII digit.

 @param character - one character of the text

 @returns whether it is a digit
 */
function isDigit(character: string,): boolean {
  return (character >= '0') && (character <= '9');
}

/**
 Tells whether a character is an ASCII letter or digit, so a number that touches one is part of a longer word, count or request
 number.

 @param character - one character of the text

 @returns whether it is a letter or a digit
 */
function isLetterOrDigit(character: string,): boolean {
  return (isDigit(character,)) || ((character >= 'a') && (character <= 'z'));
}

/**
 Finds the status code a failure text starts with.

 A response writes its status before the body that explains it, so a number anywhere else is a count, a request number or a limit
 the provider states, and none of those is a status. Reading only the first word is what keeps `max_tokens must not exceed 500`
 from being read as a server error another attempt may fix.

 @param text - text folded to lower case

 @returns the digits of the status, or an empty string when the text starts with anything else

 @example
 ```ts
 const status = leadingStatusCode('429 too many requests',);
 ```
 */
export function leadingStatusCode(text: string,): string {
  /**
   How many digits a status code has.
   */
  const statusDigits = 3;
  for (let at = 0; at < statusDigits; at += 1) {
    if (!isDigit(text.charAt(at,),)) {
      return '';
    }
  }
  // A letter or digit right after those three digits makes them part of a longer word or number, which is no status.
  return isLetterOrDigit(text.charAt(statusDigits,),) ? '' : text.slice(
    0,
    statusDigits,
  );
}

/**
 Finds where the separators between a wait's key and its number end.

 @param text - text folded to lower case

 @param from - where to start looking

 @returns the first index holding anything else, or the end of the text
 */
function firstIndexOutside({
  text,
  from,
}: {
  readonly text: string;
  readonly from: number
},): number {
  for (let at = from; at < text.length; at += 1) {
    if (!WAIT_SEPARATORS.has(text.charAt(at,),)) {
      return at;
    }
  }
  return text.length;
}

/**
 Finds where a run of digits ends.

 @param text - text folded to lower case

 @param from - where to start looking

 @returns the first index that is no digit, or the end of the text
 */
function firstIndexNotDigit({
  text,
  from,
}: {
  readonly text: string;
  readonly from: number
},): number {
  for (let at = from; at < text.length; at += 1) {
    if (!isDigit(text.charAt(at,),)) {
      return at;
    }
  }
  return text.length;
}

/**
 The number a text names after one of these keys, or that it names none of them.
 */
export type NamedSeconds =
  | { readonly named: false }
    | {
      readonly named: true;

    /**
     The number of seconds the text names.
     */
      readonly seconds: number
    };

/**
 Reads the first whole number of seconds that follows one of these keys, on its own or after separators.

 @param text - text folded to lower case

 @param keys - keys to look for

 @returns the seconds it names, or that the text names none of them

 @example
 ```ts
 const named = secondsNamedBy({ text: '{"retry_after":60}', keys: ['retry_after'], },);
 ```
 */
export function secondsNamedBy({
  text,
  keys,
}: {
  readonly text: string;
  readonly keys: readonly string[]
},): NamedSeconds {
  /**
   Where the earliest key ends, and nothing until one is found.
   */
  const afterKey = keys.reduce(
    function earlierKeyEnd(
      earliest,
      key,
    ) {
      /**
       Where this key starts, when the text holds it.
       */
      const found = text.indexOf(key,);
      if (found === (-1)) {
        return earliest;
      }
      /**
       Where this key ends.
       */
      const end = found + key.length;
      return ((earliest === (-1)) || (end < earliest)) ? end : earliest;
    },
    -1,
  );
  if (afterKey === (-1)) {
    return { named: false, };
  }
  /**
   Where the number starts, once every separator between it and its key is passed.
   */
  const start = firstIndexOutside({
    text,
    from: afterKey,
  },);
  /**
   Where the number ends.
   */
  const end = firstIndexNotDigit({
    text,
    from: start,
  },);
  if (end === start) {
    return { named: false, };
  }
  /**
   The number the text names.
   */
  const seconds = Number(text.slice(
    start,
    end,
  ),);
  return Number.isFinite(seconds,) ? {
    named: true,
    seconds,
  } : { named: false, };
}
