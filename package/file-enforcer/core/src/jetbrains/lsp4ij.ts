import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { l, } from '../logger.ts';
import {
  ABSENT_FILE_CONTENT,
  overwrite,
  readExisting,
} from '../io/write.ts';
import { ABSENT_XML_ENTRY, } from '../pipeline/xml.ts';
import { trackRead, } from '../tracker.ts';
import {
  findBaseServerEntry,
  updatedLanguageSettingsXml,
  updatedUserDefinedXml,
} from './lsp4ij-apply.ts';
import {
  latestJetbrainsOptionsDirectory,
  NO_JETBRAINS_OPTIONS_DIRECTORY,
} from './options-dir.ts';

import type {
  Lsp4ijOptionsFiles,
  Lsp4ijServerSettings,
} from './lsp4ij-types.ts';

//region Helpers: read tracked files

/**
 Default LSP4IJ persistent-state file names under a JetBrains options directory.
 */
const DEFAULT_OPTIONS_FILES: Lsp4ijOptionsFiles = {
  languageSettings: 'LanguageServersSettings.xml',
  userDefined: 'UserDefinedLanguageServerSettings.xml',
};

/**
 Reads a tracked local file, registering it for watch mode.
 
 @param filePath - File path to read and register for watch mode.
 
 @returns File content, or {@link ABSENT_FILE_CONTENT} when the file does not exist.
 
 @example
 ```ts
 await readTrackedExisting({ filePath });
 ```
 */
async function readTrackedExisting(
  { filePath, }: { readonly filePath: string; },
): Promise<string | typeof ABSENT_FILE_CONTENT> {
  trackRead(filePath,);
  return await readExisting(filePath,);
}

//endregion Helpers

//region Public task: non-blocking editor-local LSP4IJ sync

/**
 Syncs LSP4IJ language-server settings in the latest JetBrains product config,
 located via {@link latestJetbrainsOptionsDirectory}. Missing JetBrains
 config, settings files, or base server (matched via
 {@link findBaseServerEntry}) are intentionally non-blocking because not
 every developer uses this editor integration. Corrupt config discovery,
 unreadable settings, XML update failures (from
 {@link updatedLanguageSettingsXml} and {@link updatedUserDefinedXml}), and
 write failures (from {@link overwrite}) propagate so broken local state
 does not look like absence.
 
 @param settings - Declarative LSP4IJ server settings policy.
 
 @example
 ```ts
 await manageLsp4ijServerSettings({
   productPrefixes: ['IntelliJIdea'],
   baseServerMatch: { templateId: 'harper-ls' },
   scopedServers: [],
 });
 ```
 */
export async function manageLsp4ijServerSettings(settings: Lsp4ijServerSettings,): Promise<void> {
  /**
   Function-scoped tagged logger.
   */
  const log = tagged({
    tag: manageLsp4ijServerSettings.name,
    l,
  },);
  /**
   Latest matching JetBrains product options directory, or absent sentinel.
   */
  const latest = await latestJetbrainsOptionsDirectory({ productPrefixes: settings.productPrefixes, },);
  if (latest === NO_JETBRAINS_OPTIONS_DIRECTORY) {
    log.warn(`No JetBrains product config found for prefixes [${settings.productPrefixes
      .join(', ',)}]; skipping LSP4IJ settings sync.`,);
    return;
  }
  /**
   LSP4IJ persistent-state file names to manage.
   */
  const files = settings.optionsFiles ?? DEFAULT_OPTIONS_FILES;
  /**
   Absolute path to the global language-server settings file.
   */
  const languageSettingsPath = join(
    latest.optionsDirectory,
    files.languageSettings,
  );
  /**
   Absolute path to the user-defined language-server settings file.
   */
  const userDefinedPath = join(
    latest.optionsDirectory,
    files.userDefined,
  );
  /**
   Current contents of both settings files, or absent sentinels.
   */
  const [languageXml, userDefinedXml,] = await Promise.all([
    readTrackedExisting({ filePath: languageSettingsPath, },),
    readTrackedExisting({ filePath: userDefinedPath, },),
  ],);
  if ((languageXml === ABSENT_FILE_CONTENT) || (userDefinedXml === ABSENT_FILE_CONTENT)) {
    log.warn(`LSP4IJ settings files not found in ${latest.productDirectory}; skipping LSP4IJ settings sync.`,);
    return;
  }
  /**
   Server ids ineligible to be the base server (the managed scoped servers).
   */
  const excludedIds = new Set([
    ...(settings.baseServerMatch
      .excludeServerIds
      ?? []),
    ...settings.scopedServers
      .map(function scopedId(scoped,): string {
        return scoped.id;
      },),
  ],);
  /**
   Base user-defined server entry, or absent sentinel.
   */
  const baseEntry = findBaseServerEntry({
    userDefinedXml,
    match: settings.baseServerMatch,
    excludedIds,
  },);
  if (baseEntry === ABSENT_XML_ENTRY) {
    log.warn(`No base LSP4IJ server matched in ${latest.productDirectory}; skipping LSP4IJ settings sync.`,);
    return;
  }
  /**
   Manifest override spread into each write; empty so production callers keep the default manifest.
   */
  const manifestPathOption = settings.manifestPath === undefined
    ? {}
    : { manifestPath: settings.manifestPath, };
  await Promise.all([
    overwrite({
      dest: languageSettingsPath,
      content: updatedLanguageSettingsXml({
        languageXml,
        baseEntry,
        settings,
      },),
      ...manifestPathOption,
    },),
    overwrite({
      dest: userDefinedPath,
      content: updatedUserDefinedXml({
        userDefinedXml,
        baseEntry,
        scopedServers: settings.scopedServers,
      },),
      ...manifestPathOption,
    },),
  ],);
}

//endregion Public task
