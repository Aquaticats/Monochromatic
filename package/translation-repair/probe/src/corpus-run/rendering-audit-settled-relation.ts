import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { ArtifactLaneSelection, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import { anchoredClaims, } from './rendering-audit-settled-read.ts';
import type { SettledAuditRow, } from './rendering-audit-settled-row.ts';
import type {
  WouldShipDecider,
  WouldShipReading,
  WouldShipSilence,
} from '@monochromatic-dev/translation-repair-page-check/ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Page relation
// Says whether the wording one audit subject carries is the wording a document
// assembled today would carry, WITHOUT changing what the audit reads.
//
// WHY BESIDE RATHER THAN INSTEAD. A prescription was once recorded to route the
// settled audit through `wouldShipTextPerSlice`, and measuring the archive
// refuted it. Of 47 settled artifacts, 33 carry `pending-human-decision` and no
// consolidation, so under a would-ship reading their text falls all the way to
// the archive: routing the audit through it would put the ARCHIVE's own
// English under audit at 227 of 271 subjects, which is precisely the case
// `auditsArchiveText` exists to separate out, since the instrument was built
// for output with no BEFORE text.
//
// SO THE SUBJECT KEEPS ITS LANE MEANING and gains this annotation. The audit
// still reads what the judges really decided; a reading of the results can now
// tell which of those decisions a later stage overruled.
//
// THREE ANSWERS, NOT TWO, and the third is the point. The archive standing on
// an artifact nobody has decided is the ABSENCE of a decision, not a
// displacement: the artifact records it as pending a human decision.
// Collapsing it into "the archive ships instead" would be stale the day that is answered.
//
// THE DECIDER IS CARRIED BY REFERENCE rather than spelled into member names
// like `displaced-by-consolidation`. Spelling them would duplicate
// `WouldShipDecider`'s list, so a decider added later would silently produce no
// new relation. That drift class was measured twice in this family.

/**
 How one audited rendering relates to what a document would carry today.

 @example
 ```ts
 const relation: SettledPageRelation = { kind: 'displaced', decidedBy: 'consolidation', };
 ```
 */
export type SettledPageRelation = {
  /**
   No stage has decided this entry, so nothing here is displaced YET.

   Says nothing about the wording's quality: it says the contest has not run.
   */
  readonly kind: 'undecided';
} | {
  /**
   This exact wording is what a document would carry.
   */
  readonly kind: 'survives';
} | {
  /**
   A later stage replaced this wording, so the audit read text no reader of a
   document would meet.
   */
  readonly kind: 'displaced';

  /**
   Stage whose decision survived every stage after it.
   */
  readonly decidedBy: WouldShipDecider;
} | {
  /**
   Nothing at all would stand here, so there is no replacement to name.
   */
  readonly kind: 'nothing-would-ship';

  /**
   Which stage left the slice with no wording.
   */
  readonly reason: WouldShipSilence;

  /**
   Whether the archive held wording here, which decides what the silence
   means: wording the deciders removed, or a gap they left as they found it.
   */
  readonly incumbentKind: 'present' | 'absent';
} | {
  /**
   Row persisted before this annotation existed.

   A TAGGED ABSENCE for the same reason `AuditedTextIdentity` carries one:
   rows come off disk, and a run written by an older build is a valid run
   whose other readings all still answer. Reading a missing field as
   `survives` would assert the strongest claim here from no evidence at all.
   */
  readonly kind: 'unrecorded';
};

/**
 Classifies one subject against what would ship at its slice.

 ORDER IS THE DESIGN. The undecided answer comes first, before any text is
 compared, because on an artifact the contest never ran over every reading
 names the archive and every comparison would report a displacement that no
 stage performed.

 @param laneSelection - whether any stage decided this entry

 @param reading - what would stand at this slice

 @param candidateText - wording the audit was actually shown

 @returns Relation between them

 @example
 ```ts
 const relation = pageRelationOf({ laneSelection, reading, candidateText, },);
 ```
 */
export function pageRelationOf(
  {
    laneSelection,
    reading,
    candidateText,
  }: {
    readonly laneSelection: ArtifactLaneSelection;
    readonly reading: WouldShipReading;
    readonly candidateText: string;
  },
): SettledPageRelation {
  if (laneSelection.kind === 'pending-human-decision')
    return { kind: 'undecided', };

  // A SLICE SHIPPING ITS ORIGINAL (the owner, 2026-10-23, "Source text + note") SHIPS NO ENGLISH, which is what this
  // relation says: no wording the audit was shown stands there, and the four recorded kinds stay the four.
  if ((reading.kind === 'nothing-ships') || (reading.kind === 'source-ships'))
    return {
      kind: 'nothing-would-ship',
      reason: reading.reason,
      incumbentKind: reading.incumbentKind,
    };

  if (reading.text === candidateText)
    return { kind: 'survives', };

  return {
    kind: 'displaced',
    decidedBy: reading.decidedBy,
  };
}

/**
 Whether a value off disk is a relation this build knows how to read.

 @param value - field as persisted

 @returns Whether it names one of the four recorded kinds

 @example
 ```ts
 const known = isRecordedRelation(row.pageRelation,);
 ```
 */
function isRecordedRelation(value: unknown,): value is SettledPageRelation {
  if (!isJsonRecord(value,))
    return false;

  /**
   Its discriminant, read without assuming the shape around it.
   */
  const { kind, } = value as { readonly kind?: unknown; };

  return (kind === 'undecided')
    || (kind === 'survives')
    || (kind === 'displaced')
    || (kind === 'nothing-would-ship');
}

/**
 Reads a row's page relation, including rows written before it existed.

 RETURNS `unrecorded` RATHER THAN THROWING, for the reason `textIdentityOf`
 does: an older run is a valid run, and refusing to read the file would cost
 every other reading to serve one.

 The runtime check is deliberate and not redundant with the type. Rows come
 off disk through an unchecked cast in `rendering-audit-settled-report.ts`,
 where the declared type is a claim about what this build writes rather than
 about what wrote the file.

 @param row - one persisted audit row

 @returns Its relation, or a positive statement that nobody recorded one

 @example
 ```ts
 const relation = pageRelationFor({ row, },);
 ```
 */
export function pageRelationFor(
  { row, }: { readonly row: SettledAuditRow; },
): SettledPageRelation {
  /**
   Field as it came off disk.
   */
  const recorded: unknown = row.pageRelation;
  if (!isRecordedRelation(recorded,))
    return { kind: 'unrecorded', };
  return recorded;
}

/**
 Which of the three things a silence is, as the word a line leads with.

 THREE RATHER THAN TWO, because a row off disk can be older than the field.
 `silent` was the only word this column had until 2026-08-24, and it is kept
 for a row that predates the distinction rather than guessed at: a run that
 never recorded whether the archive had a span here has not said `emptied`,
 and printing it would put a removal that nobody observed into a report.
 `isRecordedRelation` checks the discriminant and nothing under it, so this
 reads the field as the unchecked value it is.

 @param relation - silence to name

 @returns Leading word, one of `gap`, `emptied` or `silent`

 @example
 ```ts
 const word = silenceWord({ relation, },);
 ```
 */
function silenceWord(
  {
    relation,
  }: {
    readonly relation: Extract<SettledPageRelation, { readonly kind: 'nothing-would-ship'; }>;
  },
): 'gap' | 'emptied' | 'silent' {
  /**
   What the row carries, which is `undefined` on one written before the field.
   */
  const recorded: unknown = relation.incumbentKind;

  if (recorded === 'absent')
    return 'gap';
  if (recorded === 'present')
    return 'emptied';
  return 'silent';
}

/**
 Names a relation in one column-width token, for a line being watched.

 @param relation - what to name

 @returns Padded token

 @example
 ```ts
 console.log(pageRelationLabel({ relation, },),);
 ```
 */
export function pageRelationLabel(
  { relation, }: { readonly relation: SettledPageRelation; },
): string {
  if (relation.kind === 'displaced')
    return `displaced:${relation.decidedBy}`;
  if (relation.kind === 'nothing-would-ship')
    return `${silenceWord({ relation, },)}:${relation.reason}`;
  return relation.kind;
}

/**
 One relation, and how much of the audit describes it.

 @example
 ```ts
 const tally: PageRelationTally = { label: 'displaced:consolidation', subjects: 20, claimed: 31, };
 ```
 */
export type PageRelationTally = {
  /**
   Relation these subjects share, named the way a line prints it.
   */
  readonly label: string;

  /**
   Audited slices carrying it.
   */
  readonly subjects: number;

  /**
   Claims that anchored on them, summed over every voice.
   */
  readonly claimed: number;
};

/**
 Counts accumulated for one relation while a run is walked.

 @example
 ```ts
 const running: RelationRunning = { subjects: 0, claimed: 0, };
 ```
 */
type RelationRunning = {
  /**
   Subjects counted so far.
   */
  readonly subjects: number;

  /**
   Claims counted so far.
   */
  readonly claimed: number;
};

/**
 Reads how much of an audit describes wording a later stage overruled.

 COUNTS CLAIMS BESIDE SUBJECTS, because those answer different questions.
 A displaced subject the roster said nothing about cost the run a call and
 nothing else; a displaced subject carrying claims means the instrument
 reported defects in wording no reader of a document would meet.

 NOT A DEFECT RATE, and it may not be read as one. The instrument's own
 error rate is unmeasured, so a count here says how much of
 its output describes overruled text, never how much of that text is bad.

 @param rows - every persisted row of one run

 @returns One tally per relation present, largest first

 @example
 ```ts
 const tallies = relationTallyOf({ rows, },);
 ```
 */
export function relationTallyOf(
  { rows, }: { readonly rows: readonly SettledAuditRow[]; },
): readonly PageRelationTally[] {
  /**
   Running subject and claim counts, keyed by printed label.
   */
  const byLabel = new Map<string, RelationRunning>();

  for (const row of rows) {
    /**
     How this row prints, which is also how it groups.
     */
    const label = pageRelationLabel({ relation: pageRelationFor({ row, },), },);

    /**
     What has been counted under it so far.
     */
    const running = byLabel.get(label,) ?? {
      subjects: 0,
      claimed: 0,
    };

    /**
     Claims this row's roster made that anchored.
     */
    const claims = anchoredClaims({ row, },);

    byLabel.set(
      label,
      {
        subjects: running.subjects + 1,
        claimed: running.claimed + claims.length,
      },
    );
  }

  return [...byLabel.entries(),]
    .map(function asTally([label, tally,]: ReadonlyEntry<string, RelationRunning>,): PageRelationTally {
      return {
        label,
        subjects: tally.subjects,
        claimed: tally.claimed,
      };
    },)
    .toSorted(function bySubjects(
      left,
      right,
    ): number {
      return right.subjects - left.subjects;
    },);
}

//endregion Page relation
