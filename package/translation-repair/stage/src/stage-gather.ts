import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Stage gather
// What a quorum gather hands back (`gatherStageVoices` in `stage-quorum.ts`):
// the voices it heard and the verdict and findings its stage reads them by.

/**
 One heard voice with its speaker.

 @example
 ```ts
 const voice: HeardVoice<CriticReportWire> = { modelId, value: report, };
 ```
 */
export type HeardVoice<ValueT,> = {
  /**
   Model that answered.
   */
  readonly modelId: RosterModelId;

  /**
   Validated reply value.
   */
  readonly value: ValueT;
};

/**
 Everything a quorum gather produced.

 @example
 ```ts
 const { voices, quorumMet, } = await gatherStageVoices({ ... },);
 ```
 */
export type StageGather<ValueT,> = {
  /**
   Heard voices in arrival-round then roster order.
   */
  readonly voices: readonly HeardVoice<ValueT>[];

  /**
   Whether at least half the roster, rounded up, was heard; on a bench
   short of that for want of wet providers, at least half the reachable
   seats and never fewer than two (`reachableQuorum`, 2026-09-09); on a
   bench whose live seats are exactly its quorum, one voice fewer once the
   grace ends (owner, 2026-10-22, "Let the grace open").
   */
  readonly quorumMet: boolean;

  /**
   Voices the gather closed its round on: the number `quorumMet` compared the
   heard voices with, and the one a short bench's finding names. Handed back
   so a caller that needs it (the checker stage, whose recheck counts an
   issue's ballots against it) never sizes it a second time or reads it back
   out of a finding's wording.
   */
  readonly quorum: number;

  /**
   Degradation findings in scorecard-stable wording: a short bench, each
   asked seat that stayed unheard, and the round's shortfall. Empty where
   every asked seat was heard on a bench no seat of which was refused.
   */
  readonly findings: readonly string[];

  /**
   Seats some round asked, so a caller taking its majority over "everyone
   asked" counts the window, not the bench: a seat the window spared was
   never silent.
   */
  readonly asked: ReadonlySet<RosterModelId>;

  /**
   Seats the router refused because no wet provider serves them, so a
   caller sizing its minimum by the reachable bench (the select stage since
   the owner's decision of 2026-09-09) counts the bench that could answer
   rather than the bench that was seated.
   */
  readonly unreachable: ReadonlySet<RosterModelId>;

  /**
   Seats that answered but nothing could read, still unread after the
   recovery round: a reply the completion cap cut before its content, or a
   shape the guard refused. They were reached and they answered, so a
   caller telling an outage from a bench it could not read counts them as
   answering. On 2026-09-16 Mio13 was interrupted `provider-unavailable`
   at 4 of 12 heard with every provider wet, seven seats having spent
   their whole completion cap reasoning about one chat translation and
   sent no content (class thirty-one).
   */
  readonly unreadable: ReadonlySet<RosterModelId>;
};
//endregion Stage gather
