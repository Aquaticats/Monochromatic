/**
 Guards against a fake clock installed without clearing the native timers it meets (ledger B837). The fake
 `clearTimeout` of a clock refuses a timer it did not make: it writes `FakeTimers: clearTimeout was invoked to clear a
 native timer` to stderr and leaves the timer running. A module that starts a timer when it is loaded and clears it
 when its start settles meets exactly that: the start settled under the fake clock, the timer is a real one, and the
 process then stays alive until the real timer fires (five seconds for the logger's start check), so a suite of
 milliseconds held its process for five seconds. The fake `clearInterval` and `clearImmediate` of a clock do the same
 for an interval and an immediate it did not make, with the message naming the function. The option
 `shouldClearNativeTimers: true` hands such a clear to the real function of the same name.

 WHAT THE SCAN READS, in every file of every family package, tests and fixtures alike: each call of `useFakeTimers`, as
 a method of any object or called bare. A call whose clock fakes any of the three clear functions (`clearTimeout`,
 `clearInterval`, `clearImmediate`) must pass an object literal that holds `shouldClearNativeTimers: true`. A clock
 fakes a clear function when its options hold no `toFake` list (every timer is faked then), an empty `toFake` list (the
 library fakes every timer for an empty list as well, so it is read as naming them all), or a `toFake` list that names
 it. A clock whose non-empty list leaves all three out, the date alone for one, cannot meet the hazard and is left
 alone. A finding names the first clear function the clock fakes. A call with no options, options that are no object
 literal, an object literal with a spread, a `toFake` that is no list of string literals, and an option that is not the
 literal `true` are findings as they stand, since the scan cannot tell what they hold. A finding names the file, the
 line and the reason.

 OUT OF THE SCAN'S REACH: a clock installed with the `install` function of the fake-timers library itself (no family
 file does that), an options object built in another binding, and `toNotFake`, which a clock that lists it is read as
 if it did not.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a scan shown able to find each form
 (ledger M21). Fixtures are cat-themed; the package case reads the family's own source.

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
  literalText,
  memberName,
  nodesUnder,
  parseSource,
  type SourceText,
  type TreeNode,
  unwrapped,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Fake clock installs
// Which calls install a fake clock, and whether the clock that fakes a clear function clears native timers.

/**
 Name of the sandbox method that installs a fake clock.
 */
const INSTALL_NAME = 'useFakeTimers';

/**
 Option that hands a clear of a native timer to the real clear function.
 */
const CLEAR_OPTION = 'shouldClearNativeTimers';

/**
 Timer functions whose fake refuses a native timer, in the order the library lists its timers.
 */
const CLEARED_TIMERS = [
  'clearTimeout',
  'clearInterval',
  'clearImmediate',
] as const;

/**
 One call that installs a fake clock, and what the scan makes of it.
 */
type Install = {
  /**
   Place of the call, as the file's location and the line it starts on.
   */
  readonly place: string;

  /**
   Why the call is a finding, empty where its clock clears native timers or cannot meet them.
   */
  readonly refusal: string;
};

/**
 Line a node starts on.

 @param file - file holding the node

 @param node - node read

 @returns One-based line number

 @example
 ```ts
 const line = lineOf({ file, node, },);
 ```
 */
function lineOf({ file, node, }: { readonly file: SourceText; readonly node: TreeNode; },): number {
  return file.text
    .slice(
    0,
    node.start,
    )
    .split('\n',)
    .length;
}

/**
 Whether a call installs a fake clock, as a method of any object or called bare.

 @param node - node read

 @returns Whether it is a call of `useFakeTimers`

 @example
 ```ts
 const installs = isInstall({ node, },); // true for sinon.useFakeTimers({}) and for useFakeTimers()
 ```
 */
function isInstall({ node, }: { readonly node: TreeNode; },): boolean {
  if (node.type !== 'CallExpression')
    return false;
  /**
   The callee inside any parentheses and casts.
   */
  const { inner: callee, } = unwrapped({ node: node.callee, },);
  if (!isTreeNode(callee,))
    return false;
  return (callee.type === 'MemberExpression')
    ? memberName({ node: callee, },) === INSTALL_NAME
    : identifierName({ node: callee, },) === INSTALL_NAME;
}

