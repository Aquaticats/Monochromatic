/**
 Disposable fixture factory for bypass integration test.
 
 @module
 */

import { createHash, } from 'node:crypto';
import { mkdtemp, } from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  runBypassOperationAllowingFailure,
  type BypassFixture,
} from './tunnel-bypass-command-fixture.ts';
import {
  cleanupBypassFixture,
  setupHostLinks,
  setupNamespaceLinks,
} from './tunnel-bypass-network-fixture.ts';

/**
 PID characters retained for short Linux device names.
 */
const NAME_SUFFIX_LENGTH = 5;

/**
 Hex characters retained from production interface-state key.
 */
const STATE_KEY_LENGTH = 32;

/**
 Creates dual-stack physical links and dummy tunnel in disposable namespace.
 
 @returns Fully configured fixture with deterministic cleanup.
 
 @example
 ```ts
 await using fixture = await createBypassFixture();
 ```
 */
export async function createBypassFixture(): Promise<BypassFixture> {
  /**
   Short process suffix keeping Linux device names under limit.
   */
  const nameSuffix = String(process.pid,)
    .slice(-NAME_SUFFIX_LENGTH,);
  /**
   Namespace unique to test process.
   */
  const namespace = `wgq${nameSuffix}`;
  /**
   Primary host veth name.
   */
  const hostPrimary = `h0${nameSuffix}`;
  /**
   Secondary host veth name.
   */
  const hostSecondary = `h1${nameSuffix}`;
  /**
   Primary namespace veth name.
   */
  const peerPrimary = `n0${nameSuffix}`;
  /**
   Secondary namespace veth name.
   */
  const peerSecondary = `n1${nameSuffix}`;
  /**
   Private runtime directory removed with fixture.
   */
  const stateDirectory = await mkdtemp(join(
    tmpdir(),
    'wgq-bypass-integration-',
  ),);
  /**
   Interface state key matching production hash.
   */
  const stateKey = createHash('sha256',)
    .update('wgtest',)
    .digest('hex',)
    .slice(
    0,
    STATE_KEY_LENGTH,
    );
  /**
   Final state path used by checks.
   */
  const statePath = join(
    stateDirectory,
    `interface-${stateKey}.json`,
  );
  /**
   Fixture object available during setup and disposal.
   */
  const fixture: BypassFixture = {
    namespace,
    hostPrimary,
    hostSecondary,
    peerPrimary,
    peerSecondary,
    stateDirectory,
    statePath,
    async [Symbol.asyncDispose](): Promise<void> {
      await runBypassOperationAllowingFailure({
        fixture,
        source: "await removeExemptRule({ interfaceName: 'wgtest' });",
      },);
      await cleanupBypassFixture({ fixture, },);
    },
  };
  try {
    await setupHostLinks({ fixture, },);
    await setupNamespaceLinks({ fixture, },);
    return fixture;
  }
  catch (error) {
    await cleanupBypassFixture({ fixture, },);
    throw error;
  }
}
