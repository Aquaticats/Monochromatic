/**
 Guards against numbers read out of text by a reader more lenient than the
 text's writer (ledger B73): `Number` reads an empty or blank text as 0 and
 takes a radix prefix, an exponent, a sign and spaces; `parseInt` and
 `parseFloat` stop at the first character they cannot read and keep the
 rest; `Date.parse` and `new Date` take spellings no writer here makes, a
 date-time without a zone among them, which reads as local time. Each read
 here is listed with the rule that holds it to its writer's spelling, so a
 new read fails this test until it names one.

 WHAT THE SCAN READS. A call of `Number`, `parseInt`, `parseFloat`,
 `BigInt`, `Number.parseInt`, `Number.parseFloat` or `Date.parse`, written
 with or without `new`; `new Date` given an argument; a unary plus on
 anything but a literal; and any of those readers handed on as a value, as
 in `.map(Number)`. A reader reached another way (`globalThis.Number`, a
 renamed binding, arithmetic on text) is out of reach. Reads are keyed
 `path#site: reader`, where the site is the nearest enclosing named
 function, and counted per key.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each kind (ledger M21). Fixtures are cat-themed;
 the package case reads this package's own source.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup';
import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  childNodes,
  identifierName,
  isTreeNode,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import { expectRecordAsListed, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 Global functions that read a number out of text when called.
 */
const GLOBAL_READERS: ReadonlySet<string> = new Set([
  'BigInt',
  'Number',
  'parseFloat',
  'parseInt',
],);

/**
 Members of a global that read a number out of text, keyed by the global.
 */
const MEMBER_READERS: ReadonlyMap<string, ReadonlySet<string>> = new Map([
  [
    'Number',
    new Set([
      'parseFloat',
      'parseInt',
    ],),
  ],
  [
    'Date',
    new Set(['parse',],),
  ],
],);

/**
 Node kinds that open a function, which names the site of the reads inside.
 */
const FUNCTION_KINDS: ReadonlySet<string> = new Set(['ArrowFunctionExpression', 'FunctionDeclaration', 'FunctionExpression',],);

/**
 Node kinds that call their callee.
 */
const CALL_KINDS: ReadonlySet<string> = new Set([
  'CallExpression',
  'NewExpression',
],);

/**
 Why a read of a count holds to its writer's spelling.
 */
const WHOLE_NUMBER = 'reads only text isWholeNumberText takes: ASCII digits that a double holds exactly';

/**
 Why a read of an amount holds to its writer's spelling.
 */
const PLAIN_DECIMAL = 'reads only text isDecimalText takes, then refuses a reading that is not finite';

/**
 Why a read of a truth value is no read of text.
 */
const TRUTH_VALUE = 'turns a boolean into 1 or 0 for a comparator; no text is read';

/**
 Why a read of a hash prefix holds to its writer's spelling.
 */
const HASH_PREFIX = 'reads a fixed-length prefix of a hexadecimal digest hashContent wrote, in radix 16';

/**
 Why a read of a time from epoch milliseconds is no read of text.
 */
const EPOCH_MILLISECONDS = 'builds a date from epoch milliseconds, a number; no text is read';

/**
 Why a read of a log stamp holds to its writer's spelling.
 */
const LOG_STAMP = 'reads only a stamp isIsoStampText takes: exactly what toISOString, the logger\'s writer, writes';

/**
 Each read the package makes, as `path#site: reader`, with how many there
 are and the rule that holds them to their writer's spelling.
 */
