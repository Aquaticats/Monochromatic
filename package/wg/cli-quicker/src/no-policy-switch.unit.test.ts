import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parsePrivilegeContext, } from '../dist/final/node/privilege-context-data.mjs';

/**
 Directory of this package's sources.
 */
const SOURCE_DIRECTORY = import.meta.dirname;

/**
 Directory of the Rust companion's sources.
 */
const COMPANION_SOURCE_DIRECTORY = join(SOURCE_DIRECTORY, '..', '..', 'cli-quicker-exempt', 'src',);

/**
 Every environment variable the production code of the two wg packages touches,
 each with the reason it cannot switch an update policy off.

 A new name fails the test until a reviewer adds it here with a reason.
 A name that matches a policy word below must be in `POLICY_WORD_EXCEPTIONS` instead.
 */
const REVIEWED_ENVIRONMENT: Readonly<Record<string, string>> = {
  PATH: 'Searches the root-owned secure path for tools by name; it never names the process watcher by path.',
  HOME: 'Locates the caller\'s own caches and toolchain directories after a check that the home belongs to the caller.',
  SUDO_UID: 'Identifies the user whose processes the watcher follows; sudo sets it, and no other name selects the user.',
  XDG_CACHE_HOME: 'Locates scratch and cache directories only.',
  XDG_RUNTIME_DIR: 'Set by the watcher for its own bus client so the client reaches the user\'s manager; never read from the caller.',
  IPINFO_TOKEN: 'Credential for the allowed-IPs lookup; it does not affect update routing.',
  WG_ALLOWEDIPS_CACHE_DIRECTORY: 'Locates the allowed-IPs cache only.',
  WG_QUICKER_CALLER_PATH: 'The caller\'s PATH, searched last for a companion installed by name. It can substitute a different watcher when neither the sibling build nor the secure path has one, so it is the same class as the removed overrides and is an open question for the reviewer.',
  WG_QUICKER_OPENSNITCH_DAEMON_CONFIG: 'Locates the OpenSnitch daemon config for the rule maintenance command only.',
  WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG: 'Locates the OpenSnitch firewall config for the rule maintenance command only.',
  WG_QUICKER_RUNTIME_DIRECTORY: 'Locates the directory of the bypass state file; the state is still written and read.',
  WG_QUICKER_TEST_SUDO_RECORD: 'Used by a fake sudo in tests only.',
  WG_QUICKER_EXEMPT_TEST_FORCE_PIN_INVALID: 'Debug builds only (cfg(debug_assertions)); compiled out of release builds, and it makes marking fail loudly.',
  WG_QUICKER_EXEMPT_TEST_FAIL_AFTER_ATTACH: 'Debug builds only (cfg(debug_assertions)); compiled out of release builds, and it makes the holder fail loudly.',
  TMPDIR: 'Locates the private temporary directory of the sudo hand-off only.',
};

/**
 Words that mark a name as able to change what an update policy does.
 */
const POLICY_WORDS: readonly string[] = [
  'EXEMPT',
  'CLEARNET',
  'BYPASS',
  'POLICY',
  'UPDATE',
  'TOOLCHAIN',
  'DISABLE',
  'OPT_OUT',
  'SKIP',
];

/**
 Names that carry a policy word and were reviewed as harmless,
 each with the reason.
 */
const POLICY_WORD_EXCEPTIONS: Readonly<Record<string, string>> = {
  WG_QUICKER_EXEMPT_TEST_FORCE_PIN_INVALID: REVIEWED_ENVIRONMENT.WG_QUICKER_EXEMPT_TEST_FORCE_PIN_INVALID ?? '',
  WG_QUICKER_TEST_SUDO_RECORD: REVIEWED_ENVIRONMENT.WG_QUICKER_TEST_SUDO_RECORD ?? '',
  WG_QUICKER_EXEMPT_TEST_FAIL_AFTER_ATTACH: REVIEWED_ENVIRONMENT.WG_QUICKER_EXEMPT_TEST_FAIL_AFTER_ATTACH ?? '',
};

/**
 Words that name a switch for the update policies in prose or code.
 */
const SWITCH_WORDS: readonly string[] = [
  'MONOCHROMATIC_TOOLCHAIN_CLEARNET',
  'WG_QUICKER_NO_UPDATE',
  'WG_QUICKER_DISABLE',
  'opt-out',
  'optOut',
];

/* oxlint-disable no-restricted-syntax/no-regex -- The scanner finds environment names in source text of this package and its Rust sibling. A regex is clearer than a hand-written parser for these fixed access forms, and each pattern uses bounded character classes with no nested repetition, over files of a few tens of kilobytes. */
/**
 Shape of an environment variable name.
 */
const NAME_SHAPE = /^[A-Z][A-Z0-9]*(?:_[A-Z0-9]+)*$/v;

