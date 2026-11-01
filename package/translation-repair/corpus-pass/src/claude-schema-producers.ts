import {
  ADJUDICATION_RESPONSE_FORMAT,
  CANDIDATE_SELECT_RESPONSE_FORMAT,
  CRITIC_RESPONSE_FORMAT,
  INTRODUCED_DEFECT_RESPONSE_FORMAT,
  RESOLUTION_RESPONSE_FORMAT,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import {
  ABSOLUTE_NATURALNESS_REVIEW_RESPONSE_FORMAT,
  ARCHIVE_BLOCK_REVIEW_RESPONSE_FORMAT,
  COVERAGE_RESPONSE_FORMAT,
} from '@monochromatic-dev/translation-repair-archive/ts';
import { PAGE_TITLE_LEXICON_RESPONSE_FORMAT, } from '@monochromatic-dev/translation-repair-artifact/ts';
import {
  CONSOLIDATE_GATE_RESPONSE_FORMAT,
  CONSOLIDATION_POLISH_GATE_RESPONSE_FORMAT,
} from '@monochromatic-dev/translation-repair-consolidate/ts';
import {
  consistencyChoiceResponseFormat,
  consistencyResponseFormat,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import { LANE_CONTEST_RESPONSE_FORMAT, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  REFERENCE_ATTEST_CONFIRM_RESPONSE_FORMAT,
  REFERENCE_ATTEST_RESPONSE_FORMAT,
} from '@monochromatic-dev/translation-repair-reference/ts';
import { RENDERING_AUDIT_RESPONSE_FORMAT, } from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import {
  DERIVABILITY_RESPONSE_FORMAT,
  EDITOR_RESPONSE_FORMAT,
  REFINE_RESPONSE_FORMAT,
  RESTORATION_JUDGE_RESPONSE_FORMAT,
} from '@monochromatic-dev/translation-repair-repair/ts';
import { blockPairingProtocol, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { SECTION_PAIRING_RESPONSE_FORMAT, } from '@monochromatic-dev/translation-repair-stage/ts';
import {
  TRANSLATE_REPAIR_RESPONSE_FORMAT,
  TRANSLATE_RESPONSE_FORMAT,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Claude schema producers
// Every place a stage builds an answer schema, by the file that declares it. The pass-start check reads this list to
// refuse a schema the Claude route cannot send before any provider is asked, and the family test reads the same list.
// A producer that builds more than one schema (the contest ballot asks about the archive or not; the consistency
// sheets name their labels) lists each.

/**
 One file that declares an answer schema, and every schema it can build.
 */
export type SchemaProducer = {
  /**
   Path of the file under `package/translation-repair`, as the family's source scan names it.
   */
  readonly file: string;

  /**
   Each schema the file builds, in the shapes a stage asks for.
   */
  readonly formats: readonly JsonSchemaResponseFormat[];
};

/**
 The labels a consistency sheet shows, for the schemas built per ask.
 */
const SHEET_LABELS = [
  'CANDIDATE 1',
  'CANDIDATE 2',
];

/**
 Every producer of an answer schema in the family.
 */
export const CLAUDE_SCHEMA_PRODUCERS: readonly SchemaProducer[] = [
  {
    file: 'adjudication/src/adjudicate-wire.ts',
    formats: [ADJUDICATION_RESPONSE_FORMAT,],
  },
  {
    file: 'adjudication/src/candidate-select-wire.ts',
    formats: [CANDIDATE_SELECT_RESPONSE_FORMAT,],
  },
  {
    file: 'adjudication/src/critic-wire.ts',
    formats: [CRITIC_RESPONSE_FORMAT,],
  },
  {
    file: 'adjudication/src/introduced-defect-format.ts',
    formats: [INTRODUCED_DEFECT_RESPONSE_FORMAT,],
  },
  {
    file: 'adjudication/src/resolution-wire.ts',
    formats: [RESOLUTION_RESPONSE_FORMAT,],
  },
  {
    file: 'archive/src/absolute-naturalness-review-wire.ts',
    formats: [ABSOLUTE_NATURALNESS_REVIEW_RESPONSE_FORMAT,],
  },
  {
    file: 'archive/src/archive-block-review-wire.ts',
    formats: [ARCHIVE_BLOCK_REVIEW_RESPONSE_FORMAT,],
  },
  {
    file: 'archive/src/coverage-wire.ts',
    formats: [COVERAGE_RESPONSE_FORMAT,],
  },
  {
    file: 'artifact/src/page-title-lexicon-wire.ts',
    formats: [PAGE_TITLE_LEXICON_RESPONSE_FORMAT,],
  },
  {
    file: 'consistency/src/consistency-choice.ts',
    formats: [consistencyChoiceResponseFormat({ labels: SHEET_LABELS, },),],
  },
  {
    file: 'consistency/src/consistency-wire.ts',
    formats: [consistencyResponseFormat({ labels: SHEET_LABELS, },),],
  },
  {
    file: 'lane-contest/src/contest-ballot-wire.ts',
    // The builder is shared by three stages, so the schemas it makes are read as the stages send them.
    formats: [
      LANE_CONTEST_RESPONSE_FORMAT,
      CONSOLIDATE_GATE_RESPONSE_FORMAT,
      CONSOLIDATION_POLISH_GATE_RESPONSE_FORMAT,
    ],
  },
  {
    file: 'reference/src/reference-attest-confirm-wire.ts',
    formats: [REFERENCE_ATTEST_CONFIRM_RESPONSE_FORMAT,],
  },
  {
    file: 'reference/src/reference-attest-wire.ts',
    formats: [REFERENCE_ATTEST_RESPONSE_FORMAT,],
  },
  {
    file: 'rendering-audit/src/rendering-audit-wire.ts',
    formats: [RENDERING_AUDIT_RESPONSE_FORMAT,],
  },
  {
    file: 'repair/src/derivability-wire.ts',
    formats: [DERIVABILITY_RESPONSE_FORMAT,],
  },
  {
    file: 'repair/src/edit-wire.ts',
    formats: [EDITOR_RESPONSE_FORMAT,],
  },
  {
    file: 'repair/src/refine-wire.ts',
    formats: [REFINE_RESPONSE_FORMAT,],
  },
  {
    file: 'repair/src/restoration-judge-wire.ts',
    formats: [RESTORATION_JUDGE_RESPONSE_FORMAT,],
  },
  {
    file: 'slice-reading/src/block-pairing-protocol.ts',
    formats: [
      blockPairingProtocol({
        sourceBlocks: [{
          index: 0,
          text: 'one',
        },],
        targetBlocks: [{
          index: 0,
          text: 'uno',
        },],
        pictureContext: 'A cat sleeps in a sunbeam.',
      },)
        .responseFormat,
    ],
  },
  {
    file: 'stage/src/pair-sections-stage.ts',
    formats: [SECTION_PAIRING_RESPONSE_FORMAT,],
  },
  {
    file: 'translate-floor/src/translate-repair-wire.ts',
    formats: [TRANSLATE_REPAIR_RESPONSE_FORMAT,],
  },
  {
    file: 'translate-floor/src/translate-wire.ts',
    formats: [TRANSLATE_RESPONSE_FORMAT,],
  },
];

//endregion Claude schema producers