/**
 The properties of an object literal under a name, written plainly or as a string.

 @param properties - properties of the literal

 @param name - key asked for

 @returns Properties keyed so, none where the literal holds no such key

 @example
 ```ts
 const [toFake,] = propertiesNamed({ properties, name: 'toFake', },);
 ```
 */
function propertiesNamed(
  {
    properties,
    name,
  }: {
    readonly properties: readonly TreeNode[];
    readonly name: string;
  },
): readonly TreeNode[] {
  return properties.filter(function isNamed(property,): boolean {
    return (property.type === 'Property') && (property.computed !== true)
      && ((identifierName({ node: property.key, },) === name) || (literalText({ node: property.key, },) === name));
  },);
}

/**
 A `toFake` list that was read: the clear functions it fakes.
 */
type FakedClears = {
  /**
   Which reading this is.
   */
  readonly kind: 'faked';

  /**
   Clear functions faked, in the order the list names them, none where it names none.
   */
  readonly clears: readonly string[];
};

/**
 A `toFake` list that could not be read.
 */
type UnreadableToFake = {
  /**
   Which reading this is.
   */
  readonly kind: 'unreadable';

  /**
   Why the scan cannot read it, as the finding words it.
   */
  readonly reason: string;
};

/**
 What the `toFake` list of a clock says: why it cannot be read, or the clear functions it fakes.
 */
type ToFakeReading = FakedClears | UnreadableToFake;

/**
 The reading of a clock that fakes every timer, as one with no `toFake` list or an empty one does.
 */
const EVERY_TIMER_FAKED: ToFakeReading = {
  kind: 'faked',
  clears: CLEARED_TIMERS,
};

/**
 Why the `toFake` list of a clock cannot be read, or which clear functions it fakes.

 @param toFake - value of the `toFake` property

 @returns Reason it cannot be read, or the clear functions it names in the order it lists them (all of them for an
 empty list, none for `['Date']`)

 @example
 ```ts
 const reading = toFakeReading({ toFake: property.value, },); // faked, with no clears, for ['Date']
 ```
 */
function toFakeReading({ toFake, }: { readonly toFake: unknown; },): ToFakeReading {
  /**
   The value without wrappers.
   */
  const { inner: list, } = unwrapped({ node: toFake, },);
  if ((!isTreeNode(list,)) || (list.type !== 'ArrayExpression')) {
    return {
      kind: 'unreadable',
      reason: 'passes a toFake that is no written-out list, so the scan cannot read it',
    };
  }
  /**
   The list's elements.
   */
  const elements = childNodes({ node: list, },);
  if (!elements.every(function isString(element,): boolean {
    return literalText({ node: element, },) !== '';
  },)) {
    return {
      kind: 'unreadable',
      reason: 'passes a toFake element that is no string literal, so the scan cannot read it',
    };
  }
  // The library fakes every timer for an empty list, so an empty list names every clear function too.
  if (elements.length === 0)
    return EVERY_TIMER_FAKED;
  return {
    kind: 'faked',
    clears: elements
      .map(function nameOf(element,): string {
        return literalText({ node: element, },);
      },)
      .filter(function isClear(name,): boolean {
        return (CLEARED_TIMERS as readonly string[]).includes(name,);
      },),
  };
}

/**
 Why a call's clock may leave a native timer uncleared, or that it cannot.

 @param node - call of `useFakeTimers`

 @returns Reason, empty where the clock fakes no clear function or clears native timers

 @example
 ```ts
 const refusal = refusalOf({ node, },); // 'passes no options, so every timer is faked ...'
 ```
 */
