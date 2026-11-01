import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  blocksOrWholeText,
  withoutHtmlComments,
} from './translate-address-drop.ts';
import { lowerCaseLatinWords, } from '@monochromatic-dev/translation-repair-markup/ts';

//region A suicide the passage names
// CLASS ONE HUNDRED FIFTY (shi_Yumiaoya36, 2026-09-26). The original states a
// suicide attempt she survived (six days in intensive care, then she woke),
// and the page shipped a sentence naming only the means: the attempt the
// original states was gone, and the means the house rule keeps vague stood in
// its place. The polish round chose a rendering that said the attempt 2.5 to 2
// with two ballots citing the survived-attempt rule, and the polish gate
// settled on neither, so the base shipped. (The original's sentence and both
// renderings named the means; they are left out here under the same house
// rule, ledger X3.) The house rule says a survived attempt is
// still an attempt and the page says she attempted suicide or tried to end her
// life; a rendering with no wording for suicide at all has dropped what the
// ORIGINAL states and is refused before any judge reads it, as a dropped
// address is (class ninety-seven). THE FLOOR IS NARROW: it asks only that the
// suicide be said, never how, so a vague rendering the house rule prefers
// passes, and whether the means are too specific stays with the judges. The
// corpus census of 2026-09-26 found 44 entries whose original names a suicide;
// the archive English of 40 says it, two (shi_Yumiaoya, XIEPT2) are partial
// archives without those passages, and two render a quoted line without the
// word (自杀痛苦 as "the pain of dying", 别自杀 as "please don't follow her"),
// which the floor now refuses as a standing.
//
// LEDGER F-4 (2026-09-27). That census counted entries, not slices. Replayed
// slice by slice over 1,264 archive slices and 3,975 would-ship slices, the
// floor refused 11, and three of those were correct English: an attempt on a
// life said as "attempts on her own life", and one quotation of a published
// work given in its published English, whose Chinese translation had added
// 自杀. An attempt on a life, a death by one's own hand (after a death word,
// so a hand that only wrote does not count) and an attributed quotation of a
// work (quoted block closing on a dash and a title in 《》) now pass; the
// other eight refusals, each dropping or blurring the word, stand, and none
// was added. The finding no longer assumes the person is "she".

/**
 Words the original names a suicide with.
 */
const SUICIDE_WORDS: readonly string[] = [
  '自杀',
  '自尽',
  '轻生',
];

/**
 Start of every English word naming a suicide (suicide, suicides,
 suicidal).
 */
const SUICIDE_STEM = 'suicid';

/**
 Verbs that take a reflexive to name a suicide ("killed herself").
 */
const KILL_VERBS: ReadonlySet<string> = new Set([
  'kill',
  'kills',
  'killed',
  'killing',
]);

/**
 Reflexive pronouns a kill verb takes.
 */
const REFLEXIVES: ReadonlySet<string> = new Set([
  'herself',
  'himself',
  'themself',
  'themselves',
  'myself',
  'yourself',
  'yourselves',
  'ourselves',
  'oneself',
]);

/**
 Verbs that take "life" to name a suicide ("ended her life", "took her own
 life").
 */
const LIFE_VERBS: ReadonlySet<string> = new Set([
  'end',
  'ends',
  'ended',
  'ending',
  'take',
  'takes',
  'took',
  'taken',
  'taking',
]);

/**
 Nouns a life verb takes.
 */
const LIFE_NOUNS: ReadonlySet<string> = new Set([
  'life',
  'lives',
]);

/**
 Possessives that may stand between a life verb and its noun ("her life").
 */
const POSSESSIVES: ReadonlySet<string> = new Set([
  'her',
  'his',
  'their',
  'my',
  'your',
  'our',
]);

/**
 Word that may follow the possessive ("her own life").
 */
const OWN = 'own';

/**
 Nouns that take "on" and a life to name a suicide attempt ("an attempt on
 her own life").
 */
const ATTEMPT_NOUNS: ReadonlySet<string> = new Set([
  'attempt',
  'attempts',
]);

/**
 Words after which "by her own hand" names how a death came ("died by her own
 hand"); after any other word the hand only wrote or made something.
 */
const DEATH_WORDS: ReadonlySet<string> = new Set([
  'die',
  'died',
  'dies',
  'dead',
  'death',
  'perished',
]);

/**
 Nouns "by her own" takes to name a death by suicide.
 */
const HAND_NOUNS: ReadonlySet<string> = new Set([
  'hand',
  'hands',
]);

/**
 Whether the words at an offset close on one of some nouns: an optional
 possessive, an optional "own" after it, then the noun.

 @param words - rendering's words in order

 @param after - offset of the word just past the verb or preposition

 @param nouns - nouns the phrase may close on

 @returns True where the phrase closes on one of them

 @example
 ```ts
 closesOn({ words: ['took', 'her', 'own', 'life',], after: 1, nouns: LIFE_NOUNS, },); // true
 ```
 */
