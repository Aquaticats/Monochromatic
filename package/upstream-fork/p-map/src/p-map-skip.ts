/**
 Mapper sentinel for dropped results and the mapper call signature.
 
 Derived from [`p-map`](https://github.com/sindresorhus/p-map) by Sindre
 Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution.
 
 @module
 */

//region Sentinel

/**
 Returned from a mapper to drop that input's slot from the collected results.
 
 The fork mints its own symbol rather than reusing upstream `p-map`'s
 `pMapSkip`,
 so a fork mapper and an upstream mapper each return their own
 implementation's sentinel;
 the two symbols are never interchangeable.
 
 @example
 ```ts
 const results = await pMap({
   iterable: [1, 2, 3,],
   mapper: function dropOdds(value: number,): number | typeof pMapSkip {
     return (value % 2) === 0 ? value : pMapSkip;
   },
 });
 ```
 */
export const pMapSkip: unique symbol = Symbol('p-map mapper skip sentinel',);

//endregion Sentinel
