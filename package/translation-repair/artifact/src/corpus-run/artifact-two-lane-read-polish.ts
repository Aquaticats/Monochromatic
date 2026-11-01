import { requireExactKeys, } from '../artifact-exact-guard.ts';
import {
  ArtifactParseError,
  requireBoolean,
  requireCount,
  requireRecord,
  requireString,
  requireStringList,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { ArtifactConsolidationPolish, } from './artifact-two-lane-consolidate.ts';
import { parseNaturalnessReview, } from './artifact-two-lane-read-naturalness-review.ts';
import { parsePolishGate, } from './artifact-two-lane-read-polish-gate.ts';

//region Artifact consolidation polish read

/**
 Reasons a polish that did not run may record, keyed by the frozen
 `ArtifactConsolidationPolish` rather than listed, so the compiler refuses a
 reason the polish record gains that this reader lacks. The order is the order
 a refusal names them in.
 */
const NOT_RUN_REASONS: Readonly<
  Record<Extract<ArtifactConsolidationPolish, { readonly kind: 'not-run'; }>['reason'], true>
  > = {
    'front-matter': true,
    'not-configured': true,
    'unsafe-baseline': true,
  };

/**
 Whether a recorded value names a reason a polish may not have run.

 @param value - reason as the artifact carries it

 @returns Whether it is one of the reasons `NOT_RUN_REASONS` holds

 @example
 ```ts
 const known = isNotRunReason('front-matter',);
 ```
 */
function isNotRunReason(
  value: unknown,
): value is Extract<ArtifactConsolidationPolish, { readonly kind: 'not-run'; }>['reason'] {
  return ((typeof value) === 'string')
    && Object.hasOwn(
      NOT_RUN_REASONS,
      value,
    );
}

/**
 Reads generation-six post-consolidation polish record.

 @param value - unknown polish field

 @param path - artifact path

 @param reviewRequired - whether generation carries exact-text absolute review

 @param correctionChainRequired - whether review requires digest-bound corrections

 @param everyBodyBlockReviewed - whether reviewed paragraphs are every body
 block rather than the refinable paragraphs alone

 @param quorumBasisRequired - whether generation records its wider review quorum

 @param lastSeatGrace - whether review rounds were decided with the last-seat
 grace of 2026-10-22 (generation twenty on)

 @returns Parsed polish record

 @example
 ```ts
 const polish = parseConsolidationPolish({ value, path, });
 ```
 */
export function parseConsolidationPolish(
  {
    value,
    path,
    reviewRequired = false,
    correctionChainRequired = false,
    everyBodyBlockReviewed = false,
    quorumBasisRequired = false,
    lastSeatGrace = false,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly reviewRequired?: boolean;
    readonly correctionChainRequired?: boolean;
    readonly everyBodyBlockReviewed?: boolean;
    readonly quorumBasisRequired?: boolean;
    readonly lastSeatGrace?: boolean;
  },
): ArtifactConsolidationPolish {
  /**
   Polish field under exact generation-six shape.
   */
  const record = requireRecord({
    value,
    path,
  },);
  if (record.kind === 'not-run') {
    requireExactKeys({
      record,
      allowed: [
        'kind',
        'reason',
      ],
      path,
    },);
    /**
     Reason the record names, before it is known to be one a polish may carry.
     */
    const { reason, } = record;
    if (!isNotRunReason(reason,)) {
      throw new ArtifactParseError({
        path: `${path}.reason`,
        reason: `one of ${
          Object.keys(NOT_RUN_REASONS,)
            .join(', ',)
        }`,
      },);
    }
    return {
      kind: 'not-run',
      reason,
    };
  }
  if (record.kind !== 'settled') {
    throw new ArtifactParseError({
      path: `${path}.kind`,
      reason: 'one of settled, not-run',
    },);
  }
  requireExactKeys({
    record,
    allowed: [
      'kind',
      'baseText',
      'proposedText',
      'text',
      'changed',
      'refinersHeard',
      'contributors',
      'roundCount',
      'gate',
      ...(reviewRequired ? ['review',] : []),
      'findings',
    ],
    path,
  },);
  /**
   Approved wording before naturalness stage.
   */
  const baseText = requireString({
    value: record.baseText,
    path: `${path}.baseText`,
  },);
  /**
   Selected rewrite before final gate.
   */
  const proposedText = requireString({
    value: record.proposedText,
    path: `${path}.proposedText`,
  },);
  /**
   Wording after final gate.
   */
  const text = requireString({
    value: record.text,
    path: `${path}.text`,
  },);
  /**
   Recorded final replacement status.
   */
  const changed = requireBoolean({
    value: record.changed,
    path: `${path}.changed`,
  },);
  if (changed !== (text !== baseText)) {
    throw new ArtifactParseError({
      path: `${path}.changed`,
      reason: `${String(text !== baseText,)}, which is whether final text differs from baseText`,
    },);
  }
  /**
   Optional final gate as explicit presence reading.
   */
  const gateReading = (record.gate === undefined)
    ? { kind: 'absent', } as const
    : {
      kind: 'present',
      gate: parsePolishGate({
        value: record.gate,
        path: `${path}.gate`,
      },),
    } as const;
  if (changed && (gateReading.kind === 'absent')) {
    throw new ArtifactParseError({
      path: `${path}.gate`,
      reason: 'final gate approving every changed polish',
    },);
  }
  if (gateReading.kind === 'present') {
    /**
     Parsed final gate under present reading.
     */
    const { gate, } = gateReading;
    /**
     Shipping role implied by panel choice under conservative gate rule.
     */
    const expectedShips = (gate.choice === 'polished')
      ? 'polished'
      : 'base';
    if (gate.ships !== expectedShips) {
      throw new ArtifactParseError({
        path: `${path}.gate.ships`,
        reason: `${expectedShips}, derived from gate choice`,
      },);
    }
    if (changed && ((gate.ships !== 'polished') || (text !== proposedText))) {
      throw new ArtifactParseError({
        path,
        reason: 'changed polish whose gate ships polished proposedText',
      },);
    }
    if ((!changed) && (gate.ships === 'polished')) {
      throw new ArtifactParseError({
        path,
        reason: 'unchanged polish whose gate keeps base',
      },);
    }
  }
  return {
    kind: 'settled',
    baseText,
    proposedText,
    text,
    changed,
    refinersHeard: requireStringList({
      value: record.refinersHeard,
      path: `${path}.refinersHeard`,
    },),
    contributors: requireStringList({
      value: record.contributors,
      path: `${path}.contributors`,
    },),
    roundCount: requireCount({
      value: record.roundCount,
      path: `${path}.roundCount`,
    },),
    ...((gateReading.kind === 'present') ? { gate: gateReading.gate, } : {}),
    ...(reviewRequired
      ? {
        review: parseNaturalnessReview({
          value: record.review,
          path: `${path}.review`,
          finalText: text,
          correctionChainRequired,
          everyBodyBlockReviewed,
          quorumBasisRequired,
          lastSeatGrace,
        },),
      }
      : {}),
    findings: requireStringList({
      value: record.findings,
      path: `${path}.findings`,
    },),
  };
}

//endregion Artifact consolidation polish read
