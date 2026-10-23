import type { ChatMessage, } from '@monochromatic-dev/module-llm-type/ts';

import {
  BILINGUAL_LINE_CLAUSE,
  HOUSE_POLICY_BLOCK,
  NAME_FORM_SCOPE_RULE,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  isJsonRecord,
  QUOTED_WORDS_REMINDER,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { selectFence, } from '@monochromatic-dev/translation-repair-stage/ts';
import {
  type BreakRendering,
  renderedBreakContract,
  renderedBreakFacts,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { sourceBreakDisplay, } from './source-break-display.ts';
import { PAGE_APPARATUS_IS_KEPT, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  IncumbentKind,
  TranslateAbsenceReason,
} from './translate-absence.ts';

//region Translate wire
// The translator sheet and the guard on what comes back, both of which
// `runTranslateStage` uses on every slice. Written as a prototype for the
// translate-first re-design proposal, and in production since the lane was built.
//
// `doc/decision/translation-repair-output-goal.md` decides that the pipeline
// yields a good translation of the ORIGINAL even where the translation fed in
// does not make sense. The stages that exist cannot express that. Every one of
// them is defect-driven: critics find defects in the input translation, a panel
// adjudicates them, an editor fixes the accepted ones. A passage with no
// translation can only enter that loop as an omission filed against text that
// is not there, which is what produced omission after omission on a section
// running 76 source blocks against 5 target blocks, with the editor authoring
// English one issue at a time and no stage aware it was translating.
//
// This asks the question the loop cannot: render this passage. The existing
// translation is offered as EVIDENCE and as a starting point worth keeping
// where it is right, never as the thing being corrected, which is the same
// anchoring the introduced-defect probe was moved to on 2026-08-12 after the
// old wording made the pre-edit translation the standard of accuracy.

/**
 Instructions every translator call shares.

 The prior translation is described as partial and possibly wrong on purpose.
 Told to "improve" it, a model treats its wording as the baseline and edits
 around it, which is how a passage that was never translated ends up
 paraphrased rather than rendered. Told it may be absent, it translates.
 */
const TRANSLATE_RULES =
  `You are a bilingual Chinese-to-English translator working on a memorial archive.

Render the ORIGINAL passage into English.

An EXISTING TRANSLATION may be shown. It may be complete, partial, wrong, or absent altogether. Treat it as evidence about how this archive reads and as wording worth keeping WHERE IT IS RIGHT. It is never the standard: your output is judged against the ORIGINAL.

Rules:
- Translate everything the ORIGINAL says. Content the existing translation never covered is the point of this call, not an optional extra.
- Keep wording from the existing translation wherever it is already a good rendering. Reaching the same English by different words is not an improvement, and a reader who knows this archive should not see it churn.
- Preserve every Markdown structure the ORIGINAL uses: block quotes, list markers, headings, footnote markers, links, and the paragraph breaks between blocks. WHERE THE EXISTING TRANSLATION SHAPES THE PASSAGE DIFFERENTLY, splitting one block into two, merging two into one, or quoting a line the ORIGINAL runs as prose, KEEP THE EXISTING TRANSLATION'S SHAPE. It is the shape the archive's page already has, and a rendering that drops one of its blocks deletes that block from the page. Where the EXISTING TRANSLATION stands a block of another kind in place of one the ORIGINAL has (a farewell line where the original has a poem in a block quote), keep the existing block and add the original's after it in the original's own kind. The same holds for a link whose destination the EXISTING TRANSLATION renders differently from the ORIGINAL (a moved domain, an article in the reader's language): keep the existing destination, and never carry both.
- ONE TENSE, IN THIS ORDER. The ORIGINAL marks no tense, so the English decides: first the past tense the house rule sets for the life of a person who has died (a line of the life the EXISTING TRANSLATION tells in the present is moved to the past, and that move is a correction the house rule asks for, not a change of your own); then the tense the page holds for what is still true as it is read. Moving a passage the EXISTING TRANSLATION tells in the past into the present is a change you made. This is not the shape rule and the verse rule does not displace it. Never let one sentence start in one tense and finish in another. ${QUOTED_WORDS_REMINDER}
- When used to refer to their entities, names, handles and place names already used by the existing translation are authoritative, and you keep them exactly even where they match the original neither phonetically nor semantically. WHERE DECLARED NAMES IS SHOWN AND SPELLS THE SAME PERSON OR PLACE DIFFERENTLY, THE DECLARED SPELLING WINS, because the archive's prose contradicts its own front matter on real pages and the front matter is what the archive declares. Where DECLARED NAMES gives more than one form, any declared form is right. Never invent a third spelling of your own. A name the existing translation left in Han is not a rendering to keep: render it as the house rules say. A DECLARED NAMES line beginning "target contributor:" records a public identity from the existing English attribution. Preserve that spelling exactly on contributor lines; a source-script name or different handle does not override it. ${NAME_FORM_SCOPE_RULE}
- ${PAGE_APPARATUS_IS_KEPT}
- Do not add content the ORIGINAL does not support. From the existing translation carry only its apparatus, as the rule on page apparatus says, never an event, an action, a method, a time, a cause or a characterization the ORIGINAL does not state.

${HOUSE_POLICY_BLOCK}`;

/**
 Reply-format instruction, kept LAST in the assembled sheet.

 Split out so a conditional rule can be inserted before it. Wire instructions
 that end up above content rules are the ones models drop first.
 */
const TRANSLATE_REPLY_RULE =
  'Reply with ONLY a JSON object of shape {"translation": "..."}. No prose, no code fences, no commentary.';

/**
 Instruction added when the enclosing chunk's ORIGINAL is line-structured.

 Written for a translator rather than an editor, which is why it is not
 `LINE_STRUCTURE_RULE` from `line-structure-addendum.ts`. That one asks an
 editor to leave existing lines where they are; here there may be no existing
 lines at all, and the shape has to be built from the original instead.

 The failure it answers is `Toka_ls`, whose verse chunk runs 21 source blocks
 at median 22 characters against 18 target blocks at median 101: the existing
 translation already merged the lines. A translator shown that translation and
 told nothing would keep reproducing the merge, since the only shape in front
 of it is the merged one.

 SHARED WITH THE CONSOLIDATE WIRE, whose producer is a translator too. A
 second wording of the same rule would drift from the one `Toka_ls` was
 measured against.

 SAYS OUTRIGHT THAT IT OUTRANKS THE SHAPE RULE, because the two disagree on
 exactly the case this exists for. `TRANSLATE_RULES` tells a producer to keep
 the existing translation's shape where it merges blocks the ORIGINAL keeps
 apart, and on `Toka_ls` that means keeping 18 blocks where the Chinese has
 21. Both rules arrive in one system prompt and neither used to defer, so a
 producer met a contradiction and resolved it however it liked.

 THE GUARD AGREES WITH THIS RULE AND CANNOT ENFORCE IT.
 `validateTranslatedSlice` refuses a candidate that MERGES the page's blocks,
 and allows extra blocks only where the ORIGINAL has them: measured, a
 three-block candidate against a one-block page is valid when the source is
 verse and invalid when it is prose. So the unmerge this rule asks for is
 licensed rather than punished, and a producer that instead kept the page's
 merge is equally valid. Nothing downstream would have caught the wrong
 choice, which is why precedence is stated here.
 */
export const TRANSLATE_LINE_STRUCTURE_RULE: string = 'The ORIGINAL is line-structured: each '
  + 'original line is a unit. THIS RULE OUTRANKS THE STANDING RULE ASKING YOU TO '
  + 'KEEP THE EXISTING TRANSLATION\'S SHAPE, which governs prose and not verse. '
  + 'Produce one output line per original line, in the '
  + 'same order. Never merge two original '
  + 'lines into one output line, never split one across two, and never invent or '
  + `drop a line. Where the EXISTING TRANSLATION has merged lines, unmerge them. ${BILINGUAL_LINE_CLAUSE}`;

/**
 Instruction added when the entry attested archive details a cited reference
 states (class thirty-nine, 2026-09-16).

 Mio23 slice 2: the archive's "She has an older sister who is also trans" was
 attested by five of six voices against the blog the ORIGINAL cites, the
 repair lane kept it, and the translate lane, written from the ORIGINAL with
 the archive as evidence only, rendered "and she has an older sister". The
 lane contest chose translate three ballots to two on other merits, one
 ballot calling the repair candidate wrong for keeping the clause, and the
 page shipped without it. A lane that cannot carry the detail loses it for
 the page whenever it wins, so the translators are told which details are
 attested and to carry them.
 */
export const TRANSLATE_ATTESTED_RULE: string = 'ATTESTED DETAILS lists details the EXISTING TRANSLATION carries that a '
  + 'page the ORIGINAL itself cites states, each checked word for word. Every one is accurate detail the '
  + 'translator took from the ORIGINAL\'s own references: carry every one in your rendering, in the existing '
  + 'translation\'s words where they fit. The ORIGINAL not stating it is not a reason to drop it, and a '
  + 'rendering that drops one drops accurate detail from the page. They license nothing else: from the existing '
  + 'translation carry only these and its apparatus, never other content the ORIGINAL does not state. '
  + 'Reader protection outranks them: in the account of how a death or an attempt happened, a method, a '
  + 'substance, a dose, or a place that was the means stays out however plainly a reference states it; a means '
  + 'named outside that account is not covered by this rule.';

/**
 Instruction added whenever the sheet shows the pictures block.

 THE ONE HUNDRED SIXTEENTH CLASS (zheermao11, 2026-09-24). The block carries
 the pictures in and beside the passage, one slice each way, and the sheet
 never said what it was for, so the writers rendered a neighbouring picture's
 transcript into a passage whose ORIGINAL is a picture component or the
 sentence introducing it. Every such candidate ran past the produced volume
 bound and was cut: 22 translate streams on that run and 31 to 43 on every
 earlier zheermao run, and with the reasoning seats reachable each cut came
 after 60 to 170 seconds of thinking, so the translate lane took 616 seconds
 against zheermao9's 77. A passage whose archive rendering does carry a
 transcript (the archive translated the picture in that slice's span) keeps
 it, which is why the scope names the existing translation.
 */
export const TRANSLATE_PICTURE_SCOPE_RULE: string = 'WHAT THE PICTURES HERE SAY transcribes the pictures in and '
  + 'beside this passage so that words about them read right; it is not part of the ORIGINAL passage. Render a '
  + 'picture\'s words only where the ORIGINAL passage itself writes them out or the EXISTING TRANSLATION of this '
  + 'passage already carries them. Render a picture component (a tag naming picture files) exactly as the ORIGINAL '
  + 'has it, with no transcript added beside it: a transcript neither the passage nor its existing translation '
  + 'carries is an addition, and where the page translates that picture in a block of its own it would stand on '
  + 'the page twice.';

/**
 The same scope for a judge, spliced into the label of the pictures evidence
 on the translate slate and the consolidation slate (ledger S16, H7).

 WRITTEN FOR A JUDGE: the writer's rule tells a producer what to render; a
 judge reading the transcript with no rule saw a candidate rendering a
 neighbouring picture's words as faithful, and one leaving a picture component
 bare as having dropped them.

 @example
 ```ts
 const label = `WHAT THE PICTURES HERE SAY; ${JUDGE_PICTURE_SCOPE_RULE}`;
 ```
 */
export const JUDGE_PICTURE_SCOPE_RULE: string = 'it is not part of the ORIGINAL passage: a candidate '
  + 'rendering a picture\'s words the passage does not write out and the existing '
  + 'translation of this passage does not carry has added them, and a candidate keeping a picture component exactly '
  + 'as the ORIGINAL has it has dropped nothing.';

/**
 Instruction for visible YAML page metadata.
 */
export const TRANSLATE_FRONT_MATTER_RULE: string = 'The passage is complete YAML front matter, including its '
  + '--- fence lines. Return one complete YAML front matter block and nothing outside it. Preserve every field name, '
  + 'nesting level, container length, scalar kind, comment, and fence. Translate human-language values from the '
  + 'ORIGINAL into natural English. For this metadata only, ORIGINAL field values are source facts and outrank the '
  + 'existing translation. The name field is the visible person name, never an entry directory id. When ORIGINAL name '
  + 'and info.alias are the same identity, the translated name must appear among the comma-separated renderings in '
  + 'translated info.alias, which may also carry the original script and other renderings; a name that appears nowhere '
  + 'in its alias is invalid. In info.location comments, keep the established target contributor spelling '
  + 'after `, by ` where source and archive spell that contributor differently. Existing aliases, handles, and contributor '
  + 'names may supply established English forms only where they identify the same source value. '
  + 'THIS RULE OUTRANKS the general rule that names already used by the existing translation are authoritative.';

/**
 Messages for one translation call.

 @example
 ```ts
 const messages = buildTranslateMessages({ sourceText, existingText, },);
 ```
 */
export type TranslatePromptPlan = {
  /**
   Messages ready for `chatJson`.
   */
  readonly messages: readonly ChatMessage[];
};

/**
 Latest exact rejected slate and structured findings grounding next rendering.

 @example
 ```ts
 const evidence: TranslateFollowupEvidence = {
   reason: 'declined-rejection',
   candidateTexts: ['The cat sleeps.',],
   findings: ['translate-declined (rejection)',],
 };
 ```
 */
export type TranslateFollowupEvidence = {
  /**
   Why latest slate could not fill absent passage.
   */
  readonly reason: TranslateAbsenceReason;

  /**
   Exact candidate texts latest panel rejected.
   */
  readonly candidateTexts: readonly string[];

  /**
   Latest structured stage findings.
   */
  readonly findings: readonly string[];
};

/**
 Builds the translator sheet for one passage.

 @param sourceText - original passage to render

 @param existingText - translation as it stands, empty when there is none

 @param incumbentKind - explicit source-only provenance for break presentation;
 callers that cannot establish it keep the original view

 @param identityContext - declared names and handles, omitted when absent

 @param attestedLines - archive details a cited reference states, one line
 each as the sheets carry them, none when nothing was attested

 @param syntax - syntax role requiring dedicated preservation rules

 @param followupEvidence - latest exact rejected slate and findings when this
 is stage-local repair rather than initial rendering

 @param lineStructured - whether the enclosing CHUNK's original is
 line-structured, decided by the caller because a slice is too small a unit to
 decide it on; see `buildEditorAddendum`

 @returns Messages for the call

 @example
 ```ts
 const plan = buildTranslateMessages({ sourceText, existingText: '', },);
 ```
 */
export function buildTranslateMessages(
  {
    sourceText,
    existingText,
    incumbentKind = 'present',
    identityContext = '',
    pictureContext = '',
    attestedLines = [],
    archiveDisputeNote = '',
    syntax,
    followupEvidence,
    lineStructured = false,
  }: {
    readonly sourceText: string;
    readonly existingText: string;
    readonly incumbentKind?: IncumbentKind;
    readonly identityContext?: string;
    readonly archiveDisputeNote?: string;
    readonly pictureContext?: string;
    readonly attestedLines?: readonly string[];
    readonly syntax?: SliceSyntax;
    readonly followupEvidence?: TranslateFollowupEvidence;
    readonly lineStructured?: boolean;
  },
): TranslatePromptPlan {
  /**
   Presentation-only break spelling, with canonical source still used for facts.
   */
  const sourceForModel = sourceBreakDisplay({
    sourceText,
    archiveText: existingText,
    incumbentKind,
    ...((syntax === undefined) ? {} : { syntax, }),
  },);
  /**
   Fence no enclosed text can reproduce, chosen against every string this
   sheet carries, since all of them are arbitrary prose.
   */
  const fence = selectFence({
    texts: [
      sourceForModel,
      existingText,
      identityContext,
      pictureContext,
      archiveDisputeNote,
      ...attestedLines,
      ...(followupEvidence?.candidateTexts ?? []),
      ...(followupEvidence?.findings ?? []),
    ],
  },);

  /**
   Candidates the judges declined, under the labels their blocks carry, none
   on the initial round. Their break counts go to the sheet as well, since
   the judges who declined them had those counts (ledger B29).
   */
  const rejectedRenderings: readonly BreakRendering[] = (followupEvidence?.candidateTexts ?? [])
    .map(function rejectedRendering(
      text,
      index,
    ): BreakRendering {
      return {
        label: `REJECTED CANDIDATE ${String(index + 1,)}`,
        text,
      };
    },);

  /**
   Translator sheet, with the line-structure fact inserted above the reply
   instruction when the enclosing chunk's original is verse.
   */
  const system = [
    TRANSLATE_RULES,
    followupEvidence === undefined
      ? ''
      : `This is stage-local repair after prior ${followupEvidence.reason}. Produce a materially new rendering that resolves every listed finding and does not repeat any rejected candidate. Recheck complete source coverage, fidelity, Markdown structure, contributor identity, and natural English.`,
    syntax === 'front-matter' ? TRANSLATE_FRONT_MATTER_RULE : '',
    lineStructured ? TRANSLATE_LINE_STRUCTURE_RULE : '',
    (attestedLines.length === 0) ? '' : TRANSLATE_ATTESTED_RULE,
    (pictureContext === '') ? '' : TRANSLATE_PICTURE_SCOPE_RULE,
    // THE INSTRUCTION ONLY. The measured counts are this call's own values, so they ride in the user turn
    // (R4): a system prompt that carried them would be a new cache entry for every passage.
    renderedBreakContract({
      sourceText,
      archiveText: existingText,
      ...((syntax === undefined) ? {} : { syntax, }),
    },),
    TRANSLATE_REPLY_RULE,
  ]
    .filter(function isPresent(part,): boolean {
      return part !== '';
    },)
    .join('\n\n',);

  /**
   Exact rejected candidates rendered as independently fenced blocks.
   */
  const rejectedCandidates = rejectedRenderings
    .map(function rejectedCandidate(rendering,): string {
      return `${fence} ${rendering.label} ${fence}\n${rendering.text}`;
    },)
    .join('\n',);
  /**
   Measured break counts of the original and of the rejected candidates, none where the
   instruction is not due.
   */
  const breakFacts = renderedBreakFacts({
    sourceText,
    archiveText: existingText,
    ...((syntax === undefined) ? {} : { syntax, }),
    renderings: rejectedRenderings,
  },);
  /**
   Latest structured findings as prompt text.
   */
  const followupFindings = followupEvidence === undefined
    ? ''
    : followupEvidence
      .findings
      .join('\n',);
  /**
   Latest rejection evidence shown only on stage-local repair.
   */
  const followupSection = followupEvidence === undefined
    ? ''
    : `
${fence} LATEST REJECTION ${fence}
reason: ${followupEvidence.reason}
findings:
${followupFindings}
${rejectedCandidates}`;

  return {
    messages: [
      {
        role: 'system',
        content: system,
      },
      {
        role: 'user',
        content: `${fence} ORIGINAL ${fence}
${sourceForModel}${
          pictureContext === ''
            ? ''
            : `
${fence} WHAT THE PICTURES HERE SAY ${fence}
${pictureContext}`
        }
${fence} EXISTING TRANSLATION ${fence}
${existingText === '' ? '(none: this passage has no translation yet)' : existingText}${
          archiveDisputeNote === ''
            ? ''
            : `
${fence} ARCHIVE RENDERING DISPUTED ${fence}
${archiveDisputeNote}`
        }${
          (attestedLines.length === 0)
            ? ''
            : `
${fence} ATTESTED DETAILS ${fence}
${attestedLines.join('\n',)}`
        }${
          identityContext === ''
            ? ''
            : `
${fence} DECLARED NAMES ${fence}
${identityContext}`
        }${followupSection}${
          breakFacts === ''
            ? ''
            : `
${fence} RENDERED LINE STRUCTURE, measured facts ${fence}
${breakFacts}`
        }
${fence} END ${fence}`,
      },
    ],
  };
}

/**
 One translator reply on the wire.

 @example
 ```ts
 const wire: TranslateReportWire = { translation: 'The cat naps.', };
 ```
 */
export type TranslateReportWire = {
  /**
   Rendered English for the whole passage.
   */
  readonly translation: string;
};

/**
 Guards a translator reply.

 A REPLY THAT SAYS NOTHING IS NOT A REPLY. The structured-output schema is
 satisfied by `{"translation": ""}`, which used to arrive as a heard voice
 proposing to render the passage as nothing and was then dropped further down
 while the model that sent it was recorded as answered. Refusing it here makes
 it a lost voice instead, so the roster re-asks that model in the next round
 and the loss is reported as one; a slice with no translation in the archive
 has nothing else to fall back on, which is where the difference is felt.

 Every source slice says something, so no legitimate reply is blank: an empty
 run cannot become a slice at all.

 BLANK MEANS SHOWING A READER NOTHING (ledger B40), not what `trim()` leaves:
 a reply of one zero-width space passed a trimmed check, the intake fold then
 emptied it, and an empty candidate reached the judges.

 @param value - parsed model JSON

 @returns Whether value carries a translation that shows something

 @example
 ```ts
 const ok = isTranslateReportWire(JSON.parse(text,),);
 ```
 */
export function isTranslateReportWire(value: unknown,): value is TranslateReportWire {
  if (!isJsonRecord(value,))
    return false;
  if ((typeof value.translation) !== 'string')
    return false;

  return !rendersAsNothing({ text: value.translation, },);
}

/**
 Structured-output constraint for translator calls.
 */
export const TRANSLATE_RESPONSE_FORMAT: JsonSchemaResponseFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'translation_report',
    schema: {
      type: 'object',
      required: ['translation',],
      additionalProperties: false,
      properties: { translation: { type: 'string', }, },
    },
  },
};

//endregion Translate wire
