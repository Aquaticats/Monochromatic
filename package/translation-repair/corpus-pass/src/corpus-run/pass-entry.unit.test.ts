/**
 Tests for settling one entry from both lanes.

 WHAT THESE COVER that neither the driver's nor the builder's tests can: that
 an entry reaches disk as ONE version 2 artifact over ONE preparation, that a
 failed entry keeps the slices it bought and writes nothing, and that an abort
 landing after the lanes return stops the write rather than being noticed only
 on the next call.

 Everything runs against throwaway directories under the system temp root, so
 no case can reach a real artifacts directory or a real slice cache.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  chmod,
  mkdir,
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/module-async-time/ts';
import { caughtValueText, } from '@monochromatic-dev/module-caught-value/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { RunClient, } from '@monochromatic-dev/translation-repair-run';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary';
import { readEntryInfoRecord, } from '@monochromatic-dev/translation-repair-insertion';
import { parseSettledTwoLaneArtifact, } from '@monochromatic-dev/translation-repair-artifact';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  EveryProviderDryError,
  messageText,
} from '@monochromatic-dev/translation-repair-provider';
import { fixedPagePath, } from '@monochromatic-dev/translation-repair-page-check';
import { settleEntry, } from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  HOUR_MS,
  stubWallClock,
  WALL_START_MS,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/wall-clock-stub.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

import { NO_OUTSIDE_READS, } from './pass-outside-reads.test-fixture.ts';
import { NO_PICTURE_SOURCES, } from './pass-picture-sources.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import {
  dialSaying,
  PASS_FALLBACK_OVERLAP,
} from '@monochromatic-dev/translation-repair-slice-cache/ts/corpus-run/slice-overlap-dial.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import { metersAllWet, } from '@monochromatic-dev/translation-repair-provider/ts/meter-states.test-fixture.ts';

/**
 Built pipeline these fixtures claim to have run under.
 */
const DIGEST = `sha256-tree-v1:${'c'.repeat(64,)}` as PipelineDigest;

/**
 Clone and commit every entry is settled at, invented, so a record naming
 any other commit shows.
 */
const ENTRY_PIN = {
  cloneDir: '/cats/corpus/clone',
  commitSha: 'feedface'.repeat(5,),
} as const;

/**
 Successful model calls in flight for one pass stage.
 */
type PassStageConcurrency = {
  now: number;
  peak: number;
};

/**
 Per-slice driver activity observed at shared client boundary.
 */
type PassConcurrency = {
  readonly repair: PassStageConcurrency;
  readonly refine: PassStageConcurrency;
  readonly translate: PassStageConcurrency;
  readonly contest: PassStageConcurrency;
  readonly consolidate: PassStageConcurrency;
};

/**
 Builds zeroed instruments for one pass arm.

 @returns Independent activity counters per per-slice driver

 @example
 ```ts
 const activity = emptyPassConcurrency();
 ```
 */
function emptyPassConcurrency(): PassConcurrency {
  return {
    repair: {
      now: 0,
      peak: 0,
    },
    refine: {
      now: 0,
      peak: 0,
    },
    translate: {
      now: 0,
      peak: 0,
    },
    contest: {
      now: 0,
      peak: 0,
    },
    consolidate: {
      now: 0,
      peak: 0,
    },
  };
}

/**
 Sentence every scripted translator returns for the first section.
 */
const FRESH = 'The cat naps on the windowsill.';

/**
 Sentence every translator returns for second section.
 */
const BIRD_FRESH = 'A bird sits on the windowsill.';

/**
 Literal consolidation paragraph long enough for naturalness stage.
 */
const POLISH_BASE_PARAGRAPH = 'The cat faced life proactively and spent a good time with everyone, while doing her best to stay hopeful and connected to the people around her.';

/**
 Idiomatic final paragraph approved by polish gate.
 */
const POLISH_FINAL_PARAGRAPH = 'The cat maintained a positive outlook on life and spent some good times with everyone, doing her best to stay hopeful and connected to those around her.';

/**
 That paragraph as the page carries it: wrapped at its semantic boundary
 before the polish gate judged it, so the gate approved the bytes that ship.
 */
const POLISH_FINAL_WRAPPED = 'The cat maintained a positive outlook on life and spent some good times with everyone,\n'
  + 'doing her best to stay hopeful and connected to those around her.';

/**
 Stable phrase identifying literal consolidation after semantic wrapping.
 */
const POLISH_BASE_NEEDLE = 'faced life proactively';

/**
 Stable phrase identifying idiomatic polish after semantic wrapping.
 */
const POLISH_FINAL_NEEDLE = 'maintained a positive outlook on life';

/**
 Complete consolidated slice before naturalness stage.
 */
const POLISH_BASE = `## Section one\n\n${POLISH_BASE_PARAGRAPH}`;

/**
 Original document: two sections, each one paragraph.
 */
const SOURCE_TEXT = `## 第一节

猫猫在窗台上打盹。

## 第二节

窗台上有一只鸟。
`;

/**
 Translation as it stands, awkward but complete.
 */
const TARGET_TEXT = `## Section one

The cat is doing the sleeping on the windowsill throughout the quiet afternoon while the sunlight is moving slowly across the room.

## Section two

On the windowsill there is being a bird that is watching the garden for a long time while the cat continues sleeping nearby.
`;

/**
 Entry every case settles.
 */
const ENTRY = {
  id: 'CatEntry1',
  sourceText: SOURCE_TEXT,
  targetText: TARGET_TEXT,
};

/** Unsupported archive-only factual block removed before lanes. */
const UNSUPPORTED_ARCHIVE_BLOCK = 'The cat won an interplanetary award.';

/** Entry proving preparation correction reaches artifact and page boundaries. */
const ARCHIVE_REVISION_ENTRY = {
  id: 'CatEntryArchiveRevision',
  sourceText: SOURCE_TEXT,
  targetText: `${TARGET_TEXT.trimEnd()}\n\n${UNSUPPORTED_ARCHIVE_BLOCK}\n`,
};

/**
 Source page made entirely of visible metadata.
 */
const FRONT_MATTER_SOURCE = '---\nname: 猫猫\ninfo:\n  alias: 猫猫\n---\n';

/**
 Archive metadata carrying entry id as visible name and no Latin rendering
 beside it: the one shape the lanes still render (the owner's rule of
 2026-09-08), since nothing in it makes the directory id stand for the name.
 */
const FRONT_MATTER_TARGET = '---\nname: CatFrontMatter\ninfo:\n  alias: 猫猫\n---\n';

/**
 Source-faithful metadata rendering scripted for ensemble.
 */
const FRONT_MATTER_FRESH = '---\nname: Maomao\ninfo:\n  alias: Maomao\n---\n';

/**
 Entry exercising complete metadata publication path.
 */
const FRONT_MATTER_ENTRY = {
  id: 'CatFrontMatter',
  sourceText: FRONT_MATTER_SOURCE,
  targetText: FRONT_MATTER_TARGET,
};

/**
 Entry whose archive translated its metadata, which therefore stands: the
 lanes never see slice zero and the page carries the archive's bytes.
 */
const FRONT_MATTER_STANDING_ENTRY = {
  id: 'CatFrontMatterStanding',
  sourceText: `${FRONT_MATTER_SOURCE}${SOURCE_TEXT}`,
  targetText: `${FRONT_MATTER_FRESH}${TARGET_TEXT}`,
};

/**
 Entry inserting source metadata into target page that has none.
 */
const FRONT_MATTER_SOURCE_ONLY_ENTRY = {
  id: 'CatFrontMatterInsertion',
  sourceText: `${FRONT_MATTER_SOURCE}${SOURCE_TEXT}`,
  targetText: TARGET_TEXT,
};

/**
 Entry whose archive carries invisible separator outside replacements.
 */
const INVISIBLE_ENTRY = {
  id: 'CatEntryInvisible',
  sourceText: SOURCE_TEXT,
  targetText: TARGET_TEXT.replace('## Section two', '\uFEFF## Section two',),
};

/**
 Entry the cleanup case settles, under its own id.

 SEPARATE because that case reads what was PRINTED, and the runner runs cases
 concurrently in one process: a capture keyed on nothing collects whatever
 other cases logged while it was installed. Filtering by an id no other case
 uses is what makes the reading this case's own.
 */
const CLEANUP_ENTRY = {
  id: 'CatEntry2',
  sourceText: SOURCE_TEXT,
  targetText: TARGET_TEXT,
};

/**
 Visual reference whose asset is intentionally absent from pinned corpus.
 */
const MISSING_VISUAL = `<PhotoScroll photos={[ '\${path}/photos/missing.webp' ]} />`;

/**
 Entry proving unresolved referenced visual stops before lanes.
 */
const MISSING_VISUAL_ENTRY = {
  id: 'CatEntryMissingVisual',
  sourceText: `${SOURCE_TEXT}\n\n${MISSING_VISUAL}\n`,
  targetText: `${TARGET_TEXT}\n\n${MISSING_VISUAL}\n`,
};

/**
 Original carrying a linked factual paragraph absent from archive.
 */
const GAP_SOURCE_TEXT = `猫猫在睡觉。

猫猫最后的[记录](https://example.test/cat-record)说她去世时 26 岁。
`;

/**
 Archive omitting linked factual paragraph.
 */
const GAP_TARGET_TEXT = 'The cat sleeps.\n';

/**
 Entry reproducing known source gap at publication seam.
 */
const GAP_ENTRY = {
  id: 'CatEntryGap',
  sourceText: GAP_SOURCE_TEXT,
  targetText: GAP_TARGET_TEXT,
};

/** Entry whose aligned archive wording dropped source destination. */
const LINK_RECOVERY_ENTRY = {
  id: 'CatEntryLinkRecovery',
  sourceText: '[猫猫的记录](https://example.test/cat-record)。',
  targetText: 'The archive mentions the cat record.',
};

/**
 Gap plus reviewed visual for successful guard traversal control.
 */
const REVIEWED_VISUAL_ENTRY = {
  id: 'CatEntryReviewedVisual',
  sourceText: `${GAP_SOURCE_TEXT}\n\n${MISSING_VISUAL}\n`,
  targetText: `${GAP_TARGET_TEXT}\n\n${MISSING_VISUAL}\n`,
};

/**
 Rendering scripted for linked source-only paragraph once admitted.
 */
const GAP_FRESH = "The cat's final [record](https://example.test/cat-record) says she died at age 26.";

/**
 Entry whose source-only block is already rendered inside target page.
 */
const CARRIED_ENTRY = {
  id: 'CatEntryCarriedGap',
  sourceText: GAP_SOURCE_TEXT,
  targetText: `${GAP_TARGET_TEXT}\n${GAP_FRESH}\n`,
};

/**
 Clause of the archive's first section the original lacks and the cited page
 states: the detail the attestation bench attests.
 */
const CITED_ARCHIVE_QUOTE = 'throughout the quiet afternoon while the sunlight is moving slowly across the room';

/**
 Words of the cited page stating that detail.
 */
const CITED_REFERENCE_QUOTE = 'She slept there all through the quiet afternoon, while the sunlight moved slowly across the room.';

/**
 Address of the one page the original cites, which only the reference block
 names.
 */
const CITED_ADDRESS = 'https://cats.example/posts/windowsill';

/**
 What the outside reader says the cited page says, one reference line.
 */
const CITED_REFERENCE = `- reference 1 ${CITED_ADDRESS} ("The windowsill cat"): ${CITED_REFERENCE_QUOTE}`;

/**
 Entry the cited-reference case settles, under its own id.
 */
const CITED_ENTRY = {
  id: 'CatEntryCited',
  sourceText: SOURCE_TEXT,
  targetText: TARGET_TEXT,
};

/**
 What the archive's translators wrote over its second section: everything
 under the note (以下) is the English original (原文, 英文), and neither
 whole-page mark appears, so it seals a span and declines nothing.
 */
const SPAN_NOTE_TEXT = '以下内容原文为英文';

/**
 Archive paragraph under that note, which ships as the archive has it.
 */
const SEALED_PARAGRAPH = 'On the windowsill there is being a bird that is watching the garden for a long time while the cat '
  + 'continues sleeping nearby.';

/**
 Entry whose archive seals its second section as the English original.
 */
const SEALED_ENTRY = {
  id: 'CatEntrySealedSpan',
  sourceText: SOURCE_TEXT,
  targetText: TARGET_TEXT.replace(
    '## Section two\n\n',
    `## Section two\n\n<!-- ${SPAN_NOTE_TEXT} -->\n\n`,
  ),
};

