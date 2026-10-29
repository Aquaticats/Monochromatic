/**
 Tests that every settlement the real consolidation stage writes goes through
 the store's rules end to end.

 WHAT THIS FILE EXISTS TO STOP. The store (`consolidate-cache-store.ts`) holds
 rules written from a reading of the stage's writers, and its own cases feed it
 hand-built settlements. A rule that refuses a settlement the stage really
 writes costs a re-bought slate and gate per slice per run and errors nowhere,
 so these cases drive `settleConsolidation` through `persistConsolidationSettlement`
 into a store and resume it through a second one, for every way the stage can
 leave a slice.

 Fixtures are cat-themed invention written into throwaway directories.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ConsolidationSettlement,
  persistConsolidationSettlement,
  type ProposalValidity,
  settleConsolidation,
} from '@monochromatic-dev/translation-repair-consolidate';
import { createSyntheticClient, } from '@monochromatic-dev/translation-repair-provider';
import {
  buildTranslateCandidates,
  describeSlate,
  type LaneText,
  rotateCandidates,
} from '@monochromatic-dev/translation-repair-translate-floor';
import { openConsolidateCache, } from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { ballot, } from '@monochromatic-dev/translation-repair-test-fixture/ts/structural-ballot.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Roster of three, the smallest that can produce a two-to-one split.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Built pipeline the stores are filled under.
 */
const TEST_GENERATION = `sha256-tree-v1:${'b'.repeat(64,)}`;

/**
 Key every case writes and reads under.
 */
const CAT_KEY = 'd'.repeat(64,);

/**
 Original the slice renders.
 */
const SOURCE_TEXT = '猫在窗边睡着了。她四点醒来。';

/**
 Wording in place when the stage begins.
 */
const STANDING = 'The cat fell asleep by the window.\nShe woke at four.';

/**
 A consolidation that differs from what stands.
 */
const FRESH = 'The cat fell asleep beside the window. She woke at four in the afternoon.';

/**
 Phrase separating the gate's sheet from the slate judge's.
 */
const GATE_MARKER = 'Return JSON: assessment one sentence';

/**
 What a scenario has the slate judges do.
 */
type JudgeWish = 'decline' | 'standing' | 'fresh';

/**
 One way a slice can leave the stage, as the scripted roster produces it.
 */
type Scenario = {
  /**
   Proposals reaching the stage, in roster order.
   */
  readonly proposals: readonly {
    readonly translation: string;
    readonly valid: boolean;
  }[];

  /**
   Wording in place.
   */
  readonly standingText: string;

  /**
   Whether the standing passed the deterministic gate.
   */
  readonly standingEligible: boolean;

  /**
   Whom the slate judges back.
   */
  readonly judge: JudgeWish;

  /**
   What every gate voice answers, or that the gate is unreachable.
   */
  readonly gate: 'standing' | 'consolidated' | 'unreachable';

  /**
   Lane texts offered on the slate beside the proposals, none when absent.
   */
  readonly laneTexts?: readonly LaneText[];
};

/**
 Every way the stage can leave a slice that the store holds a rule for, keyed
 by what the case reaches.
 */
