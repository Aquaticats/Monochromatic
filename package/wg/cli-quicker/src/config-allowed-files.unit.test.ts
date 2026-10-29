import {
  mkdtemp,
  rm,
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
  loadConfig,
  parseConfigText,
} from '../dist/final/node/config.mjs';

/**
 Disposable config fixture directory.
 */
type TempDir = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates disposable config fixture directory.
 
 @returns Directory removed recursively on disposal.
 
 @example
 ```ts
 await using directory = await makeTempDir();
 ```
 */
async function makeTempDir(): Promise<TempDir> {
  /**
   Fresh operating-system temporary path.
   */
  const path = await mkdtemp(join(
    tmpdir(),
    'wg-quicker-config-',
  ),);
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        path,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 Captures synchronous failure.
 
 @param operation - Operation expected to throw.
 
 @returns Thrown value.
 
 @example
 ```ts
 captureThrown({ operation: () => { throw new Error('fixture'); } });
 ```
 */
function captureThrown(
  { operation, }: { readonly operation: () => unknown; },
): unknown {
  try {
    operation();
  }
  catch (error: unknown) {
    return error;
  }
  throw new Error('Expected operation to throw.',);
}

/**
 Captures asynchronous failure.
 
 @param operation - Operation expected to reject.
 
 @returns Rejection value.
 
 @example
 ```ts
 await captureRejected({ operation: async () => { throw new Error('fixture'); } });
 ```
 */
async function captureRejected(
  { operation, }: { readonly operation: () => Promise<unknown>; },
): Promise<unknown> {
  try {
    await operation();
  }
  catch (error: unknown) {
    return error;
  }
  throw new Error('Expected operation to reject.',);
}

await describe({
  name: '',
  children: [
    describe({
      name: parseConfigText.name,
      children: [
        it({
          name: 'associates source paths with containing peer insertion point',
          fn: async () => {
            /**
             Parsed config with source directive in first of two peers.
             */
            const config = parseConfigText({
              interfaceName: 'wg0',
              text: [
                '[Interface]',
                'PrivateKey = fixture',
                '[Peer]',
                'PublicKey = first',
                'AllowedIPsFromFiles = allowed.txt disallowed.txt',
                'Endpoint = first.example:51820',
                '[Peer]',
                'PublicKey = second',
                'AllowedIPs = 192.0.2.0/24',
              ].join('\n',),
            },);
            expect(config.allowedFromFiles,).toEqual([{
              allowed: 'allowed.txt',
              disallowed: 'disallowed.txt',
              peerIndex: 0,
              wgLineIndex: 4,
            },],);
            expect(config.wgConfig,).not.toContain('AllowedIPsFromFiles',);
          },
        },),
        ...[
          {
            name: 'rejects source directive in Interface section',
            lines: [
              '[Interface]',
              'AllowedIPsFromFiles = allowed.txt disallowed.txt',
            ],
            expected: 'must occur inside a [Peer] section',
          },
          {
            name: 'rejects literal AllowedIPs before source directive in same peer',
            lines: [
              '[Peer]',
              'AllowedIPs = 0.0.0.0/0',
              'AllowedIPsFromFiles = allowed.txt disallowed.txt',
            ],
            expected: 'cannot contain both AllowedIPs and AllowedIPsFromFiles',
          },
          {
            name: 'rejects literal AllowedIPs after source directive in same peer',
            lines: [
              '[Peer]',
              'AllowedIPsFromFiles = allowed.txt disallowed.txt',
              'AllowedIPs = 0.0.0.0/0',
            ],
            expected: 'cannot contain both AllowedIPs and AllowedIPsFromFiles',
          },
          {
            name: 'rejects duplicate source directives in same peer',
            lines: [
              '[Peer]',
              'AllowedIPsFromFiles = allowed.txt disallowed.txt',
              'AllowedIPsFromFiles = other-allowed.txt other-disallowed.txt',
            ],
            expected: 'cannot contain more than one AllowedIPsFromFiles directive',
          },
        ].map(function invalidDirective({
          name,
          lines,
          expected,
        },) {
          return it({
            name,
            fn: async () => {
              /**
               Config semantic failure for invalid fixture.
               */
              const error = captureThrown({
                operation: function parseInvalidConfig(): unknown {
                  return parseConfigText({
                    interfaceName: 'wg0',
                    text: lines.join('\n',),
                  },);
                },
              },);
              expect(String(error,),).toContain(expected,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: loadConfig.name,
      children: [
        it({
          name: 'inserts generated line inside each owning peer',
          fn: async () => {
            await using directory = await makeTempDir();
            /**
             First peer allowed input.
             */
            const firstAllowed = join(directory.path, 'first-allowed.txt',);
            /**
             First peer disallowed input.
             */
            const firstDisallowed = join(directory.path, 'first-disallowed.txt',);
            /**
             Second peer allowed input.
             */
            const secondAllowed = join(directory.path, 'second-allowed.txt',);
            /**
             Second peer disallowed input.
             */
            const secondDisallowed = join(directory.path, 'second-disallowed.txt',);
            await Promise.all([
              writeFile(firstAllowed, '10.0.0.0/8\n',),
              writeFile(firstDisallowed, '10.0.0.0/9\n127.0.0.0/8\n::1/128\n',),
              writeFile(secondAllowed, '192.0.2.0/24\n',),
              writeFile(secondDisallowed, '127.0.0.0/8\n::1/128\n',),
            ],);
            /**
             Config path containing independent source directives.
             */
            const configPath = join(directory.path, 'wg-test.conf',);
            await writeFile(
              configPath,
              [
                '[Interface]',
                'PrivateKey = fixture',
                '[Peer]',
                'PublicKey = first',
                `AllowedIPsFromFiles = ${firstAllowed} ${firstDisallowed}`,
                'Endpoint = first.example:51820',
                '[Peer]',
                'PublicKey = second',
                `AllowedIPsFromFiles = ${secondAllowed} ${secondDisallowed}`,
                'Endpoint = second.example:51820',
              ].join('\n',),
            );
            /**
             Fully expanded config used by up lifecycle.
             */
            const config = await loadConfig({
              arg: configPath,
              expandAllowedIps: true,
            },);
            expect(config.wgConfig,).toContain(
              'PublicKey = first\nAllowedIPs = 10.128.0.0/9\nEndpoint = first.example:51820',
            );
            expect(config.wgConfig,).toContain(
              'PublicKey = second\nAllowedIPs = 192.0.2.0/24\nEndpoint = second.example:51820',
            );
          },
        },),
        it({
          name: 'skips file reads and generation for down parsing',
          fn: async () => {
            await using directory = await makeTempDir();
            /**
             Config path pointing source directive at absent files.
             */
            const configPath = join(directory.path, 'wg-down.conf',);
            await writeFile(
              configPath,
              [
                '[Interface]',
                'PrivateKey = fixture',
                '[Peer]',
                'PublicKey = peer',
                'AllowedIPsFromFiles = absent-allowed.txt absent-disallowed.txt',
              ].join('\n',),
            );
            /**
             Parsed down config that retains paths without reading them.
             */
            const config = await loadConfig({
              arg: configPath,
              expandAllowedIps: false,
            },);
            expect(config.allowedFromFiles,).toHaveLength(1,);
            expect(config.wgConfig,).not.toContain('AllowedIPsFromFiles',);
          },
        },),
        it({
          name: 'surfaces absent source file during up parsing',
          fn: async () => {
            await using directory = await makeTempDir();
            /**
             Config path pointing source directive at absent files.
             */
            const configPath = join(directory.path, 'wg-up.conf',);
            await writeFile(
              configPath,
              [
                '[Interface]',
                'PrivateKey = fixture',
                '[Peer]',
                'PublicKey = peer',
                'AllowedIPsFromFiles = absent-allowed.txt absent-disallowed.txt',
              ].join('\n',),
            );
            /**
             File-read failure from required up expansion.
             */
            const error = await captureRejected({
              operation: async function loadUpConfig(): Promise<unknown> {
                return await loadConfig({
                  arg: configPath,
                  expandAllowedIps: true,
                },);
              },
            },);
            expect(String(error,),).toContain('ENOENT',);
          },
        },),
      ],
    },),
  ],
},);
