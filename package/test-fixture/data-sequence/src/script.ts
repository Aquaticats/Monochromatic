/* v8 ignore file -- @preserve */

// Don't test fixtures.

//region Fixture Data Generation Utility: Provides manual generation logic for creating fixture data arrays and promises used in testing scenarios

/**
 * Manual fixture data generation utility script.
 *
 * This module provides the underlying generation logic used to create the fixture
 * data exported by other fixture modules. It demonstrates how to manually construct
 * arrays of consecutive integers and promise arrays with progressive timing delays.
 * While the pre-generated fixtures (array0to999, promises0to999) are preferred for
 * most use cases, this utility is valuable for understanding the generation patterns
 * or creating custom fixture variations.
 *
 * The generation logic here mirrors the implementation used to create the static
 * fixture exports, providing transparency into how the test data is constructed
 * and enabling developers to create similar fixtures with different parameters.
 *
 * @example
 * Using the array generation logic:
 * ```ts
 * import { generateConsecutiveArray } from '@monochromatic-dev/test-fixture-data-sequence';
 *
 * // Generate smaller array for quick tests
 * const smallArray = generateConsecutiveArray(100); // [0, 1, 2, ..., 99]
 *
 * // Generate larger array for stress tests
 * const largeArray = generateConsecutiveArray(5000); // [0, 1, 2, ..., 4999]
 *
 * console.log(`Small array length: ${smallArray.length}`); // 100
 * console.log(`Large array length: ${largeArray.length}`); // 5000
 * ```
 *
 * @example
 * Using the promise generation logic:
 * ```ts
 * import { generateProgressivePromises } from '@monochromatic-dev/test-fixture-data-sequence';
 *
 * async function testCustomPromises(): Promise<void> {
 *   // Generate fewer promises for faster tests
 *   const fastPromises = generateProgressivePromises(50);
 *
 *   const start = Date.now();
 *   const results = await Promise.all(fastPromises);
 *   const elapsed = Date.now() - start;
 *
 *   console.log(`Processed ${results.length} promises in ${elapsed}ms`);
 *   // Expected time: ~49ms (longest delay is 49ms)
 * }
 * ```
 *
 * @example
 * Creating custom fixture variations:
 * ```ts
 * import { generateConsecutiveArray, generateProgressivePromises } from '@monochromatic-dev/test-fixture-data-sequence';
 *
 * // Custom fixtures for specific test scenarios
 * const mediumArray = generateConsecutiveArray(500); // Mid-size for balanced tests
 * const quickPromises = generateProgressivePromises(20); // Fast async tests
 * const slowPromises = generateProgressivePromises(100); // Slower timing tests
 *
 * // Use in test suites with different performance characteristics
 * async function runPerformanceTests(): Promise<void> {
 *   console.log('Testing with medium array:', mediumArray.length);
 *
 *   const quickStart = Date.now();
 *   await Promise.all(quickPromises);
 *   console.log('Quick promises took:', Date.now() - quickStart);
 *
 *   const slowStart = Date.now();
 *   await Promise.all(slowPromises);
 *   console.log('Slow promises took:', Date.now() - slowStart);
 * }
 * ```
 */

/**
 * Generates an array of consecutive integers from 0 to count-1.
 *
 * This function creates an array containing consecutive integers starting from 0
 * up to but not including the specified count. It uses the same generation logic
 * as the static array0to999 fixture but allows for custom array sizes. Perfect
 * for creating test fixtures of various sizes to match specific testing needs.
 *
 * @param count - Number of consecutive integers to generate (array length)
 * @returns Array containing integers from 0 to count-1
 *
 * @example
 * Generating arrays of different sizes:
 * ```ts
 * import { generateConsecutiveArray } from '@monochromatic-dev/test-fixture-data-sequence';
 *
 * const tiny = generateConsecutiveArray(5);     // [0, 1, 2, 3, 4]
 * const small = generateConsecutiveArray(10);   // [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
 * const medium = generateConsecutiveArray(100); // [0, 1, 2, ..., 99]
 *
 * console.log('Tiny:', tiny);
 * console.log('Small length:', small.length);
 * console.log('Medium range:', `${medium[0]} to ${medium[medium.length - 1]}`);
 * ```
 *
 * @example
 * Using for test parameterization:
 * ```ts
 * import { generateConsecutiveArray } from '@monochromatic-dev/test-fixture-data-sequence';
 *
 * function testArrayProcessing(size: number): void {
 *   const testData = generateConsecutiveArray(size);
 *
 *   // Test your array processing logic
 *   const sum = testData.reduce((acc, val) => acc + val, 0);
 *   const expectedSum = (size * (size - 1)) / 2; // Sum formula for 0..n-1
 *
 *   console.log(`Size ${size}: sum=${sum}, expected=${expectedSum}`);
 *   console.assert(sum === expectedSum, 'Sum calculation failed');
 * }
 *
 * // Test with different array sizes
 * [10, 100, 1000].forEach(testArrayProcessing);
 * ```
 */
