/**
 Guards ledger X18: what the preparation's evidence rounds report (the
 reference attestation's findings, the page title lexicon's, and the findings for
 an entry whose info.yml was unreadable or whose death was of a kind nobody can
 read) reaches the entry's findings on every
 way out of the preparation. The way out through a
 corrected archive once rebuilt its findings from the re-preparation and
 dropped the attestation's.

 READ OFF THE CHANNEL THE ARTIFACT RECORDS (ledger B54): the preparation's
 `alignmentFindings`. These cases read a findings list beside the
 preparation, which the entry pipeline never read, so the findings they
 pinned reached no artifact.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { REFERENCE_ATTEST_RESPONSE_FORMAT, } from '@monochromatic-dev/translation-repair-reference';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary';
import { PAGE_TITLE_LEXICON_RESPONSE_FORMAT, } from '@monochromatic-dev/translation-repair-artifact';
import type { EntryInfoRecord, } from '@monochromatic-dev/translation-repair-insertion';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { ARCHIVE_BLOCK_REVIEW_RESPONSE_FORMAT, } from '@monochromatic-dev/translation-repair-archive';
import { preparePassEntry, } from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

import { NO_OUTSIDE_READS, } from './pass-outside-reads.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the preparation writes to, whose lines are not under test.
 */
const l = tagged({ tag: 'pass-prepare-findings-test', },);

/**
 Roster the preparation asks: four seats, so the correction slate has judges
 who did not write the revision, whose ballots count in full.
 */
const ROSTER: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 Cache generation the preparation's caches are stamped with.
 */
const DIGEST = 'pass-prepare-findings-test' as PipelineDigest;

/**
 Original repeating one title, in one block.
 */
const SOURCE = '小猫唱了《猫之歌》，又唱了《猫之歌》。';

/**
 Archive rendering of the original's one block.
 */
const RENDERED = 'The kitten sang the song twice.';

/**
 What the one page the original links says.
 */
const REFERENCE_LINES = '- reference 1 https://cats.example/posts/mittens ("Mittens"): Mittens had a brother who sat by the stove.';

/**
 Sentence the info.yml reader gave for a file that refused to parse.
 */
const UNREADABLE_REFUSAL = 'The info.yml of entry CatEntry at the pin refused to parse at line 2 column 1 (BAD_INDENT); '
  + 'corpus metadata parses upstream, so this signals corruption.';

/**
 What the info.yml reader answers for a file that refused to parse.
 */
const UNREADABLE_RECORD: EntryInfoRecord = {
  kind: 'unreadable',
  refusal: UNREADABLE_REFUSAL,
};

/**
 What the info.yml reader answers for a file whose death is a truth value.
 */
const TRUTH_VALUE_RECORD: EntryInfoRecord = {
  kind: 'read',
  info: { info: { died: true, }, },
};

/**
 The one finding a death that is a truth value puts on the artifact.
 */
const TRUTH_VALUE_FINDING = 'entry info: info.died is a truth value, which is neither text nor a finite number. '
  + 'No death is recorded for the entry.';

/**
 How the archive reviewers answer an unclaimed block.
 */
type Review = 'revise' | 'unanchored';

/**
 Builds a client answering every sheet the preparation asks: an attestation
 whose archive quote is not in the archive, so the round reports a discard;
 one rendering of the title; a review that revises the unclaimed block or
 claims support the original does not hold; a ballot for the first
 candidate; and one pairing for every pairing round.

 @param review - how the reviewers answer

 @param asked - sink for the name of every sheet asked

 @returns Client serving only structured calls

 @example
 ```ts
 const client = evidenceClient({ review: 'revise', asked: [], },);
 ```
 */
