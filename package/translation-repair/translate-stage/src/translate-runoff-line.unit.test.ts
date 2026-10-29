/**
 Guards ledger B806: the line that announces a run-off over the finalists of an
 undecided round says which round that was. A plurality (the leader named by
 one judge alone) is not a tie, and the line said "tied finalists" for both.

 WHY: in a corpus pass of 2026-10-21 (UTC) the line `the same panel is asked
 again over the 2 tied finalists` was printed for a slice whose leader drew 1.5
 against a minimum of 2 and whose runner-up drew 1, so no two candidates were
 tied. `runoffFinalists` picked the leader and its runner-up and handed on only
 the two candidates, and the line could not tell the case from a tie. That
 leader would be decided on the first round since 2026-10-25, when the owner
 removed the weight minimum ("Every weight floor"): two ballots decide.

 WHAT THESE PIN: the decline of a round carries the basis of its run-off beside
 the finalists (`tied-at-top` or `leader-and-runner-up`), and the run-off's line
 names the basis it was given. The plurality is built from a judge who wrote one
 of the candidates (its ballot for its own text counts half) backing it, a second
 judge naming another candidate at 1, and a third abstaining: the second
 candidate leads at 1 from one ballot and the first draws a half.

 Fixtures are cat-themed invention. No call leaves the process.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { producerModelIds, } from '@monochromatic-dev/translation-repair-house-rule';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  judgeTranslateSlate,
  produceTranslateSlate,
  runoffTiedFinalists,
  type TranslateStageResult,
} from '../dist/final/node/index.mjs';
import { candidateCarrying, } from '@monochromatic-dev/translation-repair-test-fixture/ts/translate-ballot.test-fixture.ts';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Schema name the producing half asks translators for; every other structured
 ask is a judge sheet.
 */
const TRANSLATE_SCHEMA = 'translation_report';

/**
 Original slice.
 */
const SOURCE_TEXT = '猫猫在窗台上打盹，尾巴垂在暖气片旁边。';

/**
 Translation already in the archive, kept by a decline.
 */
const INCUMBENT_TEXT = 'The cat is doing the sleeping on the windowsill, with tail hanging by the radiator.';

/**
 Translator whose text the interested judge is.
 */
const CAT_A = 'hf:cat/Cat-A' as unknown as RosterModelId;

/**
 Four translators, one rendering each in call order.
 */
const TRANSLATORS: readonly RosterModelId[] = [
  CAT_A,
  'hf:cat/Cat-B' as unknown as RosterModelId,
  'hf:cat/Cat-D' as unknown as RosterModelId,
  'hf:cat/Cat-E' as unknown as RosterModelId,
];

/**
 Judges: the first wrote one of the candidates, the other two wrote none.
 */
const JUDGES: readonly RosterModelId[] = [
  CAT_A,
  'hf:cat/Cat-F' as unknown as RosterModelId,
  'hf:cat/Cat-G' as unknown as RosterModelId,
];

/**
 The word each rendering carries and no other does.
 */
const WORDS: readonly string[] = ['dozes', 'naps', 'curls', 'yawns',];

/**
 What the translators render, one each in call order.
 */
const RENDERINGS: readonly string[] = [
  'The cat dozes on the windowsill, tail draped beside the radiator.',
  'A cat naps on the sill, its tail hanging near the heater.',
  'The cat curls up on the ledge, tail tucked by the warmth.',
  'The cat yawns on the sill, tail resting against the pipe.',
];

/**
 What one judge says in a scenario: the interested judge's own text, another candidate, or nothing.
 */
type Vote = 'own' | 'other' | 'none';

/**
 How the three judges vote in a scenario, in the order of {@link JUDGES}.
 */
type Scenario = {
  /**
   One vote per judge.
   */
  readonly votes: readonly [Vote, Vote, Vote];
};

/**
 The first judge, who wrote one of the candidates, rejects everything; the others name the interested judge's text and
 another: two candidates at one ballot each, a tie.
 */
const TIE: Scenario = { votes: ['none', 'own', 'other',], };

/**
 The first judge backs its own text, the second names it too, the third another: the leader draws 1.5 from two ballots
 against a runner-up at 1, which two ballots decide.
 */
const PLURALITY_OF_TWO: Scenario = { votes: ['own', 'own', 'other',], };

/**
 The first judge backs its own text (half a vote), the second names another candidate (a full vote), the third
 abstains: the leader at 1 is strictly ahead of its runner-up at a half and was named by one judge alone.
 */
