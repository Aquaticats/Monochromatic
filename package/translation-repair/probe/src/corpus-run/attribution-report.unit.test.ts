/**
 Tests for the reader that turns recorded critic attribution into rates.

 Built alongside the writer on purpose. This pipeline's recurring failure is
 telemetry that is recorded and never read, and a data path with no reader is
 indistinguishable from one that was never built. These cases guard the two
 ways a reader can quietly lie about a population.

 The first is mixing eligibility with silence. An entry settled before
 attribution existed records no proposer for a claim its critics did raise, so
 averaging it in understates every critic at once, and "this critic raised
 nothing" becomes unreadable against "this entry could not have recorded it".

 The second is conflating self-repetition with agreement. One critic saying a
 thing twice and two critics saying it once produce the same claim, and whether
 a duplicate counts against precision turns on which of those it came from.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildAttributionReport,
  type AttributionEntry,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Critic that raises most of the claims.
 */
const TABBY = SEAT_SYNTHETIC_TEXT_EVERYWHERE;

/**
 Critic that is heard everywhere and rarely raises anything.
 */
const QUIET = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

/**
 Claim both critics can propose.
 */
const NAP_CLAIM = 'issue/nap';

/**
 Second claim identity.
 */
const PURR_CLAIM = 'issue/purr';

/**
 Entry carrying attribution, with one chunk both critics were asked.

 @param proposers - proposers of the nap claim

 @param issueClaimIds - claims the single accepted issue represents

 @returns Eligible entry

 @example
 ```ts
 const entry = eligibleEntry({ proposers: [{ modelId: TABBY, emissionCount: 1, },], },);
 ```
 */
function eligibleEntry(
  {
    proposers,
    issueClaimIds = [NAP_CLAIM,],
  }: {
    readonly proposers: readonly { readonly modelId: string; readonly emissionCount: number; }[];
    readonly issueClaimIds?: readonly string[];
  },
): AttributionEntry {
  return {
    id: 'Whiskers',
    sliceCritics: [
      {
        sliceIndex: 0,
        heardCriticIds: [QUIET, TABBY,],
        claimAttributions: [
          {
            claimId: NAP_CLAIM,
            proposers,
          },
        ],
      },
    ],
    issues: [
      {
        status: 'accepted',
        claimIds: issueClaimIds,
      },
    ],
  } as AttributionEntry;
}

