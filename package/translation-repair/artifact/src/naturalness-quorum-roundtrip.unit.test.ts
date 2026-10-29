/**
 Naturalness review must retain its wider quorum across artifact serialization.

 @module
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  confirmAbsoluteNaturalness,
  reviewAbsoluteNaturalness,
} from '@monochromatic-dev/translation-repair-archive';
import { parseNaturalnessReview, } from '../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_OPENROUTER_ONLY,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/** Asked seats matching the live failure's provider-independent shape. */
const ROSTER = [
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_ONLY,
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_OPENROUTER_ONLY,
] as const;
/** Exact candidate shared by every review and its artifact. */
const TEXT = 'The cat sleeps on the windowsill.';
/** Wider independent bench requiring five usable voices. */
const WIDE_BENCH = 9;

/**
 Scripts four usable voices, either immediately or only during confirmation.

 @param loseOnChallenge - whether discovery first hears every seat

 @returns Client with no provider traffic
 */
function clientFor({ loseOnChallenge = false, }: { readonly loseOnChallenge?: boolean; },): ModelClient {
  /** Per-seat calls separate discovery from its same-seat challenge. */
  const calls = new Map<RosterModelId, number>();
  return {
    chatText: async () => { throw new Error('Unexpected text call',); },
    quotas: async () => { throw new Error('Unexpected quota read',); },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      /** Earlier calls by this independently scripted seat. */
      const previous = calls.get(request.modelId,) ?? 0;
      calls.set(request.modelId, previous + 1,);
      if (((!loseOnChallenge) || (previous > 0))
        && ((request.modelId === SEAT_HYPER_ONLY) || (request.modelId === SEAT_OPENROUTER_ONLY))) {
        return { kind: 'schema-mismatch', rawText: '{}', detail: 'Fixture unusable seat', };
      }
      /** Every usable seat independently accepts the candidate. */
      const value: unknown = { isAcceptable: true, findings: [], assessment: 'The cat sentence is natural.', };
      if (!request.validate(value,))
        throw new Error('Fixture does not satisfy reviewer schema',);
      return { kind: 'ok', value, rawText: JSON.stringify(value,), };
    },
  };
}

/**
 Uses production whole-bench quorum with the asked-seat window made explicit.

 @param client - scripted independent reviewer replies

 @returns Shared request for discovery and confirmation
 */
function requestFor(client: ModelClient,): Parameters<typeof reviewAbsoluteNaturalness>[0] {
  return {
    client,
    modelIds: ROSTER,
    quorumOver: WIDE_BENCH,
    subject: { lineStructured: false, sourceText: '猫睡在窗台上。', candidateText: TEXT, paragraphs: [TEXT,], },
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    exchangeTimeoutMs: HANG_STOP_MS,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
    fanOut: 'whole-bench',
    l: tagged({ tag: 'naturalness-quorum-roundtrip', },),
  };
}