/**
 Finds the names a TypeScript text reads from the process environment,
 in every form the code base uses:
 member access,
 index access with a literal or a named constant,
 destructuring,
 an alias of `process.env`,
 and the privilege-context allowlist.

 @param text - Source text.

 @returns Names found.

 @example
 ```ts
 typescriptEnvironmentNames({ text: 'process.env.HOME' }); // ['HOME']
 ```
 */
function typescriptEnvironmentNames({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Constants that hold a plain string,
   so `process.env[CONSTANT]` resolves.
   */
  const constants = new Map(
    [...text.matchAll(/(?:export\s+)?const\s+(?<constant>[A-Z][A-Z0-9_]*)\s*=\s*['"](?<value>[^'"]+)['"]/gv,),].map(
      function entry(match,): readonly [string, string] {
        return [match.groups?.constant ?? '', match.groups?.value ?? '',];
      },
    ),
  );
  /**
   Names of aliases bound to `process.env`.
   */
  const aliases = [...text.matchAll(/const\s+(?<alias>[A-Za-z_]\w*)\s*=\s*process\.env\s*;/gv,),].map(
    function aliasName(match,): string {
      return match.groups?.alias ?? '';
    },
  );
  /**
   Objects whose members are read as environment names.
   */
  const bases = [String.raw`process\s*\.\s*env`, ...aliases,];
  /**
   Names found by member access.
   */
  const members = bases.flatMap(function memberNames(base,): readonly string[] {
    return [...text.matchAll(new RegExp(`${base}\\s*\\.\\s*(?<name>[A-Za-z_]\\w*)`, 'gv',),),].map(
      function nameOf(match,): string {
        return match.groups?.name ?? '';
      },
    );
  },);
  /**
   Names found by index access.
   */
  const indexed = bases.flatMap(function indexNames(base,): readonly string[] {
    return [...text.matchAll(new RegExp(`${base}\\s*\\[\\s*(?<key>[^\\]]+?)\\s*\\]`, 'gv',),),].map(
      function keyOf(match,): string {
        /**
         Raw key text.
         */
        const key = match.groups?.key ?? '';
        if (/^['"]/v.test(key,))
          return key.slice(1, -1,);
        return constants.get(key,) ?? '';
      },
    );
  },);
  /**
   Names found by destructuring.
   */
  const destructured = [...text.matchAll(/\{(?<names>[^\}]*)\}\s*=\s*process\.env/gv,),].flatMap(
    function destructuredNames(match,): readonly string[] {
      return (match.groups?.names ?? '').split(',',)
        .map(function firstWord(part,): string {
          return part.trim()
            .split(':',)[0]
            ?.trim() ?? '';
        },);
    },
  );
  /**
   Names in the privilege-context allowlist.
   */
  const allowlisted = [...text.matchAll(/ALLOWED_ENVIRONMENT_KEYS\s*=\s*new Set\(\[(?<body>[^\]]*)\]/gv,),].flatMap(
    function allowlistNames(match,): readonly string[] {
      return [...(match.groups?.body ?? '').matchAll(/'(?<name>[^']+)'/gv,),].map(function nameOf(item,): string {
        return item.groups?.name ?? '';
      },);
    },
  );
  /**
   Other literals that look like this tool's own variable names.
   */
  const literals = [...text.matchAll(/['"](?<name>(?:WG|MONOCHROMATIC)_[A-Z0-9_]+)['"]/gv,),].map(
    function nameOf(match,): string {
      return match.groups?.name ?? '';
    },
  );
  return [...members, ...indexed, ...destructured, ...allowlisted, ...literals,].filter(
    function isName(name,): boolean {
      return NAME_SHAPE.test(name,);
    },
  );
}

/**
 Finds the names a Rust text reads or sets in the process environment.

 @param text - Source text.

 @returns Names found.

 @example
 ```ts
 rustEnvironmentNames({ text: 'std::env::var_os("HOME")' }); // ['HOME']
 ```
 */
function rustEnvironmentNames({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Constants that hold a plain string.
   */
  const constants = new Map(
    [...text.matchAll(/const\s+(?<constant>[A-Z][A-Z0-9_]*)\s*:\s*&str\s*=\s*"(?<value>[^"]+)"/gv,),].map(
      function entry(match,): readonly [string, string] {
        return [match.groups?.constant ?? '', match.groups?.value ?? '',];
      },
    ),
  );
  /**
   Names read with a literal or a constant.
   */
  const read = [...text.matchAll(/env::var(?:_os)?\(\s*(?<key>"[^"]+"|[A-Z][A-Z0-9_]*)\s*\)/gv,),].map(
    function keyOf(match,): string {
      /**
       Raw key text.
       */
      const key = match.groups?.key ?? '';
      return key.startsWith('"',) ? key.slice(1, -1,) : constants.get(key,) ?? '';
    },
  );
  /**
   Names set for a child process.
   */
  const set = [...text.matchAll(/\.env\(\s*"(?<name>[^"]+)"/gv,),].map(function nameOf(match,): string {
    return match.groups?.name ?? '';
  },);
  /**
   Other literals that look like this tool's own variable names.
   */
  const literals = [...text.matchAll(/"(?<name>(?:WG|MONOCHROMATIC)_[A-Z0-9_]+)"/gv,),].map(
    function nameOf(match,): string {
      return match.groups?.name ?? '';
    },
  );
  return [...read, ...set, ...literals,].filter(function isName(name,): boolean {
    return NAME_SHAPE.test(name,);
  },);
}

/* oxlint-enable no-restricted-syntax/no-regex */

/**
 Reads the production source files of a directory.

 @param directory - Directory to read.

 @param extension - File extension including the dot.

 @returns File name and text pairs.

 @example
 ```ts
 await productionSources({ directory, extension: '.ts' });
 ```
 */
async function productionSources(
  {
    directory,
    extension,
  }: {
    readonly directory: string;
    readonly extension: string;
  },
): Promise<readonly (readonly [string, string])[]> {
  return Promise.all(
    (await readdir(directory,))
      .filter(function isProduction(name,): boolean {
        return name.endsWith(extension,) && (!name.includes('test',)) && (!name.includes('fixture',));
      },)
      .map(async function read(name,): Promise<readonly [string, string]> {
        return [name, await readFile(join(directory, name,), 'utf8',),];
      },),
  );
}

/**
 Every environment name the production code of both packages touches,
 with the files that touch it.

 @returns Map from name to file names.

 @example
 ```ts
 await productionEnvironment();
 ```
 */
async function productionEnvironment(): Promise<ReadonlyMap<string, readonly string[]>> {
  /**
   Names by file.
   */
  const found = [
    ...(await productionSources({ directory: SOURCE_DIRECTORY, extension: '.ts', },)).map(
      function scan([name, text,],): readonly [string, readonly string[]] {
        return [name, typescriptEnvironmentNames({ text, },),];
      },
    ),
    ...(await productionSources({ directory: COMPANION_SOURCE_DIRECTORY, extension: '.rs', },)).map(
      function scan([name, text,],): readonly [string, readonly string[]] {
        return [name, rustEnvironmentNames({ text, },),];
      },
    ),
  ];
  /**
   Names with their files.
   */
  const byName = new Map<string, string[]>();
  for (const [file, names,] of found) {
    for (const name of names)
      byName.set(name, [...byName.get(name,) ?? [], file,],);
  }
  return byName;
}

/**
 Reports whether a name carries a policy word.

 @param name - Environment variable name.

 @returns Whether a word of the policy vocabulary appears in it.

 @example
 ```ts
 hasPolicyWord({ name: 'WG_QUICKER_EXEMPT_COMMAND' }); // true
 ```
 */
function hasPolicyWord({ name, }: { readonly name: string; },): boolean {
  return POLICY_WORDS.some(function appears(word,): boolean {
    return name.includes(word,);
  },);
}

await describe({
  name: 'the scanner for environment names',
  children: [
    it({
      name: 'finds a name in every access form the code base uses',
      fn: async () => {
        expect(typescriptEnvironmentNames({ text: 'const x = process.env.A_ONE;', },),).toContain('A_ONE',);
        expect(typescriptEnvironmentNames({ text: 'const x = process.env\n  .B_TWO\n  ?? 1;', },),).toContain('B_TWO',);
        expect(typescriptEnvironmentNames({ text: 'const x = process.env[\'C_THREE\'];', },),).toContain('C_THREE',);
        expect(typescriptEnvironmentNames({ text: 'const KEY = \'D_FOUR\';\nconst x = process.env[KEY];', },),).toContain('D_FOUR',);
        expect(typescriptEnvironmentNames({ text: 'const { E_FIVE: five, F_SIX, } = process.env;', },),).toContain('E_FIVE',);
        expect(typescriptEnvironmentNames({ text: 'const { E_FIVE: five, F_SIX, } = process.env;', },),).toContain('F_SIX',);
        expect(typescriptEnvironmentNames({ text: 'const environment = process.env;\nenvironment.G_SEVEN = x;', },),).toContain('G_SEVEN',);
        expect(typescriptEnvironmentNames({ text: 'const ALLOWED_ENVIRONMENT_KEYS = new Set([\n  \'H_EIGHT\',\n],);', },),).toContain('H_EIGHT',);
        expect(typescriptEnvironmentNames({ text: 'run(\'WG_QUICKER_I_NINE\')', },),).toContain('WG_QUICKER_I_NINE',);
        expect(rustEnvironmentNames({ text: 'std::env::var_os("J_TEN")', },),).toContain('J_TEN',);
        expect(rustEnvironmentNames({ text: 'const K: &str = "L_ELEVEN";\nstd::env::var(K)', },),).toContain('L_ELEVEN',);
        expect(rustEnvironmentNames({ text: 'command.env("M_TWELVE", x)', },),).toContain('M_TWELVE',);
      },
    },),
    it({
      name: 'finds the names the code base is known to use, so an empty scan cannot pass',
      fn: async () => {
        /**
         Names found in production.
         */
        const found = await productionEnvironment();
        for (
          const name of [
            'SUDO_UID',
            'HOME',
            'XDG_CACHE_HOME',
            'WG_QUICKER_RUNTIME_DIRECTORY',
            'WG_QUICKER_CALLER_PATH',
            'XDG_RUNTIME_DIR',
            'WG_QUICKER_EXEMPT_TEST_FORCE_PIN_INVALID',
          ]
        )
          expect(`${name}: ${String(found.has(name,),)}`,).toBe(`${name}: true`,);
      },
    },),
  ],
},);

await describe({
  name: 'always-on policy',
  children: [
    it({
      name: 'reads no environment variable that was not reviewed, in either wg package',
      fn: async () => {
        /**
         Names found in production.
         */
        const found = await productionEnvironment();
        /**
         Names missing from the reviewed list,
         with their files.
         */
        const unreviewed = [...found.entries(),]
          .filter(function isUnreviewed([name,],): boolean {
            return !(name in REVIEWED_ENVIRONMENT);
          },)
          .map(function describeName([name, files,],): string {
            return `${name} (${[...new Set(files,),].join(', ',)})`;
          },);
        // A name listed here is an environment variable nobody reviewed for switches: add it to REVIEWED_ENVIRONMENT with a reason.
        expect(unreviewed,).toEqual([],);
      },
    },),
    it({
      name: 'keeps no reviewed name that production no longer reads',
      fn: async () => {
        /**
         Names found in production.
         */
        const found = await productionEnvironment();
        /**
         Reviewed names nothing reads.
         */
        const stale = Object.keys(REVIEWED_ENVIRONMENT,).filter(function isStale(name,): boolean {
          return (!found.has(name,)) && (name !== 'WG_QUICKER_TEST_SUDO_RECORD');
        },);
        expect(stale,).toEqual([],);
      },
    },),
    it({
      name: 'has no environment variable whose name carries a policy word, other than the reviewed exceptions',
      fn: async () => {
        /**
         Names found in production.
         */
        const found = await productionEnvironment();
        /**
         Names that can change what a policy does.
         */
        const offenders = [...found.keys(),].filter(function isOffender(name,): boolean {
          return hasPolicyWord({ name, },) && (!(name in POLICY_WORD_EXCEPTIONS));
        },);
        expect(offenders,).toEqual([],);
        for (const reason of Object.values(POLICY_WORD_EXCEPTIONS,))
          expect(reason.length,).toBeGreaterThan(20,);
      },
    },),
    it({
      name: 'does not carry a policy-affecting name across sudo, whatever its spelling',
      fn: async () => {
        /**
         Names to try:
         the two removed overrides,
         the removed opt-out,
         and any other policy-worded name production mentions.
         */
        const candidates = new Set([
          'WG_QUICKER_EXEMPT_COMMAND',
          'WG_QUICKER_EXEMPT_UID',
          'MONOCHROMATIC_TOOLCHAIN_CLEARNET',
          'WG_QUICKER_BYPASS_OFF',
          ...[...(await productionEnvironment()).keys(),].filter(function isPolicyWorded(name,): boolean {
            return hasPolicyWord({ name, },);
          },),
        ],);
        for (const name of candidates) {
          expect(() => parsePrivilegeContext({
            text: JSON.stringify({
              environment: {
                HOME: '/home/caller',
                [name]: '1',
              },
              uid: 1_000,
              version: 1,
            },),
          },),).toThrow('environment is invalid',);
        }
      },
    },),
    it({
      name: 'names no switch, flag or opt-out in either wg package',
      fn: async () => {
        /**
         Production sources of both wg packages.
         */
        const sources = [
          ...await productionSources({ directory: SOURCE_DIRECTORY, extension: '.ts', },),
          ...await productionSources({ directory: COMPANION_SOURCE_DIRECTORY, extension: '.rs', },),
        ];
        /**
         Sources that mention a switch word.
         */
        const offenders = sources.flatMap(function mentionsSwitch([name, text,],): readonly string[] {
          return SWITCH_WORDS.filter(function appears(word,): boolean {
            return text.includes(word,);
          },).map(function describeHit(word,): string {
            return `${name}: ${word}`;
          },);
        },);
        expect(offenders,).toEqual([],);
      },
    },),
  ],
},);
