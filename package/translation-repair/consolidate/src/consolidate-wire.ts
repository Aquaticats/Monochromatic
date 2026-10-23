import type { ChatMessage, } from '@monochromatic-dev/module-llm-type/ts';

import { renderConsolidationBrief, } from './consolidate-brief.ts';
import { QUOTED_WORDS_REMINDER, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type DeclaredNamePair,
  HOUSE_POLICY_BLOCK,
  NAME_FORM_SCOPE_RULE,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { LaneContestBallot, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import { selectFence, } from '@monochromatic-dev/translation-repair-stage/ts';
import {
  type BreakRendering,
  renderedBreakContract,
  renderedBreakFacts,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { PAGE_APPARATUS_IS_KEPT, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type DisputedWording,
  TRANSLATE_FRONT_MATTER_RULE,
  TRANSLATE_LINE_STRUCTURE_RULE,
  TRANSLATE_PICTURE_SCOPE_RULE,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  CITED_REFERENCE_CANDIDATE_RULE,
  type SliceSyntax,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';

//region Consolidate wire
// Asks for the rendering that ships, given the two the lanes produced and what
// the contest's judges said about them.
//
// WHY A THIRD RENDERING RATHER THAN A CHOICE. The lane contest answers which
// candidate the original supports BETTER, which is not the same as either being
// good. `doc/audit/eight-entries-read-against-the-original.md` found the lanes
// failing in opposite directions, the repair lane inheriting the archive's
// inventions and the translate lane discarding what the archive knew, and found
// at least one slice where each lane was better than the other in a DIFFERENT
// PLACE of the same passage. No selection can produce that slice's best text,
// because that text is neither candidate.
//
// So this call may take one clause from one candidate and the next from the
// other, and the sheet says so: a producer told only to pick a side will pick
// one.
//
// THE REPLY SHAPE IS THE TRANSLATE LANE'S OWN, `{"translation": "..."}`, so
// `isTranslateReportWire`, `repairInvalidCandidates`, `buildTranslateCandidates`
// and `judgeTranslateSlate` all serve this stage unchanged. The sheet is the
// only new thing in the producing half.
//
// THE FINDINGS ARE SHOWN AS CLAIMS. See `consolidate-brief.ts` for why that
// framing is load-bearing rather than polite.
//
// THE PAGE'S SHAPE IS THE STRUCTURAL STANDARD, NOT THE ORIGINAL'S, and this was
// measured rather than assumed. On the first calibration slice the Chinese is
// one paragraph while the archive renders it as a block quote followed by an
// attribution line, so all six consolidations were called invalid against the
// original, five were sent back, and the rendering that then won had flattened
// the quote and its attribution into one plain paragraph. That is damage this
// stage would have authored. What a consolidation replaces is the text on the
// page, so the page is what it has to match.

/**
 Instructions every consolidating producer shares.

 OPENS BY SAYING NEITHER CANDIDATE IS KNOWN TO BE GOOD. A producer shown two
 renderings and asked to improve on them treats their agreement as settled,
 which is exactly the inherited-invention case: where both lanes carry the
 same archive invention, their agreement is the defect.
 */
const CONSOLIDATE_RULES =
  `You are a bilingual Chinese-to-English translator finishing one passage of a memorial archive for publication.

Two English renderings of this passage already exist. Neither is known to be good, and where they agree they may both be wrong. Write the rendering that ships.

THE ORIGINAL IS THE STANDARD. Judge everything against the Chinese, never against the archive rendering and never against either candidate.

Rules:
- Say everything the ORIGINAL says: every clause, qualifier, named object and speaker aside.
- Say nothing the ORIGINAL does not say. Do not strengthen a claim, invent a time period, or characterize anyone the original leaves uncharacterized.
- ${PAGE_APPARATUS_IS_KEPT} Here the EXISTING TRANSLATION is the ARCHIVE RENDERING.
- DECLARED NAMES from the documents' front matter settle HOW to spell a name this passage already refers to, and they OUTRANK THE ARCHIVE RENDERING where the two spell the same person or place differently. A candidate using a declared form has added nothing, and copying the archive's spelling is not a defence where the front matter declares another. They are not content this passage owes: a passage that does not refer to this person does not gain their name, and a line attributing the passage to someone ELSE never takes it. A line beginning "target contributor:" instead records public identity from existing English attribution. Preserve that spelling exactly on contributor lines; a source-script name or different handle does not override it. ${NAME_FORM_SCOPE_RULE}
- Where a candidate already renders a clause well, KEEP ITS WORDING. Reaching the same English by different words is not an improvement, and a reader who knows this archive should not see it churn.
- You may take one clause from one candidate and the next from the other. Neither candidate has to be right about the whole passage, and that is the point of this call.
- Write English, not a repair of English. The result must read as prose a person wrote, carrying the original's voice: its warmth, grief, humour or anger. A stiff literal rendering that loses the feeling is not a good rendering.
- KEEP THE SHAPE OF THE PAGE. Every block of the ARCHIVE RENDERING appears in your rendering, of the same kind and in the same order, carrying the same block quotes, list markers, headings, footnote markers and links. Where the Chinese is shaped differently, the page's shape wins: a passage the archive sets as a block quote followed by an attribution line stays a block quote followed by an attribution line. ADD A BLOCK ONLY to carry something the ORIGINAL has and the archive left out.
- ONE TENSE, IN THIS ORDER. The Chinese marks no tense, so the English decides: first the past tense the house rule sets for the life of a person who has died (a line of the life the ARCHIVE RENDERING tells in the present is moved to the past, and that move is a correction the house rule asks for, not a change of your own); then the tense the page holds for what is still true as it is read. Moving a passage the archive tells in the past into the present is a change you made. Hold one tense throughout, and never let one sentence start in one tense and finish in another. ${QUOTED_WORDS_REMINDER}

THE JUDGE FINDINGS ARE CLAIMS, NOT FACTS. Other models wrote them about these two candidates, and they can be wrong. Check each one against the ORIGINAL before you act on it, and ignore any the ORIGINAL does not support. A finding you obey that the original does not support is a defect you introduced yourself.

${HOUSE_POLICY_BLOCK}`;

/**
 Reply-format instruction, kept LAST in the assembled sheet.

 Split out for the same reason the translate sheet splits it: a conditional
 rule has to sit above it, and wire instructions that end up above content
 rules are the ones models drop first.
 */
const CONSOLIDATE_REPLY_RULE =
  'Reply with ONLY a JSON object of shape {"translation": "..."}. No prose, no code fences, no commentary.';

/**
 What a consolidating producer is shown for one slice.
 */
export type ConsolidateSubject = {
  /**
   Original passage, which is the standard.
   */
  readonly sourceText: string;

  /**
   Syntax role requiring dedicated production and validation rules.
   */
  readonly syntax?: SliceSyntax;

  /**
   Archive rendering, as evidence rather than as the standard.
   */
  readonly incumbentText: string;

  /**
   What the repair lane would ship.
   */
  readonly repairText: string;

  /**
   What the translate lane would ship.
   */
  readonly translateText: string;

  /**
   Lane contest ballots for this slice, shown as claims to check.

   MAY BE EMPTY, and the sheet then carries no findings block at all. A slice
   whose contest never reached quorum still has two candidates worth
   consolidating, and an empty findings heading would read as judges having
   looked and found nothing.
   */
  readonly ballots: readonly LaneContestBallot[];

  /**
   Why the archive rendering shown is the repair lane's stand-in, absent on
   an undisputed slice (class one hundred eight, 2026-09-24).
   */
  readonly archiveDisputeNote?: string;

  /**
   Names and handles both documents' front matter declares, when either does.
   */
  readonly identityContext?: string;

  /**
   What the pages the original cites say, one line per page, when the
   original links anywhere (class thirty-six, 2026-09-16): the writer that
   kept "Accurate detail the ARCHIVE RENDERING adds" as a rule had no way to
   tell such detail from an invention and wrote from the lane that dropped it.
   */
  readonly referenceContext?: string;

  /**
   What the pictures this slice and its neighbours show were read to say, when
   any could be.

   OPTIONAL, AND HONESTLY SO, unlike `lineStructured` below it. A slice sitting
   near no picture, or near none any reader could make out, is a slice with
   nothing to say here, so absence is a real state rather than a caller's
   omission.

   WHICH IS EXACTLY WHY IT WENT UNWRITTEN FOR SO LONG. From this field's birth
   until 2026-08-22 every consolidating producer read the absence, because no
   caller wrote it and an optional field cannot say whether the caller had
   nothing or forgot. `slicePictureContexts` is the writer, and the window it
   hands over is the one the translate lane already saw.
   */
  readonly pictureContext?: string;

  /**
   Whether the enclosing CHUNK's original is line-structured, decided by the
   caller because a slice is too small a unit to decide it on.

   REQUIRED, UNLIKE THE TWO CONTEXTS ABOVE IT, and the difference is the
   whole reason this reads as it does. A slice genuinely may declare no
   names and hold no pictures, so a reader defaulting those is reading a
   real absence. Every slice is either line-structured or not, so an
   optional spelling here could only ever mean a caller forgot.

   ONE DID. The field and its reader were born together on 2026-08-21 and
   no writer ever arrived, so for a day every consolidating producer was
   told its passage was prose, verse included. An optional field cannot owe
   a writer; a required one can, and the type checker collects.
   */
  readonly lineStructured: boolean;

  /**
   Name pairs the front matter declares, which the publication rule reads for
   a linked title naming a declared person (class one hundred fourteen).
   */
  readonly declared?: readonly DeclaredNamePair[];

  /**
   Wordings a disputed slice refuses as proposal, absent on an undisputed
   slice (owner, 2026-09-27, "No eligible standing").
   */
  readonly disputedWordings?: readonly DisputedWording[];
};

/**
 Renders one labelled block, or nothing when its text is empty.

 @param fence - fence enclosing every block in this sheet

 @param label - heading naming what this block is

 @param text - block contents

 @returns Lines for this block, empty when there is nothing to show

 @example
 ```ts
 const lines = renderBlock({ fence: '=====', label: 'DECLARED NAMES', text, },);
 ```
 */
function renderBlock(
  {
    fence,
    label,
    text,
  }: {
    readonly fence: string;
    readonly label: string;
    readonly text: string;
  },
): readonly string[] {
  if (text === '')
    return [];
  return [
    `${fence} ${label} ${fence}`,
    text,
    '',
  ];
}

/**
 Label for one candidate block, saying so where the candidate IS the archive.

 WHY IT HAS TO SAY SO. A lane declining to change the incumbent is a
 legitimate outcome meaning the archive was already right, and it happened at
 5 of the 13 bed slices. Unlabelled, the sheet shows one text twice under two
 headings, and a producer weighing the agreement of the archive with a
 candidate is counting one source twice.

 @param name - candidate's name in this contest

 @param text - what that lane returned

 @param archiveText - rendering already on the page

 @returns Label, carrying the identity where there is one

 @example
 ```ts
 const label = candidateLabel({ name: 'repair', text, archiveText, },);
 ```
 */
function candidateLabel(
  {
    name,
    text,
    archiveText,
  }: {
    readonly name: string;
    readonly text: string;
    readonly archiveText: string;
  },
): string {
  return (text === archiveText)
    ? `CANDIDATE "${name}", which is the ARCHIVE RENDERING unchanged: this lane found nothing to change`
    : `CANDIDATE "${name}"`;
}

/**
 Builds the sheet asking one producer to consolidate one slice.

 @param subject - passage, archive rendering, both candidates and the ballots

 @returns Messages for one exchange

 @example
 ```ts
 const messages = buildConsolidateMessages({ subject, },);
 ```
 */
export function buildConsolidateMessages(
  { subject, }: { readonly subject: ConsolidateSubject; },
): readonly ChatMessage[] {
  /**
   Declared names as one block, empty when neither side declares any.
   */
  const declared = subject.identityContext ?? '';

  /**
   What the pictures said, empty when none were read.
   */
  const pictures = subject.pictureContext ?? '';

  /**
   The pages the original cites with their rule, empty when it cites none.
   */
  const references = ((subject.referenceContext === undefined) || (subject.referenceContext === ''))
    ? ''
    : `${subject.referenceContext}\n${CITED_REFERENCE_CANDIDATE_RULE}`;

  /**
   The repair lane's candidate under the label its block carries.
   */
  const repairCandidate: BreakRendering = {
    label: candidateLabel({
      name: 'repair',
      text: subject.repairText,
      archiveText: subject.incumbentText,
    },),
    text: subject.repairText,
  };

  /**
   The translate lane's candidate under the label its block carries.
   */
  const translateCandidate: BreakRendering = {
    label: candidateLabel({
      name: 'translate',
      text: subject.translateText,
      archiveText: subject.incumbentText,
    },),
    text: subject.translateText,
  };

  /**
   Candidates the sheet shows a block for, which are the ones whose break
   counts it states (ledger B29: the gates after this writer carried their
   candidates' counts while this sheet carried none).
   */
  const displayedCandidates = [
    repairCandidate,
    translateCandidate,
  ].filter(function isDisplayed(candidate,): boolean {
    return candidate.text !== '';
  },);

  /**
   Judge findings as the producer will see them, empty when none were heard.
   */
  const brief = renderConsolidationBrief({ ballots: subject.ballots, },);

  /**
   Fence no enclosed text can reproduce, chosen against every string this
   sheet carries, since all of them are arbitrary prose.
   */
  const fence = selectFence({
    texts: [
      subject.sourceText,
      subject.incumbentText,
      subject.repairText,
      subject.translateText,
      declared,
      pictures,
      references,
      brief,
      subject.archiveDisputeNote ?? '',
    ],
  },);

  /**
   Producer sheet, with the line-structure fact inserted above the reply
   instruction when the enclosing chunk's original is verse.

   BORROWED FROM THE TRANSLATE WIRE rather than restated, because this
   producer is a translator and a second wording of the same rule would drift
   from the one `Toka_ls` was measured against.
   */
  const system = [
    CONSOLIDATE_RULES,
    (subject.syntax === 'front-matter' ? TRANSLATE_FRONT_MATTER_RULE : ''),
    (subject.lineStructured ? TRANSLATE_LINE_STRUCTURE_RULE : ''),
    (((subject.pictureContext ?? '') === '') ? '' : TRANSLATE_PICTURE_SCOPE_RULE),
    // THE INSTRUCTION ONLY: the measured counts ride in the user turn (R4).
    renderedBreakContract({
      sourceText: subject.sourceText,
      archiveText: subject.incumbentText,
      ...((subject.syntax === undefined) ? {} : { syntax: subject.syntax, }),
    },),
    CONSOLIDATE_REPLY_RULE,
  ]
    .filter(function isPresent(part,): boolean {
      return part !== '';
    },)
    .join('\n\n',);
  return [
    {
      role: 'system',
      content: system,
    },
    {
      role: 'user',
      content: [
        ...renderBlock({
          fence,
          label: 'DECLARED NAMES',
          text: declared,
        },),
        ...renderBlock({
          fence,
          label: 'ORIGINAL (Chinese), the standard',
          text: subject.sourceText,
        },),
        ...renderBlock({
          fence,
          label: 'WHAT THE PICTURES HERE SAY',
          text: pictures,
        },),
        ...renderBlock({
          fence,
          // NAMED AS THE EXISTING TRANSLATION TOO, because the verse rule
          // borrowed from the translate wire asks the producer to unmerge lines
          // the EXISTING TRANSLATION merged, and nothing else in this sheet
          // carries that label.
          label: 'ARCHIVE RENDERING, the EXISTING TRANSLATION, evidence only',
          text: subject.incumbentText,
        },),
        // Why that rendering is the repair lane's stand-in, right under it, so
        // the accepted additions it answers for are not carried as page
        // content (class one hundred eight).
        ...renderBlock({
          fence,
          label: 'ARCHIVE RENDERING DISPUTED',
          text: subject.archiveDisputeNote ?? '',
        },),
        ...renderBlock({
          fence,
          ...repairCandidate,
        },),
        ...renderBlock({
          fence,
          ...translateCandidate,
        },),
        ...renderBlock({
          fence,
          label: 'CITED REFERENCES, EVIDENCE ONLY',
          text: references,
        },),
        ...renderBlock({
          fence,
          label: 'WHAT THE JUDGES FOUND, claims to check against the original',
          text: brief,
        },),
        ...renderBlock({
          fence,
          label: 'RENDERED LINE STRUCTURE, measured facts',
          text: renderedBreakFacts({
            sourceText: subject.sourceText,
            archiveText: subject.incumbentText,
            ...((subject.syntax === undefined) ? {} : { syntax: subject.syntax, }),
            renderings: displayedCandidates,
          },),
        },),
        `${fence} END ${fence}`,
      ].join('\n',),
    },
  ];
}

//endregion Consolidate wire
