import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  type ChatTextReply,
  type ClaudeAttemptAsk,
  type ModelClient,
  reportSpend,
  type SpendReckoning,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  CLAUDE_COMPLETION_CAP,
  runSpendUsd,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  callCeilingUsd,
  messageChars,
  pricesOf,
  usdText,
} from './editor-fidelity-ceiling.ts';

//region Editor fidelity spend guard
// WHAT HOLDS AN ARM TO `--max-usd`, call by call, on the money the client reports and not on the replies the guard sees.
//
// A POOL OF FORTY TRIALS PRICES PAST ANY AMOUNT, because the ceiling takes the whole completion cap on every ask
// (`editor-fidelity-ceiling.ts`). Refusing to start on that total would refuse every arm of the pool, so the guard asks
// the question at each call instead: would the spend so far, plus the most this call can cost, pass the amount? If it
// would, the call is refused and the arm ends there, with the rows it finished already in their files.
//
// SPEND IS READ OFF THE RUN METER, WHICH IS WHAT THE CLIENT WRITES. Every priced reply of the Claude client is a SPEND
// line that moves the meter by the cost the client priced from the four counts its stream reported, and since the
// client reports abandoned attempts (`claude-abandoned-spend.ts`) every attempt it retried inside a call after a cut
// stream is a line too, at the most it could have been billed. The first guard priced the replies it saw by a second
// rule, which differed from the client's cost on every reply and could not see a retried attempt at all: it bounded
// the replies and not the money.
//
// WHAT NO LINE HOLDS IS COUNTED AT THE CALL'S CEILING, by a SPEND line the guard writes itself, so the meter, the log
// and the guard's total stay one figure. A reply that reports no usage has an unknown cost, and an unknown cost may not
// be assumed small. A call that throws may have been billed for a stream that never reached the caller, so it counts
// unless the error proves no billable body was delivered, which an HTTP error status with no response body does.
//
// THE CLIENT RETRIES INSIDE A CALL, so the guard looks at each attempt and not once per call. The Claude client sends up
// to five attempts for one call, and each that delivers a stream and is cut is billed at the most it could have been.
// When the client is built over the guard (the client builder takes the guard's admit and hands it to the Claude client,
// `claude-attempt-admit.ts`), every attempt is admitted before it is sent: the spend so far plus that attempt's bound
// must stay within the amount, or the attempt is never sent and the arm ends. An attempt that fails is counted once: its
// line when the client wrote one, else, for the last attempt of a call that throws, the call's ceiling. A call that
// threw before any attempt was told never left (a layer above the client rejected it), and costs nothing.
//
// ONE CALL AT A TIME. The attempts told are kept for the call in flight, which holds because the probe asks one row at a
// time (`overlap: 1` in `editor-fidelity-probe-run.ts`). The guard asserts it: a call asked while another is in flight
// throws `ConcurrentGuardCallError` before anything is sent, so a change that overlaps calls fails loudly and does not
// quietly read one call's attempts as another's.
//
// A REPLY WITH NO REQUEST BEHIND IT COSTS NOTHING AND MEASURES NOTHING. A client built over the guard's admit tells the guard
// of each attempt it sends. A call that returns a reply with no attempt told was answered by the prompt reuse layer from an
// earlier reply, in no time, so no line is written for it, at the ceiling or otherwise: on the billed high arm of
// `dd4b15b75c` 62 such replies were written at the ceiling and booked 23.31 USD that was never spent, and the arm ended at 150
// of 264 units. It is no measurement either, since a repeat that is a replay compares an answer with itself, so the guard ends
// the arm with `RepliedWithoutCallError` before the reply reaches a row.
//
// THE REFUSAL ENDS THE ARM, NOT ONE VOICE OF IT. The stage turns a call that throws into a lost voice and asks again, but
// rethrows once the arm's signal has aborted. The guard aborts that signal with the refusal as its reason before it
// throws, so every layer between the call and the probe carries the refusal out, and every later call is refused with the
// same one without reaching the client.

/**
 What a reply reports about the tokens it used.
 */
type ReportedUsage = NonNullable<ChatTextReply['usage']>;

/**
 Thrown when a call is asked of the guard while another is still in flight, which the guard's one-call-at-a-time
 accounting of attempts cannot reckon.

 @example
 ```ts
 throw new ConcurrentGuardCallError();
 ```
 */
export class ConcurrentGuardCallError extends Error {
  /**
   The message is one sentence this class writes.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the error with its fixed message.
   */
  public constructor() {
    super('the spend guard admits one call at a time, and a call was asked while another was in flight; nothing was sent',);
    this.name = 'ConcurrentGuardCallError';
  }
}

