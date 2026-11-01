import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  sameAuditedText,
  textIdentityOf,
} from './rendering-audit-settled-digest.ts';
import { anchoredClaims, } from './rendering-audit-settled-read.ts';
import {
  type SettledAuditRow,
  SLOT_SEPARATOR,
} from './rendering-audit-settled-row.ts';

//region Settled audit repeat readings
// Pairs audits of ONE text so the instrument's own spread can be read off its
// own output.
//
// WHY THIS EXISTS. The headline of the settled audit is a comparison: the archive's own
// English against a fresh rendering. A comparison is unreadable below the band
// the instrument moves through on unchanged input, and that band has already
// been seen to be wide. One subject, `grace-remeasure/Aniloviraw#0`, read
// `claimed=1 corroborated=0` on a capped buy and `claimed=5 corroborated=1
// agreed=1` on the full run: identical text, identical roster, minutes apart.
// Reported without a band, any difference between the two halves is a number a
// reader will take for a finding.
//
// TWO PAIRINGS, both mechanical joins with nothing tunable in them:
//
//   WITHIN one run, two artifacts of one entry can carry the same characters at
//   the same slice. Those are one text audited twice inside a single run.
//
//   ACROSS two runs, every subject appears in both. That is the band directly,
//   at the full width of the population.
//
// BOTH REQUIRE RECORDED TEXT IDENTITY. Pairing by slot would assume that one
// entry settled twice produced the same text at the same index, which is the
// thing being checked rather than something to assume.

/**
 What one side of a repeat said.

 @example
 ```ts
 const side: AuditRepeatSide = { runSet, claimed: 5, corroborated: 1, agreed: 1, near: 0, };
 ```
 */
export type AuditRepeatSide = {
  /**
   Archive subdirectory this side came from, which for an across-run pair is
   the same on both sides.
   */
  readonly runSet: string;

  /**
   Claims that anchored, summed over every voice.
   */
  readonly claimed: number;

  /**
   Defects two auditors located identically.
   */
  readonly corroborated: number;

  /**
   Groups that agreed without quoting identical spans.
   */
  readonly agreed: number;

  /**
   Pairs that nearly agreed.
   */
  readonly near: number;
};

/**
 One text audited twice, with what each audit said.

 @example
 ```ts
 const pair: AuditRepeatPair = { entryId, sliceIndex: 0, left, right, };
 ```
 */
export type AuditRepeatPair = {
  /**
   Corpus entry.
   */
  readonly entryId: string;

  /**
   Global slice index.
   */
  readonly sliceIndex: number;

  /**
   Which half of the population this pair sits in, so a band can be read for
   archive text and fresh text apart rather than blurred across both.
   */
  readonly auditsArchiveText: boolean;

  /**
   First audit of this text.
   */
  readonly left: AuditRepeatSide;

  /**
   Second audit of the same text.
   */
  readonly right: AuditRepeatSide;
};

/**
 Reads a row's slice index under the name its generation wrote it.

 A run written before the rename of 2026-08-24 (`49e5a41cd`, "as generation 4")
 carries `chunkIndex` where this generation writes `sliceIndex`. The rename
 changed the field's name and nothing it holds: the rows of both generations
 describe the global slice index, so an older run pairs with a newer one by
 this value like any other.

 @param row - one row of a run file

 @returns Its `chunkIndex` when it carries no numeric `sliceIndex` and a numeric
 `chunkIndex`, otherwise its `sliceIndex`

 @example
 ```ts
 const index = sliceIndexOf({ row, },);
 ```
 */
function sliceIndexOf(
  { row, }: { readonly row: SettledAuditRow; },
): number {
  /**
   The row as the file wrote it, whatever the cast claims.
   */
  const written: unknown = row;
  if (!isJsonRecord(written,))
    throw new Error('unreachable: a row that is not an object reached its subject key, though the run reader refuses every such row',);

  /**
   The slice index under the earlier generation's name.
   */
  const { chunkIndex, } = written;
  if (((typeof written.sliceIndex) !== 'number') && ((typeof chunkIndex) === 'number'))
    return chunkIndex;
  return row.sliceIndex;
}

/**
 Names the subject one row describes, run set included.

 ONE BUILDER, used by both the map and the lookup. Spelling the key twice is
 how two builders come to disagree, and a disagreement here reports that two
 runs share no subjects at all, which reads exactly like an honest null.

 @param row - one audited slice

 @returns Key that is equal for one subject across two runs

 @example
 ```ts
 const key = subjectKey({ row, },);
 ```
 */
function subjectKey(
  { row, }: { readonly row: SettledAuditRow; },
): string {
  return [
    row.runSet,
    row.entryId,
    String(sliceIndexOf({ row, },),),
  ].join(SLOT_SEPARATOR,);
}

/**
 Whether a row carries the three parts a subject key joins, as the types the
 key reads them in.

 @param row - one row of a run file, as the cast calls it

 @returns Whether its run set and entry id are text and its slice index, under either generation's name, a number

 @example
 ```ts
 const named = namesItsSubject({ row, },);
 ```
 */
