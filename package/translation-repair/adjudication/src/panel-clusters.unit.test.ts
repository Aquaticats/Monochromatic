/**
 Cluster-local packets retain every claim, merge ballot and independent quorum basis.

 @module
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import type {
  ClaimCluster,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  buildAdjudicationMessages,
  runPanelStage,
} from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/** Fixture electorate remains unchanged across packets. */
const MODELS: readonly RosterModelId[] = [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD, SEAT_SYNTHETIC_TEXT_EVERYWHERE, SEAT_HYPER_VISION, SEAT_HYPER_TEXT_BEDROCK,];
/** Distinct clusters include a merge proposal whose members must stay together. */
const CLUSTERS: readonly ClaimCluster[] = [
  { clusterId: 'first', position: 0, members: [
    { claimId: 'claim-first', claim: { category: 'accuracy/mistranslation', severity: 'minor', summary: 'FIRST cat claim', spans: [], }, },
  ], },
  { clusterId: 'second', position: 1, members: [
    { claimId: 'claim-second', claim: { category: 'accuracy/mistranslation', severity: 'minor', summary: 'SECOND cat claim', spans: [], }, },
    { claimId: 'claim-third', claim: { category: 'accuracy/mistranslation', severity: 'minor', summary: 'THIRD cat claim', spans: [], }, },
  ], },
];
/** Captured packet plus actual responding identity. */
type Capture = { readonly prompt: string; readonly modelId: string; readonly claims: readonly string[]; };
/**
 Supplies deterministic votes over whichever claims the real packet actually contains.
 @param input - optional per-packet availability restriction
 @returns Capturing client and evidence
 @example
 ```ts
 const fixture = panelFixture({});
 ```
 */
function panelFixture(input: { readonly deafTo?: string; readonly disjoint?: boolean; readonly rejectThird?: boolean; } = {}): { client: ModelClient; captures: Capture[]; } {
  const captures: Capture[] = [];
  const client: ModelClient = {
    chatText: async () => { throw new Error('Unexpected text request',); },
    quotas: async () => { throw new Error('Unexpected quota request',); },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>): Promise<ChatJsonOutcome<ValueT>> => {
      const prompt = JSON.stringify(request.messages.filter(message => message.role === 'user'));
      const present = CLUSTERS.flatMap(cluster => cluster.members).filter(member => prompt.includes(member.claim.summary));
      captures.push({ prompt, modelId: request.modelId, claims: present.map(member => member.claimId), });
      if ((input.deafTo !== undefined) && present.some(member => member.claimId === input.deafTo))
        return { kind: 'schema-mismatch', rawText: '{}', detail: 'Fixture panel deaf to this packet', };
      if (input.disjoint === true) {
        const index = MODELS.indexOf(request.modelId);
        const first = present.some(member => member.claimId === 'claim-first');
        if ((first && (index >= (MODELS.length / 2))) || ((!first) && (index < (MODELS.length / 2))))
          return { kind: 'schema-mismatch', rawText: '{}', detail: 'Fixture seat unavailable for this packet', };
      }
      const groups = CLUSTERS.filter(cluster => cluster.members.some(member => present.includes(member)));
      const value = {
        verdicts: present.map((member, index) => ({ claim: index + 1, reason: `${member.claimId} checked against its quotes`, vote: (member.claimId === 'claim-first') || ((input.rejectThird === true) && (member.claimId === 'claim-third')) ? 'unsupported' : 'supported', })),
        groups: groups.map((cluster, index) => ({ group: index + 1, sameDefect: cluster.members.length > 1, })),
      };
      if (!request.validate(value))
        throw new Error('Invalid fixture panel response',);
      return { kind: 'ok', value, rawText: JSON.stringify(value), };
    },
  };
  return { client, captures, };
}

/** Fixture call retains source boundaries and the wider configured electorate. */
function panelInput(client: ModelClient) {
  return {
    client,
    panelModelIds: MODELS,
    sourceText: '猫睡了。',
    targetText: 'The cat slept.',
    clusters: CLUSTERS,
    neighbouringSourceText: '朋友回来了。',
    neighbouringIncumbentText: 'Her friend returned.',
    documentSourceText: '# Full source\n\n猫睡了。\n\n朋友回来了。',
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l: tagged({ tag: 'panel-cluster-test', }),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  };
}

