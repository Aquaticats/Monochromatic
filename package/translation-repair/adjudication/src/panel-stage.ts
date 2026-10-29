import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  AdjudicatedIssue,
  ClaimCluster,
  PanelBallot,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type AdjudicationPromptPlan,
  buildAdjudicationMessages,
} from './adjudicate-prompt.ts';
import {
  ADJUDICATION_RESPONSE_FORMAT,
  usablePanelBallotFor,
  resolvePanelBallot,
} from './adjudicate-wire.ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  gatherStageVoices,
  mapOverlapped,
  type OverlappedRow,
  silentStageLabel,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import { logBallotIrregularities, } from './ballot-irregularity-log.ts';
import { tallyVotes, } from './tally-votes.ts';

//region Cluster-local panel

/**
 Everything the panel produced for one chunk.

 @example
 ```ts
 const { issues } = await runPanelStage(input);
 ```
 */
export type PanelStageResult = {
  /**
   Adjudicated issues in cluster document order.
   */
  readonly issues: readonly AdjudicatedIssue[];

  /**
   Distinct panelists heard in at least one packet, for reporting only.
   Each claim's reading retains its own ballots and configured electorate.
   */
  readonly heardPanelists: number;

  /**
   Packet-located ballot irregularities in scorecard-stable wording.
   */
  readonly findings: readonly string[];

  /**
   One label for each packet the panel heard fewer voices than its quorum on, naming the cluster, such as
   `panel 0/6 (cluster/abc)`; empty when every packet reached its quorum.

   DATA BESIDE THE LOCATED FINDING, because that finding begins `panel-packet (cluster): ` and so no reader of
   where a finding begins finds the silence: XingZ6018 slice 85 heard nobody on five packets, its 12 issues all stood
   at needs-human, and the settlement was cached as examined.
   */
  readonly silentStages: readonly string[];
};

/**
 Immutable review unit materialized before the panel makes any provider call.

 @example
 ```ts
 const packet: PanelPacket = { cluster, plan };
 ```
 */
type PanelPacket = {
  /**
   Original merge proposal with every member retained.
   */
  readonly cluster: ClaimCluster;

  /**
   Prompt and local wire-number maps for this exact cluster.
   */
  readonly plan: AdjudicationPromptPlan;
};

/**
 Independent tally and provenance from one packet.

 @example
 ```ts
 const ids = packetResult.heardIds;
 ```
 */
type PanelPacketResult = {
  /**
   Decisions using only this packet's ballots.
   */
  readonly issues: readonly AdjudicatedIssue[];

  /**
   Actual responding identities, never a substituted quorum basis.
   */
  readonly heardIds: readonly RosterModelId[];

  /**
   Packet-specific participation and wire findings.
   */
  readonly findings: readonly string[];

  /**
   The label of this packet when its gather heard fewer voices than quorum, else empty.
   */
  readonly silentStages: readonly string[];
};

/**
 One issue's decision as the run log says it: status, severity, and the
 weight behind every vote state on each member claim.

 LOGGED BECAUSE THE ARTIFACT ALONE CARRIED IT (ledger E5). The stage wrote
 only its packet count, so an issue the editor then served had nothing in the
 log saying the panel accepted it, or how narrowly. The ballots stay in the
 artifact's `readings`; this is the line a reader of the log needs.

 @param issue - issue as the tally decided it

 @returns One line naming the issue, its fate and its weights

 @throws {@link Error} when a member claim has no tally, which the tally never
 leaves out of an issue it builds

 @example
 ```ts
 packetLogger.info(describePanelDecision(issue,),);
 ```
 */
function describePanelDecision(issue: AdjudicatedIssue,): string {
  /**
   Weights per member claim, in claim order.
   */
  const weights = issue.claims
    .map(function weightsOf({ claimId, },): string {
      /**
       Weights this claim drew, which `tallyVotes` keys for every member claim
       of an issue it builds.
       */
      const tally = issue.tallies[claimId];
      // NOT ZEROED AND NOT SKIPPED: zeros would read as a tally in which no
      // ballot carried weight, which is a different fact.
      if (tally === undefined)
        throw new Error(`unreachable: ${claimId} is a member claim of ${issue.issueId} and has no tally, though the tally of an issue keys every member claim`,);
      /**
       Weight behind each vote state.
       */
      const {
        supported,
        unsupported,
        ambiguous,
        sourceDefect,
        abstain,
      } = tally;
      return `${claimId}: supported ${String(supported,)}, unsupported ${String(unsupported,)}, ambiguous ${
        String(ambiguous,)
      }, source-defect ${String(sourceDefect,)}, abstain ${String(abstain,)}`;
    },);
  return `${issue.issueId} ${issue.status} at ${issue.severity}; ${weights.join('; ',)}`;
}

