/**
 Tests for the constructed comparison that asks whether the translate judges
 can tell a complete rendering from one missing a sentence.

 What these pin is that the trial SCORES what came back rather than where it
 sat: the same judge behaviour must read as correct in one direction and wrong
 in the other, and a decline must never be counted as a hit.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { CANDIDATE_NONE, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  type FidelityDirection,
  runFidelityTrial,
} from '../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the trial writes its progress to.
 */
const l = tagged({ tag: 'judge-fidelity-test', },);

/**
 Complete English for the fixture slice.
 */
const CLEAN_TEXT = 'The cat sleeps on the windowsill each morning. She watches the birds outside.\n';

/**
 Same English with the second sentence deleted.
 */
const DAMAGED_TEXT = 'The cat sleeps on the windowsill each morning.\n';

/**
 Sentence the deletion removed, so it appears in the complete text alone.
 */
const CLEAN_ONLY_SENTENCE = 'She watches the birds outside.';

/**
 Chinese the candidates claim to render.
 */
const SOURCE_TEXT = '小猫每天早上在窗台上睡觉。她看着外面的鸟。\n';

/**
 Roster the ballots go to.
 */
const ROSTER = [
  'hf:cat/Cat-A',
  'hf:cat/Cat-B',
  'hf:cat/Cat-C',
].map(function toModelId(id,): RosterModelId {
  return id as unknown as RosterModelId;
},);

/**
 Which text a scripted judge votes for.
 */
type ScriptedPick = 'clean' | 'damaged';

/**
 What a scripted judge does with its ballot: back one of the texts, or name no
 candidate at all.
 */
type ScriptedVote = ScriptedPick | 'decline';

/**
 Builds a client whose judges vote by TEXT rather than by position, each one
 following its own entry in the script, so a test states a judge behaviour and
 a mixed roster is expressible.

 @param script - vote per roster model id

 @returns Client the trial can be driven with

 @example
 ```ts
 const client = judgesVoting({ script: { 'hf:cat/Cat-A': 'decline', },  },);
 ```
 */
function judgesVoting(
  { script, }: { readonly script: Readonly<Record<string, ScriptedVote>>; },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by selection',);
    },
    quotas: async () => {
      throw new Error('quotas unused by selection',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Whole sheet the judge was sent, which carries both candidates.
       */
      const sheet = request.messages
        .map(function toContent(message,) {
          return messageText({ message, },);
        },)
        .join('\n',);

      /**
       Where the second candidate begins, which the sheet labels.

       READ RATHER THAN ASSUMED, because the trial rotates the ballot and a
       scripted judge that voted by position would agree with itself no matter
       what the trial did with the texts.
       */
      const secondAt = sheet.indexOf('CANDIDATE 2',);

      /**
       Whether the complete text is the first candidate, decided by a sentence
       only it carries. The deletion is otherwise a PREFIX of the complete
       text, so searching for either whole text finds the same position.
       */
      const cleanIsFirst = sheet.indexOf(CLEAN_ONLY_SENTENCE,) < secondAt;

      /**
       What this particular judge was told to do.
       */
      const vote = script[request.modelId];
      if (vote === undefined)
        throw new Error(`no scripted vote for ${request.modelId}`,);

      /**
       Ballot position this judge is scripted to back, one-based, or
       `CANDIDATE_NONE` where it names nothing.
       */
      const wantedPosition = (vote === 'decline')
        ? CANDIDATE_NONE
        : (((vote === 'clean') === cleanIsFirst) ? 1 : 2);

      /**
       Wire value carrying that vote.
       */
      const value: unknown = {
        best: wantedPosition,
        assessment: 'fixture',
      };
      if (!request.validate(value,)) {
        return {
          kind: 'schema-mismatch',
          rawText: JSON.stringify(value,),
          detail: 'reply failed the wire guard',
        };
      }
      return {
        kind: 'ok',
        value: value as ValueT,
        rawText: JSON.stringify(value,),
      };
    },
  };
}

