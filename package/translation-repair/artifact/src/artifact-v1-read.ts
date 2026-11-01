import {
  ArtifactParseError,
  parseRecordRepair,
  type RecordRepairReading,
  requireArray,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  type ArtifactChangeSets,
  readArtifactChangeSets,
} from './artifact-change-sets.ts';
import type {
  GradableClaim,
  GradableIssue,
  GradableSpan,
} from '@monochromatic-dev/translation-repair-house-rule/ts';

//region Artifact version 1 reading
// Parses a LEGACY or version 1 artifact's JSON back into the accepted issues
// that form the
// precision denominator. This is a MEASUREMENT INSTRUMENT, not a lenient
// deserializer: a malformed accepted issue is surfaced loudly (thrown), never
// skipped, because silently dropping one would shrink the accepted population
// and bias the graded precision. Non-accepted issues (rejected, needs-human)
// are legitimately excluded -- they are not accepted, so not in the
// denominator. Category and severity stay plain display strings, so no
// off-taxonomy value is ever a reason to drop an issue.

/**
 Parses one span, requiring a known side and a quoted-text string.

 @param value - the span JSON

 @param path - dotted path for the error message

 @returns The gradable span
 */
function parseSpan(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): GradableSpan {
  /**
   The span as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);

  /**
   The claimed document side; only source and target are valid.
   */
  const {side} = record;
  if ((side !== 'source') && (side !== 'target'))
    throw new ArtifactParseError({
      path: `${path}.side`,
      reason: "'source' or 'target'",
    },);

  return {
    side,
    quotedText: requireString({
      value: record.quotedText,
      path: `${path}.quotedText`,
    },),
  };
}

/**
 Parses one member claim wrapper into its gradable claim.

 @param value - the member (aggregated-claim) JSON

 @param path - dotted path for the error message

 @returns The wrapped gradable claim
 */
function parseMember(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): { readonly claim: GradableClaim; } {
  /**
   The member as a record.
   */
  const member = requireRecord({
    value,
    path,
  },);

  /**
   The inner claim as a record.
   */
  const claim = requireRecord({
    value: member.claim,
    path: `${path}.claim`,
  },);

  /**
   The claim's anchored spans.
   */
  const spans = requireArray({
    value: claim.spans,
    path: `${path}.claim.spans`,
  },)
    .map(function parseSpanAt(
      spanValue,
      spanIndex,
    ) {
      return parseSpan({
        value: spanValue,
        path: `${path}.claim.spans[${String(spanIndex,)}]`,
      },);
    },);

  return {
    claim: {
      category: requireString({
        value: claim.category,
        path: `${path}.claim.category`,
      },),
      summary: requireString({
        value: claim.summary,
        path: `${path}.claim.summary`,
      },),
      spans,
    },
  };
}

/**
 Parses one accepted issue into its gradable shape.

 @param issue - the issue record (already confirmed accepted)

 @param path - dotted path for the error message

 @returns The gradable issue
 */
function parseAcceptedIssue(
  {
    issue,
    path,
  }: {
    readonly issue: Readonly<Record<string, unknown>>;
    readonly path: string;
  },
): GradableIssue {
  return {
    issueId: requireString({
      value: issue.issueId,
      path: `${path}.issueId`,
    },),
    severity: requireString({
      value: issue.severity,
      path: `${path}.severity`,
    },),
    claims: requireArray({
      value: issue.claims,
      path: `${path}.claims`,
    },)
      .map(function parseMemberAt(
        memberValue,
        memberIndex,
      ) {
        return parseMember({
          value: memberValue,
          path: `${path}.claims[${String(memberIndex,)}]`,
        },);
      },),
  };
}

/**
 One accepted issue with whatever the run recorded about repairing it.

 Kept a wrapper rather than folded into {@link GradableIssue} so the issue
 shape stays the thing a pipeline `AdjudicatedIssue` satisfies structurally,
 and so repair provenance stays visibly a property of the RECORD the driver
 built rather than of the panel's decision.

 @example
 ```ts
 const accepted: ParsedAcceptedIssue = { issue, repair: { kind: 'unrecorded', }, };
 ```
 */
