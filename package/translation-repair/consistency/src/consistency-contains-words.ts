import {
  codePointAt,
  codePointBefore,
  isSpacedScriptWordCharacter,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { cutsWord, } from './consistency-word-joints.ts';

//region Consistency contains words
// WHETHER ONE QUOTE, READ IN THE FOLD, STANDS INSIDE ANOTHER AT THE EDGES OF WORDS. Two readings of the judges' quotes ask it: the
// spans a strict majority of the ballots' quotes contain (`consistency-span-agreement.ts`) and the slot words given beside any quote
// that contains the located words (`consistency-majority.ts`), so both read one test and a place is located and its slot agreed by
// the same notion of containment. A match is no containment when an edge of it is a joint of a longer word, which the locator
// refuses too (`consistency-word-joints.ts`: an apostrophe or a hyphen between letters or digits, a full stop, comma, colon or
// slash between digits), so `cat` is not contained in `cat's bowl` or `cat-nap` and `5` is not contained in `0.5`.
// ONE KNOWN DIFFERENCE FROM THE LOCATOR: the fold writes a single corner bracket (`『cat』and`) as an apostrophe and the quote
// handed here has lost which character it was, so the locator, which keeps it, finds `cat` in `『cat』and` and this test does not
// contain `cat` in the folded quote `cat'and`. The difference only withholds a candidate span, which leaves a place unlocated
// rather than located at the wrong words. A RECOUNT OF AN ARTIFACT WRITTEN BEFORE THE LOCATOR'S TEST OF A JOINT COUNTED asks for the
// earlier edges (`ContainmentEdges`, `letters`), under which only a letter, a digit or a combining mark beside the match bars
// containment, as the pass that wrote the artifact counted; every other caller reads the locator's.

/**
 Which joints of a longer word bar containment: `locator` for the locator's own test of a joint (`cutsWord`), which every pass and
 every reader of a page uses, and `letters` for the earlier reading, which asked for no letter, digit or combining mark of a spaced
 script beside the match and nothing more. The earlier reading is that of a recount of an artifact written before the generation that
 moved to the locator's (`artifactGenerationCutsContainmentAtJoints`, `artifact-two-lane-contract.ts`); it is no reading for a page.
 */
export type ContainmentEdges = 'letters' | 'locator';

/**
 No offsets: a quote in the fold holds no markup, so no closing joint and no emphasis delimiter is read in it.
 */
const NO_JOINTS: ReadonlySet<number> = new Set();

/**
 No delimiters of emphasis, for the same reason.
 */
const NO_EMPHASIS: ReadonlySet<number> = new Set();

/**
 Whether the words stand in the text at the edges of words, not inside a longer word.

 @param outer - folded words of the quote that may contain them

 @param inner - folded words that may be contained

 @param edges - which joints bar containment: the locator's test of a joint, or none but a letter, a digit or a combining mark beside
 the match, as a recount of an artifact written before the locator's test counted reads it; the locator's where absent

 @returns Whether `inner` stands in `outer` with no letter, digit or combining mark of a spaced script beside it on either side
 and, under the locator's edges, no joint of a longer word at either edge, read as the locator reads one; an empty `inner` stands
 only in an empty `outer`

 @example
 ```ts
 const contained = containsWords({ outer: 'wanted to stay', inner: 'to stay', },); // true
 ```
 */
export function containsWords(
  {
    outer,
    inner,
    edges = 'locator',
  }: {
    readonly outer: string;
    readonly inner: string;
    readonly edges?: ContainmentEdges;
  },
): boolean {
  // An empty text is found at every offset and once more past the last, so the search never runs out of finds; a blank
  // quote is contained only in another blank one.
  if (inner === '')
    return outer === '';
  for (let from = outer.indexOf(inner,); from >= 0; from = outer.indexOf(
    inner,
    from + 1,
  )) {
    /**
     The character before the match, empty at the start of the quote.
     */
    const before = codePointBefore({
      text: outer,
      at: from,
    },);
    /**
     The character after the match, empty at the end of the quote.
     */
    const after = codePointAt({
      text: outer,
      at: from + inner.length,
    },);
    // A word runs on through a letter, a digit or a combining mark of a spaced script, as it does for the locator
    // (`consistency-shown-edge.ts`): a decomposed accent after the match makes it part of a longer word.
    if (isSpacedScriptWordCharacter({ character: before, },) || isSpacedScriptWordCharacter({ character: after, },))
      continue;
    // The earlier reading asks for nothing more than the letters beside the match.
    if (edges === 'letters')
      return true;
    // A JOINT OF A LONGER WORD IS READ AS THE LOCATOR READS IT: the quote is plain text in the fold, so no emphasis, link or
    // closing joint is read, and the characters the fold wrote are the characters the quote holds.
    if (!cutsWord({
      folded: outer,
      originals: outer.split('',),
      joints: NO_JOINTS,
      emphasis: NO_EMPHASIS,
      at: from,
      end: from + inner.length,
    },))
      return true;
  }
  return false;
}

//endregion Consistency contains words
