/**
 Tests for the aligner that can refuse.

 The case that matters is `XingZ60`, whose headings are reproduced here from
 the section-alignment defect record. The shipped aligner paired every one of its
 sections with the wrong one, shifted by two, because its scorer cannot
 withhold a pairing: pairing two headings that share nothing scores zero
 against a negative for leaving both unpaired, so the maximum always prefers
 the unsupported pairing.

 Other fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  alignHeadingsForced,
  type ForcedAlignStep,
  type InsertionAnchor,
} from '../dist/final/node/index.mjs';

/**
 Reads the target a source index was paired with, or undefined.

 @param steps - alignment steps

 @param sourceIndex - source unit to look up

 Returns an ARRAY rather than an optional number, because the repo models
 absence without nullish unions and a bare -1 sentinel is the same mistake
 wearing a different hat.

 @returns Single-element array with the target index, empty when unpaired

 @example
 ```ts
 const partner = pairedWith({ steps, sourceIndex: 0, },);
 ```
 */
function pairedWith(
  {
    steps,
    sourceIndex,
  }: {
    readonly steps: readonly {
      readonly kind: string;
      readonly sourceIndex?: number;
      readonly targetIndex?: number;
    }[];
    readonly sourceIndex: number;
  },
): readonly number[] {
  return steps.flatMap(function toTarget(step,): readonly number[] {
    if ((step.kind !== 'paired') || (step.sourceIndex !== sourceIndex))
      return [];
    return (step.targetIndex === undefined) ? [] : [step.targetIndex,];
  },);
}


/**
 Reads one source unit's insertion anchor out of a step list.

 @param steps - aligner output

 @param sourceIndex - unit to read

 @returns Its anchor

 @throws Error when that unit was paired or is missing, since a test expecting
 an anchor there is asserting about something the aligner did not produce and
 should say so rather than compare against a blank

 @example
 ```ts
 const anchor = anchorOf({ steps, sourceIndex: 1, },);
 ```
 */
function anchorOf(
  {
    steps,
    sourceIndex,
  }: {
    readonly steps: readonly ForcedAlignStep[];
    readonly sourceIndex: number;
  },
): InsertionAnchor {
  /**
   The unpaired step for that unit.
   */
  const found = steps.find(function atIndex(step,): boolean {
    return (step.kind === 'source-only') && (step.sourceIndex === sourceIndex);
  },);

  if ((found === undefined) || (found.kind !== 'source-only'))
    throw new Error(
      `no source-only step at index ${String(sourceIndex,)}; the aligner either paired that unit `
        + 'or never emitted it, and an anchor assertion about it is asserting about nothing',
    );

  return found.anchor;
}