const LONE_LEADER: Scenario = { votes: ['own', 'other', 'none',], };

/**
 Client whose translators render in call order and whose judges answer per the scenario.

 @param scenario - how the judges vote

 @param ownWord - word of the candidate the interested judge wrote, known once the slate exists

 @returns Client

 @example
 ```ts
 const client = scriptedClient({ scenario, ownWord, },);
 ```
 */
function scriptedClient(
  {
    scenario,
    ownWord,
  }: {
    readonly scenario: Scenario;
    readonly ownWord: { value: string; };
  },
): ModelClient {
  /**
   Translator calls served so far.
   */
  const served = { count: 0, };
  return {
    chatText: async () => {
      throw new Error('chatText unused by the translate lane',);
    },
    quotas: async () => {
      throw new Error('quotas unused by the translate lane',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Which sheet this is.
       */
      const schema = request.responseFormat
        ?.json_schema
        .name;
      if (schema === TRANSLATE_SCHEMA) {
        const translation = RENDERINGS[served.count % RENDERINGS.length] ?? '';
        served.count += 1;
        const value: unknown = { translation, };
        if (!request.validate(value,))
          throw new Error('the fixture translation failed the wire guard',);
        return {
          kind: 'ok',
          value,
          rawText: JSON.stringify(value,),
        };
      }
      /**
       Sheet text, for finding the wanted candidate.
       */
      const content = request.messages
        .map(function toContent(message,) {
          return messageText({ message, },);
        },)
        .join('\n',);
      /**
       Word of a candidate other than the interested judge's own.
       */
      const otherWord = WORDS.find(function isOther(word,): boolean {
        return word !== ownWord.value;
      },) ?? '';
      /**
       What this seat says in the scenario.
       */
      const vote = scenario.votes[JUDGES.indexOf(request.modelId,)] ?? 'none';
      /**
       Word of the candidate this seat names, when it names one.
       */
      const needle = (vote === 'other') ? otherWord : ownWord.value;
      const ballot: unknown = {
        assessment: 'fixture',
        best: (vote === 'none')
          ? 0
          : candidateCarrying({
            content,
            needle,
          },),
      };
      if (!request.validate(ballot,))
        throw new Error('the fixture ballot failed the wire guard',);
      return {
        kind: 'ok',
        value: ballot as ValueT,
        rawText: JSON.stringify(ballot,),
      };
    },
  };
}

/**
 Judges one slate over an archived incumbent, keeping the lines the stage logged.

 @param scenario - how the judges vote

 @returns What the judging was handed, the first round's result and the lines logged so far

 @example
 ```ts
 const { first, } = await judgeFirstRound({ scenario: TIE, },);
 ```
 */
async function judgeFirstRound(
  { scenario, }: { readonly scenario: Scenario; },
): Promise<{
  readonly judging: Parameters<typeof judgeTranslateSlate>[0];
  readonly first: TranslateStageResult;
  readonly lines: string[];
}> {
  /**
   Lines the stage logged.
   */
  const lines: string[] = [];
  /**
   Logger keeping them.
   */
  const l = capturingLogger({ messages: lines, },);
  /**
   Word of the interested judge's own rendering, set once the slate exists.
   */
  const ownWord = { value: '', };
  const client = scriptedClient({ scenario, ownWord, },);
  const produced = await produceTranslateSlate({
    client,
    translatorModelIds: TRANSLATORS,
    sourceText: SOURCE_TEXT,
    incumbentText: INCUMBENT_TEXT,
    lineStructured: false,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    // The scenarios read the interested judge's own rendering, so every translator is asked: a window of quorum plus one
    // translators, rotated by the prompt text, may leave that translator out and the judge with no rendering of its own
    // (ledger B701).
    fanOut: 'whole-bench',
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  /**
   The candidate the interested judge wrote.
   */
  const own = produced.candidates.find(function isOwn(candidate,): boolean {
    return producerModelIds(candidate.producer,).includes(CAT_A,);
  },);
  ownWord.value = WORDS.find(function carriedBy(word,): boolean {
    return (own?.rendered ?? '').includes(word,);
  },) ?? '';
  /**
   Everything the judging is handed.
   */
  const judging = {
    client,
    produced,
    judgeModelIds: JUDGES,
    sourceText: SOURCE_TEXT,
    incumbentText: INCUMBENT_TEXT,
    incumbentKind: 'present' as const,
    lineStructured: false,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    // Every judge is asked for the same reason: the scenarios script each judge's ballot, and a rotated window may leave
    // the declining or the interested judge out.
    fanOut: 'whole-bench' as const,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  };
  const first = await judgeTranslateSlate(judging,);
  return {
    judging,
    first,
    lines,
  };
}

/**
 Judges one slate over an archived incumbent and runs off its finalists, keeping
 the lines the stage logged.

 @param scenario - how the judges vote

 @returns First round's decline and lines logged while the run-off was announced

 @example
 ```ts
 const { first, lines, } = await declinedThenRunoff({ scenario: TIE, },);
 ```
 */
async function declinedThenRunoff(
  { scenario, }: { readonly scenario: Scenario; },
): Promise<{
  readonly first: TranslateStageResult;
  readonly lines: readonly string[];
}> {
  const {
    judging,
    first,
    lines,
  } = await judgeFirstRound({ scenario, },);
  /**
   Lines logged before the run-off is announced.
   */
  const before = lines.length;
  await runoffTiedFinalists({
    judging,
    first,
    finalists: first.runoffFinalists ?? [],
    basis: first.runoffBasis ?? 'tied-at-top',
  },);
  return {
    first,
    lines: lines.slice(before,),
  };
}

/**
 Weights the candidates drew, lowest first, leaving out the candidates nobody named.

 @param first - decline whose per-candidate record is read

 @returns Positive weights in ascending order

 @example
 ```ts
 const weights = drawnWeightsOf({ first, },); // [1, 1.5]
 ```
 */
function drawnWeightsOf({ first, }: { readonly first: TranslateStageResult; },): readonly number[] {
  return first.perCandidate
    .map(function weightOf(drawn,): number {
      return drawn.weight;
    },)
    .filter(function drewSomething(weight,): boolean {
      return weight > 0;
    },)
    .toSorted(function ascending(left, right,): number {
      return left - right;
    },);
}

await describe({
  name: 'the run-off announcement names the round it follows (ledger B806)',
  concurrency: 1,
  children: [
    it({
      name: 'CALLS A TIE A TIE: two candidates at one ballot each, a third judge declining, are the tied finalists',
      fn: async () => {
        const { first, lines, } = await declinedThenRunoff({ scenario: TIE, },);
        // The control: this is the shape the old line described.
        expect(first.decision,).toBe('declined-indecision',);
        expect(drawnWeightsOf({ first, },),).toEqual([1, 1,],);
        expect(first.runoffFinalists?.length,).toBe(2,);
        expect(first.runoffBasis,).toBe('tied-at-top',);
        /**
         The line announcing the run-off.
         */
        const line = lines.find(function announces(entry,): boolean {
          return entry.includes('asked again over',);
        },);
        expect(line,).toContain('asked again over the 2 tied finalists',);
        expect(line,).not.toContain('runner-up',);
      },
    },),

    it({
      name: 'DOES NOT CALL A PLURALITY A TIE: a leader at 1 from one ballot against a runner-up at a half are the '
        + 'leader and its runner-up, and the leader was named by one judge alone',
      fn: async () => {
        const { first, lines, } = await declinedThenRunoff({ scenario: LONE_LEADER, },);
        // The control: a plurality of one judge, and no tie.
        expect(first.decision,).toBe('declined-indecision',);
        expect(drawnWeightsOf({ first, },),).toEqual([0.5, 1,],);
        expect(first.runoffFinalists?.length,).toBe(2,);
        expect(first.runoffBasis,).toBe('leader-and-runner-up',);
        /**
         The line announcing the run-off.
         */
        const line = lines.find(function announces(entry,): boolean {
          return entry.includes('asked again over',);
        },);
        expect(line,).toContain('asked again over the leader and its runner-up',);
        expect(line,).toContain('the leader was named by one judge alone',);
        expect(line,).not.toContain('tied',);
      },
    },),

    it({
      name: 'DECIDES A PLURALITY OF TWO BALLOTS: a leader at 1.5 from two ballots against a runner-up at 1 wins the '
        + 'first round, so no run-off follows (it declined against the weight minimum of 2 until 2026-10-25)',
      fn: async () => {
        const { first, } = await judgeFirstRound({ scenario: PLURALITY_OF_TWO, },);
        expect(first.decision,).toBe('judged',);
        expect(drawnWeightsOf({ first, },),).toEqual([1, 1.5,],);
        expect(first.runoffFinalists,).toBeUndefined();
      },
    },),
  ],
},);
