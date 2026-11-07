/**
 Checks that the catalog-specifiers workflow gets its git policy CLI from the GitHub release archive that
 `toolchain.lock.json` pins, the way the repository tools action fetches and verifies the other repository tools, and never
 from crates.io.

 A version that carries an `-alpha.N` label is published to GitHub releases only, so asking crates.io for the version the
 lock pins fails with `no version matching requirement` (the cause of every red run of the workflow before this check).
 The files are read as text; nothing is executed.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { readFile, } from 'node:fs/promises';

/**
 The workflow under test.
 */
const WORKFLOW = await readFile(new URL('../../../../.github/workflows/catalog-specifiers.yml', import.meta.url,), 'utf8',);

/**
 The lock the workflow installs from.
 */
const LOCK = JSON.parse(await readFile(new URL('../../../../toolchain.lock.json', import.meta.url,), 'utf8',),) as {
  readonly tools: Readonly<Record<string, { readonly artifacts: Readonly<Record<string, { readonly url: string; readonly sha256: string; }>>; }>>;
};

await describe({
  name: 'catalog-specifiers workflow install',
  children: [
    it({
      name: 'installs the git policy CLI with the repository tools action, not with cargo-binstall',
      fn: async function usesRepositoryToolsAction() {
        expect(WORKFLOW,).toContain('uses: ./.github/actions/repo-tools-setup',);
        expect(WORKFLOW,).toContain('tools: git-policy-cli',);
        expect(WORKFLOW,).not.toContain('binstall',);
        expect(WORKFLOW,).not.toContain('crates.io',);
      },
    },),
    it({
      name: 'has a GitHub release archive address and a SHA-256 in the lock for both Linux platforms the action supports',
      fn: async function lockHasArchives() {
        /**
         The lock row of the git policy CLI.
         */
        const row = LOCK.tools['cargo:monochromatic-git-policy-cli'];
        for (const platform of ['linux-x64', 'linux-arm64',] as const) {
          const artifact = row?.artifacts[platform];
          expect(artifact?.url.startsWith('https://github.com/Aquaticats/Monochromatic/releases/download/',),).toBe(true,);
          expect(artifact?.sha256,).toMatch(/^[0-9a-f]{64}$/u,);
        }
      },
    },),
  ],
},);