/**
 Thrown, and carried by the arm's signal, when a client that tells the guard of each attempt it sends returns a reply for a
 call it told the guard of none: the reply came from the prompt reuse layer and not from the model.

 @example
 ```ts
 throw new RepliedWithoutCallError();
 ```
 */
export class RepliedWithoutCallError extends Error {
  /**
   The message is one sentence this class writes.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the error with its fixed message.
   */
  public constructor() {
    super(
      'a reply reached the spend guard with no request sent behind it: the prompt was answered from reuse, and not by the '
        + 'model (the PROMPT-REUSE line of the log names the model and the digest of the prompt), so the probe will not score '
        + 'it, nothing was booked for it, and the arm ends here',
    );
    this.name = 'RepliedWithoutCallError';
  }
}

/**
 What ended an arm: the guard's refusal of a call, or a reply that no request produced.
 */
type Ending = RepliedWithoutCallError | StatedRefusalError;

/**
 What the guard counted, for the line that ends an arm.

 @example
 ```ts
 const total: GuardTotal = { usd: 0.322, calls: 3, atCeiling: 1, };
 ```
 */
export type GuardTotal = {
  /**
   Dollars on the run meter since the guard was built: the client's lines and the guard's own.
   */
  readonly usd: number;

  /**
   Calls the guard let through.
   */
  readonly calls: number;

  /**
   Calls the guard counted at their ceiling itself, by a line of its own.
   */
  readonly atCeiling: number;
};

/**
 A client held to an amount, which also says what it counted.

 @example
 ```ts
 const { usd, } = client.guardTotal();
 ```
 */
export type GuardedClient = ModelClient & {
  /**
   Reads what the guard has counted so far.
   */
  readonly guardTotal: () => GuardTotal;
};

/**
 The line that says what holds the arm to its amount.

 @param maxUsd - amount from `--max-usd`

 @param benchSpentUsd - amount from `--bench-spent-usd`, what the arms run before this one measured

 @returns Line, without a newline

 @example
 ```ts
 log.info(guardLine({ maxUsd: 2, benchSpentUsd: 0, },),);
 ```
 */
export function guardLine(
  {
    maxUsd,
    benchSpentUsd,
  }: {
    readonly maxUsd: number;
    readonly benchSpentUsd: number;
  },
): string {
  return 'spend guard: each call, and each attempt the client makes inside it, is refused, and the arm ends, when the '
    + `spend so far plus its ceiling would pass --max-usd ${usdText({ usd: maxUsd, },)}; the arms run before this one `
    + `spent --bench-spent-usd ${usdText({ usd: benchSpentUsd, },)}; the spend so far is every SPEND line the client `
    + 'wrote, the attempts it abandoned included, and a call whose last attempt wrote no line and threw, or a reply '
    + 'that reports no usage, is counted at its ceiling';
}

/**
 Decimals the total line prints dollars with, enough to tell the sum of SPEND lines from the total they are compared
 with.
 */
const TOTAL_DECIMALS = 6;

/**
 The line that says what the guard counted when an arm ends, in the grammar the read step totals.

 @param total - what the guard counted

 @returns Line, without a newline

 @example
 ```ts
 log.info(guardTotalLine({ total: client.guardTotal(), },),);
 ```
 */
export function guardTotalLine({ total, }: { readonly total: GuardTotal; },): string {
  /**
   The parts of the total, named for the line.
   */
  const {
    usd,
    calls,
    atCeiling,
  } = total;
  return `GUARD total=${usd.toFixed(TOTAL_DECIMALS,)} calls=${String(calls,)} atCeiling=${String(atCeiling,)}`;
}

/**
 Whether a thrown error proves no billable body was delivered: an HTTP error status that carried no response body.

 @param error - what the call threw

 @returns Whether the call is free

 @example
 ```ts
 const free = deliveredNothing({ error, },);
 ```
 */
function deliveredNothing({ error, }: { readonly error: unknown; },): boolean {
  return (error instanceof ProviderHttpError) && (error.bodyExcerpt === '');
}

/**
 What an exchange did: answered, or threw.

 @example
 ```ts
 const attempt: Attempt<string> = { kind: 'replied', reply: 'meow', };
 ```
 */
type Attempt<ReplyT,> =
  | {
    /**
     The exchange answered.
     */
    readonly kind: 'replied';

    /**
     What it answered.
     */
    readonly reply: ReplyT;
  }
    | {
    /**
     The exchange threw.
     */
      readonly kind: 'threw';

    /**
     What it threw.
     */
      readonly error: unknown;
    };

