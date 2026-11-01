/**
 Tests for retirement log wording.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  formatAbstentions,
  formatFailedProvider,
  formatLiveModelWarning,
  formatPlanningSummary,
  formatRefreshFailure,
  formatRetirementLine,
  formatSkippedProvider,
  type AbstentionCounts,
} from '../dist/final/node/index.mjs';

//region Fixtures

/** Reasons a tally fixture overrides, every one optional. */
type TallyOverrides = {
  readonly keeperAmbiguity?: number;
  readonly unorderedPair?: number;
  readonly loserNewerThanKeeper?: number;
  readonly versionlessProtected?: number;
  readonly duplicateIdentity?: number;
};

/**
 Build an abstention tally with only the given reasons set.

 @param overrides - reasons to set, all others zero

 @returns tally with every reason present
 */
function tally(overrides: TallyOverrides,): AbstentionCounts {
  return {
    keeperAmbiguity: 0,
    unorderedPair: 0,
    loserNewerThanKeeper: 0,
    versionlessProtected: 0,
    duplicateIdentity: 0,
    ...overrides,
  };
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: formatAbstentions.name,
      children: [
        it({
          name: 'says nothing was declined when every reason is zero',
          fn: async function runEmptyTally() {
            expect(formatAbstentions({
              abstentions: tally({},),
            },),).toBe('declined nothing',);
          },
        },),
        it({
          name: 'lists non-zero reasons in a fixed order',
          fn: async function runOrderedTally() {
            expect(formatAbstentions({
              abstentions: tally({
                versionlessProtected: 3,
                keeperAmbiguity: 1,
                unorderedPair: 2,
              },),
            },),).toBe('declined keeperAmbiguity=1 unorderedPair=2 versionlessProtected=3',);
          },
        },),
        it({
          name: 'omits reasons that did not occur',
          fn: async function runPartialTally() {
            expect(formatAbstentions({
              abstentions: tally({
                duplicateIdentity: 1,
              },),
            },),).toBe('declined duplicateIdentity=1',);
          },
        },),
      ],
    },),
    describe({
      name: formatPlanningSummary.name,
      children: [
        it({
          name: 'reports counts of retired models and re-registered providers',
          fn: async function runSummary() {
            expect(formatPlanningSummary({
              counts: {
                planCount: 1,
                retirementCount: 1,
                abstentions: tally({
                  unorderedPair: 1,
                },),
              },
            },),).toBe('retired 1 model across 1 provider; declined unorderedPair=1',);
          },
        },),
        it({
          name: 'pluralizes a pass that retired several models',
          fn: async function runPluralSummary() {
            expect(formatPlanningSummary({
              counts: {
                planCount: 2,
                retirementCount: 6,
                abstentions: tally({},),
              },
            },),).toBe('retired 6 models across 2 providers; declined nothing',);
          },
        },),
        it({
          name: 'reports an empty pass',
          fn: async function runEmptySummary() {
            expect(formatPlanningSummary({
              counts: {
                planCount: 0,
                retirementCount: 0,
                abstentions: tally({},),
              },
            },),).toBe('retired 0 models across 0 providers; declined nothing',);
          },
        },),
      ],
    },),
    describe({
      name: formatRetirementLine.name,
      children: [
        it({
          name: 'names the provider, the retired id, and the keeper',
          fn: async function runRetirementLine() {
            /**
             Line for one measured retirement.
             */
            const line = formatRetirementLine({
              retirement: {
                provider: 'hyper',
                api: 'openai-completions',
                retiredId: 'glm-5.2',
                keeperId: 'glm-5.3',
              },
            },);
            expect(line,).toBe('retired hyper/glm-5.2 in favor of glm-5.3',);
          },
        },),
      ],
    },),
    describe({
      name: formatLiveModelWarning.name,
      children: [
        it({
          name: 'names the live model, the successor, and the inaction',
          fn: async function runLiveWarning() {
            expect(formatLiveModelWarning({
              retirement: {
                provider: 'opencode-go',
                api: 'openai-completions',
                retiredId: 'glm-5.2',
                keeperId: 'glm-5.3',
              },
            },),).toBe(
              'session model opencode-go/glm-5.2 is retired in favor of glm-5.3; leaving the session on it',
            );
          },
        },),
      ],
    },),
    describe({
      name: formatSkippedProvider.name,
      children: [
        it({
          name: 'names the provider, the missing field, and the consequence',
          fn: async function runSkippedLine() {
            expect(formatSkippedProvider({
              skipped: {
                provider: 'azure-openai-responses',
                reason: 'model gpt-4.1 carries no baseUrl',
              },
            },),).toBe(
              'skipped azure-openai-responses: model gpt-4.1 carries no baseUrl; its retired entries stay listed',
            );
          },
        },),
      ],
    },),
    describe({
      name: formatFailedProvider.name,
      children: [
        it({
          name: 'names the provider, the caught failure, and the session consequence',
          fn: async function runFailedLine() {
            expect(formatFailedProvider({
              failed: {
                provider: 'azure-openai-responses',
                reason: '"baseUrl" is required',
              },
            },),).toBe(
              'pi refused to re-register azure-openai-responses: "baseUrl" is required; its retired entries stay listed for this session',
            );
          },
        },),
      ],
    },),
    describe({
      name: formatRefreshFailure.name,
      children: [
        it({
          name: 'names the caught failure and the fallback',
          fn: async function runRefreshFailureLine() {
            expect(formatRefreshFailure({
              reason: 'models.json unreadable',
            },),).toBe(
              'catalog refresh failed, filtering the unrefreshed catalog instead: models.json unreadable',
            );
          },
        },),
      ],
    },),
  ],
},);
