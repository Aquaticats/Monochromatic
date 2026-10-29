/**
 Guards the notes the archive's translators left each other against the
 archive block review (zheermao101, run zheermao15, 2026-10-20). The pinned
 archive holds, inside the blockquote that translates the One Among Us reply
 email, a JSX comment in Han asking the other translators how to translate a
 sentence. The review revised the whole blockquote and
 the revision, written from a sheet that showed the note, had none, so the
 prepared archive text no longer held it and the page assembly, which carries
 a note back only when the archive text it compares with still holds one,
 carried nothing.

 Whatever the review does to a block, the prepared archive keeps each note
 between translators the block held. Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { UnclaimedTargetBlock, } from '@monochromatic-dev/translation-repair-corpus-source';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  archiveBlockIdentity,
  repairArchiveBlocks,
} from '@monochromatic-dev/translation-repair-page-check';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Four-seat review and selection roster.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

/**
 Source section the block is reviewed against.
 */
const SOURCE = '猫在睡觉。';

/**
 Note the archive's translators left each other inside a quoted line, in
 the shape the pinned zheermao101 archive has it.
 */
const NOTE = '{/* 猫可以躲一躲。 这句不知道如何翻译 */}';

/**
 A second note, in Han, of the same block.
 */
const SECOND_NOTE = '<!-- 猫窝：Cat Bed -->';

/**
 Nested quoted lines around a note, as the archive writes the email block.
 */
const BLOCK = `> - Mittens\n>   > Please prepare a little.\n>   > ${NOTE}\n>   > All of us are with you.`;

/**
 The revision every reviewer writes for the block, without the note.
 */
const REVISION = '> - Mittens\n>   > Please get ready.\n>   > Everyone here is with you.';

/**
 Scripted reviewer answering every review with one replacement.

 @param replacement - complete replacement every reviewer writes

 @returns Direct scripted client, whose selection ballot names the first candidate

 @example
 ```ts
 const client = revisingClient({ replacement: 'The cat sleeps.', },);
 ```
 */
function revisingClient({ replacement, }: { readonly replacement: string; },): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText not used',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Requested structured role.
       */
      const schema = request.responseFormat?.json_schema.name ?? '';
      /**
       Scripted valid value.
       */
      const value: unknown = schema === 'archive_block_review'
        ? {
          disposition: 'revise',
          sourceQuote: '',
          replacementText: replacement,
          assessment: 'Reword the archive-only block.',
        }
        : {
          best: 1,
          assessment: 'The correction is the better wording.',
        };
      if (!request.validate(value,))
        throw new Error(`scripted ${schema} reply failed validator`,);
      return {
        kind: 'ok',
        value,
        rawText: JSON.stringify(value,),
      };
    },
    quotas: async () => {
      throw new Error('quotas not used',);
    },
  };
}

/**
 Reviews the one block between an opening and a closing paragraph.

 @param block - block's wording

 @param replacement - what every reviewer writes for it

 @param source - original section the block is reviewed against

 @returns Archive after the round, its findings, every line logged and
 how many structured calls the round made

 @example
 ```ts
 const { targetText, } = await reviewedBlock({ block: BLOCK, replacement: REVISION, source: SOURCE, },);
 ```
 */
