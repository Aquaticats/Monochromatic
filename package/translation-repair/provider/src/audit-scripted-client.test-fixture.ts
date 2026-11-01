import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '../dist/final/node/index.mjs';
import { replyWith, } from './scripted-reply-outcome.test-fixture.ts';

//region Audit scripted client
// A CLIENT ANSWERING EVERY RENDERING AUDITOR WITH ONE SCRIPTED REPORT, chosen
// by model id, for the cases that run the audit without a live provider.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The reply goes through the live request's
// own wire guard, so a fixture that stopped matching the wire fails here.

/**
 One claimed defect, in the wire's own fields.

 @example
 ```ts
 const finding: AuditWireFinding = { ...POLARITY_FINDING, reason: 'again', };
 ```
 */
type AuditWireFinding = {
  /**
   Category from the closed vocabulary, or any text the screen should drop.
   */
  readonly category: string;

  /**
   Span of the original identifying the occurrence.
   */
  readonly sourceLocator: string;

  /**
   Smallest span of the original carrying the change.
   */
  readonly sourceFocus: string;

  /**
   Span of the candidate identifying the occurrence.
   */
  readonly candidateLocator: string;

  /**
   Smallest span of the candidate carrying the change.
   */
  readonly candidateFocus: string;

  /**
   What the auditor says the spans amount to.
   */
  readonly reason: string;
};

/**
 One auditor's whole answer, in the wire's own fields.

 @example
 ```ts
 const report: AuditWireReport = { verdict: 'no-defect-found', findings: [], };
 ```
 */
export type AuditWireReport = {
  /**
   Overall verdict.
   */
  readonly verdict: string;

  /**
   Claims, empty when none.
   */
  readonly findings: readonly AuditWireFinding[];
};

/**
 Answer of an auditor that found nothing.
 */
export const QUIET_AUDIT_REPORT: AuditWireReport = {
  verdict: 'no-defect-found',
  findings: [],
};

/**
 Claim that the flipped rendering reverses the denial, anchored on the planted
 span in both texts.
 */
export const POLARITY_FINDING: AuditWireFinding = {
  category: 'altered-polarity',
  sourceLocator: '她们不吃罐头',
  sourceFocus: '不吃罐头',
  candidateLocator: 'They eat canned food',
  candidateFocus: 'eat canned food',
  reason: 'the original denies it and the candidate asserts it',
};

/**
 The same claim quoting a narrower span, so it overlaps the planted span
 without matching it.
 */
export const NARROW_POLARITY_FINDING: AuditWireFinding = {
  ...POLARITY_FINDING,
  sourceFocus: '不吃',
  candidateFocus: 'They eat',
};

/**
 Claim that the candidate adds the planted wording, resting on the candidate
 alone.
 */
export const ADDITION_FINDING: AuditWireFinding = {
  category: 'unsupported-addition',
  sourceLocator: '',
  sourceFocus: '',
  candidateLocator: 'They eat canned food',
  candidateFocus: 'eat canned food',
  reason: 'nothing in the original says it',
};

/**
 Claim about the sleeping place, nowhere near the planted span, resting on the
 original alone.
 */
export const ELSEWHERE_FINDING: AuditWireFinding = {
  category: 'omission',
  sourceLocator: '那只虎斑猫睡在窗台上',
  sourceFocus: '窗台',
  candidateLocator: '',
  candidateFocus: '',
  reason: 'the sleeping place is not carried over',
};

/**
 Claim in a category the vocabulary does not have, which the screen drops.
 */
export const UNKNOWN_CATEGORY_FINDING: AuditWireFinding = {
  ...POLARITY_FINDING,
  category: 'bogus',
};

/**
 Refuses a text call, which this scripted protocol never makes.

 @throws {@link Error} on any invocation
 */
function unexpectedText(): never {
  throw new Error('chatText unused by the audit',);
}

/**
 Refuses a quota read, which fixtures cannot answer.

 @throws {@link Error} on any invocation
 */
function unexpectedQuotas(): never {
  throw new Error('quotas unused by the audit',);
}

/**
 Client answering each auditor with the report the script names for it.

 @param reportFor - report each model casts, chosen by model id

 @param silent - models whose voice is always lost, none by default

 @param asked - shared log of every model id the client was asked, in order

 @returns Client the audit calls

 @example
 ```ts
 const client = auditScriptedClient({ reportFor: function quiet() { return QUIET_AUDIT_REPORT; }, asked: [], },);
 ```
 */
export function auditScriptedClient(
  {
    reportFor,
    asked,
    silent = [],
  }: {
    readonly reportFor: (modelId: string,) => AuditWireReport;
    readonly silent?: readonly string[];
    readonly asked: string[];
  },
): ModelClient {
  return {
    chatText: unexpectedText,
    /**
     Answers with the scripted report, through the wire guard.
     */
    chatJson: function chatJson<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
      asked.push(request.modelId,);

      if (silent.includes(request.modelId,)) {
        return Promise.resolve({
          kind: 'schema-mismatch',
          rawText: '',
          detail: 'scripted silence',
        },);
      }

      return Promise.resolve(replyWith({
        report: reportFor(request.modelId,),
        request,
      },),);
    },
    quotas: unexpectedQuotas,
  };
}

//endregion Audit scripted client
