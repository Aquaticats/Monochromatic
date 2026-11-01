import type { AbsoluteNaturalnessReviewOutcome, } from '@monochromatic-dev/translation-repair-archive/ts';
import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { ConsolidationPolishGateOutcome, } from './consolidation-polish-gate-stage.ts';
import type { RepairJudgedRound, } from '@monochromatic-dev/translation-repair-assembly/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Consolidation naturalness model

/**
 Model roles and document facts needed by final body polish.

 @example
 ```ts
 const config: ConsolidationPolishConfig = { refinerModelIds, judgeModelIds, gateModelIds, declaredNames: [], definitions: '', };
 ```
 */
export type ConsolidationPolishConfig = {
  /**
   Rewriters proposing idiomatic paragraph replacements.
   */
  readonly refinerModelIds: readonly RosterModelId[];

  /**
   Judges selecting rewrite slate winner.
   */
  readonly judgeModelIds: readonly RosterModelId[];

  /**
   Fidelity-first roster deciding whether selected rewrite may ship.
   */
  readonly gateModelIds: readonly RosterModelId[];

  /**
   Declared target name forms protected from deletion.
   */
  readonly declaredNames: readonly string[];

  /**
   The declared forms among them that name the subject, which a rewrite may
   swap for one another; absent when preparation named none.
   */
  readonly declaredSubjectForms?: readonly string[];

  /**
   Name pairs the front matter declares, read by the publication rule a
   polish is validated under (class one hundred fourteen).
   */
  readonly declaredNamePairs?: readonly DeclaredNamePair[];

  /**
   Link and footnote definitions used by rewrite guards.
   */
  readonly definitions: string;
};

/**
 One rejected-input to gated-correction digest transition.

 @example
 ```ts
 const correction: ConsolidationNaturalnessCorrectionAudit = { inputDigest, findingsDigest, gatedTextDigest, };
 ```
 */
export type ConsolidationNaturalnessCorrectionAudit = {
  /**
   Exact rejected candidate supplied to correction.
   */
  readonly inputDigest: string;

  /**
   Canonical structured findings supplied to correction.
   */
  readonly findingsDigest: string;

  /**
   Exact post-fidelity-gate correction supplied to next review.
   */
  readonly gatedTextDigest: string;
};

/**
 Absolute review rounds binding publication approval to exact final wording.

 @example
 ```ts
 const review: ConsolidationNaturalnessAudit = { correctionCount: 0, corrections: [], rounds: [], confirmations: [] };
 ```
 */
export type ConsolidationNaturalnessAudit = {
  /**
   Dedicated corrective generations bought after initial rejection.
   */
  readonly correctionCount: number;

  /**
   Digest chain binding each accepted correction transition.
   */
  readonly corrections: readonly ConsolidationNaturalnessCorrectionAudit[];

  /**
   Decisive whole-passage reviews, one per generated candidate.
   */
  readonly rounds: readonly AbsoluteNaturalnessReviewOutcome[];

  /**
   Earlier acceptable readings before each decisive same-candidate review.
   */
  readonly confirmations: readonly AbsoluteNaturalnessReviewOutcome[];
};

/**
 Auditable final polish decision for one consolidated slice.

 @example
 ```ts
 const polish: ConsolidationPolish = { kind: 'not-run', reason: 'front-matter', };
 ```
 */
export type ConsolidationPolish =
  | {
    /**
     No naturalness stage was applicable or configured.
     */
    readonly kind: 'not-run';

    /**
     Why no body polish was bought.
     */
    readonly reason: 'front-matter' | 'not-configured' | 'unsafe-baseline';
  }
    | {
    /**
     Naturalness stage examined approved base.
     */
      readonly kind: 'settled';

    /**
     Already-approved text before naturalness work.
     */
      readonly baseText: string;

    /**
     Selected rewrite proposal before final fidelity gate.
     */
      readonly proposedText: string;

    /**
     Final text after conservative gate.
     */
      readonly text: string;

    /**
     Whether final text differs from approved base.
     */
      readonly changed: boolean;

    /**
     Rewriters heard with usable answer.
     */
      readonly refinersHeard: readonly RosterModelId[];

    /**
     Models contributing selected proposal.
     */
      readonly contributors: readonly RosterModelId[];

    /**
     Naturalness selection rounds retained for audit.
     */
      readonly rounds: readonly RepairJudgedRound[];

    /**
     Final fidelity and naturalness gate, absent when no rewrite survived.
     */
      readonly gate?: ConsolidationPolishGateOutcome;

    /**
     Absolute whole-passage approval bound to final text.
     */
      readonly review: ConsolidationNaturalnessAudit;

    /**
     Findings from proposal, validation, gate and absolute review.
     */
      readonly findings: readonly string[];
    };
// NO UNSETTLED POLISH (ledger B52): a third kind recorded a correction loop
// that never earned publication. Nothing has built it since the loop went on
// 2026-09-01; artifacts never stored it and the consolidation cache refused to
// keep it, so no record carries it, and its guards went with it.

//endregion Consolidation naturalness model
