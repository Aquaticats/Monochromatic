import {
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { resolveOpenSnitchRulesDirectory, } from '../dist/final/node/opensnitch-daemon-config.mjs';
import { parseOpenSnitchRulesArgs, } from '../dist/final/node/opensnitch-rules-command.mjs';
import {
  pathSpellings,
  TOOLCHAIN_RULE_NAME_PREFIX,
} from '../dist/final/node/opensnitch-toolchain-rule-patterns.mjs';
import {
  applyToolchainRules,
  inspectToolchainRules,
  removeSupersededRejects,
  removeToolchainRules,
} from '../dist/final/node/opensnitch-toolchain-rules.mjs';

/**
 Roots for a fictional user whose home has both spellings.
 */
const ROOTS = {
  dataHomes: pathSpellings({ path: '/var/home/tester/.local/share', aliasPairs: [['/home', '/var/home',],], },),
  cargoHomes: pathSpellings({ path: '/var/home/tester/.cargo', aliasPairs: [['/home', '/var/home',],], },),
  rustupHomes: pathSpellings({ path: '/var/home/tester/.rustup', aliasPairs: [['/home', '/var/home',],], },),
  repositoryRoots: pathSpellings({ path: '/var/home/tester/Monochromatic', aliasPairs: [['/home', '/var/home',],], },),
};

/**
 Fixed moment so rewritten timestamps are predictable.
 */
const NOW = new Date(Date.UTC(2026, 9, 31, 12, 0, 0,),);

/**
 Disposable rules directory removed on disposal.
 */
type Fixture = AsyncDisposable & {
  readonly directory: string;
  readonly backups: string;
};

/**
 Creates a disposable rules directory.

 @returns Fixture with a rules directory and a sibling backup path.

 @example
 ```ts
 await using fixture = await createFixture();
 ```
 */
async function createFixture(): Promise<Fixture> {
  const base = await mkdtemp(join(scratchRoot(), 'wg-quicker-toolchain-rules-',),);
  const directory = join(base, 'rules',);
  await mkdir(directory,);
  return {
    directory,
    backups: join(base, 'backups',),
    [Symbol.asyncDispose]: async function dispose(): Promise<void> {
      await rm(base, { recursive: true, force: true, },);
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
async function writeDaemonRule(
  {
    directory,
    rule,
  }: {
    readonly directory: string;
    readonly rule: Readonly<Record<string, unknown>> & { readonly name: string; };
  },
): Promise<string> {
  await writeFile(
    join(directory, `${rule.name}.json`,),
    `${JSON.stringify({
      created: '2026-08-28T21:06:32-04:00',
      updated: '2026-08-28T21:06:32-04:00',
      description: '',
      action: 'reject',
      duration: 'always',
      operator: { operand: 'process.path', data: '/usr/bin/x', type: 'simple', list: [], sensitive: false, },
      enabled: true,
      precedence: false,
      nolog: false,
      ...rule,
    }, null, 2,)}\n`,
    { mode: 0o600, },
  );
  return rule.name;
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
async function readRule(
  {
    directory,
    name,
  }: {
    readonly directory: string;
    readonly name: string;
  },
): Promise<Record<string, unknown>> {
  return JSON.parse(await readFile(join(directory, `${name}.json`,), 'utf8',),) as Record<string, unknown>;
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
 or `undefined` when no rule matches.

 @example
 ```ts
 firstMatch({ directoryRules: [], processPath: '/usr/bin/curl' });
 ```
 */
function firstMatch(
  {
    directoryRules,
    processPath,
  }: {
    readonly directoryRules: readonly Record<string, any>[];
    readonly processPath: string;
  },
): string | undefined {
  /**
   Name of the last matching rule, kept while iteration continues.
   */
  let match: string | undefined;
  for (const rule of directoryRules.filter((candidate,) => candidate.enabled === true).toSorted((a, b,) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0)) {
    /**
     Whether the operator matches.
     */
    const hit = rule.operator.type === 'regexp'
      ? new RegExp(rule.operator.sensitive === true ? rule.operator.data : String(rule.operator.data,).toLowerCase(),)
        .test(rule.operator.sensitive === true ? processPath : processPath.toLowerCase(),)
      : rule.operator.data === processPath;
    if (!hit)
      continue;
    match = rule.name;
    if ((rule.action === 'reject') || (rule.action === 'deny') || (rule.precedence === true))
      return rule.name;
  }
  return match;
}

await describe({
  name: applyToolchainRules.name,
  children: [
    it({
      name: 'creates one mode 0600 file per rule named exactly like its rule',
      fn: async () => {
        await using fixture = await createFixture();
        const report = await applyToolchainRules({ rulesDirectory: fixture.directory, roots: ROOTS, now: NOW, },);
        expect(report,).toEqual({
          created: [
            `${TOOLCHAIN_RULE_NAME_PREFIX}install-roots`,
            `${TOOLCHAIN_RULE_NAME_PREFIX}exposure-links`,
            `${TOOLCHAIN_RULE_NAME_PREFIX}rust-tools`,
          ],
          updated: [],
          unchanged: [],
        },);
        expect((await readdir(fixture.directory,)).toSorted(),).toEqual([
          `${TOOLCHAIN_RULE_NAME_PREFIX}exposure-links.json`,
          `${TOOLCHAIN_RULE_NAME_PREFIX}install-roots.json`,
          `${TOOLCHAIN_RULE_NAME_PREFIX}rust-tools.json`,
        ],);
        const rule = await readRule({ directory: fixture.directory, name: `${TOOLCHAIN_RULE_NAME_PREFIX}rust-tools`, },);
        expect(rule.name,).toBe(`${TOOLCHAIN_RULE_NAME_PREFIX}rust-tools`,);
        expect(rule.created,).toBe('2026-10-31T12:00:00Z',);
        expect(rule.precedence,).toBe(true,);
        expect((await stat(join(fixture.directory, `${TOOLCHAIN_RULE_NAME_PREFIX}rust-tools.json`,),)).mode & 0o777,)
          .toBe(0o600,);
      },
    },),
    it({
      name: 'leaves byte-identical files untouched on a second run',
      fn: async () => {
        await using fixture = await createFixture();
        await applyToolchainRules({ rulesDirectory: fixture.directory, roots: ROOTS, now: NOW, },);
        const file = join(fixture.directory, `${TOOLCHAIN_RULE_NAME_PREFIX}install-roots.json`,);
        const before = await stat(file,);
        const text = await readFile(file, 'utf8',);
        const report = await applyToolchainRules({
          rulesDirectory: fixture.directory,
          roots: ROOTS,
          now: new Date(NOW.getTime() + 60_000,),
        },);
        expect(report.created,).toEqual([],);
        expect(report.updated,).toEqual([],);
        expect(report.unchanged,).toHaveLength(3,);
        expect(await readFile(file, 'utf8',),).toBe(text,);
        expect((await stat(file,)).mtimeMs,).toBe(before.mtimeMs,);
      },
    },),
    it({
      name: 'rewrites a changed rule in place on the same inode and keeps its creation time',
      fn: async () => {
        await using fixture = await createFixture();
        await applyToolchainRules({ rulesDirectory: fixture.directory, roots: ROOTS, now: NOW, },);
        const name = `${TOOLCHAIN_RULE_NAME_PREFIX}install-roots`;
        const file = join(fixture.directory, `${name}.json`,);
        const before = await stat(file,);
        const smaller = { ...ROOTS, dataHomes: ['/d',], };
        const report = await applyToolchainRules({
          rulesDirectory: fixture.directory,
          roots: smaller,
          now: new Date(NOW.getTime() + 60_000,),
        },);
        expect(report.updated,).toEqual([name,],);
        expect((await stat(file,)).ino,).toBe(before.ino,);
        const rule = await readRule({ directory: fixture.directory, name, },);
        expect(rule.created,).toBe('2026-10-31T12:00:00Z',);
        expect(rule.updated,).toBe('2026-10-31T12:01:00Z',);
        expect((rule.operator as { data: string; }).data,).toBe('^(?:/d/monochromatic-dev)/toolchain(?:-global)?/',);
        expect((await readFile(file, 'utf8',)).endsWith('  ',),).toBe(true,);
      },
    },),
    it({
      name: 'refuses to overwrite a same-named rule it did not write',
      fn: async () => {
        await using fixture = await createFixture();
        await writeDaemonRule({
          directory: fixture.directory,
          rule: { name: `${TOOLCHAIN_RULE_NAME_PREFIX}install-roots`, description: 'mine', },
        },);
        let caught: unknown;
        try {
          await applyToolchainRules({ rulesDirectory: fixture.directory, roots: ROOTS, now: NOW, },);
        }
        catch (error) {
          caught = error;
        }
        expect(String(caught,),).toContain('was not written by wg-quicker',);
        expect((await readRule({
          directory: fixture.directory,
          name: `${TOOLCHAIN_RULE_NAME_PREFIX}install-roots`,
        },)).description,).toBe('mine',);
      },
    },),
    it({
      name: 'fails with a diagnostic when the rules directory is absent',
      fn: async () => {
        await using fixture = await createFixture();
        let caught: unknown;
        try {
          await applyToolchainRules({ rulesDirectory: join(fixture.directory, 'missing',), roots: ROOTS, now: NOW, },);
        }
        catch (error) {
          caught = error;
        }
        expect(String(caught,),).toContain('Cannot list OpenSnitch rules directory',);
      },
    },),
  ],
},);

await describe({
  name: removeToolchainRules.name,
  children: [
    it({
      name: 'removes only the files the allowance wrote',
      fn: async () => {
        await using fixture = await createFixture();
        await applyToolchainRules({ rulesDirectory: fixture.directory, roots: ROOTS, now: NOW, },);
        await writeDaemonRule({ directory: fixture.directory, rule: { name: '000-allow-cargo', action: 'allow', }, },);
        await writeDaemonRule({
          directory: fixture.directory,
          rule: { name: `${TOOLCHAIN_RULE_NAME_PREFIX}look-alike`, description: 'user rule', },
        },);
        await writeFile(join(fixture.directory, 'broken.json',), '{',);
        const removed = await removeToolchainRules({ rulesDirectory: fixture.directory, },);
        expect(removed.toSorted(),).toHaveLength(3,);
        expect((await readdir(fixture.directory,)).toSorted(),).toEqual([
          '000-allow-cargo.json',
          `${TOOLCHAIN_RULE_NAME_PREFIX}look-alike.json`,
          'broken.json',
        ],);
      },
    },),
  ],
},);

await describe({
  name: inspectToolchainRules.name,
  children: [
    it({
      name: 'lists covered rejects and rules that sort ahead, ignoring other shapes',
      fn: async () => {
        await using fixture = await createFixture();
        const covered = '/var/home/tester/.local/share/monochromatic-dev/toolchain/node/26.13.0/bin/node';
        await writeDaemonRule({
          directory: fixture.directory,
          rule: {
            name: 'reject-always-simple-covered',
            operator: { operand: 'process.path', data: covered, type: 'simple', list: [], sensitive: false, },
          },
        },);
        await writeDaemonRule({
          directory: fixture.directory,
          rule: {
            name: 'deny-always-simple-covered-cargo',
            action: 'deny',
            operator: { operand: 'process.path', data: '/home/tester/.cargo/bin/rustup', type: 'simple', list: [], sensitive: false, },
          },
        },);
        await writeDaemonRule({
          directory: fixture.directory,
          rule: {
            name: 'reject-always-simple-uncovered',
            operator: {
              operand: 'process.path',
              data: '/var/home/tester/.local/share/mise/installs/zig/0.15.2/zig',
              type: 'simple',
              list: [],
              sensitive: false,
            },
          },
        },);
        await writeDaemonRule({
          directory: fixture.directory,
          rule: {
            name: 'reject-always-simple-disabled',
            enabled: false,
            operator: { operand: 'process.path', data: covered, type: 'simple', list: [], sensitive: false, },
          },
        },);
        await writeDaemonRule({
          directory: fixture.directory,
          rule: {
            name: 'allow-always-simple-covered',
            action: 'allow',
            operator: { operand: 'process.path', data: covered, type: 'simple', list: [], sensitive: false, },
          },
        },);
        await writeDaemonRule({
          directory: fixture.directory,
          rule: {
            name: 'reject-always-simple-port',
            operator: { operand: 'dest.port', data: '53', type: 'simple', list: [], sensitive: false, },
          },
        },);
        await writeDaemonRule({ directory: fixture.directory, rule: { name: '000-allow-cargo', action: 'allow', precedence: true, }, },);
        await writeDaemonRule({ directory: fixture.directory, rule: { name: '000-allow-localhost', action: 'allow', }, },);
        // Sorts after the allowance, so the allowance returns first and this reject is no threat.
        await writeDaemonRule({ directory: fixture.directory, rule: { name: '000-deny-tmp', action: 'reject', }, },);
        // Sorts before the allowance, so this reject would be reached first.
        await writeDaemonRule({ directory: fixture.directory, rule: { name: '000-aaa-reject-first', action: 'reject', }, },);
        await writeFile(join(fixture.directory, 'broken.json',), '{',);
        await applyToolchainRules({ rulesDirectory: fixture.directory, roots: ROOTS, now: NOW, },);
        const inspection = await inspectToolchainRules({ rulesDirectory: fixture.directory, roots: ROOTS, },);
        expect(inspection.present,).toHaveLength(3,);
        expect(inspection.missing,).toEqual([],);
        expect(inspection.superseded.map((rule,) => rule.name).toSorted(),).toEqual([
          'deny-always-simple-covered-cargo',
          'reject-always-simple-covered',
        ],);
        expect(inspection.earlierWinners.toSorted((a, b,) => a.name < b.name ? -1 : 1),).toEqual([
          { name: '000-aaa-reject-first', reason: 'reject', },
          { name: '000-allow-cargo', reason: 'precedence', },
        ],);
        expect(inspection.unreadable,).toHaveLength(1,);
      },
    },),
    it({
      name: 'reports missing rules before apply',
      fn: async () => {
        await using fixture = await createFixture();
        const inspection = await inspectToolchainRules({ rulesDirectory: fixture.directory, roots: ROOTS, },);
        expect(inspection.present,).toEqual([],);
        expect(inspection.missing,).toHaveLength(3,);
      },
    },),
  ],
},);

await describe({
  name: removeSupersededRejects.name,
  children: [
    it({
      name: 'lists in a dry run, then backs up and removes covered rejects only',
      fn: async () => {
        await using fixture = await createFixture();
        const covered = '/home/tester/.rustup/toolchains/nightly/bin/cargo';
        await writeDaemonRule({
          directory: fixture.directory,
          rule: {
            name: 'reject-always-simple-covered',
            operator: { operand: 'process.path', data: covered, type: 'simple', list: [], sensitive: false, },
          },
        },);
        await writeDaemonRule({ directory: fixture.directory, rule: { name: 'reject-always-simple-usr-bin-getent', }, },);
        const dry = await removeSupersededRejects({
          rulesDirectory: fixture.directory,
          roots: ROOTS,
          backupDirectory: fixture.backups,
          dryRun: true,
        },);
        expect(dry.map((rule,) => rule.name),).toEqual(['reject-always-simple-covered',],);
        expect(await readdir(fixture.directory,),).toHaveLength(2,);
        const removed = await removeSupersededRejects({
          rulesDirectory: fixture.directory,
          roots: ROOTS,
          backupDirectory: fixture.backups,
          dryRun: false,
        },);
        expect(removed.map((rule,) => rule.name),).toEqual(['reject-always-simple-covered',],);
        expect(await readdir(fixture.directory,),).toEqual(['reject-always-simple-usr-bin-getent.json',],);
        expect(await readdir(fixture.backups,),).toEqual(['reject-always-simple-covered.json',],);
      },
    },),
    it({
      name: 'keeps the original when a backup of that name already exists',
      fn: async () => {
        await using fixture = await createFixture();
        await writeDaemonRule({
          directory: fixture.directory,
          rule: {
            name: 'reject-always-simple-covered',
            operator: {
              operand: 'process.path',
              data: '/home/tester/.cargo/bin/cargo',
              type: 'simple',
              list: [],
              sensitive: false,
            },
          },
        },);
        await mkdir(fixture.backups,);
        await writeFile(join(fixture.backups, 'reject-always-simple-covered.json',), 'earlier',);
        let caught: unknown;
        try {
          await removeSupersededRejects({
            rulesDirectory: fixture.directory,
            roots: ROOTS,
            backupDirectory: fixture.backups,
            dryRun: false,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(String(caught,),).toContain('EEXIST',);
        expect(await readdir(fixture.directory,),).toEqual(['reject-always-simple-covered.json',],);
        expect(await readFile(join(fixture.backups, 'reject-always-simple-covered.json',), 'utf8',),).toBe('earlier',);
      },
    },),
  ],
},);

await describe({
  name: 'daemon rule selection model',
  children: [
    it({
      name: 'lets the allowance beat an auto-saved reject for a covered path',
      fn: async () => {
        await using fixture = await createFixture();
        const covered = '/var/home/tester/.local/share/monochromatic-dev/toolchain/zig/0.16.0/zig';
        await writeDaemonRule({
          directory: fixture.directory,
          rule: {
            name: 'reject-always-simple-var-home-tester-zig',
            operator: { operand: 'process.path', data: covered, type: 'simple', list: [], sensitive: false, },
          },
        },);
        await applyToolchainRules({ rulesDirectory: fixture.directory, roots: ROOTS, now: NOW, },);
        const rules = await Promise.all((await readdir(fixture.directory,)).map(
          (file,) => readRule({ directory: fixture.directory, name: file.slice(0, -'.json'.length,), }),
        ),) as Record<string, unknown>[];
        expect(firstMatch({ directoryRules: rules, processPath: covered, },),).toBe(
          `${TOOLCHAIN_RULE_NAME_PREFIX}install-roots`,
        );
        expect(firstMatch({
          directoryRules: rules,
          processPath: '/var/home/tester/.local/share/mise/installs/zig/0.15.2/zig',
        },),).toBeUndefined();
      },
    },),
    it({
      name: 'shows precedence and a leading name are each necessary',
      fn: async () => {
        const covered = '/home/tester/.cargo/bin/rustup';
        const reject = {
          name: 'reject-always-simple-rustup',
          action: 'reject',
          enabled: true,
          precedence: false,
          operator: { operand: 'process.path', data: covered, type: 'simple', sensitive: false, },
        };
        const allowOperator = { operand: 'process.path', data: '^/home/tester/', type: 'regexp', sensitive: true, };
        expect(firstMatch({
          directoryRules: [
            reject,
            { name: '000-allow-without-precedence', action: 'allow', enabled: true, precedence: false, operator: allowOperator, },
          ],
          processPath: covered,
        },),).toBe('reject-always-simple-rustup',);
        expect(firstMatch({
          directoryRules: [
            reject,
            { name: 'zzz-allow-with-precedence', action: 'allow', enabled: true, precedence: true, operator: allowOperator, },
          ],
          processPath: covered,
        },),).toBe('reject-always-simple-rustup',);
        expect(firstMatch({
          directoryRules: [
            reject,
            { name: '000-allow-with-precedence', action: 'allow', enabled: true, precedence: true, operator: allowOperator, },
          ],
          processPath: covered,
        },),).toBe('000-allow-with-precedence',);
      },
    },),
  ],
},);

await describe({
  name: parseOpenSnitchRulesArgs.name,
  children: [
    it({
      name: 'parses an action with repeated repository roots and path options',
      fn: async () => {
        expect(parseOpenSnitchRulesArgs({
          argv: [
            'remove-superseded',
            '--dry-run',
            '--repository-root',
            '/a',
            '--repository-root',
            '/b',
            '--rules-directory',
            '/r',
            '--data-home',
            '/d',
            '--cargo-home',
            '/c',
            '--rustup-home',
            '/u',
            '--backup-directory',
            '/k',
          ],
        },),).toEqual({
          action: 'remove-superseded',
          rulesDirectory: '/r',
          repositoryRoots: ['/a', '/b',],
          dataHome: '/d',
          cargoHome: '/c',
          rustupHome: '/u',
          backupDirectory: '/k',
          dryRun: true,
        },);
        expect(parseOpenSnitchRulesArgs({ argv: ['apply',], },),).toEqual({
          action: 'apply',
          rulesDirectory: undefined,
          repositoryRoots: [],
          dataHome: undefined,
          cargoHome: undefined,
          rustupHome: undefined,
          backupDirectory: undefined,
          dryRun: false,
        },);
      },
    },),
    it({
      name: 'rejects an unknown action, extra words, unknown options and relative paths',
      fn: async () => {
        for (
          const argv of [
            [],
            ['purge',],
            ['apply', 'now',],
            ['apply', '--force',],
            ['apply', '--rules-directory', 'relative',],
            ['apply', '--repository-root', 'relative',],
            ['apply', '--rules-directory',],
          ]
        ) {
          let caught: unknown;
          try {
            parseOpenSnitchRulesArgs({ argv, },);
          }
          catch (error) {
            caught = error;
          }
          expect(String(caught,).startsWith('CliUsageError',) || String(caught,).includes('CliUsageError',),).toBe(true,);
        }
      },
    },),
  ],
},);

await describe({
  name: resolveOpenSnitchRulesDirectory.name,
  children: [
    it({
      name: 'reads Rules.Path, defaults when absent, and rejects a relative path',
      fn: async () => {
        await using fixture = await createFixture();
        const original = process.env.WG_QUICKER_OPENSNITCH_DAEMON_CONFIG;
        const config = join(fixture.directory, 'default-config.json',);
        try {
          process.env.WG_QUICKER_OPENSNITCH_DAEMON_CONFIG = config;
          expect(await resolveOpenSnitchRulesDirectory(),).toBe('/etc/opensnitchd/rules',);
          await writeFile(config, JSON.stringify({ Rules: { Path: '/srv/rules/', }, },),);
          expect(await resolveOpenSnitchRulesDirectory(),).toBe('/srv/rules/',);
          await writeFile(config, JSON.stringify({ Firewall: 'nftables', },),);
          expect(await resolveOpenSnitchRulesDirectory(),).toBe('/etc/opensnitchd/rules',);
          await writeFile(config, JSON.stringify({ Rules: { Path: 'relative/rules', }, },),);
          let caught: unknown;
          try {
            await resolveOpenSnitchRulesDirectory();
          }
          catch (error) {
            caught = error;
          }
          expect(String(caught,),).toContain('Rules.Path must be an absolute path',);
        }
        finally {
          if (original === undefined)
            delete process.env.WG_QUICKER_OPENSNITCH_DAEMON_CONFIG;
          else
            process.env.WG_QUICKER_OPENSNITCH_DAEMON_CONFIG = original;
        }
      },
    },),
  ],
},);
