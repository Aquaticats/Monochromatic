/**
 Tests for the rendering audit stage, driven through the whole instrument.

 WHY END TO END rather than through the matcher alone: the question this exists
 to answer is whether a defect survives the trip from a scripted reply, through
 anchoring, into an aggregate. The pieces are tested apart in their own files;
 these cases exist to catch a seam between them.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { runRenderingAudit, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the audits under test.
 */
const l = tagged({ tag: 'rendering-audit-test', },);

/**
 Auditors the fixtures configure.
 */
const AUDITORS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 Original passage, carrying a negation the candidate can drop.
 */
const SOURCE_TEXT = '三只猫住在书店的阁楼里。她们不吃罐头，每天傍晚只喝一碗温牛奶。';

/**
 Rendering with the negation dropped.
 */
const CANDIDATE_TEXT = 'Three cats live in the attic of the bookshop. They eat canned food, '
  + 'and every evening they drink one bowl of warm milk.';

/**
 What one scripted auditor answers.
 */
type ScriptedVoice = {
  /**
   Verdict it casts.
   */
  readonly verdict: string;

  /**
   Findings it claims, already in wire shape.
   */
  readonly findings: readonly Readonly<Record<string, string>>[];
};

/**
 One honest polarity finding, located however this voice chose to locate it.

 @param sourceLocator - original span this voice quotes

 @param candidateLocator - candidate span it quotes

 @param reason - what it says the spans amount to

 @returns Finding in wire shape

 @example
 ```ts
 const finding = polarityFinding({ sourceLocator, candidateLocator, reason, },);
 ```
 */
function polarityFinding(
  {
    sourceLocator,
    candidateLocator,
    reason,
  }: {
    readonly sourceLocator: string;
    readonly candidateLocator: string;
    readonly reason: string;
  },
): Readonly<Record<string, string>> {
  return {
    category: 'altered-polarity',
    sourceLocator,
    sourceFocus: '不吃',
    candidateLocator,
    candidateFocus: 'eat canned food',
    reason,
  };
}

/**
 A voice that found nothing.
 */
const QUIET_VOICE: ScriptedVoice = {
  verdict: 'no-defect-found',
  findings: [],
};

/**
 Client answering with one scripted reply per auditor.

 @param script - what each auditor answers, keyed by model id

 @returns Client the stage calls

 @example
 ```ts
 const client = catClient({ script, },);
 ```
 */
function catClient(
  { script, }: { readonly script: Readonly<Record<string, ScriptedVoice>>; },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the audit',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       What this auditor was scripted to answer, or silence.
       */
      const scripted: unknown = script[request.modelId];

      if (scripted === undefined) {
        return {
          kind: 'schema-mismatch',
          rawText: '',
          detail: 'scripted silence',
        };
      }
      if (!request.validate(scripted,))
        throw new Error('scripted payload failed the guard',);

      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the audit',);
    },
  };
}

/**
 Runs one audit over the flipped rendering.

 @param script - what each auditor answers

 @returns What the stage reported

 @example
 ```ts
 const report = await auditWith({ script, },);
 ```
 */
