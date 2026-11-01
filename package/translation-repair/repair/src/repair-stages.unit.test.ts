/**
 Tests for the adjudication panel stage.

 `runPanelStage` had no test. It decides which critic claims become accepted
 issues, so it sets the numerator of the precision the milestone is graded on:
 an accepted issue a human later calls wrong is a false positive, and one the
 panel wrongly rejected never reaches the sheet to be counted either way.

 The vote arithmetic lives in `tallyVotes`, which has its own suite. What is
 untested here is the wiring: that a heard panelist becomes exactly one
 ballot, that a lost voice shrinks the electorate rather than passing as an
 abstention nobody notices, and that a panelist voting on a claim number it
 was never shown reaches the findings.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import type {
  AggregatedClaim,
  ClaimCluster,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { runPanelStage, } from '@monochromatic-dev/translation-repair-adjudication';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the stages under test.
 */
const l = tagged({ tag: 'panel-stage-test', },);

/**
 Original the panel judges against.
 */
const SOURCE_TEXT = '猫猫在窗台上睡觉。太阳移动时她会醒来。';

/**
 Translation under adjudication.
 */
const TARGET_TEXT = 'The cat sleeps on the windowsill.';

/**
 Panel roster large enough for a majority to be visible.
 */
const PANELISTS = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
] as const;

/**
 Builds one aggregated claim.

 @param suffix - id suffix, so claims differ

 @returns Claim the panel votes on

 @example
 ```ts
 const claim = catClaim({ suffix: 'waking', },);
 ```
 */
function catClaim({ suffix, }: { readonly suffix: string; },): AggregatedClaim {
  return {
    claimId: `issue/${suffix}`,
    claim: {
      category: 'accuracy/omission',
      severity: 'major',
      summary: `The ${suffix} sentence about the cat is missing.`,
      spans: [
        {
          side: 'target',
          nodeId: 'block/0',
          nodeHash: hashContent({ content: TARGET_TEXT, },),
          startOffset: TARGET_TEXT.length,
          endOffset: TARGET_TEXT.length,
          quotedText: '',
        },
      ],
    },
  };
}

/**
 Single-member cluster around one claim.

 @param suffix - id suffix of the member claim

 @returns Cluster the panel is shown

 @example
 ```ts
 const cluster = soloCluster({ suffix: 'waking', },);
 ```
 */
function soloCluster({ suffix, }: { readonly suffix: string; },): ClaimCluster {
  return {
    clusterId: `cluster/${suffix}`,
    position: 10,
    members: [catClaim({ suffix, },),],
  };
}

/**
 Client answering each panelist with a scripted ballot, or losing its voice.

 @param ballotFor - ballot per model; returning undefined loses that voice

 @returns Client honoring that script

 @example
 ```ts
 const client = panelClient({ ballotFor: () => ({ verdicts: [], }), },);
 ```
 */