const SCENARIOS: Readonly<Record<string, Scenario>> = {
  'floor refused every proposal': {
    proposals: [{ translation: FRESH, valid: false, },],
    standingText: STANDING,
    standingEligible: true,
    judge: 'decline',
    gate: 'unreachable',
  },
  'no standing text': {
    proposals: [],
    standingText: '',
    standingEligible: true,
    judge: 'decline',
    gate: 'unreachable',
  },
  'judges endorse the standing': {
    proposals: [{ translation: FRESH, valid: true, },],
    standingText: STANDING,
    standingEligible: true,
    judge: 'standing',
    gate: 'unreachable',
  },
  'slate holds only the standing': {
    proposals: [{ translation: STANDING, valid: true, },],
    standingText: STANDING,
    standingEligible: true,
    judge: 'decline',
    gate: 'unreachable',
  },
  'judges decline the slate': {
    proposals: [{ translation: FRESH, valid: true, },],
    standingText: STANDING,
    standingEligible: true,
    judge: 'decline',
    gate: 'unreachable',
  },
  'gate keeps the standing': {
    proposals: [{ translation: FRESH, valid: true, },],
    standingText: STANDING,
    standingEligible: true,
    judge: 'fresh',
    gate: 'standing',
  },
  'gate ships the consolidation': {
    proposals: [{ translation: FRESH, valid: true, },],
    standingText: STANDING,
    standingEligible: true,
    judge: 'fresh',
    gate: 'consolidated',
  },
  'wrap erases the only difference over a refused standing': {
    proposals: [{ translation: STANDING, valid: true, },],
    standingText: STANDING,
    standingEligible: false,
    judge: 'fresh',
    gate: 'consolidated',
  },
  'archive kept over a refused standing': {
    proposals: [{ translation: FRESH, valid: false, },],
    standingText: STANDING,
    standingEligible: false,
    judge: 'decline',
    gate: 'unreachable',
  },
  'gate heard nobody': {
    proposals: [{ translation: FRESH, valid: true, },],
    standingText: STANDING,
    standingEligible: true,
    judge: 'fresh',
    gate: 'unreachable',
  },
  'no proposer was heard and nothing shows on the slate': {
    proposals: [],
    standingText: ' ',
    standingEligible: true,
    judge: 'decline',
    gate: 'unreachable',
    laneTexts: [{ lane: 'repair', text: '', },],
  },
};

/**
 Proposals as the produce half hands them over, one voice per seat in roster order.

 @param proposals - wordings the scenario proposes

 @returns Voices the stage settles over

 @example
 ```ts
 const voices = voicesOf({ proposals: [{ translation: FRESH, valid: true, },], },);
 ```
 */
function voicesOf(
  { proposals, }: { readonly proposals: Scenario['proposals']; },
) {
  return proposals.map(function toVoice(proposal, at,) {
    return {
      modelId: ROSTER[at] ?? ROSTER[0],
      value: { translation: proposal.translation, },
    };
  },);
}

/**
 Ballot index of one wording on the slate the judges are shown.

 @param scenario - case being driven

 @param wanted - wording whose ballot number is sought

 @returns One-based ballot index, or zero to decline

 @example
 ```ts
 const best = ballotIndexOf({ scenario, wanted: FRESH, },);
 ```
 */
function ballotIndexOf(
  {
    scenario,
    wanted,
  }: {
    readonly scenario: Scenario;
    readonly wanted: string;
  },
): number {
  /**
   Slate as the stage builds it for the judges.
   */
  const built = buildTranslateCandidates({
    voices: voicesOf({
      proposals: scenario.proposals
        .filter(function isValid(proposal,): boolean {
          return proposal.valid;
        },),
    },),
    translatorModelIds: ROSTER,
    incumbentText: scenario.standingEligible ? scenario.standingText : '',
    lineStructured: false,
    laneTexts: [],
  },);
  return describeSlate({
    candidates: rotateCandidates({
      candidates: built.candidates,
      sourceText: SOURCE_TEXT,
    },),
  },).find(function carries(entry,): boolean {
    return entry.text === wanted;
  },)?.index ?? 0;
}

/**
 Looks one scenario up by name.

 @param name - key into the scenario table

 @returns Scenario

 @throws Error when the table holds no such name, which a case spelling its own key wrongly causes

 @example
 ```ts
 const scenario = scenarioNamed({ name: 'gate keeps the standing', },);
 ```
 */
function scenarioNamed({ name, }: { readonly name: string; },): Scenario {
  for (const [key, scenario,] of Object.entries(SCENARIOS,))
    if (key === name)
      return scenario;
  throw new Error(`unreachable: the scenario table holds no case named ${name}`,);
}

/**
 Runs the real stage over one scenario and returns what it settled.

 @param name - key of the case being driven

 @returns Stage's settlement

 @example
 ```ts
 const settled = await settleScenario({ name: 'gate keeps the standing', },);
 ```
 */
