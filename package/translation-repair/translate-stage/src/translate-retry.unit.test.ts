/**
 Tests for the second judging a declined slate buys.

 Three doors: a first judging that decides is returned as it stands with no
 second ask; a first decline followed by a decision keeps the decision and
 carries both rounds' findings; two declines settle as `no-candidate-backed`.
 The thrown door, for a slice with nothing in the archive, is covered by the
 stage suite.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
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
  BlankSelectionError,
  laneCandidate,
  type LaneText,
  TranslateAbsenceError,
} from '@monochromatic-dev/translation-repair-translate-floor';
import {
  judgeSlateWithRetry,
  type ProducedSlate,
  produceTranslateSlate,
  type TranslateStageResult,
} from '../dist/final/node/index.mjs';
import { candidateCarrying, } from '@monochromatic-dev/translation-repair-test-fixture/ts/translate-ballot.test-fixture.ts';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the judgings under test.
 */
const l = tagged({ tag: 'translate-retry-test', },);

/**
 Schema name the producing half asks translators for; every other structured
 ask is a judge sheet.
 */
const TRANSLATE_SCHEMA = 'translation_report';

/**
 Original slice both halves work over.
 */
const SOURCE_TEXT = '猫猫在窗台上打盹，尾巴垂在暖气片旁边。';

/**
 Translation already in the archive, awkward but present.
 */
const INCUMBENT_TEXT = 'The cat is doing the sleeping on the windowsill, with tail hanging by the radiator.';

/**
 Models that render the slice.
 */
const TRANSLATORS: readonly RosterModelId[] = [
  'hf:cat/Cat-A',
  'hf:cat/Cat-B',
].map(function toId(id,) {
  return id as unknown as RosterModelId;
},);

/**
 Judges, three so selection can reach its two ballots with one to spare.
 */
const JUDGES: readonly RosterModelId[] = [
  'hf:cat/Cat-A',
  'hf:cat/Cat-B',
  'hf:cat/Cat-C',
].map(function toId(id,) {
  return id as unknown as RosterModelId;
},);

/**
 Seat no judge holds, standing for an author before the slate exists.
 */
const NOBODY = 'hf:cat/nobody' as unknown as RosterModelId;

/**
 What the translators render, one each in call order.
 */
const RENDERINGS: readonly string[] = [
  'The cat dozes on the windowsill, tail draped beside the radiator.',
  'A cat naps on the sill, its tail hanging near the heater.',
];

/**
 Four translators for a slate wide enough to narrow twice (class
 eighty-two).
 */
const FOUR_TRANSLATORS: readonly RosterModelId[] = [
  'hf:cat/Cat-A',
  'hf:cat/Cat-B',
  'hf:cat/Cat-D',
  'hf:cat/Cat-E',
].map(function toId(id,) {
  return id as unknown as RosterModelId;
},);

/**
 Judges who rendered nothing, so every ballot carries a whole weight.
 */
const DISINTERESTED_JUDGES: readonly RosterModelId[] = [
  'hf:cat/Cat-C',
  'hf:cat/Cat-F',
  'hf:cat/Cat-G',
].map(function toId(id,) {
  return id as unknown as RosterModelId;
},);

/**
 Four renderings, each carrying one word the others lack.
 */
const FOUR_RENDERINGS: readonly string[] = [
  ...RENDERINGS,
  'The cat curls up on the ledge, tail tucked by the warmth.',
  'The cat yawns on the sill, tail resting against the pipe.',
];

/**
 What a scripted judge says: a rejection, or the word only the wanted
 rendering carries.
 */
type ScriptedBallot = 'reject' | 'dozes' | 'naps' | 'curls' | 'yawns';

/**
 Client whose translators render in call order and whose judges answer as the
 per-round script says.

 @param ballotFor - what the judges say, given how many judgings have been
 asked so far (the first is 1)

 @returns Client plus the count of judge calls made

 @example
 ```ts
 const rig = scriptedRig({ ballotFor: () => 'reject', },);
 ```
 */
