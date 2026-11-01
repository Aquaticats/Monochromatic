/**
 Mapper sentinel that ends an iteration early.

 Derived from [`p-map`](https://github.com/sindresorhus/p-map) by Sindre
 Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution.

 @module
 */

//region Sentinel

/**
 Returned from a mapper to stop iterating, like `break` in a loop.

 The run resolves with the results of the elements before the stop, in order,
 without waiting for the elements after it; mappers still running are not
 canceled and their results are dropped. Like {@link pMapSkip}, the fork mints
 its own symbol rather than reusing upstream `p-map`'s `pMapStop`, so a fork
 mapper and an upstream mapper each return their own implementation's
 sentinel.

 @example
 ```ts
 const pages = await pMap({
   iterable: pageNumbers(),
   mapper: async function readPage(pageNumber: number,): Promise<string | typeof pMapStop> {
     const page = await getPage(pageNumber,);
     return page.length === 0 ? pMapStop : page;
   },
   options: { concurrency: 4, },
 });
 ```
 */
export const pMapStop: unique symbol = Symbol('p-map mapper stop sentinel',);

//endregion Sentinel