/**
 Each ballot behind an issue's decision as the run log says it: claim,
 panelist, vote, and the reason it gave before voting (owner, 2026-09-27,
 "Reason before vote").

 @param issue - issue as the tally decided it

 @returns One line per stored ballot, in claim then ballot order

 @throws {@link Error} when the issue or one of its member claims has no stored
 reading, which the tally never leaves out of an issue it builds

 @example
 ```ts
 for (const line of describePanelReasons(issue,)) packetLogger.info(line,);
 ```
 */
function describePanelReasons(issue: AdjudicatedIssue,): readonly string[] {
  /**
   Stored readings keyed by claim id, which `tallyVotes` writes for every
   issue it builds.
   */
  const { readings, } = issue;
  if (readings === undefined)
    throw new Error(`unreachable: ${issue.issueId} carries no readings, though the tally stores the ballots of every issue it builds`,);
  return issue.claims
    .flatMap(function linesOf({ claimId, },): readonly string[] {
      /**
       Reading stored for this claim, one per member claim.
       */
      const reading = readings[claimId];
      if (reading === undefined)
        throw new Error(`unreachable: ${claimId} is a member claim of ${issue.issueId} and has no reading, though the tally stores one for every member claim`,);
      /**
       Ballots stored for this claim.
       */
      const { ballots, } = reading;
      return ballots.map(function lineOf({
        panelistId,
        vote,
        reason,
      },): string {
        return `${issue.issueId} ${claimId} ${panelistId} ${vote}: ${reason ?? '(gave no reason)'}`;
      },);
    },);
}

/**
 Names a packet on the lines the overlapped map writes about a failure it
 leaves unthrown, by the cluster it reviews, which reveals no
 proposer.

 @param item - packet, named by its cluster

 @returns Cluster by its identity

 @example
 ```ts
 packetClusterName({ item: packet, },); // 'cluster C2'
 ```
 */
function packetClusterName(
  { item, }: { readonly item: { readonly cluster: { readonly clusterId: string; }; }; },
): string {
  return `cluster ${item.cluster
    .clusterId}`;
}

/**
 Reviews precomputed clusters independently within one fixed panel stage.
 Ballots cannot create another packet or change its membership. Sequential
 packet execution bounds provider fan-out; the existing model window and
 recovery policy remain inside each packet.

 @param client - injected provider client

 @param panelModelIds - unchanged configured electorate for every packet

 @param sourceText - current source slice defining coverage

 @param targetText - current translation being reviewed

 @param clusters - original merge proposals, fixed before any panel call

 @param neighbouringSourceText - nearby factual evidence for current claims

 @param neighbouringIncumbentText - nearby archive placement context

 @param documentSourceText - optional complete same-entry source evidence, not extra coverage

 @param referenceContext - what the original's cited pages say (class
 thirty-five), evidence for addition claims

 @param identityContext - declared names and handles, which the panel judges
 a claim against a declared name by (ledger S4)

 @param signal - caller cancellation

 @param perCallTimeoutMs - deadline per exchange

 @param l - parent logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Ordered issue decisions with independent per-claim readings

 @example
 ```ts
 const panel = await runPanelStage(input);
 ```
 */
