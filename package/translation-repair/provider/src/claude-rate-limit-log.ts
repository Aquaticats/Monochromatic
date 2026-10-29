import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { TransportReply, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import { oneWayFlag, } from './one-way-flag.ts';

//region Claude rate-limit log
// THE LIMITS THE KEY STATES, read off the first reply that carries them and logged once, so a changed tier shows in the
// run's own log and not only in a probe's notes. Split out of `claude-client.ts` when the admit of each attempt took that
// file past its line budget.

/**
 Logger root for this package's model-facing shell.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Response-header family Anthropic states its rate limits in.
 */
export const RATE_LIMIT_HEADER_PREFIX = 'anthropic-ratelimit-';

/**
 Builds the reader that logs the rate-limit headers the first reply carries and no later one.

 @param tag - logger tag the line is written under, client's own name

 @returns Function to hand every attempt's reply to: it logs the limits of the first reply that states any, and does
 nothing for a reply that states none or for any reply after the logged one

 @example
 ```ts
 const logLimitsOnce = limitsLoggerOnce({ tag: createClaudeClient.name, },);
 ```
 */
export function limitsLoggerOnce({ tag, }: { readonly tag: string; },): (attemptReply: TransportReply,) => void {
  /**
   Raised once the rate-limit headers have been logged: once per client, which is once per run.
   */
  const limitsLogged = oneWayFlag();

  return function logLimitsOnce(attemptReply: TransportReply,): void {
    if (limitsLogged.isRaised())
      return;

    /**
     The rate-limit headers this reply carries, as `name=value` pairs.
     */
    const limits = Object.keys(attemptReply.headers ?? {},)
      .filter(function isLimit(name,): boolean {
        return name.startsWith(RATE_LIMIT_HEADER_PREFIX,);
      },)
      .map(function pairOf(name,): string {
        return `${name}=${attemptReply.headers?.[name] ?? ''}`;
      },);
    if (limits.length === 0)
      return;
    limitsLogged.raise();
    tagged({
      tag,
      l,
    },)
      .info(`rate limits on the first reply: ${limits.join(', ',)}`,);
  };
}

//endregion Claude rate-limit log