/**
 Original whose one paragraph names a favourite colour.
 */
const US_FORMS_SOURCE = '猫猫最喜欢的颜色是灰色。\n';

/**
 Rendering every scripted translator returns for it, spelled the United
 States way, which the Canadian forms page pass corrects.
 */
const US_FORMS_FRESH = 'The cat\'s favorite color was gray.';

/**
 That rendering as the page pass leaves it.
 */
const CANADIAN_FORMS_FRESH = 'The cat\'s favourite colour was grey.';

/**
 Entry whose settled wording the page passes rewrite.
 */
const US_FORMS_ENTRY = {
  id: 'CatEntryUsForms',
  sourceText: US_FORMS_SOURCE,
  targetText: 'Of all of the colours, the cat is liking the grey one the most out of every one of them.\n',
};

/**
 Entry whose two sections repeat one source sentence, worded differently by
 the lanes, which the consistency pass puts to its judges.
 */
const REPEATED_SENTENCE_ENTRY = {
  id: 'CatEntryRepeat',
  sourceText: '## 第一节\n\n窗台上的阳光让小猫睡着了。\n\n## 第二节\n\n傍晚的时候，窗台上的阳光让小猫睡着了，小鱼干也笑了。\n',
  targetText: '## Section one\n\nThe sunny window put the kitten to sleep.\n\n## Section two\n\n'
    + 'In the evening, the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too.\n',
};

/**
 Words only a consolidation producer's sheet carries, which tell it apart
 from a translator's under the one translation schema they share.
 */
const CONSOLIDATION_MARK = 'Two English renderings of this passage already exist';

/**
 Fence label over the attested details on a translator's sheet.
 */
const ATTESTED_DETAILS_LABEL = 'ATTESTED DETAILS';

/**
 One sheet a stage sent, under the schema that asked.
 */
type SentSheet = {
  readonly schema: string;
  readonly content: string;
};

/**
 Which sheets a case reads: the contest's, the consolidation producer's, or a
 translator's.
 */
type SheetKind = 'contest' | 'consolidation' | 'translator';

/**
 Whether a sheet is of one kind.

 @param sheet - sheet sent

 @param kind - kind asked about

 @returns Whether the sheet is of that kind

 @example
 ```ts
 const isContest = isSheetOf({ sheet, kind: 'contest', },);
 ```
 */
function isSheetOf(
  {
    sheet,
    kind,
  }: {
    readonly sheet: SentSheet;
    readonly kind: SheetKind;
  },
): boolean {
  if (kind === 'contest')
    return sheet.schema === 'lane_contest';
  if (sheet.schema !== 'translation_report')
    return false;
  return (kind === 'consolidation') === sheet.content.includes(CONSOLIDATION_MARK,);
}

/**
 What the sheets of one kind carried of the cited page: how many were sent,
 how many named the page, and how many listed attested details.

 @param sheets - every sheet sent

 @param kind - kind read

 @returns The three counts

 @example
 ```ts
 const contest = citedEvidenceIn({ sheets, kind: 'contest', },);
 ```
 */
function citedEvidenceIn(
  {
    sheets,
    kind,
  }: {
    readonly sheets: readonly SentSheet[];
    readonly kind: SheetKind;
  },
): {
  readonly sent: number;
  readonly namingThePage: number;
  readonly listingAttested: number;
} {
  /**
   Sheets of that kind.
   */
  const ofKind = sheets.filter(function isOfKind(sheet,): boolean {
    return isSheetOf({
      sheet,
      kind,
    },);
  },);
  return {
    sent: ofKind.length,
    namingThePage: ofKind.filter(function namesThePage({ content, },): boolean {
      return content.includes(CITED_ADDRESS,);
    },).length,
    listingAttested: ofKind.filter(function listsAttested({ content, },): boolean {
      return content.includes(ATTESTED_DETAILS_LABEL,) && content.includes(CITED_ARCHIVE_QUOTE,);
    },).length,
  };
}

/**
 Coverage behavior of pass fixture.
 */
type CoverageScript = 'lost' | 'absent' | 'full';

/**
 Renders one slice the way a translator that respected block structure would.

 @param content - translator prompt, which carries the slice original

 @returns Rendering for that slice

 @example
 ```ts
 const rendering = renderingFor({ content, },);
 ```
 */
function renderingFor({ content, }: { readonly content: string; },): string {
  if (content.includes('name: 猫猫',))
    return FRONT_MATTER_FRESH;
  if (content.includes('## 第一节',))
    return `## Section one\n\n${FRESH}`;
  if (content.includes('## 第二节',))
    return `## Section two\n\n${BIRD_FRESH}`;
  if (content.includes('missing.webp',))
    return MISSING_VISUAL;
  if (content.includes('cat-record',))
    return GAP_FRESH;
  if (content.includes('最喜欢的颜色',))
    return US_FORMS_FRESH;
  return FRESH;
}

/**
 Finds the one-based candidate index whose rendering carries a needle.

 @param content - judge user message

 @returns One-based index, or zero when no candidate carries it

 @example
 ```ts
 const best = pickCandidate({ content, },);
 ```
 */
function pickCandidate({ content, }: { readonly content: string; },): number {
  /**
   Sheet split at each candidate heading; the first piece is the evidence.
   */
  const [, ...blocks] = content.split('CANDIDATE ',);
  for (const block of blocks) {
    /**
     Heading line carrying this candidate's number.
     */
    const [heading = '',] = block.split('\n',);

    /**
     Number the heading states.
     */
    const index = Math.trunc(Number(heading,),);
    if (Number.isInteger(index,)
      && (block.includes(POLISH_FINAL_NEEDLE,)
        || block.includes(POLISH_BASE_NEEDLE,)
        || block.includes(FRESH,)
        || block.includes(BIRD_FRESH,)
        || block.includes(GAP_FRESH,)
        || block.includes(US_FORMS_FRESH,)
        || block.includes(FRONT_MATTER_FRESH,)))
      return index;
  }
  return 0;
}

/**
 Scripted reply for one stage.

 The repair lane's critics find nothing, so that lane keeps the archive; the
 translate lane renders each slice afresh and its judges pick that rendering.
 The two lanes therefore disagree by construction, which is what makes the
 artifact worth reading.

 @param schema - schema name the stage asked for

 @param content - everything the stage sent

 @param coverageScript - whether coverage roster answers absent or loses voice

 @param consolidation - whether translation schema belongs to third rendering

 @param polishScript - whether final naturalness rewrite is exercised

 @param contestChoice - lane scripted contest endorses

 @returns Wire value for that stage

 @throws {@link Error} when a stage this script does not serve asks

 @example
 ```ts
 const value = replyFor({ schema: 'critic_report', content, },);
 ```
 */
function replyFor(
  {
    schema,
    content,
    coverageScript,
    consolidation = false,
    polishScript = false,
    contestChoice = 'translate',
  }: {
    readonly schema: string;
    readonly content: string;
    readonly coverageScript: CoverageScript;
    readonly consolidation?: boolean;
    readonly polishScript?: boolean;
    readonly contestChoice?: 'repair' | 'translate';
  },
): unknown {
  // THE PAIRING ROUND RUNS BEFORE EITHER LANE, so the script has to serve it or
  // every entry falls back to scoring and, worse, a resumed entry re-buys the
  // round it could not cache. An empty pairing is a legal answer meaning nothing
  // corresponds, which keeps this fixture's slicing exactly as it was before the
  // stage existed.
  if (schema === 'block_pairing') {
    if (content.includes(UNSUPPORTED_ARCHIVE_BLOCK,)) {
      return { pairs: [
        { source: 0, target: 0, },
        { source: 1, target: 1, },
      ], };
    }
    return content.includes('cat-record',)
      ? { pairs: [{ source: 0, target: 0, },], }
      : { pairs: [], };
  }
  if (schema === 'archive_block_review') {
    return {
      disposition: 'revise',
      sourceQuote: '',
      replacementText: '',
      assessment: 'Remove unsupported archive-only factual wording.',
    };
  }
  if (schema === 'coverage_report') {
    if (coverageScript === 'lost')
      throw new Error('scripted coverage voice lost',);
    return coverageScript === 'full'
      ? {
        coverage: 'full',
        copiedSpan: GAP_FRESH,
        assessment: 'scripted source passage carried',
      }
      : {
        coverage: 'none',
        copiedSpan: '',
        assessment: 'scripted source passage absent',
      };
  }
  if (schema === 'critic_report')
    return { issues: [], };
  if (schema === 'refine_report') {
    return (polishScript && content.includes(POLISH_BASE_NEEDLE,))
      ? {
        rewrites: [{
          paragraph: 1,
          newText: POLISH_FINAL_PARAGRAPH,
        },],
      }
      : { rewrites: [], };
  }
  if (schema === 'translation_report') {
    if (polishScript
      && (consolidation || content.includes('CANDIDATE "repair"',))
      && content.includes('## 第一节',))
      return { translation: POLISH_BASE, };
    return { translation: renderingFor({ content, },), };
  }
  if (schema === 'candidate_ballot') {
    return {
      best: content.includes('CURRENT ARCHIVE BLOCK',) ? 1 : pickCandidate({ content, }),
      assessment: 'scripted',
    };
  }

  // THE CONTEST RUNS AFTER BOTH LANES, over the slices they worded differently.
  // Serving it is what makes this fixture exercise a DECIDED contest rather than
  // an unheard one: without a script every voice is lost, the roster never
  // reaches quorum, and the artifact records the pass reaching the stage instead
  // of the stage reaching an answer.
  if (schema === 'lane_contest') {
    return {
      verdict: contestChoice,
      unsupported: [],
      dropped: [],
      assessment: 'scripted',
    };
  }
  if (schema === 'consolidate_gate') {
    return {
      verdict: 'consolidated',
      unsupported: [],
      dropped: [],
      assessment: 'scripted consolidation gate',
    };
  }
  if (schema === 'consolidation_polish_gate') {
    return {
      verdict: 'polished',
      unsupported: [],
      dropped: [],
      assessment: 'scripted fidelity-first naturalness gate',
    };
  }
  if (schema === 'absolute_naturalness_review') {
    return {
      isAcceptable: true,
      findings: [],
      assessment: 'whole candidate is publication-ready',
    };
  }

  // THE ATTESTATION ROUNDS RUN ONLY WHERE A CITED PAGE SAYS SOMETHING, so only
  // the cited-reference case reaches them: every voice attests the one detail
  // the page states, quoting both sides word for word, and then confirms it.
  if (schema === 'reference_attest') {
    return content.includes(CITED_REFERENCE_QUOTE,)
      ? {
        attested: [{
          archiveQuote: CITED_ARCHIVE_QUOTE,
          reference: 1,
          referenceQuote: CITED_REFERENCE_QUOTE,
        },],
      }
      : { attested: [], };
  }
  if (schema === 'reference_attest_confirm')
    return { confirmed: [1,], };
  throw new Error(`no script for ${schema}`,);
}

/**
 Client serving both lanes from one script.

 @param served - schema names appended in call order

 @param failOnSchema - schema every call of which throws, standing in for a
 provider that is down for one stage; absent means the script never fails

 @param failWith - what those calls throw; a plain error when absent

 @param activity - optional admission instrument for every per-slice driver

 @param coverageScript - whether coverage roster answers or loses every voice

 @param polishScript - whether final naturalness rewrite is exercised

 @param contestChoice - lane scripted contest endorses

 @param quotaReads - counter of meter readings, for the case that proves the
 judge seats are read before each phase and not once per entry

 @param sheets - every sheet sent, under the schema that asked, for a case
 reading what a stage was shown

 @returns Client honoring the script

 @example
 ```ts
 const client = entryClient({ served, },);
 ```
 */
