import { createHash, } from 'node:crypto';
import {
  chmod,
  link as createHardLink,
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  readlink,
  rm,
  stat,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  installOpenSnitchEndpointAllowance,
  OPENSNITCH_CONFIG_ENVIRONMENT,
  OPENSNITCH_DAEMON_CONFIG_ENVIRONMENT,
  removeOpenSnitchEndpointAllowance,
} from '../dist/final/node/opensnitch.mjs';

/**
 Current namespace key matching production ownership derivation.
 */
const NETWORK_NAMESPACE_KEY = createHash('sha256',)
  .update(await readlink('/proc/self/ns/net',),)
  .digest('hex',)
  .slice(
  0,
  32,
  );

/**
 Creates scoped managed-rule description for current test namespace.
 
 @param interfaceName - WireGuard interface owner.
 
 @param port - Exact endpoint destination port.
 
 @returns Production-format ownership description.
 
 @example
 ```ts
 managedDescription({ interfaceName: 'wg0', port: 51820 });
 ```
 */
function managedDescription(
  {
    interfaceName,
    port,
  }: {
    readonly interfaceName: string;
    readonly port: number;
  },
): string {
  return `wg-quicker managed endpoint [${interfaceName}] [netns:${NETWORK_NAMESPACE_KEY}] UDP destination port ${String(port,)}`;
}

/**
 Disposable OpenSnitch config fixture and environment restoration.
 */
