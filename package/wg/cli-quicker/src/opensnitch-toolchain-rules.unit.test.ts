import {
  mkdir,
  readdir,
  readFile,
  stat,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { resolveOpenSnitchRulesDirectory, } from '../dist/final/node/opensnitch-daemon-config.mjs';
import {
  assertHomeBelongsToCaller,
  parseOpenSnitchRulesArgs,
} from '../dist/final/node/opensnitch-rules-command.mjs';
import { TOOLCHAIN_RULE_NAME_PREFIX, } from '../dist/final/node/opensnitch-toolchain-rule-patterns.mjs';
import {
  applyToolchainRules,
  inspectToolchainRules,
  removeSupersededRejects,
  removeToolchainRules,
} from '../dist/final/node/opensnitch-toolchain-rules.mjs';
import {
  createRulesFixture,
  createScratch,
  exactPathOperator,
  firstMatch,
  type FixtureRule,
  NO_RULE_MATCHES,
  NOW,
  readRule,
  ROOTS,
  writeDaemonRule,
} from './fixture.toolchain-rules.ts';

/**
 Name of the install-roots allowance rule.
 */
const INSTALL_ROOTS = `${TOOLCHAIN_RULE_NAME_PREFIX}install-roots`;

/**
 Name of the exposure-links allowance rule.
 */
const EXPOSURE_LINKS = `${TOOLCHAIN_RULE_NAME_PREFIX}exposure-links`;

/**
 Name of the Rust-tools allowance rule.
 */
const RUST_TOOLS = `${TOOLCHAIN_RULE_NAME_PREFIX}rust-tools`;

/**
 Path of a tool installed below the toolchain root,
 which the allowance covers.
 */
const COVERED_NODE = '/var/home/tester/.local/share/monochromatic-dev/toolchain/node/26.13.0/bin/node';

/**
 Builds an allow rule matching every path under the fictional home.

 @param name - Rule name.

 @param precedence - Whether the rule is a priority rule.

 @returns Rule object.

 @example
 ```ts
 allowRule({ name: '000-allow-x', precedence: true });
 ```
 */
function allowRule({ name, precedence, }: { readonly name: string; readonly precedence: boolean; },): FixtureRule {
  return {
    name,
    action: 'allow',
    enabled: true,
    precedence,
    operator: { operand: 'process.path', data: '^/home/tester/', type: 'regexp', sensitive: true, },
  };
}

/**
 Parses a command line expected to be invalid and returns the failure text.

 @param argv - Arguments after `opensnitch-rules`.

 @returns The thrown value as text,
 or an empty string when parsing succeeded.

 @example
 ```ts
 usageFailure({ argv: ['purge',] });
 ```
 */
function usageFailure({ argv, }: { readonly argv: readonly string[]; },): string {
  try {
    parseOpenSnitchRulesArgs({ argv, },);
  }
  catch (error) {
    return String(error,);
  }
  return '';
}

/**
 Catches the failure of an async call so a test can inspect its text.

 @param call - Call expected to fail.

 @returns The thrown value as text.

 @example
 ```ts
 await failureText({ call: async () => { throw new Error('x'); } });
 ```
 */
async function failureText({ call, }: { readonly call: () => Promise<unknown>; },): Promise<string> {
  try {
    await call();
  }
  catch (error) {
    return String(error,);
  }
  return '';
}

await describe({
  name: applyToolchainRules.name,
  children: [
    it({
      name: 'creates one mode 0600 file per rule named exactly like its rule',
      fn: async () => {
        await using fixture = await createRulesFixture();
        expect(await applyToolchainRules({ rulesDirectory: fixture.directory, roots: ROOTS, now: NOW, },),).toEqual({
          created: [INSTALL_ROOTS, EXPOSURE_LINKS, RUST_TOOLS,],
          updated: [],
          unchanged: [],
        },);
        expect((await readdir(fixture.directory,)).toSorted(),).toEqual([
          `${EXPOSURE_LINKS}.json`,
          `${INSTALL_ROOTS}.json`,
          `${RUST_TOOLS}.json`,
        ],);
        /**
         Rule file read back from disk.
         */
        const rule = await readRule({ directory: fixture.directory, name: RUST_TOOLS, },);
        expect(rule.name,).toBe(RUST_TOOLS,);
        expect(rule.created,).toBe('2026-10-31T12:00:00Z',);
        expect(rule.precedence,).toBe(true,);
        expect((await stat(join(fixture.directory, `${RUST_TOOLS}.json`,),)).mode & 0o777,).toBe(0o600,);
      },
    },),
    it({
      name: 'leaves byte-identical files untouched on a second run',
      fn: async () => {
        await using fixture = await createRulesFixture();
        await applyToolchainRules({ rulesDirectory: fixture.directory, roots: ROOTS, now: NOW, },);
        /**
         File whose bytes and modification time must not change.
         */
        const file = join(fixture.directory, `${INSTALL_ROOTS}.json`,);
        /**
         Metadata before the second run.
         */
        const before = await stat(file,);
        /**
         Text before the second run.
         */
        const text = await readFile(file, 'utf8',);
        /**
         Report of the second run.
         */
        const report = await applyToolchainRules({
          rulesDirectory: fixture.directory,
          roots: ROOTS,
          now: new Date('2026-10-31T12:01:00Z',),
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
        await using fixture = await createRulesFixture();
        await applyToolchainRules({ rulesDirectory: fixture.directory, roots: ROOTS, now: NOW, },);
        /**
         Rule file rewritten by the second run.
         */
        const file = join(fixture.directory, `${INSTALL_ROOTS}.json`,);
        /**
         Metadata before the rewrite.
         */
        const before = await stat(file,);
        /**
         Report of the rewrite with a shorter data-home list.
         */
        const report = await applyToolchainRules({
          rulesDirectory: fixture.directory,
          roots: { ...ROOTS, dataHomes: ['/d',], },
          now: new Date('2026-10-31T12:01:00Z',),
        },);
        expect(report.updated,).toEqual([INSTALL_ROOTS,],);
        expect((await stat(file,)).ino,).toBe(before.ino,);
        /**
         Rule read back after the rewrite.
         */
        const rule = await readRule({ directory: fixture.directory, name: INSTALL_ROOTS, },);
        expect(rule.created,).toBe('2026-10-31T12:00:00Z',);
        expect(rule.updated,).toBe('2026-10-31T12:01:00Z',);
        expect((rule.operator as FixtureRule).data,).toBe('^(?:/d/monochromatic-dev)/toolchain(?:-global)?/',);
        expect((await readFile(file, 'utf8',)).endsWith('  ',),).toBe(true,);
      },
    },),
    it({
      name: 'refuses to overwrite a same-named rule it did not write',
      fn: async () => {
        await using fixture = await createRulesFixture();
        await writeDaemonRule({ directory: fixture.directory, rule: { name: INSTALL_ROOTS, description: 'mine', }, },);
        expect(
          await failureText({
            call: () => applyToolchainRules({ rulesDirectory: fixture.directory, roots: ROOTS, now: NOW, },),
          },),
        ).toContain('was not written by wg-quicker',);
        expect((await readRule({ directory: fixture.directory, name: INSTALL_ROOTS, },)).description,).toBe('mine',);
      },
    },),
    it({
      name: 'fails with a diagnostic when the rules directory is absent',
      fn: async () => {
        await using fixture = await createRulesFixture();
        expect(
          await failureText({
            call: () =>
              applyToolchainRules({ rulesDirectory: join(fixture.directory, 'missing',), roots: ROOTS, now: NOW, },),
          },),
        ).toContain('Cannot list OpenSnitch rules directory',);
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
        await using fixture = await createRulesFixture();
        await applyToolchainRules({ rulesDirectory: fixture.directory, roots: ROOTS, now: NOW, },);
        await writeDaemonRule({ directory: fixture.directory, rule: { name: '000-allow-cargo', action: 'allow', }, },);
        await writeDaemonRule({
          directory: fixture.directory,
          rule: { name: `${TOOLCHAIN_RULE_NAME_PREFIX}look-alike`, description: 'user rule', },
        },);
        await writeFile(join(fixture.directory, 'broken.json',), '{',);
        expect((await removeToolchainRules({ rulesDirectory: fixture.directory, },)).toSorted(),).toEqual([
          EXPOSURE_LINKS,
          INSTALL_ROOTS,
          RUST_TOOLS,
        ],);
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
        await using fixture = await createRulesFixture();
        /**
         Auto-saved reject or deny rules and neighbors, each with the fields that decide its role.
         */
        const fixtures: readonly (FixtureRule & { readonly name: string })[] = [
          { name: 'reject-always-simple-covered', operator: exactPathOperator({ data: COVERED_NODE, },), },
          {
            name: 'deny-always-simple-covered-cargo',
            action: 'deny',
            operator: exactPathOperator({ data: '/home/tester/.cargo/bin/rustup', },),
          },
          {
            name: 'reject-always-simple-uncovered',
            operator: exactPathOperator({ data: '/var/home/tester/.local/share/mise/installs/zig/0.15.2/zig', },),
          },
          {
            name: 'reject-always-simple-disabled',
            enabled: false,
            operator: exactPathOperator({ data: COVERED_NODE, },),
          },
          { name: 'allow-always-simple-covered', action: 'allow', operator: exactPathOperator({ data: COVERED_NODE, },), },
          {
            name: 'reject-always-simple-port',
            operator: { operand: 'dest.port', data: '53', type: 'simple', list: [], sensitive: false, },
          },
          { name: '000-allow-cargo', action: 'allow', precedence: true, },
          { name: '000-allow-localhost', action: 'allow', },
          // Sorts after the allowance, so the allowance returns first and this reject is no threat.
          { name: '000-deny-tmp', action: 'reject', },
          // Sorts before the allowance, so this reject would be reached first.
          { name: '000-aaa-reject-first', action: 'reject', },
        ];
        await Promise.all(fixtures.map((rule,) => writeDaemonRule({ directory: fixture.directory, rule, },)),);
        await writeFile(join(fixture.directory, 'broken.json',), '{',);
        await applyToolchainRules({ rulesDirectory: fixture.directory, roots: ROOTS, now: NOW, },);
        /**
         Inspection of the populated directory.
         */
        const inspection = await inspectToolchainRules({ rulesDirectory: fixture.directory, roots: ROOTS, },);
        expect(inspection.present,).toHaveLength(3,);
        expect(inspection.missing,).toEqual([],);
        expect(inspection.superseded.map((rule,) => rule.name).toSorted(),).toEqual([
          'deny-always-simple-covered-cargo',
          'reject-always-simple-covered',
        ],);
        expect(inspection.earlierWinners.toSorted((left, right,) => left.name < right.name ? -1 : 1),).toEqual([
          { name: '000-aaa-reject-first', reason: 'reject', },
          { name: '000-allow-cargo', reason: 'precedence', },
        ],);
        expect(inspection.unreadable,).toHaveLength(1,);
      },
    },),
    it({
      name: 'reports missing rules before apply',
      fn: async () => {
        await using fixture = await createRulesFixture();
        /**
         Inspection of an empty directory.
         */
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
        await using fixture = await createRulesFixture();
        await writeDaemonRule({
          directory: fixture.directory,
          rule: {
            name: 'reject-always-simple-covered',
            operator: exactPathOperator({ data: '/home/tester/.rustup/toolchains/nightly/bin/cargo', },),
          },
        },);
        await writeDaemonRule({ directory: fixture.directory, rule: { name: 'reject-always-simple-usr-bin-getent', }, },);
        /**
         Arguments shared by the dry and real runs.
         */
        const shared = { rulesDirectory: fixture.directory, roots: ROOTS, backupDirectory: fixture.backups, };
        expect((await removeSupersededRejects({ ...shared, dryRun: true, },)).map((rule,) => rule.name),).toEqual([
          'reject-always-simple-covered',
        ],);
        expect(await readdir(fixture.directory,),).toHaveLength(2,);
        expect((await removeSupersededRejects({ ...shared, dryRun: false, },)).map((rule,) => rule.name),).toEqual([
          'reject-always-simple-covered',
        ],);
        expect(await readdir(fixture.directory,),).toEqual(['reject-always-simple-usr-bin-getent.json',],);
        expect(await readdir(fixture.backups,),).toEqual(['reject-always-simple-covered.json',],);
      },
    },),
    it({
      name: 'keeps the original when a backup of that name already exists',
      fn: async () => {
        await using fixture = await createRulesFixture();
        await writeDaemonRule({
          directory: fixture.directory,
          rule: {
            name: 'reject-always-simple-covered',
            operator: exactPathOperator({ data: '/home/tester/.cargo/bin/cargo', },),
          },
        },);
        await mkdir(fixture.backups,);
        await writeFile(join(fixture.backups, 'reject-always-simple-covered.json',), 'earlier',);
        expect(
          await failureText({
            call: () =>
              removeSupersededRejects({
                rulesDirectory: fixture.directory,
                roots: ROOTS,
                backupDirectory: fixture.backups,
                dryRun: false,
              },),
          },),
        ).toContain('EEXIST',);
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
        await using fixture = await createRulesFixture();
        await writeDaemonRule({
          directory: fixture.directory,
          rule: { name: 'reject-always-simple-var-home-tester-node', operator: exactPathOperator({ data: COVERED_NODE, },), },
        },);
        await applyToolchainRules({ rulesDirectory: fixture.directory, roots: ROOTS, now: NOW, },);
        /**
         Every rule file parsed back from disk.
         */
        const rules = await Promise.all((await readdir(fixture.directory,)).map(
          (file,) => readRule({ directory: fixture.directory, name: file.slice(0, -'.json'.length,), },),
        ),);
        expect(firstMatch({ directoryRules: rules, processPath: COVERED_NODE, },),).toBe(INSTALL_ROOTS,);
        expect(
          firstMatch({
            directoryRules: rules,
            processPath: '/var/home/tester/.local/share/mise/installs/zig/0.15.2/zig',
          },),
        ).toBe(NO_RULE_MATCHES,);
      },
    },),
    it({
      name: 'shows precedence and a leading name are each necessary',
      fn: async () => {
        /**
         Path both rules compare.
         */
        const covered = '/home/tester/.cargo/bin/rustup';
        /**
         Auto-saved reject for the path.
         */
        const reject = {
          name: 'reject-always-simple-rustup',
          action: 'reject',
          enabled: true,
          precedence: false,
          operator: exactPathOperator({ data: covered, },),
        };
        expect(
          firstMatch({
            directoryRules: [reject, allowRule({ name: '000-allow-without-precedence', precedence: false, },),],
            processPath: covered,
          },),
        ).toBe('reject-always-simple-rustup',);
        expect(
          firstMatch({
            directoryRules: [reject, allowRule({ name: 'zzz-allow-with-precedence', precedence: true, },),],
            processPath: covered,
          },),
        ).toBe('reject-always-simple-rustup',);
        expect(
          firstMatch({
            directoryRules: [reject, allowRule({ name: '000-allow-with-precedence', precedence: true, },),],
            processPath: covered,
          },),
        ).toBe('000-allow-with-precedence',);
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
        expect(
          parseOpenSnitchRulesArgs({
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
          },),
        ).toEqual({
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
          repositoryRoots: [],
          dryRun: false,
        },);
      },
    },),
    it({
      name: 'rejects an unknown action, extra words, unknown options and relative paths',
      fn: async () => {
        expect(
          [
            [],
            ['purge',],
            ['apply', 'now',],
            ['apply', '--force',],
            ['apply', '--rules-directory', 'relative',],
            ['apply', '--repository-root', 'relative',],
            ['apply', '--rules-directory',],
          ].filter((argv,) => !usageFailure({ argv, },).includes('CliUsageError',)),
        ).toEqual([],);
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
        await using scratch = await createScratch({ prefix: 'wg-quicker-daemon-config-', },);
        /**
         Daemon config the override variable points at.
         */
        const config = join(scratch.path, 'default-config.json',);
        /**
         Override value before the test,
         restored on disposal.
         */
        const original = process.env.WG_QUICKER_OPENSNITCH_DAEMON_CONFIG;
        await using restore = {
          [Symbol.asyncDispose]: async (): Promise<void> => {
            if (original === undefined)
              delete process.env.WG_QUICKER_OPENSNITCH_DAEMON_CONFIG;
            else
              process.env.WG_QUICKER_OPENSNITCH_DAEMON_CONFIG = original;
          },
        };
        process.env.WG_QUICKER_OPENSNITCH_DAEMON_CONFIG = config;
        expect(await resolveOpenSnitchRulesDirectory(),).toBe('/etc/opensnitchd/rules',);
        await writeFile(config, JSON.stringify({ Rules: { Path: '/srv/rules/', }, },),);
        expect(await resolveOpenSnitchRulesDirectory(),).toBe('/srv/rules/',);
        await writeFile(config, JSON.stringify({ Firewall: 'nftables', },),);
        expect(await resolveOpenSnitchRulesDirectory(),).toBe('/etc/opensnitchd/rules',);
        await writeFile(config, JSON.stringify({ Rules: { Path: 'relative/rules', }, },),);
        expect(await failureText({ call: () => resolveOpenSnitchRulesDirectory(), },),).toContain(
          'Rules.Path must be an absolute path',
        );
        expect(restore,).toBeDefined();
      },
    },),
  ],
},);

await describe({
  name: assertHomeBelongsToCaller.name,
  children: [
    it({
      name: 'refuses root running for a sudo caller whose home this is not, unless every home is explicit',
      fn: async () => {
        /**
         Failure text of `sudo` started by uid 1000 with root's home.
         */
        const refused = (() => {
          try {
            assertHomeBelongsToCaller({
              home: '/root',
              homeOwnerUid: 0,
              effectiveUid: 0,
              sudoUid: '1000',
              explicitHomes: false,
            },);
          }
          catch (error) {
            return String(error,);
          }
          return '';
        })();
        expect(refused,).toContain('HOME is /root, owned by uid 0, but sudo was started by uid 1000',);
        expect(refused,).toContain('as that user',);
        expect(() =>
          assertHomeBelongsToCaller({
            home: '/root',
            homeOwnerUid: 0,
            effectiveUid: 0,
            sudoUid: '1000',
            explicitHomes: true,
          },)
        ).not.toThrow();
      },
    },),
    it({
      name: 'accepts the caller home, a non-root run and a root run without sudo',
      fn: async () => {
        expect(() =>
          assertHomeBelongsToCaller({
            home: '/home/me',
            homeOwnerUid: 1_000,
            effectiveUid: 0,
            sudoUid: '1000',
            explicitHomes: false,
          },)
        ).not.toThrow();
        expect(() =>
          assertHomeBelongsToCaller({
            home: '/home/me',
            homeOwnerUid: 1_000,
            effectiveUid: 1_000,
            sudoUid: '2000',
            explicitHomes: false,
          },)
        ).not.toThrow();
        expect(() =>
          assertHomeBelongsToCaller({
            home: '/root',
            homeOwnerUid: 0,
            effectiveUid: 0,
            explicitHomes: false,
          },)
        ).not.toThrow();
      },
    },),
  ],
},);