const HELD_READS: Readonly<Record<string, {
  readonly calls: number;
  readonly why: string;
}>> = {
  'provider/deepseek-balance.ts#readBalanceText: Number': {
    calls: 1,
    why: 'reads only an amount isUnsignedNumberText took, so a signed, spelled or exponent form is refused before it',
  },
  'provider/bedrock-ledger.ts#bedrockCreditUsdFrom: Number': {
    calls: 1,
    why: PLAIN_DECIMAL,
  },
  'adjudication/candidate-select-wire.ts#readCandidateBallotWire: Number': {
    calls: 1,
    why: 'reads only a ballot index isCanonicalIndexText took when the reply was checked',
  },
  'consistency/consistency-quote-references.ts#decode: Number': {
    calls: 1,
    why: 'reads the digits of a character reference referenceEndOf took, ASCII digits or, behind 0x, hexadecimal digits',
  },
  'glossary/corpus-run/asked-count.ts#readAskedCount: Number': {
    calls: 1,
    why: WHOLE_NUMBER,
  },
  'slice-cache/corpus-run/cache-account-commits.ts#sourceCommitOf: Number': {
    calls: 1,
    why: WHOLE_NUMBER,
  },
  'slice-cache/corpus-run/cache-account-commits.ts#utcMinutes: new Date': {
    calls: 1,
    why: EPOCH_MILLISECONDS,
  },
  'slice-cache/corpus-run/cache-account-read.ts#declaresVersion: Number': {
    calls: 1,
    why: WHOLE_NUMBER,
  },
  'slice-cache/corpus-run/cache-account-read.ts#versionOf: Number': {
    calls: 1,
    why: WHOLE_NUMBER,
  },
  'page-check/corpus-run/canadian-date-parts.ts#readDay: Number': {
    calls: 1,
    why: 'reads a run its own scan bounded to ASCII digits, then refuses a run longer than a day takes',
  },
  'run/corpus-run/cap-census-read.ts#<module>: Date.parse': {
    calls: 1,
    why: 'reads a literal with its zone written, so it names the same instant on every machine',
  },
  'run/corpus-run/cap-census-read.ts#stampOf: Date.parse': {
    calls: 1,
    why: LOG_STAMP,
  },
  'run/corpus-run/cap-census-read.ts#streamContentOf: Number': {
    calls: 1,
    why: WHOLE_NUMBER,
  },
  'run/corpus-run/cap-override.ts#resolveHardCapMinutes: Number': {
    calls: 1,
    why: PLAIN_DECIMAL,
  },
  'run/corpus-run/command-flags.ts#wholeNumberFlag: Number': {
    calls: 1,
    why: WHOLE_NUMBER,
  },
  'corpus-pass/corpus-run/corpus-pass-order.ts#byResumeThenBandThenAttempts: Number': {
    calls: 2,
    why: TRUTH_VALUE,
  },
  'coverage-census/corpus-run/coverage-census-steps.ts#commandExit: Number': {
    calls: 1,
    why: 'reads an exit code Number.isInteger took as a number; no text is read',
  },
  'run/corpus-run/meter-report-text.ts#stampText: new Date': {
    calls: 1,
    why: EPOCH_MILLISECONDS,
  },
  'run/corpus-run/meter-sample-read.ts#stampOf: Date.parse': {
    calls: 1,
    why: LOG_STAMP,
  },
  'page-pass/corpus-run/ordinal-style.ts#readHanNumeral: Number': {
    calls: 1,
    why: WHOLE_NUMBER,
  },
  'bench/corpus-run/roster-card-render.ts#numberFrom: Number': {
    calls: 1,
    why: 'reads only a listing price isUnsignedNumberText takes, then refuses a reading that is not finite',
  },
  'run/corpus-run/run-timing-parse.ts#countIn: Number': {
    calls: 1,
    why: WHOLE_NUMBER,
  },
  'run/corpus-run/run-timing-parse.ts#durationIn: Number': {
    calls: 1,
    why: WHOLE_NUMBER,
  },
  'run/corpus-run/run-timing-parse.ts#readCallTiming: Date.parse': {
    calls: 1,
    why: LOG_STAMP,
  },
  'slice-cache/corpus-run/slice-overlap.ts#readOverlapSetting: Number': {
    calls: 1,
    why: WHOLE_NUMBER,
  },
  'run/corpus-run/spend-ceiling.ts#resolveSpendCeilingUsd: Number': {
    calls: 1,
    why: PLAIN_DECIMAL,
  },
  'run/corpus-run/spend-read.ts#countOf: Number': {
    calls: 1,
    why: WHOLE_NUMBER,
  },
  'run/corpus-run/spend-read.ts#usdOf: Number': {
    calls: 1,
    why: 'reads an amount the package wrote with String, and refuses text String does not write back',
  },
  'bench/corpus-run/window-trial-order.ts#armOrderFor: Number.parseInt': {
    calls: 1,
    why: HASH_PREFIX,
  },
  'repair/fidelity-alteration.ts#unsupportedVariant: Number': {
    calls: 1,
    why: 'reads the last character of a run its own scan bounded to ASCII digits',
  },
  'glossary/grace-override.ts#readWindowDial: Number': {
    calls: 1,
    why: WHOLE_NUMBER,
  },
  'slice-reading/grade-sheet-read.ts#toItem: Number': {
    calls: 1,
    why: WHOLE_NUMBER,
  },
  'markup/iso-stamp-text.ts#isIsoStampText: Date.parse': {
    calls: 1,
    why: 'the shared rule itself, which writes what it read back with toISOString and compares',
  },
  'markup/iso-stamp-text.ts#isIsoStampText: new Date': {
    calls: 1,
    why: EPOCH_MILLISECONDS,
  },
  'rendering-audit/page-headings.ts#htmlHeadings: Number': {
    calls: 1,
    why: 'reads one character after `<h`, then refuses any reading outside the heading levels',
  },
  'reference/reference-line-head.ts#numbered: Number': {
    calls: 1,
    why: WHOLE_NUMBER,
  },
  'provider/neuralwatt-return-words.ts#keyUnblockTextOf: new Date': {
    calls: 1,
    why: EPOCH_MILLISECONDS,
  },
  'provider/neuralwatt-return-words.ts#keyUnblockTextOf: Date.parse': {
    calls: 1,
    why: 'reads the instant a quota reply gives for a blocked key so the refusal spells it in UTC, and a text that is '
      + 'no instant is not echoed',
  },
  'provider/neuralwatt-ledger-config.ts#neuralWattDailyWhFrom: Number': {
    calls: 1,
    why: PLAIN_DECIMAL,
  },
  'provider/neuralwatt-day.ts#neuralWattTimeOfDayUtc: new Date': {
    calls: 1,
    why: EPOCH_MILLISECONDS,
  },
  'provider/neuralwatt-day.ts#neuralWattDayAfter: new Date': {
    calls: 1,
    why: EPOCH_MILLISECONDS,
  },
  'provider/neuralwatt-day.ts#neuralWattSecondsAfter: new Date': {
    calls: 1,
    why: EPOCH_MILLISECONDS,
  },
  'provider/neuralwatt-day.ts#neuralWattDayStartAt: new Date': {
    calls: 2,
    why: EPOCH_MILLISECONDS,
  },
  'provider/neuralwatt-day.ts#neuralWattDayStartOfInstant: Date.parse': {
    calls: 1,
    why: 'reads an instant a reply spelled, refusing a reading that is not finite',
  },
  'provider/neuralwatt-day.ts#twoDigitsBelow: Number': {
    calls: 1,
    why: 'reads two characters its own check took as ASCII digits',
  },
  'provider/neuralwatt-energy.ts#refusalTextOf: new Date': {
    calls: 1,
    why: LOG_STAMP,
  },
  'provider/neuralwatt-key-allowance.ts#allowanceFrom: Date.parse': {
    calls: 1,
    why: 'reads an instant a reply spelled, refusing a reading that is not finite',
  },
  'provider/neuralwatt-key-allowance.ts#allowanceFrom: new Date': {
    calls: 1,
    why: EPOCH_MILLISECONDS,
  },
  'provider/neuralwatt-key-allowance.ts#blockedUntilOf: new Date': {
    calls: 1,
    why: 'reads the reset allowanceFrom wrote with toISOString',
  },
  'provider/neuralwatt-usage.ts#echoedAt: Date.parse': {
    calls: 1,
    why: 'reads an instant a reply spelled, refusing a reading that is not finite',
  },
  'provider/neuralwatt-usage.ts#echoedAt: new Date': {
    calls: 1,
    why: EPOCH_MILLISECONDS,
  },
  'provider/neuralwatt-quota.ts#describePlanSpend: Number': {
    calls: 1,
    why: 'reads the text toFixed has just written from a number, a plain decimal; no other spelling is possible',
  },
  'provider/request-pace.ts#hyperRequestsPerHour: Number': {
    calls: 1,
    why: PLAIN_DECIMAL,
  },
  'provider/retry-stated-wait.ts#readWaitPart: Number': {
    calls: 1,
    why: 'reads a run its own scan bounded to ASCII digits with at most one point between digits',
  },
  'shared-contract/run-json-read.ts#offsetIn: Number.parseInt': {
    calls: 1,
    why: 'reads the digits opening an engine message nobody here wrote, refusing NaN, unsafe and negative readings',
  },
  'translate-floor/slice-cost-read.ts#countField: Number': {
    calls: 1,
    why: WHOLE_NUMBER,
  },
  'stage/stage-roster.ts#readVoices: Number': {
    calls: 2,
    why: WHOLE_NUMBER,
  },
  'translate-floor/translate-slate.ts#rotateCandidates: Number.parseInt': {
    calls: 1,
    why: HASH_PREFIX,
  },
  'markup/whole-number-text.ts#isNegativeWholeNumberText: Number': {
    calls: 1,
    why: 'the shared rule itself, reading the magnitude isWholeNumberText took',
  },
  'markup/whole-number-text.ts#isWholeNumberText: Number': {
    calls: 1,
    why: 'the shared rule itself, reading only ASCII digits',
  },
  'reference/work-title-lookup.ts#namingFirst: Number': {
    calls: 2,
    why: TRUTH_VALUE,
  },
};