async function settleScenario(
  { name, }: { readonly name: string; },
): Promise<ConsolidationSettlement> {
  /**
   The case being driven.
   */
  const scenario = scenarioNamed({ name, },);

  /**
   Body every slate judge returns.
   */
  const judgeBody = JSON.stringify({
    best: (scenario.judge === 'decline')
      ? 0
      : ballotIndexOf({
        scenario,
        wanted: (scenario.judge === 'standing') ? scenario.standingText : (scenario.proposals[0]?.translation ?? ''),
      },),
    assessment: 'it says what the original says',
  },);

  /**
   Proposals as the produce half hands them over.
   */
  const voices = voicesOf({ proposals: scenario.proposals, },);

  /**
   What the structural guard made of each.
   */
  const validity: ProposalValidity[] = scenario.proposals
    .map(function toVerdict(proposal, at,): ProposalValidity {
      return {
        modelId: ROSTER[at] ?? ROSTER[0],
        validation: proposal.valid
          ? { kind: 'valid', pageGrammar: 'strict', }
          : { kind: 'invalid', findings: ['The page as it stands is 2 blocks and your rendering is 1.',], },
      };
    },);
  return await settleConsolidation({
    standingMayShip: true,
    client: createSyntheticClient({
      apiKey: 'test-key',
      retryPolicy: {
        limit: 0,
        baseMs: 1,
      },
      transport: async function routingTransport(exchange,) {
        if (!JSON.stringify(exchange,).includes(GATE_MARKER,))
          return { status: 200, bodyText: streamOf({ content: judgeBody, }), };
        if (scenario.gate === 'unreachable')
          return { status: 500, bodyText: 'no gate voice reachable', };
        return { status: 200, bodyText: streamOf({ content: ballot({ choice: scenario.gate, },), }), };
      },
    },),
    roster: ROSTER,
    subject: {
      sourceText: SOURCE_TEXT,
      incumbentText: STANDING,
    },
    voices,
    validity,
    producedFindings: [],
    standingText: scenario.standingText,
    lineStructured: false,
    standingEligible: scenario.standingEligible,
    ...((scenario.laneTexts === undefined) ? {} : { laneTexts: scenario.laneTexts, }),
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l: capturingLogger({ messages: [], },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

/**
 Frames one reply body as the stream the transport returns.

 @param content - reply body

 @returns Event stream text

 @example
 ```ts
 const bodyText = streamOf({ content: '{"best":1}', },);
 ```
 */
function streamOf({ content, }: { readonly content: string; },): string {
  return `data: ${JSON.stringify({ choices: [{ index: 0, delta: { content, }, },], },)}\n\ndata: [DONE]\n\n`;
}

/**
 What came of one settlement written and resumed.

 @example
 ```ts
 const outcome: RoundTrip = { terminal: 'consolidated', written: true, resumedAsWritten: true, };
 ```
 */
type RoundTrip = {
  /**
   Terminal the stage left the slice in.
   */
  readonly terminal: string;

  /**
   Whether `persistConsolidationSettlement` wrote it.
   */
  readonly written: boolean;

  /**
   Whether a second store resumed exactly what the stage returned.
   */
  readonly resumedAsWritten: boolean;
};

/**
 Settles one scenario, persists it, and resumes it through a second store.

 @param name - key into the scenario table

 @returns Terminal, whether it was written, and whether it came back whole

 @example
 ```ts
 const outcome = await roundTripScenario({ name: 'gate ships the consolidation', },);
 ```
 */
async function roundTripScenario(
  { name, }: { readonly name: string; },
): Promise<RoundTrip> {
  await using scratch = await scratchDir({ prefix: 'whiskers-settlement-', },);

  /**
   What the real stage settled.
   */
  const settlement = await settleScenario({ name, },);

  /**
   Whether the persistence rule wrote it.
   */
  const written = await persistConsolidationSettlement({
    key: CAT_KEY,
    settlement,
    cache: await openConsolidateCache({
      dir: scratch.path,
      generation: TEST_GENERATION,
    },),
    standingMayShip: true,
    signal: new AbortController().signal,
  },);

  /**
   Store a later run would resume through.
   */
  const reading = await openConsolidateCache({
    dir: scratch.path,
    generation: TEST_GENERATION,
  },);
  return {
    terminal: settlement.terminal,
    written,
    resumedAsWritten: JSON.stringify(reading.resumed
      .get(CAT_KEY,),) === JSON.stringify(settlement,),
  };
}

/**
 Writes a settlement straight into a store, past the persistence rule, and
 reports whether a second store resumes it.

 @param settlement - what the stage returned

 @returns Whether the second store resumed anything under the key

 @example
 ```ts
 const resumed = await resumedPastTheRule({ settlement, },);
 ```
 */
async function resumedPastTheRule(
  { settlement, }: { readonly settlement: ConsolidationSettlement; },
): Promise<boolean> {
  await using scratch = await scratchDir({ prefix: 'whiskers-settlement-', },);
  await (await openConsolidateCache({
    dir: scratch.path,
    generation: TEST_GENERATION,
  },)).persist({
    key: CAT_KEY,
    serialized: JSON.stringify(settlement,),
  },);
  return (await openConsolidateCache({
    dir: scratch.path,
    generation: TEST_GENERATION,
  },)).resumed
    .has(CAT_KEY,);
}

await describe({
  name: persistConsolidationSettlement.name,
  concurrency: 1,
  children: [
    it({
      name: 'WRITES AND RESUMES WHOLE every settlement the real stage leaves worth keeping, and writes none '
        + 'of the rest: the store refused nothing the persistence rule wrote',
      fn: async () => {
        /**
         Outcome per scenario, in table order.
         */
        const outcomes = Object.fromEntries(
          await Promise.all(Object.keys(SCENARIOS,)
            .map(async function toOutcome(name,): Promise<[string, RoundTrip,]> {
              return [name, await roundTripScenario({ name, },),];
            },),),
        );
        expect(outcomes,).toStrictEqual({
          'floor refused every proposal': { terminal: 'incumbent-only', written: true, resumedAsWritten: true, },
          'no standing text': { terminal: 'no-standing-text', written: true, resumedAsWritten: true, },
          'judges endorse the standing': {
            terminal: 'slate-endorsed-standing',
            written: true,
            resumedAsWritten: true,
          },
          'slate holds only the standing': {
            terminal: 'slate-unjudged-standing',
            written: true,
            resumedAsWritten: true,
          },
          'judges decline the slate': {
            terminal: 'slate-declined-standing',
            written: false,
            resumedAsWritten: false,
          },
          'gate keeps the standing': { terminal: 'gate-kept-standing', written: true, resumedAsWritten: true, },
          'gate ships the consolidation': { terminal: 'consolidated', written: true, resumedAsWritten: true, },
          'wrap erases the only difference over a refused standing': {
            terminal: 'wrap-erased-difference',
            written: false,
            resumedAsWritten: false,
          },
          'archive kept over a refused standing': {
            terminal: 'incumbent-only',
            written: false,
            resumedAsWritten: false,
          },
          'gate heard nobody': { terminal: 'gate-kept-standing', written: false, resumedAsWritten: false, },
          'no proposer was heard and nothing shows on the slate': {
            terminal: 'slate-unjudged-standing',
            written: false,
            resumedAsWritten: false,
          },
        },);
      },
    },),

    it({
      name: 'REFUSES TO RESUME A GATE THAT HEARD TOO FEW VOICES, the settlement the real stage returns when no gate '
        + 'voice answers and which the persistence rule never writes, since a short gate is the hour and not the slice',
      fn: async () => {
        /**
         What the real stage returns when the gate is unreachable.
         */
        const settlement = await settleScenario({ name: 'gate heard nobody', },);
        expect({
          terminal: settlement.terminal,
          usable: settlement.gate?.usable,
          resumed: await resumedPastTheRule({ settlement, },),
        },).toStrictEqual({
          terminal: 'gate-kept-standing',
          usable: 0,
          resumed: false,
        },);
      },
    },),
  ],
},);
