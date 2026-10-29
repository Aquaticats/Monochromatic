import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

import {
  APPARATUS_KINDS,
  ISSUE_CATEGORIES,
  ISSUE_SEVERITIES,
  NARRATIVE_DETAIL_IS_NOT_APPARATUS,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { HOUSE_POLICY_BLOCK, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  DECLARED_IDENTITY_RULES,
  declaredNamesBlock,
} from './declared-identity-rule.ts';
import { citedReferenceBlockText, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  ACCURACY_CATEGORY_SCOPE,
  FOREIGN_PHRASE_NAME_TITLE_SCOPE,
  OBLIGATORY_DIFFERENCES,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import { withoutTranslatorsNotes, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { selectFence, } from '@monochromatic-dev/translation-repair-stage/ts';
import { REPAIR_EVIDENCE_ROLE, } from './repair-evidence-role.ts';
import {
  CRITIC_NOTHING_WRONG_RULE,
  SEVERITY_SCALE,
} from './severity-scale.ts';

//region Critic prompt
// One strict prompt for every critic model: exact-quote evidence rules, the closed
// category vocabulary, JSON-only output. Document text is embedded between fence
// lines; the fence is decorative for the model, not a security boundary, and the
// deterministic quote resolver is what actually gates fabricated evidence.


/**
 What the neighbouring blocks are for, stated inside the prompt.

 SAID IN THE PROMPT RATHER THAN ASSUMED, because the failure it prevents is
 the one the window itself creates: a critic handed more text can start
 reporting the neighbours as untranslated or as surplus, which would turn one
 relocation into claims against passages that are perfectly correct where they
 are. The blocks exist to be RECOGNISED IN, never judged.

 THE QUOTE SENTENCE WAS ADDED AFTER MEASURING, and it is the half that was
 missing. Forbidding claims ABOUT the neighbours does not forbid QUOTING them
 as evidence for a claim about this slice, and a quote from next door cannot
 anchor here, so the claim is discarded whole. Measured on `saurikissa`
 settled with the window against the same entry without it, at an identical
 roster of 11 critic stages and 65 voices heard:

 ```text
                        without window   with window
 quote-not-found                     9            16
 findings in total                 112            98
 issues with no region              16            37
 issues shipped                     26             7
 issues resolved                    23             6
 ```

 A claim that cannot anchor costs everything downstream of it, so nearly
 doubling the unanchorable share is what took the lane from nine shipped
 slices to three.
 */
const NEARBY_RULE = `${REPAIR_EVIDENCE_ROLE} Every quoted anchor must still be copied exactly from the current ORIGINAL or TRANSLATION. When the source basis exists only nearby, omit sourceQuote, describe the relevant contextual fact in the summary, and anchor the actual current defect with targetQuote; never place a nearby quote in a local anchor field.`;

/**
 System instructions shared by every critic call.
 */
const CRITIC_SYSTEM_PROMPT = `You are a strict bilingual translation reviewer.
Compare the ORIGINAL document with its TRANSLATION and report every defect you find in the translation.

${ACCURACY_CATEGORY_SCOPE}

Report each defect as one atomic issue:
- category: one of ${ISSUE_CATEGORIES.join(', ',)}
- severity: one of ${ISSUE_SEVERITIES.join(', ',)}, as the SEVERITY SCALE below defines
- sourceQuote: exact substring copied character-for-character from the ORIGINAL that evidences the defect (omit if none applies)
- summary: one sentence stating the single defect
- targetQuote: exact substring copied character-for-character from the TRANSLATION where the defect manifests (omit only if the defect has no target-side anchor)

Quote rules, strictly enforced by a machine:
- Quotes must be copied exactly, byte for byte. Paraphrases are discarded.
- Each quote must be long enough to occur exactly once in its document.
- A quote must stay inside one paragraph or block; never span a blank line.
- For omitted content: sourceQuote is the untranslated original text, targetQuote is the translated sentence adjacent to where the content should have appeared.

${SEVERITY_SCALE}
${CRITIC_NOTHING_WRONG_RULE}

${HOUSE_POLICY_BLOCK}

Translation policy, applied when deciding what counts as a defect:
- Judge emotional completeness and naturalness, not word-for-word correspondence. A rendering whose wording, sentence boundaries, or clause order differ from the ORIGINAL is correct when it reads naturally and carries the same feeling. Never report a defect merely because a rendering is not literal.
- Report lost feeling as a defect: when the TRANSLATION carries the facts but flattens the ORIGINAL's voice, warmth, humour, irony, grief, or intimacy, report style/emotional-flattening.
- Report stiff literal renderings no fluent writer would produce as style/awkward-phrasing, even when every word matches.
- When the ORIGINAL quotes a phrase in a language other than its own (an Esperanto, Japanese, or Latin line inside a Chinese page), the TRANSLATION must keep that phrase in its original wording AND give its meaning alongside, so the reader gets both. Report policy/foreign-phrase-gloss when the TRANSLATION drops the original wording, or reproduces it with no meaning alongside. This does not apply when the quoted phrase is already in the TRANSLATION's own language, where the wording alone suffices. ${FOREIGN_PHRASE_NAME_TITLE_SCOPE}
- The ORIGINAL's own language is never such a phrase. Text written in the ORIGINAL's language belongs in the TRANSLATION fully rendered into the TRANSLATION's language, including inside quotations and stylized multilingual lines; report accuracy/untranslated when it survives unrendered.

Judge every span in the context of the whole excerpt you were given, never in isolation. A word choice is often licensed by a neighbouring clause: a contrast set up in the preceding half-sentence, an image established a sentence earlier, a fact stated once and relied on afterwards. Before reporting a word as wrong, look for what licenses it nearby, and drop the issue if you find it.

These documents memorialize real people, written by their communities, and they carry in-group vocabulary: nicknames, community slang, and shorthand whose conventional meaning is not its literal reading. When the TRANSLATION renders such a term by its conventional meaning, that is correct even though a literal reading of the ORIGINAL's characters says something else. Never report a rendering as fabricated, invented, or unsubstantiated on the strength of a literal reading alone. If you do not recognize a term, assume the translator knew the community's usage and say nothing.

The ORIGINAL is not golden. It is ordinary writing, sometimes hurried or informal, and a TRANSLATION that repairs a deficiency in it is doing its job. Never report a defect merely because the TRANSLATION is clearer, better punctuated, or more explicit than the ORIGINAL.

${OBLIGATORY_DIFFERENCES}
- Report a defect for such a difference only when the supplied or rendered choice is the WRONG one, and then say which reading the ORIGINAL supports.
- ACCURATE page apparatus a translator added is not an addition defect: ${APPARATUS_KINDS}. Each of these is correct information a reader benefits from, and reporting it as unsupported content leads to it being deleted. Report such detail ONLY when it is WRONG, and then say what is wrong with it. ${NARRATIVE_DETAIL_IS_NOT_APPARATUS}

${DECLARED_IDENTITY_RULES}
- Never report a rendering the block makes correct as a defect.

If the TRANSLATION is not a translation of the ORIGINAL at all (unrelated content, gibberish, a different document), report exactly one issue: category accuracy/non-translation, severity critical, targetQuote copied from the start of the TRANSLATION body. Do not enumerate further issues for such a pair.
If only one section is unrelated while the rest translates the original, report accuracy/non-translation for that section alone, anchored by its quotes, alongside any other issues.

Reply with ONLY a JSON object of shape {"issues": [...]}. No prose, no code fences.
An empty issues array is a valid answer when the translation is faithful.`;

/**
 The texts whose comments are the original's own on a critic's sheet: the
 original, and the original next door.

 THE ONE DEFINITION BOTH SIDES OF THE SHEET READ. The sheet cuts the notes
 between translators from the translation against these texts
 (`buildCriticMessages`), and the quotes the critic copies out of it are
 resolved against the same ones (`resolveCriticIssue`); two spellings of the
 list could disagree about which comment is the original's and read two
 different texts.

 @param sourceText - original document

 @param neighbouringSourceText - original next door, absent when the slice
 stands alone

 @returns Texts joined by line feeds, for `withoutTranslatorsNotes`

 @example
 ```ts
 const original = criticOriginalsOf({ sourceText, },);
 ```
 */
export function criticOriginalsOf(
  {
    sourceText,
    neighbouringSourceText,
  }: {
    readonly sourceText: string;
    readonly neighbouringSourceText?: string;
  },
): string {
  return [
    sourceText,
    neighbouringSourceText ?? '',
  ].join('\n',);
}

/**
 Builds the message list for one critic call.

 @param sourceText - original document, front matter included

 @param targetText - translation under review, front matter included

 @param identityContext - declared names and handles from both sides' front
 matter; omitted when neither side declares any, so the block never appears
 empty

 @param referenceContext - what the pages the original links say (class
 thirty-five); omitted when it links nowhere

 @returns Messages ready for `chatJson`

 @example
 ```ts
 const messages = buildCriticMessages({ sourceText, targetText, },);
 ```
 */
export function buildCriticMessages(
  {
    sourceText,
    targetText,
    identityContext,
    referenceContext,
    neighbouringIncumbentText,
    neighbouringSourceText,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly neighbouringIncumbentText?: string;
    readonly neighbouringSourceText?: string;
  },
): readonly ChatMessage[] {
  // THE CRITIC IS SHOWN THE TRANSLATION WITHOUT THE NOTES THE ARCHIVE'S
  // TRANSLATORS LEFT EACH OTHER, as every other judging sheet is
  // (`withoutTranslatorsNotes`): no reader sees one, the page keeps each one
  // whatever a judge says, and a critic told to report the original's language
  // left in the translation (`accuracy/untranslated`) would report a Chinese
  // comment, which an accepted issue turns into an envelope for an editor to
  // render, and the carry then puts the note back beside the rendering. Only
  // what is SHOWN is cut. A quote the critic gives is located in the text as
  // shown and carried back to the stored one (`resolveCriticIssue` with the
  // same originals, `criticOriginalsOf`), so a quote across a place a note was
  // cut from anchors in the stretches of words either side of the note and no
  // anchor holds it. The original's own comments stay in view, and so do the
  // notes of the original next door.
  /**
   Texts whose comments are the original's own.
   */
  const originals = criticOriginalsOf({
    sourceText,
    ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
  },);
  /**
   The translation as the critic reads it.
   */
  const targetShown = withoutTranslatorsNotes({
    text: targetText,
    original: originals,
  },);
  /**
   The existing translation next door as the critic reads it.
   */
  const incumbentShown = withoutTranslatorsNotes({
    text: neighbouringIncumbentText ?? '',
    original: originals,
  },);

  /**
   Fence no enclosed text can reproduce, chosen against every text the sheet
   encloses, each text as written and as shown.
   */
  const fence = selectFence({
    texts: [
      sourceText,
      targetText,
      targetShown,
      identityContext ?? '',
      referenceContext ?? '',
      neighbouringSourceText ?? '',
      neighbouringIncumbentText ?? '',
      incumbentShown,
    ],
  },);

  /**
   What the original's cited pages say, after the pair and before the
   neighbours: evidence about the pair, not part of it (class thirty-five).
   */
  const referenceBlock = citedReferenceBlockText({
    fence,
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
  },);

  /**
   Identity block plus its fence, or nothing at all when undeclared.
   Placed BEFORE the documents so the declarations are read as given facts
   rather than as a footnote to evidence already weighed.
   */
  const identityBlock = declaredNamesBlock({
    fence,
    ...((identityContext === undefined) ? {} : { identityContext, }),
  },);

  /**
   The passages either side, or nothing when this slice stands alone.

   PLACED AFTER THE PAIR BEING JUDGED, unlike the identity block, and the
   order carries meaning. Identity is a given fact and is read first; this is
   evidence ABOUT the pair, and putting it first would invite a critic to
   treat the neighbours as part of what it was asked to judge.

   WHY A CRITIC NEEDS IT AT ALL: where the archive carried a
   passage across a section boundary, the translation here holds English with
   no original to support it, and the original next door holds Chinese with no
   English. Shown this slice alone, the only available readings are invention
   and omission, both of which are wrong and both of which lead the editor to
   damage text. Shown the neighbours, the passage is locatable.

   BOTH SIDES OR THE WINDOW IS HALF BLIND. The neighbouring original says what
   the passage next door is ABOUT; the neighbouring archive says where the
   English actually went. A relocation is only visible in the second.
   */
  const nearbyBlock = ((neighbouringSourceText === undefined)
    || (neighbouringSourceText === ''))
    && (incumbentShown === '')
    ? ''
    : `${fence} NEARBY ORIGINAL, CONTEXT ONLY ${fence}
${neighbouringSourceText ?? ''}
${fence} NEARBY EXISTING TRANSLATION, CONTEXT ONLY ${fence}
${incumbentShown}
${fence} ${NEARBY_RULE} ${fence}
`;

  return [
    {
      role: 'system',
      content: CRITIC_SYSTEM_PROMPT,
    },
    {
      role: 'user',
      content: `${identityBlock}${fence} ORIGINAL ${fence}
${sourceText}
${fence} TRANSLATION ${fence}
${targetShown}
${referenceBlock}${nearbyBlock}${fence} END ${fence}`,
    },
  ];
}

//endregion Critic prompt
