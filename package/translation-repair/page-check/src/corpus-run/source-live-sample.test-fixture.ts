import {
  CONTAINERS,
  ENGLISH_MIDDLES,
  FOOTNOTE_STAGES,
  RUN_FRAGMENTS,
  RUN_SEPARATORS,
} from './source-live-sample-pool.test-fixture.ts';

//region Seeded samples of source texts
// THE TEXTS THE WHOLE-PAGE CHECK OF LIVE CONSTRUCTS IS RUN OVER (`source-live-sample.unit.test.ts`). Two sets are
// drawn, each with a seed, so a failure names texts that can be run again:
//
// - the generated footnote texts, a footnote definition followed by lines indented four spaces or more, one of them
//   starting with a fence mark, in a bare, list item and block quote setting and in a bare setting with lone carriage
//   returns for line endings (32256 texts, every combination of the parts in
//   `source-live-sample-pool.test-fixture.ts`);
// - pairs of runs, each built from a pool of fragments that hold braces, tags, comments, fences, math, footnotes,
//   addresses and the other syntax the encoder writes, with English between the runs (the pairs a seeded search of
//   forty thousand draws from the seeds 31415926 and 27182818 reads, the draws here being a fixed fraction of them).
//
// A SAMPLE, NOT THE SETS: the sets take about a minute to read whole, and the cases that pin each defect found
// (`source-ships-encoding.unit.test.ts`, `source-shipped-page.unit.test.ts`) read the shapes one by one. The sample
// is the check that no shape no case names puts code on a page, and it stays at about ten seconds.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.

/**
 One pair of runs of source text and the English the archive holds between them.
 */
export type RunPair = {
  /**
   Source text of the first run.
   */
  readonly first: string;

  /**
   English between the runs.
   */
  readonly middle: string;

  /**
   Source text of the second run.
   */
  readonly second: string;
};

/**
 A pair of runs with its place in the stream of draws it was taken from.
 */
type SampledPair = RunPair & {
  /**
   Index of the draw, so a failure can be replayed by drawing up to it.
   */
  readonly at: number;
};

/**
 Most fragments a run holds.
 */
const FRAGMENTS_AT_MOST = 3;

/**
 First step of the seeded generator, added to the state before every draw.
 */
const STATE_STEP = 0x6D_2B_79_F5;

/**
 Shifts and bit masks of the scramble the generator applies to its state.
 */
const SCRAMBLE = {
  firstShift: 15,
  secondShift: 7,
  thirdShift: 14,
  secondMask: 61,
};

/**
 Makes a seeded generator, with 32-bit integer arithmetic only so no product leaves the range a double holds exactly.

 @param seed - starting state

 @returns Function giving an integer under its limit

 @example
 ```ts
 const draw = seededDraw({ seed: 31_415_926, },);
 ```
 */
function seededDraw({ seed, }: { readonly seed: number; },): (limit: number,) => number {
  /**
   State the generator advances.
   */
  const state = { seed, };
  return function draw(limit: number,): number {
    // A product by one reads the sum as a 32-bit integer, which wraps it as the generator needs.
    state.seed = Math.imul(
      state.seed + STATE_STEP,
      1,
    );
    /**
     Mixed copy of the state.
     */
    const mixed = Math.imul(
      state.seed ^ (state.seed >>> SCRAMBLE.firstShift),
      state.seed | 1,
    );
    /**
     The bits of the mixed copy that the mask also sets.
     */
    const masked = mixed & SCRAMBLE.secondMask;
    /**
     The mixed copy with the mask's bits set, which is `mixed | mask` written as a sum.
     */
    const raised = (mixed - masked) + SCRAMBLE.secondMask;
    /**
     Mixed copy again, the final scramble.
     */
    const scrambled = mixed ^ (mixed + Math.imul(
      mixed ^ (mixed >>> SCRAMBLE.secondShift),
      raised,
    ));
    return ((scrambled ^ (scrambled >>> SCRAMBLE.thirdShift)) >>> 0) % limit;
  };
}

/**
 Draws one element of a list.

 @param list - elements to draw from, not empty

 @param draw - generator of integers under a limit

 @returns Element the generator picks

 @example
 ```ts
 const separator = drawFrom({ list: RUN_SEPARATORS, draw, },);
 ```
 */
function drawFrom(
  {
    list,
    draw,
  }: {
    readonly list: readonly string[];
    readonly draw: (limit: number,) => number;
  },
): string {
  /**
   Index the generator picks.
   */
  const at = draw(list.length,);
  return list[at] ?? '';
}

