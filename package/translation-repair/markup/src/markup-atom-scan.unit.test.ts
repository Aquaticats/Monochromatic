/**
 Tests for the markup atom scan the preservation gate reads (ledger L4): each
 destination-grammar boundary a fragment of MDX can put in its way.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  type MarkupAtom,
  scanMarkupAtoms,
} from '../dist/final/node/index.mjs';

/**
 The most characters after the caret that the strict grammar reads in a footnote label.
 */
const MOST_READ = 999;

/**
 One scan and the atoms it must find.
 */
type ScanCase = {
  /**
   What the case pins.
   */
  readonly name: string;

  /**
   Fragment scanned.
   */
  readonly text: string;

  /**
   Atoms expected, in order.
   */
  readonly atoms: readonly MarkupAtom[];
};

/**
 Every boundary the scan must respect.
 */
const CASES: readonly ScanCase[] = [
  {
    name: 'finds each kind, in the order the fragment carries them',
    text: 'a[^1] `c` {d} <Paw/> [e](f)',
    atoms: [
      { kind: 'footnote-reference', value: '[^1]', },
      { kind: 'inline-code', value: '`c`', },
      { kind: 'mdx-expression', value: '{d}', },
      { kind: 'tag', value: '<Paw/>', },
      { kind: 'link-destination', value: '(f)', },
    ],
  },
  {
    name: 'reads an escaped opener as literal text',
    text: String.raw`the cat \[^1] and \{paws} and \<Tail/>`,
    atoms: [],
  },
  {
    name: 'keeps an escaped close inside an expression\'s quoted string, so the expression closes on the real '
      + 'one',
    text: String.raw`{"a\}b"}`,
    atoms: [
      { kind: 'mdx-expression', value: String.raw`{"a\}b"}`, },
    ],
  },
  {
    name: 'leaves a link destination unclosed at the newline literal, since a destination cannot run on '
      + 'into the next line',
    text: '[cat](www\nmore)',
    atoms: [],
  },
  {
    name: 'leaves an unterminated comment with no atom, since nothing says where it ends',
    text: '<!-- open',
    atoms: [],
  },
  {
    name: 'leaves a tag whose attribute expression never closes with no atom',
    text: '<X y={open>',
    atoms: [],
  },
  {
    name: 'consumes a code span whole, so markup inside it is code rather than atoms',
    text: 'run `a[^1]{b}<i>](c)` now',
    atoms: [{ kind: 'inline-code', value: '`a[^1]{b}<i>](c)`', },],
  },
  {
    name: 'closes a code span only on a run of the same length',
    text: 'use ``a`b`` here',
    atoms: [{ kind: 'inline-code', value: '``a`b``', },],
  },
  {
    name: 'leaves an unmatched backtick literal and reads on past it',
    text: 'a ` b[^1]',
    atoms: [{ kind: 'footnote-reference', value: '[^1]', },],
  },
  {
    name: 'keeps balanced parentheses inside a link destination',
    text: '[the tabby](https://example.org/cat_(tabby)) naps',
    atoms: [{ kind: 'link-destination', value: '(https://example.org/cat_(tabby))', },],
  },
  {
    name: 'keeps nested braces inside one MDX expression',
    text: 'x {paws({left: 1})} y',
    atoms: [{ kind: 'mdx-expression', value: '{paws({left: 1})}', },],
  },
  {
    name: 'reads an angle bracket before a space or digit as prose',
    text: 'a < b and 3<4',
    atoms: [],
  },
  {
    name: 'reads closing tags and comments as tags',
    text: '</p> and <!-- keeper note -->',
    atoms: [
      { kind: 'tag', value: '</p>', },
      { kind: 'tag', value: '<!-- keeper note -->', },
    ],
  },
  {
    name: 'reads a tag where the MDX compiler opens one (ledger B18): a name outside ASCII and a fragment are '
      + 'tags, and a `<!` opening no comment is prose, which the compiler refuses',
    text: 'the <猫 /> and <>paw</> and <!x> nap',
    atoms: [
      { kind: 'tag', value: '<猫 />', },
      { kind: 'tag', value: '<>', },
      { kind: 'tag', value: '</>', },
    ],
  },
  {
    name: 'steps over an attribute expression and a quoted value holding a closing bracket',
    text: '<Paw n={a > b} alt="x > y" /> naps',
    atoms: [{ kind: 'tag', value: '<Paw n={a > b} alt="x > y" />', },],
  },
  {
    name: 'yields no atom for a construct the fragment edge cuts off',
    text: 'the cat {unclosed and ](https://example.org and <Paw and [^2',
    atoms: [],
  },
  {
    name: 'refuses a footnote label broken by a newline',
    text: 'the [^a\nb] cat',
    atoms: [],
  },
  {
    name: 'refuses a footnote label holding a space or nothing, as GFM does',
    text: 'the [^a b] and [^] cat',
    atoms: [],
  },
  {
    name: 'refuses a footnote label of more than 999 characters and reads one of 999 (micromark-extension-gfm-footnote@2.1.0, '
      + 'lib/syntax.js:354-381)',
    text: `the [^${'a'.repeat(MOST_READ + 1,)}] and [^${'b'.repeat(MOST_READ,)}] cat`,
    atoms: [{ kind: 'footnote-reference', value: `[^${'b'.repeat(MOST_READ,)}]`, },],
  },
  {
    name: 'reads a no-break space in a footnote label as part of its name, since GFM refuses only the space, the tab '
      + 'and the line endings',
    text: 'the [^a\u00A0b] cat',
    atoms: [{ kind: 'footnote-reference', value: '[^a\u00A0b]', },],
  },
  {
    name: 'reads an escaped bracket in a footnote label as part of its name, and does not close the label at an escaped '
      + 'closing bracket',
    text: String.raw`the [^a\[b] and [^c\]d] cat`,
    atoms: [
      { kind: 'footnote-reference', value: String.raw`[^a\[b]`, },
      { kind: 'footnote-reference', value: String.raw`[^c\]d]`, },
    ],
  },
  {
    name: 'refuses a footnote label that ends in a backslash with no closing bracket after it',
    text: String.raw`the [^a\]`,
    atoms: [],
  },
];

await describe({
  name: scanMarkupAtoms.name,
  children: CASES.map(function toCase({ name, text, atoms, },) {
    return it({
      name,
      fn: async () => {
        expect(scanMarkupAtoms({ text, },),).toStrictEqual(atoms,);
      },
    },);
  },),
},);
