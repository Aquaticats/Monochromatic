import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { ExtractedCompletion, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  noteRunSpend,
  type ProviderName,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Spend line
// WHAT ONE CALL COST, on the one line a reader can total.
//
// WHY THIS EXISTS. Both clients already read the provider's own `usage` block
// off the completion and both already print it, to `debug`. Every run this
// project has ever made logged at `info`, so a grep for token counts across the
// whole run archive returns nothing, and the question "what did that pass
// spend" has never once been answerable from a log.
//
// IT DID NOT MATTER UNTIL IT DID. Synthetic is a flat subscription: a call
// either fits the weekly allowance or it does not, and the meter reads a
// percentage that the `METERS` line already carries. Charm Hyper is metered per
// token, at rates that differ by two orders of magnitude across the same
// roster, so a run's cost there depends on WHICH seats answered and by how much.
// Nothing recorded that.
//
// THE COMPLETION HALF IS THE EXPENSIVE HALF, AND IT IS NOT THE ANSWER. Output
// is priced two to five times input across this roster, and `completion_tokens`
// counts thinking tokens, which dominate output on these models. A reader
// pricing a run off answer length would underread it by most of the bill.
//
// SHAPED LIKE THE `METERS` LINE, `name=value` after a marker word, so a reader
// can split rather than match. See `provider-budget.ts` for that line and
// `corpus-run/meter-sample-read.ts` for the reader it grew.
//
// PRINTS IDS AND COUNTS. Never a passage: a spend line rides in the same log as
// a corpus pass, and a run log holds unlicensed wording.

/**
 Logger root for spend reporting.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Why a line's counts and cost are reckoned rather than reported: `abandoned`
 is an estimate off what an unfinished stream delivered; `abandoned-bound`
 and `unreported-bound` are the most an attempt could have been billed, no
 more prompt tokens than its body had bytes and no more completion tokens than
 its `max_tokens`, for an attempt left unfinished or one that finished with
 no usage block (ledger P1, Bedrock, whose ledger guards the owner's card and
 must not under-read).

 ONE LIST FOR EVERY READER (ledger P14): the Bedrock ledger and the spend
 reader recognise the marks this writer puts on a line from here, so a mark
 added to the writer cannot go unread by either.

 @example
 ```ts
 const known = SPEND_RECKONINGS.includes('abandoned-bound',);
 ```
 */
export const SPEND_RECKONINGS = [
  'abandoned',
  'abandoned-bound',
  'unreported-bound',
] as const;

/**
 Why a line's counts and cost are reckoned rather than reported; see
 `SPEND_RECKONINGS`.

 @example
 ```ts
 const reckoning: SpendReckoning = 'abandoned-bound';
 ```
 */
export type SpendReckoning = typeof SPEND_RECKONINGS[number];

/**
 Whether a field names a reckoning this package writes.

 @param value - field as a line or ledger entry carries it

 @returns Whether it is one of `SPEND_RECKONINGS`

 @example
 ```ts
 if (isSpendReckoning(estimated,)) return { ...entry, estimated, };
 ```
 */
export function isSpendReckoning(value: unknown,): value is SpendReckoning {
  return SPEND_RECKONINGS.some(function isThis(reckoning,): boolean {
    return reckoning === value;
  },);
}

/**
 Marker word a reader finds the line by.

 EXPORTED RATHER THAN RESTATED IN THE READER, so the writer and the reader
 cannot drift apart the way two spellings of one literal always eventually do.

 NO LEADING SPACE, unlike METERS_MARKER in corpus-run/meter-sample-read.ts.
 That one only ever meets lines carrying a logger tag prefix, so it can demand
 the space in front. This marker opens the line this module RETURNS, and
 `readSpendLine` accepts it at the start of a line or after a space, so the
 writer's own output round-trips through the reader instead of reading as
 prose.
 */
export const SPEND_MARKER = 'SPEND ';

/**
 Value written where the provider returned no usage block at all.

 NAMED RATHER THAN OMITTED, and the line is printed anyway. A run whose
 provider stayed quiet and a run that spent nothing produce the same total,
 and only this tells them apart. Leaving the line out would let a reader
 report a cheap run when what happened was an unreported one.
 */
const UNREPORTED = 'unreported';

/**
 Records what one completed exchange cost, on its own line.

 @param provider - meter this call drew on, since only one of the two is
 priced per token and a reader totalling credits must not add the other

 @param label - model as the serving provider names it, matching the label
 `reportStreamProgress` already prints so the two lines can be joined

 @param extracted - completion whose `usage` block the provider filled in,
 or did not

 @param costUsd - USD the wire reported for this call, on the provider that
 bills in USD and said so

 @param endpoint - upstream the gateway named as serving this call, on the
 provider that fronts many; percent-encoded on the line because a display
 name may hold a space and this line's grammar splits on spaces

 @param estimated - why the counts and cost are reckoned rather than
 reported, on a call the wire never finished; the line carries it as a
 trailing field so a reader can total such lines beside the others or apart

 @param cachedTokens - prompt tokens the upstream served from its cache,
 where the wire reported the count, so the saving price-sorted routing and
 a stable sheet buy can be read off the line

 @returns Line that was logged, so a test can assert what a reader will parse
 rather than a paraphrase of it

 @example
 ```ts
 reportSpend({ provider: 'hyper', label: 'qwen3.8-max', extracted, },);
 ```
 */
export function reportSpend(
  {
    provider,
    label,
    extracted,
    costUsd,
    endpoint,
    estimated,
    cachedTokens,
  }: {
    readonly provider: ProviderName;
    readonly label: string;
    readonly extracted: ExtractedCompletion;
    readonly costUsd?: number;
    readonly endpoint?: string;
    readonly estimated?: SpendReckoning;
    readonly cachedTokens?: number;
  },
): string {
  /**
   Usage block as the provider reported it, absent where it did not.
   */
  const { usage, } = extracted;

  /**
   Prompt and completion counts, or the mark saying nobody reported them.
   */
  const counts = (usage === undefined)
    ? `prompt=${UNREPORTED} completion=${UNREPORTED}`
    : `prompt=${String(usage.prompt_tokens,)} completion=${String(usage.completion_tokens,)}`;

  /**
   USD the wire reported for this call, as a trailing field only where a
   provider bills in USD and said so; older lines and the other providers
   carry no such field, and the reader treats its absence as unreported.
   */
  const cost = (costUsd === undefined)
    ? ''
    : ` cost=${String(costUsd,)}`;

  // THE SAME FIGURE FEEDS THE RUN'S METER, so the ceiling the scheduler asks
  // (`corpus-run/spend-ceiling.ts`) and the total a reader sums off these
  // lines can never disagree about what was counted.
  if (costUsd !== undefined) {
    noteRunSpend({
      provider,
      costUsd,
    },);
  }

  /**
   Upstream that served the call, as a trailing field only where a gateway
   named one; encoded so a name with a space stays one field.
   */
  const servedBy = (endpoint === undefined)
    ? ''
    : ` endpoint=${encodeURIComponent(endpoint,)}`;

  /**
   Mark for a reckoned line, absent on a reported one.
   */
  const reckoned = (estimated === undefined)
    ? ''
    : ` estimated=${estimated}`;

  /**
   Cached prompt tokens, as a trailing field where the wire reported them.
   */
  const cached = (cachedTokens === undefined)
    ? ''
    : ` cached=${String(cachedTokens,)}`;

  /**
   Line assembled before the call so the logger chain stays one step per line.
   */
  const line = `${SPEND_MARKER}provider=${provider} model=${label} ${counts}${cost}${servedBy}${reckoned}${cached}`;

  /**
   Logger tagged with this report.
   */
  const rl = tagged({
    tag: reportSpend.name,
    l,
  },);
  rl.info(line,);
  return line;
}

//endregion Spend line