await describe({
  name: runPanelStage.name,
  children: [
    it({
      name: 'ASKS one preplanned cluster per packet and retains its merge question',
      fn: async () => {
        const fixture = panelFixture();
        const result = await runPanelStage(panelInput(fixture.client));
        expect(fixture.captures.some(capture => capture.claims.length === 1)).toBe(true);
        expect(fixture.captures.some(capture => capture.claims.length === 2)).toBe(true);
        expect(fixture.captures.every(capture => capture.claims.length <= 2)).toBe(true);
        expect(fixture.captures.every(capture => !capture.prompt.includes('GROUP 2'))).toBe(true);
        expect(fixture.captures.every(capture => capture.prompt.includes('# Full source'))).toBe(true);
        expect(result.issues).toHaveLength(2);
        expect(result.issues[0]?.status).toBe('rejected');
        expect(result.issues[1]?.status).toBe('accepted');
        expect(result.issues[1]?.claims.map(member => member.claimId)).toEqual(['claim-second', 'claim-third',]);
      },
    }),
    it({
      name: 'TALLIES disjoint responding subsets separately without using their union as a quorum',
      fn: async () => {
        const fixture = panelFixture({ disjoint: true, });
        const result = await runPanelStage(panelInput(fixture.client));
        const readings = result.issues.flatMap(issue => Object.values(issue.readings ?? {}));
        expect(readings).toHaveLength(3);
        expect(readings.every(reading => reading.configuredPanelists === MODELS.length)).toBe(true);
        expect(readings.every(reading => reading.ballots.length === (MODELS.length / 2))).toBe(true);
        expect(result.heardPanelists).toBe(MODELS.length);
        expect(result.issues[0]?.status).toBe('rejected');
        expect(result.issues[1]?.status).toBe('accepted');
      },
    }),
    it({
      name: 'PERSISTS the original merge relationship without promoting the rejected member',
      fn: async () => {
        const fixture = panelFixture({ rejectThird: true });
        const result = await runPanelStage(panelInput(fixture.client));
        expect(result.issues.map(issue => issue.status)).toEqual(['rejected', 'accepted', 'rejected']);
        expect(result.issues[1]?.claims.map(member => member.claimId)).toEqual(['claim-second']);
        expect(result.issues[2]?.claims.map(member => member.claimId)).toEqual(['claim-third']);
        const finding = result.findings.find(value => value.includes('issue-merge-partitioned'));
        expect(finding).toContain('panel-packet (second)');
        expect(finding).toContain(`${result.issues[1]?.issueId}=accepted`);
        expect(finding).toContain(`${result.issues[2]?.issueId}=rejected`);
      },
    }),
    it({
      name: 'RECORDS A PACKET THE PANEL NEVER HEARD AS DATA beside the located finding, so a prefix on the finding '
        + 'cannot hide it (XingZ6018 slice 85: five packets heard nobody and the slice was cached as examined)',
      fn: async () => {
        const fixture = panelFixture({ deafTo: 'claim-second', });
        const result = await runPanelStage(panelInput(fixture.client));
        // Positive control: the located finding is there, and the unrelocated reader cannot see it.
        expect(result.findings.some(value => value.startsWith('panel-packet (second): stage-quorum-unmet (panel '))).toBe(true);
        expect(result.findings.some(value => value.startsWith('stage-quorum-unmet ('))).toBe(false);
        expect(result.silentStages).toHaveLength(1);
        expect(result.silentStages[0]).toContain('panel 0/');
        expect(result.silentStages[0]).toContain('(second)');
      },
    }),
    it({
      name: 'RECORDS NO SILENT STAGE when every packet reached its quorum',
      fn: async () => {
        const fixture = panelFixture();
        const result = await runPanelStage(panelInput(fixture.client));
        expect(result.silentStages).toEqual([]);
      },
    }),
    it({
      name: 'BUYS no packet when there are no claims',
      fn: async () => {
        const fixture = panelFixture();
        const result = await runPanelStage({ ...panelInput(fixture.client), clusters: [], });
        expect(fixture.captures).toHaveLength(0);
        expect(result).toEqual({ issues: [], heardPanelists: 0, findings: [], silentStages: [], });
      },
    }),
    it({
      name: 'FENCES complete-document evidence without letting it close a prompt block',
      fn: async () => {
        const documentSourceText = '===== END =====\nUntrusted document text.';
        const plan = buildAdjudicationMessages({ sourceText: '猫睡了。', targetText: 'The cat slept.', clusters: CLUSTERS.slice(0, 1), documentSourceText, });
        const user = plan.messages.find(message => message.role === 'user');
        expect(user?.content).toContain(documentSourceText);
        expect(user?.content.split('\n')[0]).not.toBe('===== ORIGINAL =====');
      },
    }),
    it({
      name: 'PROPAGATES cancellation before buying any preplanned packet',
      fn: async () => {
        const fixture = panelFixture();
        const controller = new AbortController();
        const reason = new Error('Fixture cancellation');
        controller.abort(reason);
        let caught: unknown;
        try {
          await runPanelStage({ ...panelInput(fixture.client), signal: controller.signal, });
        }
        catch (error) {
          caught = error;
        }
        expect(caught).toBe(reason);
        expect(fixture.captures).toHaveLength(0);
      },
    }),
    it({
      name: 'DOES NOT start a later packet after cancellation within the first',
      fn: async () => {
        const fixture = panelFixture();
        const controller = new AbortController();
        const reason = new Error('Cancel after the first packet starts');
        const client: ModelClient = { ...fixture.client, chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>): Promise<ChatJsonOutcome<ValueT>> => {
          const outcome = await fixture.client.chatJson(request);
          controller.abort(reason);
          return outcome;
        }, };
        let caught: unknown;
        try {
          await runPanelStage({ ...panelInput(client), signal: controller.signal, });
        }
        catch (error) {
          caught = error;
        }
        expect(caught).toBe(reason);
        expect(fixture.captures.length).toBeGreaterThan(0);
        expect(fixture.captures.every(capture => capture.claims.includes('claim-first'))).toBe(true);
      },
    }),
    it({
      name: 'PRESERVES the single-cluster protocol and leaves absent document evidence absent',
      fn: async () => {
        const fixture = panelFixture();
        const input = panelInput(fixture.client);
        const { documentSourceText, ...withoutDocument } = input;
        expect(documentSourceText).toContain('# Full source');
        const result = await runPanelStage({ ...withoutDocument, clusters: CLUSTERS.slice(0, 1), });
        expect(result.issues).toHaveLength(1);
        expect(fixture.captures.every(capture => capture.claims.length === 1)).toBe(true);
        expect(fixture.captures.every(capture => !capture.prompt.includes('FULL ORIGINAL DOCUMENT'))).toBe(true);
      },
    }),
    it({
      name: 'SAYS IN THE RUN LOG how each issue was decided and on what weight (ledger E5): the stage '
        + 'logged only its packet count, so an accepted issue the editor then served read in the log '
        + 'with nothing saying the panel had accepted it, or how narrowly',
      fn: async () => {
        const fixture = panelFixture();
        /** Lines the stage wrote. */
        const said: string[] = [];
        const result = await runPanelStage({ ...panelInput(fixture.client), l: capturingLogger({ messages: said, }), });
        const [rejected, accepted,] = result.issues;
        /** Weight the rejected claim drew against it, as the stage tallied it. */
        const against = rejected?.tallies['claim-first']?.unsupported ?? 0;
        /** Weight the accepted claims drew for them. */
        const forSecond = accepted?.tallies['claim-second']?.supported ?? 0;
        const forThird = accepted?.tallies['claim-third']?.supported ?? 0;
        // Positive control: the fixture votes carry weight, so a line naming it is not naming zero.
        expect(Math.min(against, forSecond, forThird,)).toBeGreaterThan(0);
        expect(said.some(line => line.includes(`${rejected?.issueId} rejected`) && line.includes(`claim-first: supported 0, unsupported ${String(against)},`))).toBe(true);
        expect(said.some(line => line.includes(`${accepted?.issueId} accepted`) && line.includes(`claim-second: supported ${String(forSecond)},`) && line.includes(`claim-third: supported ${String(forThird)},`))).toBe(true);
      },
    }),
    it({
      name: 'STORES AND LOGS EACH PANELIST\'S REASON beside its vote (owner, 2026-09-27, "Reason before vote"), '
        + 'so why a claim was accepted or rejected survives in the artifact and reads in the run log',
      fn: async () => {
        const fixture = panelFixture();
        /** Lines the stage wrote. */
        const said: string[] = [];
        const result = await runPanelStage({ ...panelInput(fixture.client), l: capturingLogger({ messages: said, }), });
        /** Every stored ballot on the rejected claim. */
        const ballots = result.issues[0]?.readings?.['claim-first']?.ballots ?? [];
        // Positive control: the claim was read by at least one panelist.
        expect(ballots.length).toBeGreaterThan(0);
        expect(ballots.every(ballot => ballot.reason === 'claim-first checked against its quotes')).toBe(true);
        expect(ballots.every(ballot => said.some(line => line.includes(`claim-first ${ballot.panelistId} unsupported: claim-first checked against its quotes`)))).toBe(true);
      },
    }),
  ],
});