await describe({
  name: 'naturalness quorum provenance',
  children: [
    it({
      name: 'ROUND TRIPS a live-shaped four-of-six review requiring five voices',
      fn: async () => {
        /** Real producing stage, not a hand-written verdict fixture. */
        const round = await reviewAbsoluteNaturalness(
          requestFor(clientFor({},),),
        );
        expect(round.usable,).toBe(4,);
        expect(round.verdict,).toBe('quorum-not-met',);
        /** Exact wire bytes must retain every fact the reader needs. */
        const encoded = JSON.stringify({ correctionCount: 0, corrections: [], confirmations: [], rounds: [round,], },);
        const parsed = parseNaturalnessReview({
          value: JSON.parse(encoded,),
          path: 'consolidation.slices[3].polish.review',
          finalText: TEXT,
          correctionChainRequired: true,
          everyBodyBlockReviewed: true,
          quorumBasisRequired: true,
        },);
        expect(parsed.rounds[0]?.verdict,).toBe('quorum-not-met',);
        expect(parsed.rounds[0],).toHaveProperty('quorumOver', WIDE_BENCH,);
      },
    },),
    it({
      name: 'KEEPS the discovery quorum when confirmation asks its reduced roster',
      fn: async () => {
        /** Discovery accepts, but confirmation hears only four of the six seats. */
        const confirmed = await confirmAbsoluteNaturalness(
          requestFor(clientFor({ loseOnChallenge: true, },),),
        );
        expect(confirmed.confirmations,).toHaveLength(1,);
        expect(confirmed.review.usable,).toBe(4,);
        expect(confirmed.review.verdict,).toBe('quorum-not-met',);
        /** Both distinct perspectives retain the same quorum basis in wire bytes. */
        const encoded = JSON.stringify({ correctionCount: 0, corrections: [], confirmations: confirmed.confirmations, rounds: [confirmed.review,], },);
        const parsed = parseNaturalnessReview({
          value: JSON.parse(encoded,),
          path: 'review',
          finalText: TEXT,
          correctionChainRequired: true,
          everyBodyBlockReviewed: true,
          quorumBasisRequired: true,
        },);
        expect(parsed.confirmations?.[0],).toHaveProperty('quorumOver', WIDE_BENCH,);
        expect(parsed.rounds[0],).toHaveProperty('quorumOver', WIDE_BENCH,);
      },
    },),
    it({
      name: 'REFUSES confirmation evidence computed under a different quorum basis',
      fn: async () => {
        const confirmed = await confirmAbsoluteNaturalness(
          requestFor(clientFor({ loseOnChallenge: true, },),),
        );
        expect(function parseChangedConfirmationBasis() {
          parseNaturalnessReview({
            value: {
              correctionCount: 0, corrections: [], rounds: [confirmed.review,],
              confirmations: confirmed.confirmations.map(function alterBasis(round,) {
                return { ...round, quorumOver: ROSTER.length, };
              },),
            },
            path: 'review', finalText: TEXT,
            correctionChainRequired: true, everyBodyBlockReviewed: true, quorumBasisRequired: true,
          },);
        },).toThrow('same requested reviewer roster and quorum basis',);
      },
    },),
    it({
      name: 'MATERIALIZES the default full bench even when the window asks fewer seats',
      fn: async () => {
        /** Remove the wider override to exercise runtime's actual default. */
        const { quorumOver, ...request } = requestFor(clientFor({ loseOnChallenge: true, },),);
        expect(quorumOver,).toBe(WIDE_BENCH,);
        const round = await reviewAbsoluteNaturalness({ ...request, fanOut: 'window', },);
        expect(round,).toHaveProperty('quorumOver', ROSTER.length,);
        expect(round.seats.length,).toBeLessThan(ROSTER.length,);
        const parsed = parseNaturalnessReview({
          value: { correctionCount: 0, corrections: [], rounds: [round,], },
          path: 'review', finalText: TEXT,
          correctionChainRequired: true, everyBodyBlockReviewed: true, quorumBasisRequired: true,
        },);
        expect(parsed.rounds[0],).toHaveProperty('quorumOver', ROSTER.length,);
      },
    },),
    ...[undefined, null, -1, 1.5, 2, Number.NaN, Number.POSITIVE_INFINITY, Number.MAX_SAFE_INTEGER + 1,].map(function invalidBasis(quorumOver,) {
      return it({
        name: `REFUSES malformed or undersized stored quorum ${String(quorumOver,)}`,
        fn: async () => {
          const round = await reviewAbsoluteNaturalness(
            requestFor(clientFor({},),),
          );
          expect(function parseInvalidBasis() {
            parseNaturalnessReview({
              value: { correctionCount: 0, corrections: [], rounds: [{ ...round, quorumOver, },], },
              path: 'review', finalText: TEXT,
              correctionChainRequired: true, everyBodyBlockReviewed: true, quorumBasisRequired: true,
            },);
          },).toThrow('quorumOver',);
        },
      },);
    },),
    ...[-1, 0, 1.5, Number.NaN, Number.POSITIVE_INFINITY, Number.MAX_SAFE_INTEGER + 1,].map(function invalidRuntimeBasis(quorumOver,) {
      return it({
        name: `REFUSES invalid runtime quorum ${String(quorumOver,)} before asking any reviewer`,
        fn: async ctx => {
          const client = clientFor({},);
          const calls = ctx.sinon.spy(client, 'chatJson',);
          let caught: unknown;
          try {
            await reviewAbsoluteNaturalness({ ...requestFor(client,), quorumOver, },);
          }
          catch (error) {
            caught = error;
          }
          expect(caught,).toHaveProperty('name', 'NaturalnessQuorumError',);
          expect(calls,).not.toHaveBeenCalled();
        },
      },);
    },),
    it({
      name: 'REJECTS a stored verdict contradicted by its wider quorum',
      fn: async () => {
        const round = await reviewAbsoluteNaturalness(
          requestFor(clientFor({},),),
        );
        expect(function parseTamperedVerdict() {
          parseNaturalnessReview({
            value: { correctionCount: 0, corrections: [], rounds: [{ ...round, verdict: 'acceptable', },], },
            path: 'review', finalText: TEXT,
            correctionChainRequired: true, everyBodyBlockReviewed: true, quorumBasisRequired: true,
          },);
        },).toThrow('expected quorum-not-met',);
      },
    },),
    it({
      name: 'KEEPS legacy seat-count interpretation without inventing missing provenance',
      fn: async () => {
        const { quorumOver, ...request } = requestFor(clientFor({},),);
        expect(quorumOver,).toBe(WIDE_BENCH,);
        const round = await reviewAbsoluteNaturalness(request,);
        // A ROUND STORED BEFORE PROVENANCE carried neither field: `unreachable`
        // arrived after `quorumOver` (ledger X8), so stripping one alone would
        // build a record no earlier schema wrote.
        const { quorumOver: basis, unreachable, ...legacyRound } = round;
        expect(basis,).toBe(ROSTER.length,);
        expect(unreachable,).toBe(0,);
        const parsed = parseNaturalnessReview({
          value: { correctionCount: 0, corrections: [], rounds: [legacyRound,], },
          path: 'review', finalText: TEXT,
          correctionChainRequired: true, everyBodyBlockReviewed: true,
        },);
        expect(parsed.rounds[0]?.verdict,).toBe('acceptable',);
        expect(parsed.rounds[0],).not.toHaveProperty('quorumOver',);
        expect(parsed.rounds[0],).not.toHaveProperty('unreachable',);
      },
    },),
  ],
},);
