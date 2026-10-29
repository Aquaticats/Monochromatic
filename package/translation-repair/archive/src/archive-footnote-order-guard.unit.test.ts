import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { reorderFootnoteDefinitions, } from '../dist/final/node/index.mjs';

/**
 Note the cats' translators left each other: a closed comment whose words hold Han.
 */
const CAT_NOTE = '<!-- 这句猫的翻译不确定 -->';

/**
 Label of the definition whose opener is the nearest one before a note.

 @param text - page holding the note once
 @param note - note as the page writes it
 @returns Label, empty when no definition opens before the note
 */
function ownerOf(
  {
    text,
    note,
  }: {
    readonly text: string;
    readonly note: string;
  },
): string {
  /**
   Everything before the note.
   */
  const before = text.slice(
    0,
    text.indexOf(note,),
  );
  /**
   The last line before the note that opens a definition.
   */
  const opener = before.split('\n',)
    .filter(function opensDefinition(line,): boolean {
      return line.startsWith('[^',) && line.includes(']:',);
    },)
    .at(-1,);
  return opener?.slice(
    2,
    opener.indexOf(']:',),
  ) ?? '';
}

await describe({
  name: 'footnote definition movement boundaries',
  children: [
    it({
      name: 'retains each distinct whitespace gap exactly once',
      fn: async () => {
        const text = 'Body[^1][^2][^3].\n\n[^3]: Third.\n\n\n[^1]: First.\r\n\r\n[^2]: Second.\n';
        expect(reorderFootnoteDefinitions({ text, order: ['1', '2', '3'] }).text).toBe(
          'Body[^1][^2][^3].\n\n[^1]: First.\n\n\n[^2]: Second.\r\n\r\n[^3]: Third.\n');
      },
    }),
    it({
      name: 'does not duplicate or relocate comments between definition blocks',
      fn: async () => {
        const text = 'Body[^1][^2][^3].\n\n[^3]: Third.\n\n<!-- Annotation -->\n\n[^1]: First.\n\n[^2]: Second.\n';
        const result = reorderFootnoteDefinitions({ text, order: ['1', '2', '3'] });
        expect(result.text).toBe(text);
        expect(result.changed).toBe(false);
        expect(result.note).toBeDefined();
      },
    }),
    it({
      name: 'compares ranked labels using parser-equivalent identities',
      fn: async () => {
        const text = 'Body[^A][^B].\n\n[^A]: First.\n\n[^B]: Second.\n';
        expect(reorderFootnoteDefinitions({ text, order: ['b', 'a'] }).text).toBe(
          'Body[^A][^B].\n\n[^B]: Second.\n\n[^A]: First.\n');
      },
    }),
    it({
      name: 'does not separate widened container delimiters from their definition blocks',
      fn: async () => {
        const text = 'Body[^1][^2].\n\n<details>\n\n[^2]: Second.\n\n[^1]: First.\n\n</details>\n';
        let outcome: unknown;
        try {
          outcome = reorderFootnoteDefinitions({ text, order: ['1', '2'] });
        }
        catch (error) {
          outcome = error;
        }
        expect(outcome).toMatchObject({ text, changed: false });
      },
    }),
    it({
      name: 'keeps a note on the line under the last definition with that definition',
      fn: async () => {
        const text = `Body[^1][^2].\n\n[^2]: Mochi naps.\n\n[^1]: Biscuit purrs.\n  ${CAT_NOTE}\n`;
        const result = reorderFootnoteDefinitions({ text, order: ['1', '2'] });
        expect(ownerOf({ text: result.text, note: CAT_NOTE })).toBe('1');
        expect(result.text).toBe(text);
        expect(result.changed).toBe(false);
        expect(result.note).toBeDefined();
      },
    }),
    it({
      name: 'keeps a note under the last definition with it when the page uses CRLF line ends',
      fn: async () => {
        const text = `Body[^1][^2].\r\n\r\n[^2]: Mochi naps.\r\n\r\n[^1]: Biscuit purrs.\r\n  ${CAT_NOTE}\r\n`;
        const result = reorderFootnoteDefinitions({ text, order: ['1', '2'] });
        expect(ownerOf({ text: result.text, note: CAT_NOTE })).toBe('1');
        expect(result.text).toBe(text);
        expect(result.note).toBeDefined();
      },
    }),
    it({
      name: 'keeps a JSX note under the last definition with that definition',
      fn: async () => {
        const jsxNote = '{/* 猫的名字怎么翻 */}';
        const text = `Body[^1][^2].\n\n[^2]: Mochi naps.\n\n[^1]: Biscuit purrs.\n  ${jsxNote}\n`;
        const result = reorderFootnoteDefinitions({ text, order: ['1', '2'] });
        expect(ownerOf({ text: result.text, note: jsxNote })).toBe('1');
        expect(result.text).toBe(text);
      },
    }),
    it({
      name: 'keeps a note in the last definition\'s indented continuation paragraph with that definition',
      fn: async () => {
        const text = `Body[^1][^2].\n\n[^2]: Mochi naps.\n\n[^1]: Biscuit purrs.\n\n    Biscuit also dreams.\n\n    ${CAT_NOTE}\n`;
        const result = reorderFootnoteDefinitions({ text, order: ['1', '2'] });
        expect(ownerOf({ text: result.text, note: CAT_NOTE })).toBe('1');
        expect(result.text).toBe(text);
        expect(result.note).toBeDefined();
      },
    }),
    it({
      name: 'keeps the order when a tab-indented note follows the last definition after a blank line',
      fn: async () => {
        const text = `Body[^1][^2].\n\n[^2]: Mochi naps.\n\n[^1]: Biscuit purrs.\n\n\t${CAT_NOTE}\n`;
        const result = reorderFootnoteDefinitions({ text, order: ['1', '2'] });
        expect(result.text).toBe(text);
        expect(result.note).toBeDefined();
      },
    }),
    it({
      name: 'moves the definitions past a note that closes the page on a line of its own',
      fn: async () => {
        const text = `Body[^1][^2].\n\n[^2]: Mochi naps.\n\n[^1]: Biscuit purrs.\n\n${CAT_NOTE}\n`;
        const result = reorderFootnoteDefinitions({ text, order: ['1', '2'] });
        expect(result.text).toBe(`Body[^1][^2].\n\n[^1]: Biscuit purrs.\n\n[^2]: Mochi naps.\n\n${CAT_NOTE}\n`);
        expect(result.changed).toBe(true);
        expect(result.note).toBeUndefined();
      },
    }),
    it({
      name: 'moves the definitions past a heading on the line under the last definition',
      fn: async () => {
        const text = 'Body[^1][^2].\n\n[^2]: Mochi naps.\n\n[^1]: Biscuit purrs.\n## Cats\n';
        const result = reorderFootnoteDefinitions({ text, order: ['1', '2'] });
        expect(result.text).toBe('Body[^1][^2].\n\n[^1]: Biscuit purrs.\n\n[^2]: Mochi naps.\n## Cats\n');
        expect(result.note).toBeUndefined();
      },
    }),
    it({
      name: 'moves the definitions past a note indented two columns after a blank line',
      fn: async () => {
        const text = `Body[^1][^2].\r\n\r\n[^2]: Mochi naps.\r\n\r\n[^1]: Biscuit purrs.\r\n\r\n  ${CAT_NOTE}\r\n`;
        const result = reorderFootnoteDefinitions({ text, order: ['1', '2'] });
        expect(result.text).toBe(`Body[^1][^2].\r\n\r\n[^1]: Biscuit purrs.\r\n\r\n[^2]: Mochi naps.\r\n\r\n  ${CAT_NOTE}\r\n`);
        expect(result.changed).toBe(true);
      },
    }),
  ],
});