export type ParsedAcceptedIssue = {
  /**
   Accepted issue as the panel decided it.
   */
  readonly issue: GradableIssue;

  /**
   What became of its repair, carried as the READING rather than as an
   optional field.

   The reading already distinguishes an artifact written before repair
   recording existed from one recording a repair, and `parseRecordRepair`
   names that distinction for the stated reason that an absent field is the
   one thing a caller can forget to check. This used to hold
   `repair?: GradableRepair`, which threw the distinction away one call after
   it was drawn: a caller then met `undefined` and had to decide for itself
   whether that meant an older artifact or a run that repaired nothing, which
   is the question the reading answers.
   */
  readonly repair: RecordRepairReading;
};

/**
 One artifact's settled identity and its accepted issues.

 @example
 ```ts
 const parsed: ParsedArtifact = {
   id: 'MushroomGuuuu',
   status: 'repaired',
   acceptedIssues: [],
   changeSets: { kind: 'unrecorded', },
 };
 ```
 */
export type ParsedArtifact = {
  /**
   Corpus entry id.
   */
  readonly id: string;

  /**
   Settled status the artifact carries (repaired, unchanged, or blocked).
   */
  readonly status: string;

  /**
   Every accepted issue, the precision denominator for this entry.
   */
  readonly acceptedIssues: readonly ParsedAcceptedIssue[];

  /**
   Which slices the settled document changed, with its generation named.

   A DOCUMENT fact rather than an issue one, which is why it rides here beside
   the issues rather than inside them: a slice can be withdrawn while carrying
   no accepted issue of its own, and a run that recorded nothing is not a run
   that changed nothing.
   */
  readonly changeSets: ArtifactChangeSets;
};

/**
 Parses a run artifact into its accepted issues. Every issue is inspected: a
 malformed accepted issue throws {@link ArtifactParseError} rather than being
 skipped, so the accepted population is never silently short. Non-accepted
 issues are excluded because they are not part of the precision denominator.

 @param value - the artifact JSON, freshly parsed and still untyped

 @returns Entry id, status, accepted issues, and which slices the document
 changed

 @throws {@link ArtifactParseError} when the artifact or an accepted issue is
 structurally malformed, when it carries one index set without the other, or
 when the two sets break a rule the writing lanes hold them to

 @example
 ```ts
 const parsed = parseSettledArtifact({
   value: await readRunJson({ path, },),
 },);
 ```
 */
export function parseSettledArtifact(
  { value, }: { readonly value: unknown; },
): ParsedArtifact {
  /**
   The artifact as a record.
   */
  const artifact = requireRecord({
    value,
    path: 'artifact',
  },);

  /**
   Entry id, used both in the result and in nested error paths.
   */
  const id = requireString({
    value: artifact.id,
    path: 'artifact.id',
  },);

  /**
   Settled status string.
   */
  const status = requireString({
    value: artifact.status,
    path: `${id}.status`,
  },);

  /**
   Accepted issues gathered across every issue record.
   */
  const acceptedIssues: readonly ParsedAcceptedIssue[] = requireArray({
    value: artifact.issues,
    path: `${id}.issues`,
  },)
    .flatMap(function acceptedAt(
      recordValue,
      recordIndex,
    ) {
      /**
       The issue record wrapper at this index.
       */
      const record = requireRecord({
        value: recordValue,
        path: `${id}.issues[${String(recordIndex,)}]`,
      },);

      /**
       The adjudicated issue inside the record.
       */
      const issue = requireRecord({
        value: record.issue,
        path: `${id}.issues[${String(recordIndex,)}].issue`,
      },);

      /**
       The issue's fate; must be readable to classify it.
       */
      const issueStatus = requireString({
        value: issue.status,
        path: `${id}.issues[${String(recordIndex,)}].issue.status`,
      },);

      if (issueStatus !== 'accepted')
        return [];

      /**
       Repair provenance of this record, or a named absence when the artifact
       predates repair recording.
       */
      const reading = parseRecordRepair({
        record,
        path: `${id}.issues[${String(recordIndex,)}]`,
      },);

      return [
        {
          issue: parseAcceptedIssue({
            issue,
            path: `${id}.issues[${String(recordIndex,)}].issue`,
          },),

          // The reading ITSELF, rather than its repair spread in when there is
          // one. Collapsing it here handed the caller an absent field and the
          // question the reading had just answered.
          repair: reading,
        },
      ];
    },);

  return {
    id,
    status,
    acceptedIssues,
    changeSets: readArtifactChangeSets({
      artifact,
      path: id,
    },),
  };
}

//endregion Artifact version 1 reading
