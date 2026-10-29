import assert from 'node:assert/strict';
import {
  chmod,
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  runSudo,
  runSudoAllowingFailure,
  type FixtureCommandResult,
} from './tunnel-bypass-command-fixture.ts';

/**
 Built CLI bundle exercised at user boundary.
 */
const CLI_BUNDLE_PATH = new URL(
  '../dist/final/node/index.mjs',
  import.meta.url,
).pathname;

/**
 UDP port isolated inside server namespace.
 */
const SERVER_PORT = 51_888;

/**
 Disposable endpoint-routing integration resources.
 */
type RouteFixture = {
  /**
   Client network namespace.
   */
  readonly clientNamespace: string;

  /**
   Client physical interface.
   */
  readonly clientPhysical: string;

  /**
   Root-private config and runtime directory.
   */
  readonly directory: string;

  /**
   Server network namespace.
   */
  readonly serverNamespace: string;

  /**
   Server physical interface.
   */
  readonly serverPhysical: string;

  /**
   Removes all fixture resources.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Runs command inside selected fixture namespace.
 
 @param namespace - Network namespace name.
 
 @param command - Executable and arguments.
 
 @returns Captured standard output.
 
 @example
 ```ts
 await runInNamespace({ namespace: 'client', command: ['ip', 'route'] });
 ```
 */
async function runInNamespace(
  {
    namespace,
    command,
  }: {
    readonly namespace: string;
    readonly command: readonly string[];
  },
): Promise<string> {
  return await runSudo({
    args: [
      'ip',
      'netns',
      'exec',
      namespace,
      ...command,
    ],
  },);
}

/**
 Runs built CLI inside client namespace while preserving failure result.
 
 @param fixture - Disposable network fixture.
 
 @param operation - Tunnel lifecycle operation.
 
 @param configPath - Root-readable config path.
 
 @returns Command exit and captured output.
 
 @example
 ```ts
 await runFixtureCli({ fixture, operation: 'up', configPath: '/tmp/wg.conf' });
 ```
 */
async function runFixtureCli(
  {
    fixture,
    operation,
    configPath,
  }: {
    readonly fixture: RouteFixture;
    readonly operation: 'down' | 'up';
    readonly configPath: string;
  },
): Promise<FixtureCommandResult> {
  return await runSudoAllowingFailure({
    args: [
      'ip',
      'netns',
      'exec',
      fixture.clientNamespace,
      'env',
      `WG_QUICKER_RUNTIME_DIRECTORY=${fixture.directory}`,
      process.execPath,
      CLI_BUNDLE_PATH,
      operation,
      configPath,
    ],
  },);
}

/**
 Builds two isolated physical peers and returns cleanup ownership.
 
 @returns Configured namespace fixture.
 
 @example
 ```ts
 await using fixture = await createRouteFixture();
 ```
 */
async function createRouteFixture(): Promise<RouteFixture> {
  /**
   Short suffix keeping Linux names under interface limit.
   */
  const suffix = String(process.pid,).slice(-5,);
  /**
   Client namespace name.
   */
  const clientNamespace = `wgrc${suffix}`;
  /**
   Server namespace name.
   */
  const serverNamespace = `wgrs${suffix}`;
  /**
   Client veth name.
   */
  const clientPhysical = `wgc${suffix}`;
  /**
   Server veth name.
   */
  const serverPhysical = `wgs${suffix}`;
  /**
   Private fixture directory.
   */
  const directory = await mkdtemp(join(tmpdir(), 'wgq-route-integration-',),);
  await chmod(directory, 0o700,);
  /**
   Resource owner available before setup for rollback.
   */
  const fixture: RouteFixture = {
    clientNamespace,
    clientPhysical,
    directory,
    serverNamespace,
    serverPhysical,
    async [Symbol.asyncDispose](): Promise<void> {
      await runSudoAllowingFailure({ args: ['ip', 'netns', 'delete', clientNamespace,], },);
      await runSudoAllowingFailure({ args: ['ip', 'netns', 'delete', serverNamespace,], },);
      await runSudoAllowingFailure({ args: ['rm', '--recursive', '--force', '--', directory,], },);
    },
  };
  try {
    await runSudoAllowingFailure({ args: ['ip', 'netns', 'delete', clientNamespace,], },);
    await runSudoAllowingFailure({ args: ['ip', 'netns', 'delete', serverNamespace,], },);
    await runSudo({ args: ['ip', 'netns', 'add', clientNamespace,], },);
    await runSudo({ args: ['ip', 'netns', 'add', serverNamespace,], },);
    await runSudo({ args: ['ip', 'link', 'add', clientPhysical, 'type', 'veth', 'peer', 'name', serverPhysical,], },);
    await runSudo({ args: ['ip', 'link', 'set', clientPhysical, 'netns', clientNamespace,], },);
    await runSudo({ args: ['ip', 'link', 'set', serverPhysical, 'netns', serverNamespace,], },);
    await runInNamespace({ namespace: clientNamespace, command: ['ip', 'link', 'set', 'lo', 'up',], },);
    await runInNamespace({ namespace: serverNamespace, command: ['ip', 'link', 'set', 'lo', 'up',], },);
    await runInNamespace({ namespace: clientNamespace, command: ['ip', 'address', 'add', '198.51.100.2/24', 'dev', clientPhysical,], },);
    await runInNamespace({ namespace: serverNamespace, command: ['ip', 'address', 'add', '198.51.100.1/24', 'dev', serverPhysical,], },);
    await runInNamespace({ namespace: clientNamespace, command: ['ip', '-6', 'address', 'add', '2001:db8:100::2/64', 'dev', clientPhysical,], },);
    await runInNamespace({ namespace: serverNamespace, command: ['ip', '-6', 'address', 'add', '2001:db8:100::1/64', 'dev', serverPhysical,], },);
    await runInNamespace({ namespace: serverNamespace, command: ['ip', 'address', 'add', '203.0.113.1/32', 'dev', 'lo',], },);
    await runInNamespace({ namespace: serverNamespace, command: ['ip', '-6', 'address', 'add', '2001:db8:200::1/128', 'dev', 'lo',], },);
    await runInNamespace({ namespace: clientNamespace, command: ['ip', 'link', 'set', clientPhysical, 'up',], },);
    await runInNamespace({ namespace: serverNamespace, command: ['ip', 'link', 'set', serverPhysical, 'up',], },);
    await runInNamespace({ namespace: clientNamespace, command: ['ip', 'route', 'add', 'default', 'via', '198.51.100.1', 'dev', clientPhysical,], },);
    await runInNamespace({ namespace: clientNamespace, command: ['ip', '-6', 'route', 'add', 'default', 'via', '2001:db8:100::1', 'dev', clientPhysical,], },);
    return fixture;
  }
  catch (error) {
    await fixture[Symbol.asyncDispose]();
    throw error;
  }
}

await using fixture = await createRouteFixture();
/**
 Client private key generated inside isolated namespace.
 */
const clientPrivate = (await runInNamespace({
  namespace: fixture.clientNamespace,
  command: ['wg', 'genkey',],
},)).trim();
/**
 Server private key generated inside isolated namespace.
 */
const serverPrivate = (await runInNamespace({
  namespace: fixture.serverNamespace,
  command: ['wg', 'genkey',],
},)).trim();
/**
 Private-key files consumed by `wg set` without shell input plumbing.
 */
const clientKeyPath = join(fixture.directory, 'client.key',);
const serverKeyPath = join(fixture.directory, 'server.key',);
await Promise.all([
  writeFile(clientKeyPath, `${clientPrivate}\n`, { mode: 0o600, },),
  writeFile(serverKeyPath, `${serverPrivate}\n`, { mode: 0o600, },),
],);
await runInNamespace({ namespace: fixture.clientNamespace, command: ['ip', 'link', 'add', 'keyprobe', 'type', 'wireguard',], },);
await runInNamespace({ namespace: fixture.clientNamespace, command: ['wg', 'set', 'keyprobe', 'private-key', clientKeyPath,], },);
/**
 Client public key admitted by server peer.
 */
const clientPublic = (await runInNamespace({ namespace: fixture.clientNamespace, command: ['wg', 'show', 'keyprobe', 'public-key',], },)).trim();
await runInNamespace({ namespace: fixture.clientNamespace, command: ['ip', 'link', 'delete', 'keyprobe',], },);
await runInNamespace({ namespace: fixture.serverNamespace, command: ['ip', 'link', 'add', 'wgpeer', 'type', 'wireguard',], },);
await runInNamespace({ namespace: fixture.serverNamespace, command: ['wg', 'set', 'wgpeer', 'private-key', serverKeyPath, 'listen-port', String(SERVER_PORT,), 'peer', clientPublic, 'allowed-ips', '10.200.0.1/32',], },);
/**
 Server public key used by client config.
 */
const serverPublic = (await runInNamespace({ namespace: fixture.serverNamespace, command: ['wg', 'show', 'wgpeer', 'public-key',], },)).trim();
await runInNamespace({ namespace: fixture.serverNamespace, command: ['ip', 'address', 'add', '10.200.0.2/32', 'dev', 'wgpeer',], },);
await runInNamespace({ namespace: fixture.serverNamespace, command: ['ip', 'link', 'set', 'wgpeer', 'up',], },);
await runInNamespace({ namespace: fixture.serverNamespace, command: ['ip', 'route', 'add', '10.200.0.1/32', 'dev', 'wgpeer',], },);
/**
 Client config whose non-default prefix contains public peer endpoint.
 */
const configPath = join(fixture.directory, 'wgtest.conf',);
await writeFile(configPath, [
  '[Interface]',
  `PrivateKey = ${clientPrivate}`,
  'Address = 10.200.0.1/32',
  '',
  '[Peer]',
  `PublicKey = ${serverPublic}`,
  `Endpoint = 203.0.113.1:${String(SERVER_PORT,)}`,
  'AllowedIPs = 10.200.0.2/32, 192.0.0.0/2, 2000::/3',
].join('\n',), { mode: 0o600, },);

//region Conflicting IVPN policy preflight

await runInNamespace({ namespace: fixture.clientNamespace, command: ['ip', '-4', 'rule', 'add', 'fwmark', '0xca6c', 'table', '17',], },);
/**
 Rejected up result before any interface creation.
 */
const conflict = await runFixtureCli({
  fixture,
  operation: 'up',
  configPath,
},);
assert.notEqual(conflict.exitCode, 0,);
assert.ok(conflict.stderr.includes('IVPN Desktop split tunneling is active',));
assert.notEqual((await runSudoAllowingFailure({ args: ['ip', 'netns', 'exec', fixture.clientNamespace, 'ip', 'link', 'show', 'dev', 'wgtest',], },)).exitCode, 0,);
await runInNamespace({ namespace: fixture.clientNamespace, command: ['ip', '-4', 'rule', 'delete', 'fwmark', '0xca6c', 'table', '17',], },);
await runInNamespace({ namespace: fixture.clientNamespace, command: ['ip', '-6', 'rule', 'add', 'fwmark', '0xca6c', 'table', '17',], },);
/**
 Rejected up result for IPv6 conflict before interface creation.
 */
const conflictV6 = await runFixtureCli({
  fixture,
  operation: 'up',
  configPath,
},);
assert.notEqual(conflictV6.exitCode, 0,);
assert.ok(conflictV6.stderr.includes('IVPN Desktop split tunneling is active',));
assert.notEqual((await runSudoAllowingFailure({ args: ['ip', 'netns', 'exec', fixture.clientNamespace, 'ip', 'link', 'show', 'dev', 'wgtest',], },)).exitCode, 0,);
await runInNamespace({ namespace: fixture.clientNamespace, command: ['ip', '-6', 'rule', 'delete', 'fwmark', '0xca6c', 'table', '17',], },);

//endregion Conflicting IVPN policy preflight

//region Endpoint recursion prevention and transfer

/**
 Successful built CLI activation after conflict removal.
 */
const activated = await runFixtureCli({
  fixture,
  operation: 'up',
  configPath,
},);
assert.equal(activated.exitCode, 0, activated.stderr,);
assert.ok(activated.stderr.includes(`Config for ${configPath} has no ExemptMark; Ghostty, Steam, Helium, Pale Moon, Firefox Nightly, ChatGPT, Interpreter, and Qure will use the tunnel.`,),);
assert.ok(activated.stderr.includes('Add `ExemptMark = 100` under `[Interface]`',),);
/**
 Positive interface fwmark naming policy table.
 */
const table = Number((await runInNamespace({ namespace: fixture.clientNamespace, command: ['wg', 'show', 'wgtest', 'fwmark',], },)).trim(),);
assert.ok(Number.isSafeInteger(table,) && (table > 0),);
/**
 Marked outer endpoint route which must remain on physical path.
 */
const endpointRoute = await runInNamespace({
  namespace: fixture.clientNamespace,
  command: ['ip', '-4', 'route', 'get', '203.0.113.1', 'mark', String(table,),],
},);
assert.ok(endpointRoute.includes(`dev ${fixture.clientPhysical}`,),);
assert.ok(endpointRoute.includes('via 198.51.100.1',),);
/**
 Unmarked physical gateway route proving main suppress rule runs first.
 */
const gatewayRoute = await runInNamespace({
  namespace: fixture.clientNamespace,
  command: ['ip', '-4', 'route', 'get', '198.51.100.1',],
},);
assert.ok(gatewayRoute.includes(`dev ${fixture.clientPhysical}`,),);
assert.equal(gatewayRoute.includes('dev wgtest',), false,);
/**
 Inner peer route selected from WireGuard policy table.
 */
const innerRoute = await runInNamespace({
  namespace: fixture.clientNamespace,
  command: ['ip', '-4', 'route', 'get', '10.200.0.2',],
},);
assert.ok(innerRoute.includes('dev wgtest',),);
assert.ok(innerRoute.includes(`table ${String(table,)}`,),);
/**
 IPv6 connected route preserved before broad WireGuard policy prefix.
 */
const gatewayRouteV6 = await runInNamespace({
  namespace: fixture.clientNamespace,
  command: ['ip', '-6', 'route', 'get', '2001:db8:100::1',],
},);
assert.ok(gatewayRouteV6.includes(`dev ${fixture.clientPhysical}`,),);
assert.equal(gatewayRouteV6.includes('dev wgtest',), false,);
/**
 IPv6 public route selected from shared WireGuard policy table.
 */
const innerRouteV6 = await runInNamespace({
  namespace: fixture.clientNamespace,
  command: ['ip', '-6', 'route', 'get', '2001:4860:4860::8888',],
},);
assert.ok(innerRouteV6.includes('dev wgtest',),);
assert.ok(innerRouteV6.includes(`table ${String(table,)}`,),);
await runInNamespace({ namespace: fixture.clientNamespace, command: ['ping', '-c', '1', '-W', '3', '10.200.0.2',], },);
/**
 Client transfer counters after bidirectional ping.
 */
const transferFields = (await runInNamespace({ namespace: fixture.clientNamespace, command: ['wg', 'show', 'wgtest', 'transfer',], },)).trim().split('\t',);
assert.ok(Number(transferFields.at(-2,),) > 0,);
assert.ok(Number(transferFields.at(-1,),) > 0,);
/**
 Successful teardown result.
 */
const deactivated = await runFixtureCli({
  fixture,
  operation: 'down',
  configPath,
},);
assert.equal(deactivated.exitCode, 0, deactivated.stderr,);
assert.equal(deactivated.stderr.includes('has no ExemptMark',), false,);
assert.notEqual((await runSudoAllowingFailure({ args: ['ip', 'netns', 'exec', fixture.clientNamespace, 'ip', 'link', 'show', 'dev', 'wgtest',], },)).exitCode, 0,);

//endregion Endpoint recursion prevention and transfer

console.log('wg-quicker endpoint routing integration passed',);
