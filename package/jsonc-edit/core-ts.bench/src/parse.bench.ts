/**
 Parse benchmark: jsonc-edit against microsoft `jsonc-parser` and
 `jsonc-eslint-parser`, on a clean document and on a commented document.
 jsonc-edit parses both structurally, so the two inputs differ only in the
 comment and trailing-comma work each parser must carry. Run with the `bench`
 task.
 
 @module
 */

import type { StringJsonc, } from '@monochromatic-dev/jsonc-edit-core-ts/ts/brand.ts';
import { parseJsonc, } from '@monochromatic-dev/jsonc-edit-core-ts/ts';
import { parseForESLint, } from 'jsonc-eslint-parser';
import { parseTree, } from 'jsonc-parser';

//region Constants

/**
 Number of record entries in each benchmark input.
 */
const ENTRY_COUNT = 300;

/**
 Timed iterations per parser.
 */
const ITERATIONS = 3_000;

/**
 Untimed warm-up iterations to let the JIT settle.
 */
const WARMUP = 200;

/**
 Milliseconds per second, for the ops/second figure.
 */
const MS_PER_SECOND = 1_000;

//endregion Constants

//region Inputs

/**
 Builds a clean (comment-free, valid JSON) document of `count` entries.
 
 @param count - Number of record entries.
 
 @returns Pretty-printed JSON string.
 
 @example
 ```ts
 buildCleanInput({ count: 2 });
 ```
 */
function buildCleanInput({
  count,
}: {
  readonly count: number;
},): string {
  /**
   Accumulated entry lines.
   */
  const lines: string[] = [];
  for (let index = 0; index < count; index += 1)
    lines.push(`  "key${index}": { "id": ${index}, "name": "item-${index}", "tags": [1, 2, 3] }`,);
  return `{\n${lines.join(',\n',)}\n}`;
}

/**
 Builds a commented document of `count` entries, forcing the structured parser.
 
 @param count - Number of record entries.
 
 @returns JSONC string with a trailing comment on each entry.
 
 @example
 ```ts
 buildCommentedInput({ count: 2 });
 ```
 */
function buildCommentedInput({
  count,
}: {
  readonly count: number;
},): string {
  /**
   Accumulated entry lines, each carrying a trailing comment.
   */
  const lines: string[] = [];
  for (let index = 0; index < count; index += 1)
    lines.push(`  "key${index}": { "id": ${index}, "name": "item-${index}" }, // entry ${index}`,);
  return `{\n${lines.join('\n',)}\n}`;
}

//endregion Inputs

//region Parser adapters

/**
 Parses with jsonc-edit.
 
 @param source - Document to parse.
 
 @example
 ```ts
 runJsoncEdit('{}');
 ```
 */
function runJsoncEdit(source: string,): void {
  parseJsonc({ source: source as StringJsonc, },);
}

/**
 Parses with microsoft `jsonc-parser` into an offset tree.
 
 @param source - Document to parse.
 
 @example
 ```ts
 runMicrosoft('{}');
 ```
 */
function runMicrosoft(source: string,): void {
  parseTree(source,);
}

/**
 Parses with `jsonc-eslint-parser` into an ESTree-style AST.
 
 @param source - Document to parse.
 
 @example
 ```ts
 runEslint('{}');
 ```
 */
function runEslint(source: string,): void {
  parseForESLint(source, { jsonSyntax: 'JSONC', },);
}

//endregion Parser adapters

//region Harness

/**
 Times a parser over an input and prints its throughput.
 
 @param label - Parser label.
 
 @param parse - Parser adapter.
 
 @param input - Document to parse repeatedly.
 
 @example
 ```ts
 measure({ label: 'x', parse: runJsoncEdit, input: '{}' });
 ```
 */
function measure({
  label,
  parse,
  input,
}: {
  readonly label: string;
  readonly parse: (source: string) => void;
  readonly input: string;
},): void {
  for (let index = 0; index < WARMUP; index += 1)
    parse(input,);
  /**
   High-resolution start timestamp.
   */
  const start = performance.now();
  for (let index = 0; index < ITERATIONS; index += 1)
    parse(input,);
  /**
   Elapsed milliseconds across all timed iterations.
   */
  const elapsed = performance.now() - start;
  /**
   Parses per second.
   */
  const opsPerSecond = Math.round(ITERATIONS / (elapsed / MS_PER_SECOND),);
  console.log(`  ${label.padEnd(36,)} ${`${elapsed.toFixed(1,)}ms`.padStart(9,)}  ${String(opsPerSecond,).padStart(9,)} ops/s`,);
}

//endregion Harness

//region Run

/**
 Clean benchmark document.
 */
const clean = buildCleanInput({ count: ENTRY_COUNT, },);

/**
 Commented benchmark document.
 */
const commented = buildCommentedInput({ count: ENTRY_COUNT, },);

console.log(
  `jsonc-edit parse benchmark: ${ENTRY_COUNT} entries, ${ITERATIONS} iterations\n`
    + `clean ${clean.length} B, commented ${commented.length} B\n`,
);

console.log('CLEAN input (no comments or trailing commas):',);
measure({ label: 'jsonc-edit parseJsonc', parse: runJsoncEdit, input: clean, },);
measure({ label: 'microsoft jsonc-parser parseTree', parse: runMicrosoft, input: clean, },);
measure({ label: 'jsonc-eslint-parser parseForESLint', parse: runEslint, input: clean, },);

console.log('\nCOMMENTED input (comments and trailing commas):',);
measure({ label: 'jsonc-edit parseJsonc', parse: runJsoncEdit, input: commented, },);
measure({ label: 'microsoft jsonc-parser parseTree', parse: runMicrosoft, input: commented, },);
measure({ label: 'jsonc-eslint-parser parseForESLint', parse: runEslint, input: commented, },);

//endregion Run
