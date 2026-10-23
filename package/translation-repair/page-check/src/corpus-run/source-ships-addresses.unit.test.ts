/**
 Tests for the bare addresses GFM reads in a text, and for writing the ones a less-than sign ends as links.

 An address matters to the encoder because the strict grammar reads it to the next space or less-than sign as one link:
 a brace inside it is part of it, and a backslash written there would show.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  addressAsLink,
  type BareAddress,
  bareAddresses,
} from '../../dist/final/node/index.mjs';

/**
 Finds the addresses of a text read whole as text, outside any link.

 @param text - the text

 @returns Addresses found

 @example
 ```ts
 const found = addressesOf({ text: '见 https://example.net/a', },);
 ```
 */
function addressesOf({ text, }: { readonly text: string; },): readonly BareAddress[] {
  return bareAddresses({
    text,
    ranges: [
      {
        start: 0,
        end: text.length,
      },
    ],
    insideLinks: [],
  },);
}

/**
 The text of each address found, as it stands in the text.

 @param text - the text

 @returns Each address's characters

 @example
 ```ts
 const shown = shownAddresses({ text: '见 https://example.net/a b', },); // ['https://example.net/a']
 ```
 */
function shownAddresses({ text, }: { readonly text: string; },): readonly string[] {
  return addressesOf({ text, },).map(function shown({ start, end, },): string {
    return text.slice(
      start,
      end,
    );
  },);
}

await describe({
  name: 'source ships addresses',
  children: [
    describe({
      name: bareAddresses.name,
      children: [
        it({
          name: 'FINDS AN ADDRESS WITH A PROTOCOL OR WITH www. and reads it to the next space or less-than sign',
          fn: async () => {
            /**
             Text holding an address with a brace, one ended by a tag and one without a path.
             */
            const text = '见 https://example.net/b{c 与 www.example.net/d<e> 与 http://x.net';
            expect(shownAddresses({ text, },),).toEqual([
              'https://example.net/b{c',
              'www.example.net/d',
              'http://x.net',
            ],);
          },
        },),

        it({
          name: 'SAYS WHETHER A LESS-THAN SIGN ENDS THE ADDRESS, the case a backslash cannot be written before',
          fn: async () => {
            expect(addressesOf({ text: 'https://example.net/a<b> https://example.net/c', },).map(function ends(
              { endsAtTag, },
            ): boolean {
              return endsAtTag;
            },),).toEqual([true, false,],);
          },
        },),

        it({
          name: 'FINDS NO ADDRESS WHEN A PUNCTUATION MARK OR A SPACE FOLLOWS THE PROTOCOL, which GFM reads as none',
          fn: async () => {
            expect(shownAddresses({ text: '见 https://{x 与 https:// {y 与 https://.net', },),).toEqual([],);
            expect(shownAddresses({ text: 'https://', },),).toEqual([],);
          },
        },),

        it({
          name: 'FINDS NO ADDRESS RIGHT AFTER A LETTER, nor www. after a character that is no separator',
          fn: async () => {
            expect(shownAddresses({ text: 'xhttps://example.net 与 awww.example.net', },),).toEqual([],);
            expect(shownAddresses({ text: '(www.example.net', },),).toEqual(['www.example.net',],);
          },
        },),

        it({
          name: 'FINDS NO ADDRESS AFTER A LABEL NOTHING HAS CLOSED, to the tokenizer: a closed pair of brackets, an '
            + 'escaped bracket, an image opener and the start of the next paragraph change the count',
          fn: async () => {
            expect(shownAddresses({ text: '[ https://example.net/b{c', },),).toEqual([],);
            expect(shownAddresses({ text: 'x ![ www.example.net/b{c', },),).toEqual([],);
            expect(shownAddresses({ text: '[a [b] https://example.net/c', },),).toEqual([],);
            expect(shownAddresses({ text: '[a] https://example.net/c', },),).toEqual(['https://example.net/c',],);
            expect(shownAddresses({ text: '[a] [b [c] https://example.net/c', },),).toEqual([],);
            expect(shownAddresses({ text: String.raw`\[ https://example.net/c`, },),)
              .toEqual(['https://example.net/c',],);
            expect(shownAddresses({ text: 'x ] https://example.net/c', },),).toEqual(['https://example.net/c',],);
            /**
             Two paragraphs, the first of which opens a label.
             */
            const text = '[ x\n\nhttps://example.net/c';
            expect(bareAddresses({
              text,
              ranges: [
                {
                  start: 0,
                  end: 3,
                },
                {
                  start: 5,
                  end: text.length,
                },
              ],
              insideLinks: [],
              streamStarts: [0, 5,],
            },).map(function startOf({ start, },): number {
              return start;
            },),).toEqual([5,],);
          },
        },),

        it({
          name: 'KEEPS AN ADDRESS A LESS-THAN SIGN ENDS AFTER AN OPEN LABEL, since only the transform reads it '
            + 'there and it runs on past the sign',
          fn: async () => {
            expect(addressesOf({ text: '[a https://example.net/a<b> {', },).map(function ends(
              { endsAtTag, },
            ): boolean {
              return endsAtTag;
            },),).toEqual([true,],);
          },
        },),

        it({
          name: 'FINDS NO ADDRESS INSIDE THE TEXT OF A LINK, where GFM reads none',
          fn: async () => {
            /**
             A link whose text is an address, then an address of its own.
             */
            const text = '[https://example.net/a{ b](u) https://example.net/c';
            expect(bareAddresses({
              text,
              ranges: [
                {
                  start: 1,
                  end: 25,
                },
                {
                  start: 30,
                  end: text.length,
                },
              ],
              insideLinks: [
                {
                  start: 0,
                  end: 29,
                },
              ],
            },).map(function startOf({ start, },): number {
              return start;
            },),).toEqual([30,],);
          },
        },),
      ],
    },),

    describe({
      name: addressAsLink.name,
      children: [
        it({
          name: 'WRITES AN ADDRESS AS A LINK WITH THE SAME TEXT AND TARGET, www. addresses to http://',
          fn: async () => {
            /**
             Edit for an address that stands for the whole text.
             */
            function editFor({ text, }: { readonly text: string; },): ReturnType<typeof addressAsLink> {
              return addressAsLink({
                text,
                address: {
                  start: 0,
                  end: text.length,
                },
              },);
            }
            expect(editFor({ text: 'https://example.net/a_b', },),).toEqual({
              start: 0,
              end: 23,
              replacement: String.raw`[https://example.net/a\_b](https://example.net/a_b)`,
            },);
            expect(editFor({ text: 'www.example.net/a', },).replacement,)
              .toBe('[www.example.net/a](http://www.example.net/a)',);
          },
        },),

        it({
          name: 'LEAVES OUT OF THE LINK THE PUNCTUATION GFM LEAVES OUT and a closing parenthesis nothing opened, '
            + 'and keeps one that closes a parenthesis of the address',
          fn: async () => {
            /**
             Edit for an address that stands for the whole text.
             */
            function editFor({ text, }: { readonly text: string; },): ReturnType<typeof addressAsLink> {
              return addressAsLink({
                text,
                address: {
                  start: 0,
                  end: text.length,
                },
              },);
            }
            expect(editFor({ text: 'https://example.net/a.', },).end,).toBe(21,);
            expect(editFor({ text: 'https://example.net/a?!)', },).end,).toBe(21,);
            expect(editFor({ text: 'https://example.net/a(b)', },).end,).toBe(24,);
          },
        },),
      ],
    },),
  ],
},);