function scriptedRig(
  {
    ballotFor,
    renderings,
    judgeCount,
  }: {
    readonly ballotFor: (judging: number, seat: RosterModelId) => ScriptedBallot;
    readonly renderings: readonly string[];
    readonly judgeCount: number;
  },
): {
  readonly client: ModelClient;
  readonly judgeCalls: { count: number; };
  readonly judgePrompts: string[];
} {
  /**
   Translator calls served so far.
   */
  const served = { count: 0, };

  /**
   Judge calls made so far, which says which judging this is.
   */
  const judgeCalls = { count: 0, };
  /**
   Exact model-plus-message identities across judging rounds.
   */
  const judgePrompts: string[] = [];

  return {
    judgeCalls,
    judgePrompts,
    client: {
      chatText: async () => {
        throw new Error('chatText unused by the translate lane',);
      },
      quotas: async () => {
        throw new Error('quotas unused by the translate lane',);
      },
      chatJson: async <ValueT,>(
        request: ChatJsonRequest<ValueT>,
      ): Promise<ChatJsonOutcome<ValueT>> => {
        /**
         Which sheet this is.
         */
        const schema = request.responseFormat
          ?.json_schema
          .name;
        if (schema === TRANSLATE_SCHEMA) {
          /**
           Rendering this call gets.
           */
          const translation = renderings[served.count % renderings.length] ?? '';
          served.count += 1;
          /**
           Reply as the wire expects it.
           */
          const value: unknown = { translation, };
          if (!request.validate(value,))
            throw new Error('the fixture translation failed the wire guard',);
          return {
            kind: 'ok',
            value,
            rawText: JSON.stringify(value,),
          };
        }
        judgeCalls.count += 1;
        judgePrompts.push(JSON.stringify({
          modelId: request.modelId,
          messages: request.messages,
        },),);

        /**
         Which judging this call belongs to, every judge answering once per
         judging.
         */
        const judging = Math.ceil(judgeCalls.count / judgeCount,);

        /**
         Sheet text, for finding the wanted candidate.
         */
        const content = request.messages
          .map(function toContent(message,) {
            return messageText({ message, },);
          },)
          .join('\n',);

        /**
         What this seat says this judging: a rejection, or the rendering it
         wants by a word only that rendering carries.
         */
        const wanted = ballotFor(
          judging,
          request.modelId,
        );
        /**
         Ballot for this judging.
         */
        const ballot: unknown = {
          best: (wanted === 'reject')
            ? 0
            : candidateCarrying({
              content,
              needle: wanted,
            },),
          assessment: 'fixture',
        };
        if (!request.validate(ballot,))
          throw new Error('the fixture ballot failed the wire guard',);
        return {
          kind: 'ok',
          value: ballot as ValueT,
          rawText: JSON.stringify(ballot,),
        };
      },
    },
  };
}

/**
 Judges one freshly produced slate through the retry, under one script.

 @param ballotFor - what the judges say per judging

 @param laneTexts - lane texts the slate offers beside the proposals, none
 by default

 @param lines - where the lines the judging logs go, none by default

 @returns Stage result plus the judge calls it cost

 @example
 ```ts
 const { result, } = await judgedUnder({ ballotFor: () => 'dozes', },);
 ```
 */
