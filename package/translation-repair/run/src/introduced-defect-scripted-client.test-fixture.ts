import type { ScreenedDefectClaim, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { RUN_MODELS, } from '../dist/final/node/index.mjs';
import { replyWith, } from '@monochromatic-dev/translation-repair-provider/ts/scripted-reply-outcome.test-fixture.ts';

//region Introduced defect scripted client
// A CLIENT ANSWERING EVERY INTRODUCED-DEFECT PROBER WITH ONE SCRIPTED CHECK ON
// REGION ONE, chosen by model id, for the cases that run a probe over a single
// region without a live provider.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The reply goes through the live request's
// own wire guard, so a fixture that stopped matching the wire fails here.

/**
 One check a prober casts on the region, in the wire's own fields.

 @example
 ```ts
 const check: ProbeCheck = { ...NO_DEFECT_CHECK, verdict: 'uncertain', };
 ```
 */
export type ProbeCheck = {
  /**
   Verdict, one of the wire's three.
   */
  readonly verdict: string;

  /**
   Defect class in the prober's words, empty for a negative verdict.
   */
  readonly category: string;

  /**
   Severity as claimed, empty for a negative verdict.
   */
  readonly severity: string;

  /**
   Wording quoted from the replacement, for added damage.
   */
  readonly evidence: string;

  /**
   Wording quoted from the replaced text, for dropped content.
   */
  readonly omittedText: string;

  /**
   Why the prober says the replaced text lacked the defect, under the wire's own key (ledger B762).
   */
  readonly verdictReason: string;
};

/**
 Check of a prober that looked and found nothing.
 */
export const NO_DEFECT_CHECK: ProbeCheck = {
  verdict: 'no-introduced-defect-found',
  category: '',
  severity: '',
  evidence: '',
  omittedText: '',
  verdictReason: '',
};

/**
 Check claiming the edit dropped the second clause of the nap edit, which the
 replaced text carries and the replacement lacks: screened as removal
 corroborated.
 */
export const OMISSION_CHECK: ProbeCheck = {
  verdict: 'introduced-defect',
  category: 'accuracy/omission',
  severity: 'major',
  evidence: '',
  omittedText: 'she wakes at dusk',
  verdictReason: 'the clause is gone',
};

/**
 Check claiming the replacement added wording that it carries and the
 replaced text lacks: screened as corroborated.
 */
export const ADDITION_CHECK: ProbeCheck = {
  verdict: 'introduced-defect',
  category: 'accuracy/addition',
  severity: 'minor',
  evidence: 'The cat sleeps.',
  omittedText: '',
  verdictReason: 'the wording is new',
};

/**
 Claim a screened omission leaves, for one prober.

 @param modelId - prober that made the claim

 @returns Claim as the screen records it

 @example
 ```ts
 const claim = omissionClaimBy({ modelId: 'cat-house/tabbyscribe-2', },);
 ```
 */
export function omissionClaimBy({ modelId, }: { readonly modelId: string; },): ScreenedDefectClaim {
  return {
    modelId,
    category: OMISSION_CHECK.category,
    severity: OMISSION_CHECK.severity,
    evidence: '',
    omittedText: OMISSION_CHECK.omittedText,
    reason: OMISSION_CHECK.verdictReason,
    admissibility: 'removal-corroborated',
  };
}

/**
 Claims every prober of the roster makes when each casts the omission check.

 @returns One claim per roster prober, in roster order

 @example
 ```ts
 const claims = omissionClaimsOfRoster();
 ```
 */
export function omissionClaimsOfRoster(): readonly ScreenedDefectClaim[] {
  /**
   Probers of the roster.
   */
  const { checkerModelIds, } = RUN_MODELS;

  return checkerModelIds.map(function claimBy(modelId,): ScreenedDefectClaim {
    return omissionClaimBy({ modelId, },);
  },);
}

/**
 Check claiming added wording that the replaced text already carried:
 screened as contradicted.
 */
export const CONTRADICTED_CHECK: ProbeCheck = {
  verdict: 'introduced-defect',
  category: 'accuracy/addition',
  severity: 'minor',
  evidence: 'The cat',
  omittedText: '',
  verdictReason: 'the wording is new',
};

/**
 Check claiming damage and quoting nothing: screened as unanchored.
 */
export const UNANCHORED_CHECK: ProbeCheck = {
  verdict: 'introduced-defect',
  category: 'style/awkward-phrasing',
  severity: 'minor',
  evidence: '',
  omittedText: '',
  verdictReason: 'it reads badly',
};

/**
 Refuses a text call, which this scripted protocol never makes.

 @throws {@link Error} on any invocation
 */
function unexpectedText(): never {
  throw new Error('chatText unused by the probe',);
}

/**
 Refuses a quota read, which fixtures cannot answer.

 @throws {@link Error} on any invocation
 */
function unexpectedQuotas(): never {
  throw new Error('quotas unused by the probe',);
}

/**
 Client answering each prober with the check the script names for its model.

 @param checkFor - check each model casts on the one region, chosen by model id

 @param asked - shared log of every model id the client was asked, in order

 @param prompts - shared log of the user sheet of every call, in order, when
 the case reads what was sent

 @returns Client the probe calls

 @example
 ```ts
 const client = probeScriptedClient({ checkFor: function none() { return NO_DEFECT_CHECK; }, asked: [], },);
 ```
 */
export function probeScriptedClient(
  {
    checkFor,
    asked,
    prompts = [],
  }: {
    readonly checkFor: (modelId: string,) => ProbeCheck;
    readonly asked: string[];
    readonly prompts?: string[];
  },
): ModelClient {
  return {
    chatText: unexpectedText,
    /**
     Answers with the scripted check on region one, through the wire guard.
     */
    chatJson: function chatJson<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
      asked.push(request.modelId,);

      /**
       Messages the request carried.
       */
      const { messages, } = request;

      /**
       Last message, whose text the case may read.
       */
      const sent = messages.at(-1,);

      /**
       Text of that message, empty when the request carried none.
       */
      const text = (sent === undefined) ? '' : messageText({ message: sent, },);
      prompts.push(text,);
      return Promise.resolve(replyWith({
        report: {
          checks: [
            {
              region: 1,
              ...checkFor(request.modelId,),
            },
          ],
        },
        request,
      },),);
    },
    quotas: unexpectedQuotas,
  };
}

/**
 Builder that hands out the given clients one after another, one per call, so
 a case can see how many clients a run built and what each was asked.

 @param clients - clients to hand out, in order

 @returns The builder, and a reading of how many it has handed out

 @throws {@link Error} from the builder when it is called after the last client

 @example
 ```ts
 const { newClient, built, } = successiveClients({ clients: [first, second,], },);
 ```
 */
export function successiveClients(
  { clients, }: { readonly clients: readonly ModelClient[]; },
): {
  readonly newClient: () => ModelClient;
  readonly built: () => number;
} {
  /**
   Clients handed out so far.
   */
  const handed = { count: 0, };
  return {
    newClient: function newClient(): ModelClient {
      /**
       Client this call is owed.
       */
      const next = clients[handed.count];
      if (next === undefined)
        throw new Error('the run asked for more clients than the case scripted',);
      handed.count += 1;
      return next;
    },
    built: function built(): number {
      return handed.count;
    },
  };
}

//endregion Introduced defect scripted client
