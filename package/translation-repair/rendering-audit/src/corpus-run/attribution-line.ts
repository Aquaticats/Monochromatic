//region Attribution line
// A SIGNATURE LINE AS THE ORIGINAL AND THE PAGE WRITE IT: a blockquote mark
// or an HTML tag may open the line, then one or more em dashes, then the
// signer's name, then a comma and the date, or (a song credit, class
// eighty-three) a bracketed work title 【...】 or 《...》. The name is what
// this module reads and where it stands, so a page-assembly pass can put
// another rendering in its place without touching the rest of the line.

/**
 Em dash the original opens a signature with, usually doubled.
 */
const EM_DASH = '—';

/**
 Marks that end a signer's name: the comma before a date, or the bracket
 opening a work's title in a song credit (—— 雨狸【妄想症】《零重祈愿》).
 */
const NAME_ENDS: readonly string[] = [
  ',',
  '，',
  '【',
  '《',
];

/**
 Marks that end a signer's name only after its first character: the ASCII
 bracket a page writes for the original's 【album】 (「—— Yuli [album], “title”」,
 class one hundred thirty-nine, XingZ6012), which would otherwise ride into
 the name up to the comma. At the name's first character the bracket opens a
 linked name instead, so it ends nothing there.
 */
const INNER_NAME_ENDS: readonly string[] = ['[',];

/**
 Words that open a source credit ("from", "excerpted from", "quoted from")
 rather than name a signer: 「——来自《title》，作者 handle」 credits a work, and
 the signer, where there is one, stands after 作者 (class one hundred
 thirty-eight, XingZ6011). 来自 is the one the corpus writes; the rest say
 the same thing.
 */
const CREDIT_LEADS: ReadonlySet<string> = new Set([
  '来自',
  '摘自',
  '出自',
  '选自',
  '引自',
  '转自',
  '节选自',
],);

/**
 Where a signature line's name stands.
 */
export type Signature = {
  /**
   Name as written, trimmed.
   */
  readonly name: string;

  /**
   Offset the name starts at, after the dashes and any spaces.
   */
  readonly nameStart: number;

  /**
   Offset of the comma ending the name.
   */
  readonly nameEnd: number;
};

/**
 What a line read as a signature says: the signature, or that the line is
 none (it opens with no em dash, or carries no comma after the name).
 */
export type SignatureReading =
  | {
    readonly signed: true;

    /**
     Signature the line carries.
     */
    readonly signature: Signature;
  }
    | { readonly signed: false; };

/**
 Offset past the line's blockquote marks, whitespace and opening HTML tags.

 @param line - one line

 @returns Offset of the first character of the line's own text

 @example
 ```ts
 pastPrefix({ line: '> <p>——Cat, today</p>', },); // 5
 ```
 */
function pastPrefix({ line, }: { readonly line: string; },): number {
  /**
   Offset scanned to.
   */
  let at = 0;
  while (at < line.length) {
    /**
     Character at the offset.
     */
    const character = line[at];
    /**
     Whether the character is a blockquote mark or blank.
     */
    const isMarkOrBlank = (character === ' ')
      || (character === '\t')
      || (character === '>');
    if (isMarkOrBlank) {
      at += 1;
    } else if (character === '<') {
      /**
       Where the tag closes, -1 when it never does.
       */
      const close = line.indexOf(
        '>',
        at,
      );
      if (close === (-1))
        break;
      at = close + 1;
    } else {
      break;
    }
  }
  return at;
}

/**
 Offset the name starts at: past the dashes and the spaces after them.

 @param line - signature line

 @param start - offset of the first dash

 @returns Offset of the name's first character

 @example
 ```ts
 nameStartAt({ line: '—— Cat, today', start: 0, },); // 3
 ```
 */
function nameStartAt(
  {
    line,
    start,
  }: {
    readonly line: string;
    readonly start: number;
  },
): number {
  /**
   Offset scanned to.
   */
  let at = start;
  while (line[at] === EM_DASH)
    at += 1;
  while (line[at] === ' ')
    at += 1;
  return at;
}

/**
 Offset of the earliest name-ending mark at or past an offset, -1 for none.

 @param line - signature line

 @param from - offset the name starts at

 @returns Offset of the mark ending the name

 @example
 ```ts
 nameEndAt({ line: '——Cat, today', from: 2, },); // 5
 ```
 */
function nameEndAt(
  {
    line,
    from,
  }: {
    readonly line: string;
    readonly from: number;
  },
): number {
  /**
   Earliest mark so far, -1 for none.
   */
  let earliest = -1;
  for (const mark of [
    ...NAME_ENDS,
    ...INNER_NAME_ENDS,
  ]) {
    /**
     Where this mark stands; an inner mark is looked for past the name's first
     character.
     */
    const at = line.indexOf(
      mark,
      INNER_NAME_ENDS.includes(mark,) ? from + 1 : from,
    );
    /**
     Whether this mark stands before any found so far.
     */
    const isEarlier = (earliest === (-1)) || (at < earliest);
    if ((at !== (-1)) && isEarlier)
      earliest = at;
  }
  return earliest;
}

/**
 Reads the signer's name off a signature line.

 @param line - one line of the original, the archive or the page

 @returns Signature and where its name stands, or that the line is none

 @example
 ```ts
 readSignature({ line: '——Cat, today', },); // { signed: true, signature: { name: 'Cat', nameStart: 2, nameEnd: 5, }, }
 ```
 */
export function readSignature({ line, }: { readonly line: string; },): SignatureReading {
  /**
   Where the line's own text opens.
   */
  const start = pastPrefix({ line, },);
  if (!line.startsWith(
    EM_DASH,
    start,
  ))
    return { signed: false, };
  /**
   Where the name starts.
   */
  const nameStart = nameStartAt({
    line,
    start,
  },);
  /**
   Where the name ends.
   */
  const nameEnd = nameEndAt({
    line,
    from: nameStart,
  },);
  if (nameEnd === (-1))
    return { signed: false, };
  /**
   Name as written.
   */
  const name = line.slice(
    nameStart,
    nameEnd,
  )
    .trim();
  if ((name === '') || CREDIT_LEADS.has(name,))
    return { signed: false, };
  return {
    signed: true,
    signature: {
      name,
      nameStart,
      nameEnd,
    },
  };
}

/**
 Signatures of one text, in line order.

 @param text - one slice's text

 @returns Signature per signature line

 @example
 ```ts
 const signatures = signaturesOf({ text, },);
 ```
 */
export function signaturesOf({ text, }: { readonly text: string; },): readonly Signature[] {
  return text.split('\n',)
    .flatMap(function read(line,): readonly Signature[] {
      /**
       What the line says.
       */
      const reading = readSignature({ line, },);
      return reading.signed ? [reading.signature,] : [];
    },);
}

//endregion Attribution line
