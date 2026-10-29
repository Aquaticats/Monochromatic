/**
 Tests the single fixed polish round: refiners propose, judges select,
 the deterministic gate applies, and the absolute review that follows is
 recorded evidence on the settlement, never withholding authority and
 never buying a correction round.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { SettledArtifact, } from '@monochromatic-dev/translation-repair-artifact';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { unflooredFinding, } from '@monochromatic-dev/translation-repair-translate-stage';
import {
  polishConsolidation,
  reviewParagraphsOf,
} from '@monochromatic-dev/translation-repair-consolidate';
import {
  NaturalnessCompletenessError,
  prepareDocumentPair,
} from '@monochromatic-dev/translation-repair-adjudication';
import { floorReach, } from '@monochromatic-dev/translation-repair-translate-floor';
import { assertFinalNaturalnessComplete, } from '../dist/final/node/index.mjs';
import { settledArtifactOver, } from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/settled-artifact.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Active invented-size roster for every synthetic role.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Literal but faithful base wording.
 */
const BASE = 'She viewed rainy days proactively and spent many a cozy afternoon with the other cats, while doing her best to stay curious and close to the cats around her.';

/**
 Faithful idiomatic rewrite, as a refiner emits it: one line.
 */
const POLISHED = 'She kept a cheerful outlook on rainy days and spent many cozy afternoons with the other cats, doing her best to stay curious and close to those around her.';

/**
 The same rewrite as the page carries it: wrapped at its semantic boundary
 before the gate sees it (one entry, 2026-09-03: a gate judge preferred the
 unwrapped rewrite for "removing the stilted line breaks" and the page
 shipped single-line).
 */
const WRAPPED_POLISHED = 'She kept a cheerful outlook on rainy days and spent many cozy afternoons with the other cats,\n'
  + 'doing her best to stay curious and close to those around her.';

/**
 The base wording as the wrap would write it: a refinement that is exactly
 this changes nothing and must not ship as a change.
 */
const WRAPPED_BASE = 'She viewed rainy days proactively and spent many a cozy afternoon with the other cats,\n'
  + 'while doing her best to stay curious and close to the cats around her.';

/**
 Short literal prose final polish must still review.
 */
const SHORT_BASE = 'She had many cozy afternoons with the other cats.';

/**
 Faithful idiomatic rewrite of short prose.
 */
const SHORT_POLISHED = 'She spent many snug afternoons with the other cats.';

/**
 Target-authoritative contributor attribution baseline.
 */
const CONTRIBUTOR_BASE = 'Contributors for this entry: [Snow](https://example.test/snow)';

/**
 Words of the finding a polish writes when a review rejection is recorded as
 evidence while the gated text ships, which both rejection cases look for.
 */
const REJECTION_RECORDED = 'absolute naturalness rejection recorded as evidence';

/**
 Client serving rewrite, selection and final gate schemas.
 */
const client: ModelClient = {
  chatText: async () => {
    throw new Error('chatText unused by structured polish stages',);
  },
  chatJson: async <ValueT,>(
    request: ChatJsonRequest<ValueT>,
  ): Promise<ChatJsonOutcome<ValueT>> => {
    /**
     Schema identifying stage role.
     */
    const schema = request.responseFormat
      ?.json_schema
      .name;
    /**
     Whether request carries sentence-scale final polish fixture.
     */
    const short = JSON.stringify(request.messages,).includes(SHORT_BASE,);
    const value: unknown = (schema === 'refine_report')
      ? {
        rewrites: [
          {
            paragraph: 1,
            newText: short ? SHORT_POLISHED : POLISHED,
          },
        ],
      }
      : (schema === 'candidate_ballot')
        ? {
          best: 1,
          assessment: 'clear idiomatic improvement with same meaning',
        }
        : (schema === 'consolidation_polish_gate')
          ? {
            verdict: 'polished',
            unsupported: [],
            dropped: [],
            assessment: 'equally faithful and more idiomatic',
          }
          : (schema === 'absolute_naturalness_review')
            ? {
              isAcceptable: true,
              findings: [],
              assessment: 'whole passage is publication-ready',
            }
            : {};
    if (!request.validate(value,))
      throw new Error(`synthetic ${String(schema,)} reply failed validation`,);
    return {
      kind: 'ok',
      value,
      rawText: JSON.stringify(value,),
    };
  },
  quotas: async () => {
    throw new Error('quotas unused by polish stages',);
  },
};

