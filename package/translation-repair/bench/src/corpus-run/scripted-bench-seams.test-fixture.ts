import type { BenchSlice, } from '../../dist/final/node/index.mjs';

//region Scripted bench seams
// THE TWO THINGS A BENCH OR CALIBRATION RUN READS FROM THE PINNED CORPUS AND
// THE REPOSITORY, as scripts that record what they were asked: a drawer
// answering with invented slices of one cat, and a reader of the pipeline
// head.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. No clone is read and no git is run.

/**
 Commit the scripted head reports.
 */
export const SCRIPTED_HEAD = '0123456789abcdef0123456789abcdef01234567';

/**
 What the scripted seams were asked.
 */
type SeamsAsked = {
  /**
   Arguments of every draw.
   */
  readonly draws: unknown[];

  /**
   How many times the head was read.
   */
  heads: number;
};

/**
 Slice at a position of the invented entry.

 @param index - position within the entry

 @returns The slice

 @example
 ```ts
 const slice = benchSliceAt({ index: 0, },);
 ```
 */
function benchSliceAt({ index, }: { readonly index: number; },): BenchSlice {
  return {
    entryId: 'mittens',
    index,
    sourceText: '猫猫在窗台上打盹，尾巴垂在暖气片旁边。',
    incumbentText: 'The cat is doing the sleeping on the windowsill, with tail hanging by the radiator.',
    lineStructured: false,
  };
}

/**
 Drawer answering with a fixed number of slices, and a head reader, both
 recording what they were asked.

 @param slices - how many slices every draw answers with

 @returns The seams and their record

 @example
 ```ts
 const { asked, seams, } = scriptedDrawing({ slices: 2, },);
 ```
 */
export function scriptedDrawing(
  { slices, }: { readonly slices: number; },
): {
  readonly asked: SeamsAsked;
  readonly seams: {
    readonly drawSlices: (input: { readonly count: number; },) => Promise<readonly BenchSlice[]>;
    readonly readHead: () => Promise<string>;
  };
} {
  /**
   What the seams were asked.
   */
  const asked: SeamsAsked = {
    draws: [],
    heads: 0,
  };
  return {
    asked,
    seams: {
      drawSlices: function drawScripted(input,): Promise<readonly BenchSlice[]> {
        /**
         Draws recorded so far.
         */
        const { draws, } = asked;
        draws.push(input,);

        /**
         The slices this draw answers with.
         */
        const drawn = Array.from(
          { length: slices, },
          function at(
            _unused,
            index,
          ): BenchSlice {
            return benchSliceAt({ index, },);
          },
        );
        return Promise.resolve(drawn,);
      },
      readHead: function readScripted(): Promise<string> {
        asked.heads += 1;
        return Promise.resolve(SCRIPTED_HEAD,);
      },
    },
  };
}

//endregion Scripted bench seams
