/**
 Tests for the lone container tag masker's tag read.

 A LINE IS A CONTAINER TAG AND NOTHING ELSE to be masked, and the read
 refuses the shapes that name no element: a name that is no letter word, and
 a closer carrying anything past its name.

 FIXTURES ARE CAT-THEMED.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { maskLoneContainerTags, } from '../dist/final/node/index.mjs';
import {
  grammarAcceptsDocument,
  tagNameSpellings,
} from './tag-name-spellings.test-fixture.ts';

//region Mask container tags tests

/**
 Whitespace the strict grammar steps over between a tag's name and the rest of
 the tag (`unicodeWhitespace` of `micromark-util-character`, which is the
 ECMAScript `\s`), less the line feed, which a one-line read cannot hold.
 */
const STEPPED_OVER: readonly string[] = [
  '\t',
  '\u{000B}',
  '\u{000C}',
  '\r',
  ' ',
  '\u{00A0}',
  '\u{1680}',
  '\u{2000}',
  '\u{2001}',
  '\u{2002}',
  '\u{2003}',
  '\u{2004}',
  '\u{2005}',
  '\u{2006}',
  '\u{2007}',
  '\u{2008}',
  '\u{2009}',
  '\u{200A}',
  '\u{2028}',
  '\u{2029}',
  '\u{202F}',
  '\u{205F}',
  '\u{3000}',
  '\u{FEFF}',
];

/**
 Characters the strict grammar refuses there, which look like whitespace or
 like nothing: a zero-width space, a next line mark, a Mongolian vowel
 separator, a word joiner, the zero-width joiners, a soft hyphen, a Hangul
 filler, a combining grapheme joiner and a letter.
 */
const REFUSED: readonly string[] = [
  '\u{200B}',
  '\u{0085}',
  '\u{180E}',
  '\u{2060}',
  '\u{200C}',
  '\u{200D}',
  '\u{00AD}',
  '\u{3164}',
  '\u{034F}',
  'x',
];

/**
 One place a separator may stand in a tag, as the document the strict grammar
 is asked and the slice the masker is handed.

 @example
 ```ts
 const form: SeparatorForm = { form: 'closer', grammar: '</b >', slice: '</b >', };
 ```
 */
type SeparatorForm = {
  /**
   Where in a tag the separator stands.
   */
  readonly form: string;

  /**
   Document the grammar reads whole when the separator is whitespace to it.
   */
  readonly grammar: string;

  /**
   Slice the masker finds every tag of paired when the separator is whitespace to it.
   */
  readonly slice: string;
};

/**
 The places a separator may stand in a tag: before an opener's bracket,
 before an opener's attribute, before a closer's bracket, and before the
 bracket of a tag that shares its line with content. A tag sharing its line
 cannot be half of a whole element to the grammar, which pairs a block tag with
 a block tag, so the grammar is asked of the same tag inside one line of text
 and the masker of it beside a tag line.

 @param separator - what stands between the name and the rest of the tag

 @returns Forms, in a fixed order

 @example
 ```ts
 const forms = separatorForms({ separator: '\u{00A0}', },);
 ```
 */
function separatorForms({ separator, }: { readonly separator: string; },): readonly SeparatorForm[] {
  /**
   The forms whose tag stands inside a line of text, which a carriage return
   cannot be asked of: Markdown reads it there as a line ending, and the
   grammar refuses the tag that crosses one.
   */
  const sharing: readonly SeparatorForm[] = (separator === '\r')
    ? []
    : [
      {
        form: 'opener sharing its line with content',
        grammar: `A <blockquote${separator}>cat</blockquote> naps.\n`,
        slice: `A <blockquote${separator}>cat naps.\n\n</blockquote>\n`,
      },
      {
        form: 'closer sharing its line with content',
        grammar: `A <blockquote>cat</blockquote${separator}> naps.\n`,
        slice: `<blockquote>\n\nA cat naps.</blockquote${separator}>\n`,
      },
    ];
  return [
    {
      form: 'opener before its bracket',
      grammar: `<details${separator}>\n\nA cat naps.\n\n</details>\n`,
      slice: `<details${separator}>\n\nA cat naps.\n\n</details>\n`,
    },
    {
      form: 'opener before its attribute',
      grammar: `<details${separator}open>\n\nA cat naps.\n\n</details>\n`,
      slice: `<details${separator}open>\n\nA cat naps.\n\n</details>\n`,
    },
    {
      form: 'closer before its bracket',
      grammar: `<details>\n\nA cat naps.\n\n</details${separator}>\n`,
      slice: `<details>\n\nA cat naps.\n\n</details${separator}>\n`,
    },
    ...sharing,
  ];
}