export async function runPanelStage(
  {
    client,
    panelModelIds,
    sourceText,
    targetText,
    clusters,
    neighbouringIncumbentText,
    neighbouringSourceText,
    documentSourceText,
    referenceContext,
    identityContext,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly panelModelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly targetText: string;
    readonly clusters: readonly ClaimCluster[];
    readonly neighbouringIncumbentText?: string;
    readonly neighbouringSourceText?: string;
    readonly documentSourceText?: string;
    readonly referenceContext?: string;
    readonly identityContext?: string;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<PanelStageResult> {
  signal.throwIfAborted();
  /**
   Stage boundary for packet diagnostics.
   */
  const pl = tagged({
    l,
    tag: runPanelStage.name,
  });
  /**
   All work units exist before the first asynchronous review starts.
   */
  const packets = clusters.map(function planPacket(cluster,): PanelPacket {
    return {
      cluster,
      plan: buildAdjudicationMessages({
        sourceText,
        targetText,
        clusters: [cluster,],
        ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
        ...((neighbouringIncumbentText === undefined) ? {} : { neighbouringIncumbentText, }),
        ...((documentSourceText === undefined) ? {} : { documentSourceText, }),
        ...((referenceContext === undefined) ? {} : { referenceContext, }),
        ...((identityContext === undefined) ? {} : { identityContext, }),
      },),
    };
  },);
  pl.info(`panel stage: ${String(packets.length,)} preplanned cluster packets`);
  /**
   Each tally observes its own packet, never the union of responding seats.
   */
  const outcomes = await mapOverlapped({
    items: packets,
    overlap: 1,
    signal,
    l: pl,
    nameOf: packetClusterName,
    oneItem: async function reviewPacket(
      { item: packet, }: OverlappedRow<PanelPacket>,
    ): Promise<PanelPacketResult> {
      /**
       Function and cluster tags keep repeated panel diagnostics locatable.
       */
      const functionLogger = tagged({
        l: pl,
        tag: reviewPacket.name,
      });
      /**
       Stable cluster identity reveals no proposer identity.
       */
      const packetLogger = tagged({
        l: functionLogger,
        tag: packet.cluster
          .clusterId,
      });
      /**
       Original packet and its local numeric wire mapping.
       */
      const {
        cluster,
        plan,
      } = packet;
      /**
       Existing window and recovery operate within this one preplanned packet.
       */
      const gather = await gatherStageVoices({
        client,
        modelIds: panelModelIds,
        messages: plan.messages,
        signal,
        exchangeTimeoutMs: perCallTimeoutMs,
        responseFormat: ADJUDICATION_RESPONSE_FORMAT,
        validate: usablePanelBallotFor({
          claimCount: plan.claimIds
            .length,
        },),
        stage: 'panel',
        l: packetLogger,
        graceMs: stageSettings
          .grace
          .roundMs,
      },);
      /**
       Wire indices resolve only against this packet's actual members.
       */
      const ballots: Record<string, PanelBallot> = Object.fromEntries(
        gather.voices
          .map(function ballot(voice,): readonly [
            string,
            PanelBallot
          ] {
            return [
              voice.modelId,
              resolvePanelBallot({
                wire: voice.value,
                claimIds: plan.claimIds,
                clusterIds: plan.clusterIds,
              },),
            ];
          },),
      );
      logBallotIrregularities({
        ballots,
        stage: 'panel',
        l: packetLogger,
      },);
      /**
       Configured electorate remains unchanged even if another packet hears different seats.
       */
      const {
        issues,
        findings: tallyFindings,
      } = tallyVotes({
        clusters: [cluster,],
        ballots,
        configuredPanelists: panelModelIds.length,
      },);
      for (const issue of issues) {
        packetLogger.info(describePanelDecision(issue,),);
        for (const line of describePanelReasons(issue,))
          packetLogger.info(line,);
      }
      return {
        issues,
        heardIds: gather.voices
          .map(function identity(voice,): RosterModelId {
            return voice.modelId;
          },),
        findings: [
          ...tallyFindings,
          ...gather.findings,
          ...Object.values(ballots,)
            .flatMap(function irregularities(ballot,): readonly string[] {
              return ballot.findings;
            },),
        ].map(function locate(finding,): string {
          return `panel-packet (${cluster.clusterId}): ${finding}`;
        },),
        silentStages: gather.quorumMet
          ? []
          : [
            silentStageLabel({
              stage: 'panel',
              heard: gather.voices
                .length,
              asked: gather.asked
                .size,
              where: cluster.clusterId,
            },),
          ],
      };
    },
  },);
  /**
   Distinct reporting identities do not participate in any claim's tally.
   */
  const heard = new Set(outcomes.flatMap(function identities(outcome,): readonly RosterModelId[] {
    return outcome.heardIds;
  },),);
  return {
    issues: outcomes.flatMap(function decisions(outcome,): readonly AdjudicatedIssue[] {
      return outcome.issues;
    },),
    heardPanelists: heard.size,
    findings: outcomes.flatMap(function diagnostics(outcome,): readonly string[] {
      return outcome.findings;
    },),
    silentStages: outcomes.flatMap(function silence(outcome,): readonly string[] {
      return outcome.silentStages;
    },),
  };
}

//endregion Cluster-local panel
