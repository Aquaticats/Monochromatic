/**
 Main terminal resolution algorithm.
 Dispatches to platform-specific resolution:
 
 **Linux/FreeBSD** (XDG):
 1. Explicit entries from `xdg-terminals.list` config files
 2. KDE `kdeglobals` TerminalService fallback (when no explicit entries exist)
 3. All `TerminalEmulator`-category desktop entries as fallback
 
 **Windows**:
 1. Windows Terminal (`wt.exe`)
 2. `cmd.exe`
 
 @module
 */
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { parseConfigFiles, } from './config.ts';
import {
  DESKTOP_ENTRY_UNREADABLE,
  parseDesktopEntry,
} from './desktop-entry.ts';
import {
  kdeTerminalService,
  NO_KDE_TERMINAL,
} from './kde.ts';
import { scanEntries, } from './scan.ts';
import {
  NO_TERMINAL,
  type ValidatedEntry,
  validateEntry,
} from './validate.ts';
import { resolveWindowsTerminal, } from './windows.ts';
import {
  applicationDirs,
  configPaths,
  currentDesktops,
} from './xdg-paths.ts';

/**
 Logger root for terminal-exec after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'terminal-exec', },);

/**
 Tagged logger for this module.
 */
const l = tagged({
  tag: 'resolve',
  l: parentLogger,
},);

/**
 Successful terminal resolution result.
 */
export type ResolvedTerminal = ValidatedEntry & {
  /**
   Desktop entry ID (Linux) or executable name (Windows) that was selected.
   */
  readonly entryId: string;
};

/* oxlint-disable require-await -- resolveTerminal delegates to async resolveXdgTerminal; async needed for uniform Promise return */
/**
 Resolves the preferred terminal emulator for the current platform,
 dispatching to {@link resolveXdgTerminal} or {@link resolveWindowsTerminal}.
 
 @returns Resolved terminal entry, or {@link NO_TERMINAL} if no valid terminal is found.
 
 @example
 ```ts
 const terminal = await resolveTerminal()
 // Linux: terminal.entryId === 'com.mitchellh.ghostty.desktop'
 // Windows: terminal.entryId === 'wt.exe'
 ```
 */
export async function resolveTerminal(): Promise<ResolvedTerminal | typeof NO_TERMINAL> {
  if (process.platform
    === 'win32') {
    l.debug('platform: win32',);
    return resolveWindowsTerminal();
  }

  l.debug(`platform: ${process.platform} (XDG)`,);
  return resolveXdgTerminal();
}
/* oxlint-enable require-await */

/**
 Resolves the terminal emulator using the XDG Desktop Entry Specification.
 Used on Linux, FreeBSD, and other Unix-like systems. Merges
 {@link parseConfigFiles} preferences with a {@link scanEntries} fallback
 scan, computing the explicit candidate order via {@link resolveExplicitIds}
 and validating each candidate through {@link tryEntry}.
 
 @returns Resolved terminal entry, or {@link NO_TERMINAL} if no valid terminal is found.
 */