function namesItsSubject(
  { row, }: { readonly row: unknown; },
): boolean {
  return isJsonRecord(row,)
    && ((typeof row.runSet) === 'string')
    && ((typeof row.entryId) === 'string')
    && (((typeof row.sliceIndex) === 'number') || ((typeof row.chunkIndex) === 'number'));
}

/**
 Keys one run's rows by subject, refusing a row that names no subject and a
 subject two rows name.

 THE ROWS COME FROM A RUN FILE THE READER CASTS RATHER THAN CHECKS, so the
 parts of the key are checked here, where they are joined. A later row under
 a key the map already holds would replace the earlier, and an earlier run
 naming a subject twice would pair its later row with both and count one
 audit twice in the band. The refusal names positions, never a run set, an
 entry or a slice, which came out of a file.

 @param rows - rows of one run

 @param which - which run these are, `earlier` or `later`, for the refusal

 @returns Each row under its subject key

 @throws {@link StatedRefusalError} when a row names its subject by anything
 but a text run set, a text entry id and a numeric slice index, or when two
 rows name one subject

 @example
 ```ts
 const bySubject = rowsBySubject({ rows, which: 'later', },);
 ```
 */
function rowsBySubject(
  {
    rows,
    which,
  }: {
    readonly rows: readonly SettledAuditRow[];
    readonly which: 'earlier' | 'later';
  },
): ReadonlyMap<string, SettledAuditRow> {
  /**
   Rows by subject key.
   */
  const byKey = new Map<string, SettledAuditRow>();

  /**
   Position of the row that first named each subject.
   */
  const firstAt = new Map<string, number>();
  for (const [position, row,] of rows.entries()) {
    if (!namesItsSubject({ row, },)) {
      throw new StatedRefusalError({
        says: `row ${String(position,)} of the ${which} run does not name its subject by a text run set, `
          + 'a text entry id and a numeric slice index',
      },);
    }

    /**
     Subject this row names.
     */
    const key = subjectKey({ row, },);

    /**
     Position of an earlier row naming the same subject, when there is one.
     */
    const seenAt = firstAt.get(key,);
    if (seenAt !== undefined) {
      throw new StatedRefusalError({
        says: `the ${which} run names one subject on two rows, row ${String(seenAt,)} and row ${
          String(position,)
        }, so which of them the pairing should read cannot be told`,
      },);
    }
    firstAt.set(
      key,
      position,
    );
    byKey.set(
      key,
      row,
    );
  }
  return byKey;
}

/**
 Whether a row says what its audit was shown.

 @param row - one audited slice

 @returns Whether the run recorded a text identity for it

 @example
 ```ts
 const vouched = recorded({ row, },);
 ```
 */
function recorded(
  { row, }: { readonly row: SettledAuditRow; },
): boolean {
  /**
   What the run wrote down about this subject's texts.
   */
  const identity = textIdentityOf({ row, },);
  return identity.kind === 'digested';
}

/**
 Reads one row down to what a repeat comparison needs.

 @param row - one audited slice

 @returns That side of a pair

 @example
 ```ts
 const side = repeatSideOf({ row, },);
 ```
 */
function repeatSideOf(
  { row, }: { readonly row: SettledAuditRow; },
): AuditRepeatSide {
  /**
   Tiers this audit reached.
   */
  const {
    corroborated,
    agreed,
    near,
  } = row.report;

  /**
   Every claim this subject's roster made that anchored.
   */
  const claims = anchoredClaims({ row, },);

  return {
    runSet: row.runSet,
    claimed: claims.length,
    corroborated: corroborated.length,
    agreed: agreed.length,
    near: near.length,
  };
}

/**
 One repeat pair from two audits of one slice. Both the within-run and the
 across-run readings build pairs here (audit area six, ledger B18: each kept
 its own copy).

 @param left - earlier audit, whose slice names the pair

 @param right - later audit of the same slice

 @returns The pair

 @example
 ```ts
 const pair = repeatPairOf({ left, right, },);
 ```
 */
function repeatPairOf(
  {
    left,
    right,
  }: {
    readonly left: SettledAuditRow;
    readonly right: SettledAuditRow;
  },
): AuditRepeatPair {
  return {
    entryId: left.entryId,
    sliceIndex: sliceIndexOf({ row: left, },),
    auditsArchiveText: left.auditsArchiveText,
    left: repeatSideOf({ row: left, },),
    right: repeatSideOf({ row: right, },),
  };
}

/**
 Finds texts one run audited more than once.

 Two artifacts of one entry sit in different run sets, so a repeat here is a
 pair of rows sharing an entry and a slice index across run sets whose
 recorded text identity also matches.

 @param rows - every audited slice of one run

 @returns One pair per repeated text, in first-seen order

 @example
 ```ts
 const repeats = auditRepeatsWithin({ rows, },);
 ```
 */
