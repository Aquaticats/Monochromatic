import type { ProposalValidity, } from './consolidate-validity-floor.ts';
import type { ConsolidationSubject, } from './consolidate-settle.ts';
import type { SliceValidation, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Consolidate settle context
// WHAT BOTH ROUNDS OF THE SETTLEMENT ARE HANDED BESIDE THE SLATE, split out
// of `consolidate-settle.ts` at the line cap on 2026-09-04 when the ineligible
// standing rule joined that file. Nothing here decides anything: it shapes the
// subject's optional fields the way the sheets and the keys take them, and
// strips proposals to the verdicts a run may record.

/**
 One voice's structural verdict WITHOUT its text, which is what a record of
 this stage may carry.

 The proposals themselves are corpus renderings and do not belong in a
 settlement a run writes out. Who was refused and why does: run 8 carried 7
 invalid candidates across slices that all shipped normally, and nothing
 downstream could see them.

 @example
 ```ts
 const verdict: ProposalVerdict = { modelId: 'hf:cat/Cat-A', kind: 'valid', findings: [], };
 ```
 */
export type ProposalVerdict = {
  /**
   Voice that wrote the proposal.
   */
  readonly modelId: string;

  /**
   What the structural guard made of it.
   */
  readonly kind: SliceValidation['kind'];

  /**
   Why it was refused, empty when it was not.
   */
  readonly findings: readonly string[];
};

/**
 Front matter identity and cited references as the slate judge and the gate
 take them, each absent when the pair has none.

 @example
 ```ts
 const identity: SettlementIdentity = { identityContext: 'name: Mimi', };
 ```
 */
export type SettlementIdentity = {
  readonly identityContext?: string;
  readonly referenceContext?: string;
  readonly archiveDisputeNote?: string;
  readonly lifeEnded?: boolean;
  readonly subjectNames?: readonly string[];
};

/**
 Identity and evidence as the two rounds take them.

 SPREAD PER FIELD RATHER THAN PASSED WHOLE. An absent identity is absent
 rather than declared empty, and an absent picture and an empty picture are
 the same state and must render as no heading at all, since a heading
 promising readings and carrying none reads as a picture nobody could make
 sense of.

 ONE VALUE FEEDS THE SHEET AND THE KEY, which is the judging window's lesson stated in
 `translate-document.ts` as well: a key that did not name the evidence would
 let a narrow run's answer be resumed for a wide one, and nothing anywhere
 would report the two as different questions.

 @param subject - slice in the archive's terms

 @returns Identity fields and evidence fields, each present only when carried

 @example
 ```ts
 const { identity, evidence, } = settlementContextOf({ subject, },);
 ```
 */
export function settlementContextOf(
  { subject, }: { readonly subject: ConsolidationSubject; },
): {
  readonly identity: SettlementIdentity;
  readonly evidence: {
    readonly pictureContext?: string;
    readonly neighbouringSourceText?: string;
    readonly neighbouringIncumbentText?: string;
  };
} {
  /**
   Identity as the two rounds take it.
   */
  const identity: SettlementIdentity = {
    ...((subject.identityContext === undefined) ? {} : { identityContext: subject.identityContext, }),
    ...((subject.referenceContext === undefined) ? {} : { referenceContext: subject.referenceContext, }),
    ...((subject.archiveDisputeNote === undefined) ? {} : { archiveDisputeNote: subject.archiveDisputeNote, }),
    ...((subject.lifeEnded === undefined) ? {} : { lifeEnded: subject.lifeEnded, }),
    ...((subject.subjectNames === undefined) ? {} : { subjectNames: subject.subjectNames, }),
  };

  /**
   Words about this passage that are not the passage: what its pictures were
   read to say and what stands either side of it.
   */
  const evidence = {
    ...(((subject.pictureContext === undefined) || (subject.pictureContext === ''))
      ? {}
      : { pictureContext: subject.pictureContext, }),
    ...(((subject.neighbouringSourceText === undefined) || (subject.neighbouringSourceText === ''))
      ? {}
      : { neighbouringSourceText: subject.neighbouringSourceText, }),
    ...(((subject.neighbouringIncumbentText === undefined)
      || (subject.neighbouringIncumbentText === ''))
      ? {}
      : { neighbouringIncumbentText: subject.neighbouringIncumbentText, }),
  };

  return {
    identity,
    evidence,
  };
}

/**
 Every voice's verdict without its text, which is what a run may record.

 @param validity - each voice's structural verdict, keyed by model id

 @returns Verdicts in the same order, refusals carrying their findings

 @example
 ```ts
 const verdicts = verdictsOf({ validity, },);
 ```
 */
export function verdictsOf(
  { validity, }: { readonly validity: readonly ProposalValidity[]; },
): readonly ProposalVerdict[] {
  return validity.map(
    function toVerdict(
      {
        modelId,
        validation,
      }: ProposalValidity,
    ): ProposalVerdict {
      return {
        modelId,
        kind: validation.kind,
        findings: (validation.kind === 'invalid') ? validation.findings : [],
      };
    },
  );
}

//endregion Consolidate settle context
