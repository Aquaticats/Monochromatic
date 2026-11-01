import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { HeardVoice, } from './stage-gather.ts';
import {
  silentStageLabel,
  stageQuorumUnmetFinding,
} from './stage-silence.ts';

//region Stage voice lost
// SPLIT OUT OF `stage-quorum.ts` at its line budget: the per-model findings a gather leaves for the voices it asked
// and never heard.
//
// Naming of every model that went quiet, which the ARTIFACT carries and a
// log line does not.
//
// Voice loss reached only `l.warn` before this. That made every question
// about it, which model, which stage, how often, answerable solely from a
// captured run log, and on 2026-08-13 a run spent twenty minutes writing its
// log into a pipe whose reader had exited: the losses happened and nothing
// recorded them. Findings travel into the per-entry artifact, which is
// written durably and survives whatever spawned the pass.
//
// Emitted even when quorum was MET, which is the case the old findings
// dropped entirely and the one that hides a model degrading quietly while
// the stage still looks healthy.
//
// ONE FINDING PER MODEL rather than one naming a list, so counting the
// findings counts voices lost. A list-valued finding counts GATHERS that
// lost at least one voice, which is a different number, and reading the
// first as the second is the mistake that made the earlier per-model tally
// unusable: it summed to 113 mentions over 97 lines and was reported as
// though it were events.

/**
 Findings naming every model that was asked and never answered, one per model, in roster order.

 @param stage - stage the gather ran for, such as `critic`

 @param seated - the roster the stage seated, in roster order

 @param asked - models the gather asked at least once

 @param voices - voices the gather heard

 @returns One `stage-voice-lost (<stage> <model>)` finding per unheard model, empty when every asked model answered

 @example
 ```ts
 const lost = lostVoiceFindings({ stage: 'critic', seated, asked, voices, },);
 ```
 */
export function lostVoiceFindings(
  {
    stage,
    seated,
    asked,
    voices,
  }: {
    readonly stage: string;
    readonly seated: readonly RosterModelId[];
    readonly asked: ReadonlySet<RosterModelId>;
    readonly voices: readonly HeardVoice<unknown>[];
  },
): readonly string[] {
  /**
   Models that were asked and never answered, in roster order.
   */
  const unheard = seated.filter(function askedAndNeverHeard(modelId,): boolean {
    if (!asked.has(modelId,))
      return false;
    return !voices.some(function isVoice(voice,): boolean {
      return voice.modelId === modelId;
    },);
  },);

  return unheard.map(function toFinding(modelId,): string {
    return `stage-voice-lost (${stage} ${modelId})`;
  },);
}

/**
 The finding a roster that ended short leaves beside the per-model ones: quorum unmet when it was, a note that the roster
 was incomplete when quorum was met with a voice missing, nothing when every asked model answered.

 EMITTED WHENEVER THE ROSTER ENDED SHORT, not only when retries were still chasing it. The ratio is the part per-model
 loss findings cannot carry, and a stage that met quorum with a voice missing is exactly the case that reads as healthy
 everywhere else. THE RATIO IS SPELT BY THE ONE LABEL a silent stage carries as data on the settlement
 (`silentStageLabel`), heard against asked, since a seat the window spared was never owed an answer, so the finding and
 the data cannot drift apart.

 @param stage - stage the gather ran for, such as `critic`

 @param quorumMet - whether the gather reached its quorum

 @param voices - voices the gather heard

 @param asked - models the gather asked at least once

 @returns The one shortfall finding, or none

 @example
 ```ts
 const findings = rosterShortfallFindings({ stage: 'critic', quorumMet: false, voices, asked, },);
 ```
 */
export function rosterShortfallFindings(
  {
    stage,
    quorumMet,
    voices,
    asked,
  }: {
    readonly stage: string;
    readonly quorumMet: boolean;
    readonly voices: readonly HeardVoice<unknown>[];
    readonly asked: ReadonlySet<RosterModelId>;
  },
): readonly string[] {
  /**
   Heard against asked.
   */
  const shortfall = silentStageLabel({
    stage,
    heard: voices.length,
    asked: asked.size,
  },);
  if (!quorumMet)
    return [stageQuorumUnmetFinding({ shortfall, },),];
  if (voices.length < asked.size)
    return [`stage-roster-incomplete (${shortfall})`,];
  return [];
}

//endregion Stage voice lost