/**
 Runs an exchange and says what it did as data, so the caller counts a throw before it rethrows.

 @param exchange - inner client's exchange

 @returns Reply, or what the exchange threw

 @example
 ```ts
 const attempt = await attemptOf({ exchange, },);
 ```
 */
async function attemptOf<ReplyT,>(
  { exchange, }: { readonly exchange: () => Promise<ReplyT>; },
): Promise<Attempt<ReplyT>> {
  try {
    return {
      kind: 'replied',
      reply: await exchange(),
    };
  }
  catch (error) {
    return {
      kind: 'threw',
      error,
    };
  }
}

/**
 Wraps a client so no call is made that could take the arm past its amount.

 THE LIMIT OF THE REUSE CHECK: only a client built over the guard's admit tells the guard of its attempts, so only it can
 prove that a reply came with no request behind it. A client given as itself tells the guard nothing, a reply of the
 prompt reuse layer looks to the guard exactly like a reply whose provider wrote no line on the meter, and the guard counts it
 at its ceiling as an unreported cost. Every construction in production passes a builder (`probe-run.ts`), so a reply
 with no request behind it ends the arm there; an ungated client is given by tests alone.

 @param inner - client to wrap, or the builder of one that is handed the guard's admit and passes it to the client
 it builds, which then tells the guard before each attempt it sends and is held to the amount attempt by attempt

 @param maxUsd - most the arm may spend, from `--max-usd`

 @param controller - arm's own, aborted with the refusal as its reason when a call is refused

 @returns Client with every member of the inner one, at run time, its exchanges refused once the spend so far plus the
 call's ceiling would pass the amount (and, where the client tells the guard of its attempts, each attempt's bound),
 and the reader of what it counted

 @throws StatedRefusalError from an exchange whose ceiling plus the spend so far passes the amount, before the inner
 client is reached, and from every exchange after it

 @throws StatedRefusalError from an exchange naming a model with no price table

 @example
 ```ts
 const client = spendGuardedClient({ inner, maxUsd: 2, controller, },);
 ```
 */
