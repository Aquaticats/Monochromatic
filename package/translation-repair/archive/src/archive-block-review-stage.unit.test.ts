/**
 Tests archive-only provenance, correction, and recorded naturalness under
 the single-round contract: reviewer indecision retains the block with
 findings and never buys a second round.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  reachableQuorum,
  rosterQuorumSize,
} from '@monochromatic-dev/translation-repair-stage';
import { scriptedClient, } from './scripted-review-client.test-fixture.ts';
import {
  ARCHIVE_BLOCK_SELECTION_CRITERIA,
  isArchiveReferenceQuoteAnchored,
  isArchiveSourceQuoteAnchored,
  isVerifiableEditorialArchiveBlock,
  runArchiveBlockReviewStage,
} from '../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/** Four seats let two producers receive disinterested selection ballots. */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

/**
 Bench most of whose seats the router refuses, so the share of it a wet
 provider serves is short of the whole bench's quorum; the case built on it
 asserts that before relying on it.
 */
const SHORT_BENCH = [
  'hf:cat/Cat-A',
  'hf:cat/Cat-B',
  'hf:cat/Cat-C',
  'hf:cat/Cat-D',
  'hf:cat/Cat-E',
  'hf:cat/Cat-F',
  'hf:cat/Cat-G',
  'hf:cat/Cat-H',
].map(function toModelId(id,): RosterModelId {
  return id as unknown as RosterModelId;
},);

/** Seats of that bench no provider serves. */
const REFUSED_SEATS: ReadonlySet<string> = new Set([
  'hf:cat/Cat-D',
  'hf:cat/Cat-E',
  'hf:cat/Cat-F',
  'hf:cat/Cat-G',
  'hf:cat/Cat-H',
],);

/**
 What the one page the invented original links says, then one detail the
 attestation verified in it, in the shape `pass-prepare.ts` joins them.
 */
const REFERENCE_CONTEXT = [
  '- reference 1 https://cats.example/posts/mittens ("Mittens"): Mittens had an older sister who was also a tabby, '
    + 'and she slept by the stove.',
  '- attested: the ARCHIVE\'s "The cat has a tabby sister who sleeps by the stove." is stated by reference 1 '
    + '("an older sister who was also a tabby"), 3 of 4 voices checked word for word',
].join('\n',);

/** Declared identity of the invented page. */
const IDENTITY_CONTEXT = '- name: ORIGINAL declares "猫猫", TRANSLATION declares "Mittens"';

/** Archive block only the cited page supports. */
const SISTER_BLOCK = 'The cat has a tabby sister who sleeps by the stove.';

/**
 A text as a captured prompt carries it: the scripted client records each
 exchange as JSON, which escapes quotation marks.

 @param text - text to find

 @returns Text as JSON writes it inside a string
 */
function asCaptured(text: string,): string {
  return JSON.stringify(text,).slice(1, -1,);
}

/** Logger for archive-block stage tests. */
const l = tagged({ tag: 'archive-block-review-stage-test', },);

/** Acceptable absolute-naturalness reply. */
const ACCEPTABLE_NATURALNESS = {
  isAcceptable: true,
  findings: [],
  assessment: 'Publication-ready wording.',
} as const;

/**
 Reply anchoring the archive quote for an archive-block review request, and
 the acceptable-naturalness reply for every other schema: the scripted reply
 every case that reaches a reachable bench's naturalness round by anchoring
 the whole roster shares.

 @param schema - structured schema requested of this reply

 @returns Reply scripted for the archive_block_review schema, or the
 acceptable-naturalness reply otherwise
 */
function anchoredReviewReplyFor(
  { schema, }: { readonly schema: string; },
): unknown {
  return schema === 'archive_block_review'
    ? {
      disposition: 'source-supported',
      sourceQuote: '窗边安静地睡觉',
      replacementText: '',
      assessment: 'Expected section supports this sentence.',
    }
    : ACCEPTABLE_NATURALNESS;
}

