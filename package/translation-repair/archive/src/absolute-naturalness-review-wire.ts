import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  HOUSE_FORM_CORRECTIONS,
  MEASUREMENT_POLICY_BLOCK,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  isJsonRecord,
  withoutTranslatorsNotes,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { selectFence, } from '@monochromatic-dev/translation-repair-stage/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';
import { foldSoftBreaks, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Absolute naturalness review wire

/**
 One paragraph-located material naturalness defect.

 @example
 ```ts
 const finding: AbsoluteNaturalnessFinding = { paragraph: 1, problem: 'Replace source-language word order.' };
 ```
 */
export type AbsoluteNaturalnessFinding = {
  /**
   One-based paragraph number shown in reviewer sheet.
   */
  readonly paragraph: number;

  /**
   Concise actionable naturalness defect.
   */
  readonly problem: string;
};

/**
 Provider reply judging one whole English candidate against absolute publication quality.

 @example
 ```ts
 const reply: AbsoluteNaturalnessReviewWire = { assessment: 'publication-ready English', findings: [], isAcceptable: true };
 ```
 */
export type AbsoluteNaturalnessReviewWire = {
  /**
   Concise explanation of verdict.

   NAMED `assessment` so that it sorts first, which keeps the reason ahead of the verdict on a provider that returns a
   tool call's keys alphabetically (owner, 2026-10-21, "Reason first everywhere"; ledger B762).
   */
  readonly assessment: string;

  /**
   Actionable naturalness defects, empty only for acceptable candidate.
   */
  readonly findings: readonly AbsoluteNaturalnessFinding[];

  /**
   Whether whole candidate meets absolute naturalness floor.

   NAMED `isAcceptable` so that it sorts after `findings` (ledger B762).
   */
  readonly isAcceptable: boolean;
};

/**
 Candidate and context shown to absolute reviewer.

 @example
 ```ts
 const subject: AbsoluteNaturalnessReviewSubject = { sourceText: '猫睡了。', candidateText: 'The cat slept.', paragraphs: ['The cat slept.'] };
 ```
 */
export type AbsoluteNaturalnessReviewSubject = {
  /**
   Chinese passage clarifying deliberate source-language terms.
   */
  readonly sourceText: string;

  /**
   Exact English wording that would ship. The sheet shows it without the notes
   between translators it holds, as it does each of `paragraphs`.
   */
  readonly candidateText: string;

  /**
   Structurally refinable paragraphs in displayed one-based order.
   */
  readonly paragraphs: readonly string[];

  /**
   Whether the line rule governs the passage. On prose the sheet shows each
   paragraph on one line, as it renders (ledger H6); `sourceText`, `candidateText` and
   `paragraphs` stay exact, since the review digests them and the artifact reader recomputes
   those digests from the shipped text. REQUIRED, NOT DEFAULTED, the H2
   lesson.
   */
  readonly lineStructured: boolean;

  /**
   Declared names and public handles candidate must treat as intentional.
   */
  readonly identityContext?: string;
};

/**
 Substantive responsibility assigned to one exact-candidate review.
 */
export type AbsoluteNaturalnessReviewPerspective =
  | 'defect-discovery'
    | 'acceptance-challenge';

/**
 Structured-output contract for absolute naturalness reviewer.
 */
export const ABSOLUTE_NATURALNESS_REVIEW_RESPONSE_FORMAT: JsonSchemaResponseFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'absolute_naturalness_review',
    strict: true,
    schema: {
      type: 'object',
      // DECLARED IN THE ORDER THE KEYS SORT (ledger B762), the reason first by the owner's ruling of 2026-10-21.
      properties: {
        assessment: { type: 'string', },
        findings: {
          type: 'array',
          items: {
            type: 'object',
            properties: {
              paragraph: {
                type: 'integer',
                minimum: 1,
              },
              problem: { type: 'string', },
            },
            required: [
              'paragraph',
              'problem',
            ],
            additionalProperties: false,
          },
        },
        isAcceptable: { type: 'boolean', },
      },
      required: [
        'assessment',
        'findings',
        'isAcceptable',
      ],
      additionalProperties: false,
    },
  },
};

/**
 Checks reviewer reply shape and verdict-to-findings consistency.

 @param value - parsed provider value

 @returns Whether value is usable absolute review

 @example
 ```ts
 if (isAbsoluteNaturalnessReviewWire(value)) consume(value);
 ```
 */