async function judgedUnder(
  {
    ballotFor,
    incumbentKind = 'present',
    translators = TRANSLATORS,
    judges = JUDGES,
    renderings = RENDERINGS,
    laneTexts = [],
    lines,
  }: {
    readonly ballotFor: (
      judging: number,
      seat: RosterModelId,
      dozesAuthor: RosterModelId,
    ) => ScriptedBallot;
    readonly incumbentKind?: 'present' | 'absent';
    readonly translators?: readonly RosterModelId[];
    readonly judges?: readonly RosterModelId[];
    readonly renderings?: readonly string[];
    readonly laneTexts?: readonly LaneText[];
    readonly lines?: string[];
  },
): Promise<{
  readonly result: TranslateStageResult;
  readonly judgeCalls: number;
  readonly judgePrompts: readonly string[];
}> {
  /**
   Logger for the judging, keeping its lines where a case asked for them.
   */
  const logger = (lines === undefined) ? l : capturingLogger({ messages: lines, },);
  /**
   Translator whose rendering carries the word the scripts ask for, known
   only once the slate is produced, since the rig renders in call order.
   */
  const author = { dozes: NOBODY, };
  /**
   Scripted client and its counter.
   */
  const rig = scriptedRig({
    ballotFor: function withAuthor(
      judging,
      seat,
    ): ScriptedBallot {
      return ballotFor(
        judging,
        seat,
        author.dozes,
      );
    },
    renderings,
    judgeCount: judges.length,
  },);

  /**
   Slate the translators produced.
   */
  const produced = await produceTranslateSlate({
    client: rig.client,
    translatorModelIds: translators,
    sourceText: SOURCE_TEXT,
    incumbentText: INCUMBENT_TEXT,
    lineStructured: false,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l: logger,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);

  /**
   Translator credited first for the rendering that carries the word.
   */
  const [dozesAuthor = NOBODY,] = produced.candidates
    .filter(function carriesDozes(candidate,): boolean {
      return candidate.rendered.includes('dozes',);
    },)
    .flatMap(function toAuthors(candidate,): readonly RosterModelId[] {
      return producerModelIds(candidate.producer,);
    },);
  author.dozes = dozesAuthor;

  /**
   Lane texts joined to the slate after the proposals, as the consolidation
   offers them (class forty).
   */
  const withLanes = {
    ...produced,
    candidates: [
      ...produced.candidates,
      ...laneTexts.map(function asLaneCandidate(laneText,) {
        return laneCandidate(laneText,);
      },),
    ],
  };

  /**
   What the retry settled on.
   */
  const result = await judgeSlateWithRetry({
    judging: {
      client: rig.client,
      produced: withLanes,
      judgeModelIds: judges,
      sourceText: SOURCE_TEXT,
      incumbentText: (incumbentKind === 'present') ? INCUMBENT_TEXT : '',
      incumbentKind,
      lineStructured: false,
      signal: AbortSignal.timeout(HANG_STOP_MS,),
      perCallTimeoutMs: HANG_STOP_MS,
      l: logger,
      stageSettings: SCRIPTED_STAGE_SETTINGS,
    },
  },);
  return {
    result,
    judgeCalls: rig.judgeCalls.count,
    judgePrompts: rig.judgePrompts,
  };
}

/**
 Finding the retry writes between the two rounds' findings.
 */
const RETRY_FINDING = 'translate-declined-retried';

/**
 Script under which the first disinterested judge names one rendering, the
 second names another and the third rejects, every judging alike: a tie
 between two that no run-off can narrow.

 @param first - word the first judge's rendering carries

 @param second - word the second judge's rendering carries

 @returns Ballot script for the rig

 @example
 ```ts
 const ballotFor = tiedEveryJudging({ first: 'naps', second: 'curls', },);
 ```
 */
function tiedEveryJudging(
  {
    first,
    second,
  }: {
    readonly first: ScriptedBallot;
    readonly second: ScriptedBallot;
  },
): (judging: number, seat: RosterModelId) => ScriptedBallot {
  return function splitAlike(
    _judging,
    seat,
  ): ScriptedBallot {
    if (seat === DISINTERESTED_JUDGES[0])
      return first;
    return (seat === DISINTERESTED_JUDGES[1]) ? second : 'reject';
  };
}

/**
 Ballot script for the class eighty-two run-off (slice 14): a four-translator
 slate that narrows from three finalists to two before deciding, shared by
 the case where the slice has no archive incumbent and the case where the
 archive wording is the incumbent each intermediate decline keeps.

 @param judging - which judging this ballot answers, counting from one

 @param seat - judge whose ballot is scripted

 @returns Ballot the seat casts

 @example
 ```ts
 const { result, } = await judgedUnder({ judges: DISINTERESTED_JUDGES, ballotFor: narrowTwice, },);
 ```
 */
function narrowTwice(
  judging: number,
  seat: RosterModelId,
): ScriptedBallot {
  if (seat === DISINTERESTED_JUDGES[0])
    return 'dozes';
  if (seat === DISINTERESTED_JUDGES[1])
    return (judging === 3) ? 'dozes' : 'naps';
  if (judging === 1)
    return 'curls';
  return (judging === 2) ? 'reject' : 'naps';
}

/**
 Repair lane text carrying a word no translator's rendering carries.
 */
const REPAIR_LANE: LaneText = {
  lane: 'repair',
  text: 'The cat curls up on the ledge, tail tucked by the warmth.',
};

/**
 Translate lane text carrying a word no translator's rendering carries.
 */
const TRANSLATE_LANE: LaneText = {
  lane: 'translate',
  text: 'The cat yawns on the sill, tail resting against the pipe.',
};

/**
 Finding naming a run-off tie the preference broke.
 */
const TIE_BROKEN = 'translate-runoff-tie-broken';

/**
 What the retry raises over a slice with no archive wording, for a judging that
 settles nothing.

 @param judges - judges who answer
 @param laneTexts - lane texts the slate offers beside the proposals
 @param ballotFor - what the judges say per judging

 @returns Raised error, or the string `settled` where the judging did not raise

 @example
 ```ts
 const raised = await absenceUnder({ judges: DISINTERESTED_JUDGES, laneTexts: [], ballotFor: () => 'reject', },);
 ```
 */
async function absenceUnder(
  {
    judges,
    laneTexts,
    ballotFor,
  }: {
    readonly judges: readonly RosterModelId[];
    readonly laneTexts: readonly LaneText[];
    readonly ballotFor: (
      judging: number,
      seat: RosterModelId,
      dozesAuthor: RosterModelId,
    ) => ScriptedBallot;
  },
): Promise<unknown> {
  try {
    await judgedUnder({
      incumbentKind: 'absent',
      judges,
      laneTexts,
      ballotFor,
    },);
    return 'settled';
  }
  catch (error) {
    return error;
  }
}

await describe({
  name: judgeSlateWithRetry.name,
  children: [
    it({
      name: 'KEEPS a second judging\'s decision after a first decline, carrying the first round\'s findings and '
        + 'the retry marker, so the record shows both asks',
      fn: async () => {
        const { result, judgeCalls, judgePrompts, } = await judgedUnder({
          ballotFor: function firstRejects(judging,): 'reject' | 'dozes' {
            return (judging === 1) ? 'reject' : 'dozes';
          },
        },);

        expect(result.origin,).toBe('fresh',);
        expect(result.text.includes('dozes',),).toBe(true,);
        expect(result.findings.includes(RETRY_FINDING,),).toBe(true,);
        expect(result.findings.includes('translate-declined (rejection)',),).toBe(true,);
        expect(judgeCalls,).toBe(JUDGES.length * 2,);
        expect(new Set(judgePrompts,).size,).toBe(judgeCalls,);
      },
    },),

    it({
      name: 'SEATS the run-off leader named by two ballots where the slice has no '
        + 'incumbent (class seventy-three, mikaela4 slice 28, 2026-09-19): a tie between two valid finalists, '
        + 'then Cat-A and Cat-C name the same one while Cat-B abstains',
      fn: async () => {
        // Judging 1: Cat-A names its own rendering (half), Cat-B its own
        // (half), Cat-C rejects: a tie, so the challenge is a run-off over
        // both. Judging 2: Cat-A names its own again (half), Cat-C names it
        // (full), Cat-B rejects: 1.5 from two ballots, which two ballots decide
        // (it was declined against a weight minimum of 2 until 2026-10-25).
        const { result, judgeCalls, } = await judgedUnder({
          incumbentKind: 'absent',
          ballotFor: function splitThenLean(
            judging,
            seat,
            dozesAuthor,
          ): 'reject' | 'dozes' | 'naps' {
            /** Whether this seat rendered a candidate. */
            const wrote = TRANSLATORS.includes(seat,);
            if (judging === 1) {
              if (!wrote)
                return 'reject';
              return (seat === dozesAuthor) ? 'dozes' : 'naps';
            }
            if (!wrote)
              return 'dozes';
            return (seat === dozesAuthor) ? 'dozes' : 'reject';
          },
        },);
        expect(result.origin,).toBe('fresh',);
        expect(result.text.includes('dozes',),).toBe(true,);
        expect(result.findings.includes(RETRY_FINDING,),).toBe(true,);
        expect(result.findings.includes('select-runoff-under-minimum',),).toBe(false,);
        expect(judgeCalls,).toBe(JUDGES.length * 2,);
      },
    },),
    it({
      name: 'ASKS a run-off AGAIN while the tie keeps narrowing the finalists, and seats the round that decides '
        + '(class eighty-two, XingZ621 slice 14, 2026-09-22: eight valid candidates split 1/1/1/1, the run-off '
        + 'over the four leaders split 1/1/0.5, and the entry stopped though the tie had narrowed 4 to 2)',
      fn: async () => {
        // Judging 1: C dozes, F naps, G curls, yawns unnamed: a tie of three of
        // four. Judging 2: C dozes, F naps, G rejects: a tie of two of three.
        // Judging 3: C dozes, F dozes, G naps: dozes at weight 2.
        /** Lines the judging logged. */
        const lines: string[] = [];
        const { result, judgeCalls, } = await judgedUnder({
          incumbentKind: 'absent',
          translators: FOUR_TRANSLATORS,
          judges: DISINTERESTED_JUDGES,
          renderings: FOUR_RENDERINGS,
          ballotFor: narrowTwice,
          lines,
        },);
        expect(result.origin,).toBe('fresh',);
        expect(result.text.includes('dozes',),).toBe(true,);
        // The second narrowing is announced without calling the round before it a tie: a round that leaves no winner is
        // a tie or a plurality one judge alone drew (ledger B806).
        expect(
          lines.some(function announcesSecondNarrowing(line,): boolean {
            return line.includes('run-off left no winner over 3 finalists and narrowed to 2; asking the same panel over them',);
          },),
        ).toBe(true,);
        expect(result.findings.includes('translate-runoff (finalists 3 of 4)',),).toBe(true,);
        expect(result.findings.includes('translate-runoff (finalists 2 of 3)',),).toBe(true,);
        expect(judgeCalls,).toBe(DISINTERESTED_JUDGES.length * 3,);
      },
    },),
    it({
      name: 'RAISES the absence, shipping the repair lane text by no preference, when a run-off over valid finalists '
        + 'ties again one ballot each and cannot narrow where the slice has no incumbent (class one hundred '
        + 'seventy-four, TianqiChen6669 slice 4, 2026-09-26, settled by "Two judges must name it": no judge pair '
        + 'backed either rendering, so the order ships neither and the passage goes to the source-text path)',
      fn: async () => {
        const raised = await absenceUnder({
          judges: DISINTERESTED_JUDGES,
          laneTexts: [REPAIR_LANE,],
          ballotFor: tiedEveryJudging({
            first: 'naps',
            second: 'curls',
          },),
        },);
        expect(raised,).toBeInstanceOf(TranslateAbsenceError,);
        expect((raised as TranslateAbsenceError).findings.some(function isTieBroken(finding,): boolean {
          return finding.startsWith(TIE_BROKEN,);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'RAISES the absence where the tied finalists are a writer\'s proposal and the translate lane text, '
        + 'one ballot each, with the repair lane text no finalist',
      fn: async () => {
        const raised = await absenceUnder({
          judges: DISINTERESTED_JUDGES,
          laneTexts: [
            REPAIR_LANE,
            TRANSLATE_LANE,
          ],
          ballotFor: tiedEveryJudging({
            first: 'dozes',
            second: 'yawns',
          },),
        },);
        expect(raised,).toBeInstanceOf(TranslateAbsenceError,);
      },
    },),
    it({
      name: 'RAISES the absence where two writers\' proposals tie one ballot each and neither lane text is a '
        + 'finalist, since slate order alone backs no candidate',
      fn: async () => {
        const raised = await absenceUnder({
          judges: DISINTERESTED_JUDGES,
          laneTexts: [REPAIR_LANE,],
          ballotFor: tiedEveryJudging({
            first: 'dozes',
            second: 'naps',
          },),
        },);
        expect(raised,).toBeInstanceOf(TranslateAbsenceError,);
      },
    },),
    it({
      name: 'KEEPS the incumbent when a run-off ties again over a slice the archive translates, breaking no tie',
      fn: async () => {
        const { result, } = await judgedUnder({
          judges: DISINTERESTED_JUDGES,
          ballotFor: tiedEveryJudging({
            first: 'dozes',
            second: 'naps',
          },),
        },);
        expect(result.origin,).toBe('incumbent',);
        expect(result.text,).toBe(INCUMBENT_TEXT,);
        expect(result.findings.some(function isTieBroken(finding,): boolean {
          return finding.startsWith(TIE_BROKEN,);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'ASKS A RUN-OFF AGAIN WHERE THE ARCHIVE WORDING IS KEPT while the tie keeps narrowing, since each '
        + 'decline that keeps the archive still names the finalists a further run-off would ask about',
      fn: async () => {
        // The narrowing script of the class eighty-two case over a slice the
        // archive translates: each decline keeps the archive's wording and
        // returns, naming the finalists, rather than raising.
        const { result, judgeCalls, } = await judgedUnder({
          translators: FOUR_TRANSLATORS,
          judges: DISINTERESTED_JUDGES,
          renderings: FOUR_RENDERINGS,
          ballotFor: narrowTwice,
        },);
        expect({
          origin: result.origin,
          text: result.text,
          runoffs: result.findings.filter(function isRunoff(finding,): boolean {
            return finding.startsWith('translate-runoff (',);
          },),
          judgeCalls,
        },).toEqual({
          origin: 'fresh',
          text: FOUR_RENDERINGS[0],
          runoffs: [
            'translate-runoff (finalists 3 of 4)',
            'translate-runoff (finalists 2 of 3)',
          ],
          judgeCalls: DISINTERESTED_JUDGES.length * 3,
        },);
      },
    },),
    it({
      name: 'RAISES AN EMPTY SLATE\'S REFUSAL AT ONCE, asking no second round, since no challenge can change a slate '
        + 'nobody was heard on',
      fn: async () => {
        /**
         Slate with nothing on it, every translator lost.
         */
        const produced: ProducedSlate = {
          candidates: [],
          heardTranslators: 0,
          findings: [],
        };
        /**
         What the retry raised.
         */
        const raised = await (async function attempt(): Promise<unknown> {
          try {
            return await judgeSlateWithRetry({
              judging: {
                client: {} as unknown as ModelClient,
                produced,
                judgeModelIds: JUDGES,
                sourceText: SOURCE_TEXT,
                incumbentText: '',
                incumbentKind: 'absent',
                lineStructured: false,
                signal: AbortSignal.timeout(HANG_STOP_MS,),
                perCallTimeoutMs: HANG_STOP_MS,
                l,
                stageSettings: SCRIPTED_STAGE_SETTINGS,
              },
            },);
          }
          catch (error) {
            return error;
          }
        })();
        expect(raised,).toBeInstanceOf(TranslateAbsenceError,);
        expect({
          reason: (raised as TranslateAbsenceError).reason,
          findings: (raised as TranslateAbsenceError).findings,
        },).toEqual({
          reason: 'no-voice-heard',
          findings: ['translate-no-voice-heard',],
        },);
      },
    },),
    it({
      name: 'PASSES ANY OTHER FAILURE THROUGH UNCHANGED: a judged winner that says nothing raises the blank '
        + 'selection, which no second round could change',
      fn: async () => {
        /**
         Client whose every judge names the only candidate.
         */
        const client: ModelClient = {
          chatText: async () => {
            throw new Error('chatText unused by the translate lane',);
          },
          quotas: async () => {
            throw new Error('quotas unused by the translate lane',);
          },
          chatJson: async <ValueT,>(
            request: ChatJsonRequest<ValueT>,
          ): Promise<ChatJsonOutcome<ValueT>> => {
            /**
             Ballot naming the only candidate.
             */
            const ballot: unknown = {
              best: 1,
              assessment: 'fixture',
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
        /**
         Slate built by hand, since the producer keeps a blank candidate off a
         real one; its sole candidate says nothing.
         */
        const produced: ProducedSlate = {
          candidates: [{
            producer: {
              kind: 'model',
              modelId: 'hf:cat/Cat-D' as unknown as RosterModelId,
            },
            value: {
              text: '   ',
              origin: 'fresh',
            },
            rendered: '   ',
          },],
          heardTranslators: 1,
          findings: ['translate-blank-fixture-marker',],
        };
        /**
         What the retry raised.
         */
        const raised = await (async function attempt(): Promise<unknown> {
          try {
            return await judgeSlateWithRetry({
              judging: {
                client,
                produced,
                judgeModelIds: JUDGES,
                sourceText: SOURCE_TEXT,
                incumbentText: INCUMBENT_TEXT,
                incumbentKind: 'present',
                lineStructured: false,
                signal: AbortSignal.timeout(HANG_STOP_MS,),
                perCallTimeoutMs: HANG_STOP_MS,
                l,
                stageSettings: SCRIPTED_STAGE_SETTINGS,
              },
            },);
          }
          catch (error) {
            return error;
          }
        })();
        expect(raised,).toBeInstanceOf(BlankSelectionError,);
        expect((raised as BlankSelectionError).findings,).toEqual(['translate-blank-fixture-marker',],);
      },
    },),
    it({
      name: 'asks ONCE when the first judging decides, and writes no retry marker',
      fn: async () => {
        const { result, judgeCalls, } = await judgedUnder({
          ballotFor: function alwaysDecides(): 'reject' | 'dozes' {
            return 'dozes';
          },
        },);

        expect(result.origin,).toBe('fresh',);
        expect(result.findings.includes(RETRY_FINDING,),).toBe(false,);
        expect(judgeCalls,).toBe(JUDGES.length,);
      },
    },),

    it({
      name: 'SETTLES two declines as no-candidate-backed rather than as the momentary reason, keeping the '
        + 'incumbent and both rounds\' findings',
      fn: async () => {
        const { result, judgeCalls, } = await judgedUnder({
          ballotFor: function alwaysRejects(): 'reject' | 'dozes' {
            return 'reject';
          },
        },);

        expect(result.decision,).toBe('no-candidate-backed',);
        expect(result.origin,).toBe('incumbent',);
        expect(result.text,).toBe(INCUMBENT_TEXT,);
        expect(result.findings.filter(function isRetry(finding,): boolean {
          return finding === RETRY_FINDING;
        },).length,).toBe(1,);
        expect(judgeCalls,).toBe(JUDGES.length * 2,);
      },
    },),
  ],
},);