async function resolveXdgTerminal(): Promise<ResolvedTerminal | typeof NO_TERMINAL> {
  /**
   Lowercased XDG_CURRENT_DESKTOP list; needed for ShowIn checks below.
   */
  const desktops = currentDesktops();
  /**
   Ordered config file paths; later parseConfigFiles reads in priority order.
   */
  const configs = configPaths({ desktops, },);
  /**
   Merged config across all files; consumed for entry preferences and execarg defaults.
   */
  const config = await parseConfigFiles({ paths: configs, },);

  /**
   Ascending-priority application directory list.
   */
  const dirs = applicationDirs();
  /**
   Destructure the scan result: registry maps ids to paths; fallbackIds is the priority-ordered scan list.
   */
  const {
    registry,
    fallbackIds,
  } = await scanEntries({ dirs, },);

  //region Build candidate list: explicit entries, then KDE fallback, then fallback scan
  /**
   Config preferences, or KDE TerminalService when config has no entries.
   */
  const explicitIds = await resolveExplicitIds({ configEntryIds: config.entryIds, },);

  /**
   Fallback IDs with exclusions applied.
   */
  const filteredFallbackIds = fallbackIds.filter(function notExcluded(id,) {
    return !config.excludedIds
      .has(id,);
  },);
  //endregion

  //region Try explicit entries first (bypass OnlyShowIn/NotShowIn)
  for (const entryId of explicitIds) {
    /* oxlint-disable no-await-in-loop -- sequential: first valid entry wins */
    /**
     Per-entry validation attempt; first non-null wins.
     */
    const result = await tryEntry({
      entryId,
      registry,
      desktops,
      isFallback: false,
      config,
    },);
    /* oxlint-enable no-await-in-loop */
    if (result !== NO_TERMINAL) {
      return {
        ...result,
        entryId,
      };
    }
  }
  //endregion

  //region Try fallback entries
  for (const entryId of filteredFallbackIds) {
    /* oxlint-disable no-await-in-loop -- sequential: first valid entry wins */
    /**
     Per-entry validation attempt against the fallback list; first non-null wins.
     */
    const result = await tryEntry({
      entryId,
      registry,
      desktops,
      isFallback: true,
      config,
    },);
    /* oxlint-enable no-await-in-loop */
    if (result !== NO_TERMINAL) {
      return {
        ...result,
        entryId,
      };
    }
  }
  //endregion

  l.debug('no valid terminal emulator found',);
  return NO_TERMINAL;
}

/**
 Resolves the explicit entry id list: config preferences when present, otherwise the KDE TerminalService fallback.
 
 @param configEntryIds - Explicit entries from parsed xdg-terminals.list config.
 
 @returns Ordered list of entry ids to try as explicit candidates; empty when neither source provided one.
 
 @example
 ```ts
 const ids = await resolveExplicitIds({ configEntryIds: [] })
 // ['com.mitchellh.ghostty.desktop'] when kdeglobals has TerminalService set
 ```
 */
async function resolveExplicitIds(
  { configEntryIds, }: { readonly configEntryIds: readonly string[]; },
): Promise<readonly string[]> {
  if (configEntryIds.length
    > 0)
    return configEntryIds;

  l.debug('no explicit entries in config, checking kdeglobals',);
  /**
   KDE fallback used only when explicit entries are empty.
   */
  const kdeId = await kdeTerminalService();
  if (kdeId === NO_KDE_TERMINAL)
    return [];

  l.debug(`using KDE TerminalService '${kdeId}' as explicit entry`,);
  return [kdeId,];
}

/**
 Attempts to resolve a single entry ID into a validated terminal.
 
 @param entryId - Desktop entry ID to try.
 
 @param registry - Entry registry from scanning.
 
 @param desktops - Current desktop names.
 
 @param isFallback - Whether this is a fallback entry.
 
 @param config - Parsed config for execarg defaults.
 
 @returns Validated entry or {@link NO_TERMINAL}.
 */
async function tryEntry({
  entryId,
  registry,
  desktops,
  isFallback,
  config,
}: {
  readonly entryId: string;
  readonly registry: ReadonlyMap<string, {
    readonly id: string;
    readonly path: string;
  }>;
  readonly desktops: readonly string[];
  readonly isFallback: boolean;
  readonly config: { readonly execArgDefaults: ReadonlyMap<string, string>; };
},): Promise<ValidatedEntry | typeof NO_TERMINAL> {
  /**
   Registry lookup; missing id means we cannot resolve this preference.
   */
  const reg = registry.get(entryId,);
  if (reg === undefined) {
    l.debug(`entry '${entryId}' not found in registry`,);
    return NO_TERMINAL;
  }

  /**
   Parsed desktop entry contents; DESKTOP_ENTRY_UNREADABLE on read failure.
   */
  const entry = await parseDesktopEntry({ path: reg.path, },);
  if (entry === DESKTOP_ENTRY_UNREADABLE)
    return NO_TERMINAL;

  return validateEntry({
    entry,
    entryId,
    desktops,
    isFallback,
    execArgDefault: config.execArgDefaults
      .get(entryId,)
      ?? '',
  },);
}
