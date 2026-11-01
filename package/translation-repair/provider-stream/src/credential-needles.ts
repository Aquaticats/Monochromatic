//region Credential needles
// Every text a credential can be searched for as, in a reply that may carry it
// plainly, written for a JSON string, written as a form value, or as part of a
// base64 run whose start the reader cannot know.
//
// BASE64 NEEDS ONE NEEDLE PER ALIGNMENT. Base64 writes three bytes as four
// characters, so where the token falls against those groups depends on how many
// bytes came before it, which the reader of a reply does not know. The three
// possible alignments give three texts. In each, the characters at the edges
// that share bits with a neighbouring byte could be any of a few, so the needle
// is the stretch between them, which is the same whatever the neighbours are,
// and the mask widens a find by the one character at each edge that holds
// bits of the token. What it hides of the neighbours is that one character.

/**
 A text to search for, and how far a find of it reaches past itself.
 */
export type Needle = {
  /**
   The text to find.
   */
  readonly text: string;

  /**
   Characters before the find that hold bits of the credential too.
   */
  readonly before: number;

  /**
   Characters after the find that hold bits of the credential too. Where it is
   one, the run may end on that character and its `=` padding, which the mask
   takes with it.
   */
  readonly after: number;
};

/**
 Bytes base64 writes as one group.
 */
const BYTES_PER_GROUP = 3;

/**
 Characters base64 writes a group as.
 */
const CHARS_PER_GROUP = 4;

/**
 Bytes of prefix, per alignment, that a token may follow in a base64 run: the
 alignment is the prefix length modulo the group.
 */
const ALIGNMENTS: readonly number[] = [
  0,
  1,
  2,
];

/**
 Writes bytes as unpadded base64.

 @param bytes - bytes to write

 @returns The standard-alphabet text without `=` padding

 @example
 ```ts
 base64Of({ bytes: new Uint8Array([99, 97, 116,],), },); // 'Y2F0'
 ```
 */
function base64Of({ bytes, }: { readonly bytes: Uint8Array; },): string {
  return btoa(String.fromCodePoint(...bytes,),)
    .replaceAll(
    '=',
    '',
    );
}

/**
 Writes a standard-alphabet base64 text in the url-safe alphabet.

 @param text - standard-alphabet text

 @returns The same text with `-` for `+` and `_` for `/`

 @example
 ```ts
 urlSafeOf({ text: 'a+b/c', },); // 'a-b_c'
 ```
 */
function urlSafeOf({ text, }: { readonly text: string; },): string {
  return text
    .replaceAll(
    '+',
    '-',
    )
    .replaceAll(
    '/',
    '_',
    );
}

/**
 Lists the base64 needles of a credential: each alignment, in the standard and
 the url-safe alphabet.

 @param credential - credential to write

 @returns Needles of the characters the credential's own bytes fix

 @example
 ```ts
 base64NeedlesOf({ credential: 'whisker-key-7421', },);
 ```
 */
function base64NeedlesOf({ credential, }: { readonly credential: string; },): readonly Needle[] {
  /**
   The credential's UTF-8 bytes.
   */
  const bytes = new TextEncoder().encode(credential,);
  return ALIGNMENTS.flatMap(function alignedAt(alignment,): readonly Needle[] {
    /**
     Bytes of the run up to the credential's end, the prefix standing as zeros.
     */
    const total = alignment + bytes.length;
    /**
     The whole run in the standard alphabet.
     */
    const run = base64Of({
      bytes: new Uint8Array([
        ...new Uint8Array(alignment,),
        ...bytes,
      ],),
    },);
    /**
     Bytes past the last whole group, zero to two.
     */
    const tail = total % BYTES_PER_GROUP;
    /**
     Characters the credential's bytes fix: every full group, and of the tail's
     characters as many as the tail has bytes, since a tail of one byte fixes
     its first character and one of two bytes its first two, the next one
     sharing bits with whatever follows.
     */
    const end = (CHARS_PER_GROUP * Math.floor(total / BYTES_PER_GROUP,)) + tail;
    /**
     The fixed stretch in the standard alphabet.
     */
    const standard = run.slice(
      // A prefix of one byte leaves the first group's first character to the
      // prefix alone and its second to the prefix and the token together; a prefix
      // of two bytes leaves the first two to the prefix alone and the third to
      // both. The needle starts after the last of them.
      (alignment === 0) ? 0 : (alignment + 1),
      end,
    );
    return [
      standard,
      urlSafeOf({ text: standard, },),
    ].map(function needleOf(text,): Needle {
      return {
        text,
        before: (alignment === 0) ? 0 : 1,
        after: (tail === 0) ? 0 : 1,
      };
    },);
  },);
}

/**
 Keeps the first of every needle that reads the same: the same text reaching
 the same distance each way.

 @param needles - needles, some of them repeated

 @returns The needles with each kind once, in the order first met

 @example
 ```ts
 distinctNeedles({ needles: [first, first,], },); // [first]
 ```
 */
export function distinctNeedles({ needles, }: { readonly needles: readonly Needle[]; },): readonly Needle[] {
  return needles.filter(function isFirstOfItsKind(
    needle,
    at,
  ): boolean {
    return needles.findIndex(function sameKind(other,): boolean {
      return (other.text === needle.text)
        && (other.before === needle.before)
        && (other.after === needle.after);
    },) === at;
  },);
}

/**
 Lists the texts a credential can be found as in a reply that has not been
 decoded: as it stands, as a JSON writer escapes it, with its slashes escaped
 too, with each space a plus (a form value), as the bytes of its UTF-8 written
 one unit each (what a percent decoder returns for a credential beyond ASCII),
 and in base64.

 @param credential - credential to spell

 @returns Distinct needles, the credential as it stands first

 @example
 ```ts
 needlesOf({ credential: 'a/b"c-0123456789', },);
 ```
 */
export function needlesOf({ credential, }: { readonly credential: string; },): readonly Needle[] {
  /**
   The credential between the quotes JSON writes around it.
   */
  const escaped = JSON.stringify(credential,)
    .slice(
    1,
    -1,
    );
  /**
   Spellings of the text, before the plus form of each is added.
   */
  const plain = [
    credential,
    escaped,
    escaped.replaceAll(
      '/',
      String.raw`\/`,
    ),
    String.fromCodePoint(...new TextEncoder().encode(credential,),),
  ];
  /**
   Every needle, some of them repeated.
   */
  const every: readonly Needle[] = [
    ...[
      ...plain,
      ...plain.map(function plusFormOf(text,): string {
        return text.replaceAll(
          ' ',
          '+',
        );
      },),
    ].map(function plainNeedle(text,): Needle {
      return {
        text,
        before: 0,
        after: 0,
      };
    },),
    ...base64NeedlesOf({ credential, },),
  ];
  return distinctNeedles({ needles: every, },);
}

//endregion Credential needles