function refusalOf({ node, }: { readonly node: TreeNode; },): string {
  /**
   The arguments the call passes.
   */
  const args = node.arguments as readonly unknown[];
  if (args.length === 0)
    return 'passes no options, so every timer is faked';
  /**
   The options without wrappers.
   */
  const { inner: options, } = unwrapped({ node: args[0], },);
  if ((!isTreeNode(options,)) || (options.type !== 'ObjectExpression'))
    return 'passes options that are no object literal, so the scan cannot read them';
  /**
   The options' properties.
   */
  const properties = options.properties as readonly TreeNode[];
  if (properties.some(function isSpread(property,): boolean {
    return property.type === 'SpreadElement';
  },))
    return 'passes options with a spread, so the scan cannot read them';
  /**
   The `toFake` property, absent where every timer is faked.
   */
  const [toFake,] = propertiesNamed({
    properties,
    name: 'toFake',
  },);
  /**
   What the clock fakes of the clear functions.
   */
  const reading = (toFake === undefined)
    ? EVERY_TIMER_FAKED
    : toFakeReading({ toFake: toFake.value, },);
  if (reading.kind === 'unreadable')
    return reading.reason;
  /**
   The first clear function the clock fakes, absent where it fakes none.
   */
  const [fakedClear,] = reading.clears;
  if (fakedClear === undefined)
    return '';
  /**
   The clearing option.
   */
  const [option,] = propertiesNamed({
    properties,
    name: CLEAR_OPTION,
  },);
  if (option === undefined)
    return `fakes ${fakedClear} without ${CLEAR_OPTION}`;
  /**
   The option's value without wrappers.
   */
  const { inner: value, } = unwrapped({ node: option.value, },);
  return (isTreeNode(value,) && (value.type === 'Literal') && (value.value === true))
    ? ''
    : `fakes ${fakedClear} but sets ${CLEAR_OPTION} to something other than true`;
}

/**
 Every call that installs a fake clock in the files given.

 @param files - files read, tests and fixtures among them

 @returns One entry per call, by file in input order and by source order within a file

 @example
 ```ts
 const installs = installsIn({ files, },);
 ```
 */
function installsIn({ files, }: { readonly files: readonly SourceText[]; },): readonly Install[] {
  return files.flatMap(function installsOfFile(file,): readonly Install[] {
    return nodesUnder({ root: parseSource({ file, },)
      .program, },)
      .filter(function isCall(node,): boolean {
        return isInstall({ node, },);
      },)
      .toSorted(function bySource(left, right,): number {
        return left.start - right.start;
      },)
      .map(function readInstall(node,): Install {
        return {
          place: `${file.path}:${lineOf({ file, node, },)}`,
          refusal: refusalOf({ node, },),
        };
      },);
  },);
}

/**
 Findings for the calls that install a fake clock which leaves native timers uncleared.

 @param files - files read, tests and fixtures among them

 @returns One finding per such call, naming the file, the line and the reason

 @example
 ```ts
 const found = unclearedInstalls({ files, },);
 ```
 */