function panelClient(
  { ballotFor, }: { readonly ballotFor: (modelId: string,) => unknown; },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Scripted ballot for the answering model.
       */
      const scripted = ballotFor(request.modelId,);
      if (scripted === undefined) {
        return {
          kind: 'refusal-shaped',
          rawText: '',
          marker: 'scripted voice loss',
        };
      }
      // A BALLOT THE STAGE'S GUARD REJECTS comes back as the real clients
      // return it, a schema mismatch, so the gather reads it as unreadable.
      if (!request.validate(scripted,)) {
        return {
          kind: 'schema-mismatch',
          rawText: JSON.stringify(scripted,),
          reason: 'caller-guard-rejected',
          detail: 'content parsed as JSON but failed the caller schema guard',
        };
      }
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 Runs the panel stage against a scripted client.

 @param client - scripted panel client

 @param clusters - clusters the panel is shown

 @returns Stage result

 @example
 ```ts
 const result = await runStage({ client, clusters, },);
 ```
 */
async function runStage(
  {
    client,
    clusters,
  }: {
    readonly client: ModelClient;
    readonly clusters: readonly ClaimCluster[];
  },
) {
  return await runPanelStage({
    client,
    panelModelIds: PANELISTS,
    sourceText: SOURCE_TEXT,
    targetText: TARGET_TEXT,
    clusters,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: runPanelStage.name,
  children: [
    it({
      name: 'accepts a claim a majority supported, which is how a critic claim '
        + 'becomes an issue the editors are allowed to act on',
      fn: async () => {
        /**
         Stage where every panelist supported the single claim.
         */
        const result = await runStage({
          client: panelClient({
            ballotFor: () => ({
              verdicts: [
                {
                  claim: 1,
                  vote: 'supported',
                  severity: 'major',
                },
              ],
            }),
          },),
          clusters: [soloCluster({ suffix: 'waking', },),],
        },);

        expect(result.heardPanelists,).toBe(PANELISTS.length,);
        expect(result.issues.length,).toBe(1,);
        expect(result.issues[0]?.status,).toBe('accepted',);
      },
    },),

    it({
      name: 'REJECTS a claim a majority called unsupported, which is the whole '
        + 'reason the panel exists: an unfiltered critic claim reaching the '
        + 'sheet is a false positive against the precision bar',
      fn: async () => {
        /**
         Stage where every panelist rejected the claim.
         */
        const result = await runStage({
          client: panelClient({
            ballotFor: () => ({
              verdicts: [
                {
                  claim: 1,
                  vote: 'unsupported',
                },
              ],
            }),
          },),
          clusters: [soloCluster({ suffix: 'waking', },),],
        },);

        expect(result.issues[0]?.status,).not.toBe('accepted',);
      },
    },),

    it({
      name: 'COUNTS ONLY HEARD PANELISTS, so a lost voice shrinks the '
        + 'electorate rather than passing as an abstention nobody notices: a '
        + 'stage reporting a full panel while two voices were lost would let '
        + 'one model decide what reaches the editors',
      fn: async () => {
        /**
         Panelist that answers; the others lose their voices.
         */
        const answering: ReadonlySet<string> = new Set([PANELISTS[0],],);

        /**
         Stage where only one of three panelists replied.
         */
        const result = await runStage({
          client: panelClient({
            ballotFor: (modelId,) =>
              answering.has(modelId,)
                ? {
                  verdicts: [
                    {
                      claim: 1,
                      vote: 'supported',
                      severity: 'major',
                    },
                  ],
                }
                : undefined,
          },),
          clusters: [soloCluster({ suffix: 'waking', },),],
        },);

        expect(result.heardPanelists,).toBe(1,);
        expect(result.findings.length,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'returns an issue for EVERY cluster it was shown, including one no '
        + 'panelist voted on, so a claim dropped from every ballot is recorded '
        + 'as undecided rather than disappearing before anyone can see it was '
        + 'never judged',
      fn: async () => {
        /**
         Stage over two clusters where ballots mention only the first.
         */
        const result = await runStage({
          client: panelClient({
            ballotFor: () => ({
              verdicts: [
                {
                  claim: 1,
                  vote: 'supported',
                  severity: 'major',
                },
              ],
            }),
          },),
          clusters: [
            soloCluster({ suffix: 'waking', },),
            soloCluster({ suffix: 'sunlight', },),
          ],
        },);

        expect(result.issues.length,).toBe(2,);
      },
    },),

    it({
      name: 'carries BALLOT irregularities into the findings, so a panelist '
        + 'voting on a claim number it was never shown reaches the scorecard '
        + 'rather than being dropped between the fan-out and the tally',
      fn: async () => {
        /**
         Stage where every panelist voted on the claim and also numbered one
         off the sheet.
         */
        const result = await runStage({
          client: panelClient({
            ballotFor: () => ({
              verdicts: [
                {
                  claim: 1,
                  vote: 'supported',
                  severity: 'major',
                },
                {
                  claim: 9,
                  vote: 'supported',
                  severity: 'major',
                },
              ],
            }),
          },),
          clusters: [soloCluster({ suffix: 'waking', },),],
        },);

        expect(result.heardPanelists,).toBe(PANELISTS.length,);
        expect(result.findings.some(function namesOffSheet(finding,): boolean {
          return finding.includes('verdict-index-out-of-range (9)',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'DOES NOT COUNT A BALLOT WITH NO USABLE VERDICT as a heard voice (ledger L8): an empty ballot, '
        + 'one voting only on claims never shown and one whose only vote is no vote at all each leave the '
        + 'round short, so the recovery round re-asks rather than closing on a silent seat',
      fn: async () => {
        /**
         Unusable ballot by panelist: empty, off the sheet, and an unknown vote.
         */
        const unusable: Readonly<Record<string, unknown>> = {
          [PANELISTS[0]]: { verdicts: [], },
          [PANELISTS[1]]: {
            verdicts: [{
              claim: 9,
              vote: 'supported',
            },],
          },
          [PANELISTS[2]]: {
            verdicts: [{
              claim: 1,
              vote: 'minor',
            },],
          },
        };

        /**
         Stage where no panelist cast a usable verdict.
         */
        const result = await runStage({
          client: panelClient({
            ballotFor: (modelId,) => unusable[modelId],
          },),
          clusters: [soloCluster({ suffix: 'waking', },),],
        },);

        expect(result.heardPanelists,).toBe(0,);
        expect(result.issues[0]?.status,).toBe('needs-human',);
      },
    },),

    it({
      name: 'returns no issues when there was nothing to adjudicate, so a '
        + 'slice whose critics found nothing costs no panel decision',
      fn: async () => {
        /**
         Stage over an empty cluster list.
         */
        const result = await runStage({
          client: panelClient({ ballotFor: () => ({ verdicts: [], }), },),
          clusters: [],
        },);

        expect(result.issues,).toStrictEqual([],);
      },
    },),
  ],
},);
