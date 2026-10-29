/**
 Tests for the tag-and-attribute reader.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  parseMdxBody,
  requireMdxRefusal,
} from '@monochromatic-dev/translation-repair-document-structure';
import { readTags, } from '../../dist/final/node/index.mjs';
import {
  grammarAcceptsDocument,
  tagNameSpellings,
} from '@monochromatic-dev/translation-repair-document-structure/ts/tag-name-spellings.test-fixture.ts';

/**
 A node of the strict parse as this file reads it: its kind and, on a tag,
 its attributes with their kind and name. The MDX nodes are not among the
 types the parse's root declares, so they are read by shape.
 */
type ParsedTag = {
  readonly type: string;
  readonly attributes?: readonly {
    readonly type: string;
    readonly name?: unknown;
  }[];
};

/**
 Names the strict grammar reads for the attributes of the one tag a document
 opens with, or none where the grammar refuses the document.

 @param document - one self-closing tag

 @returns Attribute names as the parse writes them, a prefixed name without
 the whitespace around its colon

 @throws Error when the document parses to anything but a tag first, which no
 document this file writes does

 @example
 ```ts
 const names = grammarAttributeNames({ document: '<Cat n="1"/>\n', },); // ['n']
 ```
 */
function grammarAttributeNames({ document, }: { readonly document: string; },): readonly string[] {
  try {
    /**
     The node the document opens with.
     */
    const [element,]: readonly ParsedTag[] = parseMdxBody({ body: document, },).children;
    if ((element?.type !== 'mdxJsxFlowElement') || (element.attributes === undefined))
      throw new Error(`a document of one tag parsed to ${String(element?.type,)} first`,);
    return element.attributes.map(function nameOf(attribute,): string {
      return ((typeof attribute.name) === 'string') ? attribute.name : attribute.type;
    },);
  }
  catch (error) {
    // Only the grammar's own refusal reads as a document it does not accept.
    requireMdxRefusal({ error, },);
    return [];
  }
}

/**
 Whole reading of the one readable tag every case puts first.
 */
const CAT_READING = {
  name: 'Cat',
  text: '<Cat n="1"/>',
  start: 0,
  end: 12,
  attributes: [{ name: 'n', value: '1', valueStart: 8, valueEnd: 9, },],
};

