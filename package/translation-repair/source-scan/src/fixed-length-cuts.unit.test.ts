/**
 Guards ledger B21 (2026-09-29): text cut at a fixed number of UTF-16 units
 can end inside a character. An error's body excerpt, a stream's opening and
 a refused reply's opening each kept half an emoji where the limit fell
 between its two units, and the log showed `\ud83d` or a replacement
 character. A text's opening is cut with `wholeOpening` (`code-points.ts`),
 which never ends inside a character; every other `.slice(0, LIMIT)` and
 `.slice(-LIMIT)` in the package's source cuts a list, text that is ASCII by
 construction, or a buffer compared unit for unit, and is listed here with
 its reason, so a new fixed-length cut is classed before it lands.

 THE FIXTURES COME FIRST, so the package-wide case is read against a scan
 shown able to find each shape (ledger M21). Fixtures are cat-themed; the
 package case reads this package's own source.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  childNodes,
  identifierName,
  isTreeNode,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 Fixed-length cuts that stay `.slice`, each as `path#LIMIT`, with why no
 character can be split there.
 */
const KEPT_CUTS: ReadonlyMap<string, string> = new Map([
  [
    'translate-floor/translate-slate.ts#ROTATION_HEX_DIGITS',
    'a hex digest',
  ],
  [
    'stage/stage-call.ts#RAW_PREVIEW_CHARS',
    'a list of grapheme clusters',
  ],
  [
    'house-rule/refusal.ts#REFUSAL_SCAN_WINDOW',
    'a window searched for refusal markers, which a cut character cannot match',
  ],
  [
    'slice-reading/image-reading-sense.ts#REFUSAL_WINDOW_CHARS',
    'a window searched for refusal markers, which a cut character cannot match',
  ],
  [
    'provider-stream/stream-recurrence-watch.ts#BUFFER_CHARS',
    'a buffer compared unit for unit with its own tail',
  ],
  [
    'provider-stream/stream-recurrence-watch.ts#TAIL_CHARS',
    'a tail compared unit for unit with the buffer it came from',
  ],
  [
    'corpus-source/repeat-groups-refrain.ts#REFRAIN_PREFIX_MINIMUM',
    'a list of code points, one per element, so no character can be split',
  ],
  [
    'corpus-source/repeat-groups-refrain.ts#REFRAIN_SUFFIX_MINIMUM',
    'a list of code points, one per element, so no character can be split',
  ],
  [
    'provider-stream/synthetic-quota.ts#RECEIVED_KEYS_LISTED',
    'a list of the keys of a refused body',
  ],
  [
    'provider-stream/synthetic-quota.ts#RECEIVED_KEY_NAME_LENGTH',
    'a list of code points, one per element, so no character can be split',
  ],
  [
    'slice-cache/corpus-run/slice-census-unpaired.ts#UNPAIRED_ENTRIES_LISTED',
    'a list of entries',
  ],
  [
    'coverage-census/corpus-run/coverage-census-report.ts#MISPLACED_NAMED',
    'a list of uncalled functions',
  ],
  [
    'bench/corpus-run/window-trial-order.ts#DIGEST_CHARS',
    'a hex digest',
  ],
  [
    'bench/corpus-run/window-trial-probe-lines.ts#PROTOCOL_LOG_CHARS',
    'a hex digest',
  ],
  [
    'probe/corpus-run/probe-store.ts#DIGEST_IN_NAME',
    'a hex digest',
  ],
  [
    'probe/corpus-run/recall-scorecard-store.ts#TIP_IN_NAME',
    'a commit hash',
  ],
  [
    'slice-cache/corpus-run/cache-account-read.ts#CITED_HASH_LENGTH',
    'a commit hash',
  ],
  [
    'slice-cache/corpus-run/cache-account-commits.ts#ISO_MINUTES_LENGTH',
    'an ISO time stamp',
  ],
  [
    'bench/corpus-run/roster-card-print.ts#DATE_CHARS',
    'an ISO date stamp',
  ],
  [
    'bench/corpus-run/editor-width-control.ts#CONTROL_SLICES',
    'a list of slices',
  ],
  [
    'probe/corpus-run/damage-sample-draw.ts#DAMAGE_SAMPLE_SIZE',
    'a list of samples',
  ],
  [
    'bench/corpus-run/translate-probe-run.ts#PROBE_SLICES',
    'a list of slices',
  ],
  [
    'run/corpus-run/run-config.ts#CHECKER_BENCH_WIDTH',
    'a list of model ids',
  ],
  [
    'slice-cache/corpus-run/pass-schema-guard.ts#NAMED_EXAMPLES',
    'a list of entry ids',
  ],
],);

/**
 Whether a name is written as a constant: letters in upper case, digits and
 underscores, with at least one letter.

 @param name - identifier

 @returns Whether it reads as a named limit

 @example
 ```ts
 isConstantName({ name: 'PURR_CHARS', },); // true
 ```
 */
function isConstantName({ name, }: { readonly name: string; },): boolean {
  return (name.toUpperCase() === name) && (name.toLowerCase() !== name);
}