export function spendGuardedClient(
  {
    inner,
    maxUsd,
    controller,
  }: {
    readonly inner: ModelClient | ((admit: (attempt: ClaudeAttemptAsk,) => void,) => ModelClient);
    readonly maxUsd: number;
    readonly controller: AbortController;
  },
): GuardedClient {
  /**
   What the meter held when the guard was built, so the spend is the arm's alone.
   */
  const baselineUsd = runSpendUsd({ provider: 'anthropic', },);
  /**
   Calls let through and calls counted at their ceiling.
   */
  const counted = {
    calls: 0,
    atCeiling: 0,
  };
  /**
   What the arm ended with, at most one.
   */
  const endings: Ending[] = [];
  /**
   Whether the client tells the guard of each attempt it sends, which it does when it was built over the guard's admit.
   */
  const gated = (typeof inner) === 'function';
  /**
   What the client has told the guard of the call in flight: how many attempts, and the meter at the start of the latest,
   or at the start of the call where none was told.
   */
  const told = {
    attempts: 0,
    sinceUsd: baselineUsd,
  };
  /**
   Whether a call is in flight, which a second call may not begin beside.
   */
  const flight = { inFlight: false, };

  /**
   Dollars the arm has spent so far.

   @returns What the meter gained since the guard was built

   @example
   ```ts
   const spentUsd = spentSoFar();
   ```
   */
  function spentSoFar(): number {
    return runSpendUsd({ provider: 'anthropic', },) - baselineUsd;
  }

  /**
   Marks a call in flight until the returned handle is disposed.

   @returns Handle whose disposal frees the guard for the next call

   @throws ConcurrentGuardCallError when another call is still in flight

   @example
   ```ts
   using _flight = enterFlight();
   ```
   */
  function enterFlight(): Disposable {
    if (flight.inFlight)
      throw new ConcurrentGuardCallError();
    flight.inFlight = true;
    return {
      [Symbol.dispose](): void {
        flight.inFlight = false;
      },
    };
  }

  /**
   Refuses anything once the arm has ended.

   @throws StatedRefusalError the refusal an earlier call or attempt ended the arm with

   @throws RepliedWithoutCallError ending that an earlier reply with no request behind it gave the arm
   */
  function throwStanding(): void {
    /**
     What an earlier call ended the arm with, absent while none has.
     */
    const [standing,] = endings;
    if (standing !== undefined)
      throw standing;
  }

  /**
   Ends the arm when what is about to be sent could take it past its amount.

   @param modelId - model the exchange names

   @param ceilingUsd - most the call or the attempt can cost

   @param subject - what is about to be sent, for the refusal's words

   @throws StatedRefusalError when the spend so far plus the ceiling passes the amount

   @example
   ```ts
   refuseOver({ modelId, ceilingUsd: 0.322, subject: 'call', },);
   ```
   */
  function refuseOver(
    {
      modelId,
      ceilingUsd,
      subject,
    }: {
      readonly modelId: string;
      readonly ceilingUsd: number;
      readonly subject: 'attempt' | 'call';
    },
  ): void {
    /**
     What the arm has spent so far.
     */
    const spentUsd = spentSoFar();
    if ((spentUsd + ceilingUsd) <= maxUsd)
      return;
    /**
     The refusal that ends the arm.
     */
    const refusal = new StatedRefusalError({
      says: `the editor fidelity probe will not ask ${modelId}: the spend so far is ${
        usdText({ usd: spentUsd, },)
      } USD, this ${subject} can cost ${usdText({ usd: ceilingUsd, },)} USD at most, and together they pass `
        + `--max-usd ${usdText({ usd: maxUsd, },)}; the arm ends here, and the rows it finished stay in their files`,
    },);
    endings.push(refusal,);
    controller.abort(refusal,);
    throw refusal;
  }

  /**
   Lets a call through, or ends the arm.

   @param modelId - model the call names

   @param messages - what the call sends

   @returns Most this call can cost and prompt size it was reckoned from

   @throws StatedRefusalError when the call could take the arm past its amount, or an earlier one did

   @example
   ```ts
   const ceiling = admit({ modelId, messages, },);
   ```
   */
  function admit(
    {
      modelId,
      messages,
    }: {
      readonly modelId: Parameters<typeof pricesOf>[0]['cardId'];
      readonly messages: Parameters<typeof messageChars>[0]['messages'];
    },
  ): {
    readonly usd: number;
    readonly promptTokens: number;
  } {
    throwStanding();
    /**
     Prompt size the ceiling is reckoned from.
     */
    const promptTokens = messageChars({ messages, },);
    /**
     The most this call can cost.
     */
    const ceilingUsd = callCeilingUsd({
      prices: pricesOf({ cardId: modelId, },),
      promptTokens,
    },);
    refuseOver({
      modelId,
      ceilingUsd,
      subject: 'call',
    },);
    return {
      usd: ceilingUsd,
      promptTokens,
    };
  }

  /**
   Lets an attempt of the call in flight through, or ends the arm: the client tells the guard before it sends each one,
   the first and every retry, and what this throws ends the call there with the attempt unsent.

   @param label - card the call is named by

   @param bound - most the attempt can be billed

   @throws StatedRefusalError when the attempt could take the arm past its amount, or an earlier call or attempt did

   @example
   ```ts
   admitAttempt({ label, bound, },);
   ```
   */
  function admitAttempt(
    {
      label,
      bound,
    }: ClaudeAttemptAsk,
  ): void {
    throwStanding();
    refuseOver({
      modelId: label,
      ceilingUsd: bound.usd,
      subject: 'attempt',
    },);
    told.attempts += 1;
    told.sinceUsd = runSpendUsd({ provider: 'anthropic', },);
  }

  /**
   Whether a throw costs the arm nothing more than the client already wrote.

   @param error - what the call threw

   @returns True for the guard's own refusal, for a call that never left (the client was told of no attempt), for a call
   whose last attempt the client wrote a line for, and for an error that proves no billable body was delivered

   @example
   ```ts
   const free = throwIsFree({ error, },);
   ```
   */
  function throwIsFree({ error, }: { readonly error: unknown; },): boolean {
    if (((error instanceof StatedRefusalError) || (error instanceof RepliedWithoutCallError)) && endings.includes(error,))
      return true;
    // A CALL THAT NEVER LEFT, rejected by a layer above the client (a retained prompt claim, a withheld seat), sent no
    // attempt, so nothing was billed. Only a client that tells the guard of its attempts can say so.
    if (gated && (told.attempts === 0))
      return true;
    // The client writes the line of an attempt it saw cut, so the last attempt is counted once, by that line.
    if (gated && (runSpendUsd({ provider: 'anthropic', },) > told.sinceUsd))
      return true;
    return deliveredNothing({ error, },);
  }

  /**
   Counts a call at its ceiling, by a SPEND line of the guard's own.

   @param modelId - model the call names

   @param ceiling - most the call could cost

   @param reckoning - why the line is reckoned, not reported

   @example
   ```ts
   countAtCeiling({ modelId, ceiling, reckoning: 'abandoned-bound', },);
   ```
   */
  function countAtCeiling(
    {
      modelId,
      ceiling,
      reckoning,
    }: {
      readonly modelId: string;
      readonly ceiling: {
        readonly usd: number;
        readonly promptTokens: number;
      };
      readonly reckoning: SpendReckoning;
    },
  ): void {
    counted.atCeiling += 1;
    reportSpend({
      provider: 'anthropic',
      label: modelId,
      extracted: {
        text: '',
        usage: {
          prompt_tokens: ceiling.promptTokens,
          completion_tokens: CLAUDE_COMPLETION_CAP,
        },
      },
      costUsd: ceiling.usd,
      estimated: reckoning,
    },);
  }

  /**
   Runs one exchange under the guard: admits it, lets it through, and counts what no line of the client holds.

   @param modelId - model the call names

   @param messages - what the call sends

   @param exchange - inner client's exchange

   @returns What the inner client answered

   @throws StatedRefusalError when the call is refused

   @throws ConcurrentGuardCallError when another call is still in flight, before anything is sent

   @throws Whatever the inner client threw, after the call was counted

   @example
   ```ts
   const reply = await guarded({ modelId, messages, exchange: askText, },);
   ```
   */
  async function guarded<ReplyT extends { readonly usage?: ReportedUsage; },>(
    {
      modelId,
      messages,
      exchange,
    }: {
      readonly modelId: Parameters<typeof pricesOf>[0]['cardId'];
      readonly messages: Parameters<typeof messageChars>[0]['messages'];
      readonly exchange: () => Promise<ReplyT>;
    },
  ): Promise<ReplyT> {
    /**
     Holds the guard's one place for a call until this one settles.
     */
    using _flight = enterFlight();
    /**
     The most this call can cost.
     */
    const ceiling = admit({
      modelId,
      messages,
    },);
    counted.calls += 1;
    // The meter at the start of the call stands for the start of its last attempt until the client tells the guard of one.
    told.attempts = 0;
    told.sinceUsd = runSpendUsd({ provider: 'anthropic', },);
    /**
     What the inner client did with the call: answered or threw.
     */
    const attempt = await attemptOf({ exchange, },);
    if (attempt.kind === 'threw') {
      if (!throwIsFree({ error: attempt.error, },)) {
        countAtCeiling({
          modelId,
          ceiling,
          reckoning: 'abandoned-bound',
        },);
      }
      throw attempt.error;
    }
    // A REPLY WITH NO ATTEMPT TOLD was not asked of the model. Only a gated client tells, so an ungated client's reuse is
    // counted at the ceiling as an unreported cost (the limit in the TSDoc of spendGuardedClient). Nothing is booked here,
    // the arm's signal carries the ending so the stage cannot take it for a lost voice and ask again, and the walk throws
    // it out of the run without scoring the row.
    if (gated && (told.attempts === 0)) {
      /**
       The ending the arm gets.
       */
      const ending = new RepliedWithoutCallError();
      endings.push(ending,);
      controller.abort(ending,);
      throw ending;
    }
    /**
     What the client reported about the tokens of its reply.
     */
    const { usage, } = attempt.reply;
    // A REPLY WITHOUT USAGE, OR WITH USAGE AND NO LINE ON THE METER SINCE ITS LAST ATTEMPT BEGAN, has a cost nobody wrote
    // down. The lines of the attempts before it, abandoned ones, do not stand for its own.
    if ((usage === undefined) || (runSpendUsd({ provider: 'anthropic', },) <= told.sinceUsd)) {
      countAtCeiling({
        modelId,
        ceiling,
        reckoning: 'unreported-bound',
      },);
    }
    return attempt.reply;
  }

  /**
   The client the guard wraps: the one given, or the one the builder makes over the guard's admit.
   */
  const client = (typeof inner) === 'function'
    ? inner(admitAttempt,)
    : inner;

  return {
    ...client,
    chatText: async function guardedText(request,) {
      return await guarded({
        modelId: request.modelId,
        messages: request.messages,
        exchange: function askText() {
          return client.chatText(request,);
        },
      },);
    },
    chatJson: async function guardedJson(request,) {
      return await guarded({
        modelId: request.modelId,
        messages: request.messages,
        exchange: function askJson() {
          return client.chatJson(request,);
        },
      },);
    },
    guardTotal: function guardTotal(): GuardTotal {
      return {
        usd: spentSoFar(),
        calls: counted.calls,
        atCeiling: counted.atCeiling,
      };
    },
  };
}

//endregion Editor fidelity spend guard
