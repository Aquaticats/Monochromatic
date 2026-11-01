//region Reading corroboration
// WHETHER TWO MODELS READ THE SAME PICTURE, which is the only check on a
// reading that neither script nor subject matter can starve.
//
// WHAT THIS REPLACED. The screen used to compare a reading against the
// transcript the archive already carried, requiring two shared anchors. Real
// traffic on 2026-08-19 refused every reading of `Mio/7`'s two pictures, four
// attempts across two assets and two readers, while those same readings agreed
// with EACH OTHER at 0.967 and 1.000 character overlap. The clause assumed a
// slice's target-only English transcribes that slice's pictures, and where it is
// some other kind of addition every correct reading there fails.
//
// IT WAS ALSO STRUCTURALLY STARVED. A reading comes back in the picture's own
// language and the archive transcript is English, so the only tokens that can be
// shared are names, handles and numbers. Measured overlap was 2, 2, 4, 5, 5, 5
// against a floor of 2: two of six sat exactly on it, and a picture of pure
// Chinese prose could never clear it however correctly it was read.
//
// TRIGRAMS RATHER THAN ANCHORS OR CHARACTERS. Anchors are Latin words, digit
// runs and handles, so two readings of a Chinese picture carry none. Single
// characters fail the other way: two unrelated English texts share most of the
// alphabet, so single-character overlap runs near one on a wrong pair. Trigrams
// are starved by neither, which is why `CORROBORATION_TRIGRAM_SHARE` is stated over them.
//
// MEASURED, WITH ITS CONTROL. Over five pictures each read by both models, and
// every cross-pair as the control:
//
//   same picture, different models      5 pairs   0.643 to 1.000
//   different pictures                 40 pairs   0.000 to 0.129
//
// The threshold sits between them with about a factor of two of margin each way.
// Full evidence in `doc/audit/reading-a-picture-at-the-user-boundary.md`; the
// stated rule is `doc/planning/when-an-image-reading-makes-no-sense.md`.
//
// NO REGEX, per `RG1`: collapsing whitespace and cutting trigrams are both
// single linear passes, which a scan states more plainly than a pattern does.

/**
 Share of the smaller reading's trigrams the larger has to carry.

 THIRTY PERCENT, from the measured gap rather than from taste. The lowest
 same-picture pair scored 0.643 and the highest different-picture pair 0.129,
 so this sits roughly a factor of two from each. Stated as revisitable: five
 same-picture pairs show the separation exists without placing the boundary
 precisely.
 */
export const CORROBORATION_TRIGRAM_SHARE = 0.3;

/**
 Characters a gram spans.

 THREE, because two is too weak and four too brittle. Measured over the same
 pairs, bigrams separate 0.746-and-up from 0.292-and-below, a narrower gap,
 while trigrams separate 0.643 from 0.129.
 */
const GRAM_LENGTH = 3;

/**
 How a reading may fail to be corroborated, or that it was.

 @example
 ```ts
 const verdict: CorroborationVerdict = { kind: 'corroborated', overlap: 0.97, };
 ```
 */
export type CorroborationVerdict = {
  /**
   Both readings describe the same picture.
   */
  readonly kind: 'corroborated';

  /**
   Share of the smaller reading's trigrams the larger carried, recorded so a
   run can be read for how close its corroborations ran to the threshold.
   */
  readonly overlap: number;
} | {
  /**
   The two readings do not describe the same picture.
   */
  readonly kind: 'disagree';

  /**
   That same share, which is what disagreement means here.
   */
  readonly overlap: number;
};

/**
 Text with every whitespace run collapsed to one space, trimmed.

 WHY COLLAPSE AT ALL. Models differ in how they lay a transcription out, one
 preserving the picture's line breaks and another running lines together, and
 a trigram spanning a newline would differ from the same trigram spanning a
 space. Collapsing makes the comparison about the words rather than the layout.

 @param text - reading as the model returned it

 @returns Same characters with runs of whitespace flattened

 @example
 ```ts
 const flat = collapsedWhitespace({ text: reading, },);
 ```
 */
function collapsedWhitespace({ text, }: { readonly text: string; },): string {
  /**
   Characters kept so far.
   */
  const kept: string[] = [];

  /**
   Whether the last character emitted was a space, which is what stops a run
   of whitespace producing more than one.
   */
  const run = { spaced: true, };

  for (const character of text) {
    if (character.trim() === '') {
      if (!run.spaced) {
        kept.push(' ',);
        run.spaced = true;
      }
      continue;
    }
    kept.push(character,);
    run.spaced = false;
  }

  /**
   Those characters as one string, whose leading and trailing space is an
   artefact of the collapse rather than content.
   */
  const joined = kept.join('',);
  return joined.trim();
}

