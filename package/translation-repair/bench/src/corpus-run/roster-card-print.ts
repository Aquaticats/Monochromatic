import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { ModelTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { CommandLineOf, } from '@monochromatic-dev/translation-repair-run/ts';
import {
  type ListedCardProvider,
  readAsk,
} from './roster-card-ask.ts';
import {
  fetchListing,
  LISTING_URL,
} from './roster-card-listing.ts';
import {
  cardFieldsFrom,
  listingRowFor,
  NOT_LISTED,
  renderProviderCard,
} from './roster-card-render.ts';

//region Roster card print
// PRINTS THE CARD FRAGMENT FOR ONE MODEL OFF A PROVIDER'S LIVE LISTING, so
// seating a model is: run this for each provider that serves it, paste the
// fragments into one card in `model-cards.ts`, add the spellings to the
// lists in `roster-id.ts`, run the package checks. The owner asked for the
// workflow on 2026-09-16 ("a reusable workflow or mise task or something
// should be built"); the cards are the workflow and this is its one tool.
//
// READ-ONLY AND KEY-SILENT: the key goes in a bearer header and is never
// printed; the listing is fetched once and only the named row is read.
//
// usage: mise run roster-card -- <synthetic|hyper|openrouter|bedrock|neuralwatt|deepseek|xiaomi> <served id>

/**
 List constant in `roster-id.ts` each provider's spellings go on.
 */
const SERVED_LIST: Readonly<Record<ListedCardProvider, string>> = {
  synthetic: 'SYNTHETIC_SERVED_IDS',
  hyper: 'HYPER_SERVED_IDS',
  openrouter: 'OPENROUTER_SERVED_IDS',
  bedrock: 'BEDROCK_SERVED_IDS',
  neuralwatt: 'NEURALWATT_SERVED_IDS',
  deepseek: 'DEEPSEEK_SERVED_IDS',
  xiaomi: 'XIAOMI_SERVED_IDS',
};

/**
 Characters of an ISO timestamp that spell the date.
 */
const DATE_CHARS = 10;

/**
 Prints the card fragment for the asked model.

 @param line - the card's command line, read whole by `reportingRefusals`

 @param env - environment the provider's key is read from: `process.env` in a run

 @param transport - transport the listing is fetched over: `fetchTransport` in a run

 @param now - the instant the fragment's provenance line names the date of

 @throws {@link StatedRefusalError} When the listing does not name the
 served id, which is the answer a typo or a retired model gets, or names it
 on rows that differ, by way of `listingRowFor`

 @example
 ```ts
 await printRosterCard({ line, env: process.env, transport: fetchTransport, now: new Date(), },);
 ```
 */
export async function printRosterCard(
  {
    line,
    env,
    transport,
    now,
  }: {
    readonly line: CommandLineOf<'roster-card'>;
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly transport: ModelTransport;
    readonly now: Date;
  },
): Promise<void> {
  /**
   Provider and served id asked for.
   */
  const {
    provider,
    servedId,
  } = readAsk({ line, },);
  /**
   Row the listing carries for this id.
   */
  const row = listingRowFor({
    body: await fetchListing({
      provider,
      env,
      transport,
    },),
    servedId,
  },);
  if ((typeof row) === 'symbol') {
    throw new StatedRefusalError({
      says: `${LISTING_URL[provider]} lists no model under ${servedId}`,
    },);
  }
  /**
   Fragment to paste, with the listing's fields filled in.
   */
  const fragment = renderProviderCard({
    provider,
    servedId,
    fields: cardFieldsFrom({
      provider,
      row,
    },),
  },);
  // Raw output, deliberately: this is text a person pastes, not pipeline
  // logging, and the tagged logger would prefix every line. That is the
  // stated exception to the tagged-logger rule, as `model-catalog` uses it.
  /**
   Today, for the fragment's provenance line.
   */
  const today = now.toISOString();
  console.log([
    `// ${provider} side for ${servedId}, read from ${LISTING_URL[provider]} on ${today.slice(
      0,
      DATE_CHARS,
    )}`,
    fragment,
    '',
    `// 1. Add '${servedId}' to ${SERVED_LIST[provider]} in roster-id.ts.`,
    '// 2. If this is a new model, add its roster id (the first serving provider\'s',
    '//    spelling: Synthetic, else Hyper, else Bedrock, else OpenRouter) to',
    '//    ROSTER_MODEL_IDS and write its card in model-cards.ts with',
    "//    completionCap: 'pooled-p99' and holds ['judge-unmeasured',",
    "//    'writer-unmeasured', 'reader-unmeasured'] until each is measured.",
    '// 3. Run the package checks; model-cards.unit.test.ts holds the lists to the cards.',
    `// A ${String(NOT_LISTED.description,)} is printed as a question to answer by hand.`,
  ].join('\n',),);
}

//endregion Roster card print
