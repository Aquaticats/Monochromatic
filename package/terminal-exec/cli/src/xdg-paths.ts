import { tagged, } from '@monochromatic-dev/module-logger/ts';

/**
 Logger root for terminal-exec after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'terminal-exec', },);
/**
 Builds XDG-compliant search paths for terminal config files and application directories.
 
 Follows the proposed Default Terminal Execution Specification:
 config files are searched with optional desktop-environment-specific prefixes,
 and application directories follow the XDG Base Directory Specification ordering.
 
 @module
 */


/**
 Tagged logger for this module.
 */
const l = tagged({
  tag: 'xdg-paths',
  l: parentLogger,
},);

/**
 Strips one or more trailing `/` characters from `dir`.
 
 Scans left from the end past any `/`, then slices once: a single linear
 pass with no recursion. An all-slash string collapses to empty.
 
 @param dir - directory path candidate
 
 @returns `dir` without any trailing slashes
 
 @example
 ```ts
 stripTrailingSlashes('/usr/share/');   // '/usr/share'
 stripTrailingSlashes('/usr/share///'); // '/usr/share'
 stripTrailingSlashes('/usr/share');    // '/usr/share'
 ```
 */
export function stripTrailingSlashes(dir: string,): string {
  return (function trim(): string {
    /**
     Cut point; walked left past every trailing slash so the slice runs once.
     */
    let end = dir.length;
    while ((end > 0) && (dir.charAt(end - 1,)
      === '/'))
      end -= 1;
    return dir.slice(
      0,
      end,
    );
  })();
}

/**
 Builds ordered list of xdg-terminals.list config file paths to search.
 Desktop-specific variants (e.g. `kde-xdg-terminals.list`) are checked before
 the generic `xdg-terminals.list` for each config directory.
 
 @param desktops - Lowercased desktop names from `$XDG_CURRENT_DESKTOP`.
 
 @returns Config file paths in descending priority order (first match wins).
 
 @example
 ```ts
 const paths = configPaths(['kde'])
 // ~/.config/kde-xdg-terminals.list, ~/.config/xdg-terminals.list, ...
 ```
 */
export function configPaths(
  { desktops, }: { readonly desktops: readonly string[]; },
): readonly string[] {
  /**
   HOME envar fallback for path roots when the variable is unset.
   */
  const home = process.env
    .HOME
    ?? '/tmp';
  /**
   XDG config base; defaults under HOME per spec.
   */
  const configHome = process.env
    .XDG_CONFIG_HOME
    ?? `${home}/.config`;
  /**
   System config search list from XDG_CONFIG_DIRS; defaults to /etc/xdg per spec.
   */
  const configDirs = (process.env
    .XDG_CONFIG_DIRS
    ?? '/etc/xdg').split(':',);
  /**
   System data dirs for the secondary `xdg-terminal-exec/` config lookup.
   */
  const dataDirs = (process.env
    .XDG_DATA_DIRS
    ?? '/usr/local/share:/usr/share').split(
    ':',
    );

  /**
   Mutable accumulator filled by the directory loops below.
   */
  const paths: string[] = [];

  //region Config directories (XDG_CONFIG_HOME + XDG_CONFIG_DIRS)
  for (const dir of [
    configHome,
    ...configDirs,
  ]) {
    for (const desktop of desktops)
      paths.push(`${dir}/${desktop}-xdg-terminals.list`,);
    paths.push(`${dir}/xdg-terminals.list`,);
  }
  //endregion

  //region Data directories (XDG_DATA_DIRS/xdg-terminal-exec/)
  for (const dir of dataDirs) {
    for (const desktop of desktops)
      paths.push(`${dir}/xdg-terminal-exec/${desktop}-xdg-terminals.list`,);
    paths.push(`${dir}/xdg-terminal-exec/xdg-terminals.list`,);
  }
  //endregion

  l.debug(`config paths: ${JSON.stringify(paths,)}`,);
  return paths;
}

/**
 Builds ordered list of XDG application directories.
 Returned in ascending priority order (last directory's entries override earlier ones
 for duplicate entry IDs).
 
 @returns Application directory paths, each with trailing `/`.
 
 @example
 ```ts
 const dirs = applicationDirs()
 // ['/usr/share/applications/', '/usr/local/share/applications/', '~/.local/share/applications/']
 ```
 */
export function applicationDirs(): readonly string[] {
  /**
   HOME envar fallback for the data-home derivation.
   */
  const home = process.env
    .HOME
    ?? '/tmp';
  /**
   XDG_DATA_HOME root; defaults under HOME/.local/share per spec.
   */
  const dataHome = process.env
    .XDG_DATA_HOME
    ?? `${home}/.local/share`;
  /**
   System data dirs; reversed below so the user dir wins on ID conflicts.
   */
  const dataDirs = (process.env
    .XDG_DATA_DIRS
    ?? '/usr/local/share:/usr/share').split(
    ':',
    );

  /**
   Ascending priority: system dirs first, user dir last
   */
  const dirs = [
    ...dataDirs.toReversed()
      .map(function ensureTrailingSlash(dir,) {
        return `${stripTrailingSlashes(dir,)}/applications/`;
      },),
    `${dataHome}/applications/`,
  ];

  l.debug(`application dirs: ${JSON.stringify(dirs,)}`,);
  return dirs;
}

/**
 Extracts lowercased desktop names from `$XDG_CURRENT_DESKTOP`.
 
 @returns Array of desktop name strings, e.g. `['kde']`.
 
 @example
 ```ts
 const desktops = currentDesktops(); // e.g. ['kde']
 ```
 */
export function currentDesktops(): readonly string[] {
  /**
   Empty fallback yields an empty desktops array, which disables desktop-prefixed lookups cleanly.
   */
  const raw = process.env
    .XDG_CURRENT_DESKTOP
    ?? '';
  return raw
    .split(':',)
    .filter(function nonEmpty(s,) {
      return s.length
        > 0;
    },)
    .map(function lower(s,) {
      return s.toLowerCase();
    },);
}