/**
 The named limit a `.slice` call cuts at, where it cuts a fixed length from
 the start (`.slice(0, LIMIT)`) or from the end (`.slice(-LIMIT)`).

 @param node - node read

 @returns Limit's name, empty where the node is no such cut

 @example
 ```ts
 const limit = cutLimit({ node, },); // 'PURR_CHARS'
 ```
 */
function cutLimit({ node, }: { readonly node: TreeNode; },): string {
  if ((node.type !== 'CallExpression') || (!isTreeNode(node.callee,)) || (!Array.isArray(node.arguments,)))
    return '';
  /**
   What is called.
   */
  const { callee, } = node;
  if ((callee.type !== 'MemberExpression') || (callee.computed === true)
    || (identifierName({ node: callee.property, },) !== 'slice'))
    return '';
  /**
   What it is called with.
   */
  const args: readonly unknown[] = node.arguments;
  /**
   First and second arguments.
   */
  const [first, second,] = args;
  if ((args.length === 2) && isTreeNode(first,) && (first.type === 'Literal') && (first.value === 0))
    return identifierName({ node: second, },);
  if ((args.length === 1) && isTreeNode(first,) && (first.type === 'UnaryExpression') && (first.operator === '-'))
    return identifierName({ node: first.argument, },);
  return '';
}

/**
 Every fixed-length cut at a named limit in the files, each as `path#LIMIT`,
 test files left out.

 @param files - files read

 @returns Cuts found, sorted and without repeats

 @example
 ```ts
 const cuts = fixedLengthCuts({ files, },);
 ```
 */
function fixedLengthCuts({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Cuts found so far.
   */
  const cuts = new Set<string>();
  for (const file of files) {
    if (file.isTest)
      continue;
    /**
     Nodes still to read, a work stack so the walk needs no recursion.
     */
    const pending: TreeNode[] = [parseSource({ file, },).program,];
    for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
      /**
       The limit this node cuts at, if any.
       */
      const limit = cutLimit({ node, },);
      if ((limit !== '') && isConstantName({ name: limit, },))
        cuts.add(`${file.path}#${limit}`,);
      pending.push(...childNodes({ node, },),);
    }
  }
  return [...cuts,].toSorted();
}

/**
 A cat's source file, as the scan reads one.

 @param text - file text

 @returns File under a fixture path

 @example
 ```ts
 const file = catFile({ text: 'const purr = 1;', },);
 ```
 */
function catFile({ text, }: { readonly text: string; },): SourceText {
  return {
    path: 'cat-log.ts',
    text,
    isTest: false,
  };
}

await describe({
  name: 'fixed-length cuts (ledger B21)',
  children: [
    it({
      name: 'FINDS a cut from the start and a cut from the end at a named limit, wherever the call breaks its lines',
      fn: async () => {
        expect(fixedLengthCuts({
          files: [catFile({
            text: [
              'const PURR_CHARS = 80;',
              'const TAIL_CHARS = 20;',
              'export function opening(purr: string,): string {',
              '  return purr.slice(0, PURR_CHARS,);',
              '}',
              'export function ending(purr: string,): string {',
              '  return purr',
              '    .slice(',
              '      -TAIL_CHARS,',
              '    );',
              '}',
            ].join('\n',),
          },),],
        },),).toEqual([
          'cat-log.ts#PURR_CHARS',
          'cat-log.ts#TAIL_CHARS',
        ],);
      },
    },),
    it({
      name: 'LEAVES a cut that starts elsewhere, one at a variable, a whole-character opening and a test file',
      fn: async () => {
        /**
         Cuts the scan must not report.
         */
        const text = [
          'const PURR_CHARS = 80;',
          'export function cuts(purr: string, width: number,): readonly string[] {',
          '  return [purr.slice(PURR_CHARS,), purr.slice(0, width,), wholeOpening({ text: purr, units: PURR_CHARS, },),];',
          '}',
        ].join('\n',);
        expect(fixedLengthCuts({ files: [catFile({ text, },),], },),).toEqual([],);
        expect(fixedLengthCuts({
          files: [{
            path: 'cat-log.unit.test.ts',
            text: 'export const opening = (purr: string,): string => purr.slice(0, PURR_CHARS,);',
            isTest: true,
          },],
        },),).toEqual([],);
      },
    },),
    it({
      name: 'CUTS no text at a fixed length in the package\'s source except through wholeOpening, and every '
        + 'listed cut still stands',
      fn: async () => {
        /**
         Cuts in the package's source.
         */
        const cuts = fixedLengthCuts({ files: await readPackageSource(), },);
        expectNoFindings({
          findings: cuts.filter(function unlisted(cut,): boolean {
            return !KEPT_CUTS.has(cut,);
          },),
        },);
        expectNoFindings({
          findings: [...KEPT_CUTS.keys(),].filter(function gone(cut,): boolean {
            return !cuts.includes(cut,);
          },),
        },);
      },
    },),
  ],
},);