function unclearedInstalls({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  return installsIn({ files, },)
    .filter(function isRefused({ refusal, },): boolean {
      return refusal !== '';
    },)
    .map(function findingOf({ place, refusal, },): string {
      return `${place}: ${INSTALL_NAME} ${refusal}`;
    },);
}

//endregion Fake clock installs

await describe({
  name: 'fake clocks clear the native timers they meet (ledger B837)',
  children: [
    it({
      name: 'FINDS a fake clock that fakes clearTimeout, clearInterval or clearImmediate without '
        + 'shouldClearNativeTimers set to true, in every form of call and of options, and leaves a clock that sets '
        + 'it, a clock that fakes the date alone and a method of the same name that installs nothing',
      fn: async () => {
        expect(unclearedInstalls({
          files: [
            {
              path: 'litter.ts',
              text: [
                'export const nap = sinon.useFakeTimers();',
                'export const purr = sinon.useFakeTimers({ toFake: [\'setTimeout\', \'clearTimeout\',], },);',
                'export const knead = ctx.sinon.useFakeTimers({ now: 0, },);',
                'export const groom = useFakeTimers({ shouldClearNativeTimers: false, },);',
                'export const stretch = sandbox[\'useFakeTimers\']({ \'toFake\': [\'clearTimeout\',], },);',
                'export const pounce = sinon.useFakeTimers(options,);',
                'export const hunt = sinon.useFakeTimers({ ...options, },);',
                'export const stalk = sinon.useFakeTimers({ toFake: faked, },);',
                'export const prowl = sinon.useFakeTimers({ toFake: [faked, \'setTimeout\',], },);',
                'export const yawn = sinon.useFakeTimers({ shouldClearNativeTimers: !!flag, },);',
                'export const roll = (sinon as Sandbox).useFakeTimers({ toFake: [\'clearTimeout\',], } as const,);',
                'export const lounge = sinon.useFakeTimers({ toFake: [], },);',
                'export const chase = sinon.useFakeTimers({ toFake: [\'setInterval\', \'clearInterval\',], },);',
                'export const pat = sinon.useFakeTimers({ toFake: [\'setImmediate\', \'clearImmediate\',], },);',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'kitten.unit.test.ts',
              text: [
                'export const clean = sinon.useFakeTimers({',
                '  toFake: [\'setTimeout\', \'clearTimeout\',],',
                '  shouldClearNativeTimers: true,',
                '},);',
                'export const bare = useFakeTimers({ shouldClearNativeTimers: true, },);',
                'export const dated = sinon.useFakeTimers({ now: new Date(0,), toFake: [\'Date\',], },);',
                'export const emptied = sinon.useFakeTimers({ toFake: [], shouldClearNativeTimers: true, },);',
                'export const ticking = sinon.useFakeTimers({ toFake: [\'setTimeout\', \'performance\',], },);',
                'export const looping = sinon.useFakeTimers({ toFake: [\'setInterval\', \'setImmediate\',], },);',
                'export const chased = sinon.useFakeTimers({',
                '  toFake: [\'setInterval\', \'clearInterval\', \'setImmediate\', \'clearImmediate\',],',
                '  shouldClearNativeTimers: true,',
                '},);',
                'export const quoted = sinon.useFakeTimers({ \'shouldClearNativeTimers\': true, } satisfies Options,);',
                'export const text = \'sinon.useFakeTimers()\';',
                '// sinon.useFakeTimers()',
                'export const other = clock.install();',
                'export const field = sinon.useFakeTimers;',
              ].join('\n',),
              isTest: true,
            },
          ],
        },),).toEqual([
          'litter.ts:1: useFakeTimers passes no options, so every timer is faked',
          'litter.ts:2: useFakeTimers fakes clearTimeout without shouldClearNativeTimers',
          'litter.ts:3: useFakeTimers fakes clearTimeout without shouldClearNativeTimers',
          'litter.ts:4: useFakeTimers fakes clearTimeout but sets shouldClearNativeTimers to something other than true',
          'litter.ts:5: useFakeTimers fakes clearTimeout without shouldClearNativeTimers',
          'litter.ts:6: useFakeTimers passes options that are no object literal, so the scan cannot read them',
          'litter.ts:7: useFakeTimers passes options with a spread, so the scan cannot read them',
          'litter.ts:8: useFakeTimers passes a toFake that is no written-out list, so the scan cannot read it',
          'litter.ts:9: useFakeTimers passes a toFake element that is no string literal, so the scan cannot read it',
          'litter.ts:10: useFakeTimers fakes clearTimeout but sets shouldClearNativeTimers to something other than '
            + 'true',
          'litter.ts:11: useFakeTimers fakes clearTimeout without shouldClearNativeTimers',
          'litter.ts:12: useFakeTimers fakes clearTimeout without shouldClearNativeTimers',
          'litter.ts:13: useFakeTimers fakes clearInterval without shouldClearNativeTimers',
          'litter.ts:14: useFakeTimers fakes clearImmediate without shouldClearNativeTimers',
        ],);
      },
    },),
    it({
      name: 'READS EVERY INSTALL OF A FAKE CLOCK IN THE FAMILY\'S SOURCE and finds none that leaves native timers '
        + 'uncleared',
      fn: async () => {
        /**
         Every package file, tests and fixtures among them.
         */
        const files = await readPackageSource();
        /**
         Every install the scan reached.
         */
        const installs = installsIn({ files, },);
        expect(installs.some(function isPaymentHoldInstall({ place, },): boolean {
          return place.startsWith('run/corpus-run/run-seats-payment-hold.unit.test.ts:',);
        },),).toBe(true,);
        expectNoFindings({ findings: unclearedInstalls({ files, },), },);
      },
    },),
  ],
},);
