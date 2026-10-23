import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

import type {
  JsonSchemaResponseFormat,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import {
  CONTEST_REFUSAL,
  contestResponseFormat,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  buildConsolidationPolishGateMessages,
  type ConsolidationPolishBallot,
  type ConsolidationPolishGateSubject,
  isConsolidationPolishGateWire,
  POLISH_NAMES,
  type PolishChoice,
  readConsolidationPolishBallot,
} from './consolidation-polish-gate-wire.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  runWindowedRounds,
  shortBenchStageFinding,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Consolidation polish gate stage

/**
 Voices required before polish replaces approved base.
 */
export const CONSOLIDATION_POLISH_GATE_QUORUM = 2;

/**
 Structured reply contract for naturalness gate.
 */
export const CONSOLIDATION_POLISH_GATE_RESPONSE_FORMAT: JsonSchemaResponseFormat = contestResponseFormat({
  schemaName: 'consolidation_polish_gate',
  verdicts: POLISH_NAMES,
  asksArchive: false,
},);

/**
 Result of final naturalness gate.

 @example
 ```ts
 const outcome: ConsolidationPolishGateOutcome = { choice: 'base', ships: 'base', ballots: [], usable: 0, findings: [], };
 ```
 */
export type ConsolidationPolishGateOutcome = {
  /**
   Panel choice, refusal included.
   */
  readonly choice: PolishChoice;

  /**
   Text role shipping after the tie rule: the polished text wins a tie, and
   the approved base ships every other outcome without a clear polish win.
   */
  readonly ships: 'polished' | 'base';

  /**
   Every usable ballot.
   */
  readonly ballots: readonly ConsolidationPolishBallot[];

  /**
   Number of usable voices.
   */
  readonly usable: number;

  /**
   Stable stage findings.
   */
  readonly findings: readonly string[];
};

/**
 Counts ballots naming candidate.

 @param ballots - usable ballots

 @param choice - candidate to count

 @returns Number naming candidate

 @example
 ```ts
 const votes = polishVotesFor({ ballots, choice: 'polished', });
 ```
 */
function polishVotesFor(
  {
    ballots,
    choice,
  }: {
    readonly ballots: readonly ConsolidationPolishBallot[];
    readonly choice: PolishChoice;
  },
): number {
  /**
   Ballots selecting requested choice.
   */
  const named = ballots.filter(function names(ballot,): boolean {
    return ballot.choice === choice;
  },);
  return named.length;
}

/**
 The rule that settled a polish gate.
 */
export type ConsolidationPolishSettlementRule =
  | 'polished-lead'
    | 'base-lead'
    | 'tie-polished-wins'
    | 'unsettled';

/**
 What the ballots settled, the rule that decided and the tally it read.

 @example
 ```ts
 const settlement: ConsolidationPolishSettlement = { choice: 'base', rule: 'base-lead', polished: 1, base: 2, };
 ```
 */
export type ConsolidationPolishSettlement = {
  /**
   Panel choice, refusal included.
   */
  readonly choice: PolishChoice;

  /**
   Rule that decided the choice.
   */
  readonly rule: ConsolidationPolishSettlementRule;

  /**
   Ballots naming the polished text.
   */
  readonly polished: number;

  /**
   Ballots naming the approved base.
   */
  readonly base: number;
};

/**
 Settles naturalness ballots and names the rule that decided.

 POLISHED TEXT WINS TIES (owner, 2026-10-21). Asked "What should a 2 to 2
 polish-gate tie do?", the owner chose "Polished text wins ties". Before it a
 tie settled on `neither` and the base shipped, so CuspariaKLSY21 slice 1
 (base, base, polished, polished) kept the base. A TIE NEEDS THE MINIMUM IN
 TOTAL: the usable ballots, refusals included, number at least
 {@link CONSOLIDATION_POLISH_GATE_QUORUM}, so a lone vote still settles
 nothing, and it needs a vote to tie on: refusals alone are no tie. Every
 other rule is as it was: a lead needs the minimum on its own side.

 @param ballots - usable ballots

 @returns Choice, deciding rule and the tally the rule read

 @example
 ```ts
 const { choice, rule, } = readConsolidationPolishSettlement({ ballots, });
 ```
 */
export function readConsolidationPolishSettlement(
  { ballots, }: { readonly ballots: readonly ConsolidationPolishBallot[]; },
): ConsolidationPolishSettlement {
  /**
   Votes backing proposed polish.
   */
  const polished = polishVotesFor({
    ballots,
    choice: 'polished',
  },);
  /**
   Votes backing approved base.
   */
  const base = polishVotesFor({
    ballots,
    choice: 'base',
  },);
  if ((polished >= CONSOLIDATION_POLISH_GATE_QUORUM) && (polished > base)) {
    return {
      choice: 'polished',
      rule: 'polished-lead',
      polished,
      base,
    };
  }
  if ((base >= CONSOLIDATION_POLISH_GATE_QUORUM) && (base > polished)) {
    return {
      choice: 'base',
      rule: 'base-lead',
      polished,
      base,
    };
  }
  /**
   Whether the two texts drew the same number of votes, at least one each.
   */
  const tiedOnVotes = (polished === base) && (polished > 0);
  /**
   Whether the usable ballots, refusals included, reach the minimum.
   */
  const meetsMinimum = ballots.length >= CONSOLIDATION_POLISH_GATE_QUORUM;
  if (tiedOnVotes && meetsMinimum) {
    return {
      choice: 'polished',
      rule: 'tie-polished-wins',
      polished,
      base,
    };
  }
  return {
    choice: CONTEST_REFUSAL,
    rule: 'unsettled',
    polished,
    base,
  };
}

/**
 Asks roster whether naturalness polish may replace approved base.

 @param client - shared provider client

 @param modelIds - fidelity and naturalness judges

 @param subject - original and both English candidates

 @param signal - caller cancellation

 @param exchangeTimeoutMs - per-call ceiling

 @param l - parent logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Panel outcome with conservative shipping choice

 @example
 ```ts
 const outcome = await gateConsolidationPolish({ client, modelIds, subject, signal, exchangeTimeoutMs, l, stageSettings, });
 ```
 */
export async function gateConsolidationPolish(
  {
    client,
    modelIds,
    subject,
    signal,
    exchangeTimeoutMs,
    l,
    stageSettings,
  }: {
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly subject: ConsolidationPolishGateSubject;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  },
): Promise<ConsolidationPolishGateOutcome> {
  /**
   Logger naming final naturalness gate.
   */
  const gl = tagged({
    l,
    tag: gateConsolidationPolish.name,
  },);
  /**
   One outcome per requested voice.
   */
  const {
    outcomes,
    quorum,
  } = await runWindowedRounds({
    client,
    modelIds,
    messages: buildConsolidationPolishGateMessages({ subject, },),
    signal,
    exchangeTimeoutMs,
    responseFormat: CONSOLIDATION_POLISH_GATE_RESPONSE_FORMAT,
    validate: isConsolidationPolishGateWire,
    stage: 'consolidation-polish-gate',
    l: gl,
    graceMs: stageSettings
      .grace
      .roundMs,
  },);
  /**
   Ballots read from usable voices.
   */
  const ballots = outcomes.flatMap(function toBallot(
    outcome,
  ): readonly ConsolidationPolishBallot[] {
    /**
     Voice heard or lost.
     */
    const { voice, } = outcome;
    if (!voice.heard)
      return [];
    /**
     This voice's ballot, read off the wire.
     */
    const ballot = readConsolidationPolishBallot({ wire: voice.value, },);
    // EVERY BALLOT IS LOGGED WITH ITS REASON, as the consolidate gate's are
    // (class fifty-six): shi_Yumiaoya36's polish gate settled on neither over
    // a polish that restored a survived suicide attempt the base had dropped
    // (class one hundred fifty), and the log carried only the tally.
    gl.info(`consolidation polish gate ballot ${outcome.modelId}: ${ballot.choice}: ${ballot.reason}`,);
    return [ballot,];
  },);
  /**
   Panel choice from ballots, the rule that decided and the tally it read.
   */
  const {
    choice,
    rule,
    polished,
    base,
  } = readConsolidationPolishSettlement({ ballots, },);
  /**
   Approved base wins every outcome without a polished choice.
   */
  const ships = (choice === 'polished') ? 'polished' as const : 'base' as const;
  gl.info(
    `consolidation polish gate: ${String(ballots.length,)}/${String(outcomes.length,)} usable, polished ${
      String(polished,)
    }, base ${String(base,)}, settled on ${choice} by rule ${rule}, ships ${ships}`,
  );
  if (rule === 'tie-polished-wins') {
    gl.info(
      `consolidation polish gate tied ${String(polished,)} to ${String(base,)} between polished and base `
        + `(usable ballots counted: ${String(ballots.length,)}, minimum ${String(CONSOLIDATION_POLISH_GATE_QUORUM,)}), `
        + 'so polished wins the tie (owner, 2026-10-21, "Polished text wins ties")',
    );
  }
  return {
    choice,
    ships,
    ballots,
    usable: ballots.length,
    findings: [
      ...((ballots.length < CONSOLIDATION_POLISH_GATE_QUORUM)
        ? [
          `consolidation-polish-gate heard ${String(ballots.length,)} usable ${
            wordForCount({
              count: ballots.length,
              one: 'ballot',
              many: 'ballots',
            },)
          }, below ${String(CONSOLIDATION_POLISH_GATE_QUORUM,)} needed to settle`,
        ]
        : []),
      // A TIE IS SAID IN THE FINDINGS, as the rule that shipped the polished
      // text, so the artifact tells a tie win from a lead.
      ...((rule === 'tie-polished-wins')
        ? [
          `consolidation-polish-gate tied ${String(polished,)} to ${String(base,)} between polished and base, `
            + 'so polished wins the tie (owner, 2026-10-21)',
        ]
        : []),
      // A SHORT BENCH IS SAID IN THE FINDINGS, as the gathers say it (ledger X8).
      ...(quorum.short
        ? [shortBenchStageFinding({
          stage: 'consolidation-polish-gate',
          quorum,
          benchSize: modelIds.length,
        },),]
        : []),
    ],
  };
}

//endregion Consolidation polish gate stage