export function isAbsoluteNaturalnessReviewWire(
  value: unknown,
): value is AbsoluteNaturalnessReviewWire {
  if (!isJsonRecord(value,))
    return false;
  if (!('isAcceptable' in value))
    return false;
  if ((typeof value.isAcceptable) !== 'boolean')
    return false;
  if (!('findings' in value))
    return false;
  if (!Array.isArray(value.findings,))
    return false;
  if (!value
    .findings
    .every(function isFinding(finding,): finding is AbsoluteNaturalnessFinding {
      if (!isJsonRecord(finding,))
        return false;
      /**
       Finding fields before primitive validation.
       */
      const {
        paragraph,
        problem,
      } = finding;
      if ((typeof paragraph) !== 'number')
        return false;
      if (!Number.isInteger(paragraph,))
        return false;
      if (paragraph < 1)
        return false;
      if ((typeof problem) !== 'string')
        return false;
      // A problem showing a reader nothing names no defect, invisible
      // characters `trim()` keeps among it (ledger B40); asked of an
      // empty-string comparison, a finding of one zero-width space passed.
      return !rendersAsNothing({ text: problem, },);
    },))
    return false;
  if (!('assessment' in value))
    return false;
  if ((typeof value.assessment) !== 'string')
    return false;
  /**
   Findings after array and element validation.
   */
  const { findings, } = value;
  if (value.isAcceptable)
    return findings.length === 0;
  return findings.length > 0;
}

/**
 Builds independent absolute-quality question over exact would-ship wording.

 THE SHEET SHOWS THE WORDING WITHOUT ITS NOTES BETWEEN TRANSLATORS: the
 candidate and each paragraph are cut with `withoutTranslatorsNotes` against the
 original, so a note the archive's translators left each other is no naturalness
 finding, and a paragraph of nothing but notes is left off the sheet with its
 number unused. The subject itself is not cut, since the review digests it.

 @param subject - source context, exact candidate and declared identities

 @param perspective - distinct first review or prior-acceptance challenge task

 @returns Fenced reviewer conversation

 @example
 ```ts
 const messages = buildAbsoluteNaturalnessReviewMessages({ subject, perspective: 'defect-discovery', });
 ```
 */