function entryClient(
  {
    served,
    failOnSchema,
    failWith = new Error('scripted provider failure',),
    activity,
    coverageScript = 'lost',
    polishScript = false,
    contestChoice = 'translate',
    quotaReads = { count: 0, },
    archivePictureSupport,
    archiveReviewSheets,
    sheets,
  }: {
    readonly served: string[];
    readonly failOnSchema?: string;
    readonly failWith?: Error;
    readonly activity?: PassConcurrency;
    readonly coverageScript?: CoverageScript;
    readonly polishScript?: boolean;
    readonly contestChoice?: 'repair' | 'translate';
    readonly quotaReads?: { count: number; };
    readonly archivePictureSupport?: string;
    readonly archiveReviewSheets?: string[];
    readonly sheets?: SentSheet[];
  },
): RunClient {
  return {
    providerMeterStates: metersAllWet,
    providerHolds: () => ({
      neuralwatt: 0,
      synthetic: 0,
      bedrock: 0,
      deepseek: 0,
      xiaomi: 0,
      hyper: 0,
      openrouter: 0,
    }),
    chatText: async () => {
      throw new Error('chatText unused by either lane',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Schema the caller asked for, which names the stage.
       */
      const schema = request.responseFormat
        ?.json_schema
        .name
        ?? 'unnamed';
      served.push(schema,);
      if (schema === failOnSchema)
        throw failWith;

      /**
       Everything the caller sent, which carries the slice original.
       */
      const content = request.messages
        .map(function toContent(message,) {
          return messageText({ message, },);
        },)
        .join('\n',);
      sheets?.push({
        schema,
        content,
      },);

      if ((schema === 'block_pairing') && (archivePictureSupport !== undefined)) {
        /** Pair only the shared component, leaving its archive translation for review. */
        const value = { pairs: [{ source: 0, target: 0, },], };
        if (!request.validate(value,))
          throw new Error('scripted picture pairing failed validator',);
        return { kind: 'ok', value, rawText: JSON.stringify(value,), };
      }
      if ((schema === 'archive_block_review') && (archivePictureSupport !== undefined)) {
        archiveReviewSheets?.push(content,);
        /** Source-supported reply depends on evidence actually reaching the review. */
        const value = {
          disposition: content.includes(archivePictureSupport,) ? 'source-supported' : 'revise',
          sourceQuote: content.includes(archivePictureSupport,) ? archivePictureSupport : '',
          replacementText: '',
          assessment: 'Read the picture translation against the supplied source evidence.',
        };
        if (!request.validate(value,))
          throw new Error('scripted picture support failed archive review validator',);
        return { kind: 'ok', value, rawText: JSON.stringify(value,), };
      }

      /**
       Whether shared translation schema belongs to consolidation producer.
       */
      const isConsolidation = (schema === 'translation_report')
        && content.includes(CONSOLIDATION_MARK,);

      /**
       Instrument matching this request's per-slice driver.
       */
      const stageActivity = schema === 'critic_report'
        ? activity?.repair
        : schema === 'refine_report'
          ? activity?.refine
          : schema === 'lane_contest'
            ? activity?.contest
            : isConsolidation
              ? activity?.consolidate
              : schema === 'translation_report'
                ? activity?.translate
                : undefined;
      if (stageActivity !== undefined) {
        stageActivity.now += 1;
        stageActivity.peak = Math.max(
          stageActivity.peak,
          stageActivity.now,
        );
        await wait(10,);
        stageActivity.now -= 1;
      }

      /**
       Reply for whichever stage asked, keyed by its schema.
       */
      const value: unknown = replyFor({
        schema,
        content,
        coverageScript,
        consolidation: isConsolidation,
        polishScript,
        contestChoice,
      },);
      if (!request.validate(value,))
        throw new Error(`scripted ${schema} failed the wire guard`,);
      return {
        kind: 'ok',
        value,
        rawText: JSON.stringify(value,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by either lane',);
    },
    providerDryness: async () => {
      // Counted, then refused: an unreadable view seats the full bench, so
      // the entry runs as before while the count says how often it was asked.
      quotaReads.count += 1;
      throw new Error('the budget view is unused by either lane',);
    },
  };
}

/**
 Throwaway artifacts, publish and cache directories for one case.

 @returns Every directory a settling entry writes into, plus how to remove them

 @example
 ```ts
 await using dirs = await throwawayDirs();
 ```
 */
async function throwawayDirs(): Promise<
  {
    readonly artifactsDir: string;
    readonly publishDir: string;
    readonly declinedDir: string;
    readonly sliceCacheDir: string;
  } & AsyncDisposable
  > {
  // Root nothing outside this case writes into.
  return await scratchDirWith({
    prefix: 'pass-entry-',
    setup: async function seeded({ path: root, },): Promise<{
      readonly artifactsDir: string;
      readonly publishDir: string;
      readonly declinedDir: string;
      readonly sliceCacheDir: string;
    }> {
      /**
       Directory settled entries write into, created here because the PASS
       creates it before settling anything: `settleEntry` writes into a
       directory it is handed, and a case that skipped this would be
       testing the atomic write's behavior on a missing parent instead.
       */
      const artifactsDir = join(
        root,
        'artifacts',
      );
      await mkdir(
        artifactsDir,
        { recursive: true, },
      );
      /**
       Root of the mirrored corpus tree, created here for the same reason
       the artifacts directory is: the PASS creates it before settling
       anything, so a case that left it out would be measuring how the
       publisher behaves against a missing root rather than how it
       publishes.
       */
      const publishDir = join(
        root,
        'fixed',
      );
      await mkdir(
        publishDir,
        { recursive: true, },
      );

      return {
        artifactsDir,
        publishDir,
        declinedDir: join(
          root,
          'declined',
        ),
        sliceCacheDir: join(
          root,
          'cache',
        ),
      };
    },
  },);
}

/**
 Reads every artifact filename a case wrote.

 @param artifactsDir - directory settled entries write into

 @returns Filenames present, or none when the directory was never created

 @example
 ```ts
 const written = await artifactNames({ artifactsDir, },);
 ```
 */
async function artifactNames(
  { artifactsDir, }: { readonly artifactsDir: string; },
): Promise<readonly string[]> {
  try {
    return await readdir(artifactsDir,);
  }
  catch (error) {
    // A settled entry creates this directory on its way to writing; a failed
    // one never reaches the write, so its absence is the answer rather than a
    // problem. Logged so a real permission failure is not read as emptiness.
    console.log(`no artifacts directory: ${caughtValueText(error,)}`,);
    return [];
  }
}

/**
 What settling the entry with one `info.yml` wrote, beside what settling it with none wrote.
 */
type SettledBesideNoInfoFile = {
  /**
   Artifact filenames the settling wrote.
   */
  readonly artifacts: readonly string[];

  /**
   The published page.
   */
  readonly page: string;

  /**
   The published page of the entry settled with no `info.yml`.
   */
  readonly referencePage: string;

  /**
   The preparation's findings the artifact recorded.
   */
  readonly findings: readonly string[];

  /**
   The findings the artifact of the entry settled with no `info.yml` recorded.
   */
  readonly referenceFindings: readonly string[];
};

/**
 Settles the entry once with an `info.yml` read through the real reader and once with none, so what the
 file changes shows against an entry whose record holds no death.

 @param infoFile - text of the entry's `info.yml` at the pin

 @returns What both settlings wrote

 @example
 ```ts
 const settled = await settledBesideNoInfoFile({ infoFile: 'info:\n  died: true\n', },);
 ```
 */
async function settledBesideNoInfoFile(
  { infoFile, }: { readonly infoFile: string; },
): Promise<SettledBesideNoInfoFile> {
  await using dirs = await throwawayDirs();
  await using reference = await throwawayDirs();
  await settleEntry({
    client: entryClient({ served: [], },),
    entry: ENTRY,
    artifactsDir: reference.artifactsDir,
    publishDir: reference.publishDir,
    declinedDir: reference.declinedDir,
    sliceCacheDir: reference.sliceCacheDir,
    tip: 'a'.repeat(40,),
    pipelineDigest: DIGEST,
    pin: ENTRY_PIN,
    outsideReads: NO_OUTSIDE_READS,
    pictureSources: NO_PICTURE_SOURCES,
    hardCapMs: HANG_STOP_MS,
    baseSignal: new AbortController().signal,
    overlap: PASS_FALLBACK_OVERLAP,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  await settleEntry({
    client: entryClient({ served: [], },),
    entry: ENTRY,
    artifactsDir: dirs.artifactsDir,
    publishDir: dirs.publishDir,
    declinedDir: dirs.declinedDir,
    sliceCacheDir: dirs.sliceCacheDir,
    tip: 'a'.repeat(40,),
    pipelineDigest: DIGEST,
    pin: ENTRY_PIN,
    outsideReads: {
      ...NO_OUTSIDE_READS,
      entryInfo: function fileEntryInfo({ entryId, l, },) {
        return readEntryInfoRecord({
          pin: ENTRY_PIN,
          entryId,
          readFile: async function fileAtPin() {
            return infoFile;
          },
          l,
        },);
      },
    },
    pictureSources: NO_PICTURE_SOURCES,
    hardCapMs: HANG_STOP_MS,
    baseSignal: new AbortController().signal,
    overlap: PASS_FALLBACK_OVERLAP,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  /**
   Artifact the entry wrote, through the production parser.
   */
  const artifact = parseSettledTwoLaneArtifact({
    value: JSON.parse(await readFile(
      join(dirs.artifactsDir, 'CatEntry1.json',),
      'utf8',
    ),),
  },);
  /**
   Artifact the entry wrote with no info.yml.
   */
  const referenceArtifact = parseSettledTwoLaneArtifact({
    value: JSON.parse(await readFile(
      join(reference.artifactsDir, 'CatEntry1.json',),
      'utf8',
    ),),
  },);
  return {
    artifacts: await artifactNames({ artifactsDir: dirs.artifactsDir, },),
    page: await readFile(
      fixedPagePath({
        publishDir: dirs.publishDir,
        entryId: ENTRY.id,
      },),
      'utf8',
    ),
    referencePage: await readFile(
      fixedPagePath({
        publishDir: reference.publishDir,
        entryId: ENTRY.id,
      },),
      'utf8',
    ),
    findings: artifact.preparation
      .alignmentFindings,
    referenceFindings: referenceArtifact.preparation
      .alignmentFindings,
  };
}

/**
 Console method a capture diverts: `log` for the pass's printed lines, `warn`
 for the warnings its entry loggers write.
 */
type CapturedMethod = 'log' | 'warn';

/**
 Diverts one console method into a list until disposed.

 @param lines - where diverted lines are appended

 @param method - console method diverted; a logger run arrives as one text
 whose records are joined by newlines

 @returns Capture holding those lines, which restores logging on disposal

 @example
 ```ts
 using capture = collectingInto({ lines, method: 'log', },);
 ```
 */
function collectingInto(
  {
    lines,
    method,
  }: {
    readonly lines: string[];
    readonly method: CapturedMethod;
  },
): { readonly lines: readonly string[]; } & Disposable {
  /**
   Real method, put back on disposal.
   */
  const printed = console[method];
  console[method] = (...parts: readonly unknown[]) => {
    lines.push(parts.map(String,)
      .join(' ',),);
  };
  return {
    lines,
    [Symbol.dispose]: () => {
      console[method] = printed;
    },
  };
}

/**
 Runs a body with every line one console method received collected instead
 of printed.

 The lines ARE the contract here: an entry that settled and then failed to
 retire its cache has to say so on a `CLEANUP` line, and the defect this
 guards against is a second `TALLY` after the success line, which made every
 reader counting statuses see one entry as both settled and errored.

 @param body - what to run while logging is captured

 @param method - console method captured: `log` for printed lines, `warn` for
 the entry loggers' warnings

 @returns Every line the body logged, in order

 @example
 ```ts
 const lines = await capturedLines({ body: async () => { await settleEntry(...); }, method: 'log', },);
 ```
 */
async function capturedLines(
  {
    body,
    method,
  }: {
    readonly body: () => Promise<void>;
    readonly method: CapturedMethod;
  },
): Promise<readonly string[]> {
  /**
   Lines the body logged.
   */
  const lines: string[] = [];

  using capture = collectingInto({
    lines,
    method,
  },);
  await body();

  // Read after the body so a case cannot assert on a capture still installed;
  // disposal at return puts the real logger back.
  return capture.lines;
}

await describe({
  name: settleEntry.name,
  // Environment dial is process-wide, so matched arms and every other case in
  // this file run one at a time rather than reading each other's settings.
  concurrency: 1,
  children: [
    it({
      name: 'THREADS THE OVERLAP IT IS HANDED through repair, translate, contest, and consolidation, while '
        + 'the process\'s overlap variable holds a value no dial can read, after sequential positive controls '
        + 'prove each stage instrument distinguishes one slice from two',
      fn: async () => {
        /**
         Activity with explicit sequential control arm.
         */
        const serial = emptyPassConcurrency();
        {
          using dial = dialSaying({ says: 'several', },);
          await using dirs = await throwawayDirs();
          await settleEntry({
            client: entryClient({
              served: [],
              activity: serial,
            },),
            entry: {
              ...ENTRY,
              id: 'CatOverlapSerial',
            },
            artifactsDir: dirs.artifactsDir,
            publishDir: dirs.publishDir,
            declinedDir: dirs.declinedDir,
            sliceCacheDir: dirs.sliceCacheDir,
            tip: 'a'.repeat(40,),
            pipelineDigest: DIGEST,
            pin: ENTRY_PIN,
            outsideReads: NO_OUTSIDE_READS,
            pictureSources: NO_PICTURE_SOURCES,
            hardCapMs: HANG_STOP_MS,
            baseSignal: new AbortController().signal,
            overlap: {
              overlap: 1,
              source: 'TRANSLATION_REPAIR_SLICE_OVERLAP',
            },
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        }

        /**
         Activity with two slices handed in.
         */
        const overlapped = emptyPassConcurrency();
        {
          using dial = dialSaying({ says: 'several', },);
          await using dirs = await throwawayDirs();
          await settleEntry({
            client: entryClient({
              served: [],
              activity: overlapped,
            },),
            entry: {
              ...ENTRY,
              id: 'CatOverlapTwo',
            },
            artifactsDir: dirs.artifactsDir,
            publishDir: dirs.publishDir,
            declinedDir: dirs.declinedDir,
            sliceCacheDir: dirs.sliceCacheDir,
            tip: 'a'.repeat(40,),
            pipelineDigest: DIGEST,
            pin: ENTRY_PIN,
            outsideReads: NO_OUTSIDE_READS,
            pictureSources: NO_PICTURE_SOURCES,
            hardCapMs: HANG_STOP_MS,
            baseSignal: new AbortController().signal,
            overlap: {
              overlap: 2,
              source: 'TRANSLATION_REPAIR_SLICE_OVERLAP',
            },
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        }

        expect(serial.repair.peak,).toBeGreaterThan(0,);
        expect(overlapped.repair.peak,).toBeGreaterThan(serial.repair.peak,);
        expect(serial.refine.peak,).toBeGreaterThan(0,);
        expect(overlapped.refine.peak,).toBeGreaterThan(serial.refine.peak,);
        expect(serial.translate.peak,).toBeGreaterThan(0,);
        expect(overlapped.translate.peak,).toBeGreaterThan(serial.translate.peak,);
        expect(serial.contest.peak,).toBeGreaterThan(0,);
        expect(overlapped.contest.peak,).toBeGreaterThan(serial.contest.peak,);
        expect(serial.consolidate.peak,).toBeGreaterThan(0,);
        expect(overlapped.consolidate.peak,).toBeGreaterThan(serial.consolidate.peak,);
      },
    },),

    it({
      name: 'STOPS BEFORE LANES when referenced visual asset has no reviewed evidence',
      fn: async () => {
        await using dirs = await throwawayDirs();
        /** Schemas reached before visual completeness guard. */
        const served: string[] = [];
        /** Entry outcome captured inside logger scope. */
        const result: { outcome?: Awaited<ReturnType<typeof settleEntry>>; } = {};
        const lines = await capturedLines({
          body: async () => {
            result.outcome = await settleEntry({
              client: entryClient({ served, },),
              entry: MISSING_VISUAL_ENTRY,
              artifactsDir: dirs.artifactsDir,
              publishDir: dirs.publishDir,
              declinedDir: dirs.declinedDir,
              sliceCacheDir: dirs.sliceCacheDir,
              tip: 'a'.repeat(40,),
              pipelineDigest: DIGEST,
              pin: ENTRY_PIN,
              outsideReads: NO_OUTSIDE_READS,
              // A GATHERER THAT FINDS NO BYTES, which is the case under test:
              // the entry names a picture and no file for it exists where it
              // would be read. Left to the corpus, this case read the pinned
              // checkout and ran the real programs (ledger M113).
              pictureSources: {
                ...NO_PICTURE_SOURCES,
                gather: async function gatherNothing(): Promise<Map<string, Uint8Array>> {
                  return new Map();
                },
              },
              hardCapMs: HANG_STOP_MS,
              baseSignal: new AbortController().signal,
              overlap: PASS_FALLBACK_OVERLAP,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
          },
          method: 'log',
        },);
        expect(result.outcome,).toEqual({ kind: 'stopped', },);
        expect(served,).not.toContain('coverage_report',);
        expect(served,).not.toContain('critic_report',);
        expect(served,).not.toContain('translation_report',);
        expect(served,).not.toContain('lane_contest',);
        expect(await artifactNames({ artifactsDir: dirs.artifactsDir, },),).toEqual([],);
        expect(await artifactNames({ artifactsDir: dirs.publishDir, },),).toEqual([],);
        expect(lines.some(function incomplete(line,): boolean {
          return line.startsWith(`TALLY ${MISSING_VISUAL_ENTRY.id} status=INCOMPLETE`,);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'SHARES picture evidence from archive review through lanes without asking the reader twice',
      fn: async () => {
        await using dirs = await throwawayDirs();
        /** Stage ordering across the actual entry orchestrator. */
        const served: string[] = [];
        /** Archive sheets read at the production call boundary. */
        const archiveReviewSheets: string[] = [];
        /** Picture-only source text, absent from both document bodies. */
        const archivePictureSupport = '手套猫：联系电话是 555-0134。';
        /** Reader invocations, not provider roster size. */
        const reads: string[] = [];
        await settleEntry({
          client: entryClient({ served, coverageScript: 'absent', archivePictureSupport, archiveReviewSheets, },),
          entry: {
            id: 'CatEntryPictureReview',
            sourceText: MISSING_VISUAL,
            targetText: `${MISSING_VISUAL}\n\n> Mittens: Call 555-0134.`,
          },
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          visualEvidenceReader: async function pictureEvidence() {
            reads.push('missing.webp',);
            served.push('picture-evidence',);
            return new Map([['missing.webp', {
              kind: 'corroborated',
              readings: [
                { modelId: SEAT_SYNTHETIC_VISION_WITHHELD, text: archivePictureSupport, },
                { modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, text: archivePictureSupport, },
              ],
              overlap: 1,
            },],]);
          },
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(archiveReviewSheets.length,).toBeGreaterThan(0,);
        expect(archiveReviewSheets.every(function supported(sheet,): boolean {
          return sheet.includes(archivePictureSupport,);
        },),).toBe(true,);
        expect(served.indexOf('picture-evidence',),).toBeLessThan(served.indexOf('archive_block_review',),);
        expect(reads,).toEqual(['missing.webp',],);
        expect(served,).toContain('critic_report',);
        expect(served,).toContain('translation_report',);
      },
    },),
    it({
      name: 'PASSES REVIEWED VISUAL into insertion and lane stages',
      fn: async () => {
        await using dirs = await throwawayDirs();
        /** Schemas reached after successful visual completeness guard. */
        const served: string[] = [];
        await settleEntry({
          client: entryClient({
            served,
            coverageScript: 'absent',
          },),
          entry: REVIEWED_VISUAL_ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          visualEvidenceReader: async function reviewedVisualEvidence() {
            return new Map([[
              'missing.webp',
              {
                kind: 'corroborated',
                readings: [
                  { modelId: SEAT_SYNTHETIC_VISION_WITHHELD, text: 'Mittens 555-0134', },
                  { modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, text: 'Mittens 555-0134', },
                ],
                overlap: 1,
              },
            ],]);
          },
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(served,).toContain('coverage_report',);
        expect(served,).toContain('critic_report',);
        expect(served,).toContain('translation_report',);
      },
    },),

    it({
      name:
        'settles an entry into ONE artifact at schema version 20 carrying BOTH lanes and absolute-reviewed '
          + 'final polish over one preparation',
      fn: async () => {
        await using dirs = await throwawayDirs();

        /**
         Schemas the run served, in order.
         */
        const served: string[] = [];
        await settleEntry({
          client: entryClient({ served, },),
          entry: ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(await artifactNames({ artifactsDir: dirs.artifactsDir, },),).toEqual(['CatEntry1.json',],);

        /**
         What reached disk.
         */
        const artifact: unknown = JSON.parse(await readFile(
          join(
            dirs.artifactsDir,
            'CatEntry1.json',
          ),
          'utf8',
        ),);

        // Read structurally rather than through the writer's own types, since
        // what is under test is the FILE: a reader holding only this has to
        // find both lanes nested and no lane at the top level.
        expect((artifact as { artifactSchemaVersion: number; }).artifactSchemaVersion,).toBe(20,);
        expect(Object.keys((artifact as { lanes: object; }).lanes,)
          .toSorted(),).toEqual([
            'repair',
            'translate',
          ],);
        /**
         Consolidation records carrying auditable polish decision.
         */
        const { consolidation, } = artifact as {
          consolidation: { slices: readonly { polish?: unknown; }[]; };
        };
        expect(consolidation.slices.every(function hasPolish(slice,): boolean {
          return slice.polish !== undefined;
        },),).toBe(true,);

        // THE STAMP AND THE SPELLING, checked together on the bytes. A writer
        // that stamped generation 7 and wrote an earlier generation's keys
        // would satisfy the `artifactSchemaVersion` assertion and produce a file its own reader
        // refuses, and every fixture in this package would still pass: they are
        // built by hand from the same names the writer uses.
        /**
         Repair lane's result as the file records it.
         */
        const repairResult = (artifact as {
          lanes: { repair: { result: Record<string, unknown>; }; };
        })
          .lanes
          .repair
          .result;

        for (const present of ['changedSliceIndices', 'withdrawnSliceIndices', 'sliceCritics',]) {
          expect(Object.hasOwn(
            repairResult,
            present,
          ),).toBe(true,);
        }
        for (const absent of ['shippedChunkIndices', 'withdrawnChunkIndices', 'chunkCritics',]) {
          expect(Object.hasOwn(
            repairResult,
            absent,
          ),).toBe(false,);
        }

        // THE INDEX IS THE HALF GENERATION 3 DID NOT CARRY, and it is spelled
        // on the ledger rows rather than beside those arrays. An artifact
        // stamped 4 whose rows still say `chunkIndex` is exactly generation 3,
        // and every assertion above it here would pass.
        /**
         Repair lane's first ledger row as the file records it.
         */
        const [firstRow,] = (artifact as {
          lanes: { repair: { delivery: readonly Record<string, unknown>[]; }; };
        })
          .lanes
          .repair
          .delivery;

        /**
         That row, present because this entry settled slices.
         */
        const repairRow = nonNullishOrThrow(firstRow,);

        expect(Object.hasOwn(
          repairRow,
          'sliceIndex',
        ),).toBe(true,);
        expect(Object.hasOwn(
          repairRow,
          'chunkIndex',
        ),).toBe(false,);
        // A CONTEST THAT RAN AND DECIDED, end to end through the pass. The two
        // lanes disagree at both slices by construction, and the scripted
        // roster backs each translate candidate.
        /**
         Contest as the file records it, read structurally like the rest.
         */
        const selection = (artifact as {
          laneSelection: {
            kind: string;
            slices: readonly {
              verdict: {
                readonly kind: string;
                readonly lane?: string;
              };
              ballots: readonly unknown[];
            }[];
          };
        }).laneSelection;
        expect(selection.kind,).toBe('contested',);
        expect(selection.slices
          .length,).toBe(2,);
        expect(selection.slices.every(function choseTranslate(slice,) {
          return (slice.verdict.kind === 'lane-won')
            && (slice.verdict.lane === 'translate');
        },),).toBe(true,);
        // AND THE BALLOTS REACHED THE FILE, which is the half a verdict alone
        // does not prove: a reader asking why this slice went to the translate
        // lane needs the reasons the judges gave, not just the count.
        expect(selection.slices
          .at(0,)
          ?.ballots
          .length,).toBeGreaterThan(1,);

        // One preparation, recorded once, which both ledgers are out of.
        expect((artifact as { preparation: { sliceCount: number; }; }).preparation
          .sliceCount,).toBeGreaterThan(0,);
        expect((artifact as { comparison: readonly unknown[]; }).comparison
          .length,).toBe((artifact as { preparation: { sliceCount: number; }; }).preparation
            .sliceCount,);

        // Both lanes were bought: the repair lane's critics and the translate
        // lane's translators each ran.
        expect(served.includes('critic_report',),).toBe(true,);
        expect(served.includes('translation_report',),).toBe(true,);
      },
    },),
    it({
      name: 'READS OUTSIDE THE PIPELINE ONLY THROUGH THE READERS IT IS HANDED, each asked about this entry\'s '
        + 'original, so a test settling an entry buys no web read with the key the suite inherits (ledger M68)',
      fn: async () => {
        await using dirs = await throwawayDirs();

        /**
         Originals each outside reader was asked about, by reader.
         */
        const asked: { readonly workTitles: string[]; readonly references: string[]; corpusNames: number; } = {
          workTitles: [],
          references: [],
          corpusNames: 0,
        };
        await settleEntry({
          client: entryClient({ served: [], },),
          entry: ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: {
            workTitles: async function recordWorkTitles({ sourceText, },) {
              asked.workTitles.push(sourceText,);
              return [];
            },
            references: async function recordReferences({ sourceText, },) {
              asked.references.push(sourceText,);
              return '';
            },
            corpusNames: async function recordCorpusNames() {
              asked.corpusNames += 1;
              return [];
            },
            entryInfo: async function noEntryInfo() {
              return { kind: 'absent', } as const;
            },
          },
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(asked.workTitles.length,).toBeGreaterThan(0,);
        expect(asked.references.length,).toBeGreaterThan(0,);
        expect(asked.corpusNames,).toBeGreaterThan(0,);
        expect(new Set([
          ...asked.workTitles,
          ...asked.references,
        ],),).toEqual(new Set([ENTRY.sourceText,],),);
      },
    },),
    it({
      name: 'SHIPS THE PAGE OF AN ENTRY WHOSE info.yml IS DAMAGED, read through the real info.yml reader, and records on '
        + 'the artifact that the record was unreadable and no death was told to the models (the owner\'s rule: a run always ships)',
      fn: async () => {
        /**
         What settling the entry with the damaged file wrote, beside the entry with none.
         */
        const settled = await settledBesideNoInfoFile({ infoFile: 'info: [\n', },);
        expect(settled.artifacts,).toEqual(['CatEntry1.json',],);
        expect(settled.page,).toBe(settled.referencePage,);
        expect(settled.findings,).toEqual([
          ...settled.referenceFindings,
          'entry info: The info.yml of entry CatEntry1 at the pin refused to parse at line 2 column 1 (BAD_INDENT); '
            + 'corpus metadata parses upstream, so this signals corruption. No death is recorded for the entry.',
        ],);
      },
    },),
    it({
      name: 'SHIPS THE PAGE OF AN ENTRY WHOSE info.yml HOLDS A died THAT IS A TRUTH VALUE, read through the real info.yml reader, '
        + 'and records on the artifact the key and the kind it held, so the artifact tells it from an entry that records no death',
      fn: async () => {
        /**
         What settling the entry with a truth value for its death wrote, beside the entry with no info.yml.
         */
        const settled = await settledBesideNoInfoFile({ infoFile: 'info:\n  died: true\n', },);
        expect(settled.artifacts,).toEqual(['CatEntry1.json',],);
        expect(settled.page,).toBe(settled.referencePage,);
        expect(settled.findings,).toEqual([
          ...settled.referenceFindings,
          'entry info: info.died is a truth value, which is neither text nor a finite number. '
            + 'No death is recorded for the entry.',
        ],);
      },
    },),
    it({
      name: 'CARRIES A CITED PAGE\'S WORDS to the contest and the consolidation, and the detail the bench '
        + 'attested from it to the translators, each on its own channel: the reference block where a sheet '
        + 'weighs wording against the original, the attested details where a translator renders it',
      fn: async () => {
        await using dirs = await throwawayDirs();
        /**
         Every sheet sent, under the schema that asked.
         */
        const sheets: SentSheet[] = [];
        await settleEntry({
          client: entryClient({
            served: [],
            sheets,
          },),
          entry: CITED_ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: {
            ...NO_OUTSIDE_READS,
            references: async function citedPage(): Promise<string> {
              return CITED_REFERENCE;
            },
          },
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        /**
         What the contest's sheets carried of the cited page.
         */
        const contest = citedEvidenceIn({
          sheets,
          kind: 'contest',
        },);
        /**
         What the consolidation producers' sheets carried of it.
         */
        const consolidation = citedEvidenceIn({
          sheets,
          kind: 'consolidation',
        },);
        /**
         What the translators' sheets carried of it.
         */
        const translator = citedEvidenceIn({
          sheets,
          kind: 'translator',
        },);
        expect({
          contestAsked: contest.sent > 0,
          everyContestNamesThePage: contest.namingThePage === contest.sent,
          consolidationAsked: consolidation.sent > 0,
          everyConsolidationNamesThePage: consolidation.namingThePage === consolidation.sent,
          translatorAsked: translator.sent > 0,
          everyTranslatorListsTheDetail: translator.listingAttested === translator.sent,
          translatorsNamingThePage: translator.namingThePage,
        },).toEqual({
          contestAsked: true,
          everyContestNamesThePage: true,
          consolidationAsked: true,
          everyConsolidationNamesThePage: true,
          translatorAsked: true,
          everyTranslatorListsTheDetail: true,
          translatorsNamingThePage: 0,
        },);
      },
    },),
    it({
      name: 'SHIPS A SECTION THE ARCHIVE\'S NOTE SEALS as the English original byte for byte, records the seal on '
        + 'the artifact, and reports no defect, while the section outside the seal takes the lanes\' wording',
      fn: async () => {
        await using dirs = await throwawayDirs();
        /**
         What settling the entry printed.
         */
        const lines = await capturedLines({
          body: async () => {
            await settleEntry({
              client: entryClient({ served: [], },),
              entry: SEALED_ENTRY,
              artifactsDir: dirs.artifactsDir,
              publishDir: dirs.publishDir,
              declinedDir: dirs.declinedDir,
              sliceCacheDir: dirs.sliceCacheDir,
              tip: 'a'.repeat(40,),
              pipelineDigest: DIGEST,
              pin: ENTRY_PIN,
              outsideReads: NO_OUTSIDE_READS,
              pictureSources: NO_PICTURE_SOURCES,
              hardCapMs: HANG_STOP_MS,
              baseSignal: new AbortController().signal,
              overlap: PASS_FALLBACK_OVERLAP,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
          },
          method: 'log',
        },);
        /**
         Serialized artifact through production parser.
         */
        const artifact = parseSettledTwoLaneArtifact({
          value: JSON.parse(await readFile(
            join(
              dirs.artifactsDir,
              `${SEALED_ENTRY.id}.json`,
            ),
            'utf8',
          ),),
        },);
        /**
         Spans the artifact records as sealed.
         */
        const spans = artifact.preparation
          .archiveOriginalSpans
          ?? [];
        /**
         Page the pass published.
         */
        const page = await readFile(
          fixedPagePath({
            publishDir: dirs.publishDir,
            entryId: SEALED_ENTRY.id,
          },),
          'utf8',
        );
        expect({
          notes: spans.map(function noteOf({ note, },): string {
            return note;
          },),
          sealsTheParagraph: spans.some(function covers({
            startOffset,
            endOffset,
          },): boolean {
            return SEALED_ENTRY.targetText
              .slice(
              startOffset,
              endOffset,
              )
              .includes(SEALED_PARAGRAPH,);
          },),
          pageKeepsTheSealedSection: page.includes(`<!-- ${SPAN_NOTE_TEXT} -->\n\n${SEALED_PARAGRAPH}\n`,),
          pageCarriesTheLanesWordingOfSectionTwo: page.includes(BIRD_FRESH,),
          pageCarriesTheLanesWordingOfSectionOne: page.includes(FRESH,),
          defectLines: lines.filter(function isDefectLine(line,): boolean {
            return line.startsWith(`DEFECTS ${SEALED_ENTRY.id} `,);
          },),
        },).toEqual({
          notes: [SPAN_NOTE_TEXT,],
          sealsTheParagraph: true,
          pageKeepsTheSealedSection: true,
          pageCarriesTheLanesWordingOfSectionTwo: false,
          pageCarriesTheLanesWordingOfSectionOne: true,
          defectLines: [],
        },);
      },
    },),
    it({
      name: 'SHIPS THE PAGE PASSES\' CORRECTION of a settled wording, records it on the artifact, and says so on '
        + 'the run log: each finding, and the count of slices rewritten',
      fn: async () => {
        await using dirs = await throwawayDirs();
        /**
         What the entry's loggers warned.
         */
        const warnings = await capturedLines({
          body: async () => {
            await settleEntry({
              client: entryClient({ served: [], },),
              entry: US_FORMS_ENTRY,
              artifactsDir: dirs.artifactsDir,
              publishDir: dirs.publishDir,
              declinedDir: dirs.declinedDir,
              sliceCacheDir: dirs.sliceCacheDir,
              tip: 'a'.repeat(40,),
              pipelineDigest: DIGEST,
              pin: ENTRY_PIN,
              outsideReads: NO_OUTSIDE_READS,
              pictureSources: NO_PICTURE_SOURCES,
              hardCapMs: HANG_STOP_MS,
              baseSignal: new AbortController().signal,
              overlap: PASS_FALLBACK_OVERLAP,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
          },
          method: 'warn',
        },);
        /**
         Serialized artifact through production parser.
         */
        const artifact = parseSettledTwoLaneArtifact({
          value: JSON.parse(await readFile(
            join(
              dirs.artifactsDir,
              `${US_FORMS_ENTRY.id}.json`,
            ),
            'utf8',
          ),),
        },);
        /**
         Page the pass published.
         */
        const page = await readFile(
          fixedPagePath({
            publishDir: dirs.publishDir,
            entryId: US_FORMS_ENTRY.id,
          },),
          'utf8',
        );
        /**
         Whether any warning run carries a text under this entry's tag.

         @param text - words the warning opens with

         @returns Whether one does

         @example
         ```ts
         const warned = warnedUnderEntry({ text: 'page assembly: ', },);
         ```
         */
        function warnedUnderEntry({ text, }: { readonly text: string; },): boolean {
          return warnings.some(function carries(run,): boolean {
            return run.includes(`[${US_FORMS_ENTRY.id}] ${text}`,);
          },);
        }
        expect({
          pageCarriesTheCorrection: page.includes(CANADIAN_FORMS_FRESH,),
          pageCarriesTheSettledSpelling: page.includes(US_FORMS_FRESH,),
          rewrittenRecorded: artifact.pageAssembly.trimmed.length > 0,
          findingRecorded: artifact.pageAssembly.findings.some(function namesTheForm(finding,): boolean {
            return finding.includes('canadian-form-rewritten',);
          },),
          findingLogged: warnedUnderEntry({ text: 'page assembly: ', },),
          countLogged: warnedUnderEntry({ text: 'page assembly rewrote ', },),
        },).toEqual({
          pageCarriesTheCorrection: true,
          pageCarriesTheSettledSpelling: false,
          rewrittenRecorded: true,
          findingRecorded: true,
          findingLogged: true,
          countLogged: true,
        },);
      },
    },),
    it({
      name: 'SHIPS THE PAGE UNCHANGED WHEN EVERY CONSISTENCY JUDGE THROWS, recording the repeated sentence as asked '
        + 'and unanswered: no winner, no changed slice, every place untouched',
      fn: async () => {
        await using dirs = await throwawayDirs();
        /**
         Schemas the entry's stages asked under.
         */
        const served: string[] = [];
        await settleEntry({
          client: entryClient({ served, failOnSchema: 'page_consistency', },),
          entry: REPEATED_SENTENCE_ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        /**
         Serialized artifact through production parser.
         */
        const artifact = parseSettledTwoLaneArtifact({
          value: JSON.parse(await readFile(
            join(
              dirs.artifactsDir,
              `${REPEATED_SENTENCE_ENTRY.id}.json`,
            ),
            'utf8',
          ),),
        },);
        /**
         Page the pass published.
         */
        const page = await readFile(
          fixedPagePath({
            publishDir: dirs.publishDir,
            entryId: REPEATED_SENTENCE_ENTRY.id,
          },),
          'utf8',
        );
        /**
         The one group the page repeats.
         */
        const group = nonNullishOrThrow(artifact.consistency.groups[0],);
        expect({
          asked: served.includes('page_consistency',),
          groups: artifact.consistency.groups.length,
          verdict: group.verdict.kind === 'asked' ? [group.verdict.usable, group.verdict.winner,] : group.verdict.kind,
          outcomes: group.occurrences.map(function outcomeOf(occurrence,): string {
            return occurrence.outcome;
          },),
          changed: artifact.consistency.slices.length,
          shipped: page.length > 0,
        },).toEqual({
          asked: true,
          groups: 1,
          verdict: [0, { kind: 'none', reason: 'no-quorum', },],
          outcomes: ['untouched', 'untouched',],
          changed: 0,
          shipped: true,
        },);
      },
    },),
    it({
      name: 'PUBLISHES CHANGED FINAL POLISH through artifact parser and page assembly',
      fn: async () => {
        await using dirs = await throwawayDirs();
        /**
         Schemas reached by full polished pass.
         */
        const served: string[] = [];
        await settleEntry({
          client: entryClient({
            served,
            polishScript: true,
          },),
          entry: ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        /**
         Serialized artifact through production parser.
         */
        const artifact = parseSettledTwoLaneArtifact({
          value: JSON.parse(await readFile(
            join(dirs.artifactsDir, 'CatEntry1.json',),
            'utf8',
          ),),
        },);
        if (artifact.consolidation.kind !== 'settled')
          throw new Error('polished pass did not record consolidation',);
        /**
         Changed polish record for first slice.
         */
        const polish = artifact.consolidation.slices
          .find(function firstSlice(slice,): boolean {
            return slice.sliceIndex === 0;
          },)
          ?.polish;
        expect(polish?.kind,).toBe('settled',);
        expect(polish?.kind === 'settled' ? polish.changed : false,).toBe(true,);
        expect(polish?.kind === 'settled' ? polish.text : '',).toContain(POLISH_FINAL_WRAPPED,);
        expect(polish?.kind === 'settled' ? polish.review?.rounds.at(-1,)?.verdict : '',).toBe('acceptable',);
        /**
         Page persisted from parsed decision stack.
         */
        const page = await readFile(fixedPagePath({
          publishDir: dirs.publishDir,
          entryId: ENTRY.id,
        },), 'utf8',);
        expect(page,).toContain(POLISH_FINAL_WRAPPED,);
        expect(page,).not.toContain(POLISH_BASE_PARAGRAPH,);
        expect(served,).toContain('consolidation_polish_gate',);
        expect(served,).toContain('absolute_naturalness_review',);
      },
    },),
    it({
      name: 'DECLINES an entry whose archive note calls the page the author\'s own English BEFORE ANY PURCHASE: a '
        + 'decline record, no model call, no artifact',
      fn: async () => {
        await using dirs = await throwawayDirs();
        /**
         Model schemas asked, none expected.
         */
        const served: string[] = [];
        /**
         Entry whose archive carries a whole-page note.
         */
        const ownEnglish = {
          id: 'CatEntryOwnEnglish',
          sourceText: SOURCE_TEXT,
          targetText: `<!-- 这只猫的原文即英文 -->\n\n${TARGET_TEXT}`,
        };
        /**
         What settling the entry answered.
         */
        const outcome = await settleEntry({
          client: entryClient({ served, },),
          entry: ownEnglish,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect({
          outcome,
          served,
          artifacts: await artifactNames({ artifactsDir: dirs.artifactsDir, },),
          declined: await readdir(dirs.declinedDir,),
        },).toEqual({
          outcome: { kind: 'declined', },
          served: [],
          artifacts: [],
          declined: [`${ownEnglish.id}.json`,],
        },);
      },
    },),
    it({
      name: 'NAMES THE COMMIT OF THE PIN IT IS HANDED on the artifact it writes and on a decline record, never one '
        + 'of its own',
      fn: async () => {
        await using dirs = await throwawayDirs();
        /**
         Entry whose archive carries a whole-page note, which is declined.
         */
        const ownEnglish = {
          id: 'CatEntryOwnEnglish',
          sourceText: SOURCE_TEXT,
          targetText: `<!-- 这只猫的原文即英文 -->\n\n${TARGET_TEXT}`,
        };
        /**
         Settles one entry into this case's directories at the invented pin.

         @param entry - corpus pair to settle

         @returns What settling it answered

         @example
         ```ts
         await settledAtPin({ entry: ENTRY, },);
         ```
         */
        async function settledAtPin(
          { entry, }: { readonly entry: typeof ownEnglish; },
        ): Promise<Awaited<ReturnType<typeof settleEntry>>> {
          return await settleEntry({
            client: entryClient({ served: [], },),
            entry,
            artifactsDir: dirs.artifactsDir,
            publishDir: dirs.publishDir,
            declinedDir: dirs.declinedDir,
            sliceCacheDir: dirs.sliceCacheDir,
            tip: 'a'.repeat(40,),
            pipelineDigest: DIGEST,
            pin: ENTRY_PIN,
            outsideReads: NO_OUTSIDE_READS,
            pictureSources: NO_PICTURE_SOURCES,
            hardCapMs: HANG_STOP_MS,
            baseSignal: new AbortController().signal,
            overlap: PASS_FALLBACK_OVERLAP,
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        }
        expect(await settledAtPin({ entry: ENTRY, },),).toEqual({ kind: 'settled', },);
        expect(await settledAtPin({ entry: ownEnglish, },),).toEqual({ kind: 'declined', },);

        /**
         The artifact the settled entry wrote, as text.
         */
        const artifactText = await readFile(
          join(
            dirs.artifactsDir,
            `${ENTRY.id}.json`,
          ),
          'utf8',
        );

        /**
         The decline record the declined entry wrote, as text.
         */
        const declineText = await readFile(
          join(
            dirs.declinedDir,
            `${ownEnglish.id}.json`,
          ),
          'utf8',
        );
        expect(JSON.parse(artifactText,),).toHaveProperty(
          'corpusSha',
          ENTRY_PIN.commitSha,
        );
        expect(JSON.parse(declineText,),).toHaveProperty(
          'corpusSha',
          ENTRY_PIN.commitSha,
        );
      },
    },),
    it({
      name: 'PUBLISHES preparation-stage archive correction through lanes, artifact, and page',
      fn: async () => {
        await using dirs = await throwawayDirs();
        /** Model schemas reached across complete revised entry. */
        const served: string[] = [];
        const outcome = await settleEntry({
          client: entryClient({ served, }),
          entry: ARCHIVE_REVISION_ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        /** Serialized artifact text for absence proof. */
        const artifactText = await readFile(
          join(dirs.artifactsDir, `${ARCHIVE_REVISION_ENTRY.id}.json`,),
          'utf8',
        );
        /** Published page produced from same corrected preparation. */
        const page = await readFile(fixedPagePath({
          publishDir: dirs.publishDir,
          entryId: ARCHIVE_REVISION_ENTRY.id,
        },), 'utf8',);

        /**
         What the written artifact says preparation found (ledger B54).
         */
        const preparationFindings = parseSettledTwoLaneArtifact({ value: JSON.parse(artifactText,), },)
          .preparation
          .alignmentFindings;

        expect(outcome,).toEqual({ kind: 'settled', });
        expect(served,).toContain('archive_block_review');
        expect(artifactText.includes(UNSUPPORTED_ARCHIVE_BLOCK,),).toBe(false,);
        expect(page.includes(UNSUPPORTED_ARCHIVE_BLOCK,),).toBe(false,);
        expect(page,).toContain(FRESH);
        expect(page,).toContain(BIRD_FRESH);
        expect(preparationFindings.some(function namesRevision(finding,): boolean {
          return finding.startsWith('archive block reviewed and revised: ',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'RECOVERS repair-lane source destination loss through consolidation before page write, and '
        + 'READS the judge seats before each of the six stages rather than once, the preparation, translate-lane and '
        + 'picture readings included (XIEPT2, 2026-09-03: Synthetic ran dry seven minutes into a '
        + '219-minute entry; the OpenRouter-only passes bought a withheld model\'s calls from stages '
        + 'that read no seats)',
      fn: async () => {
        await using dirs = await throwawayDirs();
        const served: string[] = [];
        /**
         How often the meter was read across the entry.
         */
        const quotaReads = { count: 0, };
        const outcome = await settleEntry({
          client: entryClient({
            served,
            contestChoice: 'repair',
            quotaReads,
          },),
          entry: LINK_RECOVERY_ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        const page = await readFile(fixedPagePath({
          publishDir: dirs.publishDir,
          entryId: LINK_RECOVERY_ENTRY.id,
        },), 'utf8',);

        expect(outcome,).toEqual({ kind: 'settled', });
        expect(served,).toContain('lane_contest');
        expect(served,).toContain('consolidate_gate');
        expect(page,).toContain('https://example.test/cat-record');
        // Preparation, pictures, lanes, the translate lane (re-seated when it
        // starts, the thirteenth class), lane contest, consolidation: one
        // reading each.
        expect(quotaReads.count,).toBe(6,);
      },
    },),
    it({
      name: 'BOUNDS THE SEAT READER INSIDE AN ENTRY BY THE ENTRY\'S OWN HARD CAP: a pass with hours left hands the '
        + 'entry a bound no larger than the cap, so a wait for a return never outlasts the entry\'s timer',
      fn: async () => {
        await using dirs = await throwawayDirs();
        /**
         The bound each seat reading asked the holds for.
         */
        const asked: number[] = [];
        const base = entryClient({ served: [], },);
        await settleEntry({
          client: {
            ...base,
            // A pass with a year left, far past the entry's cap.
            passLeftMs: () => Number.MAX_SAFE_INTEGER,
            providerHolds: function recordBound(bound,) {
              asked.push(bound?.returnBoundMs ?? Number.POSITIVE_INFINITY,);
              return base.providerHolds();
            },
          },
          entry: LINK_RECOVERY_ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(asked.length,).toBeGreaterThan(0,);
        expect(asked.every(function isWithinCap(bound,): boolean {
          return bound <= HANG_STOP_MS;
        },),).toBe(true,);
      },
    },),
    it({
      name: 'PUBLISHES SOURCE-ALIGNED FRONT MATTER THROUGH LANES, CONTEST, ARTIFACT READBACK, AND PAGE WRITE',
      fn: async () => {
        await using dirs = await throwawayDirs();
        /**
         Model schemas reached across complete entry boundary.
         */
        const served: string[] = [];
        await settleEntry({
          client: entryClient({ served, },),
          entry: FRONT_MATTER_ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        /**
         Serialized artifact read through production parser.
         */
        const artifact = parseSettledTwoLaneArtifact({
          value: JSON.parse(await readFile(
            join(dirs.artifactsDir, 'CatFrontMatter.json',),
            'utf8',
          ),),
        },);
        /**
         Page persisted before artifact sentinel.
         */
        const page = await readFile(fixedPagePath({
          publishDir: dirs.publishDir,
          entryId: FRONT_MATTER_ENTRY.id,
        },), 'utf8',);

        expect(artifact.artifactSchemaVersion,).toBe(20,);
        expect(artifact.preparation.sliceCount,).toBe(1,);
        if (artifact.laneSelection.kind !== 'contested')
          throw new Error('front matter pass did not record lane contest',);
        expect(artifact.laneSelection.slices[0]?.eligibility,).toEqual({
          syntax: 'front-matter',
          sourceText: FRONT_MATTER_SOURCE,
          archive: 'ineligible',
          repair: 'ineligible',
          translate: 'eligible',
        },);
        expect(page,).toBe(FRONT_MATTER_FRESH,);
        expect(served,).toContain('translation_report',);
        expect(served,).toContain('candidate_ballot',);
        expect(served,).toContain('lane_contest',);
      },
    },),
    it({
      name: 'KEEPS A TRANSLATED ARCHIVE\'S FRONT MATTER AS IT STANDS through the complete pass, the owner\'s '
        + 'rule of 2026-09-08: no metadata slice reaches the lanes or the contest, the artifact records the '
        + 'archive\'s authority, and the page carries the archive\'s bytes',
      fn: async () => {
        await using dirs = await throwawayDirs();
        const served: string[] = [];
        const outcome = await settleEntry({
          client: entryClient({ served, },),
          entry: FRONT_MATTER_STANDING_ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome,).toEqual({ kind: 'settled', },);
        /**
         Serialized artifact read through production parser.
         */
        const artifact = parseSettledTwoLaneArtifact({
          value: JSON.parse(await readFile(
            join(dirs.artifactsDir, `${FRONT_MATTER_STANDING_ENTRY.id}.json`,),
            'utf8',
          ),),
        },);
        const page = await readFile(fixedPagePath({
          publishDir: dirs.publishDir,
          entryId: FRONT_MATTER_STANDING_ENTRY.id,
        },), 'utf8',);
        expect(page.startsWith(FRONT_MATTER_FRESH,)).toBe(true,);
        expect(artifact.preparation.frontMatterAuthority,).toBe('archive',);
        if (artifact.laneSelection.kind === 'contested') {
          expect(artifact.laneSelection.slices.some(function isMetadata(slice,): boolean {
            return slice.eligibility?.syntax === 'front-matter';
          },),).toBe(false,);
        }
        expect(served,).toContain('translation_report',);
      },
    },),
    it({
      name: 'INSERTS SOURCE-ONLY FRONT MATTER through complete pass',
      fn: async () => {
        await using dirs = await throwawayDirs();
        const served: string[] = [];
        const outcome = await settleEntry({
          client: entryClient({
            served,
            coverageScript: 'absent',
          },),
          entry: FRONT_MATTER_SOURCE_ONLY_ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome,).toEqual({ kind: 'settled', },);
        const page = await readFile(fixedPagePath({
          publishDir: dirs.publishDir,
          entryId: FRONT_MATTER_SOURCE_ONLY_ENTRY.id,
        },), 'utf8',);
        expect(page.startsWith(FRONT_MATTER_FRESH,)).toBe(true,);
        expect(served,).not.toContain('coverage_report',);
        expect(served,).toContain('translation_report',);
      },
    },),
    it({
      name: 'FOLDS inherited invisible archive variants before preparation, so deciders, artifact and '
        + 'published page all carry same visible bytes',
      fn: async () => {
        await using dirs = await throwawayDirs();
        await settleEntry({
          client: entryClient({ served: [], },),
          entry: INVISIBLE_ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        const artifact = await readFile(
          join(dirs.artifactsDir, `${INVISIBLE_ENTRY.id}.json`,),
          'utf8',
        );
        const page = await readFile(
          fixedPagePath({
            publishDir: dirs.publishDir,
            entryId: INVISIBLE_ENTRY.id,
          },),
          'utf8',
        );
        expect(artifact,).not.toContain('\uFEFF',);
        expect(page,).not.toContain('\uFEFF',);
      },
    },),
    it({
      name:
        'REFUSES to write anything for an entry that raised, and keeps every slice it had already '
          + 'bought: the cache is what makes the next attempt cheaper, and a pass that discarded it on '
          + 'failure would re-buy the whole document',
      fn: async () => {
        await using dirs = await throwawayDirs();

        /**
         Schemas the run served before the write failed.
         */
        const served: string[] = [];

        // THE WRITE is what fails here, not a model call, and that is
        // deliberate: both lanes tolerate a stage losing every voice, so a
        // scripted provider failure settles the entry rather than failing it.
        // A directory that does not exist fails the one step every
        // settled entry has to reach, whatever the lanes did.
        await settleEntry({
          client: entryClient({ served, },),
          entry: ENTRY,
          artifactsDir: join(
            dirs.artifactsDir,
            'not-created',
          ),
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        // Nothing reached the artifacts directory, which is the rule that makes
        // a settled artifact mean the entry finished.
        expect(await artifactNames({ artifactsDir: dirs.artifactsDir, },),).toEqual([],);

        // And this entry's cache directory survives the failure, holding the
        // slices both lanes had already bought.
        expect(await artifactNames({ artifactsDir: dirs.sliceCacheDir, },),).toEqual(['CatEntry1',],);
      },
    },),
    it({
      name:
        'PUBLISHES THE PAGE BEFORE THE ARTIFACT, so an entry the pass will never look at again always '
          + 'has one. A pass builds its skip set from the artifacts already on disk, which makes the '
          + 'artifact the sentinel for "done": written first, a crash before publishing would strand '
          + 'that entry done forever with no page. The write that fails here is the artifact write, and '
          + 'the page is on disk regardless',
      fn: async () => {
        await using dirs = await throwawayDirs();

        /**
         Schemas the run served before the write failed.
         */
        const served: string[] = [];

        // The SAME failure the "REFUSES to write anything for an entry that
        // raised" case uses, for the opposite assertion: a
        // directory that does not exist fails the artifact write and nothing
        // before it.
        await settleEntry({
          client: entryClient({ served, },),
          entry: ENTRY,
          artifactsDir: join(
            dirs.artifactsDir,
            'not-created',
          ),
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        // Nothing says this entry settled, so a later pass attempts it again.
        expect(await artifactNames({ artifactsDir: dirs.artifactsDir, },),).toEqual([],);

        /**
         This entry's page, which has to exist even though its artifact does not.
         */
        const published = await readFile(
          fixedPagePath({
            publishDir: dirs.publishDir,
            entryId: ENTRY.id,
          },),
          'utf8',
        );

        // A whole page rather than a stub: both of the archive's sections are
        // in it, so what landed is the document and not one settled slice.
        expect(published.includes('## Section one',),).toBe(true,);
        expect(published.includes('## Section two',),).toBe(true,);
      },
    },),
    it({
      name: 'PAUSES before lanes when coverage cannot resolve known source-gap placement',
      fn: async () => {
        await using dirs = await throwawayDirs();
        /**
         Schemas reached before completeness guard stops downstream stages.
         */
        const served: string[] = [];
        const lines = await capturedLines({
          body: async () => {
            await settleEntry({
              client: entryClient({ served, },),
              entry: GAP_ENTRY,
              artifactsDir: dirs.artifactsDir,
              publishDir: dirs.publishDir,
              declinedDir: dirs.declinedDir,
              sliceCacheDir: dirs.sliceCacheDir,
              tip: 'a'.repeat(40,),
              pipelineDigest: DIGEST,
              pin: ENTRY_PIN,
              outsideReads: NO_OUTSIDE_READS,
              pictureSources: NO_PICTURE_SOURCES,
              hardCapMs: HANG_STOP_MS,
              baseSignal: new AbortController().signal,
              overlap: PASS_FALLBACK_OVERLAP,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
          },
          method: 'log',
        },);

        expect(served,).toContain('coverage_report',);
        expect(served,).not.toContain('lane_contest',);
        expect(await artifactNames({ artifactsDir: dirs.artifactsDir, },),).toEqual([],);
        expect(await artifactNames({ artifactsDir: dirs.publishDir, },),).toEqual([],);
        expect(await artifactNames({ artifactsDir: dirs.sliceCacheDir, },),).toEqual([GAP_ENTRY.id,],);
        const tally = lines.filter(function forGapEntry(line,): boolean {
          return line.startsWith(`TALLY ${GAP_ENTRY.id} `,);
        },);
        expect(tally,).toHaveLength(1,);
        expect(tally[0],).toContain('status=INCOMPLETE',);
      },
    },),
    it({
      name: 'PUBLISHES source-only passage proven carried elsewhere when final page retains proof region',
      fn: async () => {
        await using dirs = await throwawayDirs();
        const served: string[] = [];
        const outcome = await settleEntry({
          client: entryClient({
            served,
            coverageScript: 'full',
          },),
          entry: CARRIED_ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(outcome,).toEqual({ kind: 'settled', },);
        expect(served,).toContain('coverage_report',);
        const page = await readFile(
          fixedPagePath({
            publishDir: dirs.publishDir,
            entryId: CARRIED_ENTRY.id,
          },),
          'utf8',
        );
        expect(page,).toContain(GAP_FRESH,);
      },
    },),
    it({
      name: 'PAUSES when coverage admits source gap but every translator voice is lost',
      fn: async () => {
        await using dirs = await throwawayDirs();
        const served: string[] = [];
        const outcome = await settleEntry({
          client: entryClient({
            served,
            failOnSchema: 'translation_report',
            coverageScript: 'absent',
          },),
          entry: GAP_ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(outcome,).toEqual({ kind: 'stopped', },);
        expect(served,).toContain('coverage_report',);
        expect(served,).not.toContain('lane_contest',);
        expect(await artifactNames({ artifactsDir: dirs.artifactsDir, },),).toEqual([],);
        expect(await artifactNames({ artifactsDir: dirs.publishDir, },),).toEqual([],);
      },
    },),
    it({
      name: 'ADMITS and publishes a linked factual source gap when production coverage says it is absent, '
        + 'proving admission reaches translate lane rather than stopping at pass preparation',
      fn: async () => {
        await using dirs = await throwawayDirs();
        const served: string[] = [];
        await settleEntry({
          client: entryClient({
            served,
            coverageScript: 'absent',
          },),
          entry: {
            ...GAP_ENTRY,
            id: 'CatEntryGapRecovered',
          },
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(served,).toContain('coverage_report',);
        expect(await artifactNames({ artifactsDir: dirs.artifactsDir, },),)
          .toEqual(['CatEntryGapRecovered.json',],);
        const artifact = await readFile(
          join(dirs.artifactsDir, 'CatEntryGapRecovered.json',),
          'utf8',
        );
        const page = await readFile(
          fixedPagePath({
            publishDir: dirs.publishDir,
            entryId: 'CatEntryGapRecovered',
          },),
          'utf8',
        );
        expect(artifact,).toContain('insertion-corroboration',);
        expect(page,).toContain(GAP_FRESH,);
      },
    },),
    it({
      name:
        'RETURNS rather than raising for an entry that failed, because a pass over a corpus stops for '
          + 'a broken scheduler and not for a broken document: anything thrown here would end the run at '
          + 'whichever entry happened to hold the problem',
      fn: async () => {
        await using dirs = await throwawayDirs();

        /**
         Schemas served by a run whose write cannot land.
         */
        const served: string[] = [];

        // No expect(...).rejects here on purpose: the claim is that this
        // RETURNS, and a case that asserted on a thrown value would pass
        // whether or not it did.
        await settleEntry({
          client: entryClient({ served, },),
          entry: ENTRY,
          artifactsDir: join(
            dirs.artifactsDir,
            'not-created',
          ),
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(served.length,).toBeGreaterThan(0,);
      },
    },),
    it({
      name:
        'SETTLES an entry whose critic stage lost every voice, and says so in the OUTCOMES rather than '
          + 'only in the findings: a slice nobody was heard about falls back to the archive, so a reader '
          + 'joining on outcomes sees the silence instead of a lane that looks like it decided',
      fn: async () => {
        await using dirs = await throwawayDirs();

        /**
         Schemas served, with every critic call failing.
         */
        const served: string[] = [];
        await settleEntry({
          client: entryClient({
            served,
            failOnSchema: 'critic_report',
          },),
          entry: ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(await artifactNames({ artifactsDir: dirs.artifactsDir, },),).toEqual(['CatEntry1.json',],);

        /**
         What reached disk after every critic call failed.
         */
        const artifact: unknown = JSON.parse(await readFile(
          join(
            dirs.artifactsDir,
            'CatEntry1.json',
          ),
          'utf8',
        ),);

        // Every critic was attempted and every one was lost.
        expect(served.filter(function isCritic(schema,): boolean {
          return schema === 'critic_report';
        },).length,).toBeGreaterThan(0,);
        expect((artifact as { lanes: { repair: { result: { findings: readonly string[]; }; }; }; }).lanes
          .repair
          .result
          .findings
          .some(function unmet(finding,): boolean {
            return finding.includes('stage-quorum-unmet',);
          },),).toBe(true,);

        // AND THE LEDGER SAYS IT TOO, which is the half that was missing: every
        // repair slice reads as falling back to the archive rather than as a
        // decision, so nothing downstream can read this run as the lane having
        // examined the wording and chosen to keep it.
        expect((artifact as {
          lanes: { repair: { delivery: readonly { outcome: { kind: string; }; }[]; }; };
        }).lanes
          .repair
          .delivery
          .map(function toKind(row,): string {
            return row.outcome
              .kind;
          },)
          .every(function fellBack(kind,): boolean {
            return kind === 'incumbent-fallback';
          },),).toBe(true,);
      },
    },),
    it({
      name:
        'REFUSES to write an artifact when the ceiling fired while BOTH LANES FINISHED FROM CACHE, '
          + 'which is the one window the check after the driver exists for: a resumed entry buys nothing, '
          + 'so no exchange is left to notice the abort and the run ends holding two complete documents '
          + 'it is no longer entitled to record',
      fn: async () => {
        await using dirs = await throwawayDirs();

        /**
         First run, whose write cannot land, so both lanes complete and the
         cache they filled survives instead of being discarded on settlement.
         */
        const bought: string[] = [];
        await settleEntry({
          client: entryClient({ served: bought, },),
          entry: ENTRY,
          artifactsDir: join(
            dirs.artifactsDir,
            'not-created',
          ),
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(bought.length,).toBeGreaterThan(0,);

        /**
         Ceiling that fired before this attempt started, which is what a
         resumed entry meets when the pass is already over its budget.
         */
        const controller = new AbortController();
        controller.abort(new Error('entry deadline reached',),);

        /**
         Schemas the resumed run serves, which must be none.
         */
        const resumed: string[] = [];
        await settleEntry({
          client: entryClient({ served: resumed, },),
          entry: ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: controller.signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        // NOTHING WAS BOUGHT, which is what makes this the interesting case:
        // with every slice resumed, no exchange ever saw the aborted signal, so
        // the lanes returned two finished documents under an expired deadline.
        expect(resumed,).toEqual([],);

        // And no artifact was written for them.
        expect(await artifactNames({ artifactsDir: dirs.artifactsDir, },),).toEqual([],);

        // The cache is kept, so the entry can be settled properly later.
        expect(await artifactNames({ artifactsDir: dirs.sliceCacheDir, },),).toEqual(['CatEntry1',],);
      },
    },),
    it({
      name:
        'logs a CLEANUP line and NOT a second TALLY when the artifact landed but the cache could not be '
          + 'retired: the entry IS settled, so a second status line would make every reader counting '
          + 'statuses see one entry as both settled and errored',
      fn: async () => {
        await using dirs = await throwawayDirs();

        /**
         This entry's own cache directory, which the first run fills.
         */
        const entryCacheDir = join(
          dirs.sliceCacheDir,
          CLEANUP_ENTRY.id,
        );

        // FIRST RUN: the write cannot land, so the cache survives rather than
        // being retired on settlement. There is no hook between the artifact
        // write and the discard, so the cache has to exist before the run that
        // fails to remove it.
        await settleEntry({
          client: entryClient({ served: [], },),
          entry: CLEANUP_ENTRY,
          artifactsDir: join(
            dirs.artifactsDir,
            'not-created',
          ),
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(await artifactNames({ artifactsDir: dirs.sliceCacheDir, },),).toEqual([CLEANUP_ENTRY.id,],);

        // Removal unlinks the entries INSIDE this directory, which needs write
        // permission on the directory itself. Read and traverse are left, so
        // the run can still resume from it.
        //
        // Permissions do not constrain a superuser, so this case reports a
        // discard that unexpectedly succeeded rather than passing quietly: run
        // as root, the injection has no effect and there is nothing to test.
        await chmod(
          entryCacheDir,
          0o500,
        );

        /**
         Everything the settling run printed.
         */
        const lines = await capturedLines({
          body: async () => {
            await settleEntry({
              client: entryClient({ served: [], },),
              entry: CLEANUP_ENTRY,
              artifactsDir: dirs.artifactsDir,
              publishDir: dirs.publishDir,
              declinedDir: dirs.declinedDir,
              sliceCacheDir: dirs.sliceCacheDir,
              tip: 'a'.repeat(40,),
              pipelineDigest: DIGEST,
              pin: ENTRY_PIN,
              outsideReads: NO_OUTSIDE_READS,
              pictureSources: NO_PICTURE_SOURCES,
              hardCapMs: HANG_STOP_MS,
              baseSignal: new AbortController().signal,
              overlap: PASS_FALLBACK_OVERLAP,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
          },
          method: 'log',
        },);

        // Put the directory back before asserting, so a failing assertion still
        // leaves a removable tree behind for the disposal.
        await chmod(
          entryCacheDir,
          0o700,
        );

        /**
         Lines about THIS entry, since other cases log into the same capture.
         */
        const mine = lines.filter(function namesThisEntry(line,): boolean {
          return line.includes(CLEANUP_ENTRY.id,);
        },);

        /**
         Status lines this entry produced.
         */
        const tallies = mine.filter(function isTally(line,): boolean {
          return line.startsWith('TALLY ',);
        },);

        /**
         Cleanup lines it produced.
         */
        const cleanups = mine.filter(function isCleanup(line,): boolean {
          return line.startsWith('CLEANUP ',);
        },);

        /**
         Slice-overlap launch lines it produced.
         */
        const overlaps = mine.filter(function isOverlap(line,): boolean {
          return line.startsWith('OVERLAP ',);
        },);
        expect(overlaps,).toEqual([
          `OVERLAP ${CLEANUP_ENTRY.id} value=4 source=fallback`,
        ],);
        expect(cleanups.length,).toBe(1,);
        expect(cleanups[0]?.includes('cache=retained',),).toBe(true,);
        // The unlink failure is an unmarked Node error, so the line names its
        // class and never repeats its message, which carries a path.
        expect(cleanups[0]?.includes('error=refused by ',),).toBe(true,);
        expect(cleanups[0]?.includes('permission denied',),).toBe(false,);

        // EXACTLY ONE status line, and it says the entry settled.
        expect(tallies.length,).toBe(1,);
        expect(tallies[0]?.includes('status=SETTLED',),).toBe(true,);

        // The artifact is on disk and the cache it could not retire is still
        // there, which costs disk and nothing else: the next run skips this
        // entry on its artifact.
        expect(await artifactNames({ artifactsDir: dirs.artifactsDir, },),).toEqual([`${CLEANUP_ENTRY.id}.json`,],);
        expect(await artifactNames({ artifactsDir: dirs.sliceCacheDir, },),).toEqual([CLEANUP_ENTRY.id,],);
      },
    },),

    it({
      name: 'RECORDS THE ENTRY\'S OWN DURATION when the system clock is set back an hour while it settles (ledger '
        + 'B78), where the artifact carried an hour less than nothing, which the two-lane reader refuses as no count',
      fn: async ctx => {
        await using dirs = await throwawayDirs();
        const wall = stubWallClock({ sinon: ctx.sinon, atMs: WALL_START_MS, },);

        /**
         Schemas the run served, in order.
         */
        const served: string[] = [];

        /**
         The scripted client, whose first answer comes after the clock is set back.
         */
        const scripted = entryClient({ served, },);

        /**
         Whether the clock has been set back yet.
         */
        const stepped = { done: false, };
        await settleEntry({
          client: {
            ...scripted,
            chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
              if (!stepped.done) {
                stepped.done = true;
                wall.step({ byMs: -HOUR_MS, },);
              }
              return await scripted.chatJson(request,);
            },
          },
          entry: ENTRY,
          artifactsDir: dirs.artifactsDir,
          publishDir: dirs.publishDir,
          declinedDir: dirs.declinedDir,
          sliceCacheDir: dirs.sliceCacheDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          pin: ENTRY_PIN,
          outsideReads: NO_OUTSIDE_READS,
          pictureSources: NO_PICTURE_SOURCES,
          hardCapMs: HANG_STOP_MS,
          baseSignal: new AbortController().signal,
          overlap: PASS_FALLBACK_OVERLAP,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        /**
         What reached disk.
         */
        const artifact: unknown = JSON.parse(await readFile(
          join(
            dirs.artifactsDir,
            'CatEntry1.json',
          ),
          'utf8',
        ),);
        expect(stepped.done,).toBe(true,);
        expect((artifact as { durationMs: number; }).durationMs,).toBeGreaterThanOrEqual(0,);
        expect((artifact as { durationMs: number; }).durationMs,).toBeLessThan(HOUR_MS,);
      },
    },),
    ...['critic_report', 'translation_report',].map(function everyProviderDryAt(schema,) {
      return it({
        name: `FILES THE EVERY-PROVIDER-DRY REFUSAL a ${schema} call meets as INCOMPLETE stopped work through the real `
          + 'settle chain, with no pass guard and no abort in play, so the scheduler queues no second attempt and '
          + 'no artifact or page is written',
        fn: async () => {
          await using dirs = await throwawayDirs();
          /**
           Schemas served before the refusal ended the entry.
           */
          const served: string[] = [];
          /**
           Entry outcome captured inside logger scope.
           */
          const result: { outcome?: Awaited<ReturnType<typeof settleEntry>>; } = {};
          const lines = await capturedLines({
            body: async () => {
              result.outcome = await settleEntry({
                client: entryClient({
                  served,
                  failOnSchema: schema,
                  failWith: new EveryProviderDryError({ measured: 'meters read every provider dry', },),
                },),
                entry: ENTRY,
                artifactsDir: dirs.artifactsDir,
                publishDir: dirs.publishDir,
                declinedDir: dirs.declinedDir,
                sliceCacheDir: dirs.sliceCacheDir,
                tip: 'a'.repeat(40,),
                pipelineDigest: DIGEST,
                pin: ENTRY_PIN,
                outsideReads: NO_OUTSIDE_READS,
                pictureSources: NO_PICTURE_SOURCES,
                hardCapMs: HANG_STOP_MS,
                baseSignal: new AbortController().signal,
                overlap: PASS_FALLBACK_OVERLAP,
                stageSettings: SCRIPTED_STAGE_SETTINGS,
              },);
            },
            method: 'log',
          },);
          expect(served,).toContain(schema,);
          expect(result.outcome,).toEqual({ kind: 'stopped', },);
          expect(await artifactNames({ artifactsDir: dirs.artifactsDir, },),).toEqual([],);
          expect(await artifactNames({ artifactsDir: dirs.publishDir, },),).toEqual([],);
          expect(lines.some(function incomplete(line,): boolean {
            return line.startsWith('TALLY CatEntry1 status=INCOMPLETE',);
          },),).toBe(true,);
        },
      },);
    },),

    it({
      name: 'HANDS THE REFUSAL THAT ENDED AN ENTRY TO THE RUN, once, so the pass stops starting entries however the '
        + 'refusal reached the entry, and hands it nothing for a failure that does not end the run',
      fn: async () => {
        /**
         The refusal the entry's critic call meets.
         */
        const dry = new EveryProviderDryError({ measured: 'meters read every provider dry', },);
        /**
         What the run was told ended it, per entry.
         */
        const told: { dry: unknown[]; other: unknown[]; } = {
          dry: [],
          other: [],
        };
        for (const [failWith, into,] of [
          [dry, told.dry,],
          [new Error('scripted provider failure',), told.other,],
        ] as const) {
          // oxlint-disable-next-line no-await-in-loop -- each arm gets its own throwaway directories
          await using dirs = await throwawayDirs();
          // oxlint-disable-next-line no-await-in-loop -- each arm settles its entry in turn
          await capturedLines({
            body: async () => {
              await settleEntry({
                client: entryClient({ served: [], failOnSchema: 'critic_report', failWith, },),
                entry: ENTRY,
                artifactsDir: dirs.artifactsDir,
                publishDir: dirs.publishDir,
                declinedDir: dirs.declinedDir,
                sliceCacheDir: dirs.sliceCacheDir,
                tip: 'a'.repeat(40,),
                pipelineDigest: DIGEST,
                pin: ENTRY_PIN,
                outsideReads: NO_OUTSIDE_READS,
                pictureSources: NO_PICTURE_SOURCES,
                hardCapMs: HANG_STOP_MS,
                baseSignal: new AbortController().signal,
                overlap: PASS_FALLBACK_OVERLAP,
                stageSettings: SCRIPTED_STAGE_SETTINGS,
                endRun: function endRun({ error, },): void {
                  into.push(error,);
                },
              },);
            },
            method: 'log',
          },);
        }
        expect(told.dry,).toEqual([dry,],);
        expect(told.other,).toEqual([],);
      },
    },),

    it({
      name: 'ASKS THE RUN WHETHER IT PARKED FOR THE FAILURE THAT ENDED THE ENTRY, through the real settle chain, and '
        + 'prints the entry PARKED when it did, naming the failure it asks about',
      fn: async () => {
        await using dirs = await throwawayDirs();
        /**
         The refusal the entry's critic call meets, naming a return.
         */
        const dry = new EveryProviderDryError({
          measured: 'meters read every provider dry',
          neuralwatt: { kind: 'at', msOfDay: 69_308_000, },
          neuralwattReturnsInMs: 301_000,
        },);
        /**
         What the run was asked about.
         */
        const asked: unknown[] = [];
        const lines = await capturedLines({
          body: async () => {
            await settleEntry({
              client: entryClient({ served: [], failOnSchema: 'critic_report', failWith: dry, },),
              entry: ENTRY,
              artifactsDir: dirs.artifactsDir,
              publishDir: dirs.publishDir,
              declinedDir: dirs.declinedDir,
              sliceCacheDir: dirs.sliceCacheDir,
              tip: 'a'.repeat(40,),
              pipelineDigest: DIGEST,
              pin: ENTRY_PIN,
              outsideReads: NO_OUTSIDE_READS,
              pictureSources: NO_PICTURE_SOURCES,
              hardCapMs: HANG_STOP_MS,
              baseSignal: new AbortController().signal,
              overlap: PASS_FALLBACK_OVERLAP,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
              endRun: function endRun(): void {},
              parked: function parked({ error, },): boolean {
                asked.push(error,);
                return true;
              },
            },);
          },
          method: 'log',
        },);
        expect(asked,).toEqual([dry,],);
        expect(lines.some(function parkedLine(line,): boolean {
          return line.startsWith('TALLY CatEntry1 status=PARKED ',);
        },),).toBe(true,);
        expect(lines.some(function incompleteLine(line,): boolean {
          return line.startsWith('TALLY CatEntry1 status=INCOMPLETE',);
        },),).toBe(false,);
      },
    },),
  ],
},);
