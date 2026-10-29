/**
 Tests archive-block context, reverse splicing, removal, the footnote check
 reading the page the revisions already applied leave (ledger B80), and the
 linear two-step preparation: one correction round, one re-preparation,
 remaining unclaimed blocks recorded as findings instead of a cycle pause.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { UnclaimedTargetBlock, } from '@monochromatic-dev/translation-repair-corpus-source';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary';
import type { PairedReading, } from '@monochromatic-dev/translation-repair-slice-reading';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import type { BenchSeating, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  REVISION_SHAPE_REFUSED,
  revisionFootnoteFindings,
} from '@monochromatic-dev/translation-repair-archive';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  archiveBlockIdentity,
  archiveBlockSourceContexts,
  repairArchiveBlocks,
} from '@monochromatic-dev/translation-repair-page-check';
import { preparePassEntry, } from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_OPENROUTER_DECISIONS,
  SEAT_OPENROUTER_ONLY,
  SEAT_OPENROUTER_ONLY_CHECKER,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

import { NO_OUTSIDE_READS, } from './pass-outside-reads.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/** Four-seat review and selection roster. */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

/** Disposable pipeline generation. */
const GENERATION = `sha256-tree-v1:${'b'.repeat(64,)}` as PipelineDigest;

/** Test logger. */
const l = tagged({ tag: 'archive-block-repair-test', },);

/**
 Creates client selecting scripted block replacements.

 @param replacementFor - replacement derived from exact request prompt

 @returns Direct scripted client
 */
