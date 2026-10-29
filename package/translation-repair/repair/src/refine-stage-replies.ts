import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  applyPatchOperations,
  type PatchOperation,
  type PatchRejection,
  restoreTypography,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import type { Candidate, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import { gateParagraphRewrite, } from './inspect-paragraph.ts';
import {
  type EditableEnvelope,
  isUnreadableReason,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type RefineReportWire,
  type RefineResolution,
  resolveRefineRewrites,
} from './refine-wire.ts';
import type { HeardVoice, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Refinement replies
// What one heard rewriter's reply comes to before the judges see it: bound to
// the sheet's paragraphs, read by the atom gate, applied to the slice. Split
// out of `refine-stage.ts` for the file-length cap, with the findings each of
// those steps owes the stage.

/**
 One heard rewriter's reply after the resolver bound it to the sheet and the
 atom gate read each operation.

 @example
 ```ts
 const reply: ResolvedReply = { modelId, resolution, passed: [], refusals: [], };
 ```
 */
export type ResolvedReply = {
  /**
   Rewriter that sent the reply.
   */
  readonly modelId: RosterModelId;

  /**
   Operations the reply's rewrites bound to, and what the resolver dropped
   or folded on the way.
   */
  readonly resolution: RefineResolution;

  /**
   Operations whose replacement carried every protected atom through
   unchanged and in order, quote style restored, in wire order.
   */
  readonly passed: readonly PatchOperation[];

  /**
   One finding per operation the gate refused, credited to the rewriter and
   naming the paragraph and the kind of refusal, in wire order.
   */
  readonly refusals: readonly string[];
};

/**
 What one rewriter's passed operations came to once applied to the slice.

 @example
 ```ts
 const applied: AppliedReply = { modelId, candidates: [], rejections: [], };
 ```
 */
export type AppliedReply = {
  /**
   Rewriter whose operations these were.
   */
  readonly modelId: RosterModelId;

  /**
   The rewriter's whole-slice proposal, absent where no operation applied.
   */
  readonly candidates: readonly Candidate<string>[];

  /**
   One finding per operation the patch refused, credited to the rewriter and
   naming the paragraph, in wire order.
   */
  readonly rejections: readonly string[];
};

/**
 The words of a reason or detail before its first parenthetical, which is
 where the gate and the patch quote what they compared.

 Cut as the editor lane cuts its rejections into kinds
 (`repair-editor-stage.ts`), so a finding carries no atom value: a number, a
 link destination or a foreign run of the paragraph stays in the gate's log
 line and out of the artifact. A changed atom keeps its position
 (`protected atom 2 changed`), and an inspection refusal loses its reason
 (`candidate rejected`); the log line carries both whole.

 @param detail - gate's account of a refusal, or the patch's reason

 @returns Detail's words before its first parenthetical, the whole detail
 where it has none

 @example
 ```ts
 kindOf({ detail: 'protected atom count changed (1 to 0)', },);
 // => 'protected atom count changed'
 ```
 */
function kindOf({ detail, }: { readonly detail: string; },): string {
  return nonNullishOrThrow(detail.split(' (',)[0],);
}

/**
 Binds one heard reply to real paragraphs and reads each operation through the
 atom gate.

 @param voice - heard rewriter and its reply

 @param envelopes - sheet's paragraphs in prompt numbering order

 @param repairedText - `T1`, whose quote style a rewrite is restored to

 @param definitions - link and footnote definitions from the whole document,
 so a paragraph's references resolve during gating

 @param l - stage logger, which carries the gate's detail

 @returns Reply's passed operations and finding for each refusal

 @example
 ```ts
 const reply = resolveReply({ voice, envelopes, repairedText, definitions, l, },);
 ```
 */
export function resolveReply(
  {
    voice,
    envelopes,
    repairedText,
    definitions,
    l,
  }: {
    readonly voice: HeardVoice<RefineReportWire>;
    readonly envelopes: readonly EditableEnvelope[];
    readonly repairedText: string;
    readonly definitions: string;
    readonly l: Logger;
  },
): ResolvedReply {
  /**
   Operations the reply's rewrites bound to, and what the resolver
   dropped or folded on the way.
   */
  const resolution = resolveRefineRewrites({
    wire: voice.value,
    envelopes,
  },);

  /**
   Each operation the gate passed, quote style restored, or the finding
   for its refusal.
   */
  const judged = resolution
    .operations
    .map(function judge(operation,): PatchOperation | string {
      /**
       Paragraph this operation replaces, present by the resolver's
       own contract: `resolveRefineRewrites` binds an operation only
       to an envelope it found.
       */
      const envelope = nonNullishOrThrow(envelopes
        .find(function matches(candidate,) {
          return candidate.envelopeId === operation.envelopeId;
        },),);

      /**
       Replacement as it will ship, quote style restored, so the gate
       reads the shipped bytes rather than text a later pass alters.
       */
      const candidate = restoreTypography({
        replacement: operation.newText,
        replaced: envelope.baseText,
        convention: repairedText,
        definitions,
      },);

      /**
       Structural verdict over the proposed replacement.
       */
      const verdict = gateParagraphRewrite({
        base: envelope.baseText,
        candidate,
        definitions,
      },);
      if (verdict.kind === 'preserved') {
        return {
          ...operation,
          newText: candidate,
        };
      }
      // THE DETAIL GOES TO THE LOG ALONE, since it quotes the atoms it
      // compared; the finding names the paragraph by its number on the
      // sheet, as the resolver's findings do, and the refusal by kind.
      l.info(`${voice.modelId}: ${verdict.detail}`,);
      return `${voice.modelId}: refine-atom-gate-refused (paragraph ${
        String(envelopes
          .indexOf(envelope,)
          + 1,)
      }, ${kindOf({ detail: verdict.detail, },)})`;
    },);
  return {
    modelId: voice.modelId,
    resolution,
    passed: judged.filter(function wasPassed(outcome,): outcome is PatchOperation {
      return (typeof outcome) !== 'string';
    },),
    refusals: judged.filter(function wasRefused(outcome,): outcome is string {
      return (typeof outcome) === 'string';
    },),
  };
}

/**
 The finding for one operation the patch refused.

 The refusal this lane reaches is `unchanged-region`, a rewrite that
 comes out as the paragraph it replaces. Its operations are bound to the
 sheet's own envelopes by the resolver (so the envelope is known, claimed once
 and carries its own hash), the envelopes are derived from the very text the
 patch is applied to (so the region cannot have drifted), and the patch runs
 with preservation skipped (so no preservation refusal exists). A bounded probe
 over generated documents reached no other reason.

 The patch also refuses a text no grammar reads, which the atom gate has
 already refused for a rewrite read alone, so a rewrite reaches it only where
 the two readings disagree; it is named as an unreadable rewrite and never
 ends the stage.
 Any other reason is a broken invariant and throws.

 @param modelId - rewriter whose operation was refused

 @param rejection - patch's refusal

 @param envelopes - sheet's paragraphs in prompt numbering order

 @returns Refusal credited to its rewriter, naming the paragraph as the
 sheet numbers it

 @throws {@link Error} when the reason is any other than `unchanged-region`
 or an unreadable replacement, or the operation names no paragraph of the sheet

 @example
 ```ts
 const finding = rejectionFinding({ modelId, rejection, envelopes, },);
 ```
 */
function rejectionFinding(
  {
    modelId,
    rejection,
    envelopes,
  }: {
    readonly modelId: RosterModelId;
    readonly rejection: PatchRejection;
    readonly envelopes: readonly EditableEnvelope[];
  },
): string {
  /**
   The reason without the detail it may quote.
   */
  const kind = kindOf({ detail: rejection.reason, },);
  /**
   Whether the patch refused the rewrite for the text it would leave.
   */
  const unreadable = isUnreadableReason({ reason: rejection.reason, },);
  if ((kind !== 'unchanged-region') && (!unreadable)) {
    throw new Error(
      `unreachable: the patch refused ${modelId}'s rewrite as ${kind}, but this lane binds its operations `
        + 'to its own envelopes, applies them to the text those envelopes came from with preservation '
        + 'skipped, and so can be refused only as unchanged-region or as a text no grammar reads',
    );
  }

  /**
   Envelope the refused operation named.
   */
  const { envelopeId: targetId, } = rejection.operation;

  /**
   Zero-based place of the paragraph on the sheet.
   */
  const place = envelopes
    .findIndex(function isTarget(envelope,): boolean {
      return envelope.envelopeId === targetId;
    },);
  if (place === (-1)) {
    throw new Error(
      `unreachable: the patch refused ${modelId}'s rewrite of a paragraph the sheet does not hold, `
        + 'though the resolver binds an operation only to a paragraph of the sheet',
    );
  }
  return unreadable
    ? `${modelId}: refine-unreadable-rewrite (paragraph ${String(place + 1,)})`
    : `${modelId}: refine-unchanged-rewrite (paragraph ${String(place + 1,)})`;
}

/**
 Applies one rewriter's passed operations to the slice and names each the
 patch refused.

 @param reply - rewriter's reply after the atom gate

 @param repairedText - `T1`, the text the operations apply to

 @param envelopes - eligible paragraphs of `repairedText`

 @returns Rewriter's proposal, when any operation applied, and a finding
 for each refusal

 @throws {@link Error} when the patch refuses an operation for a reason this
 lane cannot reach

 @example
 ```ts
 const applied = applyReply({ reply, repairedText, envelopes, },);
 ```
 */
export function applyReply(
  {
    reply,
    repairedText,
    envelopes,
  }: {
    readonly reply: ResolvedReply;
    readonly repairedText: string;
    readonly envelopes: readonly EditableEnvelope[];
  },
): AppliedReply {
  /**
   Operations of this reply the atom gate passed.
   */
  const { passed, } = reply;
  if (passed.length === 0) {
    return {
      modelId: reply.modelId,
      candidates: [],
      rejections: [],
    };
  }

  /**
   This rewriter's whole-slice proposal through the deterministic gate.
   */
  const patch = applyPatchOperations({
    targetText: repairedText,
    envelopes,
    operations: passed,
    // EXEMPT, stated rather than defaulted. This lane rewrites a whole
    // paragraph for naturalness and has no accepted-issue quotes to license
    // that, so enforcing preservation here would reject exactly the work
    // the lane exists to do.
    preservation: { mode: 'skip', },
  },);
  return {
    modelId: reply.modelId,
    candidates: (patch.applied
      .length
      === 0)
      ? []
      : [
        {
          producer: {
            kind: 'model',
            modelId: reply.modelId,
          },
          value: patch.patchedText,
          rendered: patch.patchedText,
        },
      ],
    rejections: patch.rejected
      .map(function toFinding(rejection,): string {
        return rejectionFinding({
          modelId: reply.modelId,
          rejection,
          envelopes,
        },);
      },),
  };
}

/**
 Replies in the order their rewriters sit on the roster, so what the stage
 records against them never depends on who answered first.

 @param replies - replies in the order they were heard

 @param roster - rewriters as seated

 @returns Same replies in roster order

 @example
 ```ts
 const ordered = inRosterOrder({ replies, roster: refinerModelIds, },);
 ```
 */
export function inRosterOrder<ReplyT extends { readonly modelId: RosterModelId; },>(
  {
    replies,
    roster,
  }: {
    readonly replies: readonly ReplyT[];
    readonly roster: readonly RosterModelId[];
  },
): readonly ReplyT[] {
  return replies.toSorted(function byRoster(
    left,
    right,
  ): number {
    return roster.indexOf(left.modelId,)
      - roster.indexOf(right.modelId,);
  },);
}

//endregion Refinement replies
