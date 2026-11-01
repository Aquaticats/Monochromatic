/**
 DSH default-reviewer ranking policy for the shared selection seam:
 largest context window first, then model name, then canonical slug. @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type {
  AdvisorDefaultRanking,
  AdvisorRankingCandidate,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import { dshAdvisorLogger, } from './logging.ts';

//region Public API

/**
 Rank default-reviewer candidates for the DSH variant.

 The shared core ranks by expected call cost; the DSH variant instead prefers
 the route with the largest advertised context window (then `name`, then
 canonical slug) because DSH routes carry no text pricing and the widest
 context is the most likely to hold the full conversation. The reason names
 only the winning window, because it is rendered into model-visible tool
 output; the full observed-window roster goes to the debug decision log so a
 verdict stays reconstructable without flooding any context.

 @param candidates - eligible scoped candidates with their estimated input
 tokens

 @param maxOutputTokens - request output budget shared by every candidate

 @returns candidates ordered best-first with a winner-only window reason

 @example
 ```ts
 const ranking = dshRankDefaultCandidates({ candidates, maxOutputTokens: 4096 });
 ```
 */
export function dshRankDefaultCandidates(
  {
    candidates,
    maxOutputTokens,
  }: {
    readonly candidates: readonly AdvisorRankingCandidate[];
    readonly maxOutputTokens: number;
  },
): AdvisorDefaultRanking {
  void maxOutputTokens;
  /**
   Logger tagged with the ranking policy name.
   */
  const l = tagged({
    tag: dshRankDefaultCandidates.name,
    l: dshAdvisorLogger,
  },);
  /**
   Candidates ordered by largest context window, then name, then slug.
   */
  const ranked = [...candidates,]
    .toSorted(function byWindowNameSlug(
      a,
      b,
    ) {
    /**
     Context-window comparison, widest first.
     */
      const windowDelta = b
        .scopedModel
        .model
        .contextWindow
        - a
          .scopedModel
          .model
          .contextWindow;
      if (windowDelta
        !== 0)
        return windowDelta;
    /**
     Name comparison, lexicographic ascending.
     */
      const nameOrder = a
        .scopedModel
        .model
        .name
        .localeCompare(b
          .scopedModel
          .model
          .name,);
      if (nameOrder
        !== 0)
        return nameOrder;
      return a
        .scopedModel
        .canonicalSlug
        .localeCompare(b
          .scopedModel
          .canonicalSlug,);
    },)
    .map(function toScopedModel(candidate,) {
      return candidate.scopedModel;
    },);
  /**
   Observed context windows in ranked order, recorded as debug decision
   evidence instead of surfaced text.
   */
  const windowEvidence = ranked
    .map(function toWindowEvidence(entry,) {
      return `${entry.canonicalSlug}=${String(entry.model
        .contextWindow,)}`;
    },)
    .join(', ',);
  l.debug(`observed windows: ${windowEvidence}`,);
  /**
   Top-ranked scoped model whose window the surfaced reason reports.
   */
  const [topEntry,] = ranked;
  /**
   Policy statement plus the winning window only; the roster stays in the
   debug decision log.
   */
  const reason = topEntry === undefined
    ? 'dsh default ranking: largest context window first, then name, then slug; no eligible candidates'
    : `dsh default ranking: largest context window first, then name, then slug; widest observed window ${topEntry
      .canonicalSlug}=${String(topEntry
        .model
        .contextWindow,)}`;
  return {
    ranked,
    reason,
  };
}

//endregion Public API