function closesOn(
  {
    words,
    after,
    nouns,
  }: {
    readonly words: readonly string[];
    readonly after: number;
    readonly nouns: ReadonlySet<string>;
  },
): boolean {
  /**
   Offset past the possessive, where one stands.
   */
  const pastPossessive = POSSESSIVES.has(words[after] ?? '',) ? after + 1 : after;
  /**
   Offset past "own", where it follows the possessive.
   */
  const pastOwn = ((pastPossessive > after) && (words[pastPossessive] === OWN)) ? pastPossessive + 1 : pastPossessive;
  return nouns.has(words[pastOwn] ?? '',);
}

/**
 Whether a run of words says a suicide: a word on the suicide stem, a kill
 verb before a reflexive, a life verb whose phrase closes on a life, an
 attempt on a life, or a death by one's own hand.

 @param words - rendering's words in order

 @returns True where any of the five stands

 @example
 ```ts
 saysSuicide({ words: ['she', 'took', 'her', 'own', 'life',], },); // true
 ```
 */
function saysSuicide({ words, }: { readonly words: readonly string[]; },): boolean {
  return words.some(function namesIt(
    word,
    index,
  ): boolean {
    if (word.startsWith(SUICIDE_STEM,))
      return true;
    if (KILL_VERBS.has(word,))
      return REFLEXIVES.has(words[index + 1] ?? '',);
    if (ATTEMPT_NOUNS.has(word,)) {
      return (words[index + 1] === 'on') && closesOn({
        words,
        after: index + 2,
        nouns: LIFE_NOUNS,
      },);
    }
    if ((word === 'by') && DEATH_WORDS.has(words[index - 1] ?? '',)) {
      return closesOn({
        words,
        after: index + 1,
        nouns: HAND_NOUNS,
      },);
    }
    return LIFE_VERBS.has(word,) && closesOn({
      words,
      after: index + 1,
      nouns: LIFE_NOUNS,
    },);
  },);
}

/**
 Dashes that open an attribution line under a quotation.
 */
const ATTRIBUTION_DASHES: readonly string[] = [
  '——',
  '—',
  '―',
];

/**
 Whether a block is a quotation of a published work: every line quoted, the
 last closing on an attribution dash and a work title in 《》. Such a block is
 rendered in the work's published English, which need not carry the word a
 Chinese translation of it added (ledger F-4: a Camus line).

 @param block - block of the original

 @returns True for an attributed quotation of a work

 @example
 ```ts
 quotesPublishedWork({ block: '> 「……」\n> ——喵喵《猫的神话》', },); // true
 ```
 */
function quotesPublishedWork({ block, }: { readonly block: string; },): boolean {
  /**
   Lines of the block.
   */
  const lines = block.split('\n',);
  if (!lines.every(function quoted(line,): boolean {
    return line.trimStart()
      .startsWith('>',);
  },))
    return false;
  /**
   Last line with its quotation marks stripped; splitting yields at least one
   line, so there is always a last.
   */
  const attribution = nonNullishOrThrow(lines.at(-1,),)
    .replaceAll(
    '>',
    '',
    )
    .trim();
  return attribution.includes('《',) && ATTRIBUTION_DASHES.some(function opens(dash,): boolean {
    return attribution.startsWith(dash,);
  },);
}

/**
 Findings for a candidate that renders a passage naming a suicide with no
 wording for suicide at all.

 @param sourceText - original slice

 @param candidateText - candidate under validation

 @returns One finding, or none where the original names no suicide or the
 candidate says it

 @example
 ```ts
 const findings = droppedSuicideFindings({ sourceText: '她自杀了。', candidateText: 'She passed away.', },);
 ```
 */
export function droppedSuicideFindings(
  {
    sourceText,
    candidateText,
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
  },
): readonly string[] {
  /**
   The original outside its comments and its quotations of published works.
   */
  const original = blocksOrWholeText({ text: withoutHtmlComments({ text: sourceText, },), },)
    .filter(function ownWords(block,): boolean {
      return !quotesPublishedWork({ block, },);
    },)
    .join('\n\n',);
  /**
   Words the original names the suicide with.
   */
  const named = SUICIDE_WORDS.filter(function inOriginal(word,): boolean {
    return original.includes(word,);
  },);
  if (named.length === 0)
    return [];
  if (saysSuicide({ words: lowerCaseLatinWords({ text: withoutHtmlComments({ text: candidateText, },), },), },))
    return [];
  return [
    `Your translation drops the suicide the ORIGINAL names: the ORIGINAL passage writes ${named.join(' and ',)}, and your translation carries no wording for suicide at all. A death by suicide is said to be a suicide, and a survived attempt is still an attempt: say that the person attempted suicide or tried to end their life, with the pronoun the page uses for them, keeping the means as vague as the house rule asks.`,
  ];
}

//endregion A suicide the passage names
