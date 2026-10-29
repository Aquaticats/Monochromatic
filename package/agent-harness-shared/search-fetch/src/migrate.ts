/**
 One-time migration of search-fetch credentials into the encrypted store.
 
 Reads the old plaintext Pi configs, writes the two provider keys with sops
 through the env-local-json write helper, verifies by decrypting in memory,
 then removes the old fields, the legacy file, and the shell exports. Every
 backup is kept under `$HOME/temp/agent` with the original mode.
 
 Invoked by `mise run secrets:migrate`.
 
 @module
 */

import { rm, } from 'node:fs/promises';
import { homedir, } from 'node:os';
import {
  dirname,
  resolve,
} from 'node:path';

import {
  readEnvLocalJson,
} from '@monochromatic-dev/env-local-json-core/ts';
import { writeEnvLocalJsonValues, } from '@monochromatic-dev/env-local-json-core/ts/write.ts';

import {
  EXA_API_KEY_ENV,
  LINKUP_API_KEY_ENV,
} from './config-constants.ts';
import {
  configPathForHome,
  legacyConfigPathForHome,
} from './config-paths.ts';
import {
  backupDirectoryName,
  backupFiles,
  backupPath,
  credentialValue,
  readJsonIfPresent,
  requireFile,
  restoreBackups,
  writeJsonAtomic,
} from './migrate-files.ts';
import { cleanBashrc, } from './migrate-bashrc.ts';

/**
 Mode for the rewritten Pi config.
 */
const PI_CONFIG_FILE_MODE = 0o600;

/**
 Run the one-time credential migration.
 
 @throws when the store is missing, a config cannot be parsed, the write or
 verification fails, or a backup cannot be created
 
 @example
 ```ts
 await runMigration();
 ```
 */
async function runMigration(): Promise<void> {
  /**
   Local value for home.
   */
  const home = process.env
    .HOME
    ?? homedir();
  /**
   Local value for piConfigPath.
   */
  const piConfigPath = configPathForHome({ home, },);
  /**
   Local value for legacyConfigPath.
   */
  const legacyConfigPath = legacyConfigPathForHome({ home, },);
  /**
   Local value for storePath.
   */
  const storePath = resolve(
    process.cwd(),
    process.env
      .ENV_LOCAL_JSON_PATH
      ?? '.env.local.json',
  );
  await requireFile({
    path: storePath,
    description: 'encrypted store',
  },);
  /**
   Local value for piConfig.
   */
  const piConfig = await readJsonIfPresent({ path: piConfigPath, },);
  /**
   Local value for legacyConfig.
   */
  const legacyConfig = await readJsonIfPresent({ path: legacyConfigPath, },);
  /**
   Environment without provider keys so store verification cannot be skipped by
   an ambient export, as the reader only fills keys absent from the environment.
   */
  const migrationEnv: NodeJS.ProcessEnv = Object.fromEntries(
    Object.entries(process.env,)
      .filter(function isNotMigratedKey([key,],) {
        return (key !== EXA_API_KEY_ENV) && (key !== LINKUP_API_KEY_ENV);
      },),
  );
  /**
   Fill-in values to write into the store.
   */
  const values: Record<string, string> = {};
  /**
   Local value for exaValue.
   */
  const exaValue = credentialValue({
    record: piConfig,
    key: 'exaApiKey',
    path: piConfigPath,
  },);
  if (exaValue.present)
    values[EXA_API_KEY_ENV] = exaValue.value;
  /**
   Local value for piLinkupValue.
   */
  const piLinkupValue = credentialValue({
    record: piConfig,
    key: 'linkupApiKey',
    path: piConfigPath,
  },);
  /**
   Local value for legacyLinkupValue.
   */
  const legacyLinkupValue = credentialValue({
    record: legacyConfig,
    key: 'apiKey',
    path: legacyConfigPath,
  },);
  /**
   Local value for linkupValue.
   */
  const linkupValue = piLinkupValue.present
    ? piLinkupValue
    : legacyLinkupValue;
  if (linkupValue.present)
    values[LINKUP_API_KEY_ENV] = linkupValue.value;
  /**
   Local value for backupDir.
   */
  const backupDir = backupPath({
    home,
    name: backupDirectoryName(),
  },);
  /**
   Local value for bashrcPath.
   */
  const bashrcPath = `${home}/.bashrc`;
  /**
   Local value for backedUp.
   */
  const backedUp = await backupFiles({
    backupDir,
    paths: [
      piConfigPath,
      legacyConfigPath,
      storePath,
      bashrcPath,
    ],
  },);

  try {
    if (Object.keys(values,)
      .length
      > 0) {
      await writeEnvLocalJsonValues({
        path: storePath,
        values,
        env: migrationEnv,
        home,
      },);
      await verifyStore({
        storePath,
        values,
        home,
        env: migrationEnv,
      },);
    }
    if (piConfig.present) {
      /**
       Local value for piConfigWithoutCredentials.
       */
      const {
        exaApiKey: _exaApiKey,
        linkupApiKey: _linkupApiKey,
        ...piConfigWithoutCredentials
      } = piConfig.value;
      await writeJsonAtomic({
        path: piConfigPath,
        value: piConfigWithoutCredentials,
        mode: PI_CONFIG_FILE_MODE,
      },);
    }
    if (legacyConfig.present)
      await rm(legacyConfigPath,);
    await cleanBashrc({
      bashrcPath,
      storePath,
    },);
  }
  catch (error: unknown) {
    await restoreBackups({
      backupDir,
      backedUp,
    },);
    throw error;
  }

  console.log(
    `migration: store updated; Pi credentials removed; ${legacyConfig.present ? 'legacy file removed' : 'no legacy file'}; `
      + `.bashrc cleaned; backups: ${backupDir}`,
  );
}

/**
 Verify the migrated values by decrypting the store in memory.
 
 @param storePath - encrypted store path
 
 @param values - values expected in the store
 
 @param home - home directory used by the identity resolver
 
 @param env - environment without provider keys
 
 @throws when the store cannot be read or a migrated value differs
 */
async function verifyStore(
  {
    storePath,
    values,
    home,
    env,
  }: {
    readonly storePath: string;
    readonly values: Readonly<Record<string, string>>;
    readonly home: string;
    readonly env: Readonly<NodeJS.ProcessEnv>;
  },
): Promise<void> {
  /**
   Local value for verified.
   */
  const verified = await readEnvLocalJson({
    keys: Object.keys(values,),
    path: storePath,
    cwd: dirname(storePath,),
    env,
    home,
  },);
  for (const [key, value] of Object.entries(values,)) {
    if (verified.values[key] !== value)
      throw new Error(`migration verification failed for ${key}`,);
  }
}

await runMigration();