await describe({
  name: buildAttributionReport.name,
  children: [
    it({
      name: 'EXCLUDES entries that carry no attribution rather than counting '
        + 'them as critics that raised nothing, because an entry settled before '
        + 'attribution existed records no proposer for claims its critics did '
        + 'raise, and averaging it in understates every critic at once',
      fn: async () => {
        /**
         One eligible entry beside one settled before attribution existed.
         */
        const report = buildAttributionReport({
          entries: [
            eligibleEntry({ proposers: [{ modelId: TABBY, emissionCount: 1, },], },),
            {
              id: 'Mittens',
              issues: [{ status: 'accepted', claimIds: [PURR_CLAIM,], },],
            } as AttributionEntry,
          ],
        },);

        expect(report.eligibleEntries,).toBe(1,);
        expect(report.ineligibleEntries,).toBe(1,);
        expect(report.chunks,).toBe(1,);
        // The ineligible entry's accepted issue must not appear anywhere.
        expect(
          report.soleProposerAccepted + report.multiProposerAccepted
            + report.unattributedAccepted,
        ).toBe(1,);
      },
    },),

    it({
      name: 'gives a critic that was HEARD and raised nothing a row with a '
        + 'real denominator, which is the whole reason the roster is recorded: '
        + 'without it a quiet critic and an absent one are the same zero',
      fn: async () => {
        /**
         Entry where only one of two heard critics raised anything.
         */
        const report = buildAttributionReport({
          entries: [eligibleEntry({
            proposers: [{ modelId: TABBY, emissionCount: 1, },],
          },),],
        },);

        /**
         Row for the critic that stayed silent.
         */
        const quiet = report.critics
          .find(function isQuiet(critic,) {
            return critic.modelId === QUIET;
          },);

        expect(quiet?.chunksHeard,).toBe(1,);
        expect(quiet?.claimsRaised,).toBe(0,);
        expect(quiet?.acceptedHits,).toBe(0,);
      },
    },),

    it({
      name: 'separates SELF-REPETITION from agreement: one critic emitting a '
        + 'claim twice counts as a sole proposer that repeated itself, never '
        + 'as two critics agreeing, which is exactly what a precision count asks of a '
        + 'duplicate',
      fn: async () => {
        /**
         Entry where one critic said the same thing twice.
         */
        const repeated = buildAttributionReport({
          entries: [eligibleEntry({
            proposers: [{ modelId: TABBY, emissionCount: 2, },],
          },),],
        },);

        expect(repeated.soleProposerAccepted,).toBe(1,);
        expect(repeated.multiProposerAccepted,).toBe(0,);
        expect(repeated.selfRepeatedAccepted,).toBe(1,);

        /**
         Entry where two critics agreed once each.
         */
        const agreed = buildAttributionReport({
          entries: [eligibleEntry({
            proposers: [
              { modelId: TABBY, emissionCount: 1, },
              { modelId: QUIET, emissionCount: 1, },
            ],
          },),],
        },);

        expect(agreed.soleProposerAccepted,).toBe(0,);
        expect(agreed.multiProposerAccepted,).toBe(1,);
        expect(agreed.selfRepeatedAccepted,).toBe(0,);
      },
    },),

    it({
      name: 'counts an accepted issue whose claim the index does not hold as '
        + 'UNATTRIBUTED rather than as zero support, since on an eligible '
        + 'entry that is a broken join and reporting it as a quiet critic '
        + 'would hide the defect',
      fn: async () => {
        /**
         Accepted issue pointing at a claim no attribution covers.
         */
        const report = buildAttributionReport({
          entries: [eligibleEntry({
            proposers: [{ modelId: TABBY, emissionCount: 1, },],
            issueClaimIds: [PURR_CLAIM,],
          },),],
        },);

        expect(report.unattributedAccepted,).toBe(1,);
        expect(report.soleProposerAccepted,).toBe(0,);
        expect(report.multiProposerAccepted,).toBe(0,);
      },
    },),

    it({
      name: 'counts only ACCEPTED issues toward hits, so a rejected claim '
        + 'never credits the critic that raised it',
      fn: async () => {
        /**
         Entry whose only issue was rejected.
         */
        const report = buildAttributionReport({
          entries: [{
            ...eligibleEntry({ proposers: [{ modelId: TABBY, emissionCount: 1, },], },),
            issues: [{ status: 'rejected', claimIds: [NAP_CLAIM,], },],
          } as AttributionEntry,],
        },);

        expect(report.soleProposerAccepted,).toBe(0,);
        expect(report.unattributedAccepted,).toBe(0,);
        expect(
          report.critics
            .find(function isTabby(critic,) {
              return critic.modelId === TABBY;
            },)?.acceptedHits,
        ).toBe(0,);
        // Raising it still counts; only the ACCEPTED credit is withheld.
        expect(
          report.critics
            .find(function isTabby(critic,) {
              return critic.modelId === TABBY;
            },)?.claimsRaised,
        ).toBe(1,);
      },
    },),

    it({
      name: 'MERGES proposers when two chunks of one entry carry the same '
        + 'claim id, rather than letting the later chunk overwrite the earlier '
        + 'one. The writer keeps chunks apart so neither inflates the other, '
        + 'and a reader that overwrites turns that care into silent deletion: '
        + 'two critics who each found the defect read as one',
      fn: async () => {
        /**
         Entry where two chunks produced an identical claim id, each from a
         different critic.
         */
        const report = buildAttributionReport({
          entries: [{
            id: 'Whiskers',
            sliceCritics: [
              {
                sliceIndex: 0,
                heardCriticIds: [QUIET, TABBY,],
                claimAttributions: [{
                  claimId: NAP_CLAIM,
                  proposers: [{ modelId: TABBY, emissionCount: 1, },],
                },],
              },
              {
                sliceIndex: 1,
                heardCriticIds: [QUIET, TABBY,],
                claimAttributions: [{
                  claimId: NAP_CLAIM,
                  proposers: [{ modelId: QUIET, emissionCount: 1, },],
                },],
              },
            ],
            issues: [{ status: 'accepted', claimIds: [NAP_CLAIM,], },],
          } as AttributionEntry,],
        },);

        expect(report.multiProposerAccepted,).toBe(1,);
        expect(report.soleProposerAccepted,).toBe(0,);
        // Both critics keep their hit. Overwriting drops one of these to zero.
        expect(
          report.critics
            .map(function toHits(critic,) {
              return critic.acceptedHits;
            },),
        ).toStrictEqual([1, 1,],);
      },
    },),

    it({
      name: 'separates the RAISED count from the EMISSION count numerically, '
        + 'not merely in the support categories. Nothing else asserts an '
        + 'emissions value at all, so an implementation that added '
        + 'emissionCount into claimsRaised, or reported emissions wrongly, '
        + 'would pass the whole suite while overstating how much a repeating '
        + 'critic contributed',
      fn: async () => {
        /**
         One critic, one distinct claim, emitted twice.
         */
        const report = buildAttributionReport({
          entries: [eligibleEntry({
            proposers: [{ modelId: TABBY, emissionCount: 2, },],
          },),],
        },);

        /**
         Row for the repeating critic.
         */
        const tabby = report.critics
          .find(function isTabby(critic,) {
            return critic.modelId === TABBY;
          },);

        // One distinct claim, two emissions of it. The difference between these
        // two numbers IS the self-repetition, so collapsing them loses it.
        expect(tabby?.claimsRaised,).toBe(1,);
        expect(tabby?.emissions,).toBe(2,);
      },
    },),

    it({
      name: 'holds a PARTIALLY joined accepted issue out of sole and multi '
        + 'rather than calling it sole-proposer. An issue naming one attributed '
        + 'claim and one the index does not hold has unknown support, since the '
        + 'missing member may have come from a critic nobody credited, and '
        + 'counting it as sole support would be a guess dressed as a count',
      fn: async () => {
        /**
         Accepted issue naming one known claim and one unknown one.
         */
        const report = buildAttributionReport({
          entries: [eligibleEntry({
            proposers: [{ modelId: TABBY, emissionCount: 1, },],
            issueClaimIds: [NAP_CLAIM, PURR_CLAIM,],
          },),],
        },);

        expect(report.partialJoinAccepted,).toBe(1,);
        expect(report.soleProposerAccepted,).toBe(0,);
        expect(report.multiProposerAccepted,).toBe(0,);
        expect(report.unattributedAccepted,).toBe(0,);
        // Held out of hits too: a hit count mixing sound and unsound joins is
        // exactly the quietly-wrong number this whole reader exists to avoid.
        expect(
          report.critics
            .find(function isTabby(critic,) {
              return critic.modelId === TABBY;
            },)?.acceptedHits,
        ).toBe(0,);
      },
    },),

    it({
      name: 'reports nothing rather than throwing when no entry carries '
        + 'attribution, which is what every runs directory looks like until a '
        + 'post-attribution pass settles its first entry',
      fn: async () => {
        /**
         Report over entries that all predate attribution.
         */
        const report = buildAttributionReport({
          entries: [{ id: 'Mittens', issues: [], } as AttributionEntry,],
        },);

        expect(report.eligibleEntries,).toBe(0,);
        expect(report.ineligibleEntries,).toBe(1,);
        expect(report.critics,).toHaveLength(0,);
      },
    },),
  ],
},);
