/**
 Integration test for the one-time credential migration entry.
 
 Runs the built `migrate.mjs` as a separate process against a disposable
 home, Pi config, shell profile, and synthetic encrypted store.
 
 @module
 */

import { execFileSync, } from 'node:child_process';
import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { dirname, join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createIdentityFixture,
  writeEncryptedStore,
} from './config-store-fixture.ts';

//region Fixtures

/**
 Synthetic migrated values.
 */
const VALUES = {
  EXA_API_KEY: 'migrate-exa-sentinel',
  LINKUP_API_KEY: 'migrate-linkup-sentinel',
} as const;

/**
 Path to the built migration entry.
 */
const MIGRATE_ENTRY = join(
  import.meta.dirname,
  '..',
  'dist',
  'final',
  'node',
  'migrate.mjs',
);

//endregion Fixtures

await describe({
  name: 'credential migration',
  children: [
    it({
      name: 'moves Pi fields into the store and cleans the shell profile',
      fn: async () => {
        /**
         Local value for fixture.
         */
        const fixture = await createIdentityFixture();
        /**
         Local value for home.
         */
        const home = join(fixture.root, 'home',);
        /**
         Local value for piConfigPath.
         */
        const piConfigPath = join(home, '.pi', 'agent', 'extensions', 'pi-search-fetch.json',);
        /**
         Local value for bashrcPath.
         */
        const bashrcPath = join(home, '.bashrc',);
        await mkdir(dirname(piConfigPath,), { recursive: true, },);
        await writeFile(
          piConfigPath,
          JSON.stringify({
            exaApiKey: VALUES.EXA_API_KEY,
            linkupApiKey: VALUES.LINKUP_API_KEY,
            blocklist: ['badwikipedia.invalid',],
          },),
          'utf8',
        );
        await chmod(piConfigPath, 0o600,);
        await writeFile(
          bashrcPath,
          `export EXA_API_KEY=${VALUES.EXA_API_KEY}\nexport LINKUP_API_KEY=${VALUES.LINKUP_API_KEY}\nexport PATH="$HOME/bin:$PATH"\n`,
          'utf8',
        );
        /**
         Local value for encrypted.
         */
        const encrypted = await writeEncryptedStore({
          root: fixture.root,
          identityPath: fixture.identityPath,
          values: { OTHER: 'keep-other-sentinel', },
        },);
        /**
         Local value for output.
         */
        const output = execFileSync(
          process.execPath,
          [MIGRATE_ENTRY,],
          {
            cwd: fixture.root,
            env: {
              ...process.env,
              HOME: home,
              SOPS_AGE_KEY_FILE: fixture.identityPath,
              ENV_LOCAL_JSON_PATH: encrypted.storePath,
            },
            encoding: 'utf8',
          },
        );
        /**
         Local value for stored.
         */
        const stored = JSON.parse(execFileSync(
          'sops',
          ['--decrypt', '--output-type', 'json', encrypted.storePath,],
          {
            encoding: 'utf8',
            env: { ...process.env, SOPS_AGE_KEY_FILE: fixture.identityPath, },
            stdio: ['ignore', 'pipe', 'pipe',],
          },
        ),) as Record<string, unknown>;
        /**
         Local value for piConfig.
         */
        const piConfig = JSON.parse(await readFile(piConfigPath, 'utf8',),) as Record<string, unknown>;
        /**
         Local value for bashrc.
         */
        const bashrc = await readFile(bashrcPath, 'utf8',);

        expect(stored.EXA_API_KEY,).toBe(VALUES.EXA_API_KEY,);
        expect(stored.LINKUP_API_KEY,).toBe(VALUES.LINKUP_API_KEY,);
        expect(stored.OTHER,).toBe('keep-other-sentinel',);
        expect(piConfig,).toEqual({ blocklist: ['badwikipedia.invalid',], },);
        expect(bashrc,).toContain('ENV_LOCAL_JSON_PATH',);
        expect(bashrc,).not.toContain('export EXA_API_KEY',);
        expect(bashrc,).not.toContain('export LINKUP_API_KEY',);
        expect(output,).not.toContain(VALUES.EXA_API_KEY,);
        expect(output,).not.toContain(VALUES.LINKUP_API_KEY,);
        expect(output,).toContain('migration:');

        await rm(fixture.root, { recursive: true, force: true, },);
      },
    },),
    it({
      name: 'migrates the legacy pi-linkup apiKey and removes the legacy file',
      fn: async () => {
        /**
         Local value for fixture.
         */
        const fixture = await createIdentityFixture();
        /**
         Local value for home.
         */
        const home = join(fixture.root, 'home',);
        /**
         Local value for legacyPath.
         */
        const legacyPath = join(home, '.pi', 'agent', 'extensions', 'pi-linkup.json',);
        await mkdir(dirname(legacyPath,), { recursive: true, },);
        await writeFile(
          legacyPath,
          JSON.stringify({
            apiKey: VALUES.LINKUP_API_KEY,
            blocklist: [],
          },),
          'utf8',
        );
        await chmod(legacyPath, 0o600,);
        await writeFile(join(home, '.bashrc',), '', 'utf8',);
        /**
         Local value for encrypted.
         */
        const encrypted = await writeEncryptedStore({
          root: fixture.root,
          identityPath: fixture.identityPath,
          values: { OTHER: 'keep-other-sentinel', },
        },);

        execFileSync(
          process.execPath,
          [MIGRATE_ENTRY,],
          {
            cwd: fixture.root,
            env: {
              ...process.env,
              HOME: home,
              SOPS_AGE_KEY_FILE: fixture.identityPath,
              ENV_LOCAL_JSON_PATH: encrypted.storePath,
            },
            encoding: 'utf8',
          },
        );
        /**
         Local value for stored.
         */
        const stored = JSON.parse(execFileSync(
          'sops',
          ['--decrypt', '--output-type', 'json', encrypted.storePath,],
          {
            encoding: 'utf8',
            env: { ...process.env, SOPS_AGE_KEY_FILE: fixture.identityPath, },
            stdio: ['ignore', 'pipe', 'pipe',],
          },
        ),) as Record<string, unknown>;
        /**
         Local value for legacyRead.
         */
        let legacyRead: unknown;
        try {
          legacyRead = await readFile(legacyPath, 'utf8',);
        }
        catch (error: unknown) {
          legacyRead = error;
        }

        expect(stored.LINKUP_API_KEY,).toBe(VALUES.LINKUP_API_KEY,);
        expect((legacyRead as NodeJS.ErrnoException).code,).toBe('ENOENT',);
        await rm(fixture.root, { recursive: true, force: true, },);
      },
    },),
  ],
},);
