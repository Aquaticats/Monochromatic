import { createHash, } from 'node:crypto';

import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type {
  AuditedTextIdentity,
  SettledAuditRow,
} from './rendering-audit-settled-row.ts';

//region Audited text identity
// Says whether two audit rows were shown the same characters, without keeping
// the characters.
//
// WHY A DIGEST AND NOT THE TEXT. The only question a repeat reading asks of
// the text is whether two rows saw the same one, and a digest answers exactly
// that and nothing more, so THIS FIELD carries none. That is a claim about the
// identity field alone: the `report` beside it on the row persists document
// spans and model prose, which the row module says, and a run file of this
// probe is therefore corpus-bearing whatever this field holds.
//
// WHY BOTH SIDES. A repeat is only a repeat when the ORIGINAL and the RENDERING
// both match. Two artifacts of one entry can carry identical source at a slice
// and different English there, which is a comparison of two renderings rather
// than two readings of one, and it belongs in the archive-versus-fresh split
// instead.
//
// AND THE REFERENCES (ledger B29). An auditor shown what the pages the
// original links say is answering a different question from one shown none,
// so two rows over one pair are a repeat only when they were shown the same
// references, or none.

/**
 Marks what the digest is over, so a later change of algorithm or of what is
 fed to it cannot be mistaken for a text change.
 */
const AUDITED_DIGEST_PREFIX = 'sha256-audited-v1:';

/**
 Digests one text the audit was shown.

 @param text - exact characters

 @returns Prefixed digest

 @example
 ```ts
 const digest = auditedDigestOf({ text: sourceText, },);
 ```
 */
function auditedDigestOf({ text, }: { readonly text: string; },): string {
  return `${AUDITED_DIGEST_PREFIX}${
    createHash('sha256',)
      .update(
        text,
        'utf8',
      )
      .digest('hex',)
  }`;
}

/**
 Digests the exact texts one audit was shown.

 @param sourceText - original put in front of the roster

 @param candidateText - rendering it judged

 @param referenceContext - what the pages the original links say as the
 roster was shown them, empty where it was shown none, which leaves the
 identity keyed as every row written before references were shown

 @returns Identity to persist on the row

 @example
 ```ts
 const identity = digestAuditedText({ sourceText, candidateText, referenceContext, },);
 ```
 */
export function digestAuditedText(
  {
    sourceText,
    candidateText,
    referenceContext,
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
    readonly referenceContext: string;
  },
): AuditedTextIdentity {
  return {
    kind: 'digested',
    source: auditedDigestOf({ text: sourceText, },),
    candidate: auditedDigestOf({ text: candidateText, },),
    ...((referenceContext === '') ? {} : { references: auditedDigestOf({ text: referenceContext, },), }),
  };
}

/**
 Whether a value off disk is a recorded pair of digests.

 @param value - field as it came out of the run file, positional because a
 type predicate cannot name a destructured binding

 @returns Whether both digests are there and are strings, and the
 references digest is a string wherever it is there

 @example
 ```ts
 if (isDigested(value,)) console.log(value.source,);
 ```
 */
function isDigested(value: unknown,): value is {
  readonly kind: 'digested';
  readonly source: string;
  readonly candidate: string;
  readonly references?: string;
} {
  if (!isJsonRecord(value,))
    return false;

  /**
   Same value, reachable by key.
   */
  const fields: Record<string, unknown> = { ...value, };

  return (fields.kind === 'digested')
    && ((typeof fields.source) === 'string')
    && ((typeof fields.candidate) === 'string')
    && ((fields.references === undefined) || ((typeof fields.references) === 'string'));
}

/**
 Reads a row's text identity, including rows written before it existed.

 RETURNS `unrecorded` RATHER THAN THROWING. A run persisted before this field
 was added is a valid run whose other readings are all still answerable; only
 the repeat readings need it. Refusing to read the file would cost every other
 reading to serve one.

 The runtime check is deliberate and not redundant with the type. Rows come
 off disk, where the type is a claim about what this code writes today rather
 than about what some older run wrote.

 @param row - one persisted audit row

 @returns What it was shown, or a positive statement that nobody recorded it

 @example
 ```ts
 const identity = textIdentityOf({ row, },);
 ```
 */
export function textIdentityOf(
  { row, }: { readonly row: SettledAuditRow; },
): AuditedTextIdentity {
  /**
   Field as it came off disk.

   READ AS `unknown` rather than as the declared type. The declaration says
   what this code writes today; the value came out of a file that an older
   build wrote, where the field is simply not there.
   */
  const recorded: unknown = row.textIdentity;
  if (!isDigested(recorded,))
    return { kind: 'unrecorded', };
  return recorded;
}

/**
 Whether two rows were shown identical originals, identical renderings and
 identical references, none counting as identical to none.

 TWO UNRECORDED ROWS ARE NOT A MATCH. This is the whole reason the field is a
 tagged union: comparing two absences for equality would pair rows by their
 shared lack of evidence, and every such pair would then be read as one text
 audited twice.

 @param left - one row

 @param right - another

 @returns Whether both sides are recorded and both agree

 @example
 ```ts
 const same = sameAuditedText({ left, right, },);
 ```
 */
export function sameAuditedText(
  {
    left,
    right,
  }: {
    readonly left: SettledAuditRow;
    readonly right: SettledAuditRow;
  },
): boolean {
  /**
   What each was shown.
   */
  const mine = textIdentityOf({ row: left, },);

  /**
   The other side's.
   */
  const theirs = textIdentityOf({ row: right, },);

  if ((mine.kind === 'unrecorded') || (theirs.kind === 'unrecorded'))
    return false;
  return (mine.source === theirs.source)
    && (mine.candidate === theirs.candidate)
    && (mine.references === theirs.references);
}

//endregion Audited text identity
