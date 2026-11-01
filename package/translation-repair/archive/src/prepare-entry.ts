import type { SeededErrorSpec, } from '@monochromatic-dev/translation-repair-lane-contest/ts';

//region Benchmark entry
// What a seeded-error benchmark reads for one corpus entry. The critic
// benchmark that prepared entries here once per model attempt is gone
// (ledger B30); the recall and repair benchmarks read this shape.

/**
 One corpus entry prepared for benchmarking.

 @example
 ```ts
 const entry: BenchmarkEntry = {
   entryId: 'whiskers',
   sourceText: zh,
   targetText: en,
   seeds: deriveOmissionSeeds({ text: enBody, maxSeeds: 2, },),
 };
 ```
 */
export type BenchmarkEntry = {
  /**
   Corpus entry id, e.g. the `people/<id>` directory name.
   */
  readonly entryId: string;

  /**
   Original document, front matter included.
   */
  readonly sourceText: string;

  /**
   Clean translation; seeds are planted into it here.
   */
  readonly targetText: string;

  /**
   Errors to plant, in application order.
   */
  readonly seeds: readonly SeededErrorSpec[];
};

//endregion Benchmark entry
