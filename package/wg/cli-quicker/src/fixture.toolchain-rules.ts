import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  type PathAliasPair,
  pathSpellings,
  type ToolchainRuleRoots,
} from '../dist/final/node/opensnitch-toolchain-rule-patterns.mjs';

/**
 Host alias the real machine uses for homes.
 */
export const HOME_ALIAS: readonly PathAliasPair[] = [[
  '/home',
  '/var/home',
],];

/**
 Roots for a fictional user whose home has both spellings.
 */
export const ROOTS: ToolchainRuleRoots = {
  dataHomes: pathSpellings({
    path: '/var/home/tester/.local/share',
    aliasPairs: HOME_ALIAS,
  },),
  cargoHomes: pathSpellings({
    path: '/var/home/tester/.cargo',
    aliasPairs: HOME_ALIAS,
  },),
  rustupHomes: pathSpellings({
    path: '/var/home/tester/.rustup',
    aliasPairs: HOME_ALIAS,
  },),
  repositoryRoots: pathSpellings({
    path: '/var/home/tester/Monochromatic',
    aliasPairs: HOME_ALIAS,
  },),
};

/**
 Fixed moment so rewritten timestamps are predictable.
 */
export const NOW: Date = new Date('2026-10-31T12:00:00Z',);

/**
 Parsed JSON object of one rule file.
 */
export type FixtureRule = Readonly<Record<string, unknown>>;

/**
 Disposable directory removed on disposal.
 */
export type Scratch = AsyncDisposable & {
  /**
   Absolute directory path.
   */
  readonly path: string;
};

/**
 Creates a disposable scratch directory under the test scratch root.

 @param prefix - Directory name prefix.

 @returns Directory that removes itself on disposal.

 @example
 ```ts
 await using scratch = await createScratch({ prefix: 'wg-quicker-x-' });
 ```
 */