export function auditRepeatsWithin(
  { rows, }: { readonly rows: readonly SettledAuditRow[]; },
): readonly AuditRepeatPair[] {
  /**
   Rows sharing an entry and a slice, which is the only place a repeat can be.
   */
  const bySlot = new Map<string, SettledAuditRow[]>();
  rows.forEach(function place(row,): void {
    /**
     Slot this row occupies, which deliberately omits the run set: the whole
     point is to bring two run sets together.
     */
    const slot = [
      row.entryId,
      String(sliceIndexOf({ row, },),),
    ].join(SLOT_SEPARATOR,);
    bySlot.set(
      slot,
      [
        ...(bySlot.get(slot,) ?? []),
        row,
      ],
    );
  },);

  /**
   Slots, each holding every row that landed in it.
   */
  const slots = [...bySlot.values(),];

  return slots.flatMap(function pairsIn(sharing,): readonly AuditRepeatPair[] {
    return sharing.flatMap(function against(
      left,
      at,
    ): readonly AuditRepeatPair[] {
      return sharing
        .slice(at + 1,)
        .filter(function isRepeat(right,): boolean {
          return (left.runSet !== right.runSet) && sameAuditedText({
            left,
            right,
          },);
        },)
        .map(function asPair(right,): AuditRepeatPair {
          return repeatPairOf({
            left,
            right,
          },);
        },);
    },);
  },);
}

/**
 Pairs two runs of the same population, subject against subject.

 KEYED BY RUN SET AS WELL as entry and slice, so two artifacts of one entry
 are never crossed with each other; that pairing is `auditRepeatsWithin`'s job
 and means something different.

 THREE OUTCOMES, NOT TWO, and the third is the one that matters most in
 practice. A slot both runs recorded and whose digests DISAGREE means the
 archive changed underneath them, which invalidates that subject as a band
 measurement and is worth saying. A slot either run left UNRECORDED means
 nobody wrote down what was shown, which is a fact about the run and says
 nothing whatever about the archive.

 Collapsing those two into one list would report an older run, from before the
 identity field existed, as a population whose every subject changed text
 between the runs. That is a confident statement about the corpus assembled
 out of the absence of evidence about the probe.

 @param first - rows of the earlier run

 @param second - rows of the later one

 @returns Pairs, slots whose text moved, and slots nobody can vouch for

 @throws {@link StatedRefusalError} when a run's row names no subject, or
 when either run names one subject on two rows

 @example
 ```ts
 const { paired, textMoved, unverifiable, } = auditRepeatsAcross({ first, second, },);
 ```
 */
export function auditRepeatsAcross(
  {
    first,
    second,
  }: {
    readonly first: readonly SettledAuditRow[];
    readonly second: readonly SettledAuditRow[];
  },
): {
  readonly paired: readonly AuditRepeatPair[];
  readonly textMoved: readonly string[];
  readonly unverifiable: readonly string[];
} {
  /**
   Later run, reachable by slot.
   */
  const laterBySlot = rowsBySubject({
    rows: second,
    which: 'later',
  },);

  /**
   Earlier run keyed the same way, so a subject it names twice is refused
   before it pairs with the later row twice.
   */
  const earlierBySlot = rowsBySubject({
    rows: first,
    which: 'earlier',
  },);

  /**
   Slots both runs hold, split by whether the text also agreed.
   */
  const matched = [...earlierBySlot.values(),].flatMap(function join(row,): readonly {
    readonly left: SettledAuditRow;
    readonly right: SettledAuditRow;
  }[] {
    /**
     Same slot in the later run.
     */
    const right = laterBySlot.get(subjectKey({ row, },),);
    if (right === undefined)
      return [];
    return [{
      left: row,
      right,
    },];
  },);

  /**
   Slots where BOTH runs said what they saw, so the digests decide.
   */
  const witnessed = matched.filter(function bothRecorded({
    left,
    right,
  },): boolean {
    return recorded({ row: left, },) && recorded({ row: right, },);
  },);

  return {
    paired: witnessed
      .filter(function textAgrees({
        left,
        right,
      },): boolean {
        return sameAuditedText({
          left,
          right,
        },);
      },)
      .map(function asPair({
        left,
        right,
      },): AuditRepeatPair {
        return repeatPairOf({
          left,
          right,
        },);
      },),
    textMoved: witnessed
      .filter(function textDisagrees({
        left,
        right,
      },): boolean {
        return !sameAuditedText({
          left,
          right,
        },);
      },)
      .map(nameOf,),
    unverifiable: matched
      .filter(function eitherUnrecorded({
        left,
        right,
      },): boolean {
        return !(recorded({ row: left, },) && recorded({ row: right, },));
      },)
      .map(nameOf,),
  };
}

/**
 Names one matched slot for a reader.

 @param left - row from the earlier run, which carries the naming parts

 @returns Slot as a reader would write it

 @example
 ```ts
 const name = nameOf({ left, },);
 ```
 */
function nameOf(
  { left, }: { readonly left: SettledAuditRow; },
): string {
  return `${left.runSet}/${left.entryId}#${String(sliceIndexOf({ row: left, },),)}`;
}

//endregion Settled audit repeat readings
