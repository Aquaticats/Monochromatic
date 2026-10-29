/**
 Tests for source classification and remote URL parsing.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  isRemoteInput,
  parseRemoteUrl,
} from './source.ts';

await describe({
  name: isRemoteInput.name,
  children: [
    it({
      name: 'recognizes scheme and scp-like URLs as remote',
      fn: async ({ expect, }) => {
        expect(isRemoteInput({ input: 'https://github.com/o/r.git', })).toBe(true);
        expect(isRemoteInput({ input: 'ssh://git@host/o/r.git', })).toBe(true);
        expect(isRemoteInput({ input: 'git@github.com:o/r.git', })).toBe(true);
      },
    }),

    it({
      name: 'treats local paths as not remote',
      fn: async ({ expect, }) => {
        expect(isRemoteInput({ input: '.', })).toBe(false);
        expect(isRemoteInput({ input: '/home/user/repo', })).toBe(false);
        expect(isRemoteInput({ input: './relative/path', })).toBe(false);
      },
    }),
  ],
});

await describe({
  name: parseRemoteUrl.name,
  children: [
    it({
      name: 'parses github https owner/repo and strips .git',
      fn: async ({ expect, }) => {
        const remote = parseRemoteUrl({ url: 'https://github.com/owner/repo.git', });
        expect(remote.host).toBe('github');
        expect(remote.owner).toBe('owner');
        expect(remote.repo).toBe('repo');
      },
    }),

    it({
      name: 'parses gitlab and keeps the final segment of a nested group path',
      fn: async ({ expect, }) => {
        const remote = parseRemoteUrl({ url: 'https://gitlab.com/group/sub/repo.git', });
        expect(remote.host).toBe('gitlab');
        expect(remote.owner).toBe('group');
        expect(remote.repo).toBe('repo');
      },
    }),

    it({
      name: 'parses scp-like github URLs',
      fn: async ({ expect, }) => {
        const remote = parseRemoteUrl({ url: 'git@github.com:owner/repo.git', });
        expect(remote.host).toBe('github');
        expect(remote.owner).toBe('owner');
        expect(remote.repo).toBe('repo');
      },
    }),

    it({
      name: 'marks an unsupported host as unknown but keeps the URL',
      fn: async ({ expect, }) => {
        const remote = parseRemoteUrl({ url: 'https://example.com/a/b', });
        expect(remote.host).toBe('unknown');
        expect(remote.url).toBe('https://example.com/a/b');
      },
    }),
  ],
});
