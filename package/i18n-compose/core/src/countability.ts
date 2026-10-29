/**
 Countability validation helpers shared by locale noun-phrase renderers.
 
 @module
 */

import type { NounEntry, } from './entries.ts';

/**
 Asserts that a noun entry may appear in a `noun.counted` phrase.
 
 Mass-only nouns cannot safely combine with the bare numeric count carried
 by `noun.counted`. Consumers should model measured amounts as their own
 countable nouns or pass preformatted caller text through `noun.externalText`.
 
 @param entry - noun entry whose countability controls counted phrase validity
 
 @param noun - noun key used in the diagnostic when validation fails
 
 @throws Error when noun is marked mass-only
 
 @example
 ```ts
 assertCountableNoun({ entry: { surface: 'cat', countability: 'countable' }, noun: 'cat' });
 ```
 */
export function assertCountableNoun(
  {
    entry,
    noun,
  }: {
    readonly entry: NounEntry;
    readonly noun: string;
  },
): void {
  if (entry.countability
    === 'mass')
    throw new Error(`Cannot count mass noun '${noun}' with noun.counted`,);
}
