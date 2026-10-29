/**
 Network construction for disposable bypass integration test.
 
 @module
 */

import {
  runNamespaceIp,
  runSudo,
  runSudoAllowingFailure,
  type BypassFixture,
} from './tunnel-bypass-command-fixture.ts';

/**
 Removes namespace,
 host links,
 and root-owned runtime state idempotently.
 
 @param fixture - Fixture resources to remove.
 
 @example
 ```ts
 await cleanupBypassFixture({ fixture });
 ```
 */
export async function cleanupBypassFixture(
  { fixture, }: { readonly fixture: BypassFixture; },
): Promise<void> {
  await runSudoAllowingFailure({
    args: [
      'ip',
      'netns',
      'delete',
      fixture.namespace,
    ],
  },);
  await runSudoAllowingFailure({
    args: [
      'ip',
      'link',
      'delete',
      fixture.hostPrimary,
    ],
  },);
  await runSudoAllowingFailure({
    args: [
      'ip',
      'link',
      'delete',
      fixture.hostSecondary,
    ],
  },);
  await runSudoAllowingFailure({
    args: [
      'rm',
      '--recursive',
      '--force',
      '--',
      fixture.stateDirectory,
    ],
  },);
}

/**
 Creates host ends of two physical veth links.
 
 @param fixture - Names and runtime paths.
 
 @example
 ```ts
 await setupHostLinks({ fixture });
 ```
 */
export async function setupHostLinks(
  { fixture, }: { readonly fixture: BypassFixture; },
): Promise<void> {
  await cleanupBypassFixture({ fixture, },);
  await runSudo({ args: [
    'ip',
    'netns',
    'add',
    fixture.namespace,
  ], },);
  await runSudo({
    args: [
      'ip',
      'link',
      'add',
      fixture.hostPrimary,
      'type',
      'veth',
      'peer',
      'name',
      fixture.peerPrimary,
    ],
  },);
  await runSudo({
    args: [
      'ip',
      'link',
      'add',
      fixture.hostSecondary,
      'type',
      'veth',
      'peer',
      'name',
      fixture.peerSecondary,
    ],
  },);
  await runSudo({ args: [
    'ip',
    'link',
    'set',
    fixture.peerPrimary,
    'netns',
    fixture.namespace,
  ], },);
  await runSudo({ args: [
    'ip',
    'link',
    'set',
    fixture.peerSecondary,
    'netns',
    fixture.namespace,
  ], },);
  await runSudo({ args: [
    'ip',
    'address',
    'add',
    '198.51.100.1/24',
    'dev',
    fixture.hostPrimary,
  ], },);
  await runSudo({ args: [
    'ip',
    '-6',
    'address',
    'add',
    '2001:db8:1::1/64',
    'dev',
    fixture.hostPrimary,
  ], },);
  await runSudo({ args: [
    'ip',
    'address',
    'add',
    '198.51.101.1/24',
    'dev',
    fixture.hostSecondary,
  ], },);
  await runSudo({ args: [
    'ip',
    '-6',
    'address',
    'add',
    '2001:db8:2::1/64',
    'dev',
    fixture.hostSecondary,
  ], },);
  await runSudo({ args: [
    'ip',
    'link',
    'set',
    fixture.hostPrimary,
    'up',
  ], },);
  await runSudo({ args: [
    'ip',
    'link',
    'set',
    fixture.hostSecondary,
    'up',
  ], },);
}

/**
 Configures namespace link addresses,
 physical defaults,
 and dummy tunnel.
 
 @param fixture - Namespace fixture after host setup.
 
 @example
 ```ts
 await setupNamespaceLinks({ fixture });
 ```
 */
export async function setupNamespaceLinks(
  { fixture, }: { readonly fixture: BypassFixture; },
): Promise<void> {
  await runNamespaceIp({
    fixture,
    args: [
      'link',
      'set',
      'lo',
      'up',
    ],
  },);
  await runNamespaceIp({
    fixture,
    args: [
      'link',
      'set',
      fixture.peerPrimary,
      'up',
    ],
  },);
  await runNamespaceIp({
    fixture,
    args: [
      'link',
      'set',
      fixture.peerSecondary,
      'up',
    ],
  },);
  await runNamespaceIp({
    fixture,
    args: [
      'address',
      'add',
      '198.51.100.2/24',
      'dev',
      fixture.peerPrimary,
    ],
  },);
  await runNamespaceIp({
    fixture,
    args: [
      '-6',
      'address',
      'add',
      '2001:db8:1::2/64',
      'dev',
      fixture.peerPrimary,
    ],
  },);
  await runNamespaceIp({
    fixture,
    args: [
      'address',
      'add',
      '198.51.101.2/24',
      'dev',
      fixture.peerSecondary,
    ],
  },);
  await runNamespaceIp({
    fixture,
    args: [
      '-6',
      'address',
      'add',
      '2001:db8:2::2/64',
      'dev',
      fixture.peerSecondary,
    ],
  },);
  await runNamespaceIp({
    fixture,
    args: [
      'route',
      'add',
      'default',
      'via',
      '198.51.100.1',
      'dev',
      fixture.peerPrimary,
      'metric',
      '100',
    ],
  },);
  await runNamespaceIp({
    fixture,
    args: [
      '-6',
      'route',
      'add',
      'default',
      'via',
      '2001:db8:1::1',
      'dev',
      fixture.peerPrimary,
      'metric',
      '100',
    ],
  },);
  await runNamespaceIp({
    fixture,
    args: [
      'link',
      'add',
      'wgtest',
      'type',
      'dummy',
    ],
  },);
  await runNamespaceIp({
    fixture,
    args: [
      'link',
      'set',
      'wgtest',
      'up',
    ],
  },);
}