function correctionClient(
  { replacementFor, }: { readonly replacementFor: (prompt: string) => string; },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText not used',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /** Requested structured role. */
      const schema = request.responseFormat?.json_schema.name ?? '';
      /** Exact messages for replacement routing. */
      const prompt = JSON.stringify(request.messages,);
      /** Scripted valid value. */
      const value: unknown = schema === 'archive_block_review'
        ? {
          disposition: 'revise',
          sourceQuote: '',
          replacementText: replacementFor(prompt,),
          assessment: 'Remove unsupported archive-only wording.',
        }
        : schema === 'candidate_ballot'
          ? {
            best: 1,
            assessment: 'Correction removes unsupported wording.',
          }
          : { pairs: [{ source: 0, target: 0, },], };
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
 Constructs block offsets from exact substring.

 @param targetText - archive containing block once

 @param blockText - exact block wording

 @param blockId - parser-like audit id

 @returns Unclaimed block fixture
 */
function blockAt(
  {
    targetText,
    blockText,
    blockId,
  }: {
    readonly targetText: string;
    readonly blockText: string;
    readonly blockId: string;
  },
): UnclaimedTargetBlock {
  /** Exact start offset. */
  const startOffset = targetText.indexOf(blockText,);
  return {
    location: { kind: 'aligned-pair', pairIndex: 0, },
    blockId,
    startOffset,
    endOffset: startOffset + blockText.length,
  };
}

/**
 Review rosters an alternating hook hands over in turn, disjoint from each
 other and from the fixture's own.
 */
const ALTERNATES: readonly (readonly RosterModelId[])[] = [
  [
    SEAT_HYPER_ONLY,
    SEAT_OPENROUTER_ONLY,
  ],
  [
    SEAT_OPENROUTER_DECISIONS,
    SEAT_OPENROUTER_ONLY_CHECKER,
  ],
];

/**
 One structured call: the seat asked, and how many rosters the hook had
 handed over when it was asked.
 */
type ReviewAsk = {
  /**
   Seat the call asked.
   */
  readonly seat: RosterModelId;
  /**
   Handovers so far, zero before the first.
   */
  readonly handover: number;
};

/**
 Roster the alternating hook handed over at a handover.

 @param handover - handovers so far

 @returns That handover's roster, none before the first

 @example
 ```ts
 const roster = alternateAt({ handover: 1, },);
 ```
 */
function alternateAt({ handover, }: { readonly handover: number; },): readonly RosterModelId[] {
  return (handover === 0) ? [] : (ALTERNATES[(handover - 1) % ALTERNATES.length] ?? []);
}

/**
 Reviews two unclaimed blocks, recording every call's seat and handover.

 @param alternating - whether a hook hands over the alternates in turn, or
 none is given

 @returns Every structured call the review made

 @example
 ```ts
 const asks = await reviewsAsked({ alternating: true, },);
 ```
 */
async function reviewsAsked(
  { alternating, }: { readonly alternating: boolean; },
): Promise<readonly ReviewAsk[]> {
  /**
   Handovers the hook made so far.
   */
  const hook = { handovers: 0, };
  /**
   Every structured call.
   */
  const asks: ReviewAsk[] = [];
  /**
   Scripted reviewer removing each block.
   */
  const inner = correctionClient({
    replacementFor: function replacement(): string {
      return '';
    },
  },);
  /**
   Two archive blocks the pairing left unclaimed.
   */
  const targetText = 'The cat won an award.\n\nThe cat won a second award.';
  /**
   Those blocks.
   */
  const blocks = [
    blockAt({ targetText, blockText: 'The cat won an award.', blockId: 'block/0', }),
    blockAt({ targetText, blockText: 'The cat won a second award.', blockId: 'block/1', }),
  ];
  await repairArchiveBlocks({
    client: {
      chatText: inner.chatText,
      chatJson: async (request,) => {
        asks.push({ seat: request.modelId, handover: hook.handovers, },);
        return await inner.chatJson(request,);
      },
      quotas: inner.quotas,
    },
    modelIds: ROSTER,
    targetText,
    sourceContexts: new Map(blocks.map(function context(block,): readonly [string, string] {
      return [archiveBlockIdentity({ block, targetText, }), '猫在睡觉。',] as const;
    },),),
    blocks,
    signal: new AbortController().signal,
    exchangeTimeoutMs: HANG_STOP_MS,
    l,
    ...(alternating
      ? {
        beforeBlock: async (): Promise<BenchSeating> => {
          hook.handovers += 1;
          return { modelIds: alternateAt({ handover: hook.handovers, },), };
        },
      }
      : {}),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return asks;
}

/**
 Reviews one block, recording every call's exact request messages.

 @param sourceContexts - source-context map the block correction round reads

 @returns Serialized messages of every archive-block-review call, in order

 @example
 ```ts
 const prompts = await reviewPrompts({ sourceContexts: new Map([[identity, '猫在睡觉。',],],), },);
 ```
 */
async function reviewPrompts(
  { sourceContexts, }: { readonly sourceContexts: ReadonlyMap<string, string>; },
): Promise<readonly string[]> {
  /**
   Every archive-block-review call's exact messages, serialized.
   */
  const prompts: string[] = [];
  /**
   Scripted reviewer retaining the block unchanged.
   */
  const inner = correctionClient({
    replacementFor: function replacement(): string {
      return 'The cat napped on the mat.';
    },
  },);
  /**
   Single archive text, which is also its one unclaimed block.
   */
  const targetText = 'The cat napped on the mat.';
  /**
   That block.
   */
  const block = blockAt({ targetText, blockText: targetText, blockId: 'block/0', },);
  await repairArchiveBlocks({
    client: {
      chatText: inner.chatText,
      chatJson: async (request,) => {
        if (request.responseFormat?.json_schema.name === 'archive_block_review')
          prompts.push(JSON.stringify(request.messages,),);
        return await inner.chatJson(request,);
      },
      quotas: inner.quotas,
    },
    modelIds: ROSTER,
    targetText,
    sourceContexts,
    blocks: [block,],
    signal: new AbortController().signal,
    exchangeTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return prompts;
}

await describe({
  name: 'archive block repair',
  children: [
    describe({
      name: 'review context, splicing and the single correction round',
      children: [
        it({
          name: 'SCOPES source support to aligned section named by unclaimed location',
          fn: async () => {
            const prepared = prepareDocumentPair({
              sourceText: 'Cats nap.',
              targetText: 'Cats nap.\n\nAn aside.',
              blockPairings: new Map([[0, [{ source: 0, target: 0, },],],]),
            },);
            const [block,] = prepared.unclaimedTargetBlocks;
            if (block === undefined)
              throw new Error('fixture did not expose unclaimed block',);
            const contexts = archiveBlockSourceContexts({ prepared, },);
            /**
             Key the block's support is filed under.
             */
            const identity = archiveBlockIdentity({
              block,
              targetText: prepared.targetText,
            },);
            expect(contexts.get(identity,),).toBe('Cats nap.',);
          },
        },),
        it({
          name: 'SCOPES source support to the aligned section when the preparation carries an explicit section '
            + 'pairing, not only when alignment ran automatically',
          fn: async () => {
            const prepared = prepareDocumentPair({
              sourceText: 'Cats nap.',
              targetText: 'Cats nap.\n\nAn aside.',
              blockPairings: new Map([[0, [{ source: 0, target: 0, },],],]),
              sectionPairing: [{ source: 0, target: 0, },],
            },);
            expect(prepared.sectionPairing,).toEqual([{ source: 0, target: 0, },],);
            const [block,] = prepared.unclaimedTargetBlocks;
            if (block === undefined)
              throw new Error('fixture did not expose unclaimed block',);
            const contexts = archiveBlockSourceContexts({ prepared, },);
            /**
             Key the block's support is filed under.
             */
            const identity = archiveBlockIdentity({
              block,
              targetText: prepared.targetText,
            },);
            expect(contexts.get(identity,),).toBe('Cats nap.',);
          },
        },),
        it({
          name: 'REFUSES an aligned-pair location with no matching alignment pair, naming the block, the pair it '
            + 'sits in and how many pairs the page has: the preparation and the alignment disagree about the '
            + 'block, which is a fault, and empty support would drop every reply that cites the original without '
            + 'a word',
          fn: async () => {
            const prepared = prepareDocumentPair({
              sourceText: 'Cats nap.',
              targetText: 'Cats nap.\n\nAn aside.',
              blockPairings: new Map([[0, [{ source: 0, target: 0, },],],]),
            },);
            const [block,] = prepared.unclaimedTargetBlocks;
            if (block === undefined)
              throw new Error('fixture did not expose unclaimed block',);
            /** The same block, naming an alignment pair the page has none of. */
            const misaligned: UnclaimedTargetBlock = {
              ...block,
              location: { kind: 'aligned-pair', pairIndex: 99, },
            };
            /** The mapper over the misaligned block. */
            const contextsOfMisaligned = (): unknown => archiveBlockSourceContexts({
              prepared: {
                ...prepared,
                unclaimedTargetBlocks: [misaligned,],
              },
            },);
            expect(contextsOfMisaligned,).toThrow(misaligned.blockId,);
            expect(contextsOfMisaligned,).toThrow('aligned pair 99',);
            expect(contextsOfMisaligned,).toThrow('holds 1 pair',);
          },
        },),
        it({
          name: 'LIMITS picture support to corroborated assets referenced by the aligned source section',
          fn: async () => {
            /** Aligned source section includes repeated, unavailable and textless references. */
            const sourceText = `<PhotoScroll photos={['\${path}/photos/chat.webp', '\${path}/photos/chat.webp', '\${path}/photos/portrait.webp', '\${path}/photos/unread.webp']} />`;
            /** Unclaimed block in the same aligned section. */
            const blockText = '> Mittens greets her sister.';
            /** Explicit pairing keeps the archive block available for review. */
            const prepared = prepareDocumentPair({
              sourceText,
              targetText: `${sourceText}\n\n${blockText}`,
              blockPairings: new Map([[0, [{ source: 0, target: 0, },],],]),
            },);
            /** Corroborated transcript; the review reads the one the other readers carry most, the fuller on a tie. */
            const supported: PairedReading = {
              kind: 'corroborated',
              readings: [
                { modelId: ROSTER[0], text: '手套猫：你好，姐姐。', },
                { modelId: ROSTER[1], text: '你好，姐姐。我们一起回家。', },
              ],
              overlap: 1,
            };
            /** Unrelated and unusable evidence must not license this block. */
            const pictureReadings = new Map<string, PairedReading>([
              ['chat.webp', supported,],
              ['elsewhere.webp', { ...supported, readings: [{ modelId: ROSTER[0], text: '无关内容', },], },],
              ['portrait.webp', { kind: 'no-text', characters: 0, },],
              ['unread.webp', { kind: 'unavailable', reason: 'readers-disagree', transient: false, perReader: [], },],
            ],);
            /** Review contexts produced by the same mapper preparation uses. */
            const contexts = archiveBlockSourceContexts({ prepared, pictureReadings, },);
            /** The block's exact source support. */
            const context = [...contexts.values(),].join('\n',);
            expect(prepared.unclaimedTargetBlocks.length,).toBeGreaterThan(0,);
            expect(context,).toContain('你好，姐姐。我们一起回家。',);
            // One transcript per picture (2026-09-16): the shorter reading the
            // fuller one carries is not sent beside it.
            expect(context,).not.toContain('手套猫：你好，姐姐。\n',);
            expect(context,).toContain('2 readers agree',);
            expect(context,).not.toContain('无关内容',);
            expect(context,).not.toContain('elsewhere.webp',);
            expect(context.split('CORROBORATED PICTURE SOURCE SUPPORT chat.webp',).slice(1,),).toHaveLength(1,);

            /** A target-only location has no source authority even when readings exist. */
            const targetOnly = archiveBlockSourceContexts({
              prepared: {
                ...prepared,
                unclaimedTargetBlocks: prepared.unclaimedTargetBlocks.map(function outside(block,) {
                  return { ...block, location: { kind: 'target-section', sectionIndex: 0, }, };
                },),
              },
              pictureReadings,
            },);
            expect([...targetOnly.values(),].every(function empty(value,): boolean {
              return value === '';
            },),).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES A BLOCK THE SOURCE-CONTEXT MAP HOLDS NO ENTRY FOR, naming the block and the identities the '
            + 'map holds: the caller and the map disagree about the block, and reviewing it against an empty '
            + 'source would drop every reply that cites the original as unanchored without a word',
          fn: async () => {
            /** The archive and the one block the review is asked about. */
            const targetText = 'The cat napped on the mat.';
            /** Identity the review looks the block up under. */
            const identity = archiveBlockIdentity({
              block: blockAt({ targetText, blockText: targetText, blockId: 'block/0', },),
              targetText,
            },);
            /** Identity of a block the map holds instead, as when the archive text differs between caller and map. */
            const other = archiveBlockIdentity({
              block: blockAt({ targetText, blockText: targetText, blockId: 'block/7', },),
              targetText,
            },);
            await expect(reviewPrompts({ sourceContexts: new Map([[other, '猫在睡觉。',],],), },),).rejects.toThrow(identity,);
            await expect(reviewPrompts({ sourceContexts: new Map([[other, '猫在睡觉。',],],), },),).rejects.toThrow(other,);
            await expect(reviewPrompts({ sourceContexts: new Map(), },),).rejects.toThrow('holds no block',);
          },
        },),
        it({
          name: 'REVIEWS A BLOCK AGAINST AN EMPTY SOURCE WHEN THE MAP SPELLS THE ENTRY OUT AS EMPTY, the state a '
            + 'target-only section is filed under: an empty entry is an answer, an absent one is not',
          fn: async () => {
            const prompts = await reviewPrompts({
              sourceContexts: new Map([[
                archiveBlockIdentity({
                  block: blockAt({
                    targetText: 'The cat napped on the mat.',
                    blockText: 'The cat napped on the mat.',
                    blockId: 'block/0',
                  },),
                  targetText: 'The cat napped on the mat.',
                },),
                '',
              ],],),
            },);
            expect(prompts.length,).toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'APPLIES multiple selected revisions in reverse offsets including empty removal',
          fn: async () => {
            const first = 'The cat won an award.';
            const second = '[REMOVE BLOCK]';
            const targetText = `${first}\n\n${second}`;
            const blocks = [
              blockAt({ targetText, blockText: first, blockId: 'block/0', }),
              blockAt({ targetText, blockText: second, blockId: 'block/1', }),
            ];
            const sourceContexts = new Map(blocks.map(function context(block,): readonly [string, string] {
              return [archiveBlockIdentity({ block, targetText, }), '猫在睡觉。',] as const;
            },),);
            const repaired = await repairArchiveBlocks({
              client: correctionClient({
                replacementFor: function replacement(prompt,): string {
                  return prompt.lastIndexOf(first,) > prompt.lastIndexOf(second,)
                    ? 'The cat sleeps.'
                    : '';
                },
              },),
              modelIds: ROSTER,
              targetText,
              sourceContexts,
              blocks,
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            // CLASS NINETY-FOUR: a removal takes its own separator with it, so
            // the document does not end in the blank line the removed block owned.
            expect(repaired.targetText,).toBe('The cat sleeps.');
            // One line per block says what became of it; the stage's findings (ledger B714) stand ahead of them.
            expect(repaired.findings.filter(function saysWhatBecameOfBlock(finding,): boolean {
              return finding.startsWith('archive block reviewed and revised: ',);
            },),).toHaveLength(2);
          },
        },),
        it({
          name: 'RETAINS A REVISION THAT IS THE BLOCK WITH ITS SOFT LINE BREAKS ELSEWHERE, which the site renders '
            + 'as the same page, as it retains one repeating the block byte for byte, and still APPLIES one that '
            + 'changes a word (ledger B26)',
          fn: async () => {
            const first = 'The cat naps on the mat\nall afternoon.';
            const second = 'The dog barks.';
            const targetText = `${first}\n\n${second}`;
            const blocks = [
              blockAt({ targetText, blockText: first, blockId: 'block/0', }),
              blockAt({ targetText, blockText: second, blockId: 'block/1', }),
            ];
            const sourceContexts = new Map(blocks.map(function context(block,): readonly [string, string] {
              return [archiveBlockIdentity({ block, targetText, }), '猫在睡觉。',] as const;
            },),);
            const repaired = await repairArchiveBlocks({
              client: correctionClient({
                replacementFor: function replacement(prompt,): string {
                  return prompt.lastIndexOf('all afternoon',) > prompt.lastIndexOf(second,)
                    ? 'The cat naps on the mat all afternoon.'
                    : 'The dog sleeps.';
                },
              },),
              modelIds: ROSTER,
              targetText,
              sourceContexts,
              blocks,
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(repaired.targetText,).toBe(`${first}\n\nThe dog sleeps.`);
            expect(repaired.findings.some(function retained(finding,): boolean {
              return finding.startsWith('archive block revision repeated its original wording and was retained',);
            },),).toBe(true);
          },
        },),
        it({
          name: 'LEAVES ONE BLANK LINE where a removed block stood between two others, and none where it '
            + 'opened the body (class ninety-four, XingZ627, 2026-09-23: the archive\'s placeholder line was '
            + 'removed and the page shipped three blank lines after its front matter)',
          fn: async () => {
            const front = '---\nname: Mittens\n---';
            const placeholder = '**Come back later!**';
            const kept = 'The cat sleeps.';
            const targetText = `${front}\n\n${placeholder}\n\n${kept}\n\n[REMOVE MIDDLE]\n\nThe cat wakes.`;
            const blocks = [
              blockAt({ targetText, blockText: placeholder, blockId: 'block/0', }),
              blockAt({ targetText, blockText: '[REMOVE MIDDLE]', blockId: 'block/1', }),
            ];
            const sourceContexts = new Map(blocks.map(function context(block,): readonly [string, string] {
              return [archiveBlockIdentity({ block, targetText, }), '猫在睡觉。',] as const;
            },),);
            const repaired = await repairArchiveBlocks({
              client: correctionClient({
                replacementFor: function replacement(): string {
                  return '';
                },
              },),
              modelIds: ROSTER,
              targetText,
              sourceContexts,
              blocks,
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(repaired.targetText,).toBe(`${front}\n\n${kept}\n\nThe cat wakes.`);
          },
        },),
        it({
          name: 'WITHHOLDS A REVISION THAT THE REVISIONS ALREADY APPLIED MAKE A FOOTNOTE DEFECT: two blocks each '
            + 'drop a marker of one note, either alone leaves the note referenced, and the page keeps the earlier '
            + 'block\'s marker rather than shipping a note nothing references (ledger B80)',
          fn: async () => {
            /** Earlier block, reviewed after the later one. */
            const earlier = 'Mittens[^1] naps on the sill.';
            /** Later block, reviewed first. */
            const later = 'Biscuit[^1] naps by the door.';
            /** The note both blocks reference. */
            const note = '[^1]: Both are tabbies.\n';
            const targetText = `${earlier}\n\n${later}\n\n${note}`;
            /** Every reviewer's revision of the earlier block. */
            const earlierRevision = 'Mittens naps on the sill.';
            /** Every reviewer's revision of the later block. */
            const laterRevision = 'Biscuit naps by the door.';
            const blocks = [
              blockAt({ targetText, blockText: earlier, blockId: 'block/0', }),
              blockAt({ targetText, blockText: later, blockId: 'block/1', }),
            ];
            const sourceContexts = new Map(blocks.map(function context(block,): readonly [string, string] {
              return [archiveBlockIdentity({ block, targetText, }), '猫在睡觉。',] as const;
            },),);
            // Each revision alone leaves the note referenced, the case's premise.
            expect([
              revisionFootnoteFindings({
                modelId: ROSTER[0],
                blockText: earlier,
                blockOffset: targetText.indexOf(earlier,),
                replacementText: earlierRevision,
                targetText,
              },),
              revisionFootnoteFindings({
                modelId: ROSTER[0],
                blockText: later,
                blockOffset: targetText.indexOf(later,),
                replacementText: laterRevision,
                targetText,
              },),
            ],).toEqual([
              [],
              [],
            ],);
            /**
             Whether a prompt asks about the earlier block: the block under
             review is quoted after the page, and the later block's archive
             wording is gone from the page once its revision stands.

             @param prompt - request messages, serialized

             @returns Whether the earlier block is the one under review
             */
            function reviewsEarlier(prompt: string,): boolean {
              return prompt.lastIndexOf(earlier,) > prompt.lastIndexOf(later,);
            }
            /** Scripted reviewers revising whichever block they are asked about. */
            const inner = correctionClient({
              replacementFor: function replacement(prompt,): string {
                return reviewsEarlier(prompt,) ? earlierRevision : laterRevision;
              },
            },);
            /** Seats asked to review the earlier block: the gather asks a window of the bench, not all of it. */
            const askedEarlier: RosterModelId[] = [];
            /** Lines the review logged, each behind its level. */
            const lines: string[] = [];
            const repaired = await repairArchiveBlocks({
              client: {
                chatText: inner.chatText,
                chatJson: async (request,) => {
                  if (
                    (request.responseFormat?.json_schema.name === 'archive_block_review')
                      && reviewsEarlier(JSON.stringify(request.messages,),)
                  )
                    askedEarlier.push(request.modelId,);
                  return await inner.chatJson(request,);
                },
                quotas: inner.quotas,
              },
              modelIds: ROSTER,
              targetText,
              sourceContexts,
              blocks,
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l: levelCapturingLogger({ lines, },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            /** The page as the later block's revision leaves it. */
            const afterLater = `${earlier}\n\n${laterRevision}\n\n${note}`;
            /** Footnote refusals the review warned of, without their tag. */
            const withheld = lines
              .filter(function refusal(line,): boolean {
                return line.startsWith('warn ',) && line.includes(REVISION_SHAPE_REFUSED,);
              },)
              .map(function untagged(line,): string {
                return line.slice(line.indexOf('] ',) + 2,);
              },)
              .toSorted();
            expect({
              page: repaired.targetText,
              askedAny: askedEarlier.length > 0,
              withheld,
            },).toEqual({
              page: afterLater,
              askedAny: true,
              withheld: askedEarlier.flatMap(function refusedIn(modelId,): readonly string[] {
                return revisionFootnoteFindings({
                  modelId,
                  blockText: earlier,
                  blockOffset: afterLater.indexOf(earlier,),
                  replacementText: earlierRevision,
                  targetText: afterLater,
                },);
              },)
                .toSorted(),
            },);
            // The page the earlier revisions would join carries the defect,
            // one refusal per seat asked.
            expect(withheld,).toHaveLength(askedEarlier.length,);
          },
        },),
        it({
          name: 'SETTLES after one correction round, recording still-unclaimed blocks as findings',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'archive-block-cycle-', },);
            const dir = scratch.path;
            // The corrector that once alternated forever now gets exactly one
            // round: prepare, correct, re-prepare, and whatever stays unclaimed
            // becomes a finding on the returned preparation.
            const paired = await preparePassEntry({
              client: correctionClient({
                replacementFor: function alternate(prompt,): string {
                  return prompt.includes('Aside A',) ? 'Aside B' : 'Aside A';
                },
              },),
              entryId: 'Cat',
              entryCacheDir: dir,
              pipelineDigest: GENERATION,
              modelIds: ROSTER,
              sourceText: 'Cats nap.',
              targetText: 'Cats nap.\n\nAside A',
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              outsideReads: NO_OUTSIDE_READS,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(paired.prepared
              .targetText
              .includes('Aside B',),).toBe(true,);
            expect(
              paired.prepared
                .alignmentFindings
                .some(function namesRemaining(finding,): boolean {
                  return finding.includes('unclaimed archive blocks remain after the single correction round',);
                },),
            ).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: `${repairArchiveBlocks.name} keeps the review stage's findings (ledger B714)`,
      children: [
        it({
          name: 'CARRIES THE STAGE\'S FINDINGS INTO WHAT THE ARTIFACT RECORDS AND THE LOG, so a block that lost '
            + 'voices to the review round shows it: two of four seats answer in a shape nothing can read, and the '
            + 'block\'s record names the voices lost ahead of the line saying the block was revised',
          fn: async () => {
            /** Unclaimed block the review revises. */
            const blockText = 'The cat won an award.';
            const targetText = blockText;
            const block = blockAt({ targetText, blockText, blockId: 'block/0', });
            /** Key the block is recorded under. */
            const identity = archiveBlockIdentity({ block, targetText, },);
            /** Scripted reviewer revising the block. */
            const inner = correctionClient({
              replacementFor: function replacement(): string {
                return 'The cat sleeps.';
              },
            },);
            /** Seats whose review reply nothing can read. */
            const mute: ReadonlySet<RosterModelId> = new Set([ROSTER[2], ROSTER[3],],);
            /** Lines the pass logged, behind their levels. */
            const lines: string[] = [];
            const repaired = await repairArchiveBlocks({
              client: {
                chatText: inner.chatText,
                chatJson: async (request,) => {
                  if (
                    mute.has(request.modelId,)
                      && (request.responseFormat?.json_schema.name === 'archive_block_review')
                  ) {
                    return {
                      kind: 'schema-mismatch',
                      rawText: '',
                      reason: 'truncated-completion',
                      detail: 'provider reported a truncating completion (model stopped with finish_reason=length)',
                    };
                  }
                  return await inner.chatJson(request,);
                },
                quotas: inner.quotas,
              },
              modelIds: ROSTER,
              targetText,
              sourceContexts: new Map([[identity, '猫在睡觉。',],],),
              blocks: [block,],
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l: levelCapturingLogger({ lines, },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            /** The stage's findings that name a voice lost. */
            const lost = repaired.findings.filter(function namesVoiceLost(finding,): boolean {
              return finding.startsWith('stage-voice-lost (archive-block-review',);
            },);
            expect(lost.length,).toBeGreaterThan(0,);
            // The block's own line stays last, so the findings ahead of it are its.
            expect(repaired.findings.at(-1,),).toBe(`archive block reviewed and revised: ${identity}`,);
            expect(repaired.findings,).toContain(
              `archive block review findings (${String(repaired.findings.length - 2,)}): ${identity}`,
            );
            expect(lines.some(function logsLoss(line,): boolean {
              return line.includes('stage-voice-lost (archive-block-review',) && line.includes(identity,);
            },),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: `${repairArchiveBlocks.name} re-seated under a hold (ledger X12)`,
      children: [
        it({
          name: 'REVIEWS EACH BLOCK ON THE ROSTER ITS HOOK LAST HANDED OVER, so a roster re-read after a provider '
            + 'dry-out reviews the blocks after it rather than any roster read before it',
          fn: async () => {
            /**
             Calls a caller with no hook made.
             */
            const control = await reviewsAsked({ alternating: false, },);
            /**
             Calls made while the hook alternates the rosters.
             */
            const moved = await reviewsAsked({ alternating: true, },);
            expect({
              controlAskedAny: control.length > 0,
              controlOffRoster: control.filter(function offRoster(ask,): boolean {
                return !(ROSTER as readonly RosterModelId[]).includes(ask.seat,);
              },),
              firstBlockAsked: moved.some(function atFirstHandover(ask,): boolean {
                return ask.handover === 1;
              },),
              secondBlockAsked: moved.some(function atSecondHandover(ask,): boolean {
                return ask.handover === 2;
              },),
              movedStrayed: moved.filter(function strayed(ask,): boolean {
                return !alternateAt({ handover: ask.handover, },).includes(ask.seat,);
              },),
            },).toEqual({
              controlAskedAny: true,
              controlOffRoster: [],
              firstBlockAsked: true,
              secondBlockAsked: true,
              movedStrayed: [],
            },);
          },
        },),
      ],
    },),
  ],
},);