function evidenceClient(
  {
    review,
    asked,
  }: {
    readonly review: Review;
    readonly asked: string[];
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText not used',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Sheet asked.
       */
      const schema = request.responseFormat?.json_schema.name ?? '';
      asked.push(schema,);
      /**
       Reply per sheet.
       */
      const replies: Readonly<Record<string, unknown>> = {
        [REFERENCE_ATTEST_RESPONSE_FORMAT.json_schema.name]: {
          attested: [{
            archiveQuote: 'The cat wore a golden crown.',
            reference: 1,
            referenceQuote: 'Mittens had a brother',
          },],
        },
        [PAGE_TITLE_LEXICON_RESPONSE_FORMAT.json_schema.name]: {
          titles: [{ title: 1, rendering: 'Song of the Cat', },],
        },
        [ARCHIVE_BLOCK_REVIEW_RESPONSE_FORMAT.json_schema.name]: (review === 'revise')
          ? {
            disposition: 'revise',
            sourceQuote: '',
            replacementText: 'Aside B',
            assessment: 'Remove unsupported archive-only wording.',
          }
          : {
            disposition: 'source-supported',
            sourceQuote: 'Cats purr.',
            replacementText: '',
            assessment: 'The original supports it.',
          },
        candidate_ballot: {
          best: 1,
          assessment: 'Correction removes unsupported wording.',
        },
      };
      /**
       Reply to this sheet, one pairing where no other is scripted.
       */
      const value = replies[schema] ?? { pairs: [{ source: 0, target: 0, },], };
      if (!request.validate(value,))
        throw new Error(`scripted ${schema} reply failed validator`,);
      return { kind: 'ok', value, rawText: JSON.stringify(value,), };
    },
    quotas: async () => {
      throw new Error('quotas not used',);
    },
  };
}

/**
 Prepares the original against an archive, the original linking one page.

 @param targetText - archive page

 @param review - how the reviewers answer an unclaimed block

 @param entryInfo - what the entry's info.yml came to

 @returns Prepared archive, findings and the sheets asked

 @example
 ```ts
 const run = await prepared({ targetText: RENDERED, review: 'revise', entryInfo: UNREADABLE_RECORD, },);
 ```
 */