/**
 Which reader an expression names, written as the source spells it.

 @param node - expression read

 @returns Reader's name, empty where the expression names none

 @example
 ```ts
 const reader = readerNamed({ node: call.callee, },);
 ```
 */
function readerNamed({ node, }: { readonly node: TreeNode; },): string {
  /**
   The name a bare identifier carries.
   */
  const bare = identifierName({ node, },);
  if (GLOBAL_READERS.has(bare,))
    return bare;
  if ((node.type !== 'MemberExpression') || (node.computed === true))
    return '';
  /**
   The global the member is read from, and the member's name.
   */
  const owner = identifierName({ node: node.object, },);
  /**
   The member's name.
   */
  const member = identifierName({ node: node.property, },);
  return (MEMBER_READERS.get(owner,)?.has(member,) === true) ? `${owner}.${member}` : '';
}

/**
 Children of a node that name something rather than read a value: a member
 name, an object or class key, a type name, and the global a member is read
 from.

 @param node - node read

 @returns Its children that are names

 @example
 ```ts
 const names = namesHeld({ node, },);
 ```
 */
function namesHeld({ node, }: { readonly node: TreeNode; },): readonly unknown[] {
  if (node.type === 'MemberExpression') {
    return [
      ...((node.computed === true) ? [] : [node.property,]),
      ...(MEMBER_READERS.has(identifierName({ node: node.object, },),) ? [node.object,] : []),
    ];
  }
  if (node.type === 'TSTypeReference')
    return [node.typeName,];
  return (('key' in node) && (node.computed !== true)) ? [node.key,] : [];
}