await describe({
  name: alignHeadingsForced.name,
  children: [
    it({
      name: 'pairs XingZ60 BY NAME rather than by position, which is the defect '
        + 'itself: the shipped aligner shifted every section by two because a '
        + 'gap costs more than an unsupported pairing, so every critic call on '
        + 'that entry compared the wrong original against the wrong translation',
      fn: async () => {
        /**
         Chinese headings, which carry the names inside a numbered prefix.
         */
        const sourceHeadings = [
          '其一：伊良子',
          '其二：铃语',
          '其三：绘都',
          '其四：无常',
          '其六：Mikä',
          '其七：wing',
          '其八：白毛 suki',
        ];

        /**
         English headings, two fewer, with the names romanised.
         */
        const targetHeadings = [
          'Engagement in Trans Aid',
          'Memories by Friends',
          'Irako',
          'Lingyu',
          'Ann',
          'Shinonome',
          'Mikä',
          'wing',
          'Baimao suki',
        ];

        /**
         Steps the aligner produced.
         */
        const steps = alignHeadingsForced({
          sourceHeadings,
          targetHeadings,
        },);

        // The romanised names are the evidence, and the aligner must use them.
        expect(pairedWith({ steps, sourceIndex: 4, },),).toStrictEqual([6,],);
        expect(pairedWith({ steps, sourceIndex: 5, },),).toStrictEqual([7,],);
        expect(pairedWith({ steps, sourceIndex: 6, },),).toStrictEqual([8,],);

        // The shifted-by-two pairing the shipped aligner produced must NOT
        // reappear: 其六：Mikä paired with Ann is the exact wrong answer.
        expect(pairedWith({ steps, sourceIndex: 4, },),).not.toStrictEqual([4,],);
      },
    },),

    it({
      name: 'REFUSES rather than guessing when the counts differ and the sides '
        + 'share no evidence, which is the XIEPT2 shape and the capability the '
        + 'shipped scorer lacks. With EQUAL counts and no evidence the diagonal '
        + 'is still forced, because pairing costs no gaps and gapping costs '
        + 'four, so pairing in order there is correct rather than a guess',
      fn: async () => {
        /**
         Headings with nothing in common, and an unequal number of them, so
         no single assignment is forced.
         */
        const steps = alignHeadingsForced({
          sourceHeadings: ['第一章', '第二章',],
          targetHeadings: ['Sleeping', 'Climbing', 'Purring',],
        },);

        expect(
          steps.every(function isUnpaired(step,) {
            return step.kind !== 'paired';
          },),
        ).toBe(true,);
        expect(
          steps.every(function isAmbiguous(step,) {
            return (step.kind === 'paired') || (step.reason === 'ambiguous');
          },),
        ).toBe(true,);
      },
    },),

    it({
      name: 'pairs a clean one-to-one sequence outright, so refusing is '
        + 'reserved for genuine ambiguity and does not cost ordinary entries',
      fn: async () => {
        /**
         Identical headings on both sides.
         */
        const steps = alignHeadingsForced({
          sourceHeadings: ['Mittens', 'Whiskers', 'Shadow',],
          targetHeadings: ['Mittens', 'Whiskers', 'Shadow',],
        },);

        expect(pairedWith({ steps, sourceIndex: 0, },),).toStrictEqual([0,],);
        expect(pairedWith({ steps, sourceIndex: 1, },),).toStrictEqual([1,],);
        expect(pairedWith({ steps, sourceIndex: 2, },),).toStrictEqual([2,],);
      },
    },),

    it({
      name: 'reports a missing section as a FORCED GAP rather than as '
        + 'ambiguity, since the two readings call for different handling: '
        + 'nothing to pair with, versus too many things to choose between',
      fn: async () => {
        /**
         Target missing the middle section entirely.
         */
        const steps = alignHeadingsForced({
          sourceHeadings: ['Mittens', 'Whiskers', 'Shadow',],
          targetHeadings: ['Mittens', 'Shadow',],
        },);

        /**
         Step for the section with no counterpart.
         */
        const orphan = steps.find(function isWhiskers(step,) {
          return (step.kind === 'source-only') && (step.sourceIndex === 1);
        },);

        expect(orphan,).toBeDefined();
        expect(pairedWith({ steps, sourceIndex: 0, },),).toStrictEqual([0,],);
        expect(pairedWith({ steps, sourceIndex: 2, },),).toStrictEqual([1,],);
      },
    },),

    it({
      name: 'does NOT anchor on a name repeated across headings, since a '
        + 'trusted anchor must be the strict maximum of both its row and its '
        + 'column and a repeat is the maximum of neither',
      fn: async () => {
        /**
         The same name in two source headings.
         */
        const steps = alignHeadingsForced({
          sourceHeadings: ['Mittens morning', 'Mittens evening',],
          targetHeadings: ['Mittens',],
        },);

        // Either pairing is equally good, so neither is forced.
        expect(
          steps.filter(function isPaired(step,) {
            return step.kind === 'paired';
          },),
        ).toHaveLength(0,);
      },
    },),

    it({
      name: 'returns one step per unit on both sides, so no section can be '
        + 'silently lost between the aligner and its caller',
      fn: async () => {
        /**
         Uneven sequences.
         */
        const steps = alignHeadingsForced({
          sourceHeadings: ['Mittens', 'Whiskers',],
          targetHeadings: ['Mittens', 'Shadow', 'Dusty',],
        },);

        /**
         Source units mentioned anywhere in the output.
         */
        const sources = new Set(steps.flatMap(function toSource(step,) {
          return (step.kind === 'target-only') ? [] : [step.sourceIndex,];
        },),);

        /**
         Target units mentioned anywhere in the output.
         */
        const targets = new Set(steps.flatMap(function toTarget(step,) {
          return (step.kind === 'source-only') ? [] : [step.targetIndex,];
        },),);

        expect(sources.size,).toBe(2,);
        expect(targets.size,).toBe(3,);
      },
    },),

    it({
      name: 'PROVES WHERE A MISSING SECTION BELONGS, which is the half an insertion cannot '
        + 'proceed without: knowing a section is untranslated does not say where its rendering '
        + 'goes, and every optimal alignment here skips it at the same place',
      fn: async () => {
        /**
         One section absent from the middle of the translation.
         */
        const steps = alignHeadingsForced({
          sourceHeadings: ['Whiskers', 'Mittens', 'Paws',],
          targetHeadings: ['Whiskers', 'Paws',],
        },);

        expect(anchorOf({
          steps,
          sourceIndex: 1,
        },),)
          .toStrictEqual({
            kind: 'proven',
            beforeTargetIndex: 1,
          },);
      },
    },),

    it({
      name: 'ANCHORS A TRAILING RUN PAST THE LAST TRANSLATED SECTION, at the target length '
        + 'rather than at the last index, since the sections belong after everything the page '
        + 'carries and an off-by-one there writes them inside the previous section',
      fn: async () => {
        /**
         Two sections absent from the end.
         */
        const steps = alignHeadingsForced({
          sourceHeadings: ['Whiskers', 'Mittens', 'Paws',],
          targetHeadings: ['Whiskers',],
        },);

        expect([
          anchorOf({
            steps,
            sourceIndex: 1,
          },),
          anchorOf({
            steps,
            sourceIndex: 2,
          },),
        ],)
          .toStrictEqual([
            {
              kind: 'proven',
              beforeTargetIndex: 1,
            },
            {
              kind: 'proven',
              beforeTargetIndex: 1,
            },
          ],);
      },
    },),

    it({
      name: 'ANCHORS EVERY SECTION OF AN UNTRANSLATED PAGE at the start of a target that holds '
        + 'nothing, rather than leaving a page nobody translated with no place to put any of it',
      fn: async () => {
        /**
         A page whose translation carries no sections at all.
         */
        const steps = alignHeadingsForced({
          sourceHeadings: ['Whiskers', 'Mittens',],
          targetHeadings: [],
        },);

        expect(steps.map(function toAnchor(step,) {
          return (step.kind === 'source-only') ? step.anchor : undefined;
        },),)
          .toStrictEqual([
            {
              kind: 'proven',
              beforeTargetIndex: 0,
            },
            {
              kind: 'proven',
              beforeTargetIndex: 0,
            },
          ],);
      },
    },),

    it({
      name: 'REFUSES TO PLACE A SECTION THAT COULD PAIR, since an optimal alignment matching it '
        + 'against existing translation means whatever it says may already be on the page, and '
        + 'inserting it would write the page own content in twice',
      fn: async () => {
        /**
         Two sides sharing no evidence, where every pairing stays possible.
         */
        const steps = alignHeadingsForced({
          sourceHeadings: ['Whiskers', 'Mittens', 'Boots',],
          targetHeadings: ['Sunbeam',],
        },);

        expect(steps.flatMap(function toKind(step,) {
          return (step.kind === 'source-only') ? [step.anchor.kind,] : [];
        },),)
          .toStrictEqual([
            'may-pair',
            'may-pair',
            'may-pair',
          ],);
      },
    },),

    it({
      name: 'REFUSES TO PLACE A SECTION THE OPTIMAL ALIGNMENTS DISAGREE ABOUT, and says how many '
        + 'places it could go. An orphan between two identical headings is genuinely missing, '
        + 'but which side of the one surviving translation it belongs to is undetermined, and '
        + 'guessing files real content under the wrong section',
      fn: async () => {
        /**
         An unmatched section framed by a repeated heading.
         */
        const steps = alignHeadingsForced({
          sourceHeadings: ['Whiskers', 'Mittens', 'Whiskers',],
          targetHeadings: ['Whiskers',],
        },);

        expect(anchorOf({
          steps,
          sourceIndex: 1,
        },),)
          .toStrictEqual({
            kind: 'several-boundaries',
            boundaries: [
              0,
              1,
            ],
          },);
      },
    },),
  ],
},);