await describe({
  name: readTags.name,
  children: [
    it({
      name: 'READS A TAG EXACTLY WHERE THE STRICT GRAMMAR READS ITS NAME, over every spelling of the differential '
        + 'set, so a name starting with `$`, `_` or a letter of another script is read and a name holding a '
        + 'character the grammar refuses, or ending on a separator, is not',
      fn: async () => {
        /**
         Spellings the reader and the grammar disagree on, as the self-closing tag a page may carry.
         */
        const disagreeing = tagNameSpellings()
          .filter(function disagrees(name,): boolean {
            return grammarAcceptsDocument({ document: `<${name} n="1"/>\n`, },)
              !== (readTags({ text: `<${name} n="1"/>\n`, },).length === 1);
          },);
        expect(disagreeing,).toEqual([],);
      },
    },),

    it({
      name: 'READS AN ATTRIBUTE NAME EXACTLY WHERE THE STRICT GRAMMAR READS ONE, over every spelling of the '
        + 'differential set: a name starting with `$`, `_` or a letter of another script is read, a prefixed name '
        + 'is one name however its colon is spaced, and a tag whose attribute name holds a character the grammar '
        + 'refuses, a member separator among them, is not read',
      fn: async () => {
        /**
         Spellings on which the reader and the grammar name different attributes, each the one attribute of a
         self-closing tag.
         */
        const disagreeing = tagNameSpellings()
          .filter(function disagrees(name,): boolean {
            /**
             The one-tag document.
             */
            const document = `<Cat ${name}="1"/>\n`;
            return readTags({ text: document, },)
              .flatMap(function attributeNames(tag,): readonly string[] {
                return tag.attributes.map(function nameOf(attribute,): string {
                  return attribute.name;
                },);
              },)
              .join('|',) !== grammarAttributeNames({ document, },).join('|',);
          },);
        expect(disagreeing,).toEqual([],);
      },
    },),

    it({
      name: 'READS THE NAME OF A MEMBER OR PREFIXED TAG WITHOUT THE WHITESPACE AROUND ITS SEPARATOR, so the page\'s '
        + 'tag and the archive\'s name one element however each spaces it',
      fn: async () => {
        expect(readTags({ text: '<Cat . Paw n="1"/>', },).map(function nameOf({ name, },): string {
          return name;
        },),).toEqual(['Cat.Paw',],);
      },
    },),

    it({
      name: 'READS NO TAG out of a bracket that opens no letter, since a tag name starts with one',
      fn: async () => {
        expect(readTags({ text: '<Cat n="1"/> then </Paw> and <3', },),).toEqual([CAT_READING,],);
      },
    },),

    it({
      name: 'READS NO TAG out of a name cut off at the end of the text, since no close bracket ever follows',
      fn: async () => {
        expect(readTags({ text: '<n', },),).toEqual([],);
      },
    },),

    it({
      name: 'READS the tag and LEAVES a bare attribute with no equals out of its reading, since a name '
        + 'alone states no value',
      fn: async () => {
        expect(readTags({ text: '<Paw x>', },),).toEqual([{
          name: 'Paw',
          text: '<Paw x>',
          start: 0,
          end: 7,
          attributes: [],
        },],);
      },
    },),

    it({
      name: 'READS the tag and LEAVES a bare prefixed attribute out of its reading, the name the grammar reads '
        + 'standing alone as a bare name does',
      fn: async () => {
        expect(grammarAttributeNames({ document: '<Paw xml:lang/>\n', },),).toEqual(['xml:lang',],);
        expect(readTags({ text: '<Paw xml:lang/>', },),).toEqual([{
          name: 'Paw',
          text: '<Paw xml:lang/>',
          start: 0,
          end: 15,
          attributes: [],
        },],);
      },
    },),

    it({
      name: 'READS NO TAG out of one whose prefixed attribute is followed by a character no attribute starts with, '
        + 'which the grammar refuses, keeping the readable tag before it',
      fn: async () => {
        expect(grammarAttributeNames({ document: '<Paw xml:lang @/>\n', },),).toEqual([],);
        expect(readTags({ text: '<Cat n="1"/> then <Paw xml:lang @/>', },),).toEqual([CAT_READING,],);
      },
    },),

    it({
      name: 'READS NO TAG out of one where a quoted value is followed by a character no attribute starts with, '
        + 'which the grammar refuses, keeping the readable tag before it',
      fn: async () => {
        expect(grammarAttributeNames({ document: '<Paw n="5" @/>\n', },),).toEqual([],);
        expect(readTags({ text: '<Cat n="1"/> then <Paw n="5" @/>', },),).toEqual([CAT_READING,],);
      },
    },),

    it({
      name: 'READS NO TAG out of one the text ends inside, after the whitespace past its last attribute, keeping '
        + 'the readable tag before it',
      fn: async () => {
        expect(readTags({ text: '<Cat n="1"/> then <Paw n="5" ', },),).toEqual([CAT_READING,],);
      },
    },),

    it({
      name: 'READS NO TAG out of one whose attribute starts where no name may, keeping the readable tag '
        + 'before it',
      fn: async () => {
        expect(readTags({ text: '<Cat n="1"/> then <Paw !>', },),).toEqual([CAT_READING,],);
      },
    },),

    it({
      name: 'READS NO TAG out of one whose attribute value carries no quote, keeping the readable tag '
        + 'before it',
      fn: async () => {
        expect(readTags({ text: '<Cat n="1"/> then <Paw n=5>', },),).toEqual([CAT_READING,],);
      },
    },),

    it({
      name: 'READS NO TAG out of one whose quoted value never closes, keeping the readable tag before '
        + 'it',
      fn: async () => {
        expect(readTags({ text: '<Cat n="1"/> then <Paw n="5>', },),).toEqual([CAT_READING,],);
      },
    },),
  ],
},);
