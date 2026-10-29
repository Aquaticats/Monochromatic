import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type {
  AdjudicatedIssue,
  EditableEnvelope,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  ADDITION_IS_REMOVED_NOT_SOFTENED,
  DECLARED_IDENTITY_RULES,
  declaredNamesBlock,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import { citedReferenceBlockText, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  communityRenderingsBlock,
  FOREIGN_PHRASE_NAME_TITLE_SCOPE,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  MARKUP_ATOM_SHEET_NAMES,
  MARKUP_IDENTIFIER_KINDS,
} from '@monochromatic-dev/translation-repair-markup/ts';
import {
  flattenSpace,
  HOUSE_POLICY_BLOCK,
  NAME_FORM_SCOPE_RULE,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { selectFence, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Editor prompt
// One prompt per editor model per chunk: the document pair plus numbered
// edit regions, each carrying the accepted issues it serves and its exact
// current text with disambiguating context. Editors answer with region
// numbers, never offsets or hashes; the resolver binds numbers back to
// envelopes from the same plan that built the prompt, and the deterministic
// apply gate proves every change stayed inside its envelope.


/**
 What the neighbouring blocks are for, stated inside the editor sheet.

 THE EDITOR IS WHERE THE WINDOW EITHER PAYS OR DOES HARM, so its rule is the
 strictest of the three. The `lintong` damage is an editor outcome: handed
 a one-sentence original against a four-sentence incumbent, it repaired the
 sentence it could match and left the other three in place, and the neighbour's
 own new translation then said them a second time. Seeing next door is what
 lets it recognise that case.

 THE OPPOSITE FAILURE IS WORSE AND IS NAMED HERE TOO. An editor that removes a
 passage because it can see it next door, when the neighbouring slice does not
 in fact keep it, deletes content the document had. Removal is therefore
 allowed only against wording the NEARBY EXISTING TRANSLATION already carries,
 which is text that is already in the document rather than text some other
 stage might yet produce.
 */
const NEARBY_RULE = 'THE TWO NEARBY BLOCKS ARE CONTEXT AND MUST NOT BE EDITED. '
  + 'Never copy them into your output. If a region here repeats wording the '
  + 'NEARBY EXISTING TRANSLATION already carries, that repetition is the defect '
  + 'and removing it here is correct. Remove nothing on the grounds that a '
  + 'neighbour OUGHT to carry it';

/**
 Saying something twice is adding content, stated for the editors (ledger
 L14(b)). The audit found editors writing neighbouring text into a region,
 and 56 of 6,222 chunk candidates over every artifact carried a sentence
 more often than the slice's archive English did. NEARBY_RULE forbids copying
 the NEARBY blocks and nothing forbade repeating the translation outside the
 region, which this sheet shows whole. Conditioned on the ORIGINAL because
 the sheet's omission rules say to translate ALL of the missing content, and a
 refrain the TRANSLATION rendered once is such content.
 */
const REPEATED_CONTENT_EDITOR_RULE = 'Write into a region only what the ORIGINAL says at that place. A sentence the '
  + 'TRANSLATION already carries outside the region, or content a NEARBY ORIGINAL passage says, goes into it only '
  + 'when the ORIGINAL says it again at that place, as a refrain the TRANSLATION rendered once; otherwise the page '
  + 'says it twice.';

/**
 Characters of surrounding document shown on each side of a region,
 so editors locate it even when the base text recurs elsewhere.
 */
const REGION_CONTEXT_CHARS = 40;

/**
 Joins sheet names into one English list, with "and" before the last.

 @param names - names in sheet order, at least two, since both halves of the markup kinds hold that many

 @returns List as prose

 @throws {@link Error} when fewer than two names are given, which neither half of `MARKUP_ATOM_SHEET_NAMES`
 produces

 @example
 ```ts
 const listed = listedNames({ names: ['tags', 'inline code',], },);
 ```
 */
function listedNames({ names, }: { readonly names: readonly string[]; },): string {
  if (names.length < 2) {
    throw new Error(
      `unreachable: the sheet list [${names.join(', ',)}] holds fewer than two names, though each half of `
        + 'MARKUP_ATOM_SHEET_NAMES holds more than one kind',
    );
  }
  return `${names.slice(
    0,
    -1,
  )
    .join(', ',)} and ${nonNullishOrThrow(names.at(-1,),)}`;
}

/**
 Sheet names of the identifier kinds, which an edit may move but never
 change, built from the gate's own split (ledger L4).
 */
const IDENTIFIER_NAMES: readonly string[] = [...MARKUP_IDENTIFIER_KINDS,]
  .map(function toName(kind,): string {
    return MARKUP_ATOM_SHEET_NAMES[kind];
  },);

/**
 Sheet names of the kinds the translation may have authored itself, which an
 edit may correct but never drop: every other kind, in sheet order.
 */
const REMARKABLE_NAMES: readonly string[] = Object.values(MARKUP_ATOM_SHEET_NAMES,)
  .filter(function isRemarkable(name,): boolean {
    return !IDENTIFIER_NAMES.includes(name,);
  },);

/**
 The markup rule, as the preservation gate enforces it: identifiers kept or
 moved, markup the original carries kept, markup the translation wrote
 correctable, and the one removal the gate licenses.
 */
const MARKUP_ATOM_RULE = `Keep ${listedNames({ names: IDENTIFIER_NAMES, },)} character for character; `
  + 'one may move to the clause it belongs to, but never change or disappear. '
  + `Keep ${listedNames({ names: REMARKABLE_NAMES, },)} character for character where the ORIGINAL carries them; `
  + 'ones the TRANSLATION wrote itself may be corrected, for instance to the ORIGINAL\'s form, but never dropped. '
  + 'The one removal allowed is markup inside text an accepted accuracy/addition issue quotes, '
  + 'which goes with the detail it carries.';

/**
 Rule list of the editor system prompt; a calibration addendum splices after
 its last rule line, which is why the house rules are NOT part of it.
 */
const EDITOR_RULES_HEAD = `You are a careful bilingual translation editor.
Reviewers confirmed the numbered issues below in the TRANSLATION of the ORIGINAL document.
Fix them by rewriting ONLY the numbered edit regions.

Rules, strictly enforced by a machine:
- For each region you fix, return its number and the full replacement text for exactly that region.
- Change as little as possible; keep the surrounding wording untouched.
- An empty CURRENT TEXT marks omitted content: write the missing translation there, matching the surrounding style, and include any spacing the insertion needs.
- When an issue reports omitted content, translate ALL of the missing content in full sentences; never summarize, compress, or drop clauses. A short paraphrase of a long omission is not a fix.
- When a region fills an omission, first locate the exact ORIGINAL sentence or sentences the omission corresponds to, then translate them clause by clause: every clause and every detail of those ORIGINAL sentences must appear in your replacement, none merged or summarized away.
- Write natural, idiomatic prose carrying the ORIGINAL's feeling: its voice, warmth, humour, grief, or irony. Emotional completeness and naturalness outrank word-for-word correspondence, so recast wording, sentence boundaries, and clause order freely when that serves the feeling. A stiff literal rendering that loses the feeling is not a fix.
- Naturalness never licenses dropping content: every detail of the ORIGINAL must survive your rewrite, recast rather than removed.
- Keep any phrase the ORIGINAL writes in a language other than its own in that original wording, and put its meaning alongside it, following whatever convention the document already uses for this, otherwise in parentheses immediately after. Never replace such a phrase with its meaning alone. ${FOREIGN_PHRASE_NAME_TITLE_SCOPE}
- Render ordinary prose in the ORIGINAL's own language fully into the TRANSLATION's language, including inside quotations and stylized multilingual lines. A word or character that is itself the subject of discussion is not ordinary prose to normalize. ${NAME_FORM_SCOPE_RULE}
- ${MARKUP_ATOM_RULE}
- Never introduce content the ORIGINAL does not support.
- ${REPEATED_CONTENT_EDITOR_RULE}
- ${ADDITION_IS_REMOVED_NOT_SOFTENED}
- When the CURRENT TEXT is line-structured, meaning short lines separated by blank lines rather than paragraphs, the line is the unit: keep one output line per input line, in the same order, and recast only within a line. Never merge, split, reorder or invent lines.
- Omit a region entirely when you cannot fix it faithfully; a skipped region stays unchanged.
- THE HOUSE RULES BELOW OUTRANK EVERY RULE IN THIS LIST. Where a detail is absent from the TRANSLATION because reader protection asks for it in the account of how a death or an attempt happened, the region is not an omission to fill and the issue reporting it is wrong: omit that region entirely rather than restoring the detail. A means named anywhere else is not covered by reader protection, and the region is restored as the ORIGINAL states it.`;

/**
 Rule list followed by the rules this corpus is written under.

 WHY THE EDITOR NEEDED THEM, and why this is the stage that needed them most.
 `house-policy.ts` names the failure it exists to stop: a critic ignorant of
 reader protection reports a deliberately vague passage as an omission, and
 the editor RESTORES the detail the rule exists to remove. The critic was
 given the block. The editor never was, though the block's own comment claims
 it as a consumer, so the second half of that sentence ran unguarded while
 five separate rules here told the editor that every detail of the original
 must survive and that an omission must be filled in full.
 */
const EDITOR_RULES_BLOCK = `${EDITOR_RULES_HEAD}

${HOUSE_POLICY_BLOCK}`;

/**
 Reply-shape block closing the editor system prompt.
 */
const EDITOR_REPLY_BLOCK = `Reply with ONLY a JSON object of shape {"edits": [{"newText": "...", "region": 1}]}. No prose, no code fences.`;

/**
 System instructions shared by every baseline editor call.
 */
const EDITOR_SYSTEM_PROMPT = `${EDITOR_RULES_BLOCK}

${EDITOR_REPLY_BLOCK}`;

/**
 Messages plus the envelope order edits resolve through:
 region number N on the wire means `envelopes[N - 1]`.

 @example
 ```ts
 const plan: EditorPromptPlan = buildEditorMessages({
   sourceText,
   targetText,
   envelopes,
   issues,
 },);
 ```
 */
export type EditorPromptPlan = {
  /**
   Messages ready for `chatJson`.
   */
  readonly messages: readonly ChatMessage[];

  /**
   Envelopes in prompt numbering order.
   */
  readonly envelopes: readonly EditableEnvelope[];
};

/**
 One region block of the prompt sheet.

 @param envelope - envelope under presentation

 @param regionNumber - one-based number on the sheet

 @param targetText - full translation for context extraction

 @param issues - adjudicated issues for summary lookup

 @returns Rendered region block

 @example
 ```ts
 regionBlock({ envelope, regionNumber: 1, targetText, issues, },);
 ```
 */
function regionBlock(
  {
    envelope,
    regionNumber,
    targetText,
    issues,
  }: {
    readonly envelope: EditableEnvelope;
    readonly regionNumber: number;
    readonly targetText: string;
    readonly issues: readonly AdjudicatedIssue[];
  },
): string {
  /**
   Issue lines for every issue this envelope serves.
   */
  const issueLines = envelope
    .issueIds
    .flatMap(function toLine(issueId,) {
      /**
       Issue behind this id, when the caller supplied it.
       */
      const issue = issues.find(function matches(candidate,) {
        return candidate.issueId === issueId;
      },);
      if (issue === undefined)
        return [];
      return issue.claims
        .map(function toSummary(member,) {
          return `- issue (${member.claim
            .category}, ${issue.severity}): ${flattenSpace({ text: member.claim
              .summary, },)}`;
        },);
    },);

  /**
   Document context before the region.
   */
  const before = targetText.slice(
    Math.max(
      0,
      envelope.startOffset - REGION_CONTEXT_CHARS,
    ),
    envelope.startOffset,
  );

  /**
   Document context after the region.
   */
  const after = targetText.slice(
    envelope.endOffset,
    envelope.endOffset + REGION_CONTEXT_CHARS,
  );

  /**
   Current-text line; insertions present their emptiness explicitly.
   */
  const currentLine = envelope.baseText === ''
    ? 'CURRENT TEXT: (empty; content is missing here)'
    : `CURRENT TEXT: ${envelope.baseText}`;

  return `REGION ${regionNumber}
${issueLines.join('\n',)}
${currentLine}
CONTEXT: ...${before}«REGION ${regionNumber}»${after}...`;
}

/**
 Builds the editor sheet for one chunk:
 documents fenced, envelopes as numbered regions with their issues,
 current text, and disambiguating context.

 @param sourceText - original chunk text

 @param targetText - translation chunk text the envelopes were cut from

 @param envelopes - non-overlapping envelopes in document order

 @param issues - adjudicated issues referenced by the envelopes

 @param editorRuleAddendum - extra rule line appended to the enforced
 rule list, for prompt calibration experiments

 @param identityContext - declared names and handles, which the critic and
 the panel read before the issues reached this sheet (ledger S14)

 @param referenceContext - what the pages the original cites say, read by
 the same stages (ledger S14)

 @returns Messages plus the envelope numbering order

 @example
 ```ts
 const plan = buildEditorMessages({ sourceText, targetText, envelopes, issues, },);
 ```
 */
export function buildEditorMessages(
  {
    sourceText,
    targetText,
    envelopes,
    issues,
    editorRuleAddendum,
    neighbouringIncumbentText,
    neighbouringSourceText,
    identityContext,
    referenceContext,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
    readonly envelopes: readonly EditableEnvelope[];
    readonly issues: readonly AdjudicatedIssue[];
    readonly editorRuleAddendum?: string;
    readonly neighbouringIncumbentText?: string;
    readonly neighbouringSourceText?: string;
    readonly identityContext?: string;
    readonly referenceContext?: string;
  },
): EditorPromptPlan {
  /**
   Rendered region blocks in envelope order.
   */
  const blocks = envelopes.map(function toBlock(
    envelope,
    index,
  ) {
    return regionBlock({
      envelope,
      regionNumber: index + 1,
      targetText,
      issues,
    },);
  },);

  /**
   Fence no enclosed text can reproduce, chosen against every text the sheet encloses,
   the rendered regions included.
   */
  const fence = selectFence({
    texts: [
      sourceText,
      targetText,
      neighbouringSourceText ?? '',
      neighbouringIncumbentText ?? '',
      identityContext ?? '',
      referenceContext ?? '',
      ...blocks,
    ],
  },);

  /**
   Declared names, the pages the original cites, and the community
   renderings the translation lacks, each absent when the page has none: the
   evidence the critic, the panel and the selection judges read (ledger
   S14).
   */
  const pageEvidence = [
    declaredNamesBlock({
      fence,
      ...((identityContext === undefined) ? {} : { identityContext, }),
    },),
    citedReferenceBlockText({
      fence,
      ...((referenceContext === undefined) ? {} : { referenceContext, }),
    },),
    communityRenderingsBlock({
      sourceText,
      candidates: [{
        label: 'TRANSLATION',
        text: targetText,
      },],
    },)
      .join('\n',),
  ]
    .filter(function isPresent(part,): boolean {
      return part !== '';
    },)
    .join('',);

  /**
   The passages either side, or nothing when this slice stands alone.
   */
  const nearbyBlock = ((neighbouringSourceText === undefined)
    || (neighbouringSourceText === ''))
    && ((neighbouringIncumbentText === undefined)
      || (neighbouringIncumbentText === ''))
    ? ''
    : `${fence} NEARBY ORIGINAL, CONTEXT ONLY ${fence}
${neighbouringSourceText ?? ''}
${fence} NEARBY EXISTING TRANSLATION, CONTEXT ONLY ${fence}
${neighbouringIncumbentText ?? ''}
${fence} ${NEARBY_RULE} ${fence}
`;

  /**
   System prompt with the calibration addendum composed in as one more
   machine-enforced rule, before the reply-shape block.
   */
  const baseSystemPrompt = editorRuleAddendum === undefined
    ? EDITOR_SYSTEM_PROMPT
    : `${EDITOR_RULES_HEAD}
- ${editorRuleAddendum}

${HOUSE_POLICY_BLOCK}

${EDITOR_REPLY_BLOCK}`;

  /**
   The sheet's rules, with the rules for reading the declared names where
   the page declares any.
   */
  const systemPrompt = ((identityContext === undefined) || (identityContext === ''))
    ? baseSystemPrompt
    : `${baseSystemPrompt}\n\n${DECLARED_IDENTITY_RULES}\n- Never change a rendering the block makes correct.`;

  return {
    messages: [
      {
        role: 'system',
        content: systemPrompt,
      },
      {
        role: 'user',
        content: `${pageEvidence}${fence} ORIGINAL ${fence}
${sourceText}
${fence} TRANSLATION ${fence}
${targetText}
${nearbyBlock}${fence} EDIT REGIONS ${fence}
${blocks.join('\n\n',)}
${fence} END ${fence}`,
      },
    ],
    envelopes,
  };
}

//endregion Editor prompt
