import {
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  join,
  resolve,
} from 'node:path';
import { pathToFileURL, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  fixtureConfigPath,
  fixtureSourceRoot,
  type OxlintRuleDiagnostic,
  resolveFixtureTarget,
  runOxlintFixture,
} from '@monochromatic-dev/oxlint-plugin-test-support/ts';

//region Helpers

/** Spread and copy rules covered by this file. */
type SpreadRule = 'no-useless-spread' | 'prefer-spread';

/** Fault injection that makes every native TypeScript spawn fail as an out-of-memory spawn does. */
const REFUSE_NATIVE_SPAWN = pathToFileURL(resolve(
  import.meta.dirname,
  'refuse-native-api-spawn.test-fixture.ts',
),).href;

/** Fixture source root. */
const FIXTURES = fixtureSourceRoot({
  fixturePackageName: 'oxlint-no-restricted-syntax',
},);

/**
 Isolated config enabling only one rule, so no native fixer (notably upstream
 `unicorn/no-useless-spread` or `unicorn/prefer-spread`) rewrites sources during `--fix` runs.

 @example
 ```ts
 ruleConfig('prefer-spread');
 ```
 */
function ruleConfig(rule: SpreadRule,): string {
  return fixtureConfigPath({
    fixturePackageName: 'oxlint-no-restricted-syntax',
    fileName: `.oxlintrc.${rule}.fixture.json`,
  },);
}

/**
 Message fragment identifying each diagnostic kind, most specific first.
 */
const KIND_FRAGMENTS: Readonly<Record<SpreadRule, readonly (readonly [string, string,])[]>> = {
  'no-useless-spread': [
    ['spreadArrayInArray', 'Spreading an array literal into another array',],
    ['spreadArrayInArguments', 'Spreading an array literal into an argument list',],
    ['spreadObjectInObject', 'Spreading an object literal into another object',],
    ['iterableToArray', 'accepts any iterable',],
    ['iterableInForOf', '`for…of` iterates any iterable',],
    ['iterableInYieldStar', '`yield*` delegates to any iterable',],
    ['cloneArray', 'already returns a new array',],
    ['cloneObject', 'already returns a new object',],
    ['ambiguousConversion', 'No type information is available here',],
  ],
  'prefer-spread': [
    ['preferSpreadOverArrayFrom', 'so `Array.from()` and a spread build the same array',],
    ['preferSpreadOverCopy', 'only copies it; write `[...receiver]`',],
    ['preferSpreadOverConcat', 'instead of `concat()`',],
    ['ambiguousConcat', '`concat()` is ambiguous',],
    ['ambiguousCopy', 'is ambiguous: it copies an array',],
  ],
};

/**
 Lints a target with one rule and returns the sorted kinds it reported.

 @example
 ```ts
 await reportedKinds({ rule: 'prefer-spread', target });
 ```
 */
async function reportedKinds(
  {
    rule,
    target,
  }: {
    readonly rule: SpreadRule;
    readonly target: string;
  },
): Promise<readonly string[]> {
  /** Plugin diagnostics for the target. */
  const diagnostics = await runOxlintFixture({
    codePrefix: 'no-restricted-syntax(',
    configFlag: '-c',
    fixtureConfig: ruleConfig(rule,),
    target,
  },);
  return diagnostics
    .filter(function ownRule(diagnostic,): boolean {
      return diagnostic.code === `no-restricted-syntax(${rule})`;
    },)
    .map(function kindOf(diagnostic: OxlintRuleDiagnostic,): string {
      /** First fragment the message contains. */
      const entry = KIND_FRAGMENTS[rule].find(function matches([, fragment,],): boolean {
        return diagnostic.message.includes(fragment,);
      },);
      if (entry === undefined)
        throw new Error(`unrecognized ${rule} message: ${diagnostic.message}`,);
      return entry[0];
    },)
    .toSorted();
}

/**
 Lints a fixture under the fixture package's typed project.

 @example
 ```ts
 await fixtureKinds({ rule: 'prefer-spread', fixturePath: 'invalid/prefer-spread.ts' });
 ```
 */