/**
 Script in which every judge does the same thing.

 @param vote - what the whole roster does

 @returns Script covering every roster model

 @example
 ```ts
 const script = wholeRoster({ vote: 'decline', },);
 ```
 */
function wholeRoster({ vote, }: { readonly vote: ScriptedVote; },): Record<string, ScriptedVote> {
  return Object.fromEntries(ROSTER.map(function toEntry(modelId,) {
    return [
      modelId,
      vote,
    ];
  },),);
}

/**
 Runs one trial against a scripted roster.

 @param script - vote per roster model id

 @param direction - which side holds the clean text

 @param cleanFirst - whether the clean text is listed first

 @returns Trial outcome

 @example
 ```ts
 const outcome = await runScripted({ script, direction: 'preserve', cleanFirst: true, },);
 ```
 */
async function runScripted(
  {
    script,
    direction,
    cleanFirst,
  }: {
    readonly script: Readonly<Record<string, ScriptedVote>>;
    readonly direction: FidelityDirection;
    readonly cleanFirst: boolean;
  },
) {
  return await runFidelityTrial({
    client: judgesVoting({ script, },),
    trial: {
      trialId: 'cat/0',
      direction,
      damageKind: 'deletion',
      sourceText: SOURCE_TEXT,
      contextText: '',
      cleanText: CLEAN_TEXT,
      damagedText: DAMAGED_TEXT,
      cleanFirst,
    },
    judgeModelIds: ROSTER,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

/**
 Runs one trial where every judge backs the same text.

 @param pick - text every judge backs

 @param direction - which side holds the clean text

 @param cleanFirst - whether the clean text is listed first

 @returns Trial outcome

 @example
 ```ts
 const outcome = await trial({ pick: 'clean', direction: 'preserve', cleanFirst: true, },);
 ```
 */
async function trial(
  {
    pick,
    direction,
    cleanFirst,
  }: {
    readonly pick: ScriptedPick;
    readonly direction: FidelityDirection;
    readonly cleanFirst: boolean;
  },
) {
  return await runScripted({
    script: wholeRoster({ vote: pick, },),
    direction,
    cleanFirst,
  },);
}

await describe({
  name: runFidelityTrial.name,
  children: [
    it({
      name: 'scores the COMPLETE text correct in both directions, so a roster that reads is not '
        + 'confused with one that keeps whatever it was handed',
      fn: async () => {
        /** One outcome per direction, both scripted to back the complete text. */
        const outcomes = await Promise.all([
          trial({
            pick: 'clean',
            direction: 'preserve',
            cleanFirst: true,
          },),
          trial({
            pick: 'clean',
            direction: 'replace',
            cleanFirst: true,
          },),
        ],);
        for (const outcome of outcomes) {
          expect(outcome.verdict,).toBe('clean',);
          expect(outcome.correct,).toBe(true,);
        }
      },
    },),
    it({
      name: 'scores the DELETION wrong in both directions, including the one where choosing it means '
        + 'replacing good archive text with a shorter rendering',
      fn: async () => {
        /** One outcome per direction, both scripted to back the deletion. */
        const outcomes = await Promise.all([
          trial({
            pick: 'damaged',
            direction: 'preserve',
            cleanFirst: true,
          },),
          trial({
            pick: 'damaged',
            direction: 'replace',
            cleanFirst: true,
          },),
        ],);
        for (const outcome of outcomes) {
          expect(outcome.verdict,).toBe('damaged',);
          expect(outcome.correct,).toBe(false,);
        }
      },
    },),
    it({
      name: 'reads the same behaviour the same way when the clean text is listed SECOND, so ballot '
        + 'position cannot be mistaken for judgement',
      fn: async () => {
        const outcome = await trial({
          pick: 'clean',
          direction: 'preserve',
          cleanFirst: false,
        },);
        expect(outcome.verdict,).toBe('clean',);
        expect(outcome.correct,).toBe(true,);
        expect(outcome.cleanFirst,).toBe(false,);
        // Carried through so a row says which question it answers: a deletion
        // result and an insertion result mean different things.
        expect(outcome.damageKind,).toBe('deletion',);
        expect(outcome.ballots
          .every(function backedClean(ballot,) {
            return ballot.picked === 'clean';
          },),).toBe(true,);
      },
    },),
    it({
      name: 'reads an ABSTAINING ballot as declined rather than as a vote for the deletion, which is '
        + 'what naming no candidate would otherwise collapse into',
      fn: async () => {
        /**
         One judge names nothing while the other two back the complete text,
         which still carries the trial at full weight.
         */
        const outcome = await runScripted({
          script: {
            'hf:cat/Cat-A': 'decline',
            'hf:cat/Cat-B': 'clean',
            'hf:cat/Cat-C': 'clean',
          },
          direction: 'preserve',
          cleanFirst: true,
        },);
        expect(outcome.verdict,).toBe('clean',);
        expect(outcome.correct,).toBe(true,);
        /** Prompt rotation changes order, not which judge abstained. */
        const picks = Object.fromEntries(outcome.ballots.map(function byJudge(ballot,) {
          return [ballot.modelId, ballot.picked,];
        },),);
        expect(outcome.ballots,).toHaveLength(3,);
        expect(picks,).toEqual({
          'hf:cat/Cat-A': 'declined',
          'hf:cat/Cat-B': 'clean',
          'hf:cat/Cat-C': 'clean',
        },);
      },
    },),
    it({
      name: 'counts a roster that WHOLLY DECLINES as incorrect, so an abstention never reads as '
        + 'having found the deletion',
      fn: async () => {
        /** Every judge names nothing, which leaves the trial without a winner. */
        const outcome = await runScripted({
          script: wholeRoster({ vote: 'decline', },),
          direction: 'preserve',
          cleanFirst: true,
        },);
        expect(outcome.verdict,).toBe('declined',);
        expect(outcome.correct,).toBe(false,);
        expect(outcome.declineReason,).toBe('every judge declined',);
        // The declining exit is a separate return, so it carries the fixture
        // identity separately too.
        expect(outcome.damageKind,).toBe('deletion',);
        expect(outcome.ballots
          .every(function abstained(ballot,) {
            return ballot.picked === 'declined';
          },),).toBe(true,);
      },
    },),
    it({
      name: 'counts a panel where ONE JUDGE ALONE answered as incorrect even where that judge '
        + 'named the complete text, since one voice does not carry a stage',
      fn: async () => {
        /**
         One judge backs the complete text and the rest abstain, which draws
         one ballot against the `MIN_SELECTION_BALLOTS` a winner needs.
         */
        const outcome = await runScripted({
          script: {
            'hf:cat/Cat-A': 'clean',
            'hf:cat/Cat-B': 'decline',
            'hf:cat/Cat-C': 'decline',
          },
          direction: 'preserve',
          cleanFirst: true,
        },);
        expect(outcome.verdict,).toBe('declined',);
        expect(outcome.correct,).toBe(false,);
        expect(outcome.declineReason,).toBe('winner named by one judge alone',);
        // The ballot still records what that judge chose, which is the reading
        // that separates a panel nobody voted in from one that could not agree.
        /** Preserve per-judge evidence independently of prompt-rotated order. */
        const picks = Object.fromEntries(outcome.ballots.map(function byJudge(ballot,) {
          return [ballot.modelId, ballot.picked,];
        },),);
        expect(outcome.ballots,).toHaveLength(3,);
        expect(picks,).toEqual({
          'hf:cat/Cat-A': 'clean',
          'hf:cat/Cat-B': 'declined',
          'hf:cat/Cat-C': 'declined',
        },);
      },
    },),
  ],
},);
