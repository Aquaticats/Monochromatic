import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  lockedPnpm,
  pnpmCopyLines,
} from './native-pnpm-snapshot.ts';
import { testContainerfile, } from './native-test-containerfile.ts';

/**
 The platform of the gate image: CachyOS on x86-64, which links the GNU C library.
 */
const platform = 'linux-x64';

/**
 A lock with one pnpm row, shaped like the repository's.
 */
function lockWith({ version, url, sha256, }: {
  readonly version: string;
  readonly url: string;
  readonly sha256: string;
},): string {
  return JSON.stringify({
    tools: {
      pnpm: {
        version,
        artifacts: { [platform]: { url, sha256, }, },
      },
    },
  });
}

await describe({
  name: lockedPnpm.name,
  children: [
    it({
      name: 'returns the version, address and checksum of the row for the image platform',
      fn: async () => {
        const locked = lockedPnpm({
          lockText: lockWith({
            version: '1.2.3',
            url: 'https://example.invalid/pnpm.tar.gz',
            sha256: 'ab'.repeat(32),
          }),
        });
        expect(locked).toEqual({
          version: '1.2.3',
          url: 'https://example.invalid/pnpm.tar.gz',
          sha256: 'ab'.repeat(32),
        });
      },
    }),
    it({
      name: 'refuses a lock without a pnpm row or without the platform archive',
      fn: async () => {
        expect(() => lockedPnpm({ lockText: JSON.stringify({ tools: {}, }), })).toThrow('pnpm');
        expect(() => lockedPnpm({ lockText: JSON.stringify({ tools: { pnpm: { version: '1', artifacts: {}, }, }, }), })).toThrow(platform);
      },
    }),
    it({
      name: 'reads the repository lock, whose archive address names the row version',
      fn: async () => {
        const lockText = await readFile(
          join(
            import.meta.dirname,
            '..',
            '..',
            '..',
            '..',
            'toolchain.lock.json',
          ),
          'utf8',
        );
        const locked = lockedPnpm({ lockText, });
        expect(locked.url).toContain(locked.version);
        expect(locked.sha256).toMatch(/^[0-9a-f]{64}$/u);
      },
    }),
  ],
});

await describe({
  name: pnpmCopyLines.name,
  children: [
    it({
      name: 'ends the test stage with a pnpm that is checked at build time, after the last compile',
      fn: async () => {
        const lines = testContainerfile({
          base: 'sha256:0',
          jobs: 1,
        })
          .split('\n');
        const installed = pnpmCopyLines();
        expect(installed.length).toBeGreaterThan(0);
        expect(installed.some(function checksStart(line) {
          return line.startsWith('RUN ') && line.includes('pnpm') && line.includes('--version');
        })).toBe(true);
        // The file ends in a newline, so the last element is empty; the install lines come right before it.
        expect(lines.slice(-1 - installed.length, -1)).toEqual(installed);
        expect(lines.at(-2 - installed.length)).toContain('--no-run');
      },
    }),
  ],
});
