/**
 Semantics of the in-memory adapter, checked against the rules the
 `node:fs/promises` adapter follows: `exists` and `readSymbolicLink` look
 at the entry itself, the other probes look through links.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ABSENT,
  createMemoryRootFilesystem,
} from '@monochromatic-dev/fs-path-core';

await describe({
  name: createMemoryRootFilesystem.name,
  children: [
    it({
      name: 'an empty tree has only the root directory',
      fn: async () => {
        const fs = createMemoryRootFilesystem();
        expect(await fs.exists('/',),).toBe(true,);
        expect(await fs.isDirectory('/',),).toBe(true,);
        expect(await fs.isFile('/',),).toBe(false,);
        expect(await fs.exists('/x',),).toBe(false,);
        expect(await fs.readTextFile('/x',),).toBe(ABSENT,);
        expect(await fs.readSymbolicLink('/x',),).toBe(ABSENT,);
      },
    },),
    it({
      name: 'a file exists, is a file, reads its text, and implies its ancestors',
      fn: async () => {
        const fs = createMemoryRootFilesystem({ files: { '/repo/pkg/mise.toml': '[monorepo]\n', }, },);
        expect(await fs.exists('/repo/pkg/mise.toml',),).toBe(true,);
        expect(await fs.isFile('/repo/pkg/mise.toml',),).toBe(true,);
        expect(await fs.isDirectory('/repo/pkg/mise.toml',),).toBe(false,);
        expect(await fs.readTextFile('/repo/pkg/mise.toml',),).toBe('[monorepo]\n',);
        expect(await fs.readSymbolicLink('/repo/pkg/mise.toml',),).toBe(ABSENT,);
        expect(await fs.isDirectory('/repo/pkg',),).toBe(true,);
        expect(await fs.isDirectory('/repo',),).toBe(true,);
        expect(await fs.exists('/repo',),).toBe(true,);
        expect(await fs.readTextFile('/repo',),).toBe(ABSENT,);
      },
    },),
    it({
      name: 'a declared empty directory is a directory and not a file',
      fn: async () => {
        const fs = createMemoryRootFilesystem({ directories: ['/repo/.git',], },);
        expect(await fs.isDirectory('/repo/.git',),).toBe(true,);
        expect(await fs.isFile('/repo/.git',),).toBe(false,);
        expect(await fs.exists('/repo/.git',),).toBe(true,);
      },
    },),
    it({
      name: 'normalizes declared and probed paths',
      fn: async () => {
        const fs = createMemoryRootFilesystem({ files: { '/repo/a/../mise.toml': '', }, },);
        expect(await fs.isFile('/repo/mise.toml',),).toBe(true,);
        expect(await fs.isFile('/repo//mise.toml',),).toBe(true,);
        expect(await fs.isFile('/repo/./mise.toml',),).toBe(true,);
        expect(await fs.isDirectory('/repo/',),).toBe(true,);
        expect(await fs.isDirectory('/repo/a',),).toBe(false,);
      },
    },),
    it({
      name: 'a dangling link exists and reads its target text but is neither file nor directory',
      fn: async () => {
        const fs = createMemoryRootFilesystem({ links: { '/repo/.git/HEAD': 'refs/heads/missing', }, },);
        expect(await fs.exists('/repo/.git/HEAD',),).toBe(true,);
        expect(await fs.readSymbolicLink('/repo/.git/HEAD',),).toBe('refs/heads/missing',);
        expect(await fs.isFile('/repo/.git/HEAD',),).toBe(false,);
        expect(await fs.isDirectory('/repo/.git/HEAD',),).toBe(false,);
        expect(await fs.readTextFile('/repo/.git/HEAD',),).toBe(ABSENT,);
        expect(await fs.isDirectory('/repo/.git',),).toBe(true,);
      },
    },),
    it({
      name: 'a link to a file resolves relative targets against the link directory',
      fn: async () => {
        const fs = createMemoryRootFilesystem({
          files: { '/repo/real/HEAD': 'ref: refs/heads/main\n', },
          links: { '/repo/.git/HEAD': '../real/HEAD', },
        },);
        expect(await fs.isFile('/repo/.git/HEAD',),).toBe(true,);
        expect(await fs.readTextFile('/repo/.git/HEAD',),).toBe('ref: refs/heads/main\n',);
        expect(await fs.readSymbolicLink('/repo/.git/HEAD',),).toBe('../real/HEAD',);
      },
    },),
    it({
      name: 'a link to a file with an absolute target',
      fn: async () => {
        const fs = createMemoryRootFilesystem({
          files: { '/store/deno.json': '{}', },
          links: { '/repo/deno.json': '/store/deno.json', },
        },);
        expect(await fs.isFile('/repo/deno.json',),).toBe(true,);
        expect(await fs.readTextFile('/repo/deno.json',),).toBe('{}',);
      },
    },),
    it({
      name: 'a link to a directory is a directory and paths through it resolve',
      fn: async () => {
        const fs = createMemoryRootFilesystem({
          directories: ['/store/gitdir/objects',],
          files: { '/store/gitdir/HEAD': 'ref: refs/heads/main\n', },
          links: { '/repo/.git': '/store/gitdir', },
        },);
        expect(await fs.isDirectory('/repo/.git',),).toBe(true,);
        expect(await fs.isFile('/repo/.git',),).toBe(false,);
        expect(await fs.isFile('/repo/.git/HEAD',),).toBe(true,);
        expect(await fs.readTextFile('/repo/.git/HEAD',),).toBe('ref: refs/heads/main\n',);
        expect(await fs.isDirectory('/repo/.git/objects',),).toBe(true,);
        expect(await fs.exists('/repo/.git/HEAD',),).toBe(true,);
        expect(await fs.exists('/repo/.git/missing',),).toBe(false,);
      },
    },),
    it({
      name: 'a link chain resolves through every hop',
      fn: async () => {
        const fs = createMemoryRootFilesystem({
          files: { '/c': 'end', },
          links: {
            '/a': 'b',
            '/b': 'c',
          },
        },);
        expect(await fs.isFile('/a',),).toBe(true,);
        expect(await fs.readTextFile('/a',),).toBe('end',);
      },
    },),
    it({
      name: 'a link cycle exists but is neither file nor directory',
      fn: async () => {
        const fs = createMemoryRootFilesystem({
          links: {
            '/a': 'b',
            '/b': 'a',
          },
        },);
        expect(await fs.exists('/a',),).toBe(true,);
        expect(await fs.isFile('/a',),).toBe(false,);
        expect(await fs.isDirectory('/a',),).toBe(false,);
        expect(await fs.readTextFile('/a',),).toBe(ABSENT,);
      },
    },),
  ],
},);