await describe({
  name: 'archive block review stage',
  children: [
    it({
      name: 'REQUIRES substantive source quote inside expected aligned section',
      fn: async () => {
        expect(isArchiveSourceQuoteAnchored({
          sourceContext: '猫在窗边安静地睡觉。',
          sourceQuote: '窗边安静地睡觉',
        },),).toBe(true,);
        expect(isArchiveSourceQuoteAnchored({
          sourceContext: '猫在窗边安静地睡觉。',
          sourceQuote: '猫在',
        },),).toBe(false,);
        expect(isArchiveSourceQuoteAnchored({
          sourceContext: '猫在窗边安静地睡觉。',
          sourceQuote: '狗在门边等待',
        },),).toBe(false,);
      },
    },),
    it({
      name: 'ANCHORS A QUOTE WHOSE CORNER BRACKETS CAME BACK AS ENGLISH QUOTES (ledger B24), the evidence fold '
        + 'reading the marks and nothing else',
      fn: async () => {
        for (const sourceQuote of [
          '猫说：“窗边安静。”',
          '猫说："窗边安静。"',
        ]) {
          expect(isArchiveSourceQuoteAnchored({
            sourceContext: '猫说：「窗边安静。」然后睡了。',
            sourceQuote,
          },),).toBe(true,);
        }
        expect(isArchiveSourceQuoteAnchored({
          sourceContext: '猫说：「窗边安静。」然后睡了。',
          sourceQuote: '猫说：“门边安静。”',
        },),).toBe(false,);
      },
    },),
    it({
      name: 'COUNTS THE MINIMUM IN CHARACTERS, not UTF-16 units (ledger B24), so two ideographs past the first '
        + 'plane are two',
      fn: async () => {
        expect(isArchiveSourceQuoteAnchored({
          sourceContext: '猫\u{20000}\u{20001}猫。',
          sourceQuote: '\u{20000}\u{20001}',
        },),).toBe(false,);
      },
    },),
    it({
      name: 'ANCHORS a quote in one cited page\'s text through the evidence fold, never in its address, an '
        + 'attested line or a quote shorter than the minimum (ledger B28)',
      fn: async () => {
        expect(isArchiveReferenceQuoteAnchored({
          referenceContext: REFERENCE_CONTEXT,
          sourceQuote: 'an older sister who was also a tabby',
        },),).toBe(true,);
        expect(isArchiveReferenceQuoteAnchored({
          referenceContext: REFERENCE_CONTEXT,
          sourceQuote: '(“Mittens”): Mittens had',
        },),).toBe(true,);
        for (const sourceQuote of [
          SISTER_BLOCK,
          'https://cats.example/posts/mittens',
          'she',
        ]) {
          expect(isArchiveReferenceQuoteAnchored({
            referenceContext: REFERENCE_CONTEXT,
            sourceQuote,
          },),).toBe(false,);
        }
        expect(isArchiveReferenceQuoteAnchored({
          referenceContext: '',
          sourceQuote: 'an older sister who was also a tabby',
        },),).toBe(false,);
      },
    },),
    it({
      name: 'CORROBORATES narrow editorial apparatus without licensing factual prose',
      fn: async () => {
        expect(isVerifiableEditorialArchiveBlock({ blockText: 'Translator: Cat Friend', },),).toBe(true,);
        expect(isVerifiableEditorialArchiveBlock({ blockText: 'Source: [Cat notes](https://example.test)', },),).toBe(true,);
        expect(isVerifiableEditorialArchiveBlock({ blockText: 'The cat won an award in spring.', },),).toBe(false,);
        // The archive's contributor line, read by the contributor reader: the
        // singular label and the full-width colon fit no prefix (ledger T8,
        // eighteenth batch).
        expect(isVerifiableEditorialArchiveBlock({ blockText: 'Contributor for this entry: Mittens', },),).toBe(true,);
        expect(isVerifiableEditorialArchiveBlock({ blockText: 'Contributors for this entry：Mittens, Biscuit', },),)
          .toBe(true,);
        // A picture, and a comment only when it closes.
        expect(isVerifiableEditorialArchiveBlock({ blockText: '![A tabby asleep](tabby.png)', },),).toBe(true,);
        expect(isVerifiableEditorialArchiveBlock({ blockText: '<!-- translator note -->', },),).toBe(true,);
        expect(isVerifiableEditorialArchiveBlock({ blockText: '<!-- translator note', },),).toBe(false,);
      },
    },),
    it({
      name: 'CORROBORATES A BLOCK ONLY WHEN EVERY LINE IS APPARATUS, so prose beside a label, a picture or a '
        + 'comment is not passed as apparatus with it (ledger B81)',
      fn: async () => {
        // Prose after a label line, and before a contributor line.
        expect(isVerifiableEditorialArchiveBlock({
          blockText: 'Translator: Cat Friend\nThe cat won an award in spring.',
        },),).toBe(false,);
        expect(isVerifiableEditorialArchiveBlock({
          blockText: 'The cat won an award in spring.\nContributor for this entry: Mittens',
        },),).toBe(false,);
        // A contributor line carrying its own names declares nothing on the
        // line after it.
        expect(isVerifiableEditorialArchiveBlock({
          blockText: 'Contributor for this entry: Mittens\nThe cat won an award in spring.',
        },),).toBe(false,);
        // Prose after a picture, and prose between two comments.
        expect(isVerifiableEditorialArchiveBlock({
          blockText: '![A tabby asleep](tabby.png) The cat won an award in spring.',
        },),).toBe(false,);
        expect(isVerifiableEditorialArchiveBlock({
          blockText: '<!-- note --> The cat won an award in spring. <!-- end -->',
        },),).toBe(false,);
        // A label's words opening a longer word.
        expect(isVerifiableEditorialArchiveBlock({
          blockText: 'Translation byproducts piled up by the litter box.',
        },),).toBe(false,);
        // Every line apparatus: names continuing a label standing alone, a
        // picture beside a credit, comments alone, a label with its names.
        expect([
          'Contributors for this entry:\nMittens\nBiscuit',
          '![A tabby asleep](tabby.png)\nSource: [Cat notes](https://example.test)',
          '<!-- translator note -->\n<!-- second note -->',
          'Translated by Mittens',
          'Translation:',
        ].map(function corroborated(blockText,): boolean {
          return isVerifiableEditorialArchiveBlock({ blockText, },);
        },),).toEqual([
          true,
          true,
          true,
          true,
          true,
        ],);
      },
    },),
    it({
      name: 'RETAINS anchored source wording only after two distinct naturalness responsibilities',
      fn: async () => {
        const prompts: string[] = [];
        const outcome = await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts,
            replyFor: anchoredReviewReplyFor,
          },),
          modelIds: ROSTER,
          sourceText: '猫在窗边安静地睡觉。',
          targetText: 'The cat sleeps quietly by the window.',
          blockText: 'The cat sleeps quietly by the window.',
          blockOffset: 0,
          priorFindings: [],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(outcome.kind,).toBe('retained');
        const naturalnessPrompts = prompts.filter(function isNaturalness(prompt,): boolean {
          return prompt.includes('publication-ready English',);
        },);
        expect(new Set(naturalnessPrompts,).size,).toBe(2);
      },
    },),
    it({
      name: 'RETAINS the block with an unresolved finding when post-anchor voices fall below exact-half '
        + 'participation, buying no naturalness review: a heard roster whose support does not anchor is '
        + 'a verdict, not an outage (it ended one entry in 114 seconds on 2026-09-02 as '
        + '"provider-unavailable" with every seat answering), and the no-loop design retains an '
        + 'unresolved block with its findings',
      fn: async () => {
        const prompts: string[] = [];
        /**
         Seats asked for a review, in first-call order. The first one anchors
         its quote: the gather rotates the bench by the prompt
         (`rotatedBench`), so a seat named by roster position can be the one
         the quorum closes without whenever a sheet's wording changes.
         */
        const reviewers: string[] = [];
        const outcome = await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts,
            replyFor: ({ schema, modelId, },) => {
              if (schema !== 'archive_block_review')
                return ACCEPTABLE_NATURALNESS;
              if (!reviewers.includes(modelId,))
                reviewers.push(modelId,);
              return {
                disposition: 'source-supported',
                sourceQuote: modelId === reviewers[0] ? '窗边安静地睡觉' : '不存在的来源句子',
                replacementText: '',
                assessment: 'Source support claim.',
              };
            },
          },),
          modelIds: ROSTER,
          sourceText: '猫在窗边安静地睡觉。',
          targetText: 'The cat sleeps quietly by the window.',
          blockText: 'The cat sleeps quietly by the window.',
          blockOffset: 0,
          priorFindings: [],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome.kind,).toBe('retained',);
        expect(outcome.text,).toBe('The cat sleeps quietly by the window.',);
        expect(outcome.findings.join('\n',),).toContain('archive review left the block unresolved: 1 of',);
        expect(outcome.findings.join('\n',),).toContain('archive review discarded uncorroborated retention claim',);
        // No naturalness review is bought for a block nobody could anchor.
        expect(prompts.some(function isNaturalness(prompt,): boolean {
          return prompt.includes('publication-ready English',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'RETAINS the block when the bench answered but the cap cut most replies before their content: '
        + 'an answer nobody could read is not silence (class thirty-one: one entry was interrupted '
        + '"provider-unavailable" on 2026-09-16 at 4 of 12 heard with every provider wet, seven seats '
        + 'having spent their whole completion cap reasoning about the first chat translation)',
      fn: async () => {
        const prompts: string[] = [];
        /** Seats asked, so the case proves the whole bench was reached. */
        const asked = new Set<string>();
        const outcome = await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts,
            unreadableFor: ({ modelId, },) => {
              asked.add(modelId,);
              return modelId !== ROSTER[0];
            },
            replyFor: anchoredReviewReplyFor,
          },),
          modelIds: ROSTER,
          sourceText: '猫在窗边安静地睡觉。',
          targetText: 'The cat sleeps quietly by the window.',
          blockText: 'The cat sleeps quietly by the window.',
          blockOffset: 0,
          priorFindings: [],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(asked.size,).toBe(ROSTER.length,);
        expect(outcome.kind,).toBe('retained',);
        expect(outcome.text,).toBe('The cat sleeps quietly by the window.',);
        expect(outcome.findings.join('\n',),).toContain('stage-quorum-unmet (archive-block-review 1/',);
        expect(outcome.findings.join('\n',),).toContain('archive review left the block unresolved: 1 of 1',);
      },
    },),
    it({
      name: 'REVIEWS the block on the seats the router could serve (ledger B27): with most of the bench refused, '
        + 'every reachable seat anchoring its quote meets the reachable share, so the block buys its '
        + 'naturalness read instead of standing unresolved below the quorum the whole bench would need',
      fn: async () => {
        const prompts: string[] = [];
        /** Share the gather closes on once the refusals are known. */
        const { needed, reachable, } = reachableQuorum({
          benchSize: SHORT_BENCH.length,
          unreachable: REFUSED_SEATS.size,
        },);
        // The case tests the threshold only while every reachable seat
        // anchoring still falls short of the whole bench's quorum.
        expect(reachable,).toBeLessThan(rosterQuorumSize({ rosterSize: SHORT_BENCH.length, },),);
        expect(reachable,).toBeGreaterThanOrEqual(needed,);
        const outcome = await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts,
            refusedFor: (modelId,) => REFUSED_SEATS.has(modelId,),
            replyFor: anchoredReviewReplyFor,
          },),
          modelIds: SHORT_BENCH,
          sourceText: '猫在窗边安静地睡觉。',
          targetText: 'The cat sleeps quietly by the window.',
          blockText: 'The cat sleeps quietly by the window.',
          blockOffset: 0,
          priorFindings: [],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        // The review gather itself counted every refusal.
        expect(outcome.findings,).toContain(
          `stage-short-bench (archive-block-review reachable ${String(reachable,)} of ${
            String(SHORT_BENCH.length,)
          }, quorum ${String(needed,)})`,
        );
        expect(outcome.kind,).toBe('retained',);
        expect(outcome.text,).toBe('The cat sleeps quietly by the window.',);
        expect(outcome.findings.join('\n',),).not.toContain('archive review left the block unresolved',);
        expect(outcome.findings,).toContain('archive block absolute naturalness accepted and challenged',);
      },
    },),
    it({
      name: 'REVIEWS the block when its anchored voices exactly meet the quorum (ledger B27): the threshold is a '
        + 'floor the block reaches, not one it must exceed',
      fn: async () => {
        /** Anchored voices the four-seat bench needs, every seat reachable. */
        const quorum = rosterQuorumSize({ rosterSize: ROSTER.length, },);
        /**
         Seats asked for a review, in first-call order; exactly the first
         `quorum` of them anchor their quote, since the gather rotates the
         bench by the prompt and a seat named by roster position could fall
         outside the seats it closes on.
         */
        const reviewers: string[] = [];
        const outcome = await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts: [],
            replyFor: ({ schema, modelId, },) => {
              if (schema !== 'archive_block_review')
                return ACCEPTABLE_NATURALNESS;
              if (!reviewers.includes(modelId,))
                reviewers.push(modelId,);
              return {
                disposition: 'source-supported',
                sourceQuote: reviewers.indexOf(modelId,) < quorum ? '窗边安静地睡觉' : '不存在的来源句子',
                replacementText: '',
                assessment: 'Source support claim.',
              };
            },
          },),
          modelIds: ROSTER,
          sourceText: '猫在窗边安静地睡觉。',
          targetText: 'The cat sleeps quietly by the window.',
          blockText: 'The cat sleeps quietly by the window.',
          blockOffset: 0,
          priorFindings: [],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome.kind,).toBe('retained',);
        expect(outcome.findings.join('\n',),).not.toContain('archive review left the block unresolved',);
        expect(outcome.findings,).toContain('archive block absolute naturalness accepted and challenged',);
      },
    },),
    it({
      name: 'SHIPS a revision in the archive quote style when the reviewer wrote straight quotes (class thirty-eight)',
      fn: async () => {
        /** Prompts, captured so the judges are shown to read the restored bytes. */
        const prompts: string[] = [];
        /** Reviewer's wording, straight quotes throughout. */
        const written = 'The cat\'s "nap" by the window.';
        /** Same wording in the archive's curly convention. */
        const restored = 'The cat’s “nap” by the window.';
        /** Actual review stage over a curly-quoted archive. */
        const outcome = await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts,
            replyFor: ({ schema, },) => schema === 'archive_block_review'
              ? {
                disposition: 'revise',
                sourceQuote: '',
                replacementText: written,
                assessment: 'Remove unsupported award claim.',
              }
              : { best: 1, assessment: 'Only supported details remain.', },
          },),
          modelIds: ROSTER,
          sourceText: '猫在窗边打盹。',
          targetText: 'The cat’s “nap” by the window and won an award.',
          blockText: 'The cat’s “nap” by the window and won an award.',
          blockOffset: 0,
          priorFindings: [],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome.kind,).toBe('revised',);
        expect(outcome.text,).toBe(restored,);
        expect(prompts.some(function judgedRestored(prompt,): boolean {
          return prompt.includes('CURRENT ARCHIVE BLOCK',) && prompt.includes(restored,);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'ACCEPTS quoted source evidence beside a revision instead of losing the reviewer to schema mismatch',
      fn: async () => {
        /** Prompts prove the reply reaches independent correction selection. */
        const prompts: string[] = [];
        /** Source-supported part remains while the unsupported award is removed. */
        const corrected = 'The cat sleeps by the window.';
        /** Actual review stage, including its custom guard and quorum accounting. */
        const outcome = await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts,
            replyFor: ({ schema, },) => schema === 'archive_block_review'
              ? {
                disposition: 'revise',
                sourceQuote: '猫在窗边睡觉',
                replacementText: corrected,
                assessment: 'The quoted source supports sleeping, but not the award.',
              }
              : { best: 1, assessment: 'Keep the supported detail and remove the unsupported claim.', },
          },),
          modelIds: ROSTER,
          sourceText: '猫在窗边睡觉。',
          targetText: 'The cat sleeps by the window and won an award.',
          blockText: 'The cat sleeps by the window and won an award.',
          blockOffset: 0,
          priorFindings: [],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome.kind,).toBe('revised',);
        expect(outcome.text,).toBe(corrected,);
        expect(prompts.some(function selected(prompt,): boolean {
          return prompt.includes('CURRENT ARCHIVE BLOCK',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'SELECTS correction without exposing producer model ids to candidate judges',
      fn: async () => {
        const prompts: string[] = [];
        const outcome = await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts,
            replyFor: ({ schema, },) => schema === 'archive_block_review'
              ? {
                disposition: 'revise',
                sourceQuote: '',
                replacementText: 'The cat sleeps by the window.',
                assessment: 'Remove unsupported award claim.',
              }
              : {
                best: 1,
                assessment: 'Only supported details remain.',
              },
          },),
          modelIds: ROSTER,
          sourceText: '猫在窗边睡觉。',
          targetText: 'The cat sleeps by the window and won an award.',
          blockText: 'The cat sleeps by the window and won an award.',
          blockOffset: 0,
          priorFindings: [],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(outcome.text,).toBe('The cat sleeps by the window.');
        const selectionPrompts = prompts.filter(function isSelection(prompt,): boolean {
          return prompt.includes('CURRENT ARCHIVE BLOCK',);
        },);
        // One prompt per judge of the window: quorum plus one asks three of the
        // four seats, which three answering judges carry on self-votes alone now
        // that no summed weight is needed (`candidate-select-fanout.ts`); the
        // whole bench of four was asked while three halves fell short of it.
        expect(selectionPrompts,).toHaveLength(3);
        for (const prompt of selectionPrompts) {
          for (const modelId of ROSTER)
            expect(prompt.includes(modelId,),).toBe(false,);
          // Ledger S5: the selector's first criterion once read a translator's
          // note or a gloss as a claim to remove; every judge now reads the
          // apparatus exemption and the narrative bound beside it.
          expect(prompt,).toContain('is not such a claim: retain it unless it is wrong',);
          expect(prompt,).toContain('WHAT HAPPENED IS NEVER APPARATUS',);
        }
      },
    },),
    it({
      name: 'WITHHOLDS a revision whose block shape is not the block\'s own, so a one-paragraph label ships as the archive wrote it (class seventy-seven, 2026-09-21: a four-block letter replaced the intro line)',
      fn: async () => {
        const prompts: string[] = [];
        /** Label block under review: one paragraph. */
        const label = 'English translation of the letter the cat sent:';
        /** Revision a reviewer wrote instead: four blocks of a letter, the third cut mid-sentence. */
        const letter = 'Good evening,\n\nI just finished my supper. I would like to sit on your lap.\n\nBut trust me, the sunny spot on the porch is far more\n\nWarm purrs\nThe Cat';
        const outcome = await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts,
            replyFor: ({ schema, },) => schema === 'archive_block_review'
              ? {
                disposition: 'revise',
                sourceQuote: '',
                replacementText: letter,
                assessment: 'The block carries no translation.',
              }
              : {
                best: 1,
                assessment: 'Only the letter carries content.',
              },
          },),
          modelIds: ROSTER,
          sourceText: '猫寄来的信。',
          targetText: `${label}\n\n> Good evening.`,
          blockText: label,
          blockOffset: `${label}\n\n> Good evening.`.indexOf(label,),
          priorFindings: [],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(outcome.text,).toBe(label);
        expect(outcome.findings
          .some(function refusedOnShape(finding,): boolean {
            return finding.startsWith('archive-revision-refused',)
              && finding.includes('paragraph',);
          },),).toBe(true,);
        // No selection round is bought over a slate the floor emptied.
        expect(prompts.some(function isSelection(prompt,): boolean {
          return prompt.includes('CURRENT ARCHIVE BLOCK',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'WITHHOLDS a removal of a footnote definition the page still references, so the note ships as the archive wrote it (class eighty-four, 2026-09-22: [^10] removed as unsupported, its marker left dangling)',
      fn: async () => {
        const prompts: string[] = [];
        /** Definition block under review: a translator's note the source never carried. */
        const note = '[^1]: Formerly the Cat Café, renamed the Kitten Café in 2025.';
        const outcome = await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts,
            replyFor: ({ schema, },) => schema === 'archive_block_review'
              ? {
                disposition: 'revise',
                sourceQuote: '',
                replacementText: '',
                assessment: 'The rename is not in the original.',
              }
              : {
                best: 1,
                assessment: 'Removal is the only candidate.',
              },
          },),
          modelIds: ROSTER,
          sourceText: '到了猫咖，我们坐下等待。',
          targetText: `At the Cat Café[^1], we sat down to wait.\n\n${note}`,
          blockText: note,
          blockOffset: `At the Cat Café[^1], we sat down to wait.\n\n${note}`.indexOf(note,),
          priorFindings: [],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(outcome.text,).toBe(note);
        expect(outcome.findings
          .some(function refusedOnFootnotes(finding,): boolean {
            return finding.startsWith('archive-revision-refused',)
              && finding.includes('unresolved-reference',);
          },),).toBe(true,);
        // No selection round is bought over a slate the floor emptied.
        expect(prompts.some(function isSelection(prompt,): boolean {
          return prompt.includes('CURRENT ARCHIVE BLOCK',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'READS A REVISION THAT SHOWS A READER NOTHING AS THE REMOVAL AN EMPTY ONE SPELLS: spaces, a zero-width '
        + 'space, a Hangul filler, or both among spaces reach the selectors as the removal and ship nothing, the '
        + 'same review, slate and outcome the empty replacement gets',
      fn: async () => {
        /** Block no source supports, which every reviewer proposes to remove. */
        const award = 'The cat won an award.';
        /**
         Reviews the block with every reviewer writing one replacement, and
         keeps what the selectors were shown.

         @param replacementText - replacement every reviewer writes

         @returns Outcome beside the selection prompts

         @example
         ```ts
         const removal = await reviewedWith('',);
         ```
         */
        async function reviewedWith(replacementText: string,) {
          /** Prompts of every exchange. */
          const prompts: string[] = [];
          /** Settled review. */
          const outcome = await runArchiveBlockReviewStage({
            client: scriptedClient({
              prompts,
              replyFor: ({ schema, },) => schema === 'archive_block_review'
                ? {
                  disposition: 'revise',
                  sourceQuote: '',
                  replacementText,
                  assessment: 'The award is not in the original.',
                }
                : {
                  best: 1,
                  assessment: 'Removal is the only supported answer.',
                },
            },),
            modelIds: ROSTER,
            sourceText: '猫在窗边睡觉。',
            targetText: `The cat sleeps by the window.\n\n${award}`,
            blockText: award,
            blockOffset: `The cat sleeps by the window.\n\n${award}`.indexOf(award,),
            priorFindings: [],
            signal: new AbortController().signal,
            exchangeTimeoutMs: HANG_STOP_MS,
            l,
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
          return {
            outcome,
            selection: prompts.filter(function isSelection(prompt,): boolean {
              return prompt.includes('CURRENT ARCHIVE BLOCK',);
            },),
          };
        }
        /** The removal as the wire spells it. */
        const removal = await reviewedWith('',);
        expect({
          kind: removal.outcome.kind,
          text: removal.outcome.text,
          selectors: removal.selection.length,
          shownAsRemoval: removal.selection.every(function showsRemoval(prompt,): boolean {
            return prompt.includes('[REMOVE BLOCK]',);
          },),
        },).toEqual({
          kind: 'revised',
          text: '',
          selectors: 3,
          shownAsRemoval: true,
        },);
        /** The same review with every reviewer writing a replacement that shows nothing. */
        const padded = await Promise.all([
          '   ',
          '\u{200B}',
          '\u{3164}',
          ' \u{200B}\u{3164} ',
        ].map(reviewedWith,),);
        expect(padded,).toEqual([
          removal,
          removal,
          removal,
          removal,
        ],);
      },
    },),
    it({
      name: 'RETAINS the block when every proposed correction drops contributor identity',
      fn: async () => {
        const prompts: string[] = [];
        const outcome = await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts,
            replyFor: ({ schema, },) => schema === 'archive_block_review'
              ? {
                disposition: 'revise',
                sourceQuote: '',
                replacementText: '',
                assessment: 'Remove the attribution block.',
              }
              : {
                best: 1,
                assessment: 'Remove the only candidate.',
              },
          },),
          modelIds: ROSTER,
          sourceText: '',
          targetText: 'Contributors for this entry: Cat Friend',
          blockText: 'Contributors for this entry: Cat Friend',
          blockOffset: 0,
          priorFindings: [],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        // Every candidate dropped the authoritative identity, so the slate is
        // empty, the selection declines without a call, and the original ships.
        expect(outcome.kind,).toBe('retained');
        expect(outcome.text,).toBe('Contributors for this entry: Cat Friend');
        expect(outcome.findings
          .includes('archive correction slate declined',),).toBe(true,);
      },
    },),
    it({
      name: 'RECORDS a naturalness rejection as findings on the retained block, never a second round',
      fn: async () => {
        const prompts: string[] = [];
        const outcome = await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts,
            replyFor: ({ schema, },) => schema === 'archive_block_review'
              ? {
                disposition: 'source-supported',
                sourceQuote: '窗边安静地睡觉',
                replacementText: '',
                assessment: 'Expected section supports this sentence.',
              }
              : {
                isAcceptable: false,
                findings: [{
                  paragraph: 1,
                  problem: 'Awkward archive wording.',
                },],
                assessment: 'Revision is required.',
              },
          },),
          modelIds: ROSTER,
          sourceText: '猫在窗边安静地睡觉。',
          targetText: 'The cat by the window quietly sleeping is.',
          blockText: 'The cat by the window quietly sleeping is.',
          blockOffset: 0,
          priorFindings: [],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        // The reviewer verdict is evidence, not withholding authority: the
        // anchored block ships with the located problem on its findings.
        expect(outcome.kind,).toBe('retained');
        expect(outcome.text,).toBe('The cat by the window quietly sleeping is.');
        expect(outcome.findings
          .includes('archive naturalness paragraph 1: Awkward archive wording.',),).toBe(true,);
        const reviewPrompts = prompts.filter(function isReview(prompt,): boolean {
          return prompt.includes('Review English archive wording',);
        },);
        expect(new Set(reviewPrompts,).size,).toBe(1);
      },
    },),
    it({
      name: 'RECORDS AN UNHEARD NATURALNESS REVIEW on the retained block, each responsibility on its own: a '
        + 'defect-discovery or acceptance-challenge round short of its quorum is a finding the block ships with, '
        + 'never withholding authority',
      fn: async () => {
        /**
         The retained block's findings when the replies to one naturalness
         responsibility, named by a phrase only its sheet carries, are cut
         for every seat but the first.

         @param sheetPhrase - phrase naming the starved responsibility's sheet

         @returns Which naturalness lines the findings carry
         */
        async function linesStarving(
          { sheetPhrase, }: { readonly sheetPhrase: string; },
        ): Promise<Readonly<Record<string, boolean>>> {
          /**
           Stage result with that responsibility's replies cut.
           */
          const outcome = await runArchiveBlockReviewStage({
            client: scriptedClient({
              prompts: [],
              unreadableFor: ({ modelId, prompt, },) => (modelId !== ROSTER[0]) && prompt.includes(sheetPhrase,),
              replyFor: anchoredReviewReplyFor,
            },),
            modelIds: ROSTER,
            sourceText: '猫在窗边安静地睡觉。',
            targetText: 'The cat sleeps quietly by the window.',
            blockText: 'The cat sleeps quietly by the window.',
            blockOffset: 0,
            priorFindings: [],
            signal: new AbortController().signal,
            exchangeTimeoutMs: HANG_STOP_MS,
            l,
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
          expect(outcome.kind,).toBe('retained',);
          return {
            discoveryShort: outcome.findings.includes('archive naturalness defect-discovery review quorum not met',),
            challengeShort: outcome.findings.includes('archive naturalness acceptance-challenge review quorum not met',),
            recorded: outcome.findings.includes('archive block retained with naturalness findings recorded',),
            accepted: outcome.findings.includes('archive block absolute naturalness accepted and challenged',),
          };
        }

        expect([
          await linesStarving({ sheetPhrase: 'Discover material defects', },),
          await linesStarving({ sheetPhrase: 'A prior editor accepted this exact candidate', },),
        ],).toEqual([
          {
            discoveryShort: true,
            challengeShort: false,
            recorded: true,
            accepted: false,
          },
          {
            discoveryShort: false,
            challengeShort: true,
            recorded: true,
            accepted: false,
          },
        ],);
      },
    },),
    it({
      name: 'RETAINS the block when judges decline every correction, buying no second round',
      fn: async () => {
        const prompts: string[] = [];
        const payloads: string[] = [];
        const outcome = await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts,
            payloads,
            replyFor: ({ schema, },) => schema === 'archive_block_review'
              ? {
                disposition: 'revise',
                sourceQuote: '',
                replacementText: 'The cat sleeps.',
                assessment: 'Remove unsupported award claim.',
              }
              : {
                best: 0,
                assessment: 'No correction is yet acceptable.',
              },
          },),
          modelIds: ROSTER,
          sourceText: '猫在睡觉。',
          targetText: 'The cat won an award.',
          blockText: 'The cat won an award.',
          blockOffset: 0,
          priorFindings: [],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        // Archive wording is the shipping default; the decline is recorded
        // evidence and the stage never re-asks.
        expect(outcome.kind,).toBe('retained');
        expect(outcome.text,).toBe('The cat won an award.');
        expect(outcome.findings
          .includes('archive correction slate declined',),).toBe(true,);
        const reviewPrompts = prompts.filter(function isReview(prompt,): boolean {
          return prompt.includes('Review English archive wording',);
        },);
        expect(new Set(reviewPrompts,).size,).toBe(1);
        expect(new Set(payloads,).size,).toBe(payloads.length);
      },
    },),
    it({
      name: 'SHOWS the reviewers and the correction selectors the declared names and the cited references '
        + '(ledger B28): the review judges archive wording against the original, and a detail a cited page '
        + 'states is the translator\'s knowledge rather than an addition',
      fn: async () => {
        const prompts: string[] = [];
        await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts,
            replyFor: ({ schema, },) => schema === 'archive_block_review'
              ? {
                disposition: 'revise',
                sourceQuote: '',
                replacementText: 'The cat sleeps by the window.',
                assessment: 'Remove unsupported sister claim.',
              }
              : { best: 1, assessment: 'Only supported details remain.', },
          },),
          modelIds: ROSTER,
          sourceText: '猫在窗边安静地睡觉。',
          targetText: `The cat sleeps by the window.\n\n${SISTER_BLOCK}`,
          blockText: SISTER_BLOCK,
          blockOffset: `The cat sleeps by the window.\n\n${SISTER_BLOCK}`.indexOf(SISTER_BLOCK,),
          priorFindings: [],
          identityContext: IDENTITY_CONTEXT,
          referenceContext: REFERENCE_CONTEXT,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        /** Prompts of the review round. */
        const reviews = prompts.filter(function isReview(prompt,): boolean {
          return prompt.includes('Review English archive wording',);
        },);
        /** Prompts of the correction slate. */
        const selections = prompts.filter(function isSelection(prompt,): boolean {
          return prompt.includes('CURRENT ARCHIVE BLOCK',);
        },);
        // Both rounds ran, so the case reads prompts that exist.
        expect(reviews.length,).toBeGreaterThan(0,);
        expect(selections.length,).toBeGreaterThan(0,);
        for (const prompt of [...reviews, ...selections,]) {
          expect(prompt,).toContain(asCaptured(IDENTITY_CONTEXT,),);
          expect(prompt,).toContain(asCaptured(REFERENCE_CONTEXT.split('\n',)[0] ?? '',),);
          expect(prompt,).toContain('CITED REFERENCES',);
        }
        // The selector's first criterion counts a cited page as support, or the
        // evidence beside it is overruled by the instruction to remove.
        expect(ARCHIVE_BLOCK_SELECTION_CRITERIA[0],).toContain('CITED REFERENCES',);
      },
    },),
    it({
      name: 'ANCHORS a retention in one cited reference page (ledger B28), and the naturalness read it buys '
        + 'sees the declared names',
      fn: async () => {
        const prompts: string[] = [];
        const outcome = await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts,
            replyFor: ({ schema, },) => schema === 'archive_block_review'
              ? {
                disposition: 'source-supported',
                sourceQuote: 'an older sister who was also a tabby',
                replacementText: '',
                assessment: 'The cited page states the sister.',
              }
              : ACCEPTABLE_NATURALNESS,
          },),
          modelIds: ROSTER,
          sourceText: '猫在窗边安静地睡觉。',
          targetText: `The cat sleeps by the window.\n\n${SISTER_BLOCK}`,
          blockText: SISTER_BLOCK,
          blockOffset: `The cat sleeps by the window.\n\n${SISTER_BLOCK}`.indexOf(SISTER_BLOCK,),
          priorFindings: [],
          identityContext: IDENTITY_CONTEXT,
          referenceContext: REFERENCE_CONTEXT,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome.kind,).toBe('retained',);
        expect(outcome.text,).toBe(SISTER_BLOCK,);
        expect(outcome.findings,).not.toContain('archive review discarded uncorroborated retention claim',);
        expect(outcome.findings,).toContain('archive block absolute naturalness accepted and challenged',);
        /** Prompts of the naturalness read. */
        const naturalness = prompts.filter(function isNaturalness(prompt,): boolean {
          return prompt.includes('publication-ready English',);
        },);
        expect(naturalness.length,).toBeGreaterThan(0,);
        for (const prompt of naturalness)
          expect(prompt,).toContain(asCaptured(IDENTITY_CONTEXT,),);
      },
    },),
    it({
      name: 'DISCARDS a retention quoting the archive\'s own words off an attested line, or a reference\'s address '
        + '(ledger B28): an attested line carries an archive quote, and only a page\'s text is support',
      fn: async () => {
        /** One review per quote: the archive's own words, then the page's address. */
        const outcomes = await Promise.all([SISTER_BLOCK, 'https://cats.example/posts/mittens',]
          .map(async function reviewQuoting(sourceQuote,) {
            return await runArchiveBlockReviewStage({
              client: scriptedClient({
                prompts: [],
                replyFor: ({ schema, },) => schema === 'archive_block_review'
                  ? {
                    disposition: 'source-supported',
                    sourceQuote,
                    replacementText: '',
                    assessment: 'Supported.',
                  }
                  : ACCEPTABLE_NATURALNESS,
              },),
              modelIds: ROSTER,
              sourceText: '猫在窗边安静地睡觉。',
              targetText: `The cat sleeps by the window.\n\n${SISTER_BLOCK}`,
              blockText: SISTER_BLOCK,
              blockOffset: `The cat sleeps by the window.\n\n${SISTER_BLOCK}`.indexOf(SISTER_BLOCK,),
              priorFindings: [],
              identityContext: IDENTITY_CONTEXT,
              referenceContext: REFERENCE_CONTEXT,
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
          },),);
        for (const outcome of outcomes) {
          expect(outcome.kind,).toBe('retained',);
          expect(outcome.findings,).toContain('archive review discarded uncorroborated retention claim',);
          expect(outcome.findings.join('\n',),).toContain('archive review left the block unresolved: 0 of',);
        }
      },
    },),
    it({
      name: 'KEEPS A RETENTION QUOTED WITHOUT THE LINE BREAKS OF A WRAPPED SECTION (ledger B715): zheermao101 '
        + '(run zheermao20) dropped a reviewer\'s sentence-long quote of a screenshot transcript that the picture '
        + 'had wrapped mid-sentence, and the block lost an assessment that supported the archive',
      fn: async () => {
        const outcome = await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts: [],
            replyFor: anchoredReviewReplyFor,
          },),
          modelIds: ROSTER,
          sourceText: '猫在窗边安\n静地睡觉。',
          targetText: 'The cat sleeps quietly by the window.',
          blockText: 'The cat sleeps quietly by the window.',
          blockOffset: 0,
          priorFindings: [],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome.kind,).toBe('retained',);
        expect(outcome.findings,).not.toContain('archive review discarded uncorroborated retention claim',);
        expect(outcome.findings.join('\n',),).not.toContain('archive review left the block unresolved',);
      },
    },),
    it({
      name: 'LOGS WHY EACH SOURCE-SUPPORTED REPLY WAS DROPPED, with the inputs that decided it (ledger B715): the '
        + 'run zheermao20 logged "2 eligible assessments from 4 heard reviews" and nothing said which reply failed '
        + 'which test, so a refused quote could not be told from a refused reviewer',
      fn: async () => {
        const { logger, lines, } = capturingLoggerPair();
        /**
         Seats asked for a review, in first-call order. The first anchors
         its quote; the second quotes words the section lacks; the rest
         quote too few characters to anchor anything.
         */
        const reviewers: string[] = [];
        const outcome = await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts: [],
            replyFor: ({ schema, modelId, },) => {
              if (schema !== 'archive_block_review')
                return ACCEPTABLE_NATURALNESS;
              if (!reviewers.includes(modelId,))
                reviewers.push(modelId,);
              return {
                disposition: 'source-supported',
                sourceQuote: [
                  '窗边安静地睡觉',
                  '不存在的来源句子',
                ][reviewers.indexOf(modelId,)] ?? '猫在',
                replacementText: '',
                assessment: 'Source support claim.',
              };
            },
          },),
          modelIds: ROSTER,
          sourceText: '猫在窗边安静地睡觉。',
          targetText: 'The cat sleeps quietly by the window.',
          blockText: 'The cat sleeps quietly by the window.',
          blockOffset: 0,
          priorFindings: [],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l: logger,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome.findings,).toContain('archive review discarded uncorroborated retention claim',);
        /** Lines of the drops, one per refused reply. */
        const dropped = lines.filter(function isDrop(line,): boolean {
          return line.includes('archive review dropped the source-supported reply of',);
        },);
        expect(dropped.length,).toBe(reviewers.length - 1,);
        /** The reviewer who quoted words the section lacks. */
        const [, absentReviewer, shortReviewer,] = reviewers;
        const absent = dropped.find(function names(line,): boolean {
          return line.includes(`reply of ${String(absentReviewer,)}:`,);
        },);
        expect(absent,).toContain('no span of the expected section and its picture transcripts (10 characters',);
        expect(absent,).toContain('longest opening of the quote found there 0 of 8',);
        expect(absent,).toContain('any of the 0 cited pages',);
        const short = dropped.find(function names(line,): boolean {
          return line.includes(`reply of ${String(shortReviewer,)}:`,);
        },);
        expect(short,).toContain('its quote shows 2 characters a reader sees, below the 4 an anchor needs',);
        expect(dropped.some(function namesAnchored(line,): boolean {
          return line.includes(`reply of ${String(reviewers[0],)}:`,);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'LOGS WHY AN EDITORIAL-CONTEXT REPLY WAS DROPPED when the block is not wholly apparatus (ledger B715)',
      fn: async () => {
        const { logger, lines, } = capturingLoggerPair();
        await runArchiveBlockReviewStage({
          client: scriptedClient({
            prompts: [],
            replyFor: ({ schema, },) => schema === 'archive_block_review'
              ? {
                disposition: 'editorial-context',
                sourceQuote: '',
                replacementText: '',
                assessment: 'Apparatus.',
              }
              : ACCEPTABLE_NATURALNESS,
          },),
          modelIds: ROSTER,
          sourceText: '猫在窗边安静地睡觉。',
          targetText: 'The cat sleeps quietly by the window.',
          blockText: 'The cat sleeps quietly by the window.',
          blockOffset: 0,
          priorFindings: [],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l: logger,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        /** Lines of the drops. */
        const dropped = lines.filter(function isDrop(line,): boolean {
          return line.includes('archive review dropped the editorial-context reply of',);
        },);
        expect(dropped.length,).toBeGreaterThan(0,);
        for (const line of dropped) {
          expect(line,).toContain('the block of 37 characters is not wholly apparatus',);
        }
      },
    },),
  ],
},);
