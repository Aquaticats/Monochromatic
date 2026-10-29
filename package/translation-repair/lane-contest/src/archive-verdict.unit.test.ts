/**
 Tests for the archive verdict the lane contest settles beside its choice.

 WHAT IS UNDER TEST is a gap once found: a contest that backs neither
 candidate used to say nothing at all about the text already published, even
 though that text is what ships when both candidates lose. The verdict is an
 ORTHOGONAL BALLOT FIELD, so these cases check that it settles by the same
 rule as the choice, and that its absence stays indistinguishable from an
 artifact written before the question existed.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ArchiveVerdict,
  describeContestSlice,
  LANE_CONTEST_QUORUM,
  type LaneContestBallot,
  type LaneContestOutcome,
  settleArchiveBallots,
  settleLaneContestBallots,
} from '../dist/final/node/index.mjs';

/**
 Ballot that backed neither candidate and gave the archive a verdict.

 @param archive - what this judge made of the archive

 @returns Ballot carrying that verdict

 @example
 ```ts
 const ballot = judged({ archive: 'flawed', },);
 ```
 */
function judged(
  { archive, }: { readonly archive: ArchiveVerdict; },
): LaneContestBallot {
  return {
    archive,
    choice: 'neither',
    unsupported: [],
    unsupportedRaw: [],
    dropped: [],
    droppedRaw: [],
    reason: 'the two candidates read alike to me',
  };
}

/**
 Ballot that backed neither candidate and said nothing about the archive.

 STANDS FOR TWO SITUATIONS AT ONCE, deliberately: a model that ignored the
 schema field, and a ballot stored before the field existed. The settling
 rule must not be able to tell them apart.

 @returns Ballot with no archive answer

 @example
 ```ts
 const ballot = silent();
 ```
 */
function silent(): LaneContestBallot {
  return {
    choice: 'neither',
    unsupported: [],
    unsupportedRaw: [],
    dropped: [],
    droppedRaw: [],
    reason: 'the two candidates read alike to me',
  };
}

/**
 Ballot that backed one lane, so the contest has a winner.

 @param archive - what this judge made of the archive

 @returns Ballot naming the repair lane

 @example
 ```ts
 const ballot = backsRepair({ archive: 'flawed', },);
 ```
 */
function backsRepair(
  { archive, }: { readonly archive: ArchiveVerdict; },
): LaneContestBallot {
  return {
    archive,
    choice: 'repair',
    unsupported: [],
    unsupportedRaw: [],
    dropped: [],
    droppedRaw: [],
    reason: 'the repair candidate keeps the whisker count right',
  };
}

/**
 Wraps ballots as the outcome the record builder takes.

 @param ballots - ballots this slice heard

 @returns Outcome carrying them, settled by the stage`s own rule

 @example
 ```ts
 const outcome = outcomeOf({ ballots, },);
 ```
 */
function outcomeOf(
  { ballots, }: { readonly ballots: readonly LaneContestBallot[]; },
): LaneContestOutcome {
  return {
    choice: settleLaneContestBallots({ ballots, },),
    ballots,
    usable: ballots.length,
    findings: [],
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: settleArchiveBallots.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ENDORSES ON THE SAME BAR THE CHOICE USES, two voices and a strict lead, rather than on '
            + 'a bar of its own that every stored verdict would then be recomputed against',
          fn: async function endorsesOnQuorum() {
            expect(settleArchiveBallots({
              ballots: [
                judged({ archive: 'publishable', },),
                judged({ archive: 'publishable', },),
              ],
            },),).toBe('endorsed',);
            expect(LANE_CONTEST_QUORUM,).toBe(2,);
          },
        },),

        it({
          name: 'DECLINES when the voices go the other way, because an archive most of the roster found '
            + 'fault with is the finding this whole field exists to record',
          fn: async function declinesOnQuorum() {
            expect(settleArchiveBallots({
              ballots: [
                judged({ archive: 'flawed', },),
                judged({ archive: 'flawed', },),
              ],
            },),).toBe('declined',);
          },
        },),

        it({
          name: 'LEAVES A TIE UNJUDGED, since a strict lead is what the choice rule demands and shipping '
            + 'an endorsement off an even split would be picking by which side was counted first',
          fn: async function leavesTiesUnjudged() {
            expect(settleArchiveBallots({
              ballots: [
                judged({ archive: 'publishable', },),
                judged({ archive: 'publishable', },),
                judged({ archive: 'flawed', },),
                judged({ archive: 'flawed', },),
              ],
            },),).toBe('unjudged',);
          },
        },),

        it({
          name: 'LEAVES A LONE VOICE UNJUDGED, so one judge`s opinion of the archive is not recorded as '
            + 'the roster`s, exactly as one voice cannot win the contest',
          fn: async function leavesOneVoiceUnjudged() {
            expect(settleArchiveBallots({ ballots: [ judged({ archive: 'publishable', },), ], },),)
              .toBe('unjudged',);
          },
        },),

        it({
          name: 'READS AN OMITTED FIELD AS A VOICE THAT DID NOT SPEAK, not as a decline, because a model '
            + 'that ignored the field and a ballot stored before the field existed must settle alike',
          fn: async function readsSilenceAsSilence() {
            expect(settleArchiveBallots({
              ballots: [
                silent(),
                silent(),
                silent(),
              ],
            },),).toBe('unjudged',);
          },
        },),
      ],
    },),

    describe({
      name: describeContestSlice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RECORDS THE VERDICT where the roster backed no candidate, which is the slice whose '
            + 'shipped text is the archive and about which the record used to say nothing',
          fn: async function recordsTheVerdict() {
            expect(describeContestSlice({
              sliceIndex: 0,
              outcome: outcomeOf({
                ballots: [
                  judged({ archive: 'flawed', },),
                  judged({ archive: 'flawed', },),
                ],
              },),
            },).verdict,).toEqual({
              kind: 'settled-neither',
              archive: 'declined',
            },);
          },
        },),

        it({
          name: 'OMITS THE KEY ENTIRELY when the archive went unjudged, rather than writing "unjudged". '
            + 'This is what keeps artifacts settled before the question existed byte-identical, and it '
            + 'is why the exact-keys guard needs no exception for them',
          fn: async function omitsTheUnjudgedKey() {
            /**
             Verdict for a slice whose judges all ignored the archive question.
             */
            const { verdict, } = describeContestSlice({
              sliceIndex: 0,
              outcome: outcomeOf({
                ballots: [
                  silent(),
                  silent(),
                ],
              },),
            },);

            expect(verdict,).toEqual({ kind: 'settled-neither', },);
            expect(Object.hasOwn(verdict, 'archive',),).toBe(false,);
          },
        },),

        it({
          name: 'CARRIES NO ARCHIVE KEY ON A WON SLICE even when every ballot answered the question, '
            + 'because the archive verdict decides nothing where a candidate already beat it, and a '
            + 'field recorded there would be read as a reason the winner won',
          fn: async function leavesWonSlicesAlone() {
            /**
             Verdict for a slice the repair lane won outright.
             */
            const { verdict, } = describeContestSlice({
              sliceIndex: 0,
              outcome: outcomeOf({
                ballots: [
                  backsRepair({ archive: 'flawed', },),
                  backsRepair({ archive: 'flawed', },),
                ],
              },),
            },);

            expect(verdict,).toEqual({
              kind: 'lane-won',
              lane: 'repair',
            },);
            expect(Object.hasOwn(verdict, 'archive',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
