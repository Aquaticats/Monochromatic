/**
 Tests for enumerating the claims a disinterested judge may re-examine.

 The cases that matter are the ones that would shrink a denominator without
 looking like it: a claim the whole roster proposed, a claim an issue names
 that attribution never covered, and an entry settled before attribution
 existed. Each is a real shape in the current run's artifacts, and each would
 lift every rate above it if it silently vanished.

 Entry ids and claim ids are invented and cat-themed. No corpus text is
 involved. Model ids are the real roster, because the rule under test is about
 the relationship between authorship and the seats left over.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildCrosscheckCensus,
  statusBreakdown,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Roster the census seats judges from, which is the shipped one.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

/**
 Builds one chunk record carrying the given attributions.

 @param claims - claim id paired with its proposing model ids

 @returns Chunk view shaped as the artifact carries it

 @example
 ```ts
 const chunk = chunkWith({ claims: [['issue/whisker', [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],],], },);
 ```
 */
function chunkWith(
  { claims, }: { readonly claims: readonly (readonly [string, readonly string[],])[]; },
) {
  return {
    sliceIndex: 0,
    heardCriticIds: [...ROSTER,],
    claimAttributions: claims.map(function toAttribution([claimId, proposers,],) {
      return {
        claimId,
        // One emission each. The census never reads the count, but the view
        // carries it and the decoder rejects anything below one, so fixtures
        // that omitted it would be shapes no artifact can hold.
        proposers: proposers.map(function toProposer(modelId,) {
          return {
            modelId,
            emissionCount: 1,
          };
        },),
      };
    },),
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: buildCrosscheckCensus.name,
      children: [
        it({
          name: 'seats every model that did not propose the claim, and splits '
            + 'accepted from not-accepted into the two arms the reading compares',
          fn: async () => {
            const census = buildCrosscheckCensus({
              entries: [
                {
                  id: 'Whiskers',
                  sliceCritics: [
                    chunkWith({
                      claims: [
                        ['issue/mackerel', [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],],
                        ['issue/sardine', [SEAT_SYNTHETIC_TEXT_EVERYWHERE,],],
                      ],
                    },),
                  ],
                  issues: [
                    {
                      status: 'accepted',
                      claimIds: ['issue/mackerel',],
                    },
                    {
                      status: 'rejected',
                      claimIds: ['issue/sardine',],
                    },
                  ],
                },
              ],
              roster: ROSTER,
            },);

            expect(census.items.length,).toBe(2,);
            expect(census.unjudgeable.length,).toBe(0,);

            const [accepted, control,] = census.items;
            expect(accepted?.arm,).toBe('accepted',);
            expect(control?.arm,).toBe('control',);

            // The author is barred and the other five are seated, which is the
            // common case: sole authorship covered 298 of 299 attributed claims.
            expect(accepted?.judges.length,).toBe(ROSTER.length - 1,);
            expect(accepted?.judges.includes(SEAT_HYPER_OPENROUTER_VISION_EDITOR,),).toBe(false,);
            expect(accepted?.barred,).toEqual([SEAT_HYPER_OPENROUTER_VISION_EDITOR,],);
          },
        },),

        it({
          name: 'holds needs-human OUT of the control arm while keeping '
            + 'source-defect in it, because rejected and source-defect are '
            + 'verdicts a judge can agree with and declining to decide is not. '
            + 'Those claims lean supported 228 to 23 on the current run, so '
            + 'filing them as control would fill it with claims the panel believed',
          fn: async () => {
            const census = buildCrosscheckCensus({
              entries: [
                {
                  id: 'Mittens',
                  sliceCritics: [
                    chunkWith({
                      claims: [
                        ['issue/tuna', [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],],
                        ['issue/salmon', [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],],
                      ],
                    },),
                  ],
                  issues: [
                    {
                      status: 'needs-human',
                      claimIds: ['issue/tuna',],
                    },
                    {
                      status: 'source-defect',
                      claimIds: ['issue/salmon',],
                    },
                  ],
                },
              ],
              roster: ROSTER,
            },);

            expect(census.items.map(function toArm(item,) {
              return item.arm;
            },),).toEqual(['undecided', 'control',],);

            // Status is kept verbatim beside the arm so a control result can be
            // broken down by reason without re-reading artifacts.
            expect(census.items.map(function toStatus(item,) {
              return item.status;
            },),).toEqual(['needs-human', 'source-defect',],);
          },
        },),

        it({
          name: 'reports a claim the whole roster proposed instead of dropping it, '
            + 'which is the exclusion that would lift every rate while looking '
            + 'entirely ordinary',
          fn: async () => {
            const census = buildCrosscheckCensus({
              entries: [
                {
                  id: 'Tabby',
                  sliceCritics: [
                    chunkWith({
                      claims: [
                        ['issue/unanimous', [...ROSTER,],],
                        ['issue/ordinary', [SEAT_SYNTHETIC_VISION_WITHHELD,],],
                      ],
                    },),
                  ],
                  issues: [
                    {
                      status: 'accepted',
                      claimIds: ['issue/unanimous', 'issue/ordinary',],
                    },
                  ],
                },
              ],
              roster: ROSTER,
            },);

            expect(census.items.length,).toBe(1,);
            expect(census.unjudgeable.length,).toBe(1,);
            expect(census.unjudgeable[0]?.claimId,).toBe('issue/unanimous',);
            expect(census.unjudgeable[0]?.barred.length,).toBe(ROSTER.length,);

            // The whole population is still accounted for. Were the unjudgeable
            // claim merely filtered away, this sum would read 1 and a rate over it
            // would be computed against a denominator missing the single most
            // corroborated claim in the entry.
            expect(census.items.length + census.unjudgeable.length,).toBe(2,);
          },
        },),

        it({
          name: 'reports a claim missing from an ATTRIBUTED entry as a join '
            + 'failure rather than as a legacy claim, since on an entry whose '
            + 'critics were attributed every surviving claim should have a '
            + 'proposer and folding it in would hide a broken join',
          fn: async () => {
            const census = buildCrosscheckCensus({
              entries: [
                {
                  id: 'Calico',
                  sliceCritics: [
                    chunkWith({ claims: [['issue/known', [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],],], },),
                  ],
                  issues: [
                    {
                      status: 'accepted',
                      claimIds: ['issue/known', 'issue/orphaned',],
                    },
                  ],
                },
              ],
              roster: ROSTER,
            },);

            expect(census.items.length,).toBe(1,);
            expect(census.unattributedJoinFailures,).toBe(1,);
            expect(census.unattributedLegacyClaims,).toBe(0,);
          },
        },),

        it({
          name: 'counts entries settled before attribution existed, which is 14 of '
            + 'the 19 in the current run and the reason the census covers a '
            + 'fraction of the accepted issues',
          fn: async () => {
            const census = buildCrosscheckCensus({
              entries: [
                {
                  id: 'Ginger',
                  issues: [
                    {
                      status: 'accepted',
                      claimIds: ['issue/old',],
                    },
                  ],
                },
                {
                  id: 'Sooty',
                  sliceCritics: [
                    chunkWith({ claims: [['issue/new', [SEAT_SYNTHETIC_TEXT_EVERYWHERE,],],], },),
                  ],
                  issues: [
                    {
                      status: 'accepted',
                      claimIds: ['issue/new',],
                    },
                  ],
                },
              ],
              roster: ROSTER,
            },);

            expect(census.entriesCovered,).toBe(2,);
            expect(census.entriesWithoutAttribution,).toBe(1,);
            expect(census.items.length,).toBe(1,);

            // The legacy entry's claim is expected absence, not a broken join, and
            // the two must stay apart: 1368 claims on this run sit in the legacy
            // count, so a join failure folded in with them would be invisible.
            expect(census.unattributedLegacyClaims,).toBe(1,);
            expect(census.unattributedJoinFailures,).toBe(0,);
          },
        },),

        it({
          name: 'lets a retired proposer bar nobody, since artifacts written before '
            + 'the 2026-08-05 roster change still name two ids the provider no '
            + 'longer serves',
          fn: async () => {
            const census = buildCrosscheckCensus({
              entries: [
                {
                  id: 'Smudge',
                  sliceCritics: [
                    chunkWith({ claims: [['issue/legacy', ['hf:retired/model-that-left',],],], },),
                  ],
                  issues: [
                    {
                      status: 'accepted',
                      claimIds: ['issue/legacy',],
                    },
                  ],
                },
              ],
              roster: ROSTER,
            },);

            expect(census.items[0]?.judges.length,).toBe(ROSTER.length,);
            expect(census.items[0]?.barred,).toEqual([],);
          },
        },),

        it({
          name: 'files a status spelled as a name every object inherits as undecided, like any status it does not '
            + 'know, where a plain-object table of arms answered with the inherited function or object (ledger B77)',
          fn: async () => {
            const census = buildCrosscheckCensus({
              entries: [
                {
                  id: 'Pepper',
                  sliceCritics: [
                    chunkWith({
                      claims: [
                        ['issue/built', [SEAT_HYPER_VISION,],],
                        ['issue/linked', [SEAT_HYPER_VISION,],],
                      ],
                    },),
                  ],
                  issues: [
                    {
                      status: 'constructor',
                      claimIds: ['issue/built',],
                    },
                    {
                      status: '__proto__',
                      claimIds: ['issue/linked',],
                    },
                  ],
                },
              ],
              roster: ROSTER,
            },);

            expect(census.items.map(function toArm(item,) {
              return item.arm;
            },),).toEqual(['undecided', 'undecided',],);
          },
        },),

        it({
          name: 'bars every model that proposed a claim in any chunk of its entry, each once, where two chunks carry '
            + 'one claim id, since the writer keeps such chunks apart and the last chunk alone would seat the '
            + 'first chunk\'s author as a judge of its own claim',
          fn: async () => {
            const census = buildCrosscheckCensus({
              entries: [
                {
                  id: 'Biscuit',
                  sliceCritics: [
                    chunkWith({
                      claims: [['issue/herring', [SEAT_HYPER_VISION, SEAT_SYNTHETIC_VISION_WITHHELD,],],],
                    },),
                    {
                      ...chunkWith({
                        claims: [['issue/herring', [SEAT_SYNTHETIC_TEXT_EVERYWHERE, SEAT_HYPER_VISION,],],],
                      },),
                      sliceIndex: 1,
                    },
                  ],
                  issues: [
                    {
                      status: 'accepted',
                      claimIds: ['issue/herring',],
                    },
                  ],
                },
              ],
              roster: ROSTER,
            },);

            expect(census.items,).toEqual([
              {
                entryId: 'Biscuit',
                claimId: 'issue/herring',
                arm: 'accepted',
                status: 'accepted',
                proposers: [SEAT_HYPER_VISION, SEAT_SYNTHETIC_VISION_WITHHELD, SEAT_SYNTHETIC_TEXT_EVERYWHERE,],
                judges: [
                  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                  SEAT_HYPER_OPENROUTER_UNMEASURED,
                ],
                barred: [SEAT_HYPER_VISION, SEAT_SYNTHETIC_VISION_WITHHELD, SEAT_SYNTHETIC_TEXT_EVERYWHERE,],
              },
            ],);
          },
        },),
      ],
    },),

    describe({
      name: statusBreakdown.name,
      children: [
        it({
          name: 'COUNTS EACH STATUS in the order first met, and prints nothing for no claims',
          fn: async () => {
            expect(statusBreakdown({
              claims: [
                { status: 'rejected', },
                { status: 'needs-human', },
                { status: 'rejected', },
              ],
            },),).toBe('rejected=2 needs-human=1',);
            expect(statusBreakdown({ claims: [], },),).toBe('',);
          },
        },),

        it({
          name: 'COUNTS A STATUS SPELLED AS A NAME EVERY OBJECT INHERITS like any other, where a plain-object '
            + 'record counted on from the inherited function and dropped a __proto__ status (ledger B77)',
          fn: async () => {
            expect(statusBreakdown({
              claims: [
                { status: '__proto__', },
                { status: 'constructor', },
                { status: 'rejected', },
                { status: 'constructor', },
              ],
            },),).toBe('__proto__=1 constructor=2 rejected=1',);
          },
        },),
      ],
    },),
  ],
},);
