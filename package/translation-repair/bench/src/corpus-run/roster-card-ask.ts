import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';

import {
  CARD_PROVIDERS,
  type CardProvider,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { CommandLineOf, } from '@monochromatic-dev/translation-repair-run/ts';

//region Roster card ask
// THE COMMAND LINE OF `roster-card`, kept apart from the entry so the entry
// module exports nothing: an exported entry is bundled as a shared chunk,
// and `import.meta.main` is then false inside it and the task prints
// nothing (measured 2026-09-16 on the first build of the task).

/**
 A provider whose listing `roster-card` reads: every card provider but the Claude key.

 @example
 ```ts
 const provider: ListedCardProvider = 'hyper';
 ```
 */
export type ListedCardProvider = Exclude<CardProvider, 'anthropic'>;

/**
 Reads the provider and served id off the command line.

 @param line - the card's command line, read whole by `reportingRefusals`,
 which refuses it when either position is left out (ledger B75)

 @returns Provider and served id

 @throws {@link StatedRefusalError} When the provider is not a card provider,
 is the Claude key (whose cards the owner's lock fixes), or the served id is written empty, naming each part that is
 wrong and quoting what was typed (ledger B75)

 @example
 ```ts
 const asked = readAsk({ line, },);
 ```
 */
export function readAsk(
  { line, }: { readonly line: CommandLineOf<'roster-card'>; },
): {
  readonly provider: ListedCardProvider;
  readonly servedId: string;
} {
  /**
   Provider and served id as written; both are always written, since the
   line is refused without them, and an empty one is refused here.
   */
  const [
    providerWritten = '',
    servedId = '',
  ] = line.positionals;
  /**
   Provider as written, if it is one of the four.
   */
  const provider = CARD_PROVIDERS.find(function is(candidate,): boolean {
    return candidate === providerWritten;
  },);

  /**
   What is wrong with each part, empty when nothing is.
   */
  const problems = [
    ...((provider === undefined)
      ? [
        `roster-card's provider is one of ${CARD_PROVIDERS.join(', ',)}, and ${
          JSON.stringify(providerWritten,)
        } is none of them`,
      ]
      : []),
    ...((servedId === '') ? ['roster-card\'s served id is written empty',] : []),
  ];
  if (problems.length > 0) {
    throw new StatedRefusalError({
      says: problems.join('; ',),
    },);
  }
  /**
   Provider known to be a card provider, since no problem was found.
   */
  const asked = nonNullishOrThrow(provider,);
  // THE CLAUDE KEY HAS NOTHING TO LOOK UP: the owner's grant fixes its cards to Sonnet 5.5 and Haiku 5.5 at three
  // efforts each (`claude-card-sides.ts`), the key's own listing names Opus, Fable and older models too, and the
  // bearer reader here does not send the `x-api-key` header that listing needs.
  if (asked === 'anthropic') {
    throw new StatedRefusalError({
      says: 'roster-card does not read the Claude key: its cards are fixed by the owner\'s grant to Sonnet 5.5 and '
        + 'Haiku 5.5 and are edited in claude-card-sides.ts, not pasted from a listing',
    },);
  }
  return {
    provider: asked,
    servedId,
  };
}

//endregion Roster card ask
