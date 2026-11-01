import type { CriticTally, } from './attribution-report.ts';

//region Score attribution table
// The critic table `score-attribution` prints: its header and one row per
// critic, as RATES rather than tallies.

/**
 Column widths of the critic table, wide enough that no row runs into its
 neighbour. Object-literal values, so the width numbers stay readable here
 rather than becoming named constants nobody can picture.
 */
const COLUMN = {
  model: 42,
  heard: 7,
  raised: 8,
  emitted: 9,
  hits: 7,
  raisedPerChunk: 11,
  hitsPerChunk: 10,
} as const;

/**
 Digits kept on per-chunk rates.

 These are RATES, not percentages, and the distinction is not pedantic: one
 critic can raise many claims in a single chunk, so claims per chunk heard
 legitimately exceeds one and rendering it as a percentage produced readings
 like "3400%" the first time this ran.
 */
const RATE_DIGITS = 2;

/**
 Renders the table header.

 @returns Header line

 @example
 ```ts
 console.log(attributionHeaderLine(),);
 ```
 */
export function attributionHeaderLine(): string {
  return [
    'CRITIC'
      .padEnd(COLUMN.model,),
    'heard'
      .padStart(COLUMN.heard,),
    'raised'
      .padStart(COLUMN.raised,),
    'emitted'
      .padStart(COLUMN.emitted,),
    'hits'
      .padStart(COLUMN.hits,),
    'raised/ch'
      .padStart(COLUMN.raisedPerChunk,),
    'hits/ch'
      .padStart(COLUMN.hitsPerChunk,),
  ].join('',);
}

/**
 Renders one critic's row.

 Rates divide by chunks HEARD rather than by chunks in the run, so a critic
 that lost voices on half the run is not read as half as willing to raise a
 claim.

 @param critic - tally for one critic

 @returns Row line

 @example
 ```ts
 console.log(attributionCriticLine({ critic, },),);
 ```
 */
export function attributionCriticLine(
  {
    critic,
  }: {
    readonly critic: CriticTally;
  },
): string {
  /**
   Renders one rate over the chunks this critic heard.

   A critic heard on no chunk has no rate, and printing `0.00` for it would be
   FALSE rather than merely uninformative. No such row exists:
   `buildAttributionReport` lists a critic only from the chunks it was counted
   as heard on, so every row it hands over has heard at least one.

   @param count - numerator

   @returns Rendered rate

   @throws Error when the critic heard no chunk

   @example
   ```ts
   const rendered = rate(critic.claimsRaised,);
   ```
   */
  function rate(count: number,): string {
    if (critic.chunksHeard === 0)
      throw new Error(`unreachable: ${critic.modelId} has no chunk heard, yet a row exists only for a heard critic`,);
    return (count / critic.chunksHeard).toFixed(RATE_DIGITS,);
  }

  return [
    critic.modelId
      .padEnd(COLUMN.model,),
    String(critic.chunksHeard,)
      .padStart(COLUMN.heard,),
    String(critic.claimsRaised,)
      .padStart(COLUMN.raised,),
    String(critic.emissions,)
      .padStart(COLUMN.emitted,),
    String(critic.acceptedHits,)
      .padStart(COLUMN.hits,),
    rate(critic.claimsRaised,)
      .padStart(COLUMN.raisedPerChunk,),
    rate(critic.acceptedHits,)
      .padStart(COLUMN.hitsPerChunk,),
  ].join('',);
}

//endregion Score attribution table