export async function createScratch({ prefix, }: { readonly prefix: string; },): Promise<Scratch> {
  /**
   Fresh directory path.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    prefix,
  ),);
  return {
    path,
    [Symbol.asyncDispose]: async function dispose(): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Disposable rules directory and a sibling backup path.
 */
export type RulesFixture = AsyncDisposable & {
  /**
   Rules directory the daemon would watch.
   */
  readonly directory: string;

  /**
   Backup directory path that does not exist yet.
   */
  readonly backups: string;
};

/**
 Creates a disposable rules directory.

 @returns Fixture with a rules directory and a sibling backup path.

 @example
 ```ts
 await using fixture = await createRulesFixture();
 ```
 */
export async function createRulesFixture(): Promise<RulesFixture> {
  /**
   Scratch directory holding both paths.
   */
  const scratch = await createScratch({ prefix: 'wg-quicker-toolchain-rules-', },);
  /**
   Rules directory inside the scratch.
   */
  const directory = join(
    scratch.path,
    'rules',
  );
  await mkdir(directory,);
  return {
    directory,
    backups: join(
      scratch.path,
      'backups',
    ),
    [Symbol.asyncDispose]: async function dispose(): Promise<void> {
      await scratch[Symbol.asyncDispose]();
    },
  };
}

/**
 Writes a daemon-style rule file the way OpenSnitch saves an auto-answered prompt.

 @param directory - Rules directory.

 @param rule - Rule fields overriding the auto-saved reject defaults.

 @returns Rule name.

 @example
 ```ts
 await writeDaemonRule({ directory, rule: { name: 'reject-always-simple-usr-bin-x' } });
 ```
 */
export async function writeDaemonRule(
  {
    directory,
    rule,
  }: {
    readonly directory: string;
    readonly rule: FixtureRule & { readonly name: string; };
  },
): Promise<string> {
  await writeFile(
    join(
      directory,
      `${rule.name}.json`,
    ),
    `${
      JSON.stringify(
        {
          created: '2026-08-28T21:06:32-04:00',
          updated: '2026-08-28T21:06:32-04:00',
          description: '',
          action: 'reject',
          duration: 'always',
          operator: {
            operand: 'process.path',
            data: '/usr/bin/x',
            type: 'simple',
            list: [],
            sensitive: false,
          },
          enabled: true,
          precedence: false,
          nolog: false,
          ...rule,
        },
        null,
        2,
      )
    }\n`,
    { mode: 0o600, },
  );
  return rule.name;
}

/**
 Builds the operator of an auto-saved exact-path rule.

 @param data - Process path compared.

 @returns Operator object.

 @example
 ```ts
 exactPathOperator({ data: '/usr/bin/x' });
 ```
 */
export function exactPathOperator({ data, }: { readonly data: string; },): FixtureRule {
  return {
    operand: 'process.path',
    data,
    type: 'simple',
    list: [],
    sensitive: false,
  };
}

/**
 Reads one rule file as parsed JSON.

 @param directory - Rules directory.

 @param name - Rule name.

 @returns Parsed rule.

 @example
 ```ts
 await readRule({ directory, name });
 ```
 */
export async function readRule(
  {
    directory,
    name,
  }: {
    readonly directory: string;
    readonly name: string;
  },
): Promise<FixtureRule> {
  /**
   Parsed file content.
   */
  const parsed: unknown = JSON.parse(await readFile(
    join(
      directory,
      `${name}.json`,
    ),
    'utf8',
  ),);
  if (!isFixtureRule(parsed,))
    throw new TypeError(`Rule ${name} is not a JSON object`,);
  return parsed;
}

/**
 Reports whether a parsed JSON value is a rule-shaped object.

 @param value - Unknown parsed JSON.

 @returns Whether the value is a non-null, non-array object.

 @example
 ```ts
 isFixtureRule({ name: 'x' }); // true
 ```
 */
function isFixtureRule(value: unknown,): value is FixtureRule {
  if ((typeof value) !== 'object')
    return false;
  if (value === null)
    return false;
  return !Array.isArray(value,);
}

/**
 Sentinel for a connection no rule matches.
 */
export const NO_RULE_MATCHES: unique symbol = Symbol('wg-quicker fixture rule selection found nothing',);

/**
 Evaluates a rule's operator against a process path the way OpenSnitch 1.8.0 does.

 @param rule - Parsed rule.

 @param processPath - Executable path of the connecting process.

 @returns Whether the operator matches.

 @example
 ```ts
 operatorMatches({ rule, processPath: '/usr/bin/curl' });
 ```
 */
function operatorMatches(
  {
    rule,
    processPath,
  }: {
    readonly rule: FixtureRule;
    readonly processPath: string;
  },
): boolean {
  /**
   Operator object of the rule.
   */
  const operator = isFixtureRule(rule.operator,) ? rule.operator : {};
  /**
   Whether the daemon compares case-sensitively.
   */
  const sensitive = operator.sensitive === true;
  /**
   Compared text, lowercased by the daemon when insensitive.
   */
  const subject = sensitive ? processPath : processPath.toLowerCase();
  /**
   Rule data, lowercased by the daemon before compiling when insensitive.
   */
  const data = sensitive ? String(operator.data,) : String(operator.data,)
    .toLowerCase();
  if (operator.type !== 'regexp')
    return data === subject;
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- The fixture compiles the generated daemon pattern as the daemon does; input is a short fixed test path and the generated pattern has no nested quantifiers.
  return new RegExp(
    data,
    'u',
  ).test(subject,);
}

/**
 Model of OpenSnitch 1.8.0 rule selection.

 Mirrors `daemon/rule/loader.go`: `sortRules` sorts enabled rule names bytewise (lines 369 to 377) and
 `FindFirstMatch` (lines 497 to 515) returns at once on a matching reject, deny or precedence rule,
 otherwise the last matching rule.
 Only `process.path` operators are modelled, which is all these rules use.

 @param directoryRules - Parsed rule objects.

 @param processPath - Executable path of the connecting process.

 @returns The winning rule name,
 or the sentinel when no rule matches.

 @example
 ```ts
 firstMatch({ directoryRules: [], processPath: '/usr/bin/curl' });
 ```
 */
export function firstMatch(
  {
    directoryRules,
    processPath,
  }: {
    readonly directoryRules: readonly FixtureRule[];
    readonly processPath: string;
  },
): string | typeof NO_RULE_MATCHES {
  /**
   Enabled rules in the daemon's iteration order.
   */
  const ordered = directoryRules
    .filter(function isEnabled(rule,): boolean {
      return rule.enabled === true;
    },)
    .toSorted(function byName(
      left,
      right,
    ): number {
      return String(left.name,) < String(right.name,) ? -1 : 1;
    },);
  return ordered.reduce<string | typeof NO_RULE_MATCHES>(
    function select(
      selected,
      rule,
    ): string | typeof NO_RULE_MATCHES {
      /**
       Whether an earlier reject, deny or precedence rule already ended the walk.
       */
      const ended = (selected !== NO_RULE_MATCHES)
        && ordered.some(function isStop(candidate,): boolean {
          return (candidate.name === selected)
            && ((candidate.action === 'reject') || (candidate.action === 'deny')
              || (candidate.precedence === true));
        },);
      if (ended)
        return selected;
      return operatorMatches({
        rule,
        processPath,
      },) ? String(rule.name,) : selected;
    },
    NO_RULE_MATCHES,
  );
}
