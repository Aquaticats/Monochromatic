import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { DisputedWording, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
// Claim category that disputes the archive rendering at any severity.
import {
  ADDITION_CATEGORY,
  type AdjudicatedIssue,
  APPARATUS_KINDS,
  type IssueSeverity,
  NARRATIVE_DETAIL_IS_NOT_APPARATUS,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Archive dispute
// A slice whose archive rendering the repair lane's adjudicators found to
// ADD something the original never states (class one hundred seven, owner
// answer 2026-09-24: "Not eligible; fall back to the repair text"). On
// CuspariaKLSY10 slice 3 the archive named a suicide method the original
// does not, four `accuracy/addition` claims were accepted, the repair lane
// removed the detail, the translate slate backed nobody, and the archive
// stood because an eligible standing keeps its single round. Here such a
// slice is a dispute: the repair lane's text stands in for the archive as
// the translate lane's incumbent and as the consolidation's standing where
// the contest chose neither lane.
//
// ONLY WHERE THE REPAIR FIXED IT (owner answer 2026-09-27, "No eligible
// standing"). The repair lane's text is the archive's own wording wherever
// its accuracy patch lost at the checkers, or a refinement keeping the
// reading, so the stand-in carried the disputed reading on 86 of 87 measured
// disputed slices. It stands in now only where the checkers confirmed every
// disputing issue resolved and the lane kept the slice; elsewhere the archive
// wording and the repair lane's text are disputed wordings the deterministic
// rule refuses (`disputed-wording.ts`), so neither is a candidate, a fallback,
// a standing or a lane offer anywhere downstream.
//
// CLASS ONE HUNDRED SEVENTY-SIX (TianqiChen66610 slice 13, owner answer
// 2026-09-27: "Major+ accuracy"). The adjudicators accepted major
// mistranslation claims against the archive's gloss of the character a
// performer was remembered as, the gate tied 2 to 2, and the archive shipped
// because only additions disputed it. Any accepted accuracy claim at major
// severity or worse now disputes the archive as well; additions keep
// disputing at any severity.
//
// THE SEVERITY IS THE PANEL'S (owner answer 2026-09-27: "Adjudicated"). The
// rule first read each claim's severity as its critic filed it: on XingZ6014
// a claim filed major disputed slice 66 though the panel settled the issue
// minor, and slice 19 went undisputed though the panel settled its issue
// major. The issue's settled severity is what the rule and the note read.
//
// NO ACCEPTED ISSUE SETTLES AT NEUTRAL (ledger L5, 2026-09-28). The tally
// holds an acceptance settled at neutral, the severity asserting no defect,
// for a human, so "addition at any severity" reaches minor and worse in
// effect: an addition claim no supporter finds a real defect in disputes
// nothing. Before, five accepted neutral additions over every run could
// dispute an archive on a finding that named no defect.

/**
 Category family whose other claims dispute the archive at a disputing
 severity.
 */
const ACCURACY_FAMILY = 'accuracy/';

/**
 Severities at which a non-addition accuracy claim disputes the archive: a
 DELIBERATE SUBSET of `IssueSeverity`, typed by it so a misspelt member fails
 to compile.

 LEFT OUT are `neutral`, the severity asserting no defect, and `minor`,
 which the owner's rule does not count against the archive (additions dispute
 at any severity, by `ADDITION_CATEGORY`). A severity the taxonomy gains is
 therefore not disputing until it is added here.
 */
const DISPUTING_SEVERITIES: ReadonlySet<IssueSeverity> = new Set<IssueSeverity>([
  'major',
  'critical',
],);

/**
 How the findings and the sheets name what disputes the archive.
 */
const DISPUTING_RULE = 'accuracy/addition at any severity, any other accuracy claim the panel settled at major or worse';

/**
 Why a disputed slice's repair text may not stand in for the archive:
 the assembly withdrew it, it is the archive's own wording, or the checkers
 did not confirm every disputing issue resolved in it.

 @example
 ```ts
 const refusal: StandInRefusal = 'archive-wording';
 ```
 */
export type StandInRefusal = 'withdrawn' | 'archive-wording' | 'unresolved';

/**
 How each refusal is said (ledger L12): `outcome` ends the dispute line, and
 `reason` tells a translate author why the repair lane's text cannot ship.
 Both said "the checkers" for all three, and a withdrawn slice can carry every
 disputing issue confirmed.
 */
const REFUSALS: Readonly<Record<StandInRefusal, {
  readonly outcome: string;
  readonly reason: string
}>> = {
  'withdrawn': {
    outcome: 'and the assembly withdrew the repair lane\'s text, so the slice has no eligible standing',
    reason: 'which the repair lane\'s page assembly withdrew from its page',
  },
  'archive-wording': {
    outcome: 'and the repair lane\'s text is the archive\'s own wording, so the slice has no eligible standing',
    reason: 'which is the disputed archive rendering itself',
  },
  'unresolved': {
    outcome: 'and the checkers did not confirm every disputing issue resolved, so the slice has no eligible '
      + 'standing (owner, 2026-09-27)',
    reason: 'which the checkers did not confirm resolves the disputed reading',
  },
};

/**
 One disputed slice and the text standing in for its archive rendering.

 @example
 ```ts
 const dispute: ArchiveDispute = { sliceIndex: 3, standIn: repairedText, standInEligible: true, acceptedClaims: [], };
 ```
 */
export type ArchiveDispute =
  & {
    /**
     Slice whose archive rendering is disputed.
     */
    readonly sliceIndex: number;

    /**
     Repair lane's text for the slice, which stands in for the archive where
     {@link ArchiveDispute.standInEligible} says it may.
     */
    readonly standIn: string;

    /**
     Accepted disputing claims, each as "category severity: summary", for
     the sheets that judge or write against the stand-in (class one hundred
     eight).
     */
    readonly acceptedClaims: readonly string[];
  }
    & (
    | {
      /**
       The checkers confirmed every disputing issue resolved in the repair
       lane's text, the lane kept the slice, and the text differs from the
       archive's, so it stands in for the archive.
       */
      readonly standInEligible: true;
    }
      | {
      /**
       The slice has no eligible standing: the archive's wording and the
       repair lane's text are both refused (owner, 2026-09-27, "No eligible
       standing").
       */
        readonly standInEligible: false;

      /**
       Why the repair lane's text may not stand in, so the line and the
       refused-wording finding name it rather than blaming the checkers
       (ledger L12).
       */
        readonly standInRefusal: StandInRefusal;
      }
    );

/**
 What the reading needs of a repaired chunk: its position, its text, its
 adjudicated issues and which of them the checkers confirmed resolved. A
 structural subset of `ChunkRepairOutcome`, so a guard can build one without
 the rest of the record.

 @example
 ```ts
 const chunk: DisputableChunk = { sliceIndex: 3, repairedText, changed: true, issues, resolvedIssueIds: [], };
 ```
 */
export type DisputableChunk = {
  /**
   Chunk position within the document.
   */
  readonly sliceIndex: number;

  /**
   Winning chunk text; the archive's own when unchanged won.
   */
  readonly repairedText: string;

  /**
   Whether the winning text differs from the archive's, so a stand-in is
   never the disputed wording itself (ledger L12).
   */
  readonly changed: boolean;

  /**
   Adjudicated issues of this chunk.
   */
  readonly issues: readonly AdjudicatedIssue[];

  /**
   Accepted issues the checkers confirmed fixed in the winning text; empty
   when unchanged won.
   */
  readonly resolvedIssueIds: readonly string[];
};

/**
 One accepted issue carrying disputing claims.
 */
type DisputingIssue = {
  readonly issueId: string;
  readonly claims: readonly string[];
};

/**
 Map entry for one dispute, keyed by slice index.
 */
type DisputeEntry = readonly [
  number,
  ArchiveDispute,
];

/**
 Whether an accepted claim disputes the archive rendering: an addition at any
 severity, or another accuracy claim at major severity or worse.

 @param category - category the claim was filed under

 @param severity - severity the panel settled on the claim's issue, never
 the critic's filing (owner, 2026-09-27)

 @returns Whether the claim disputes the archive

 @example
 ```ts
 disputesArchive({ category: 'accuracy/mistranslation', severity: 'major', },); // true
 ```
 */
function disputesArchive(
  {
    category,
    severity,
  }: {
    readonly category: string;
    readonly severity: IssueSeverity;
  },
): boolean {
  if (category === ADDITION_CATEGORY)
    return true;
  return category.startsWith(ACCURACY_FAMILY,) && DISPUTING_SEVERITIES.has(severity,);
}

/**
 One claim as a sheet and a log line state it: "category severity: summary".

 @param member - claim the issue holds, as the critic filed it

 @param severity - severity the panel settled the issue at

 @returns Claim's category, settled severity and critic's summary

 @example
 ```ts
 const line = claimLineOf({ member: issue.claims[0], severity: issue.severity, },);
 ```
 */
export function claimLineOf(
  {
    member,
    severity,
  }: {
    readonly member: AdjudicatedIssue['claims'][number];
    readonly severity: IssueSeverity;
  },
): string {
  /**
   Claim's category and summary as the critic filed them.
   */
  const {
    category,
    summary,
  } = member.claim;
  return `${category} ${severity}: ${summary}`;
}

/**
 Names the accepted issues carrying disputing claims against one chunk's
 archive rendering.

 @param issues - adjudicated issues of the chunk

 @returns Each such issue with its disputing claims, each as
 "category severity: summary" with the panel's severity

 @example
 ```ts
 const disputing = disputingIssuesOf({ issues: chunk.issues, },);
 ```
 */
function disputingIssuesOf(
  { issues, }: { readonly issues: readonly AdjudicatedIssue[]; },
): readonly DisputingIssue[] {
  return issues
    .filter(function isAccepted(issue,): boolean {
      return issue.status === 'accepted';
    },)
    .flatMap(function toDisputing(issue,): readonly DisputingIssue[] {
      /**
       This issue's disputing claims.
       */
      const claims = issue.claims
        .filter(function disputes(member,): boolean {
          /**
           Category the critic filed the claim under.
           */
          const { category, } = member.claim;
          return disputesArchive({
            category,
            severity: issue.severity,
          },);
        },)
        .map(function toBody(member,): string {
          return claimLineOf({
            member,
            severity: issue.severity,
          },);
        },);
      return (claims.length === 0)
        ? []
        : [{
          issueId: issue.issueId,
          claims,
        },];
    },);
}

/**
 Reads the disputed slices off the repair lane's chunks.

 @param chunks - every chunk the repair lane settled

 @param withdrawnSliceIndices - slices whose repair the assembly withdrew,
 whose text never stands in

 @returns Disputes keyed by slice index, in chunk order

 @example
 ```ts
 const disputes = archiveDisputesOf({ chunks: repair.chunks, withdrawnSliceIndices: repair.withdrawnSliceIndices, },);
 ```
 */
export function archiveDisputesOf(
  {
    chunks,
    withdrawnSliceIndices = [],
  }: {
    readonly chunks: readonly DisputableChunk[];
    readonly withdrawnSliceIndices?: readonly number[];
  },
): ReadonlyMap<number, ArchiveDispute> {
  /**
   Slices the assembly withdrew.
   */
  const withdrawn = new Set(withdrawnSliceIndices,);
  return new Map(chunks
    .flatMap(function toDispute(chunk,): readonly DisputeEntry[] {
      /**
       Accepted issues disputing this chunk's archive rendering.
       */
      const disputing = disputingIssuesOf({ issues: chunk.issues, },);
      if (disputing.length === 0)
        return [];
      /**
       Issues the checkers confirmed fixed in the repair lane's text.
       */
      const resolved = new Set(chunk.resolvedIssueIds,);
      /**
       Whether the text may stand in, or why not. ISSUE BY ISSUE (sixteenth
       addendum): one disputing issue the checkers did not confirm fixed
       leaves its reading in the text. And never the archive's own wording
       (ledger L12), whatever the checkers voted on it.
       */
      const verdict: 'eligible' | StandInRefusal = withdrawn.has(chunk.sliceIndex,)
        ? 'withdrawn'
        : (!chunk.changed)
          ? 'archive-wording'
          : disputing.every(function isResolved(issue,): boolean {
            return resolved.has(issue.issueId,);
          },)
            ? 'eligible'
            : 'unresolved';
      return [[
        chunk.sliceIndex,
        {
          sliceIndex: chunk.sliceIndex,
          standIn: chunk.repairedText,
          ...((verdict === 'eligible')
            ? { standInEligible: true, }
            : {
              standInEligible: false,
              standInRefusal: verdict,
            }),
          acceptedClaims: disputing.flatMap(function claimsOf(issue,): readonly string[] {
            return issue.claims;
          },),
        },
      ],];
    },),);
}

/**
 Reads the disputed slices off a repair lane result, the one reading the
 translate lane and the consolidation both take.

 @param repair - repair lane's chunks and slices its assembly withdrew

 @returns Disputes keyed by slice index, in chunk order

 @example
 ```ts
 const disputes = archiveDisputesOfRepair({ repair: lanes.repair, },);
 ```
 */
export function archiveDisputesOfRepair(
  {
    repair,
  }: {
    readonly repair: {
      readonly chunks: readonly DisputableChunk[];
      readonly withdrawnSliceIndices: readonly number[];
    };
  },
): ReadonlyMap<number, ArchiveDispute> {
  return archiveDisputesOf({
    chunks: repair.chunks,
    withdrawnSliceIndices: repair.withdrawnSliceIndices,
  },);
}

/**
 The wordings a disputed slice refuses: the archive's own always, and the
 repair lane's text where it may not stand in (owner, 2026-09-27, "No
 eligible standing").

 @param dispute - disputed slice

 @param archiveText - archive's own wording of the slice

 @returns Each refused wording with why, none twice

 @example
 ```ts
 const disputedWordings = disputedWordingsOf({ dispute, archiveText, },);
 ```
 */
export function disputedWordingsOf(
  {
    dispute,
    archiveText,
  }: {
    readonly dispute: ArchiveDispute;
    readonly archiveText: string;
  },
): readonly DisputedWording[] {
  /**
   Claims the adjudicators accepted against the archive.
   */
  const { acceptedClaims, } = dispute;
  /**
   The archive's own wording, which every accepted claim is against.
   */
  const archive: DisputedWording = {
    text: archiveText,
    reason: `the archive rendering the repair lane's adjudicators disputed (${
      String(acceptedClaims.length,)
    } accepted claim(s))`,
  };
  /**
   The repair lane's text where it is refused beside the archive, with why.
   */
  const repair: readonly DisputedWording[] = (dispute.standInEligible || (dispute.standIn === archiveText))
    ? []
    : [{
      text: dispute.standIn,
      reason: `the repair lane's text for a disputed slice, ${REFUSALS[dispute.standInRefusal]
        .reason}`,
    },];
  // THE ARCHIVE'S WORDING ALWAYS, with no arm for an empty one (ledger T8,
  // eighteenth batch, which removed it): a dispute needs accepted issues, an
  // anchor's repair outcome carries none (`repair-slice-settle.ts`,
  // `notApplicableRepair`), and a present slice holds a parsed block's text;
  // and the rule never matches an empty wording to a candidate anyway
  // (`disputed-wording.ts`).
  return [
    archive,
    ...repair,
  ];
}

/**
 Names a dispute the way every other stage finding is named.

 @param dispute - disputed slice

 @returns Finding in scorecard-stable wording

 @example
 ```ts
 l.warn(describeArchiveDispute({ dispute, },),);
 ```
 */
export function describeArchiveDispute(
  { dispute, }: { readonly dispute: ArchiveDispute; },
): string {
  /**
   Claims the finding counts.
   */
  const { acceptedClaims, } = dispute;
  /**
   What the dispute leaves standing.
   */
  const outcome = dispute.standInEligible
    ? 'so the repair lane\'s text stands in for it'
    : REFUSALS[dispute.standInRefusal]
      .outcome;
  return `translate-archive-disputed (slice ${String(dispute.sliceIndex,)}): the repair lane's adjudicators accepted ${
    String(acceptedClaims.length,)
  } disputing claim(s) (${DISPUTING_RULE}) against the archive rendering, ${outcome} `
    + '(classes one hundred seven and one hundred seventy-six)';
}

/**
 Logs every dispute once, where the lanes meet.

 @param disputes - disputes keyed by slice index

 @param l - lanes logger

 @example
 ```ts
 logArchiveDisputes({ disputes, l, },);
 ```
 */
export function logArchiveDisputes(
  {
    disputes,
    l,
  }: {
    readonly disputes: ReadonlyMap<number, ArchiveDispute>;
    readonly l: Logger;
  },
): void {
  for (const dispute of disputes.values()) {
    l.warn(describeArchiveDispute({ dispute, },),);
  }
}

/**
 Sheet block for a disputed slice: what the adjudicators accepted against the
 archive rendering, and what every writer and judge is to make of a detail
 those claims name.

 THE HUNDRED-AND-EIGHTH CLASS (CuspariaKLSY11 slice 3, 2026-09-24). The
 repair lane softened the archive's invented suicide method to "She took
 medication that night", the consolidated proposal dropped it, and the gate
 kept the stand-in 2 to 1 as "dropped page content ... which the Chinese does
 not contradict": the page apparatus clause protected on the stand-in the very
 detail the stand-in exists to remove. ONE WORDING FOR EVERY SHEET, as that
 clause is, so the writer, the slate judge, the gate and the contest read the
 same rule about the same detail.

 THE ONE HUNDRED FORTY-FOURTH CLASS (hulicaijia27 chunk 69, 2026-09-26). The
 same block made one panel's verdict final on every later sheet, whatever the
 claim named. The panel accepted the archive's footnote 7, a note explaining
 the pun between 晚安 and 金刚烷胺, as an addition 3 to 2, and the block told
 the translate slate that no detail the claim names is apparatus: all four
 judges voted the note out, one citing the accepted claim. So the block's
 bar is narrative detail, stated as the page-apparatus clause bounds it,
 which is what class one hundred eight's softened medication was; a claimed
 note or gloss goes back to the apparatus rule.

 @param dispute - disputed slice

 @returns Block naming the claims and the rule, headed for the sheets

 @example
 ```ts
 const note = archiveDisputeNote({ dispute, },);
 ```
 */
export function archiveDisputeNote(
  { dispute, }: { readonly dispute: ArchiveDispute; },
): string {
  /**
   Claims the note names.
   */
  const { acceptedClaims, } = dispute;
  /**
   Claims numbered the way a ballot can cite them.
   */
  const numbered = acceptedClaims
    .map(function toLine(
      claim,
      index,
    ): string {
      return `(${String(index + 1,)}) ${claim}`;
    },)
    .join('; ',);
  /**
   Whether any accepted claim is an addition, which brings the apparatus rule.
   */
  const carriesAddition = acceptedClaims.some(function isAddition(claim,): boolean {
    return claim.startsWith(`${ADDITION_CATEGORY} `,);
  },);
  /**
   Whether any accepted claim is another accuracy claim, which brings the
   reading rule (class one hundred seventy-six).
   */
  const carriesMisreading = acceptedClaims.some(function isMisreading(claim,): boolean {
    return !claim.startsWith(`${ADDITION_CATEGORY} `,);
  },);
  /**
   Rule for a detail an accepted addition names.
   */
  const additionRule = carriesAddition
    ? ` ${NARRATIVE_DETAIL_IS_NOT_APPARATUS} A detail those addition claims name that says what happened is not `
      + 'page content and not the page\'s apparatus, in the archive\'s wording or any softer one: a candidate '
      + 'leaving it out has dropped nothing, and a candidate keeping it carries an accepted addition. Judge such a '
      + 'detail against the ORIGINAL alone: the adjudicators weighed any cited references before accepting the '
      + `claim, so a cited reference does not reopen it. A claim naming only the page's apparatus (${
        APPARATUS_KINDS
      }) does not make it an addition: judge it by the page-apparatus rule, as if no claim named it.`
    : '';
  /**
   Rule for a reading an accepted mistranslation, omission or untranslated
   claim names.
   */
  const misreadingRule = carriesMisreading
    ? ' A reading those claims name as mistranslated, omitted, left untranslated or no translation of the ORIGINAL '
      + 'at all is not the page\'s authority, '
      + 'in the archive\'s wording or a near copy of it: render what the ORIGINAL says there, and weigh a candidate '
      + 'keeping the archive\'s reading as carrying an accepted error.'
    : '';
  return `ARCHIVE RENDERING DISPUTED: the repair lane's adjudicators accepted ${
    String(acceptedClaims.length,)
  } claim(s) (${DISPUTING_RULE}) that the archive rendering departs from the ORIGINAL; the claims: ${
    numbered
  }.${additionRule}${misreadingRule}`;
}

/**
 Sheet notes per disputed slice, for a stage that reads its slices by index.

 @param chunks - every chunk the repair lane settled

 @returns Notes keyed by slice index, in chunk order

 @example
 ```ts
 const notes = archiveDisputeNotesOf({ chunks: repair.chunks, },);
 ```
 */
export function archiveDisputeNotesOf(
  { chunks, }: { readonly chunks: readonly DisputableChunk[]; },
): ReadonlyMap<number, string> {
  /**
   Disputes keyed by slice.
   */
  const bySlice = archiveDisputesOf({ chunks, },);
  return new Map([...bySlice.values(),]
    .map(function toNote(dispute,): readonly [
      number,
      string,
    ] {
      return [
        dispute.sliceIndex,
        archiveDisputeNote({ dispute, },),
      ];
    },),);
}

//endregion Archive dispute
