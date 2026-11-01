import type { DocumentNode, } from './document-node.ts';
import {
  continuesLatinWord,
  foldLatinWord,
  isLatinWordCharacter,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Block alignment
// Paragraph pairing used to assume that equal node counts mean one-to-one
// correspondence, and `slice-pair.ts` said so outright: "When both sides carry
// the same node count their paragraphs correspond one to one ... never
// drifting." The pinned corpus disproves it. One entry carries 32 blocks on
// each side, yet its translation drops a short lead-in paragraph and folds the
// line into the quotation that follows, so from that point every index-paired
// slice compares a block against its neighbour. Equal totals hid the drift
// because the translation regains a block further down.
//
// Critics then behave exactly as asked: handed two unrelated blocks, they
// report differences between them. Those become confident, well-anchored, and
// entirely false issues, which is how milestone three's graded sample scored
// its single largest false-positive cluster.
//
// This is a monotone alignment that may skip a block on either side instead of
// forcing a partner. Scoring is deliberately language-neutral, since the whole
// point is pairing Chinese against English before any model has read either:
// block kind, shared script-neutral tokens (Latin words, digit runs, component
// names), and a plausible length ratio.

/**
 Score awarded when both blocks are the same mdast kind. A quotation matches
 a quotation far more reliably than any textual signal survives translation.
 */
const KIND_MATCH_SCORE = 2;

/**
 Penalty when kinds differ. Kept mild rather than prohibitive: a translation
 may legitimately render a lead-in plus quotation as one quotation.
 */
const KIND_MISMATCH_PENALTY = -1;

/**
 Weight on shared literal tokens. Names, years, and component names survive
 translation unchanged, so agreement here is strong evidence of partnership.
 */
const TOKEN_OVERLAP_WEIGHT = 3;

/**
 Weight on a plausible expansion ratio between the two blocks' lengths.
 */
const LENGTH_PLAUSIBILITY_WEIGHT = 1;

/**
 Cost of leaving a block unpartnered. Set below the swing between a kind
 match and a kind mismatch, so a single dropped block is cheaper to skip than
 to force onto a neighbour, which is exactly the drift being fixed.

 DECLARED HERE, BESIDE THE SCORES IT IS SET AGAINST, and paid by the walk
 (`align-blocks-walk.ts`). From the first commit of both files (`35c4a4aac`)
 until 2026-09-29 the walk declared its own copy "mirrored from" this one
 and nothing read this one, so retuning it here would have changed nothing
 (ledger B31).
 */
export const GAP_PENALTY = -1.5;

/**
 Characters a Chinese block typically becomes in English, used only when the
 pair itself gives no better estimate. Chinese is written without spaces and
 packs more meaning per character, so an English rendering runs longer; this
 judges plausibility and never rejects.

 A FIXED CONSTANT IS WRONG FOR THIS CORPUS, measured 2026-08-20. Entry medians
 of English characters per Chinese character run from 1.49 to 4.10, because how
 far a translation expands is a property of the TRANSLATOR, not of the language
 pair: a plain rendering stays close and a literary one runs long.

 On `saurikissa`, whose median is 4.10, every CORRECT pair scored as implausible
 against 1.8, which destroyed the only signal the walk had. Chinese and English
 prose share no Latin tokens, and block kind is constant when every block is a
 paragraph, so length was carrying the alignment alone and pointing the wrong
 way. Six of eleven slices then paired unrelated paragraphs.
 `doc/audit/the-critics-are-shown-the-wrong-paragraph.md` records the reading.

 @internal
 */
export const FALLBACK_EXPANSION = 1.8;

/**
 Shortest token worth comparing. Single characters collide constantly across
 unrelated blocks and would drown the signal.
 */
const MIN_TOKEN_LENGTH = 2;

/**
 Extracts the script-neutral tokens of one block by a single linear scan: runs
 of Latin letters and digits, each folded (`foldLatinWord`), so a name written
 with its accent, a separate one or none is one token. Deliberately excludes
 CJK, whose characters carry meaning individually and would match across
 unrelated blocks. A scan rather than a pattern: the rule is "runs of Latin
 letters and digits", which an index walk states directly and runs in one pass
 with no backtracking on adversarial input. The runs took ASCII letters and
 digits only until ledger B18, which cut every accented word at its accent.

 @param text - block text to tokenize

 @returns Folded tokens, deduplicated

 @example
 ```ts
 const tokens = tokenize({ text: 'She played THE FINALS in 2023.', },);
 ```
 */
export function tokenize({ text, }: { readonly text: string; },): ReadonlySet<string> {
  /**
   Tokens found so far, deduplicated by construction.
   */
  const tokens = new Set<string>();

  /**
   Start index of the run currently being scanned.
   */
  let runStart = -1;
  // One step past the end, where `charAt` reads empty, closes the last run.
  for (let index = 0; index <= text.length; index += 1) {
    /**
     Code unit under the cursor, empty past the end.
     */
    const character = text.charAt(index,);

    /**
     Whether this position opens a token or goes on with the open one.
     */
    const inToken = (runStart < 0)
      ? isLatinWordCharacter({ character, },)
      : continuesLatinWord({ character, },);

    if (inToken) {
      if (runStart < 0)
        runStart = index;
      continue;
    }
    if (runStart >= 0) {
      /**
       Completed run, folded.
       */
      const token = foldLatinWord({
        word: text.slice(
          runStart,
          index,
        ),
      },);
      if (token.length >= MIN_TOKEN_LENGTH)
        tokens.add(token,);
      runStart = -1;
    }
  }
  return tokens;
}

/**
 Shared-token count at which overlap scores half its weight. Small because
 one agreeing proper noun is already meaningful across languages, while the
 tenth adds little.
 */
const OVERLAP_HALF_POINT = 2;

/**
 Overlap between two token sets, measured on the ABSOLUTE number of shared
 tokens with diminishing returns rather than as a share of either set.

 Sharing a set-relative measure was tried and is wrong: dividing by the
 smaller set lets a block carrying a single token score a perfect match
 against any long block containing that token, so a block with MORE evidence
 scores worse than one with less. On the corpus that inverted a real pairing,
 skipping the block that genuinely corresponded. Jaccard fails the opposite
 way here, since a short original against its longer rendering has a large
 union and vanishing overlap however well the two correspond.

 @param source - original block's tokens

 @param target - translation block's tokens

 @returns Overlap from zero (nothing shared) toward one, never reaching it

 @example
 ```ts
 const overlap = tokenOverlap({ source, target, },);
 ```
 */
function tokenOverlap(
  {
    source,
    target,
  }: {
    readonly source: ReadonlySet<string>;
    readonly target: ReadonlySet<string>;
  },
): number {
  /**
   Tokens both sides carry.
   */
  const shared = [...source,].filter(function inTarget(token,) {
    return target.has(token,);
  },)
    .length;
  return shared / (shared + OVERLAP_HALF_POINT);
}

/**
 How plausible the two blocks' lengths are as a translation pair, from zero
 to one. Peaks when the target runs about `expansion` times the source and
 decays smoothly, so it nudges rather than decides.

 @param sourceLength - original block's character count

 @param targetLength - translation block's character count

 @param expansion - characters this translation produces per source character

 @returns Plausibility from zero to one

 @example
 ```ts
 const fit = lengthPlausibility({ sourceLength: 10, targetLength: 18, expansion: 1.8, },);
 ```
 */
function lengthPlausibility(
  {
    sourceLength,
    targetLength,
    expansion,
  }: {
    readonly sourceLength: number;
    readonly targetLength: number;
    readonly expansion: number;
  },
): number {
  if ((sourceLength === 0) || (targetLength === 0))
    return 0;

  /**
   Observed ratio against the ratio THIS translation tends to produce.
   */
  const ratio = targetLength / (sourceLength * expansion);

  /**
   Symmetric distance from the ideal, so twice as long and half as long are
   penalized equally.
   */
  const deviation = ratio >= 1
    ? ratio
    : 1 / ratio;
  return 1 / deviation;
}

/**
 Characters across one side's blocks, each block's own text counted once.

 @param nodes - blocks of one side

 @returns Their characters, summed

 @example
 ```ts
 const chars = charsAcross({ nodes: sourceNodes, },);
 ```
 */
function charsAcross({ nodes, }: { readonly nodes: readonly DocumentNode[]; },): number {
  return nodes.reduce(
    function addChars(
      sum,
      node,
    ): number {
      /**
       This block's own characters.
       */
      const { text, } = node;
      return sum + text.length;
    },
    0,
  );
}

/**
 Estimates how far THIS translation expands, in characters per source
 character.

 Measured over the whole block lists rather than per pair, because a single
 block is exactly the thing whose pairing is in question and cannot be used to
 judge itself.

 @param sourceNodes - original blocks

 @param targetNodes - translation blocks

 @returns Characters produced per source character, or
 {@link FALLBACK_EXPANSION} when either side is empty

 @example
 ```ts
 const expansion = estimateExpansion({ sourceNodes, targetNodes, },);
 ```

 @internal
 */
export function estimateExpansion(
  {
    sourceNodes,
    targetNodes,
  }: {
    readonly sourceNodes: readonly DocumentNode[];
    readonly targetNodes: readonly DocumentNode[];
  },
): number {
  /**
   Total characters on the original side.
   */
  const sourceChars = charsAcross({ nodes: sourceNodes, },);

  /**
   Total characters on the translation side.
   */
  const targetChars = charsAcross({ nodes: targetNodes, },);
  if ((sourceChars === 0) || (targetChars === 0))
    return FALLBACK_EXPANSION;
  return targetChars / sourceChars;
}

/**
 Scores one candidate pairing. Higher is a better partnership.

 @param source - original block

 @param target - translation block

 @param expansion - characters this translation produces per source character,
 defaulting to {@link FALLBACK_EXPANSION} when the caller has no estimate

 @returns Pairing score, unbounded below and above

 @example
 ```ts
 const score = scorePairing({ source, target, },);
 ```
 */
export function scorePairing(
  {
    source,
    target,
    expansion = FALLBACK_EXPANSION,
  }: {
    readonly source: DocumentNode;
    readonly target: DocumentNode;
    readonly expansion?: number;
  },
): number {
  /**
   Structural agreement, the strongest single signal.
   */
  const kindScore = source.kind === target.kind
    ? KIND_MATCH_SCORE
    : KIND_MISMATCH_PENALTY;

  /**
   Literal agreement across names, years, and component names.
   */
  const overlapScore = TOKEN_OVERLAP_WEIGHT
    * tokenOverlap({
      source: tokenize({ text: source.text, },),
      target: tokenize({ text: target.text, },),
    },);

  /**
   Length agreement, a gentle tiebreaker.
   */
  const lengthScore = LENGTH_PLAUSIBILITY_WEIGHT
    * lengthPlausibility({
      sourceLength: source.text
        .length,
      targetLength: target.text
        .length,
      expansion,
    },);
  return kindScore + overlapScore
    + lengthScore;
}

//endregion Block alignment