/**
 Segmenter cutting text into user-perceived characters.

 GRAPHEMES RATHER THAN UTF-16 UNITS OR CODE POINTS. Indexing a string splits a
 character outside the basic plane in half, and code points split a flag or a
 modified emoji into its parts, so both would cut grams neither reader would
 produce. A grapheme is what a person reading the picture would call one
 character, which is the unit the comparison is about.

 BUILT ONCE, because constructing a segmenter is far more expensive than using
 one and every reading is cut the same way.
 */
const GRAPHEMES = new Intl.Segmenter(
  undefined,
  { granularity: 'grapheme', },
);

/**
 Distinct character trigrams of one reading.

 @param text - reading to cut

 @returns Its distinct trigrams, empty when it is shorter than one

 @example
 ```ts
 const grams = characterTrigrams({ text: reading, },);
 ```
 */
export function characterTrigrams({ text, }: { readonly text: string; },): ReadonlySet<string> {
  /**
   Reading with its whitespace flattened, which is what gets cut.
   */
  const flat = collapsedWhitespace({ text, },);

  /**
   Its graphemes, in order.
   */
  const segments = [...GRAPHEMES.segment(flat,)];

  /**
   Each grapheme as a plain string, since a segment carries an index and an
   input alongside the characters themselves.
   */
  const points = segments.map(function ofSegment(
    { segment, }: { readonly segment: string; },
  ): string {
    return segment;
  },);

  /**
   Grams found so far.
   */
  const grams = new Set<string>();

  for (let at = 0; (at + GRAM_LENGTH) <= points.length; at += 1) {
    /**
     Graphemes this gram spans.
     */
    const span = points.slice(
      at,
      at + GRAM_LENGTH,
    );
    grams.add(span.join('',),);
  }

  return grams;
}

/**
 Share of the smaller reading's trigrams the larger one carries.

 THE SMALLER SIDE IS THE DENOMINATOR, deliberately. One model transcribes more
 of a picture than the other: Kimi-K3 read `Mio/photo7.webp` as 178 characters
 against Qwen's 590. Dividing by the union would score that pair as
 disagreement when what it shows is one reader stopping early, and the shorter
 reading still vouches for every word it does carry.

 @param left - one reading

 @param right - the other

 @returns Share between zero and one; zero when either carries no trigram

 @example
 ```ts
 const overlap = trigramOverlap({ left, right, },);
 ```
 */
export function trigramOverlap(
  {
    left,
    right,
  }: {
    readonly left: string;
    readonly right: string;
  },
): number {
  /**
   Grams of one side.
   */
  const leftGrams = characterTrigrams({ text: left, },);

  /**
   Grams of the other.
   */
  const rightGrams = characterTrigrams({ text: right, },);
  if ((leftGrams.size === 0) || (rightGrams.size === 0))
    return 0;

  /**
   Smaller side, whose grams are the ones asked about.
   */
  const smaller = (leftGrams.size <= rightGrams.size) ? leftGrams : rightGrams;

  /**
   Larger side, which is asked whether it carries them.
   */
  const larger = (leftGrams.size <= rightGrams.size) ? rightGrams : leftGrams;

  /**
   Grams of the smaller side the larger also carries.
   */
  const carried = [...smaller].filter(function inBoth(gram,): boolean {
    return larger.has(gram,);
  },);

  return carried.length / smaller.size;
}

/**
 Whether two readings describe the same picture.

 @param left - one reading

 @param right - the other, from a different model shown the same picture

 @returns Whether they agree, and by how much

 @example
 ```ts
 const verdict = readingsCorroborate({ left, right, },);
 ```
 */
export function readingsCorroborate(
  {
    left,
    right,
  }: {
    readonly left: string;
    readonly right: string;
  },
): CorroborationVerdict {
  /**
   How much of the smaller reading the larger carried.
   */
  const overlap = trigramOverlap({
    left,
    right,
  },);
  if (overlap < CORROBORATION_TRIGRAM_SHARE) {
    return {
      kind: 'disagree',
      overlap,
    };
  }
  return {
    kind: 'corroborated',
    overlap,
  };
}

//endregion Reading corroboration