async function fixtureKinds(
  {
    rule,
    fixturePath,
  }: {
    readonly rule: SpreadRule;
    readonly fixturePath: string;
  },
): Promise<readonly string[]> {
  return reportedKinds({
    rule,
    target: resolveFixtureTarget({
      fixtureSourceRoot: FIXTURES,
      fixturePath,
    },),
  },);
}

/** Disposable source file in a temp directory, optionally inside its own TypeScript project. */
type ScratchSource = {
  /** Absolute path to the source file. */
  readonly filePath: string;
  /** Removes the temp directory. */
  readonly [Symbol.dispose]: () => void;
};

/**
 Writes source into a fresh temp directory; `typed` adds a tsconfig so the
 semantic bridge resolves it, otherwise the file sits outside every project.

 @example
 ```ts
 using scratch = scratchSource({ source, typed: true });
 ```
 */
function scratchSource(
  {
    source,
    typed,
  }: {
    readonly source: string;
    readonly typed: boolean;
  },
): ScratchSource {
  /** Unique temp directory. */
  const dirPath = mkdtempSync(join(
    scratchRoot(),
    'oxlint-spread-rules-',
  ),);
  /** Source file path. */
  const filePath = resolve(
    dirPath,
    'case.ts',
  );
  writeFileSync(filePath, source,);
  if (typed) {
    writeFileSync(
      resolve(
        dirPath,
        'tsconfig.json',
      ),
      JSON.stringify({
        compilerOptions: {
          lib: ['ESNext',],
          target: 'esnext',
          module: 'preserve',
          strict: true,
          noEmit: true,
          types: [],
        },
        files: ['case.ts',],
      },),
    );
  }
  return {
    filePath,
    [Symbol.dispose]: function cleanup(): void {
      rmSync(
        dirPath,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Runs `oxlint --fix` with one rule on scratch source and returns the fixed text.
 A plugin that crashes during the run fails it, so the text is never an untouched file read back after a crash.

 @example
 ```ts
 await fixedSource({ rule: 'prefer-spread', source, typed: true });
 ```
 */
async function fixedSource(
  {
    rule,
    source,
    typed,
    refuseSpawn = false,
  }: {
    readonly rule: SpreadRule;
    readonly source: string;
    readonly typed: boolean;
    readonly refuseSpawn?: boolean;
  },
): Promise<string> {
  using scratch = scratchSource({
    source,
    typed,
  },);
  await runOxlintFixture({
    codePrefix: 'no-restricted-syntax(',
    configFlag: '-c',
    fixtureConfig: ruleConfig(rule,),
    target: scratch.filePath,
    fix: 'fixes',
    ...refuseSpawn ? { environment: { NODE_OPTIONS: `--import=${REFUSE_NATIVE_SPAWN}`, }, } : {},
  },);
  return readFileSync(
    scratch.filePath,
    'utf8',
  );
}

/** Smoothie-order declarations shared by scratch sources. */
const ORDER_DECLARATIONS = [
  'type Order = {',
  '  readonly toppings: readonly string[];',
  '  readonly scoopGrams: Uint8Array;',
  '  readonly menuCode: string;',
  '};',
  'declare const order: Order;',
];

/** Spreads that are a copy on arrays and a conversion on typed arrays and strings. */
const AMBIGUOUS_SPREADS = [
  'export const firstTwo = [...order.toppings.slice(0, 2,),];',
  'export const doubled = [...order.scoopGrams.map(Number,),];',
  'export const codes = [...order.menuCode.slice(0, 2,),];',
];

/** Issue 563 reproduction: a code-point spread of a string slice. */
const ISSUE_563_SOURCE = [
  'export function hasA({ text, }: { readonly text: string; },): boolean {',
  '  return [...text.slice(0, 3,),].some(function isA(character,): boolean {',
  "    return character === 'a';",
  '  },);',
  '}',
  '',
]
  .join('\n',);

/** Issue 565 reproduction: a typed-array copy. */
const ISSUE_565_SOURCE = [
  'export function copyScoops({ scoopGrams, }: { readonly scoopGrams: Uint8Array; },): Uint8Array {',
  '  return scoopGrams.slice();',
  '}',
  '',
]
  .join('\n',);

//endregion Helpers

await describe({
  name: 'spread rules',
  children: [
    describe({
      name: 'no-useless-spread',
      children: [
        it({
          name: 'reports every invalid form in a typed project',
          fn: async () => {
            expect(await fixtureKinds({
              rule: 'no-useless-spread',
              fixturePath: 'invalid/no-useless-spread.ts',
            },),)
              .toEqual([
                'ambiguousConversion',
                'cloneArray',
                'cloneArray',
                'cloneArray',
                'cloneArray',
                'cloneObject',
                'iterableInForOf',
                'iterableInYieldStar',
                'iterableToArray',
                'iterableToArray',
                'iterableToArray',
                'spreadArrayInArguments',
                'spreadArrayInArray',
                'spreadObjectInObject',
              ],);
          },
        },),
        it({
          name: 'accepts typed conversions, Object.create, and shadowed globals',
          fn: async () => {
            expect(await fixtureKinds({
              rule: 'no-useless-spread',
              fixturePath: 'valid/no-useless-spread.ts',
            },),)
              .toEqual([],);
          },
        },),
        it({
          name: 'reports ambiguous spreads without a fix when no type is available',
          fn: async () => {
            using scratch = scratchSource({
              source: [...ORDER_DECLARATIONS, ...AMBIGUOUS_SPREADS, '',].join('\n',),
              typed: false,
            },);
            expect(await reportedKinds({
              rule: 'no-useless-spread',
              target: scratch.filePath,
            },),)
              .toEqual(['ambiguousConversion', 'ambiguousConversion', 'ambiguousConversion',],);
          },
        },),
        it({
          name: 'fixes only the proven array copy when types are available',
          fn: async () => {
            /** Scratch source before fixing. */
            const source = [...ORDER_DECLARATIONS, ...AMBIGUOUS_SPREADS, '',].join('\n',);
            expect(await fixedSource({
              rule: 'no-useless-spread',
              source,
              typed: true,
            },),)
              .toBe(source.replace(
                '[...order.toppings.slice(0, 2,),]',
                'order.toppings.slice(0, 2,)',
              ),);
          },
        },),
        it({
          name: 'leaves the issue 563 string spread untouched with or without types',
          fn: async () => {
            expect(await fixedSource({
              rule: 'no-useless-spread',
              source: ISSUE_563_SOURCE,
              typed: false,
            },),)
              .toBe(ISSUE_563_SOURCE,);
            expect(await fixedSource({
              rule: 'no-useless-spread',
              source: ISSUE_563_SOURCE,
              typed: true,
            },),)
              .toBe(ISSUE_563_SOURCE,);
          },
        },),
        it({
          name: 'fails the fix run, not reading the issue 563 source back as untouched, when the TypeScript bridge cannot start',
          fn: async () => {
            /** Failure of the fix run, which stays undefined when the run reads the untouched file back. */
            let caught: unknown;
            try {
              await fixedSource({
                rule: 'no-useless-spread',
                source: ISSUE_563_SOURCE,
                typed: true,
                refuseSpawn: true,
              },);
            }
            catch (error: unknown) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
            expect(String(caught,),).toContain('spawn ENOMEM',);
          },
        },),
        it({
          name: 'fixes certain copies, literal spreads, and iterable consumers',
          fn: async () => {
            /** Source whose every spread is provably removable. */
            const source = [
              'declare const source: Iterable<number>;',
              'export const fromArray = [...Array.from(source,),];',
              'export const sized = [...new Array(3,),];',
              'export const literal = [1, ...[2, 3,], 4,];',
              'export const empty = [1, ...[], 2,];',
              'export const set = new Set([...source,],);',
              "export const entries = { ...Object.fromEntries([['a', 1,],],), };",
              '',
            ]
              .join('\n',);
            expect(await fixedSource({
              rule: 'no-useless-spread',
              source,
              typed: false,
            },),)
              .toBe([
                'declare const source: Iterable<number>;',
                'export const fromArray = Array.from(source,);',
                'export const sized = new Array(3,).fill();',
                'export const literal = [1, 2, 3, 4,];',
                'export const empty = [1,  2,];',
                'export const set = new Set(source,);',
                "export const entries = Object.fromEntries([['a', 1,],],);",
                '',
              ]
                .join('\n',),);
          },
        },),
      ],
    },),
    describe({
      name: 'prefer-spread',
      children: [
        it({
          name: 'reports every invalid form in a typed project',
          fn: async () => {
            expect(await fixtureKinds({
              rule: 'prefer-spread',
              fixturePath: 'invalid/prefer-spread.ts',
            },),)
              .toEqual([
                'ambiguousConcat',
                'ambiguousCopy',
                'preferSpreadOverArrayFrom',
                'preferSpreadOverArrayFrom',
                'preferSpreadOverConcat',
                'preferSpreadOverCopy',
                'preferSpreadOverCopy',
              ],);
          },
        },),
        it({
          name: 'accepts typed-array and string copies, explicit Array.from, and partial slices',
          fn: async () => {
            expect(await fixtureKinds({
              rule: 'prefer-spread',
              fixturePath: 'valid/prefer-spread.ts',
            },),)
              .toEqual([],);
          },
        },),
        it({
          name: 'fixes proven array copies and conversions only',
          fn: async () => {
            /** Source mixing proven arrays with a typed array and a concat. */
            const source = [
              'declare const toppings: readonly string[];',
              'declare const scoopGrams: Uint8Array;',
              'export const copy = toppings.slice();',
              'export const fromArray = Array.from(toppings,);',
              'export const scoopCopy = scoopGrams.slice();',
              "export const joined = toppings.concat(['ice',],);",
              '',
            ]
              .join('\n',);
            expect(await fixedSource({
              rule: 'prefer-spread',
              source,
              typed: true,
            },),)
              .toBe(source
                .replace('toppings.slice()', '[...toppings]',)
                .replace('Array.from(toppings,)', '[...toppings]',),);
          },
        },),
        it({
          name: 'leaves the issue 565 typed-array copy untouched with or without types',
          fn: async () => {
            expect(await fixedSource({
              rule: 'prefer-spread',
              source: ISSUE_565_SOURCE,
              typed: true,
            },),)
              .toBe(ISSUE_565_SOURCE,);
            expect(await fixedSource({
              rule: 'prefer-spread',
              source: ISSUE_565_SOURCE,
              typed: false,
            },),)
              .toBe(ISSUE_565_SOURCE,);
          },
        },),
        it({
          name: 'fails the fix run, not reading the issue 565 source back as untouched, when the TypeScript bridge cannot start',
          fn: async () => {
            /** Failure of the fix run, which stays undefined when the run reads the untouched file back. */
            let caught: unknown;
            try {
              await fixedSource({
                rule: 'prefer-spread',
                source: ISSUE_565_SOURCE,
                typed: true,
                refuseSpawn: true,
              },);
            }
            catch (error: unknown) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
            expect(String(caught,),).toContain('spawn ENOMEM',);
          },
        },),
        it({
          name: 'treats global receivers such as Buffer as static-method namespaces when untyped',
          fn: async () => {
            using untyped = scratchSource({
              source: 'export const bytes = Buffer.concat([],);\n',
              typed: false,
            },);
            expect(await reportedKinds({
              rule: 'prefer-spread',
              target: untyped.filePath,
            },),)
              .toEqual([],);
          },
        },),
        it({
          name: 'reports the issue 565 copy as ambiguous only when no type is available',
          fn: async () => {
            using typed = scratchSource({
              source: ISSUE_565_SOURCE,
              typed: true,
            },);
            using untyped = scratchSource({
              source: ISSUE_565_SOURCE,
              typed: false,
            },);
            expect(await reportedKinds({
              rule: 'prefer-spread',
              target: typed.filePath,
            },),)
              .toEqual([],);
            expect(await reportedKinds({
              rule: 'prefer-spread',
              target: untyped.filePath,
            },),)
              .toEqual(['ambiguousCopy',],);
          },
        },),
      ],
    },),
  ],
},);