async function prepared(
  {
    targetText,
    review,
    entryInfo,
  }: {
    readonly targetText: string;
    readonly review: Review;
    readonly entryInfo: EntryInfoRecord;
  },
) {
  /**
   Name of every sheet asked.
   */
  const asked: string[] = [];
  /**
   Directory this case owns for its entry caches.
   */
  await using scratch = await scratchDir({ prefix: 'pass-prepare-findings-', },);
  const dir = scratch.path;
  /**
   The preparation.
   */
  const paired = await preparePassEntry({
    client: evidenceClient({ review, asked, },),
    entryId: 'CatEntry',
    entryCacheDir: dir,
    pipelineDigest: DIGEST,
    modelIds: ROSTER,
    sourceText: SOURCE,
    targetText,
    signal: new AbortController().signal,
    exchangeTimeoutMs: HANG_STOP_MS,
    l,
    outsideReads: {
      ...NO_OUTSIDE_READS,
      references: async () => REFERENCE_LINES,
      entryInfo: async () => entryInfo,
    },
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    targetText: paired.prepared.targetText,
    findings: paired.prepared.alignmentFindings,
    reviewed: asked.includes(ARCHIVE_BLOCK_REVIEW_RESPONSE_FORMAT.json_schema.name,),
  };
}

/**
 Whether each evidence round's report reached the findings.

 @param findings - preparation's findings

 @returns One flag per evidence round

 @example
 ```ts
 const reached = evidenceReported({ findings, },);
 ```
 */
function evidenceReported({ findings, }: { readonly findings: readonly string[]; },) {
  return {
    attestation: findings.some(function reportsDiscard(finding,): boolean {
      return finding.startsWith('reference attestation discarded',);
    },),
    pageTitles: findings.some(function reportsLexicon(finding,): boolean {
      return finding.startsWith('page title lexicon settled 1 of 1',);
    },),
    unreadableEntryInfo: findings.includes(`entry info: ${UNREADABLE_REFUSAL} No death is recorded for the entry.`,),
  };
}

/**
 The findings the info.yml reader's record put on the preparation.

 @param findings - preparation's findings

 @returns Findings that begin with the info.yml reader's label

 @example
 ```ts
 const own = entryInfoFindingsIn({ findings, },);
 ```
 */
function entryInfoFindingsIn({ findings, }: { readonly findings: readonly string[]; },): readonly string[] {
  return findings.filter(function isEntryInfoFinding(finding,): boolean {
    return finding.startsWith('entry info:',);
  },);
}

await describe({
  name: 'preparePassEntry keeps its evidence findings on every way out (ledger X18)',
  children: [
    it({
      name: 'KEEPS THEM when the archive leaves no block unclaimed, and asks no review',
      fn: async () => {
        /**
         Preparation over an archive the pairing claims whole.
         */
        const claimed = await prepared({ targetText: RENDERED, review: 'revise', entryInfo: UNREADABLE_RECORD, },);
        expect({
          reviewed: claimed.reviewed,
          ...evidenceReported({ findings: claimed.findings, },),
        },).toEqual({
          reviewed: false,
          attestation: true,
          pageTitles: true,
          unreadableEntryInfo: true,
        },);
      },
    },),
    it({
      name: 'KEEPS THEM when the review leaves the archive as it stands',
      fn: async () => {
        /**
         Preparation whose review claims support the original lacks.
         */
        const retained = await prepared({ targetText: `${RENDERED}\n\nAside A`, review: 'unanchored', entryInfo: UNREADABLE_RECORD, },);
        expect({
          reviewed: retained.reviewed,
          archiveStands: retained.targetText.includes('Aside A',),
          ...evidenceReported({ findings: retained.findings, },),
        },).toEqual({
          reviewed: true,
          archiveStands: true,
          attestation: true,
          pageTitles: true,
          unreadableEntryInfo: true,
        },);
      },
    },),
    it({
      name: 'KEEPS THEM when the review corrects the archive and the preparation runs again',
      fn: async () => {
        /**
         Preparation whose review revises the unclaimed block.
         */
        const corrected = await prepared({ targetText: `${RENDERED}\n\nAside A`, review: 'revise', entryInfo: UNREADABLE_RECORD, },);
        expect({
          reviewed: corrected.reviewed,
          corrected: corrected.targetText.includes('Aside B',),
          ...evidenceReported({ findings: corrected.findings, },),
        },).toEqual({
          reviewed: true,
          corrected: true,
          attestation: true,
          pageTitles: true,
          unreadableEntryInfo: true,
        },);
      },
    },),
    it({
      name: 'KEEPS THE FINDING OF A died THAT IS A TRUTH VALUE when the archive leaves no block unclaimed, and asks no review',
      fn: async () => {
        /**
         Preparation over an archive the pairing claims whole.
         */
        const claimed = await prepared({ targetText: RENDERED, review: 'revise', entryInfo: TRUTH_VALUE_RECORD, },);
        expect({
          reviewed: claimed.reviewed,
          entryInfoFindings: entryInfoFindingsIn({ findings: claimed.findings, },),
        },).toEqual({
          reviewed: false,
          entryInfoFindings: [TRUTH_VALUE_FINDING,],
        },);
      },
    },),
    it({
      name: 'KEEPS THE FINDING OF A died THAT IS A TRUTH VALUE when the review leaves the archive as it stands',
      fn: async () => {
        /**
         Preparation whose review claims support the original lacks.
         */
        const retained = await prepared({
          targetText: `${RENDERED}\n\nAside A`,
          review: 'unanchored',
          entryInfo: TRUTH_VALUE_RECORD,
        },);
        expect({
          reviewed: retained.reviewed,
          archiveStands: retained.targetText.includes('Aside A',),
          entryInfoFindings: entryInfoFindingsIn({ findings: retained.findings, },),
        },).toEqual({
          reviewed: true,
          archiveStands: true,
          entryInfoFindings: [TRUTH_VALUE_FINDING,],
        },);
      },
    },),
    it({
      name: 'KEEPS THE FINDING OF A died THAT IS A TRUTH VALUE when the review corrects the archive and the preparation runs again',
      fn: async () => {
        /**
         Preparation whose review revises the unclaimed block.
         */
        const corrected = await prepared({
          targetText: `${RENDERED}\n\nAside A`,
          review: 'revise',
          entryInfo: TRUTH_VALUE_RECORD,
        },);
        expect({
          reviewed: corrected.reviewed,
          corrected: corrected.targetText.includes('Aside B',),
          entryInfoFindings: entryInfoFindingsIn({ findings: corrected.findings, },),
        },).toEqual({
          reviewed: true,
          corrected: true,
          entryInfoFindings: [TRUTH_VALUE_FINDING,],
        },);
      },
    },),
  ],
},);