/**
 The read a node makes itself, where it makes one.

 @param node - node read

 @param named - nodes already known to name something or to be a callee

 @returns Reader's name as the key writes it, empty where the node reads nothing

 @example
 ```ts
 const reader = readOf({ node, named, },);
 ```
 */
function readOf(
  {
    node,
    named,
  }: {
    readonly node: TreeNode;
    readonly named: ReadonlySet<unknown>;
  },
): string {
  if (CALL_KINDS.has(node.type,) && isTreeNode(node.callee,)) {
    /**
     The callee's reader, where it is one.
     */
    const called = readerNamed({ node: node.callee, },);
    if (called !== '')
      return called;
    /**
     The arguments the call passes.
     */
    const args = node.arguments as readonly unknown[];
    return ((node.type === 'NewExpression') && (identifierName({ node: node.callee, },) === 'Date') && (args.length > 0))
      ? 'new Date'
      : '';
  }
  if ((node.type === 'UnaryExpression') && (node.operator === '+'))
    return (isTreeNode(node.argument,) && (node.argument.type !== 'Literal')) ? 'unary +' : '';
  if (named.has(node,))
    return '';
  /**
   The reader the node names, handed on as a value.
   */
  const handed = readerNamed({ node, },);
  return (handed === '') ? '' : `${handed} as a value`;
}

/**
 Numbers read out of text in the package's source, counted per
 `path#site: reader`.

 @param files - files read; tests and fixtures are skipped

 @returns Count per key, keys sorted

 @example
 ```ts
 const reads = numberReads({ files, },);
 ```
 */
