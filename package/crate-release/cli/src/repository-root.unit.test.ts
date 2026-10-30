import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CrateReleaseError } from './error.ts';
import { findRepositoryRoot } from './repository-root.ts';

/**
 Build an existence check over a fixed set of paths.

 @param paths - paths that exist
 @returns a check that is true for exactly those paths
 */
function existing(paths: readonly string[]): (path: string) => boolean {
  return function exists(path) {
    return paths.includes(path);
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: findRepositoryRoot.name,
      children: [
        it({
          name: 'finds the root three levels above the package directory',
          fn: async () => {
            expect(findRepositoryRoot({
              startDirectory: '/repo/package/crate-release/cli',
              exists: existing(['/repo/pnpm-workspace.yaml']),
            })).toBe('/repo');
          },
        }),
        it({
          name: 'returns the start directory when it holds the marker',
          fn: async () => {
            expect(findRepositoryRoot({
              startDirectory: '/repo',
              exists: existing(['/repo/pnpm-workspace.yaml']),
            })).toBe('/repo');
          },
        }),
        it({
          name: 'returns the nearest directory that holds the marker',
          fn: async () => {
            expect(findRepositoryRoot({
              startDirectory: '/repo/inner/package/x',
              exists: existing(['/repo/pnpm-workspace.yaml', '/repo/inner/pnpm-workspace.yaml']),
            })).toBe('/repo/inner');
          },
        }),
        it({
          name: 'fails and names the start directory and the marker when no ancestor holds it',
          fn: async () => {
            let caught: unknown;
            try {
              findRepositoryRoot({ startDirectory: '/a/b', exists: existing([]) });
            }
            catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('/a/b');
            expect((caught as Error).message).toContain('pnpm-workspace.yaml');
            expect((caught as Error).message).toContain('--root');
          },
        }),
      ],
    }),
  ],
});
