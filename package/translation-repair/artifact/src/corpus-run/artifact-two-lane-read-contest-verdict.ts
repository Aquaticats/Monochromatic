import {
  requireExactKeys,
  requireOneOf,
} from '../artifact-exact-guard.ts';
import {
  ArtifactParseError,
  requireRecord,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { ArtifactContestVerdict, } from '@monochromatic-dev/translation-repair-lane-contest/ts';

//region Contest verdict reading
// Recorded verdict is derived claim over raw ballots. These helpers compare it
// with parser's re-derived verdict while keeping main contest reader linear.

/**
 Names keys a recorded verdict may carry for derived shape.

 @param derived - verdict raw ballots settle on

 @returns Allowed exact key names

 @example
 ```ts
 const keys = allowedVerdictKeys({ derived, });
 ```
 */
function allowedVerdictKeys(
  { derived, }: { readonly derived: ArtifactContestVerdict; },
): readonly string[] {
  if ((derived.kind === 'lane-won') || (derived.kind === 'sole-lane')) {
    return [
      'kind',
      'lane',
    ];
  }
  if ((derived.kind === 'settled-neither') && (derived.archive !== undefined)) {
    return [
      'kind',
      'archive',
    ];
  }
  return ['kind',];
}

/**
 Names derived verdict in one token.

 @param verdict - verdict to render

 @returns Kind carrying lane or archive outcome where present

 @example
 ```ts
 const token = renderContestVerdict({ verdict, });
 ```
 */
function renderContestVerdict(
  { verdict, }: { readonly verdict: ArtifactContestVerdict; },
): string {
  if ((verdict.kind === 'lane-won') || (verdict.kind === 'sole-lane'))
    return `${verdict.kind}:${verdict.lane}`;
  if ((verdict.kind === 'settled-neither') && (verdict.archive !== undefined))
    return `${verdict.kind}:${verdict.archive}`;
  return verdict.kind;
}

/**
 Renders recorded verdict in same one-token form.

 @param recorded - verdict record artifact carries

 @param path - dotted verdict path

 @returns Recorded verdict token

 @throws ArtifactParseError when field names unknown outcome

 @example
 ```ts
 const token = renderRecordedVerdict({ recorded, path, });
 ```
 */
function renderRecordedVerdict(
  {
    recorded,
    path,
  }: {
    readonly recorded: Readonly<Record<string, unknown>>;
    readonly path: string;
  },
): string {
  if (recorded.lane !== undefined) {
    return `${requireOneOf({
      value: recorded.kind,
      allowed: [
        'lane-won',
        'sole-lane',
      ],
      path: `${path}.kind`,
    },)}:${requireOneOf({
      value: recorded.lane,
      allowed: [
        'repair',
        'translate',
      ],
      path: `${path}.lane`,
    },)}`;
  }
  if (recorded.archive !== undefined) {
    return `${requireOneOf({
      value: recorded.kind,
      allowed: ['settled-neither',],
      path: `${path}.kind`,
    },)}:${requireOneOf({
      value: recorded.archive,
      allowed: [
        'endorsed',
        'declined',
      ],
      path: `${path}.archive`,
    },)}`;
  }
  return requireOneOf({
    value: recorded.kind,
    allowed: [
      'settled-neither',
      'quorum-not-met',
    ],
    path: `${path}.kind`,
  },);
}

/**
 Refuses recorded verdict differing from raw-ballot derivation.

 @param value - recorded verdict value

 @param derived - verdict raw ballots and eligibility settle

 @param path - dotted verdict path

 @throws ArtifactParseError when shape or outcome differs

 @example
 ```ts
 assertContestVerdictMatches({ value, derived, path, });
 ```
 */
export function assertContestVerdictMatches(
  {
    value,
    derived,
    path,
  }: {
    readonly value: unknown;
    readonly derived: ArtifactContestVerdict;
    readonly path: string;
  },
): void {
  /**
   Recorded verdict before fields are read.
   */
  const recorded = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record: recorded,
    allowed: allowedVerdictKeys({ derived, }),
    path,
  },);
  /**
   Verdict artifact claims.
   */
  const claimed = renderRecordedVerdict({
    recorded,
    path,
  },);
  /**
   Verdict evidence settles.
   */
  const settled = renderContestVerdict({ verdict: derived, },);
  if (claimed !== settled) {
    throw new ArtifactParseError({
      path,
      reason: (derived.kind === 'sole-lane')
        ? `${settled}, which is the one lane the comparison leaves applicable, rather than ${claimed}`
        : `${settled}, which is what these ballots settle on, rather than ${claimed}`,
    },);
  }
}

//endregion Contest verdict reading