/**
 Draws one run, one to three fragments joined by one separator.

 @param draw - generator of integers under a limit

 @returns Source text of a run, ending in a line ending

 @example
 ```ts
 const run = drawRun({ draw, },);
 ```
 */
function drawRun({ draw, }: { readonly draw: (limit: number,) => number; },): string {
  /**
   How many fragments the run holds.
   */
  const count = 1 + draw(FRAGMENTS_AT_MOST,);
  /**
   The fragments, drawn in order.
   */
  const parts = Array.from(
    { length: count, },
    function drawPart(): string {
      return drawFrom({
        list: RUN_FRAGMENTS,
        draw,
      },);
    },
  );
  /**
   What goes between the fragments.
   */
  const separator = drawFrom({
    list: RUN_SEPARATORS,
    draw,
  },);
  return `${parts.join(separator,)}\n`;
}

/**
 Draws pairs of runs from a seeded stream and keeps a fixed fraction of them, picked by a second seeded stream.

 Every draw is made, so the pairs kept are those the same seed gives a search of `total` pairs, and a failure names the
 index of its draw.

 @param seed - seed of the stream of pairs

 @param total - how many pairs the stream holds

 @param sampleSeed - seed of the stream that picks the pairs to keep

 @param oneIn - pairs kept are one in this many, on average

 @returns Pairs kept, in the order drawn

 @example
 ```ts
 const pairs = sampledPairs({ seed: 31_415_926, total: 40_000, sampleSeed: 1, oneIn: 16, },);
 ```
 */
export function sampledPairs(
  {
    seed,
    total,
    sampleSeed,
    oneIn,
  }: {
    readonly seed: number;
    readonly total: number;
    readonly sampleSeed: number;
    readonly oneIn: number;
  },
): readonly SampledPair[] {
  /**
   Generator of the stream of pairs.
   */
  const draw = seededDraw({ seed, },);
  /**
   Generator that picks the pairs to keep.
   */
  const pick = seededDraw({ seed: sampleSeed, },);
  return Array.from(
    { length: total, },
    function drawPair(
      _unused,
      at,
    ): SampledPair {
      /**
       Source text of the first run.
       */
      const first = drawRun({ draw, },);
      /**
       Source text of the second run, drawn after the first.
       */
      const second = drawRun({ draw, },);
      return {
        at,
        first,
        middle: drawFrom({
          list: ENGLISH_MIDDLES,
          draw,
        },),
        second,
      };
    },
  )
    .filter(function isKept(): boolean {
      return pick(oneIn,) === 0;
    },);
}

/**
 Every generated footnote text: each combination of the stages, in each container.

 @returns 32256 texts, a line ending after the last line of each

 @example
 ```ts
 const texts = footnoteProductTexts();
 ```
 */
export function footnoteProductTexts(): readonly string[] {
  /**
   Every combination of the stages as lines, the last stage varying fastest.
   */
  const bodies = FOOTNOTE_STAGES.reduce<readonly (readonly string[])[]>(
    function extend(
      rows,
      stage,
    ): readonly (readonly string[])[] {
      return rows.flatMap(function withEach(row,): readonly (readonly string[])[] {
        return stage.map(function join(option,): readonly string[] {
          return [
            ...row,
            ...((option === '') ? [] : option.split('\n',)),
          ];
        },);
      },);
    },
    [[],],
  );
  return CONTAINERS.flatMap(function inContainer(
    {
      open,
      hang,
      join,
    },
  ): readonly string[] {
    return bodies.map(function toText(lines,): string {
      /**
       The lines with the container's prefix: the first line takes the opening one, the others the hanging one.
       */
      const prefixed = lines.map(function prefix(
        line,
        at,
      ): string {
        return `${(at === 0) ? open : hang}${line}`;
      },);
      return `${prefixed.join(join,)}${join}`;
    },);
  },);
}

/**
 Keeps a fixed fraction of a list of texts, picked by a seeded stream.

 @param texts - whole set, in order

 @param sampleSeed - seed of the stream that picks the texts to keep

 @param oneIn - texts kept are one in this many, on average

 @returns Texts kept, in the order given

 @example
 ```ts
 const kept = sampledTexts({ texts: footnoteProductTexts(), sampleSeed: 1, oneIn: 6, },);
 ```
 */
export function sampledTexts(
  {
    texts,
    sampleSeed,
    oneIn,
  }: {
    readonly texts: readonly string[];
    readonly sampleSeed: number;
    readonly oneIn: number;
  },
): readonly string[] {
  /**
   Generator that picks the texts to keep.
   */
  const pick = seededDraw({ seed: sampleSeed, },);
  return texts.filter(function isKept(): boolean {
    return pick(oneIn,) === 0;
  },);
}

//endregion Seeded samples of source texts
