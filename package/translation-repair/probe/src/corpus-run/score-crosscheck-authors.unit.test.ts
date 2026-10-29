/**
 Tests for the author table `score-crosscheck` prints.

 A claim counts once for every author of it, an undecided claim counts in
 neither arm, and an arm reads as able to carry a rate only from the floor up.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  authorHeaderLine,
  authorLine,
  type CrosscheckItem,
  tallyAuthors,
} from '../../dist/final/node/index.mjs';

/**
 One judgeable claim, seated by nothing the table reads.

 @param arm - arm the claim serves

 @param proposers - models that authored it

 @returns Item for the table

 @example
 ```ts
 const item = claimOf({ arm: 'accepted', proposers: ['hf:cat/Tabby-1',], },);
 ```
 */
function claimOf(
  {
    arm,
    proposers,
  }: {
    readonly arm: CrosscheckItem['arm'];
    readonly proposers: readonly string[];
  },
): CrosscheckItem {
  return {
    entryId: 'Whiskers',
    claimId: 'issue/nap',
    arm,
    status: arm,
    proposers,
    judges: [],
    barred: [],
  };
}

await describe({
  name: 'score-crosscheck-authors',
  children: [
    describe({
      name: tallyAuthors.name,
      children: [
        it({
          name: 'COUNTS a claim once for every author of it, and as sole only for a claim of one author',
          fn: async () => {
            expect(tallyAuthors({
              items: [
                claimOf({ arm: 'accepted', proposers: ['hf:cat/Tabby-1',], },),
                claimOf({ arm: 'accepted', proposers: ['hf:cat/Tabby-1', 'hf:cat/Mouser-1',], },),
                claimOf({ arm: 'control', proposers: ['hf:cat/Mouser-1',], },),
              ],
            },),).toStrictEqual([
              {
                modelId: 'hf:cat/Tabby-1',
                accepted: 2,
                control: 0,
                sole: 1,
              },
              {
                modelId: 'hf:cat/Mouser-1',
                accepted: 1,
                control: 1,
                sole: 1,
              },
            ],);
          },
        },),

        it({
          name: 'COUNTS an undecided claim in neither arm while still counting it as sole of its author',
          fn: async () => {
            expect(tallyAuthors({
              items: [claimOf({ arm: 'undecided', proposers: ['hf:cat/Tabby-1',], },),],
            },),).toStrictEqual([
              {
                modelId: 'hf:cat/Tabby-1',
                accepted: 0,
                control: 0,
                sole: 1,
              },
            ],);
          },
        },),

        it({
          name: 'ORDERS authors with as many accepted claims by model id, whichever claim came first',
          fn: async () => {
            /**
             Rows when the claim of the later model id came first.
             */
            const laterFirst = tallyAuthors({
              items: [
                claimOf({ arm: 'accepted', proposers: ['hf:cat/Tabby-1',], },),
                claimOf({ arm: 'accepted', proposers: ['hf:cat/Mouser-1',], },),
              ],
            },);

            /**
             Rows when the claim of the earlier model id came first.
             */
            const earlierFirst = tallyAuthors({
              items: [
                claimOf({ arm: 'accepted', proposers: ['hf:cat/Mouser-1',], },),
                claimOf({ arm: 'accepted', proposers: ['hf:cat/Tabby-1',], },),
              ],
            },);

            expect(laterFirst.map(function toId({ modelId, },): string {
              return modelId;
            },),).toStrictEqual(['hf:cat/Mouser-1', 'hf:cat/Tabby-1',],);
            expect(earlierFirst,).toStrictEqual(laterFirst,);
          },
        },),

        it({
          name: 'TALLIES no author for no claim',
          fn: async () => {
            expect(tallyAuthors({ items: [], },),).toStrictEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: authorHeaderLine.name,
      children: [
        it({
          name: 'RENDERS the header over the model, both arms, the sole count and the floor',
          fn: async () => {
            expect(authorHeaderLine(),).toBe(
              'AUTHOR                                                accepted  control  sole  floor',
            );
          },
        },),
      ],
    },),

    describe({
      name: authorLine.name,
      children: [
        it({
          name: 'NAMES neither arm as clearing the floor below it',
          fn: async () => {
            expect(authorLine({
              row: {
                modelId: 'hf:cat/Tabby-1',
                accepted: 29,
                control: 29,
                sole: 3,
              },
            },),).toBe(
              'hf:cat/Tabby-1                                              29       29     3  neither',
            );
          },
        },),

        it({
          name: 'NAMES the accepted arm alone where only it reaches the floor',
          fn: async () => {
            expect(authorLine({
              row: {
                modelId: 'hf:cat/Tabby-1',
                accepted: 30,
                control: 29,
                sole: 3,
              },
            },),).toBe(
              'hf:cat/Tabby-1                                              30       29     3  accepted',
            );
          },
        },),

        it({
          name: 'NAMES the control arm alone where only it reaches the floor',
          fn: async () => {
            expect(authorLine({
              row: {
                modelId: 'hf:cat/Tabby-1',
                accepted: 0,
                control: 30,
                sole: 0,
              },
            },),).toBe(
              'hf:cat/Tabby-1                                               0       30     0  control',
            );
          },
        },),

        it({
          name: 'NAMES both arms where both reach the floor',
          fn: async () => {
            expect(authorLine({
              row: {
                modelId: 'hf:cat/Tabby-1',
                accepted: 31,
                control: 40,
                sole: 12,
              },
            },),).toBe(
              'hf:cat/Tabby-1                                              31       40    12  accepted+control',
            );
          },
        },),
      ],
    },),
  ],
},);
