/**
 Tests that production pass boundaries invoke readiness guards before spend or persistence.

 Fixtures are cat-themed invention.

 @module
 */

import {
  mkdir,
  readdir,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { SettledArtifact, } from '@monochromatic-dev/translation-repair-artifact';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { NaturalnessCompletenessError, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  persistSettledEntry,
  preparePassEntry,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

import { NO_OUTSIDE_READS, } from './pass-outside-reads.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Pairing roster accepted by canned client.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
] as const;

/**
 Pipeline generation for disposable cache.
 */
const GENERATION = `sha256-tree-v1:${'a'.repeat(64,)}` as PipelineDigest;

/**
 Logger for production-boundary calls.
 */
const l = tagged({ tag: 'pass-readiness-boundaries-test', },);

/**
 Builds client whose pairing seats agree only first target block corresponds.

 @returns Client serving pairing JSON

 @example
 ```ts
 const client = pairingClient();
 ```
 */
function pairingClient(
  { sheets, pictureSupport, }: {
    readonly sheets?: string[];
    readonly pictureSupport?: string;
  } = {},
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText not used',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /** Stage schema deciding scripted reply. */
      const schema = request.responseFormat?.json_schema.name ?? '';
      /** Review sheet retained to prove which source evidence reached this boundary. */
      const sheet = JSON.stringify(request.messages,);
      if (schema === 'archive_block_review')
        sheets?.push(sheet,);
      /** Picture text is support only when preparation supplied it to this review. */
      const supported = (pictureSupport !== undefined) && sheet.includes(pictureSupport,);
      /** Pairing or archive-review value. */
      const value: unknown = schema === 'archive_block_review'
        ? {
          disposition: pictureSupport === undefined
            ? 'editorial-context'
            : supported ? 'source-supported' : 'revise',
          sourceQuote: supported ? pictureSupport : '',
          replacementText: '',
          assessment: pictureSupport === undefined
            ? 'This is an archive note.'
            : supported ? 'The picture carries this greeting.' : 'No source supports this greeting.',
        }
        : schema === 'candidate_ballot'
          ? { best: 1, assessment: 'Remove wording unsupported by the supplied source.', }
          : schema === 'absolute_naturalness_review'
            ? {
              isAcceptable: true,
              findings: [],
              assessment: 'The attribution is natural and publication-ready.',
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
 Builds artifact shape sufficient to prove persistence guard runs first.

 @returns Artifact whose contest declined archive and consolidation replaced nothing

 @example
 ```ts
 const artifact = declinedArtifact();
 ```
 */
function declinedArtifact(): SettledArtifact {
  return {
    comparison: [{
      sliceIndex: 0,
      incumbentKind: 'present',
      incumbentText: 'The cat naps.',
      repairText: 'The cat is asleep.',
      translateText: 'A cat naps.',
      laneRelation: 'both-differ',
      repairOutcome: { kind: 'decided', acceptedText: 'The cat is asleep.', },
      translateOutcome: { kind: 'decided', acceptedText: 'A cat naps.', },
      decisionComparison: { kind: 'comparable', verdict: 'different', },
      repairDelivery: { kind: 'replacement-shipped', },
      translateDelivery: { kind: 'replacement-shipped', },
    },],
    laneSelection: {
      kind: 'contested',
      slices: [{
        sliceIndex: 0,
        verdict: { kind: 'settled-neither', archive: 'declined', },
        ballots: [],
        usable: 10,
      },],
    },
    // SETTLED, AS EVERY PERSISTED ARTIFACT'S IS: `settledEntryArtifact`, the one
    // writer persistence is handed, records a contested selection over a settled
    // consolidation, and the naturalness guard throws its invariant on any other
    // pairing. The slice carries no polish, which is what that guard refuses once
    // the final-selection guard has let the declined archive through.
    consolidation: {
      kind: 'settled',
      slices: [{
        sliceIndex: 0,
        terminal: 'gate-kept-standing',
        shipped: { kind: 'unchanged', },
        rewrapped: false,
        demoted: false,
        verdicts: [],
        gate: { kind: 'not-asked', },
      },],
    },
  } as unknown as SettledArtifact;
}

/**
 Builds artifact whose final body polish lacks absolute review.

 @returns Artifact final-selection guard accepts and naturalness guard refuses

 @example
 ```ts
 const artifact = unreviewedNaturalnessArtifact();
 ```
 */
function unreviewedNaturalnessArtifact(): SettledArtifact {
  return {
    comparison: [{
      sliceIndex: 0,
      incumbentKind: 'present',
      incumbentText: 'The cat naps.',
      repairText: 'The cat is asleep.',
      translateText: 'A cat naps.',
      laneRelation: 'both-differ',
      repairOutcome: { kind: 'decided', acceptedText: 'The cat is asleep.', },
      translateOutcome: { kind: 'decided', acceptedText: 'A cat naps.', },
      decisionComparison: { kind: 'comparable', verdict: 'different', },
      repairDelivery: { kind: 'replacement-shipped', },
      translateDelivery: { kind: 'replacement-shipped', },
    },],
    laneSelection: {
      kind: 'contested',
      slices: [{
        sliceIndex: 0,
        verdict: { kind: 'lane-won', lane: 'translate', },
        ballots: [],
        usable: 2,
      },],
    },
    consolidation: {
      kind: 'settled',
      slices: [{
        sliceIndex: 0,
        terminal: 'gate-kept-standing',
        shipped: { kind: 'unchanged', },
        rewrapped: false,
        demoted: false,
        verdicts: [],
        gate: { kind: 'not-asked', },
        polish: {
          kind: 'settled',
          baseText: 'A cat naps.',
          proposedText: 'A cat naps.',
          text: 'A cat naps.',
          changed: false,
          refinersHeard: [],
          contributors: [],
          roundCount: 0,
          findings: [],
        },
      },],
    },
  } as unknown as SettledArtifact;
}

await describe({
  name: 'pass readiness boundaries',
  children: [
    it({
      name: 'REVIEWS roster-unclaimed editorial archive before lane work',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'pass-prepare-readiness-', },);
        const dir = scratch.path;
        const paired = await preparePassEntry({
          client: pairingClient(),
          entryId: 'Cat',
          entryCacheDir: dir,
          pipelineDigest: GENERATION,
          modelIds: ROSTER,
          sourceText: 'Cats nap.',
          targetText: 'Cats nap.\n\nTranslator: Cat Friend.',
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          outsideReads: NO_OUTSIDE_READS,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(paired.prepared.targetText,).toContain('Translator: Cat Friend.');
        expect(paired.prepared.alignmentFindings.some(function namesRetainedBlock(finding,): boolean {
          return finding.startsWith('archive block reviewed and retained: ');
        },),).toBe(true,);
      },
    },),
    it({
      name: 'PRESERVES archive picture translation by supplying corroborated source before block review',
      fn: async () => {
        /** Disposable pairing stores for this preparation. */
        await using scratch = await scratchDir({ prefix: 'pass-prepare-picture-', },);
        const dir = scratch.path;
        /** Exact image text absent from both pages as prose. */
        const pictureSupport = '手套猫：你好，姐姐。';
        /** Sheets the archive reviewers actually received. */
        const sheets: string[] = [];
        /** Component shared by source and archive, leaving its translation unclaimed. */
        const sourceText = `<PhotoScroll photos={['\${path}/photos/chat.webp']} />`;
        /** Archive block which must survive the review. */
        const translation = '> Mittens: Hello, sister.';
        /** Preparation through the production archive review and correction slate. */
        const paired = await preparePassEntry({
          client: pairingClient({ sheets, pictureSupport, },),
          entryId: 'Cat',
          entryCacheDir: dir,
          pipelineDigest: GENERATION,
          modelIds: [...ROSTER, SEAT_SYNTHETIC_VISION_WITHHELD, SEAT_SYNTHETIC_TEXT_EVERYWHERE,],
          sourceText,
          targetText: `${sourceText}\n\n${translation}`,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          outsideReads: NO_OUTSIDE_READS,
          readPictures: async ({ slices, },) => {
            expect(slices.some(function namesChat(slice,): boolean {
              return slice.source.text.includes('chat.webp',);
            },),).toBe(true,);
            return new Map([['chat.webp', {
              kind: 'corroborated',
              readings: ROSTER.map(function reading(modelId,) {
                return { modelId, text: pictureSupport, };
              },),
              overlap: 1,
            },],]);
          },
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(paired.prepared.targetText,).toContain(translation,);
        expect(sheets.length,).toBeGreaterThan(0,);
        expect(sheets.every(function sawSupport(sheet,): boolean {
          return sheet.includes(pictureSupport,)
            && sheet.includes('CORROBORATED PICTURE SOURCE SUPPORT',)
            && sheet.includes('chat.webp',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES body without absolute naturalness review before persistence writes page or artifact',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'pass-persist-naturalness-', },);
        const root = scratch.path;
        const publishDir = join(root, 'published',);
        const artifactsDir = join(root, 'artifacts',);
        await Promise.all([
          mkdir(publishDir,),
          mkdir(artifactsDir,),
        ],);

        let thrown: unknown;
        try {
          await persistSettledEntry({
            artifact: unreviewedNaturalnessArtifact(),
            slices: [{
              source: {
                kind: 'content',
                sliceIndex: 0,
                nodes: [],
                startOffset: 0,
                endOffset: '猫在睡觉。'.length,
                text: '猫在睡觉。',
              },
              target: {
                kind: 'content',
                sliceIndex: 0,
                nodes: [],
                startOffset: 0,
                endOffset: 'The cat naps.'.length,
                text: 'The cat naps.',
              },
            },],
            archiveText: 'The cat naps.',
            sourceText: '猫在睡觉。',
            entryId: 'Cat',
            publishDir,
            artifactsDir,
            l,
          },);
        }
        catch (error) {
          thrown = error;
        }

        const written = await Promise.all([
          readdir(publishDir,),
          readdir(artifactsDir,),
        ],);

        expect(thrown,).toBeInstanceOf(NaturalnessCompletenessError,);
        expect(written,).toEqual([
          [],
          [],
        ],);
      },
    },),

    it({
      name: 'RECORDS unendorsed archive as a finding instead of refusing final selection',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'pass-persist-readiness-', },);
        const root = scratch.path;
        const publishDir = join(root, 'published',);
        const artifactsDir = join(root, 'artifacts',);
        await Promise.all([
          mkdir(publishDir,),
          mkdir(artifactsDir,),
        ],);

        let thrown: unknown;
        try {
          await persistSettledEntry({
            artifact: declinedArtifact(),
            slices: [{
              source: {
                kind: 'content',
                sliceIndex: 0,
                nodes: [],
                startOffset: 0,
                endOffset: '猫在睡觉。'.length,
                text: '猫在睡觉。',
              },
              target: {
                kind: 'content',
                sliceIndex: 0,
                nodes: [],
                startOffset: 0,
                endOffset: 'The cat naps.'.length,
                text: 'The cat naps.',
              },
            },],
            archiveText: 'The cat naps.',
            sourceText: '猫在睡觉。',
            entryId: 'Cat',
            publishDir,
            artifactsDir,
            l,
          },);
        }
        catch (error) {
          thrown = error;
        }

        const written = await Promise.all([
          readdir(publishDir,),
          readdir(artifactsDir,),
        ],);

        // The contest non-endorsement no longer refuses the page: the next
        // boundary this minimal fixture trips is the structural naturalness
        // completeness, proving final selection recorded instead of throwing.
        expect(thrown,).toBeInstanceOf(NaturalnessCompletenessError,);
        expect(written,).toEqual([
          [],
          [],
        ],);
      },
    },),
  ],
},);