/**
 Whether the masker finds every tag of a document paired, masking none.

 @param document - document under the masker

 @returns True when no tag is reported lone

 @example
 ```ts
 const paired = maskerPairs({ document: '<details>\n\nA cat naps.\n\n</details>\n', },);
 ```
 */
function maskerPairs({ document, }: { readonly document: string; },): boolean {
  return maskLoneContainerTags({ text: document, },)
    .tags
    .length === 0;
}

/**
 Whether the masker reads a lone tag of one kind in a slice.

 @param text - slice under the masker

 @param kind - which half of a container the slice holds alone

 @returns True when the masker reports exactly one lone tag, of that kind

 @example
 ```ts
 const reads = readsLoneTag({ text: '<details>\n', kind: 'open', },);
 ```
 */
function readsLoneTag(
  {
    text,
    kind,
  }: {
    readonly text: string;
    readonly kind: 'open' | 'close';
  },
): boolean {
  /**
   Tags the masker reports.
   */
  const { tags, } = maskLoneContainerTags({ text, },);
  return (tags.length === 1) && (tags.at(0,)?.kind === kind);
}

await describe({
  name: maskLoneContainerTags.name,
  children: [
    it({
      name: 'READS NO TAG where the name starts with no letter and where a closer carries anything past '
        + 'its name, since neither names an element',
      fn: async () => {
        expect(maskLoneContainerTags({ text: '<3>\n', },),).toEqual({
          masked: '<3>\n',
          tags: [],
        },);
        expect(maskLoneContainerTags({ text: '</p x>\n', },),).toEqual({
          masked: '</p x>\n',
          tags: [],
        },);
      },
    },),

    it({
      name: 'MASKS A LONE CLOSER WITH A SPACE BEFORE ITS BRACKET as the closer of its element, since the strict '
        + 'grammar reads that spelling as closing the element and refuses it alone',
      fn: async () => {
        expect(maskLoneContainerTags({ text: 'A cat naps.\n\n</details >\n', },),).toEqual({
          masked: `A cat naps.\n\n${' '.repeat(11,)}\n`,
          tags: [{
            kind: 'close',
            name: 'details',
            text: '</details >',
            startOffset: 13,
            endOffset: 24,
          },],
        },);
      },
    },),

    it({
      name: 'MASKS A LONE CLOSER WITH A TAB BEFORE ITS BRACKET under the element\'s name alone, and an opener '
        + 'whose attribute follows a tab under its name alone',
      fn: async () => {
        expect(maskLoneContainerTags({ text: 'A cat naps.\n\n</details\t>\n', },),).toEqual({
          masked: `A cat naps.\n\n${' '.repeat(11,)}\n`,
          tags: [{
            kind: 'close',
            name: 'details',
            text: '</details\t>',
            startOffset: 13,
            endOffset: 24,
          },],
        },);
        expect(maskLoneContainerTags({ text: '<details\topen>\n\nA cat naps.\n', },),).toEqual({
          masked: `${' '.repeat(14,)}\n\nA cat naps.\n`,
          tags: [{
            kind: 'open',
            name: 'details',
            text: '<details\topen>',
            startOffset: 0,
            endOffset: 14,
          },],
        },);
      },
    },),

    it({
      name: 'PAIRS AN OPENER WITH A CLOSER WRITTEN WITH A SPACE OR A TAB BEFORE ITS BRACKET, so a whole element '
        + 'is masked nowhere',
      fn: async () => {
        expect(maskLoneContainerTags({ text: '<details>\n\nA cat naps.\n\n</details >\n', },),).toEqual({
          masked: '<details>\n\nA cat naps.\n\n</details >\n',
          tags: [],
        },);
        expect(maskLoneContainerTags({ text: '<details>\n\nA cat naps.\n\n</details\t>\n', },),).toEqual({
          masked: '<details>\n\nA cat naps.\n\n</details\t>\n',
          tags: [],
        },);
      },
    },),
    it({
      name: 'READS AS WHITESPACE EVERY SEPARATOR THE STRICT GRAMMAR STEPS OVER inside a tag, before its bracket or its '
        + 'attribute, a tag sharing its line with content included, and no spelling the grammar refuses',
      fn: async () => {
        /**
         Every form of every separator with the grammar's verdict and the masker's.
         */
        const verdicts = [
          ...STEPPED_OVER,
          ...REFUSED,
        ].flatMap(function formsOf(separator,) {
          return separatorForms({ separator, },)
            .map(function verdictOf({
              form,
              grammar,
              slice,
            },) {
              return {
                spelling: `${form} with U+${
                  nonNullishOrThrow(separator.codePointAt(0,),)
                    .toString(16,)
                    .toUpperCase()
                    .padStart(
                      4,
                      '0',
                    )
                }`,
                grammar: grammarAcceptsDocument({ document: grammar, },),
                masker: maskerPairs({ document: slice, },),
              };
            },);
        },);
        expect(verdicts.filter(function refused({ grammar, },): boolean {
          return !grammar;
        },).length,).toBe(5 * REFUSED.length,);
        expect(verdicts.filter(function accepted({ grammar, },): boolean {
          return grammar;
        },).length,).toBe((5 * STEPPED_OVER.length) - 2,);
        expect(verdicts.filter(function disagrees({
          grammar,
          masker,
        },): boolean {
          return grammar !== masker;
        },).map(function spellingOf({ spelling, },): string {
          return spelling;
        },),).toEqual([],);
      },
    },),
    it({
      name: 'READS A TAG NAME AS THE STRICT GRAMMAR DOES, a lone opener and a lone closer read exactly where the '
        + 'grammar reads the same name as the tag of a whole element, over every class of character a name holds '
        + 'in each place it may stand',
      fn: async () => {
        /**
         The grammar's verdict and the masker's, for each spelling.
         */
        const verdicts = tagNameSpellings()
          .map(function verdictOf(name,) {
            return {
              name,
              grammar: grammarAcceptsDocument({ document: `<${name}>\n\nA cat naps.\n\n</${name}>\n`, },),
              opener: readsLoneTag({
                text: `<${name}>\n\nA cat naps.\n`,
                kind: 'open',
              },),
              closer: readsLoneTag({
                text: `A cat naps.\n\n</${name}>\n`,
                kind: 'close',
              },),
            };
          },);
        // Both verdicts are reached, so agreement is not agreement on nothing.
        expect(verdicts.some(function accepted({ grammar, },): boolean {
          return grammar;
        },),).toBe(true,);
        expect(verdicts.some(function refused({ grammar, },): boolean {
          return !grammar;
        },),).toBe(true,);
        expect(verdicts.filter(function disagrees({
          grammar,
          opener,
          closer,
        },): boolean {
          return (grammar !== opener) || (grammar !== closer);
        },).map(function nameOf({ name, },): string {
          return JSON.stringify(name,);
        },),).toEqual([],);
      },
    },),

    it({
      name: 'READS THE NAME OF A TAG WITHOUT THE WHITESPACE THE GRAMMAR ALLOWS AROUND ITS SEPARATORS, so a member or '
        + 'a prefixed name is one name however it is spaced, and a name in any script or opening with `$` or `_` '
        + 'is a name',
      fn: async () => {
        expect(maskLoneContainerTags({ text: '<a . b>\n', },).tags.map(function nameOf({ name, },): string {
          return name;
        },),).toEqual(['a.b',],);
        expect(maskLoneContainerTags({ text: '</a : b >\n', },).tags.map(function nameOf({ name, },): string {
          return name;
        },),).toEqual(['a:b',],);
        expect(['猫', '$cat', '_cat', 'é', 'Ω', 'cat-nap', 'cat3',].map(function nameRead(name,): string {
          return maskLoneContainerTags({ text: `<${name}>\n`, },).tags.map(function nameOf(tag,): string {
            return tag.name;
          },).join(',',);
        },),).toEqual(['猫', '$cat', '_cat', 'é', 'Ω', 'cat-nap', 'cat3',],);
      },
    },),

    it({
      name: 'LEAVES A LONE TAG WHOSE NAME HOLDS A CHARACTER THE GRAMMAR REFUSES IN THE SLICE, unmasked and reported '
        + 'as no tag, so the strict grammar refuses the slice it would have read as an element of that odd name',
      fn: async () => {
        for (
          const text of [
            '<details\u{200B}>\n\nA cat naps.\n',
            '<details\u{200B}open>\n\nA cat naps.\n',
            'A cat naps.\n\n</details\u{200B}>\n',
            '<de\u{200E}tails>\n\nA cat naps.\n',
            '<\u{200B}details>\n\nA cat naps.\n',
            '<details\u{20000}>\n\nA cat naps.\n',
          ]
        ) {
          expect(maskLoneContainerTags({ text, },),).toEqual({
            masked: text,
            tags: [],
          },);
        }
      },
    },),

    it({
      name: 'PAIRS AN OPENER WITH AN INLINE CLOSER WRITTEN WITH WHITESPACE AROUND ITS SEPARATOR, since the grammar '
        + 'reads both spellings as one name',
      fn: async () => {
        expect(maskLoneContainerTags({ text: '<a.b>\n\nA cat naps.</a . b>\n', },),).toEqual({
          masked: '<a.b>\n\nA cat naps.</a . b>\n',
          tags: [],
        },);
      },
    },),
    it({
      name: 'MASKS A LONE OPENER WHOSE TAG CROSSES A LINE BREAK, which the strict grammar steps over as whitespace, '
        + 'keeping the line breaks so every line is where it stood',
      fn: async () => {
        expect(maskLoneContainerTags({ text: '<details\n  open>\n\nA cat naps.\n', },),).toEqual({
          masked: `${' '.repeat(8,)}\n${' '.repeat(7,)}\n\nA cat naps.\n`,
          tags: [{
            kind: 'open',
            name: 'details',
            text: '<details\n  open>',
            startOffset: 0,
            endOffset: 16,
          },],
        },);
      },
    },),

    it({
      name: 'MASKS A LONE OPENER WHOSE TAG CROSSES A LINE BREAK AND ENDS THE SLICE, no line break after its bracket',
      fn: async () => {
        expect(maskLoneContainerTags({ text: '<details\n  open>', },),).toEqual({
          masked: `${' '.repeat(8,)}\n${' '.repeat(7,)}`,
          tags: [{
            kind: 'open',
            name: 'details',
            text: '<details\n  open>',
            startOffset: 0,
            endOffset: 16,
          },],
        },);
      },
    },),

    it({
      name: 'MASKS ONLY THE TAG LINE OF TWO OPENERS NOTHING CLOSES, leaving the inline opener, which only partners '
        + 'a tag line',
      fn: async () => {
        expect(maskLoneContainerTags({ text: '<details>\n\nThe cat <details> naps.\n', },),).toEqual({
          masked: `${' '.repeat(9,)}\n\nThe cat <details> naps.\n`,
          tags: [{
            kind: 'open',
            name: 'details',
            text: '<details>',
            startOffset: 0,
            endOffset: 9,
          },],
        },);
      },
    },),

    it({
      name: 'MASKS A LONE CLOSER WHOSE ONLY OPENER IS AN INLINE TAG THAT NEVER ENDS, its quoted value running to '
        + 'the end of the slice, since that opener is no tag to partner',
      fn: async () => {
        expect(maskLoneContainerTags({ text: 'The cat <details title="nap\n\n</details>\n', },),).toEqual({
          masked: `The cat <details title="nap\n\n${' '.repeat(10,)}\n`,
          tags: [{
            kind: 'close',
            name: 'details',
            text: '</details>',
            startOffset: 29,
            endOffset: 39,
          },],
        },);
      },
    },),

    it({
      name: 'MASKS A LONE CLOSER WHOSE BRACKET STANDS ON THE NEXT LINE, and pairs an opener across lines with its closer',
      fn: async () => {
        expect(maskLoneContainerTags({ text: 'A cat naps.\n\n</details\n>\n', },),).toEqual({
          masked: `A cat naps.\n\n${' '.repeat(9,)}\n${' '.repeat(1,)}\n`,
          tags: [{
            kind: 'close',
            name: 'details',
            text: '</details\n>',
            startOffset: 13,
            endOffset: 24,
          },],
        },);
        expect(maskLoneContainerTags({ text: '<details\n  open>\n\nA cat naps.\n\n</details>\n', },),).toEqual({
          masked: '<details\n  open>\n\nA cat naps.\n\n</details>\n',
          tags: [],
        },);
      },
    },),

    it({
      name: 'READS NO TAG in a line opening like one that no bracket ends before a blank line or that text follows '
        + 'the bracket of, which is prose',
      fn: async () => {
        for (const text of [
          '<b a cat naps\n\nand a bracket > much later\n',
          '<b\n> and a cat naps\n',
          '<b a cat naps\n',
        ]) {
          expect(maskLoneContainerTags({ text, },),).toEqual({
            masked: text,
            tags: [],
          },);
        }
      },
    },),

    it({
      name: 'MASKS A LINE WHOSE BRACKET IS FOLLOWED BY SPACES, TABS AND A CARRIAGE RETURN ALONE, the white space Markdown ends a line '
        + 'with (n156, W5), and masks the tag only',
      fn: async () => {
        expect(maskLoneContainerTags({ text: '<details> \t\r\nA cat naps.\r\n', },),).toEqual({
          masked: `${' '.repeat(9,)} \t\r\nA cat naps.\r\n`,
          tags: [{
            kind: 'open',
            name: 'details',
            text: '<details>',
            startOffset: 0,
            endOffset: 9,
          },],
        },);
      },
    },),

    it({
      name: 'READS NO TAG IN A LINE WHOSE BRACKET IS FOLLOWED BY ANY WHITE SPACE BUT SPACE AND TAB (n156, W5): a no-break space, a '
        + 'wide space, a vertical tab, a form feed, a byte order mark or an ideographic space makes the line text the MDX parser '
        + 'keeps, so no mask turns part of it into spaces and leaves the rest standing',
      fn: async () => {
        for (const after of [
          '\u{00A0}',
          '\u{2003}',
          '\u{3000}',
          '\u{000B}',
          '\u{000C}',
          '\u{FEFF}',
        ]) {
          /**
           A line holding a tag and one character past its bracket, between a line of text and a line of one soft hyphen and a mark.
           */
          const text = `The kittens\n<Note>${after}\r\n\u{00AD}’ x`;
          expect(maskLoneContainerTags({ text, },),).toEqual({
            masked: text,
            tags: [],
          },);
        }
      },
    },),
  ],
},);

//endregion Mask container tags tests
