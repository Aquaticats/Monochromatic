/**
 Tests for the semantic wrap applied to shipped text.

 WHAT THESE PIN is the property the whole change rests on: the fix is
 ADD-ONLY. Every passage a lane produces goes through it without anybody
 reading them first, which is only safe because it cannot delete, move or join
 anything. A rule that rewrote text instead of inserting into it would be a
 silent editor sitting after every judge in the pipeline.

 The second thing they pin is idempotence, which is what lets this run on a
 cache replay. A resumed slice is wrapped on the way OUT of the cache, so a
 pool written before this existed needs no migration, and a slice already
 wrapped has to come back unchanged.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { wrapReplacementText, } from '../dist/final/node/index.mjs';

/**
 Reads a passage as content alone, with spacing and block markers discounted.

 WHITESPACE GOES because a break replaces the space that separated two
 sentences, so comparing spaces would call that replacement a deletion.
 `>` GOES because a break inside a blockquote carries the marker onto the new
 line, which is a character the wrapper legitimately ADDS. What is left is the
 content, and content must survive wrapping exactly.

 @param text - passage to reduce

 @returns Content characters, spacing and blockquote markers removed

 @example
 ```ts
 const content = contentOnly({ text: '> It naps.', },);
 ```
 */
function contentOnly({ text, }: { readonly text: string; },): string {
  return text.split('\n',)
    .join('',)
    .split('\r',)
    .join('',)
    .split('\t',)
    .join('',)
    .split(' ',)
    .join('',)
    .split('>',)
    .join('',);
}

await describe({
  name: wrapReplacementText.name,
  children: [
    it({
      name: 'BREAKS A RUN-ON PASSAGE at its sentence and clause boundaries, which is the whole '
        + 'point: a model returns a paragraph as one line and the archive it replaces was wrapped',
      fn: async () => {
        /**
         One line, as a model hands it over.
         */
        const flat = 'The tabby naps on the sill. It wakes at dusk, stretches, and goes hunting.';

        /**
         Same passage as the rule would have it written.
         */
        const wrapped = wrapReplacementText({ text: flat, },);

        expect(flat.split('\n',).length,).toBe(1,);
        expect(wrapped.split('\n',).length,).toBeGreaterThan(1,);

        // The words survive in order; only breaks were put between them.
        expect(
          wrapped.split('\n',)
            .map(function trimmed(line,): string {
              return line.trim();
            },)
            .join(' ',),
        ).toBe(flat,);
      },
    },),

    it({
      name: 'KEEPS EVERY VISIBLE CHARACTER, removing only the space each break replaces. Every '
        + 'produced passage goes through this without review, so a fix that could delete content '
        + 'would be an unreviewed editor standing after every judge in the pipeline',
      fn: async () => {
        /**
         A passage carrying punctuation the rule looks at, in several shapes.
         */
        const passages = [
          'It naps. It wakes. It eats.',
          '> The cat asked: is the bowl full? It was not.',
          '- Mittens; the tabby, asleep.\n- Whiskers, awake.',
          'One sentence only',
          '',
        ];

        for (const text of passages) {
          /**
           Wrapped form of this passage.
           */
          const wrapped = wrapReplacementText({ text, },);
          // EQUALITY RATHER THAN A FLOOR. The older form counted characters
          // other than breaks and asked only that the total not fall, which a
          // fix deleting a letter while adding two prefix characters would
          // satisfy. Discounting spacing and blockquote markers leaves exactly
          // the content, and the content must come back identical.
          expect(contentOnly({ text: wrapped, },),).toBe(contentOnly({ text, },),);
          // AND NO LINE ENDS IN WHITESPACE, which is the other half of putting
          // a break where a space was: one trailing space reads as rubbish and
          // two are a CommonMark hard break, so a wrapper that stepped over the
          // space instead of consuming it would show up here.
          for (const line of wrapped.split('\n',))
            expect(line,).toBe(line.trimEnd(),);
        }
      },
    },),

    it({
      name: 'RETURNS AN ALREADY WRAPPED PASSAGE UNCHANGED, which is what makes a cache replay safe: '
        + 'a slice resumed from a pool written before this existed is wrapped on the way out, and '
        + 'one wrapped already must not drift further on every resume',
      fn: async () => {
        /**
         A passage wrapped once.
         */
        const once = wrapReplacementText({
          text: 'The tabby naps on the sill. It wakes at dusk, stretches, and goes hunting.',
        },);

        expect(wrapReplacementText({ text: once, },),).toBe(once,);
      },
    },),

    it({
      name: 'LEAVES A PASSAGE WITH NO BOUNDARY ALONE rather than inventing one, so a short reply '
        + 'is byte-identical afterwards and nothing downstream sees a change nobody made',
      fn: async () => {
        expect(wrapReplacementText({ text: 'A tabby', },),).toBe('A tabby',);
        expect(wrapReplacementText({ text: '', },),).toBe('',);
      },
    },),

    it({
      name: 'KEEPS A HEADING ON ONE LINE, since the rule skips headings, tables, links and raw '
        + 'HTML: breaking those changes what they render as rather than only how they are stored',
      fn: async () => {
        /**
         A heading carrying punctuation the rule would otherwise break at.
         */
        const heading = '## The cat, the bowl, and the sill.';
        expect(wrapReplacementText({ text: heading, },),).toBe(heading,);
      },
    },),

    it({
      name: 'KEEPS A PHOTO COMPONENT ON ITS LINE, though its JSX braces make it prose to CommonMark: '
        + 'the Hangmster page of 2026-09-04 shipped with the closing `/>` broken onto its own line '
        + 'because the array carried a comma followed by a space, and sixteen source-page elements '
        + 'at the pinned corpus share that shape',
      fn: async () => {
        /**
         A one-line element with the comma-and-space shape, trailing hard-break
         spaces included, between two prose paragraphs the rule does wrap.
         */
        const element = `<PhotoScroll photos={[ '\${path}/photos/sill.webp',]} />  `;
        const text = `She drew this, and posted it:\n\n${element}\n\nThe cat, as ever, approved.`;

        /**
         What the wrapper returned.
         */
        const wrapped = wrapReplacementText({ text, },);
        expect(wrapped.includes(element,),).toBe(true,);
        expect(wrapped.split('\n',).filter(function isClose(line,): boolean {
          return line.trim() === '/>';
        },).length,).toBe(0,);
        expect(wrapped.startsWith('She drew this,\nand posted it:',),).toBe(true,);
      },
    },),
  ],
},);