/**
 Builds a client whose absolute reviews follow a per-round verdict script
 and whose refiners may answer exactly once.

 A SECOND REFINE CALL THROWS: the fixed polish round buys one proposal, so
 any correction re-ask is a regression this harness turns into a failure.

 @param reviewAcceptableByRound - acceptable status per one-based review round; absent rounds throw as lost seats

 @returns Scripted single-round client

 @example
 ```ts
 const rejecting = singleRoundClient({ reviewAcceptableByRound: [false,], },);
 ```
 */
function singleRoundClient(
  { reviewAcceptableByRound, }: { readonly reviewAcceptableByRound: readonly boolean[]; },
): ModelClient {
  /**
   Stateful call counts separating refine and review rounds.
   */
  const calls = {
    refine: 0,
    review: 0,
  };
  return {
    chatText: async () => {
      throw new Error('chatText unused by structured polish stages',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Schema identifying stage role.
       */
      const schema = request.responseFormat
        ?.json_schema
        .name;
      /**
       Scripted stage reply.
       */
      const value: unknown = (() => {
        if (schema === 'refine_report') {
          calls.refine += 1;
          if (calls.refine > 1)
            throw new Error('a second refine round was bought; the polish round must never re-ask',);
          return {
            rewrites: [{
              paragraph: 1,
              newText: POLISHED,
            },],
          };
        }
        if (schema === 'candidate_ballot') {
          return {
            best: 1,
            assessment: 'clear idiomatic improvement with same meaning',
          };
        }
        if (schema === 'consolidation_polish_gate') {
          return {
            verdict: 'polished',
            unsupported: [],
            dropped: [],
            assessment: 'scripted fidelity comparison',
          };
        }
        if (schema === 'absolute_naturalness_review') {
          calls.review += 1;
          /**
           One-based absolute review round across every roster seat.
           */
          const reviewRound = Math.ceil(calls.review / ROSTER.length,);
          /**
           Scripted acceptance for this round, absent to lose the seat.
           */
          const acceptable = reviewAcceptableByRound.at(reviewRound - 1,);
          if (acceptable === undefined)
            throw new Error('scripted lost review seat',);
          if (!acceptable) {
            return {
              isAcceptable: false,
              findings: [{
                paragraph: 1,
                problem: 'Replace stiff source-language word order.',
              },],
              assessment: 'candidate retains translationese',
            };
          }
          return {
            isAcceptable: true,
            findings: [],
            assessment: 'whole passage is publication-ready',
          };
        }
        return {};
      })();
      if (!request.validate(value,))
        throw new Error(`synthetic ${String(schema,)} reply failed validation`,);
      return {
        kind: 'ok',
        value,
        rawText: JSON.stringify(value,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by polish stages',);
    },
  };
}

/**
 Builds a client that answers every polish role and records which roles it
 was asked, the gate choosing as scripted.

 @param gateChoice - what every polish gate judge chooses

 @returns Client and schema of every request, in the order asked

 @example
 ```ts
 const { client, asked, } = recordingClient({ gateChoice: 'base', },);
 ```
 */
function recordingClient(
  { gateChoice, }: { readonly gateChoice: 'base' | 'polished'; },
): { readonly client: ModelClient; readonly asked: readonly string[]; } {
  /**
   Schema of every request, in the order asked.
   */
  const asked: string[] = [];
  return {
    asked,
    client: {
      chatText: async () => {
        throw new Error('chatText unused by structured polish stages',);
      },
      chatJson: async <ValueT,>(
        request: ChatJsonRequest<ValueT>,
      ): Promise<ChatJsonOutcome<ValueT>> => {
        /**
         Schema identifying stage role.
         */
        const schema = request.responseFormat?.json_schema.name ?? 'no schema';
        asked.push(schema,);
        /**
         Scripted stage reply.
         */
        const value: unknown = (schema === 'refine_report')
          ? { rewrites: [{ paragraph: 1, newText: POLISHED, },], }
          : (schema === 'candidate_ballot')
            ? { best: 1, assessment: 'clear idiomatic improvement with same meaning', }
            : (schema === 'consolidation_polish_gate')
              ? { verdict: gateChoice, unsupported: [], dropped: [], assessment: 'scripted fidelity comparison', }
              : { isAcceptable: true, findings: [], assessment: 'whole passage is publication-ready', };
        if (!request.validate(value,))
          throw new Error(`synthetic ${schema} reply failed validation`,);
        return {
          kind: 'ok',
          value,
          rawText: JSON.stringify(value,),
        };
      },
      quotas: async () => {
        throw new Error('quotas unused by polish stages',);
      },
    },
  };
}

/**
 Model roles shared by every case.
 */
const CONFIG = {
  refinerModelIds: [ROSTER[0],],
  judgeModelIds: ROSTER,
  gateModelIds: ROSTER,
  declaredNames: [],
  definitions: '',
} as const;

/**
 Wraps one polish record in the minimal artifact the completeness guard reads,
 with the contest record replaceable where a case needs a slice of another
 syntax.

 @param polish - settled polish record as the pipeline would persist it

 @param selection - lane contest record, contested over slice 1 by default

 @returns Artifact whose consolidated body slice carries that polish

 @example
 ```ts
 const artifact = artifactCarrying({ polish, },);
 ```
 */
function artifactCarrying(
  {
    polish,
    selection = {
      kind: 'contested',
      slices: [{
        sliceIndex: 1,
        verdict: {
          kind: 'lane-won',
          lane: 'translate',
        },
        ballots: [],
        usable: 2,
      },],
    },
  }: {
    readonly polish: unknown;
    readonly selection?: unknown;
  },
): SettledArtifact {
  // Serialization round-trip first, because persistence writes JSON and the
  // guard must accept what a resumed run would read back.
  // oxlint-disable-next-line unicorn/prefer-structured-clone -- JSON semantics are the point: persistence writes JSON, so undefined-valued keys must drop and a non-serializable field must fail here, both of which structuredClone would hide.
  return JSON.parse(JSON.stringify({
    laneSelection: selection,
    consolidation: {
      kind: 'settled',
      slices: [{
        sliceIndex: 1,
        terminal: 'gate-kept-standing',
        shipped: { kind: 'unchanged', },
        rewrapped: false,
        demoted: false,
        verdicts: [],
        gate: { kind: 'not-asked', },
        polish,
      },],
    },
  },),) as SettledArtifact;
}

await describe({
  name: polishConsolidation.name,
  children: [
    it({
      name: 'SHIPS IDIOMATIC REWRITE only after selection and fidelity-first gate',
      fn: async () => {
        const polish = await polishConsolidation({
          client,
          sourceText: '她总是乐观地看待下雨天，和猫友们度过了许多惬意的午后。',
          archiveText: BASE,
          baseText: BASE,
          lineStructured: false,
          sliceIndex: 1,
          config: CONFIG,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'consolidation-polish-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(polish.kind,).toBe('settled',);
        if (polish.kind !== 'settled')
          throw new Error('body polish fixture did not run',);
        expect(polish.changed,).toBe(true,);
        expect(polish.text,).toBe(WRAPPED_POLISHED,);
        expect(polish.gate?.ships,).toBe('polished',);
        expect(polish.rounds.length,).toBe(1,);
        expect(polish.review.correctionCount,).toBe(0,);
        expect(polish.review.rounds[0]?.verdict,).toBe('acceptable',);
        expect(polish.review.confirmations,).toHaveLength(1,);
      },
    },),

    it({
      name: 'WRAPS THE REFINEMENT BEFORE ITS GATE so the gate judges the bytes the page carries, '
        + 'LEAVES a line-structured slice as the refiner wrote it, and DEMOTES a refinement that is '
        + 'only the base re-wrapped or the base with its soft line breaks elsewhere (ledger B26)',
      fn: async () => {
        /**
         Gate subjects the scripted gate was shown, so the test can prove
         the wrapped bytes reached the deciders rather than only the page.
         */
        const gateSubjects: string[] = [];
        /**
         Client answering the refine schema with a given rewrite and
         recording what the polish gate is asked about.

         @param newText - rewrite the refiner returns for paragraph 1

         @returns Scripted client
         */
        function rewritingClient({ newText, }: { readonly newText: string; },): ModelClient {
          return {
            ...client,
            chatJson: async <ValueT,>(
              request: ChatJsonRequest<ValueT>,
            ): Promise<ChatJsonOutcome<ValueT>> => {
              /**
               Schema identifying stage role.
               */
              const schema = request.responseFormat
                ?.json_schema
                .name;
              if (schema === 'consolidation_polish_gate')
                gateSubjects.push(JSON.stringify(request.messages,),);
              if (schema !== 'refine_report')
                return await client.chatJson(request,);
              /**
               Scripted rewrite.
               */
              const value: unknown = {
                rewrites: [{
                  paragraph: 1,
                  newText,
                },],
              };
              if (!request.validate(value,))
                throw new Error('scripted refine reply failed validation',);
              return {
                kind: 'ok',
                value,
                rawText: JSON.stringify(value,),
              };
            },
          };
        }

        const wrapped = await polishConsolidation({
          client: rewritingClient({ newText: POLISHED, },),
          sourceText: '她总是乐观地看待下雨天，和猫友们度过了许多惬意的午后。',
          archiveText: BASE,
          baseText: BASE,
          lineStructured: false,
          sliceIndex: 1,
          config: CONFIG,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'consolidation-polish-wrap-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(wrapped.kind,).toBe('settled',);
        if (wrapped.kind !== 'settled')
          throw new Error('wrap fixture did not settle',);
        expect(wrapped.text,).toBe(WRAPPED_POLISHED,);
        expect(wrapped.proposedText,).toBe(WRAPPED_POLISHED,);
        // The gate was asked about the wrapped bytes as they render: on a
        // prose slice each paragraph is shown one line, so where the wrap
        // broke the line is nothing the gate can weigh (class one hundred
        // fifty-three, one entry: a one-line base beside a wrapped polish).
        expect(gateSubjects.length,).toBeGreaterThan(0,);
        expect(gateSubjects.every((subject,) => subject.includes(JSON.stringify(POLISHED,).slice(1, -1),),),).toBe(true,);
        expect(gateSubjects.some((subject,) => subject.includes(JSON.stringify(WRAPPED_POLISHED,).slice(1, -1),),),).toBe(false,);

        // Line-structured: the one-line source keeps a one-line rewrite as
        // written; a wrap here would break the line count the rule protects.
        const governed = await polishConsolidation({
          client: rewritingClient({ newText: POLISHED, },),
          sourceText: '她总是乐观地看待下雨天，和猫友们度过了许多惬意的午后。',
          archiveText: BASE,
          baseText: BASE,
          lineStructured: true,
          sliceIndex: 1,
          config: CONFIG,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'consolidation-polish-governed-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(governed.kind,).toBe('settled',);
        if (governed.kind !== 'settled')
          throw new Error('governed fixture did not settle',);
        expect(governed.text,).toBe(POLISHED,);

        // A refinement that is the base with its line break put back is not a
        // change: the slice keeps the base byte for byte and says why.
        const rewrapOnly = await polishConsolidation({
          client: rewritingClient({ newText: WRAPPED_BASE, },),
          sourceText: '她总是乐观地看待下雨天，和猫友们度过了许多惬意的午后。',
          archiveText: BASE,
          baseText: BASE,
          lineStructured: false,
          sliceIndex: 1,
          config: CONFIG,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'consolidation-polish-rewrap-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(rewrapOnly.kind,).toBe('settled',);
        if (rewrapOnly.kind !== 'settled')
          throw new Error('rewrap fixture did not settle',);
        expect(rewrapOnly.changed,).toBe(false,);
        expect(rewrapOnly.text,).toBe(BASE,);
        expect(
          rewrapOnly.findings
            .some(function namesDemotion(finding,): boolean {
              return finding === 'consolidation-polish is the base in all but layout';
            },),
        ).toBe(true,);

        // Nor is one that is the base with a soft break where the wrap would
        // never put it: the site renders it as a space, and no wrap of either
        // text makes the two equal, since the wrap only adds breaks (ledger B26).
        const softOnly = await polishConsolidation({
          client: rewritingClient({
            newText: 'She viewed rainy days proactively and spent many a cozy afternoon\nwith the other cats, '
              + 'while doing her best to stay curious and close to the cats around her.',
          },),
          sourceText: '她总是乐观地看待下雨天，和猫友们度过了许多惬意的午后。',
          archiveText: BASE,
          baseText: BASE,
          lineStructured: false,
          sliceIndex: 1,
          config: CONFIG,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'consolidation-polish-soft-break-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(softOnly.kind,).toBe('settled',);
        if (softOnly.kind !== 'settled')
          throw new Error('soft-break fixture did not settle',);
        expect(softOnly.changed,).toBe(false,);
        expect(softOnly.text,).toBe(BASE,);
        expect(softOnly.findings,).toContain('consolidation-polish is the base in all but layout',);
      },
    },),

    it({
      name: 'REVIEWS SHORT BODY PROSE below repair-lane refinement window',
      fn: async () => {
        const polish = await polishConsolidation({
          client,
          sourceText: '她和猫友们度过了许多惬意的午后。',
          archiveText: SHORT_BASE,
          baseText: SHORT_BASE,
          lineStructured: false,
          sliceIndex: 2,
          config: CONFIG,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'consolidation-polish-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(polish.kind,).toBe('settled',);
        expect(polish.kind === 'settled' ? polish.text : '',).toBe(SHORT_POLISHED,);
      },
    },),

    it({
      name: 'KEEPS TARGET CONTRIBUTOR BASELINE when polish candidate drops authority',
      fn: async () => {
        const polish = await polishConsolidation({
          client,
          sourceText: '本条目贡献者：雪猫',
          archiveText: CONTRIBUTOR_BASE,
          baseText: CONTRIBUTOR_BASE,
          lineStructured: false,
          sliceIndex: 1,
          config: CONFIG,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'consolidation-polish-contributor-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(polish.kind,).toBe('settled',);
        if (polish.kind !== 'settled')
          throw new Error('contributor authority fixture did not settle',);
        expect(polish.text,).toBe(CONTRIBUTOR_BASE,);
        expect(polish.changed,).toBe(false,);
      },
    },),

    it({
      name: 'RECORDS AN ABSOLUTE REJECTION AS EVIDENCE and ships the gated text with no correction call',
      fn: async () => {
        const polish = await polishConsolidation({
          client: singleRoundClient({ reviewAcceptableByRound: [false,], },),
          sourceText: '她总是乐观地看待下雨天，和猫友们度过了许多惬意的午后。',
          archiveText: BASE,
          baseText: BASE,
          lineStructured: false,
          sliceIndex: 1,
          config: CONFIG,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'consolidation-polish-rejection-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        // The reviewer verdict is evidence, not authority: the gated text
        // ships and the located problem lives on the findings. The scripted
        // client throws on any second refine call, so returning at all proves
        // no correction was bought.
        expect(polish.kind,).toBe('settled',);
        if (polish.kind !== 'settled')
          throw new Error('rejection fixture did not settle',);
        expect(polish.text,).toBe(WRAPPED_POLISHED,);
        expect(polish.review
          .correctionCount,).toBe(0,);
        expect(polish.review
          .rounds[0]
          ?.verdict,).toBe('unacceptable',);
        expect(
          polish.findings
            .some(function namesRecording(finding,): boolean {
              return finding.includes(REJECTION_RECORDED,);
            },),
        ).toBe(true,);
      },
    },),

    it({
      name: 'RECORDS CONFIRMATION REJECTION AS EVIDENCE instead of buying a correction',
      fn: async () => {
        const polish = await polishConsolidation({
          client: singleRoundClient({
            reviewAcceptableByRound: [
              true,
              false,
            ],
          },),
          sourceText: '她总是乐观地看待下雨天，和猫友们度过了许多惬意的午后。',
          archiveText: BASE,
          baseText: BASE,
          lineStructured: false,
          sliceIndex: 1,
          config: CONFIG,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'consolidation-polish-confirmation-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        // Discovery accepted, the acceptance challenge rejected: the decisive
        // rejection is recorded beside the earlier acceptance and nothing is
        // withheld or re-asked.
        expect(polish.kind,).toBe('settled',);
        if (polish.kind !== 'settled')
          throw new Error('confirmation rejection fixture did not settle',);
        expect(polish.text,).toBe(WRAPPED_POLISHED,);
        expect(polish.review
          .rounds[0]
          ?.verdict,).toBe('unacceptable',);
        expect(polish.review
          .confirmations,).toHaveLength(1,);
        expect(
          polish.findings
            .some(function namesRecording(finding,): boolean {
              return finding.includes(REJECTION_RECORDED,);
            },),
        ).toBe(true,);
      },
    },),

    it({
      name: 'RECORDS QUORUM-NOT-MET AS EVIDENCE and ships the gated text',
      fn: async () => {
        // Every review seat is lost, so the review roster is unheard; the
        // producing round already settled, and reviewer absence after it must
        // not withhold the entry.
        const polish = await polishConsolidation({
          client: singleRoundClient({ reviewAcceptableByRound: [], },),
          sourceText: '她总是乐观地看待下雨天，和猫友们度过了许多惬意的午后。',
          archiveText: BASE,
          baseText: BASE,
          lineStructured: false,
          sliceIndex: 1,
          config: CONFIG,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'consolidation-polish-quorum-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(polish.kind,).toBe('settled',);
        if (polish.kind !== 'settled')
          throw new Error('quorum fixture did not settle',);
        expect(polish.text,).toBe(WRAPPED_POLISHED,);
        expect(polish.review
          .rounds[0]
          ?.verdict,).toBe('quorum-not-met',);
        expect(
          polish.findings
            .some(function namesRecording(finding,): boolean {
              return finding.includes('absolute naturalness review quorum not met',);
            },),
        ).toBe(true,);
      },
    },),

    it({
      name: 'ROUND-TRIPS A REJECTED AND A QUORUMLESS FINAL REVIEW through the persistence completeness guard',
      fn: async () => {
        // The exact shape this remediation exists for: verdicts recorded as
        // findings while the page ships. The completeness guard re-parses
        // the review with the correction chain required, so any latent
        // acceptable-final assumption in that parser would refuse this
        // record at persist time, after a whole entry was paid for.
        const [
          rejected,
          quorumless,
        ] = await Promise.all([
          polishConsolidation({
            client: singleRoundClient({ reviewAcceptableByRound: [false,], },),
            sourceText: '她总是乐观地看待下雨天，和猫友们度过了许多惬意的午后。',
            archiveText: BASE,
            baseText: BASE,
            lineStructured: false,
            sliceIndex: 1,
            config: CONFIG,
            signal: AbortSignal.timeout(HANG_STOP_MS,),
            perCallTimeoutMs: HANG_STOP_MS,
            l: tagged({ tag: 'consolidation-polish-roundtrip-rejection-test', },),
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },),
          polishConsolidation({
            client: singleRoundClient({ reviewAcceptableByRound: [], },),
            sourceText: '她总是乐观地看待下雨天，和猫友们度过了许多惬意的午后。',
            archiveText: BASE,
            baseText: BASE,
            lineStructured: false,
            sliceIndex: 1,
            config: CONFIG,
            signal: AbortSignal.timeout(HANG_STOP_MS,),
            perCallTimeoutMs: HANG_STOP_MS,
            l: tagged({ tag: 'consolidation-polish-roundtrip-quorum-test', },),
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },),
        ],);
        expect(rejected.kind,).toBe('settled',);
        expect(quorumless.kind,).toBe('settled',);
        // A throw here is the failure this test exists to catch.
        assertFinalNaturalnessComplete({
          artifact: artifactCarrying({ polish: rejected, },),
        },);
        assertFinalNaturalnessComplete({
          artifact: artifactCarrying({ polish: quorumless, },),
        },);
      },
    },),

    it({
      name: 'ACCEPTS AN UNENDORSED STANDING THAT SHIPPED WITH ITS FINDING, whose polish never ran over the '
        + 'unsafe baseline (the no-loop single attempt), and still REFUSES every other body slice without a '
        + 'settled polish: one entry\'s rerun of 2026-09-02 ended INCOMPLETE after 117 minutes on exactly '
        + 'this record',
      fn: async () => {
        assertFinalNaturalnessComplete({
          artifact: artifactCarrying({
            polish: {
              kind: 'not-run',
              reason: 'unsafe-baseline',
            },
          },),
        },);
        expect(() => assertFinalNaturalnessComplete({
          artifact: artifactCarrying({
            polish: {
              kind: 'not-run',
              reason: 'not-configured',
            },
          },),
        },),).toThrow(NaturalnessCompletenessError,);
        expect(() => assertFinalNaturalnessComplete({
          artifact: artifactCarrying({ polish: undefined, },),
        },),).toThrow(NaturalnessCompletenessError,);
      },
    },),

    it({
      name: 'THROWS THE INVARIANT, naming both records, on an artifact whose contest or consolidation '
        + 'never ran, since the one writer persistence is handed records both as run',
      fn: async () => {
        /**
         One-slice cat page every pairing is built over.
         */
        const prepared = prepareDocumentPair({
          sourceText: '猫在睡觉。',
          targetText: 'The cat naps.',
        },);
        /**
         Each pairing the builder takes and no pass writes. The contested one
         carries a body slice, the record once refused as a slice short of
         the naturalness floor.
         */
        const pairings = [
          {
            laneSelection: { kind: 'pending-human-decision', },
            consolidation: { kind: 'not-run', },
          },
          {
            laneSelection: {
              kind: 'contested',
              slices: [{
                sliceIndex: 0,
                verdict: {
                  kind: 'lane-won',
                  lane: 'translate',
                },
                ballots: [],
                usable: 2,
              },],
            },
            consolidation: { kind: 'not-run', },
          },
          {
            laneSelection: { kind: 'pending-human-decision', },
            consolidation: {
              kind: 'settled',
              slices: [],
            },
          },
        ] as const;
        expect(pairings.map(function refusalOf(
          {
            laneSelection,
            consolidation,
          },
        ): string {
          return String(caught(function persists(): void {
            assertFinalNaturalnessComplete({
              artifact: settledArtifactOver({
                prepared,
                entryId: 'Whiskers',
                laneSelection,
                consolidation,
              },),
            },);
          },),);
        },),).toEqual([
          'Error: unreachable: the artifact handed to persistence records its contest as '
            + 'pending-human-decision and its consolidation as not-run, though settledEntryArtifact, its one '
            + 'writer, records them as contested and settled',
          'Error: unreachable: the artifact handed to persistence records its contest as contested and its '
            + 'consolidation as not-run, though settledEntryArtifact, its one writer, records them as '
            + 'contested and settled',
          'Error: unreachable: the artifact handed to persistence records its contest as '
            + 'pending-human-decision and its consolidation as settled, though settledEntryArtifact, its one '
            + 'writer, records them as contested and settled',
        ],);
      },
    },),

    it({
      name: 'ACCEPTS a front-matter slice whose polish records the front-matter skip, and REFUSES the '
        + 'same slice under the unsafe-baseline skip a body slice ships with',
      fn: async () => {
        /**
         Contested slice carrying front matter.
         */
        const selection = {
          kind: 'contested',
          slices: [{
            sliceIndex: 1,
            verdict: {
              kind: 'lane-won',
              lane: 'translate',
            },
            ballots: [],
            usable: 2,
            eligibility: {
              syntax: 'front-matter',
            },
          },],
        };
        assertFinalNaturalnessComplete({
          artifact: artifactCarrying({
            polish: {
              kind: 'not-run',
              reason: 'front-matter',
            },
            selection,
          },),
        },);
        /**
         What the guard throws for the same slice under the skip a body slice
         ships with, so only the front-matter rule can refuse it.
         */
        const refusal = caught(function persistsUnsafeBaseline(): void {
          assertFinalNaturalnessComplete({
            artifact: artifactCarrying({
              polish: {
                kind: 'not-run',
                reason: 'unsafe-baseline',
              },
              selection,
            },),
          },);
        },);
        expect(refusal,).toBeInstanceOf(NaturalnessCompletenessError,);
        expect(String(refusal,),).toBe('NaturalnessCompletenessError: slice 1 did not meet absolute naturalness floor',);
      },
    },),

    it({
      name: 'REFUSES a settled polish carrying no review record, since nothing proves a reviewer read '
        + 'the body slice',
      fn: async () => {
        /**
         What the guard throws for a settled polish no reviewer record backs.
         */
        const refusal = caught(function persistsUnreviewed(): void {
          assertFinalNaturalnessComplete({
            artifact: artifactCarrying({
              polish: {
                kind: 'settled',
                baseText: 'The cat naps.',
                proposedText: 'The cat is napping.',
                text: 'The cat is napping.',
                changed: true,
              },
            },),
          },);
        },);
        expect(refusal,).toBeInstanceOf(NaturalnessCompletenessError,);
        expect(String(refusal,),).toBe('NaturalnessCompletenessError: slice 1 did not meet absolute naturalness floor',);
      },
    },),

    it({
      name: 'SHOWS THE REVIEWER EVERY BODY BLOCK and records that count, so a blockquote candidate with '
        + 'no refinable paragraph still gives a reviewer one paragraph to cite (one entry, slice 10, '
        + '2026-09-02: zero refinable paragraphs, six of nine ballots refused as out of range), and the '
        + 'completeness guard recomputes the same set',
      fn: async () => {
        /**
         A letter in blockquote, which the polish may not edit but a reviewer
         must still be able to cite.
         */
        const poem = '> By the time you read this note,\n> I should already be sunbathing on the neighbour’s balcony.\n>\n> From the moment we met,\n> the cans always came on time.';
        expect(reviewParagraphsOf({ text: poem, },),).toEqual([poem,],);
        expect(reviewParagraphsOf({ text: `${poem}\n\nA closing paragraph.`, },),)
          .toEqual([poem, 'A closing paragraph.',],);
        expect(reviewParagraphsOf({ text: '', },),).toEqual([],);

        const polish = await polishConsolidation({
          client: singleRoundClient({ reviewAcceptableByRound: [true,], },),
          sourceText: '> 当你读到这张纸条的时候，\n> 我应该已经在邻居家的阳台上晒太阳了。\n>\n> 从我们相遇开始，\n> 罐头总是准时出现。',
          archiveText: poem,
          baseText: poem,
          lineStructured: true,
          sliceIndex: 1,
          config: CONFIG,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'consolidation-polish-blockquote-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(polish.kind,).toBe('settled',);
        if (polish.kind !== 'settled')
          throw new Error('the polish did not settle',);
        expect(polish.review.rounds[0]?.paragraphCount,).toBe(1,);
        // The guard recomputes the paragraph digests from the final text with
        // the same set the writer used; a mismatch here is the generation-ten
        // reader disagreeing with its writer.
        assertFinalNaturalnessComplete({
          artifact: artifactCarrying({ polish, },),
        },);
      },
    },),

    it({
      name: 'SKIPS SYNTAX-BEARING FRONT MATTER before any model call',
      fn: async () => {
        const polish = await polishConsolidation({
          client,
          sourceText: '---\nname: 猫猫\n---\n',
          archiveText: '---\nname: Maomao\n---\n',
          baseText: '---\nname: Maomao\n---\n',
          syntax: 'front-matter',
          lineStructured: false,
          sliceIndex: 0,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'consolidation-polish-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(polish,).toEqual({
          kind: 'not-run',
          reason: 'front-matter',
        },);
      },
    },),

    it({
      name: 'ASKS NO REFINER, RANKER OR GATE where the floor can compare nothing, keeping the base with the '
        + 'unfloored finding (ledger B48: the round paid them for a polish its structural check left unvalidated)',
      fn: async () => {
        /** An original the strict grammar cannot read: its brace never closes. */
        const sourceText = '她总是乐观地看待{下雨天，和猫友们度过了许多惬意的午后。';
        /** What the floor makes of this ground. */
        const reach = floorReach({
          sourceText,
          pageText: BASE,
        },);
        if (reach.kind !== 'blind')
          throw new Error('fixture original is readable, so this case would not test blind ground',);
        const { client: recording, asked, } = recordingClient({ gateChoice: 'polished', },);
        const polish = await polishConsolidation({
          client: recording,
          sourceText,
          archiveText: BASE,
          baseText: BASE,
          lineStructured: false,
          sliceIndex: 0,
          config: CONFIG,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'consolidation-polish-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        if (polish.kind !== 'settled')
          throw new Error(`expected a settled polish, got ${polish.kind}`,);
        expect(new Set(asked,),).toEqual(new Set(['absolute_naturalness_review',],),);
        expect({
          text: polish.text,
          changed: polish.changed,
          refinersHeard: polish.refinersHeard,
          rounds: polish.rounds,
        },).toEqual({
          text: BASE,
          changed: false,
          refinersHeard: [],
          rounds: [],
        },);
        expect(polish.findings,).toContain(unflooredFinding({
          stage: 'consolidation-polish',
          detail: reach.detail,
        },),);
      },
    },),

    it({
      name: 'KEEPS THE BASE when the gate backs it over a structurally valid polish, recording the gate',
      fn: async () => {
        const { client: recording, asked, } = recordingClient({ gateChoice: 'base', },);
        const polish = await polishConsolidation({
          client: recording,
          sourceText: '她总是乐观地看待下雨天，和猫友们度过了许多惬意的午后。',
          archiveText: BASE,
          baseText: BASE,
          lineStructured: false,
          sliceIndex: 0,
          config: CONFIG,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'consolidation-polish-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        if (polish.kind !== 'settled')
          throw new Error(`expected a settled polish, got ${polish.kind}`,);
        expect(asked,).toContain('consolidation_polish_gate',);
        expect({
          text: polish.text,
          changed: polish.changed,
          ships: polish.gate?.ships,
        },).toEqual({
          text: BASE,
          changed: false,
          ships: 'base',
        },);
        expect(polish.proposedText,).not.toBe(BASE,);
      },
    },),

    it({
      name: 'NAMES AN OBJECTION CORRECTION by its judges and objection count, and whether the round corrected '
        + 'the base or kept it',
      fn: async () => {
        /**
         The objection correction each run asks.
         */
        const mode = {
          kind: 'objection-correction',
          groups: [{
            origin: 'consolidation gate',
            objections: ['The base calls her outlook proactive, which the ORIGINAL does not say.',],
          },],
        } as const;
        /**
         Each gate choice, and the objection finding its run recorded.
         */
        const recorded = await Promise.all((['polished', 'base',] as const).map(
          async function findingFor(gateChoice,): Promise<readonly [string, readonly string[],]> {
            const polish = await polishConsolidation({
              client: recordingClient({ gateChoice, },).client,
              sourceText: '她总是乐观地看待下雨天，和猫友们度过了许多惬意的午后。',
              archiveText: BASE,
              baseText: BASE,
              lineStructured: false,
              sliceIndex: 0,
              config: CONFIG,
              mode,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l: tagged({ tag: 'consolidation-polish-test', },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            if (polish.kind !== 'settled')
              throw new Error(`expected a settled polish, got ${polish.kind}`,);
            return [
              gateChoice,
              polish.findings.filter(function namesObjectionPolish(finding,): boolean {
                return finding.startsWith('polish-objection-correction',);
              },),
            ];
          },
        ),);
        expect(recorded,).toEqual([
          ['polished', ['polish-objection-correction (consolidation gate: 1 objection(s), corrected)',],],
          ['base', ['polish-objection-correction (consolidation gate: 1 objection(s), base kept)',],],
        ],);
      },
    },),
  ],
},);
