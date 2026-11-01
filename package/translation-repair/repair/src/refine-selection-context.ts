import { citedReferenceEvidence, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  declaredNamesEvidence,
  type SelectEvidence,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import { HOUSE_FORM_CORRECTION_KEEPS_MEANING, } from '@monochromatic-dev/translation-repair-house-rule/ts';

//region Refinement selection context
// Keeps exploratory refinement and objection correction from asking selectors
// contradictory questions about what they are choosing for.
//
// OBJECTION CORRECTION (owner, 2026-09-27, the fourteenth and fifteenth
// addenda of `doc/decision/translation-repair-ineligible-standing.md`). Over a
// standing the deterministic rule refused, the consolidation ships whatever
// the gate or the slate judges held against it, so their objections become
// corrections to make where the ORIGINAL supports them. The text still ships
// unchanged when no candidate earns its place: an objection is a claim, and
// the base remains the fallback.
//
// NO REQUIRED NATURALNESS CORRECTION (ledger B47): a third mode corrected text
// the absolute naturalness review had rejected, with no fallback. Its loop
// went on 2026-09-01; the mode, its sheets and its "no correction" outcome
// stayed, built by tests alone, until B47 took them out.
//
// THE NOTES THE WRITERS READ (ledger B805). The final polish hands its writers
// the claims the repair panel accepted and the note on a disputed archive
// rendering, as it hands both to its gate. The judges who choose among the
// writers' rewrites read neither, so a rewrite made to a claim was judged by
// readers who could not see why it differed from the CURRENT text, under a
// criterion that a candidate says exactly what the CURRENT text says. A judge
// of a text sees what its writer saw: both notes reach the evidence, and where
// either is shown the criteria let a candidate correct what a supported claim
// names.

/**
 Judges whose objections an objection correction answers.
 */
export type ObjectionOrigin = 'consolidation gate' | 'consolidation slate';

/**
 What one set of judges objected to.

 @example
 ```ts
 const group: ObjectionGroup = { origin: 'consolidation gate', objections: ['adds a purr'], };
 ```
 */
export type ObjectionGroup = {
  /**
   Judges the objections come from.
   */
  readonly origin: ObjectionOrigin;

  /**
   Each objecting judge's reason as written, a claim to check against the
   ORIGINAL rather than a fact.
   */
  readonly objections: readonly string[];
};

/**
 Names every set of objecting judges, for a sentence.

 @param groups - objections by the judges they come from

 @returns The judges joined with "and"

 @example
 ```ts
 objectingJudgesOf({ groups, },); // => 'consolidation slate and consolidation gate'
 ```
 */
export function objectingJudgesOf(
  { groups, }: { readonly groups: readonly ObjectionGroup[]; },
): string {
  return groups
    .map(function originOf(group,): string {
      return group.origin;
    },)
    .join(' and ',);
}

/**
 Why refinement is running and whether unchanged text remains admissible.

 @example
 ```ts
 const mode: RefineStageMode = { kind: 'comparative', };
 ```
 */
export type RefineStageMode =
  | {
    /**
     Exploratory improvement where accepted input remains fallback.
     */
    readonly kind: 'comparative';
  }
    | {
    /**
     Correction of what judges objected to, the input remaining the fallback.
     */
      readonly kind: 'objection-correction';

    /**
     Objections by the judges they come from, each set under its own
     heading: the slate's reasons where it was declined twice and the gate's
     where it objected (fourteenth and fifteenth addenda).
     */
      readonly groups: readonly ObjectionGroup[];
    };

/**
 Heading every sheet gives an objection correction's objections.

 @param origin - judges the objections come from

 @returns Heading naming the judges and what the objections are

 @example
 ```ts
 objectionsHeading({ origin: 'consolidation gate', },);
 // => 'OBJECTIONS FROM THE CONSOLIDATION GATE, claims to check against the ORIGINAL'
 ```
 */
export function objectionsHeading(
  { origin, }: { readonly origin: ObjectionOrigin; },
): string {
  return `OBJECTIONS FROM THE ${origin.toUpperCase()}, claims to check against the ORIGINAL`;
}

/**
 Heading the polish gate and the polish writers give the note on a disputed
 archive rendering, which the judges' evidence gives it too.
 */
const DISPUTE_HEADING = 'ARCHIVE RENDERING DISPUTED';

/**
 Heading the polish gate and the polish writers give the claims the repair
 panel accepted, which the judges' evidence gives them too.
 */
const CLAIMS_HEADING = 'REPAIR PANEL CLAIMS ACCEPTED';

/**
 What the task adds where a note is shown, so a candidate that corrects what a
 supported claim names is not read as a departure from the CURRENT text.
 */
const CLAIM_CORRECTION_TASK = ' Where the evidence carries accepted claims against the CURRENT text, a candidate may '
  + 'also correct what a claim the ORIGINAL supports names as wrong.';

/**
 Evidence block for one note, nothing where the slice has none.

 @param label - heading the note carries on the other sheets

 @param note - the note as the archive package writes it, absent or empty for
 a slice that has none

 @returns One block, or none

 @example
 ```ts
 const blocks = noteEvidence({ label: 'REPAIR PANEL CLAIMS ACCEPTED', note, },);
 ```
 */
function noteEvidence(
  {
    label,
    note,
  }: {
    readonly label: string;
    readonly note?: string;
  },
): readonly SelectEvidence[] {
  return ((note === undefined) || (note === ''))
    ? []
    : [{
      label,
      text: note,
    },];
}

/**
 Inputs candidate selector receives after refinement generation.

 @example
 ```ts
 const context = buildRefineSelectionContext({ mode, sourceText, repairedText, });
 ```
 */
export type RefineSelectionContext = {
  /**
   One-sentence candidate task.
   */
  readonly task: string;

  /**
   Ordered candidate ranking rules.
   */
  readonly criteria: readonly string[];

  /**
   Fenced source, baseline and review data.
   */
  readonly evidence: readonly SelectEvidence[];

  /**
   What declining every candidate leads to, where the sheet names it.
   */
  readonly declineConsequence?: string;
};

/**
 Builds selector question matching refinement mode.

 @param mode - comparative exploration or objection correction

 @param sourceText - original Chinese fidelity anchor

 @param repairedText - exact current English wording

 @param referenceContext - what the pages the original cites say, with
 their rule, when the original cites any (class forty-one)

 @param identityContext - declared names and handles the refiner was told
 survive exactly, so its judges are told so too (ledger B28)

 @param acceptedRepairClaimsNote - the claims the repair panel accepted for the
 slice, as the block `acceptedRepairClaimsNote` of the archive package writes
 them, which the final polish hands its writers and its gate, so the judges of
 the writers' rewrites read them too (ledger B805); absent or empty for the
 repair lane's own refinement

 @param archiveDisputeNote - the note on a disputed archive rendering, as the
 archive package writes it, shown to the judges for the same reason (ledger
 B805); absent or empty on an undisputed slice

 @returns Candidate-ranking context with review findings fenced as evidence

 @example
 ```ts
 const context = buildRefineSelectionContext({ mode: { kind: 'comparative' }, sourceText, repairedText, });
 ```
 */
export function buildRefineSelectionContext(
  {
    mode,
    sourceText,
    repairedText,
    referenceContext,
    identityContext,
    acceptedRepairClaimsNote,
    archiveDisputeNote,
  }: {
    readonly mode: RefineStageMode;
    readonly sourceText: string;
    readonly repairedText: string;
    readonly referenceContext?: string;
    readonly identityContext?: string;
    readonly acceptedRepairClaimsNote?: string;
    readonly archiveDisputeNote?: string;
  },
): RefineSelectionContext {
  /**
   The note on a disputed archive rendering, none on an undisputed slice.
   */
  const disputeEvidence = noteEvidence({
    label: DISPUTE_HEADING,
    ...((archiveDisputeNote === undefined) ? {} : { note: archiveDisputeNote, }),
  },);
  /**
   The claims the repair panel accepted, none where it accepted nothing.
   */
  const claimsEvidence = noteEvidence({
    label: CLAIMS_HEADING,
    ...((acceptedRepairClaimsNote === undefined) ? {} : { note: acceptedRepairClaimsNote, }),
  },);
  /**
   Whether either note is shown, which changes what a candidate may do.
   */
  const noted = (disputeEvidence.length + claimsEvidence.length) > 0;
  /**
   The declared names and then the references, as evidence the judges read
   beside the texts: the names the refiner was told survive exactly (ledger
   B28), and the pages the original cites (class forty-one), each none when
   the page has none. The dispute note and the accepted claims follow them, in
   the order the polish gate's sheet gives them (ledger B805).
   */
  const pageEvidence = [
    ...declaredNamesEvidence((identityContext === undefined) ? {} : { identityContext, },),
    ...citedReferenceEvidence(
      (referenceContext === undefined) ? {} : { referenceContext, },
    ),
    ...disputeEvidence,
    ...claimsEvidence,
  ];
  if (mode.kind === 'comparative') {
    return {
      task: `Each candidate is a revision of the CURRENT English translation below, meant to read more naturally without changing what it says.${
        noted ? CLAIM_CORRECTION_TASK : ''
      }`,
      criteria: [
        noted
          ? `Says what the CURRENT text says, apart from a correction an accepted claim the ORIGINAL supports asks for: nothing else added, dropped, softened, sharpened, or reattributed. ${HOUSE_FORM_CORRECTION_KEEPS_MEANING}`
          : `Says exactly what the CURRENT text says: nothing added, dropped, softened, sharpened, or reattributed. ${HOUSE_FORM_CORRECTION_KEEPS_MEANING}`,
        'Faithful to the Chinese ORIGINAL.',
        noted
          ? 'Reads more naturally than the CURRENT text by a clear margin, brings it into line with a house rule of form, or corrects what an accepted claim the ORIGINAL supports names as wrong.'
          : 'Reads more naturally than the CURRENT text by a clear margin, or brings it into line with a house rule of form.',
      ],
      evidence: [
        {
          label: 'ORIGINAL (Chinese)',
          text: sourceText,
        },
        {
          label: 'CURRENT English translation, which ships unchanged unless a candidate clearly beats it',
          text: repairedText,
        },
        ...pageEvidence,
      ],
    };
  }
  return {
    task: `Each candidate corrects the CURRENT English translation where the ${
      objectingJudgesOf({ groups: mode.groups, },)
    } objected to it.${noted ? CLAIM_CORRECTION_TASK : ''}`,
    criteria: [
      'Faithful to the Chinese ORIGINAL: nothing it does not say, and nothing it says left out.',
      'Resolves each objection the ORIGINAL supports. An objection is a claim: one the ORIGINAL does not '
        + 'support is ignored, and a candidate acting on it has introduced an error.',
      `${
        noted
          ? 'Changes nothing a supported objection or an accepted claim the ORIGINAL supports does not concern'
          : 'Changes nothing an objection the ORIGINAL supports does not concern'
      }, beyond a clear naturalness fix that keeps the meaning. ${HOUSE_FORM_CORRECTION_KEEPS_MEANING}`,
      'Reads as natural English.',
    ],
    evidence: [
      {
        label: 'ORIGINAL (Chinese)',
        text: sourceText,
      },
      {
        label: 'CURRENT English translation, which ships unchanged unless a candidate resolves an objection '
          + 'the ORIGINAL supports',
        text: repairedText,
      },
      ...pageEvidence,
      ...mode.groups
        .map(function groupEvidence(group,): SelectEvidence {
          return {
            label: objectionsHeading({ origin: group.origin, },),
            text: group.objections
              .map(function listed(objection,): string {
                return `- ${objection}`;
              },)
              .join('\n',),
          };
        },),
    ],
    declineConsequence: 'the CURRENT text ships unchanged, with the objections recorded',
  };
}

//endregion Refinement selection context
