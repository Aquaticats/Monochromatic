import {
  codePointAt,
  codePointBefore,
  isAsciiAlphanumeric,
  isLetterOrDigitOfAnyScript,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Consistency addresses
// WHERE A WEB OR MAIL ADDRESS STANDS IN A PLACE'S ENGLISH, for the locator of a
// judge's quote (`consistency-fold.ts`) alone. A quote must not be found inside a
// link: a carry that replaced `cats` in `https://cats.example.org/page` would
// edit the address. The locator reads the English as text and no caller of it
// holds a parse, so the rule is a rule about text, not about Markdown. A RUN is a
// stretch of characters between two spaces or line breaks (the fold writes every
// run of whitespace as one of those), and the ADDRESS in a run is:
//
// - from the scheme to the end of the run, in a run that holds `://`: the scheme
//   is the letters, digits, `+`, `-` and `.` directly before the `://`, so
//   `cats](https://cats.example.org/page)` has its address from `https`, and the
//   `cats` of a link's text, which stands before it, is no part of it;
// - the whole run, in a run that starts with `www.`;
// - from the start of the local part to the end of the run, in a run that holds
//   an `@` with a letter or a digit directly on each side and a `.` somewhere
//   after it: the local part is the letters, digits, `.`, `_`, `%`, `+` and `-`
//   directly before the `@`. A run with an `@` and no `.` after it (`cats@home`)
//   is no address.
//
// Nothing else is read: a bare domain without `www.` or a scheme (`home.org`) is no
// address here. A stand of a quote is no stand when it lies wholly inside an
// address; a stand that only reaches into one is not read here. THE READING IS ONE
// FORWARD PASS: each character is looked at once to find the runs and what each
// holds, and the one backward step from a scheme or an `@` to the start of its
// address stays inside its run, so the work is linear in the English.

/**
 One address in a text: the offset where it starts and the exclusive offset where
 its run ends.

 @example
 ```ts
 const range: AddressRange = { start: 6, end: 36, };
 ```
 */
export type AddressRange = {
  /**
   Offset in the text where the address starts.
   */
  readonly start: number;

  /**
   Exclusive offset where it ends, the end of its run.
   */
  readonly end: number;
};

/**
 What the forward pass has met of the run it is in.
 */
type RunSoFar = {
  /**
   Offset where the run starts.
   */
  start: number;

  /**
   Offset of the first `://` in the run, `-1` when it holds none.
   */
  scheme: number;

  /**
   Offset of the first `@` with a letter or a digit on each side, `-1` when the run holds none.
   */
  mail: number;

  /**
   Offset of the last `.` in the run, `-1` when it holds none.
   */
  lastDot: number;
};

/**
 Characters that end a run: the fold writes each run of whitespace as a space or
 as a blank line.
 */
const RUN_ENDS: ReadonlySet<string> = new Set([
  ' ',
  '\n',
],);

/**
 Marks other than letters and digits that a scheme holds.
 */
const SCHEME_MARKS: ReadonlySet<string> = new Set([
  '+',
  '-',
  '.',
],);

/**
 The text between the scheme and the rest of a web address.
 */
const SCHEME_SEPARATOR = '://';

/**
 The lower-case start of a web address written without a scheme.
 */
const WEB_PREFIX = 'www.';

/**
 Marks other than letters and digits that the local part of a mail address holds.
 */
const LOCAL_MARKS: ReadonlySet<string> = new Set([
  '.',
  '_',
  '%',
  '+',
  '-',
],);

/**
 The offset the unit "none" is written as.
 */
const NONE = -1;

/**
 Whether a character is a letter or a digit of any script, which is what a mail
 address needs directly on each side of its `@`. This is not the test of a single mark's
 neighbour (a Latin letter or an ASCII digit, `latin-letters.ts`): an address is read as
 an address in any script.

 @param character - one whole character, empty past an edge of the text

 @returns Whether it is a letter or a digit

 @example
 ```ts
 const letter = attachesAddress({ character: 't', },); // true
 ```
 */
function attachesAddress({ character, }: { readonly character: string; },): boolean {
  return isLetterOrDigitOfAnyScript({ character, },);
}

/**
 Where the address of a run starts, going back from a scheme or an `@` over the
 characters that can belong to it without leaving the run.

 @param text - the text read

 @param from - offset of the `://` or the `@` the address is anchored on

 @param floor - offset where the run starts, which the step back does not pass

 @param belongs - whether a whole character can belong to the stretch before the anchor

 @returns The offset of the first character of the stretch

 @example
 ```ts
 const start = addressStartBefore({ text: 'See https://a.org', from: 9, floor: 4, belongs: function isLetter(character,): boolean { return character !== ' '; }, },); // 4
 ```
 */
function addressStartBefore(
  {
    text,
    from,
    floor,
    belongs,
  }: {
    readonly text: string;
    readonly from: number;
    readonly floor: number;
    readonly belongs: (character: string) => boolean;
  },
): number {
  /**
   The offset reached so far, mutated only inside this function.
   */
  const reached = { at: from, };
  while (reached.at > floor) {
    /**
     The whole character just before the offset reached.
     */
    const before = codePointBefore({
      text,
      at: reached.at,
    },);
    if (!belongs(before,))
      break;
    reached.at -= before.length;
  }
  return reached.at;
}

/**
 The address a finished run holds, as a list of zero or one.

 @param text - the text read

 @param run - what the pass met of the run

 @param end - exclusive offset where the run ends

 @returns The address of the run, none when the run holds no address

 @example
 ```ts
 const [range,] = addressOfRun({ text, run, end: 30, },);
 ```
 */
function addressOfRun(
  {
    text,
    run,
    end,
  }: {
    readonly text: string;
    readonly run: Readonly<RunSoFar>;
    readonly end: number;
  },
): readonly AddressRange[] {
  if (run.scheme !== NONE) {
    return [
      {
        start: addressStartBefore({
          text,
          from: run.scheme,
          floor: run.start,
          belongs: function inScheme(character,): boolean {
            return isAsciiAlphanumeric({ character, },) || SCHEME_MARKS.has(character,);
          },
        },),
        end,
      },
    ];
  }
  /**
   The first characters of the run, as many as the web prefix has, in lower case.
   */
  const lead = text
    .slice(
    run.start,
    run.start + WEB_PREFIX.length,
    )
    .toLowerCase();
  if (lead === WEB_PREFIX) {
    return [
      {
        start: run.start,
        end,
      },
    ];
  }
  if ((run.mail !== NONE) && (run.lastDot > run.mail)) {
    return [
      {
        start: addressStartBefore({
          text,
          from: run.mail,
          floor: run.start,
          belongs: function inLocalPart(character,): boolean {
            return attachesAddress({ character, },) || LOCAL_MARKS.has(character,);
          },
        },),
        end,
      },
    ];
  }
  return [];
}

/**
 Every address of a text, in order.

 @param text - the English in the fold

 @returns The addresses, each from its start to the end of its run, in ascending order and not overlapping

 @example
 ```ts
 const ranges = addressRangesOf({ text: 'Visit https://cats.example.org/page now.', },); // [{ start: 6, end: 35 }]
 ```
 */
export function addressRangesOf({ text, }: { readonly text: string; },): readonly AddressRange[] {
  /**
   The addresses found, in order.
   */
  const found: AddressRange[] = [];
  /**
   What has been met of the run the pass is in, mutated only inside this function.
   */
  const run: RunSoFar = {
    start: 0,
    scheme: NONE,
    mail: NONE,
    lastDot: NONE,
  };
  for (let at = 0; at <= text.length; at += 1) {
    /**
     The unit at the offset, empty past the end of the text.
     */
    const character = text.charAt(at,);
    if ((at === text.length) || RUN_ENDS.has(character,)) {
      found.push(...addressOfRun({
        text,
        run,
        end: at,
      },),);
      run.start = at + 1;
      run.scheme = NONE;
      run.mail = NONE;
      run.lastDot = NONE;
      continue;
    }
    /**
     Whether the scheme separator starts here.
     */
    const startsSeparator = text.startsWith(
      SCHEME_SEPARATOR,
      at,
    );
    if ((character === ':')
      && (run.scheme === NONE)
      && startsSeparator)
      run.scheme = at;
    /**
     Whether a letter or a digit stands on each side of the offset.
     */
    const betweenAddressCharacters = attachesAddress({
      character: codePointBefore({
        text,
        at,
      },),
    },) && attachesAddress({
      character: codePointAt({
        text,
        at: at + 1,
      },),
    },);
    if ((character === '@')
      && (run.mail === NONE)
      && betweenAddressCharacters)
      run.mail = at;
    if (character === '.')
      run.lastDot = at;
  }
  return found;
}

/**
 The starts of the stands of a needle that do not lie wholly inside an address.

 @param stands - start offsets of the stands, in ascending order

 @param length - length of the needle, the same for every stand

 @param addresses - the addresses of the text, in ascending order and not overlapping

 @returns The stands that lie outside every address, in the order they were given

 @example
 ```ts
 const kept = standsOutsideAddresses({ stands: [10, 22,], length: 4, addresses: [{ start: 20, end: 40, },], },); // [10]
 ```
 */
export function standsOutsideAddresses(
  {
    stands,
    length,
    addresses,
  }: {
    readonly stands: readonly number[];
    readonly length: number;
    readonly addresses: readonly AddressRange[];
  },
): readonly number[] {
  /**
   Position of the first address that has not ended before the stand under test, mutated only inside this function.
   */
  const next = { at: 0, };
  return stands.filter(function standsApart(start,): boolean {
    /**
     Whether the address at the position has ended by the time the stand starts.

     @returns Whether the position is to move on
     */
    function endedBefore(): boolean {
      /**
       The address at the position, none when every address has been passed.
       */
      const current = addresses[next.at];
      return (current !== undefined) && (current.end <= start);
    }
    while (endedBefore())
      next.at += 1;
    /**
     The first address that ends after the stand starts, none when every address ends before it.
     */
    const address = addresses[next.at];
    if (address === undefined)
      return true;
    return !((address.start <= start) && ((start + length) <= address.end));
  },);
}

//endregion Consistency addresses
