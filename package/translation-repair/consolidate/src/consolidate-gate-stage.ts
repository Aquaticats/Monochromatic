import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

import type {
  JsonSchemaResponseFormat,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { contestResponseFormat, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  buildConsolidateGateMessages,
  CONSOLIDATE_GATE_NAMES,
  type ConsolidateGateSubject,
  type GateBallot,
  type GateChoice,
  isConsolidateGateWire,
  readConsolidateGateBallot,
} from './consolidate-gate-wire.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type FanOutMode,
  runWindowedRounds,
  shortBenchStageFinding,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Consolidate gate stage
// DECIDES WHETHER THE RENDERING THIS RUN WROTE REPLACES THE ONE THAT WOULD SHIP.
//
// NO SINGLE MODEL DECIDES, as everywhere else in this package.
//
// THE STANDING TEXT WINS EVERYTHING THAT IS NOT A CLEAR WIN FOR THE
// CONSOLIDATION: a tie, a refusal, and a roster too thin to settle all keep it.
// Changing what a reader sees on a memorial page needs more evidence than
// leaving it, and `translate-wire.ts` already records the reason: a reader who
// knows this archive should not see it churn.
//
// WHAT THE ROSTER SETTLED ON AND WHAT SHIPS ARE REPORTED SEPARATELY, because
// they differ on exactly the cases a later reader will ask about. A slice where
// the roster refused and a slice where it never answered both ship the standing
// text, and only one of them is evidence about the passage.

/**
 Voices that must back the consolidation before it replaces anything.

 TWO, matching the lane contest and every other agreement rule here.
 */
export const CONSOLIDATE_GATE_QUORUM = 2;

/**
 Schema a reply must satisfy before it reaches the reader.
 */
export const CONSOLIDATE_GATE_RESPONSE_FORMAT: JsonSchemaResponseFormat = contestResponseFormat({
  schemaName: 'consolidate_gate',
  verdicts: CONSOLIDATE_GATE_NAMES,
  // FALSE, AND NOT AN OVERSIGHT. This contest's `standing` choice IS the
  // archive, so a judge preferring it has already said the archive is worth
  // keeping. Asking again would record the same opinion twice.
  asksArchive: false,
},);

/**
 Rendering that ships once the gate has answered.
 */
export type GateShipped = 'consolidated' | 'standing';

/**
 What the roster settled for one gated slice.

 @example
 ```ts
 const outcome: ConsolidateGateOutcome = { choice: 'standing', ships: 'standing', ballots: [], usable: 0, findings: [], };
 ```
 */
export type ConsolidateGateOutcome = {
  /**
   What the roster settled on, refusal included.
   */
  readonly choice: GateChoice;

  /**
   Rendering that ships, after the rule that a tie keeps the standing text.
   */
  readonly ships: GateShipped;

  /**
   Every usable ballot, for the audit trail.
   */
  readonly ballots: readonly GateBallot[];

  /**
   Voices whose answer arrived and could be read as a ballot.
   */
  readonly usable: number;

  /**
   What went wrong, in scorecard-stable wording.
   */
  readonly findings: readonly string[];
};

/**
 Counts how many ballots named one rendering.

 @param ballots - usable ballots

 @param name - rendering to count votes for

 @returns Voice count for that rendering

 @example
 ```ts
 const backing = countFor({ ballots, name: 'consolidated', },);
 ```
 */
function countFor(
  {
    ballots,
    name,
  }: {
    readonly ballots: readonly GateBallot[];
    readonly name: GateChoice;
  },
): number {
  /**
   Ballots naming that rendering.
   */
  const named = ballots.filter(function namesIt(ballot,): boolean {
    return ballot.choice === name;
  },);
  return named.length;
}

/**
 Reads what the roster settled on, or the refusal.

 SHARED WITH ANY LATER READER of a stored gate record, on the rule the lane
 contest already follows: a recorded verdict nobody can recompute from the
 ballots beside it can quietly become a lie.

 @param ballots - usable ballots

 @returns Rendering enough voices backed, or the refusal

 @example
 ```ts
 const choice = settleGateBallots({ ballots, },);
 ```
 */
export function settleGateBallots(
  { ballots, }: { readonly ballots: readonly GateBallot[]; },
): GateChoice {
  /**
   Voices backing the rendering this run wrote.
   */
  const consolidated = countFor({
    ballots,
    name: 'consolidated',
  },);

  /**
   Voices backing the rendering that would ship anyway.
   */
  const standing = countFor({
    ballots,
    name: 'standing',
  },);
  if ((consolidated >= CONSOLIDATE_GATE_QUORUM) && (consolidated > standing))
    return 'consolidated';
  if ((standing >= CONSOLIDATE_GATE_QUORUM) && (standing > consolidated))
    return 'standing';
  return 'neither';
}

/**
 Asks the roster whether one consolidation replaces the standing text.

 @param client - synthetic chat client

 @param modelIds - roster to ask

 @param subject - passage, archive rendering and the two renderings

 @param signal - abort shared with the rest of the entry

 @param exchangeTimeoutMs - per-call ceiling

 @param l - logger to tag

 @param fanOut - seats a round asks: the window of quorum plus one by
 default, or the whole bench a fixture scripting every seat asks for

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns What the roster settled, what ships, and every usable ballot

 @example
 ```ts
 const outcome = await gateConsolidatedSlice({ client, modelIds, subject, signal, exchangeTimeoutMs, l, stageSettings, },);
 ```
 */
export async function gateConsolidatedSlice(
  {
    client,
    modelIds,
    subject,
    signal,
    exchangeTimeoutMs,
    l,
    fanOut,
    stageSettings,
  }: {
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly subject: ConsolidateGateSubject;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  },
): Promise<ConsolidateGateOutcome> {
  /**
   Logger naming this stage.
   */
  const gl = tagged({
    l,
    tag: gateConsolidatedSlice.name,
  },);

  /**
   One reply per voice, heard or lost.
   */
  const {
    outcomes,
    quorum,
  } = await runWindowedRounds({
    client,
    modelIds,
    messages: buildConsolidateGateMessages({ subject, },),
    signal,
    exchangeTimeoutMs,
    responseFormat: CONSOLIDATE_GATE_RESPONSE_FORMAT,
    validate: isConsolidateGateWire,
    stage: 'consolidate-gate',
    l: gl,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((fanOut === undefined) ? {} : { fanOut, }),
    graceMs: stageSettings
      .grace
      .roundMs,
  },);

  /**
   Ballots read out of the replies that arrived and validated in shape.
   */
  const ballots = outcomes.flatMap(function toBallot(
    outcome,
  ): readonly GateBallot[] {
    /**
     This voice, heard or lost.
     */
    const { voice, } = outcome;
    if (!voice.heard)
      return [];
    /**
     This voice's ballot, read off the wire.
     */
    const ballot = readConsolidateGateBallot({ wire: voice.value, },);
    // EVERY BALLOT IS LOGGED WITH ITS REASON (class fifty-six, 2026-09-18):
    // XingZ606's gate kept an ineligible standing 4 of 5 and the log carried
    // only the tally, so why four judges preferred a text the rule refuses
    // could not be read afterwards.
    gl.info(`consolidate gate ballot ${outcome.modelId}: ${ballot.choice}: ${ballot.reason}`,);
    return [ballot,];
  },);

  /**
   What the roster settled on.
   */
  const choice = settleGateBallots({ ballots, },);

  /**
   Rendering that ships, after the rule that only a clear win replaces.

   THE QUORUM IS NOT CHECKED TWICE. It used to be repeated here, and removing
   that repetition failed no test, because `settleGateBallots` cannot answer
   `consolidated` on fewer than `CONSOLIDATE_GATE_QUORUM` voices naming it.
   A condition no case can reach reads as a second guard and is not one.
   */
  const ships: GateShipped = (choice === 'consolidated')
    ? 'consolidated'
    : 'standing';
  gl.info(
    `consolidate gate: ${String(ballots.length,)}/${String(outcomes.length,)} usable, settled on ${choice}, ships ${ships}`,
  );
  return {
    choice,
    ships,
    ballots,
    usable: ballots.length,
    findings: [
      ...((ballots.length < CONSOLIDATE_GATE_QUORUM)
        ? [
          `consolidate-gate heard ${String(ballots.length,)} usable ${
            wordForCount({
              count: ballots.length,
              one: 'ballot',
              many: 'ballots',
            },)
          }, below the ${String(CONSOLIDATE_GATE_QUORUM,)} needed to settle`,
        ]
        : []),
      // A SHORT BENCH IS SAID IN THE FINDINGS, as the gathers say it (ledger X8).
      ...(quorum.short
        ? [shortBenchStageFinding({
          stage: 'consolidate-gate',
          quorum,
          benchSize: modelIds.length,
        },),]
        : []),
    ],
  };
}

//endregion Consolidate gate stage