async function reviewedBlock(
  {
    block,
    replacement,
    source,
  }: {
    readonly block: string;
    readonly replacement: string;
    readonly source: string;
  },
): Promise<{
  readonly targetText: string;
  readonly findings: readonly string[];
  readonly lines: readonly string[];
  readonly identity: string;
  readonly asked: number;
}> {
  /**
   Archive the block stands in.
   */
  const archive = `The cat is asleep.\n\n${block}\n\nThe cat wakes.`;
  /**
   Where the block starts.
   */
  const startOffset = archive.indexOf(block,);
  /**
   The unclaimed block.
   */
  const unclaimed: UnclaimedTargetBlock = {
    location: { kind: 'aligned-pair', pairIndex: 0, },
    blockId: 'block/1',
    startOffset,
    endOffset: startOffset + block.length,
  };
  /**
   Identity the review files the block's source support under.
   */
  const identity = archiveBlockIdentity({
    block: unclaimed,
    targetText: archive,
  },);
  /**
   Logger and the lines it keeps.
   */
  const { logger, lines, } = capturingLoggerPair();
  /**
   The scripted reviewer, which the round is asked through.
   */
  const inner = revisingClient({ replacement, },);
  /**
   Structured calls the round made.
   */
  const calls = { asked: 0, };
  const repaired = await repairArchiveBlocks({
    client: {
      chatText: inner.chatText,
      chatJson: async (request,) => {
        calls.asked += 1;
        return await inner.chatJson(request,);
      },
      quotas: inner.quotas,
    },
    modelIds: ROSTER,
    targetText: archive,
    sourceContexts: new Map([[identity, source,],],),
    blocks: [unclaimed,],
    signal: new AbortController().signal,
    exchangeTimeoutMs: HANG_STOP_MS,
    l: tagged({
      l: logger,
      tag: 'archive-block-repair-notes-test',
    },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    targetText: repaired.targetText,
    findings: repaired.findings,
    lines,
    identity,
    asked: calls.asked,
  };
}

await describe({
  name: 'the archive block review keeps the notes between translators the block held',
  children: [
    it({
      name: 'KEEPS a note the revision lacks, on a line of its own after the revised block, and says so',
      fn: async () => {
        const { targetText, findings, lines, identity, } = await reviewedBlock({
          block: BLOCK,
          replacement: REVISION,
          source: SOURCE,
        },);
        expect(targetText,).toBe(`The cat is asleep.\n\n${REVISION}\n\n${NOTE}\n\nThe cat wakes.`,);
        expect(findings,).toContain(
          `archive block reviewed and revised: ${identity}`,
        );
        expect(findings.some(function names(finding,): boolean {
          return finding.includes(identity,) && finding.includes(JSON.stringify(NOTE,),);
        },),).toBe(true,);
        expect(lines.some(function logged(line,): boolean {
          return line.includes(NOTE,) && line.includes('kept',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'KEEPS every note in the archive\'s order, and does not repeat one the revision already carries',
      fn: async () => {
        const both = `> - Mittens\n>   > Please prepare a little.\n>   > ${NOTE}\n>   > All of us are with you.\n>   > ${SECOND_NOTE}`;
        const missingOne = await reviewedBlock({
          block: both,
          replacement: REVISION,
          source: SOURCE,
        },);
        expect(missingOne.targetText,).toBe(
          `The cat is asleep.\n\n${REVISION}\n\n${NOTE}\n\n${SECOND_NOTE}\n\nThe cat wakes.`,
        );
        const carriesOne = await reviewedBlock({
          block: both,
          replacement: `${REVISION}\n>   > ${SECOND_NOTE}`,
          source: SOURCE,
        },);
        expect(carriesOne.targetText,).toBe(
          `The cat is asleep.\n\n${REVISION}\n>   > ${SECOND_NOTE}\n\n${NOTE}\n\nThe cat wakes.`,
        );
      },
    },),
    it({
      name: 'KEEPS the notes of a block the review removes outright, in the place the block stood',
      fn: async () => {
        const { targetText, } = await reviewedBlock({
          block: BLOCK,
          replacement: '',
          source: SOURCE,
        },);
        expect(targetText,).toBe(`The cat is asleep.\n\n${NOTE}\n\nThe cat wakes.`,);
      },
    },),
    it({
      name: 'LEAVES a comment in English alone, and a comment the original itself carries',
      fn: async () => {
        const english = '<!-- check the cat\'s name -->';
        const originals = '{/* 原文的注释 */}';
        const { targetText, } = await reviewedBlock({
          block: `${BLOCK}\n>   > ${english}\n>   > ${originals}`,
          replacement: REVISION,
          source: `${SOURCE}\n\n${originals}`,
        },);
        expect(targetText,).toBe(`The cat is asleep.\n\n${REVISION}\n\n${NOTE}\n\nThe cat wakes.`,);
      },
    },),
    it({
      name: 'ASKS NOBODY about a block that is nothing but notes, which the reviewers\' sheet would show empty, '
        + 'and leaves it standing with the finding that says why',
      fn: async () => {
        const { targetText, findings, asked, identity, } = await reviewedBlock({
          block: NOTE,
          replacement: 'The cat sleeps.',
          source: SOURCE,
        },);
        expect(asked,).toBe(0,);
        expect(targetText,).toBe(`The cat is asleep.\n\n${NOTE}\n\nThe cat wakes.`,);
        expect(findings,).toEqual([`archive block holds only notes between translators and stands: ${identity}`,],);
      },
    },),
  ],
},);
