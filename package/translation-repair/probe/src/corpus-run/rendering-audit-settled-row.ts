import type { RenderingAuditReport, } from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import type { SettledPageRelation, } from './rendering-audit-settled-relation.ts';
import type { SettledReferences, } from './rendering-audit-settled-subject.ts';

//region Settled audit row
// What one audited slice becomes on disk.
//
// ITS OWN MODULE so the probe that writes rows and the reading that interprets
// them can share a shape without the reading importing the probe. Importing the
// probe would drag in a model client and a roster to read a file that is
// already on disk, and a reading that costs quota to run is a reading nobody
// runs twice.

/**
 Joins the parts of a composite key over these rows.

 NUL, WRITTEN AS AN ESCAPE so it is visible in source. A run set, an entry id
 and a slice index are joined into one string wherever rows are grouped, and
 any separator that can occur inside a part makes two different tuples collide
 on one key. NUL cannot occur in any of them.

 ONE CONSTANT, SHARED, because the failure mode here is two key builders that
 disagree. That has already happened once in this package, between a NUL and a
 space, and every fixture that spelled its own key passed while no live run
 ever paired anything.
 */
export const SLOT_SEPARATOR = '\u0000';

/**
 Name the probe store collects runs of the settled audit under, shared by
 the command that writes them and the readers that open them.

 DELIBERATELY NOT `census`. It becomes a directory name that outlives every
 caveat written beside it, and two entries are not a census.
 */
export const SETTLED_AUDIT_PROBE = 'rendering-audit-settled';

/**
 Digests of the exact texts one audit was shown: the pair, and the cited
 references where there were any.

 A TAGGED ABSENCE rather than two optional strings, because the question a
 reader asks of this field is whether two rows audited the SAME characters,
 and a missing digest must answer "cannot say" rather than compare equal to
 another missing digest. Rows persisted before this field existed carry
 nothing here, and pairing them by slot alone would assert text identity from
 index equality, which is the assumption the field was added to stop.

 @example
 ```ts
 const identity: AuditedTextIdentity = { kind: 'digested', source, candidate, };
 ```
 */
export type AuditedTextIdentity = {
  readonly kind: 'digested';

  /**
   Original this audit read.
   */
  readonly source: string;

  /**
   Rendering it judged.
   */
  readonly candidate: string;

  /**
   What the pages the original links say, as the roster was shown them
   (ledger B29).

   ABSENT WHERE NONE WERE SHOWN, rather than a digest of the empty string.
   Every row written before the audit showed references was shown none, so
   an absent field keys a row shown none today exactly as it keys those,
   and only a row that was shown references keys differently.
   */
  readonly references?: string;
} | {
  readonly kind: 'unrecorded';
};

/**
 One audited slice, with everything needed to say which decision it describes.

 @example
 ```ts
 const row: SettledAuditRow = { runSet, entryId, sliceIndex, report, ... };
 ```
 */
export type SettledAuditRow = {
  /**
   Archive subdirectory, which is the only thing separating two runs of one
   entry: both write a file named for the entry.
   */
  readonly runSet: string;

  /**
   Corpus entry.
   */
  readonly entryId: string;

  /**
   Global slice index.
   */
  readonly sliceIndex: number;

  /**
   What the lane's document carries here.
   */
  readonly deliveryKind: string;

  /**
   Whether this audited the archive's own English rather than a fresh
   rendering, which is the split every aggregate must respect.
   */
  readonly auditsArchiveText: boolean;

  /**
   Whether a later stage overruled the wording this row audited.

   BESIDE `auditsArchiveText`, never instead of it. That field says which
   text the LANE delivered, which is what the audit read and what its
   denominator must stay split by. This one says what a document assembled
   today would carry at the same slice.

   READ THROUGH `pageRelationFor`, never directly: rows persisted before
   this field existed carry nothing here, and the cast that loads them does
   not check.
   */
  readonly pageRelation: SettledPageRelation;

  /**
   Built output that produced the decision under audit.
   */
  readonly artifactDigest: string;

  /**
   Corpus commit the pair was read at.
   */
  readonly corpusSha: string;

  /**
   Whether the producing run had declared names to pass on.
   */
  readonly identityKind: string;

  /**
   Whether the roster was shown what the pages the original links say
   (ledger B29): `cited` when it was, `none` when the page links nowhere,
   `unread` when the page links pages and the read returned nothing, so
   the audit read the slice without them. Rows persisted before this
   field existed carry nothing here, and every one of them was shown none.
   */
  readonly referencesKind: SettledReferences['kind'];

  /**
   What this audit was actually shown, by digest.

   THIS FIELD carries no text: a digest answers the one question a repeat
   reading needs, which is whether two rows saw identical characters, and
   answers nothing else. It is the only field on the row built to carry
   none; see `report` for what the row does carry.
   */
  readonly textIdentity: AuditedTextIdentity;

  /**
   Everything the instrument said, WHOLE and uninterpreted.

   CARRIES DOCUMENT SPANS. Every screened finding holds `locator.text` and
   `focus.text` sliced from the original and from the rendering under audit,
   and every voice's `reason` is model prose about them, so a persisted run
   of this probe is corpus-bearing and the end-of-project sanitization must treat
   the runs directory's `rendering-audit-settled` files as such. An earlier
   version of this note claimed the row carried no text; that was true of
   `textIdentity` and never of this field.

   NOT SUMMARISED INTO COUNTS, which the first two-subject buy was bought to
   find out. Counts read `corroborated=0 agreed=0 near=1` over two voices
   claiming two defects each and a third dropping one, and nothing in the file
   could say WHAT any of them claimed. That leaves three questions
   unanswerable from the artifact this probe exists to produce: whether the
   matcher was right to bring nothing together, which voice was right
   when they disagreed, and whether a paired omission and addition on
   adjacent slices is one relocation rather than two defects, a rule
   fixed before the run and unenforceable without categories and spans.

   Every count a reader wants is derivable from this. None of this is
   recoverable from the counts.
   */
  readonly report: RenderingAuditReport;
};

//endregion Settled audit row