export function generateConsecutiveArray(
  count: number,
): number[] {
  return Array.from({ length: count, }, (_, index,) => index,);
}

/**
 * Generates an array of promises that resolve to consecutive integers with progressive delays.
 *
 * This function creates an array of promises where each promise resolves to its index
 * value after a delay equal to that index in milliseconds. It uses the same generation
 * logic as the static promises0to999 fixture but allows for custom array sizes and
 * enables creation of promise fixtures with different timing characteristics.
 *
 * Each promise at index i resolves to value i after i milliseconds, creating a
 * realistic simulation of async operations with increasing response times.
 *
 * @param count - Number of promises to generate (array length)
 * @returns Array of promises that resolve to consecutive integers with progressive delays
 *
 * @example
 * Generating promise arrays of different sizes:
 * ```ts
 * import { generateProgressivePromises } from '@monochromatic-dev/test-fixture-data-sequence';
 *
 * async function testDifferentSizes(): Promise<void> {
 *   // Quick test with 10 promises (0-9ms delays)
 *   const quick = generateProgressivePromises(10);
 *   const quickStart = Date.now();
 *   const quickResults = await Promise.all(quick);
 *   console.log(`Quick: ${quickResults.length} promises in ${Date.now() - quickStart}ms`);
 *
 *   // Medium test with 50 promises (0-49ms delays)
 *   const medium = generateProgressivePromises(50);
 *   const mediumStart = Date.now();
 *   const mediumResults = await Promise.all(medium);
 *   console.log(`Medium: ${mediumResults.length} promises in ${Date.now() - mediumStart}ms`);
 * }
 * ```
 *
 * @example
 * Testing async concurrency patterns:
 * ```ts
 * import { generateProgressivePromises } from '@monochromatic-dev/test-fixture-data-sequence';
 *
 * async function testConcurrencyPatterns(): Promise<void> {
 *   const promises = generateProgressivePromises(20);
 *
 *   // Test concurrent execution
 *   const concurrentStart = Date.now();
 *   const concurrentResults = await Promise.all(promises);
 *   const concurrentTime = Date.now() - concurrentStart;
 *
 *   // Test sequential execution
 *   const sequentialStart = Date.now();
 *   const sequentialResults: number[] = [];
 *   for (const promise of generateProgressivePromises(20)) {
 *     sequentialResults.push(await promise);
 *   }
 *   const sequentialTime = Date.now() - sequentialStart;
 *
 *   console.log(`Concurrent: ${concurrentTime}ms, Sequential: ${sequentialTime}ms`);
 *   console.log(`Results match: ${JSON.stringify(concurrentResults) === JSON.stringify(sequentialResults)}`);
 * }
 * ```
 *
 * @example
 * Creating custom timeout scenarios:
 * ```ts
 * import { generateProgressivePromises } from '@monochromatic-dev/test-fixture-data-sequence';
 *
 * async function testWithCustomTimeout(timeoutMs: number): Promise<void> {
 *   const promises = generateProgressivePromises(100);
 *
 *   const resultsWithTimeout = await Promise.allSettled(
 *     promises.map(promise =>
 *       Promise.race([
 *         promise,
 *         new Promise((_, reject) =>
 *           setTimeout(() => reject(new Error('Timeout')), timeoutMs)
 *         )
 *       ])
 *     )
 *   );
 *
 *   const successful = resultsWithTimeout.filter(r => r.status === 'fulfilled').length;
 *   const timedOut = resultsWithTimeout.filter(r => r.status === 'rejected').length;
 *
 *   console.log(`Timeout ${timeoutMs}ms: ${successful} successful, ${timedOut} timed out`);
 * }
 *
 * // Test different timeout thresholds
 * testWithCustomTimeout(25);  // Should timeout ~75 promises
 * testWithCustomTimeout(50);  // Should timeout ~50 promises
 * ```
 */
export function generateProgressivePromises(
  count: number,
): Promise<number>[] {
  return Array.from(
    { length: count, },
    (_, index,) =>
      // oxlint-disable-next-line promise/avoid-new -- Creating promise for test fixture
      new Promise<number>(resolve =>
        setTimeout(() => {
          resolve(index,);
        }, index,)
      ),
  );
}

//endregion Fixture Data Generation Utility