function numberReads({ files, }: { readonly files: readonly SourceText[]; },): Readonly<Record<string, number>> {
  /**
   Count per key so far.
   */
  const counts = new Map<string, number>();
  for (const file of files) {
    if (file.isTest)
      continue;
    /**
     Nodes still to visit, each with its enclosing function's name.
     */
    const pending: { readonly node: TreeNode; readonly site: string; }[] = [{
      node: parseSource({ file, },).program,
      site: '<module>',
    },];
    /**
     Nodes a parent showed to be a name or a callee, which read nothing as a
     value. Parents are visited before their children.
     */
    const named = new Set<unknown>();
    while (pending.length > 0) {
      /**
       Node visited now.
       */
      const { node, site, } = pending.pop() as (typeof pending)[number];
      /**
       Enclosing function's name for the node's children.
       */
      const here = (FUNCTION_KINDS.has(node.type,) && isTreeNode(node.id,)) ? identifierName({ node: node.id, },) : site;
      for (const name of namesHeld({ node, },))
        named.add(name,);
      if (CALL_KINDS.has(node.type,))
        named.add(node.callee,);
      pending.push(...childNodes({ node, },).map(function withSite(child,) {
        return {
          node: child,
          site: here,
        };
      },),);
      /**
       The read the node makes, where it makes one.
       */
      const reader = readOf({ node, named, },);
      if (reader === '')
        continue;
      /**
       The read's key.
       */
      const key = `${file.path}#${site}: ${reader}`;
      counts.set(
        key,
        (counts.get(key,) ?? 0) + 1,
      );
    }
  }
  return Object.fromEntries([...counts,].toSorted(function byKey(
    [left,],
    [right,],
  ): number {
    return compareCodePoints({
      left,
      right,
    },);
  },),);
}

await describe({
  name: 'number reads (ledger B73)',
  children: [
    it({
      name: 'FINDS each reader called, constructed or handed on as a value, a unary plus on a name, and a read '
        + 'outside any function, and leaves the members that read no text, names that only spell a reader, '
        + 'a plus on a literal, a date built with no argument, and tests',
      fn: async () => {
        expect(numberReads({
          files: [
            {
              path: 'cat.ts',
              text: [
                'export function purr(t: string): number { return Number(t) + parseInt(t, 10) + parseFloat(t); }',
                'export function knead(t: string): number { return Number.parseInt(t, 10) + Number.parseFloat(t) + Date.parse(t); }',
                'export function groom(t: string): bigint { return BigInt(t); }',
                'export function nap(t: string): Date { return new Date(t); }',
                'export function stretch(t: string): number { return +t; }',
                'export function hunt(ts: readonly string[]): readonly number[] { return ts.map(Number).concat(ts.map(Number.parseFloat)); }',
                'export function pounce(t: string): object { return new Number(t); }',
                'export const naps = Number(\'3\');',
                'export function count(n: number): boolean { return Number.isFinite(n) && (n <= Number.MAX_SAFE_INTEGER); }',
                'export function names(paws: { Number: number }): number { const tail: Number = 1; return paws.Number + { parseInt: 2 }.parseInt + +1 + tail.valueOf(); }',
                'export function clock(): Date { return new Date(); }',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'cat.unit.test.ts',
              text: 'const meow = Number(\'7\');',
              isTest: true,
            },
          ],
        },),).toEqual({
          'cat.ts#<module>: Number': 1,
          'cat.ts#groom: BigInt': 1,
          'cat.ts#hunt: Number as a value': 1,
          'cat.ts#hunt: Number.parseFloat as a value': 1,
          'cat.ts#knead: Date.parse': 1,
          'cat.ts#knead: Number.parseFloat': 1,
          'cat.ts#knead: Number.parseInt': 1,
          'cat.ts#nap: new Date': 1,
          'cat.ts#pounce: Number': 1,
          'cat.ts#purr: Number': 1,
          'cat.ts#purr: parseFloat': 1,
          'cat.ts#purr: parseInt': 1,
          'cat.ts#stretch: unary +': 1,
        },);
      },
    },),
    it({
      name: 'FINDS NO NUMBER READ across the package but those named with the rule that holds each to its writer',
      fn: async () => {
        /**
         Count per key the named reads allow.
         */
        const allowed = Object.fromEntries(Object.entries(HELD_READS,)
          .map(function toCount([
            key,
            held,
          ],) {
            return [
              key,
              held.calls,
            ];
          },),);
        expectRecordAsListed({
          found: numberReads({ files: await readPackageSource(), },),
          listed: allowed,
        },);
      },
    },),
  ],
},);