export function buildAbsoluteNaturalnessReviewMessages(
  {
    subject,
    perspective = 'defect-discovery',
  }: {
    readonly subject: AbsoluteNaturalnessReviewSubject;
    readonly perspective?: AbsoluteNaturalnessReviewPerspective;
  },
): readonly ChatMessage[] {
  /**
   Substantively different responsibility for initial and confirmation passes.
   */
  const responsibility = (perspective === 'defect-discovery')
    ? `Discover material defects without relying on any prior verdict. Start with sentence-local grammar and collocation, then independently assess paragraph and whole-passage flow.`
    : `A prior editor accepted this exact candidate. Challenge that result rather than repeating its scan. Start from whole-passage register, coherence, and memorial voice, then work backward through each paragraph and sentence to find any overlooked material defect.`;
  // THE REVIEWERS ARE SHOWN THE CANDIDATE WITHOUT THE NOTES THE ARCHIVE'S
  // TRANSLATORS LEFT EACH OTHER, as every other judging sheet is
  // (`withoutTranslatorsNotes`): no reader sees one, the page keeps each one
  // whatever a judge says, and a Chinese comment in an English text is what a
  // reviewer could raise as a naturalness finding. The cut is made here, where
  // the sheet text is built, and not on the subject: `candidateText` and
  // `paragraphs` stay exact because the review digests them and the artifact
  // reader recomputes those digests from the shipped text. The original's own
  // comments stay in view.
  /**
   A text without the notes between translators, whose own comments in the
   original stay.

   @param text - candidate or paragraph text, exact

   @returns Text with each note cut out along with the space or line it stood on
   */
  function withoutNotes({ text, }: { readonly text: string; },): string {
    return withoutTranslatorsNotes({
      text,
      original: subject.sourceText,
    },);
  }
  /**
   The candidate without its notes.
   */
  const candidateShown = withoutNotes({ text: subject.candidateText, },);
  /**
   Each exact paragraph beside the paragraph without its notes, in the
   paragraphs' order.
   */
  const paragraphCuts = subject.paragraphs
    .map(function cutParagraph(paragraph,): {
      readonly exact: string;
      readonly cut: string;
    } {
      return {
        exact: paragraph,
        cut: withoutNotes({ text: paragraph, },),
      };
    },);
  /**
   Fence absent from every untrusted block, those the sheet shows included.
   */
  const fence = selectFence({
    texts: [
      subject.sourceText,
      subject.candidateText,
      candidateShown,
      ...paragraphCuts.flatMap(function bothForms(paragraph,): readonly string[] {
        return [
          paragraph.exact,
          paragraph.cut,
        ];
      },),
      ...((subject.identityContext === undefined) ? [] : [subject.identityContext,]),
    ],
  },);
  /**
   Declared identity context or no block.
   */
  const identity = (subject.identityContext === undefined)
    ? []
    : [
      'DECLARED NAMES (intentional forms, not naturalness defects):',
      `${fence}\n${subject.identityContext}\n${fence}`,
      '',
    ];
  /**
   A text as the reviewer reads it: on prose, each paragraph as it renders.

   @param text - candidate or paragraph text

   @returns Text shown on the sheet
   */
  function shown({ text, }: { readonly text: string; },): string {
    return subject.lineStructured ? text : foldSoftBreaks({ text, },);
  }
  /**
   Numbered refinable paragraphs findings must locate, each keeping the number
   of its place in the exact paragraphs; a paragraph of nothing but notes shows
   nothing a reviewer could judge and is left out, its number unused.
   */
  const paragraphs = paragraphCuts
    .flatMap(function numberParagraph(
      {
        exact,
        cut,
      },
      index,
    ): readonly string[] {
      if ((cut !== exact) && rendersAsNothing({ text: cut, },))
        return [];
      return [`PARAGRAPH ${String(index + 1,)}\n${fence}\n${shown({ text: cut, },)}\n${fence}`,];
    },)
    .join('\n\n',);
  return [
    {
      role: 'system',
      content: `You are an independent publication-quality editor. Judge the ENTIRE English candidate against an absolute naturalness floor, not against another candidate and not by whether it is better than an earlier draft.

REVIEW RESPONSIBILITY:
${responsibility}

Mark acceptable only when the whole candidate reads as idiomatic, publication-ready English. Reject translationese a careful native editor should change: Chinese word order or parts of speech carried into English, calqued verb-object combinations, stacked time or aspect adverbs, repeated generic nouns or pronouns, stiff causal transitions, literal emotional descriptions, unclear references, and ungrammatical coordination.

Perform two independent scans before deciding. First inspect every sentence for local grammar, collocation, word order, and reference defects. Then set those observations aside and reread each complete paragraph plus the whole passage for flow, register, repetition, and any defect the local scan missed. For an unacceptable candidate, return the union of material defects from both scans rather than only the first defect that proves rejection.

Do not reject merely because another optional style is possible. Preserve memorial tone, names, handles, links, Markdown, line structure, and a term the house rules keep in English letters with its gloss. Judge naturalness only; do not rewrite the passage or decide factual fidelity. A departure from a house rule of form is a material defect to report; the house form is ${HOUSE_FORM_CORRECTIONS}.

A prose candidate is shown with each paragraph on one line, as it renders. Single newlines inside one numbered paragraph, where they appear, unless marked as explicit Markdown hard breaks, are soft breaks that render as spaces. Preserve them and do not report their source layout as choppy flow. A flow defect is material only if it remains after replacing each soft break with a space.

For an unacceptable candidate, return one concise actionable finding per material defect and cover every material defect you see. Every finding must name the one-based PARAGRAPH number shown in the sheet. Report only defects in those numbered, structurally correctable paragraphs. For an acceptable candidate, findings must be empty.

${MEASUREMENT_POLICY_BLOCK}

In this review, a candidate obeying a house rule is never unacceptable for it, and no finding may ask it to break one.`,
    },
    {
      role: 'user',
      content: [
        ...identity,
        'ORIGINAL (Chinese, context only):',
        `${fence}\n${subject.sourceText}\n${fence}`,
        '',
        subject.lineStructured
          ? 'EXACT ENGLISH CANDIDATE THAT WOULD SHIP:'
          : 'ENGLISH CANDIDATE THAT WOULD SHIP, each paragraph on one line as it renders:',
        `${fence}\n${shown({ text: candidateShown, },)}\n${fence}`,
        '',
        'STRUCTURALLY CORRECTABLE PARAGRAPHS:',
        paragraphs,
        '',
        'Return only JSON: assessment one sentence, given before the verdict; findings objects with paragraph and problem; isAcceptable boolean.',
      ].join('\n',),
    },
  ];
}

//endregion Absolute naturalness review wire