type OpenSnitchFixture = {
  /**
   System-firewall path selected by environment override.
   */
  readonly configPath: string;

  /**
   Daemon config path selected by environment override.
   */
  readonly daemonConfigPath: string;

  /**
   Fixture-private wg-quicker runtime path.
   */
  readonly runtimePath: string;

  /**
   Removes fixture and restores process environment.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates minimal OpenSnitch nftables configuration.
 
 @param rules - Existing mangle-output rules.
 
 @param enabled - Top-level system-firewall state.
 
 @returns OpenSnitch version 1 document.
 
 @example
 ```ts
 systemFirewall({ rules: [], enabled: true });
 ```
 */
function systemFirewall(
  {
    rules,
    enabled = true,
  }: {
    readonly rules: readonly unknown[];
    readonly enabled?: boolean;
  },
): Record<string, unknown> {
  return {
    Enabled: enabled,
    Version: 1,
    PreserveMe: 'yes',
    SystemRules: [
      {
        Chains: [
          {
            Name: 'mangle_output',
            Table: 'opensnitch',
            Family: 'inet',
            Rules: [...rules,],
          },
        ],
      },
    ],
  };
}

/**
 Sets isolated config and runtime environment for one test.
 
 @param content - Exact initial config text,
 or undefined to leave path absent.
 
 @param firewall - Daemon firewall backend.
 
 @returns Disposable fixture.
 
 @example
 ```ts
 await using fixture = await createFixture({ content: '{}' });
 ```
 */
async function createFixture(
  {
    content,
    firewall = 'nftables',
  }: {
    readonly content?: string;
    readonly firewall?: string;
  },
): Promise<OpenSnitchFixture> {
  const directory = await mkdtemp(join(
    tmpdir(),
    'wg-quicker-opensnitch-',
  ),);
  const configPath = join(
    directory,
    'system-fw.json',
  );
  /**
   Fixture-local daemon config proving selected backend.
   */
  const daemonConfigPath = join(
    directory,
    'default-config.json',
  );
  await writeFile(
    daemonConfigPath,
    JSON.stringify({ Firewall: firewall, },),
  );
  if (content !== undefined) {
    await writeFile(
      configPath,
      content,
      {
        mode: 0o640,
      },
    );
  }
  const originalConfig = process.env[OPENSNITCH_CONFIG_ENVIRONMENT];
  /**
   Original daemon-config override restored on disposal.
   */
  const originalDaemonConfig = process.env[OPENSNITCH_DAEMON_CONFIG_ENVIRONMENT];
  const originalRuntime = process.env.WG_QUICKER_RUNTIME_DIRECTORY;
  /**
   Original executable search path restored on disposal.
   */
  const originalPath = process.env.PATH;
  /**
   Fixture-private runtime directory.
   */
  const runtimePath = join(
    directory,
    'run',
  );
  /**
   Fixture command directory proving no stale nftables table exists.
   */
  const commandDirectory = join(
    directory,
    'bin',
  );
  await mkdir(commandDirectory,);
  /**
   Minimal nft fixture executable used only after daemon absence is confirmed.
   */
  const nftPath = join(
    commandDirectory,
    'nft',
  );
  await writeFile(
    nftPath,
    `#!/usr/bin/env node\nif (process.argv.slice(2).join(' ') !== '--numeric list tables') throw new Error('unexpected nft fixture invocation');\n`,
  );
  await chmod(
    nftPath,
    0o700,
  );
  process.env[OPENSNITCH_CONFIG_ENVIRONMENT] = configPath;
  process.env[OPENSNITCH_DAEMON_CONFIG_ENVIRONMENT] = daemonConfigPath;
  process.env.WG_QUICKER_RUNTIME_DIRECTORY = runtimePath;
  process.env.PATH = `${commandDirectory}:${originalPath ?? ''}`;
  return {
    configPath,
    daemonConfigPath,
    runtimePath,
    async [Symbol.asyncDispose](): Promise<void> {
      if (originalConfig === undefined)
        delete process.env.WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG;
      else
        process.env[OPENSNITCH_CONFIG_ENVIRONMENT] = originalConfig;
      if (originalDaemonConfig === undefined)
        delete process.env.WG_QUICKER_OPENSNITCH_DAEMON_CONFIG;
      else
        process.env[OPENSNITCH_DAEMON_CONFIG_ENVIRONMENT] = originalDaemonConfig;
      if (originalRuntime === undefined)
        delete process.env.WG_QUICKER_RUNTIME_DIRECTORY;
      else
        process.env.WG_QUICKER_RUNTIME_DIRECTORY = originalRuntime;
      if (originalPath === undefined)
        delete process.env.PATH;
      else
        process.env.PATH = originalPath;
      await rm(
        directory,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 Reads target chain rules from persisted fixture.
 
 @param path - System-firewall config path.
 
 @returns Parsed target rules.
 
 @example
 ```ts
 await readRules({ path });
 ```
 */
async function readRules(
  { path, }: { readonly path: string; },
): Promise<readonly Record<string, unknown>[]> {
  const document = JSON.parse(await readFile(
    path,
    'utf8',
  ),) as {
    readonly SystemRules: readonly {
      readonly Chains: readonly {
        readonly Rules: readonly Record<string, unknown>[];
      }[];
    }[];
  };
  return document.SystemRules[0]?.Chains[0]?.Rules ?? [];
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: installOpenSnitchEndpointAllowance.name,
      children: [
        it({
          name: 'does nothing when OpenSnitch config is absent',
          fn: async ctx => {
            await using fixture = await createFixture({});
            const warning = ctx.sinon.spy(console, 'warn',);
            await installOpenSnitchEndpointAllowance({
              interfaceName: 'wg0',
              endpointPorts: [51_820,],
            },);
            expect(warning,).not.toHaveBeenCalled();
            let missing: unknown;
            try {
              await stat(fixture.configPath,);
            }
            catch (error) {
              missing = error;
            }
            expect(missing,).toBeInstanceOf(Error,);
          },
        },),

        it({
          name: 'leaves valid config unchanged when peers have no endpoints',
          fn: async ctx => {
            const content = `${JSON.stringify(systemFirewall({ rules: [], },), null, 2,)}\n`;
            await using fixture = await createFixture({ content, });
            const warning = ctx.sinon.spy(console, 'warn',);
            await installOpenSnitchEndpointAllowance({
              interfaceName: 'wg0',
              endpointPorts: [],
            },);
            expect(await readFile(
              fixture.configPath,
              'utf8',
            ),).toBe(content,);
            expect(warning,).not.toHaveBeenCalled();
          },
        },),

        it({
          name: 'uses daemon FwOptions config path without explicit system override',
          fn: async () => {
            await using fixture = await createFixture({
              content: `${JSON.stringify(systemFirewall({ rules: [], },), null, 2,)}\n`,
            },);
            await writeFile(
              fixture.daemonConfigPath,
              JSON.stringify({
                Firewall: 'nftables',
                FwOptions: { ConfigPath: fixture.configPath, },
              },),
            );
            delete process.env.WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG;
            await installOpenSnitchEndpointAllowance({
              interfaceName: 'wg0',
              endpointPorts: [51_820,],
            },);
            expect(await readRules({ path: fixture.configPath, },),).toHaveLength(1,);
          },
        },),

        it({
          name: 'rejects relative explicit system-firewall path',
          fn: async () => {
            await using fixture = await createFixture({
              content: `${JSON.stringify(systemFirewall({ rules: [], },), null, 2,)}\n`,
            },);
            process.env.WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG = 'relative-system-fw.json';
            let caught: unknown;
            try {
              await installOpenSnitchEndpointAllowance({
                interfaceName: 'wg0',
                endpointPorts: [51_820,],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(String(caught,),).toContain('must be an absolute path',);
          },
        },),

        it({
          name: 'rejects relative daemon config override',
          fn: async () => {
            await using fixture = await createFixture({
              content: `${JSON.stringify(systemFirewall({ rules: [], },), null, 2,)}\n`,
            },);
            process.env.WG_QUICKER_OPENSNITCH_DAEMON_CONFIG = 'relative-default-config.json';
            let caught: unknown;
            try {
              await installOpenSnitchEndpointAllowance({
                interfaceName: 'wg0',
                endpointPorts: [51_820,],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(String(caught,),).toContain('must be an absolute path',);
          },
        },),

        it({
          name: 'rejects relative runtime directory override',
          fn: async () => {
            await using fixture = await createFixture({
              content: `${JSON.stringify(systemFirewall({ rules: [], },), null, 2,)}\n`,
            },);
            process.env.WG_QUICKER_RUNTIME_DIRECTORY = 'relative-runtime';
            let caught: unknown;
            try {
              await installOpenSnitchEndpointAllowance({
                interfaceName: 'wg0',
                endpointPorts: [51_820,],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(String(caught,),).toContain('WG_QUICKER_RUNTIME_DIRECTORY must be an absolute path',);
          },
        },),

        it({
          name: 'rejects relative daemon FwOptions config path',
          fn: async () => {
            await using fixture = await createFixture({
              content: `${JSON.stringify(systemFirewall({ rules: [], },), null, 2,)}\n`,
            },);
            await writeFile(
              fixture.daemonConfigPath,
              JSON.stringify({
                Firewall: 'nftables',
                FwOptions: { ConfigPath: 'relative-system-fw.json', },
              },),
            );
            delete process.env.WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG;
            let caught: unknown;
            try {
              await installOpenSnitchEndpointAllowance({
                interfaceName: 'wg0',
                endpointPorts: [51_820,],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(String(caught,),).toContain('FwOptions.ConfigPath must be an absolute path',);
          },
        },),

        it({
          name: 'rejects symbolic-link system-firewall path',
          fn: async () => {
            await using fixture = await createFixture({
              content: `${JSON.stringify(systemFirewall({ rules: [], },), null, 2,)}\n`,
            },);
            await rm(fixture.configPath,);
            await symlink(
              fixture.daemonConfigPath,
              fixture.configPath,
            );
            let caught: unknown;
            try {
              await installOpenSnitchEndpointAllowance({
                interfaceName: 'wg0',
                endpointPorts: [51_820,],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(String(caught,),).toContain('Cannot read OpenSnitch system-firewall config',);
          },
        },),

        it({
          name: 'rejects multiply-linked system-firewall file',
          fn: async () => {
            await using fixture = await createFixture({
              content: `${JSON.stringify(systemFirewall({ rules: [], },), null, 2,)}\n`,
            },);
            await createHardLink(
              fixture.configPath,
              `${fixture.configPath}.link`,
            );
            let caught: unknown;
            try {
              await installOpenSnitchEndpointAllowance({
                interfaceName: 'wg0',
                endpointPorts: [51_820,],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(String(caught,),).toContain('Cannot read OpenSnitch system-firewall config',);
          },
        },),

        it({
          name: 'writes visible managed rules preserves mode and warns about policy widening',
          fn: async ctx => {
            await using fixture = await createFixture({
              content: `${JSON.stringify(systemFirewall({ rules: [], },), null, 2,)}\n`,
            },);
            await chmod(
              fixture.configPath,
              0o640,
            );
            const warning = ctx.sinon.spy(console, 'warn',);
            await installOpenSnitchEndpointAllowance({
              interfaceName: 'wg0',
              endpointPorts: [51_820, 2_049, 51_820,],
            },);
            const rules = await readRules({ path: fixture.configPath, },);
            expect(rules,).toHaveLength(2,);
            expect(rules[0]?.Position,).toBe('0',);
            expect(rules[0]?.Description,).toBe(
              managedDescription({
                interfaceName: 'wg0',
                port: 2_049,
              },),
            );
            expect((await stat(fixture.configPath,)).mode & 0o777,).toBe(0o640,);
            expect(warning,).toHaveBeenCalledWith(
              expect.stringContaining("accepts any process's outbound UDP",),
            );
          },
        },),

        it({
          name: 'rejects unsupported OpenSnitch iptables backend without changing file',
          fn: async () => {
            const content = `${JSON.stringify(systemFirewall({ rules: [], },), null, 2,)}\n`;
            await using fixture = await createFixture({
              content,
              firewall: 'iptables',
            },);
            let caught: unknown;
            try {
              await installOpenSnitchEndpointAllowance({
                interfaceName: 'wg0',
                endpointPorts: [51_820,],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(String(caught,),).toContain('requires Firewall = nftables',);
            expect(await readFile(
              fixture.configPath,
              'utf8',
            ),).toBe(content,);
          },
        },),

        it({
          name: 'rejects disabled OpenSnitch system firewall without changing file',
          fn: async () => {
            const content = `${JSON.stringify(systemFirewall({
              rules: [],
              enabled: false,
            },), null, 2,)}\n`;
            await using fixture = await createFixture({ content, });
            let caught: unknown;
            try {
              await installOpenSnitchEndpointAllowance({
                interfaceName: 'wg0',
                endpointPorts: [51_820,],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(String(caught,),).toContain('disabled',);
            expect(await readFile(
              fixture.configPath,
              'utf8',
            ),).toBe(content,);
          },
        },),
      ],
    },),

    describe({
      name: removeOpenSnitchEndpointAllowance.name,
      children: [
        it({
          name: 'removes only requested interface rules',
          fn: async () => {
            const rules = [
              { Description: managedDescription({ interfaceName: 'wg0', port: 2_049, }), },
              { Description: managedDescription({ interfaceName: 'wg1', port: 51_820, }), },
              { Description: 'user rule', },
            ];
            await using fixture = await createFixture({
              content: `${JSON.stringify(systemFirewall({ rules, },), null, 2,)}\n`,
            },);
            /**
             Watched inode size before shorter JSON replacement.
             */
            const sizeBefore = (await stat(fixture.configPath,)).size;
            await removeOpenSnitchEndpointAllowance({ interfaceName: 'wg0', },);
            expect(await readRules({ path: fixture.configPath, },),).toEqual([
              rules[1],
              rules[2],
            ],);
            expect((await stat(fixture.configPath,)).size,).toBe(sizeBefore,);
          },
        },),

        it({
          name: 'removes from persisted install path after daemon path changes',
          fn: async () => {
            await using fixture = await createFixture({
              content: `${JSON.stringify(systemFirewall({ rules: [], },), null, 2,)}\n`,
            },);
            await installOpenSnitchEndpointAllowance({
              interfaceName: 'wg0',
              endpointPorts: [51_820,],
            },);
            const replacementPath = `${fixture.configPath}.replacement`;
            await writeFile(
              replacementPath,
              `${JSON.stringify(systemFirewall({ rules: [], },), null, 2,)}\n`,
            );
            process.env.WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG = replacementPath;
            await removeOpenSnitchEndpointAllowance({ interfaceName: 'wg0', },);
            expect(await readRules({ path: fixture.configPath, },),).toEqual([],);
            expect(await readRules({ path: replacementPath, },),).toEqual([],);
            expect((await readdir(fixture.runtimePath,))
              .filter(function stateFiles(name,): boolean {
                return name.startsWith('opensnitch-interface-',) && name.endsWith('.json',);
              },),).toEqual([],);
          },
        },),

        it({
          name: 'uses explicit removal path despite malformed daemon config',
          fn: async () => {
            const rules = [
              { Description: managedDescription({ interfaceName: 'wg0', port: 2_049, }), },
            ];
            await using fixture = await createFixture({
              content: `${JSON.stringify(systemFirewall({ rules, },), null, 2,)}\n`,
            },);
            await writeFile(
              fixture.daemonConfigPath,
              '{',
            );
            await removeOpenSnitchEndpointAllowance({ interfaceName: 'wg0', },);
            expect(await readRules({ path: fixture.configPath, },),).toEqual([],);
          },
        },),

        it({
          name: 'logs malformed external config and continues teardown',
          fn: async ctx => {
            await using fixture = await createFixture({ content: '{', });
            const diagnostic = ctx.sinon.spy(console, 'error',);
            await removeOpenSnitchEndpointAllowance({ interfaceName: 'wg0', },);
            expect(diagnostic,).toHaveBeenCalledWith(
              expect.stringContaining('remove wg-quicker-managed rules manually',),
            );
            expect(await readFile(
              fixture.configPath,
              'utf8',
            ),).toBe('{',);
          },
        },),
      ],
    },),
  ],
},);
