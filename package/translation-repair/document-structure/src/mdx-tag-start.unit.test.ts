/**
 Tests whether `<` opens a tag as the MDX compiler reads it (audit area six,
 ledger B18), against the package's own MDX parser, which uses the compiler's
 grammar: every probe's answer must be what the parser does with that text.
 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { opensMdxTag, } from '@monochromatic-dev/translation-repair-markup';
import { parseMdxBody, } from '../dist/final/node/index.mjs';

/**
 What the parser makes of a probe.
 */
type Outcome = 'tag' | 'text' | 'fails';

/**
 One probe: a text and what the parser makes of it.
 */
type Probe = {
  /**
   Text holding the `<` under test.
   */
  readonly text: string;

  /**
   Offset of that `<`.
   */
  readonly at: number;

  /**
   What the parser makes of the text.
   */
  readonly outcome: Outcome;
};

/**
 Probes for every class the compiler tells apart after `<`: a name's start in
 and outside ASCII, `$` and `_`, a closing slash, a fragment, whitespace it
 steps over, markdown's space, tab and line end, and characters it refuses.
 */
const PROBES: readonly Probe[] = [
  {
    text: 'a <x /> b',
    at: 2,
    outcome: 'tag',
  },
  {
    text: 'a <猫 /> b',
    at: 2,
    outcome: 'tag',
  },
  {
    text: 'a <é /> b',
    at: 2,
    outcome: 'tag',
  },
  {
    text: 'a <Ω /> b',
    at: 2,
    outcome: 'tag',
  },
  {
    text: 'a <$x /> b',
    at: 2,
    outcome: 'tag',
  },
  {
    text: 'a <_x /> b',
    at: 2,
    outcome: 'tag',
  },
  {
    text: 'a <b>x</b> c',
    at: 6,
    outcome: 'tag',
  },
  {
    text: 'a <>x</> b',
    at: 2,
    outcome: 'tag',
  },
  {
    text: 'a <\u{00A0}b /> c',
    at: 2,
    outcome: 'tag',
  },
  {
    text: 'a < b',
    at: 2,
    outcome: 'text',
  },
  {
    text: 'a <\tb',
    at: 2,
    outcome: 'text',
  },
  {
    text: 'a <\nb',
    at: 2,
    outcome: 'text',
  },
  {
    text: 'a <3 b',
    at: 2,
    outcome: 'fails',
  },
  {
    text: 'a <! b',
    at: 2,
    outcome: 'fails',
  },
  {
    text: 'a <| b',
    at: 2,
    outcome: 'fails',
  },
  {
    text: 'a <] b',
    at: 2,
    outcome: 'fails',
  },
  {
    text: 'a <@ b',
    at: 2,
    outcome: 'fails',
  },
  {
    text: 'a <- b',
    at: 2,
    outcome: 'fails',
  },
  {
    text: 'a <\u{24B6} /> b',
    at: 2,
    outcome: 'fails',
  },
  // A letter past U+FFFF is two units to the compiler, neither of them a
  // letter, so it starts no name.
  {
    text: 'a <\u{20000} /> b',
    at: 2,
    outcome: 'fails',
  },
  {
    text: 'a <\u{1D49C} /> b',
    at: 2,
    outcome: 'fails',
  },
  {
    text: 'a <\u{1F431} /> b',
    at: 2,
    outcome: 'fails',
  },
];

/**
 Whether a node or any node under it is JSX.

 @param node - mdast node

 @returns Whether JSX stands there

 @example
 ```ts
 carriesJsx({ node: parseMdxBody({ body: '<x />', },), },); // true
 ```
 */
function carriesJsx({ node, }: { readonly node: { readonly type: string; readonly children?: readonly unknown[]; }; },): boolean {
  return node.type.startsWith('mdxJsx',)
    || (node.children ?? []).some(function child(inner,): boolean {
      return carriesJsx({ node: inner as { readonly type: string; readonly children?: readonly unknown[]; }, },);
    },);
}

/**
 What the package's MDX parser makes of a text.

 @param text - text parsed

 @returns Tag where the tree holds JSX, fails where the parse throws, text
 otherwise

 @example
 ```ts
 parsed({ text: 'a < b', },); // 'text'
 ```
 */
function parsed({ text, }: { readonly text: string; },): Outcome {
  try {
    return carriesJsx({ node: parseMdxBody({ body: text, },), },) ? 'tag' : 'text';
  }
  catch (error) {
    // The refusal is the answer: the compiler fails the page here.
    expect(String(error,).length > 0,).toBe(true,);
    return 'fails';
  }
}

await describe({
  name: opensMdxTag.name,
  children: [
    it({
      name: 'REACHES every class: each probe parses as the class it is written for',
      fn: async () => {
        expect(PROBES.map(function outcomeOf({ text, },): Outcome {
          return parsed({ text, },);
        },),).toEqual(PROBES.map(function written({ outcome, },): Outcome {
          return outcome;
        },),);
      },
    },),
    it({
      name: 'ANSWERS as the parser does: a tag opens exactly where the parser reads one, and not where it reads '
        + 'text or refuses the page',
      fn: async () => {
        expect(PROBES.map(function answer({ text, at, },): boolean {
          return opensMdxTag({
            text,
            at,
          },);
        },),).toEqual(PROBES.map(function tagOnly({ outcome, },): boolean {
          return outcome === 'tag';
        },),);
      },
    },),
  ],
},);