async function auditWith(
  { script, }: { readonly script: Readonly<Record<string, ScriptedVoice>>; },
): Promise<Awaited<ReturnType<typeof runRenderingAudit>>> {
  return runRenderingAudit({
    client: catClient({ script, },),
    subject: {
      sourceText: SOURCE_TEXT,
      candidateText: CANDIDATE_TEXT,
    },
    modelIds: AUDITORS,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: runRenderingAudit.name,
  children: [
    it({
      name:
        'CORROBORATES one defect two auditors located, THROUGH DIFFERENT LOCATORS: the whole rebuild '
          + 'rests on two voices not having to quote the same width of text to be talking about the same '
          + 'thing, and this is the case that proves the seam between anchoring and the matcher holds',
      fn: async () => {
        const report = await auditWith({
          script: {
            [AUDITORS[0] ?? '']: {
              verdict: 'defects-found',
              findings: [
                polarityFinding({
                  sourceLocator: '她们不吃罐头',
                  candidateLocator: 'They eat canned food',
                  reason: 'the original negates this and the candidate does not',
                },),
              ],
            },
            [AUDITORS[1] ?? '']: {
              verdict: 'defects-found',
              findings: [
                polarityFinding({
                  sourceLocator: '她们不吃罐头，每天傍晚只喝一碗温牛奶',
                  candidateLocator: 'They eat canned food, and every evening they drink one bowl of warm milk',
                  reason: 'the candidate asserts what the original denies',
                },),
              ],
            },
            [AUDITORS[2] ?? '']: QUIET_VOICE,
          },
        },);
        expect(report.corroborated,).toHaveLength(1,);
        expect(report.corroborated[0]
          ?.category,).toBe('altered-polarity',);
        expect(report.corroborated[0]
          ?.voices,).toBe(2,);
        expect(report.corroborated[0]
          ?.members,).toHaveLength(2,);
        expect(report.near,).toEqual([],);
      },
    },),
    it({
      name:
        'REPORTS NO CORROBORATED DEFECT when only one auditor found it, while keeping that auditor`s '
          + 'row: a lone claim is evidence about the auditor as much as about the rendering, and '
          + 'discarding it would destroy what a later calibration has to read',
      fn: async () => {
        const report = await auditWith({
          script: {
            [AUDITORS[0] ?? '']: {
              verdict: 'defects-found',
              findings: [
                polarityFinding({
                  sourceLocator: '她们不吃罐头',
                  candidateLocator: 'They eat canned food',
                  reason: 'the negation is gone',
                },),
              ],
            },
            [AUDITORS[1] ?? '']: QUIET_VOICE,
            [AUDITORS[2] ?? '']: QUIET_VOICE,
          },
        },);
        expect(report.corroborated,).toEqual([],);
        expect(report.rows,).toHaveLength(AUDITORS.length,);
        expect(
          report.rows
            .filter(function claimed(row,): boolean {
              return row.findings
                .length > 0;
            },),
        ).toHaveLength(1,);
      },
    },),
    it({
      name:
        'does NOT let one auditor corroborate itself by filing the same defect twice, which is what a '
          + 'count over claims rather than over voices would report as agreement nobody reached',
      fn: async () => {
        const report = await auditWith({
          script: {
            [AUDITORS[0] ?? '']: {
              verdict: 'defects-found',
              findings: [
                polarityFinding({
                  sourceLocator: '她们不吃罐头',
                  candidateLocator: 'They eat canned food',
                  reason: 'the negation is gone',
                },),
                polarityFinding({
                  sourceLocator: '她们不吃罐头',
                  candidateLocator: 'They eat canned food',
                  reason: 'saying it again does not make it two opinions',
                },),
              ],
            },
            [AUDITORS[1] ?? '']: QUIET_VOICE,
            [AUDITORS[2] ?? '']: QUIET_VOICE,
          },
        },);
        expect(report.corroborated,).toEqual([],);
      },
    },),
    it({
      name:
        'drops an unanchored claim before counting, so two auditors agreeing on wording NEITHER text '
          + 'carries corroborate nothing: agreement between voices is not evidence when what they agree on '
          + 'is not in the documents',
      fn: async () => {
        /**
         Two voices quoting a rendering nobody was shown.
         */
        const invented = {
          category: 'unsupported-addition',
          sourceLocator: '',
          sourceFocus: '',
          candidateLocator: 'The cats abandoned the attic before winter.',
          candidateFocus: 'abandoned',
          reason: 'this sentence is not supported',
        };

        const report = await auditWith({
          script: {
            [AUDITORS[0] ?? '']: {
              verdict: 'defects-found',
              findings: [invented,],
            },
            [AUDITORS[1] ?? '']: {
              verdict: 'defects-found',
              findings: [invented,],
            },
            [AUDITORS[2] ?? '']: QUIET_VOICE,
          },
        },);
        expect(report.corroborated,).toEqual([],);
        expect(
          report.rows
            .flatMap(function toDropped(row,): readonly string[] {
              return row.dropped;
            },),
        ).toEqual([
          'unanchored-locator (candidate)',
          'unanchored-locator (candidate)',
        ],);
      },
    },),
    it({
      name:
        'reports two voices who NAMED THE SAME SPAN DIFFERENTLY as a near miss rather than as a defect '
          + 'or as nothing, since which of them is right is a question about the taxonomy that neither was '
          + 'asked and this instrument must not answer by merging',
      fn: async () => {
        const report = await auditWith({
          script: {
            [AUDITORS[0] ?? '']: {
              verdict: 'defects-found',
              findings: [
                polarityFinding({
                  sourceLocator: '她们不吃罐头',
                  candidateLocator: 'They eat canned food',
                  reason: 'the polarity is reversed',
                },),
              ],
            },
            [AUDITORS[1] ?? '']: {
              verdict: 'defects-found',
              findings: [
                {
                  ...polarityFinding({
                    sourceLocator: '她们不吃罐头',
                    candidateLocator: 'They eat canned food',
                    reason: 'the negation was dropped, so it reads as an omission to me',
                  },),
                  category: 'broken-structure',
                },
              ],
            },
            [AUDITORS[2] ?? '']: QUIET_VOICE,
          },
        },);
        expect(report.corroborated,).toEqual([],);
        expect(report.near,).toHaveLength(1,);
        expect(report.near[0]
          ?.kind,).toBe('same-focus-different-category',);
      },
    },),
    it({
      name:
        'accounts for an auditor it could not hear rather than reading silence as agreement, since a '
          + 'lost voice shrinks the roster that could have corroborated anything',
      fn: async () => {
        const report = await auditWith({
          script: {
            [AUDITORS[0] ?? '']: {
              verdict: 'defects-found',
              findings: [
                polarityFinding({
                  sourceLocator: '她们不吃罐头',
                  candidateLocator: 'They eat canned food',
                  reason: 'the negation is gone',
                },),
              ],
            },
            [AUDITORS[1] ?? '']: {
              verdict: 'defects-found',
              findings: [
                polarityFinding({
                  sourceLocator: '她们不吃罐头',
                  candidateLocator: 'They eat canned food',
                  reason: 'the candidate says the opposite',
                },),
              ],
            },
          },
        },);
        expect(report.rows,).toHaveLength(2,);
        expect(report.corroborated,).toHaveLength(1,);
        expect(
          report.rows
            .map(function toId(row,): string {
              return row.modelId;
            },),
        ).not
          .toContain(AUDITORS[2],);
      },
    },),
  ],
},);
