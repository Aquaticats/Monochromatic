/**
 Installing the KWin script package and the daemon's autostart entry.

 Two of these steps cross a syntax boundary, so they live in code with tests
 rather than in the inline task text they came from: the autostart entry is read
 once by the desktop-entry spec and its `Exec` value again by `/bin/sh`, and the
 path interpolated into it is the checkout's own path. A space in that path used
 to split the exec words and a quote used to end the shell string early, and the
 failure appeared only at login, as an autostart entry that silently did nothing.

 Nothing here reads `/dev/input`, starts the daemon, or writes outside the home
 directory the caller names, so a test can run every step against a disposable
 one.

 @module
 */

import {
  lstat,
  mkdir,
  readlink,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { shellQuote, } from '@monochromatic-dev/shell-argument-core/ts';

import { InstallError, } from './errors.ts';

//region Logging

/**
 Logger for this module, re-tagged with the enclosing function name at each
 site.
 */
const installLogger = tagged({ tag: 'kwin-key-helper-install', },);

//endregion Logging

//region Constants

/**
 Filename of the autostart entry, which desktop environments read at login.
 */
const AUTOSTART_FILE_NAME = 'key-helper-service.desktop';

/**
 Directory, relative to the home directory, autostart entries live in.
 */
const AUTOSTART_DIRECTORY = '.config/autostart';

/**
 Directory, relative to the home directory, KWin finds script packages in.
 */
const KWIN_SCRIPTS_DIRECTORY = '.local/share/kwin/scripts';

/**
 Name of the symlink the KWin script package is installed as.
 */
const KWIN_SCRIPT_LINK_NAME = 'key-helper-kwin';

/**
 Source directory of the KWin script package, relative to the package.
 */
const KWIN_SCRIPT_SOURCE_DIRECTORY = 'kwin-script';

/**
 Filename of the built daemon binary, relative to the package.
 */
const DAEMON_BINARY_NAME = 'key-helper-service';

/**
 `systemd-cat` tag the daemon's log lines carry in the journal.
 */
const SYSTEMD_CAT_TAG = 'key-helper';

/**
 `systemd-cat` priority the daemon's log lines carry in the journal.
 */
const SYSTEMD_CAT_PRIORITY = 'debug';

/**
 File error code naming an entry that does not exist.
 */
const MISSING_FILE_CODE = 'ENOENT';

//endregion Constants

//region Quoting

/**
 Whether a failed read or stat means the entry does not exist.

 Only `ENOENT` counts as absent: a permission failure, a path component that is
 not a directory, or a symlink loop are all faults the operator has to see,
 because the install cannot fix them by recreating the entry.

 @param error - Value a `node:fs/promises` call rejected with
 
 @returns True only for a missing entry
 
 @example
 ```ts
 isMissingEntry(Object.assign(new Error('absent'), { code: 'ENOENT' }));
 ```
 */
function isMissingEntry(error: unknown): boolean {
  return Error.isError(error)
    && ('code' in error)
    && (error.code === MISSING_FILE_CODE);
}

/**
 Encode one value as a desktop-entry `Exec` argument.

 The Desktop Entry Specification reads an `Exec` value into an argument vector
 before anything runs it, and an argument holding a reserved character must be
 enclosed in double quotes with `"`, `` ` ``, `$` and `\` escaped by a
 backslash. The shell command this file builds contains spaces and pipes, so it
 always takes this form.

 The same value then has its field codes expanded, after the quoting is undone,
 so a percentage character that is not doubled is read as the start of a field
 code: `%f` becomes the file a launcher was given, and a letter the
 specification does not list makes the entry invalid. Doubling it is the only
 literal form the specification has, and it holds inside a quoted argument as
 well.

 @param value - One argument of the entry's command line
 
 @returns The argument quoted and field-code-escaped for the desktop-entry grammar
 
 @example
 ```ts
 desktopEntryQuote({ value: 'exec /bin/true 2>&1 | systemd-cat' });
 // => '"exec /bin/true 2>&1 | systemd-cat"'
 ```
 */
export function desktopEntryQuote({ value }: { readonly value: string; }): string {
  /**
   Value with every character the desktop-entry grammar reserves escaped.
   */
  const escaped = value
    .replaceAll(
    '\\',
    String.raw`\\`,
    )
    .replaceAll(
    '`',
    '\\`',
    )
    .replaceAll(
    '$',
    String.raw`\$`,
    )
    .replaceAll(
    '"',
    String.raw`\"`,
    )
    // A literal percentage character is written doubled, or the expansion that
    // follows the quoting rule reads it as a field code and runs another path.
    .replaceAll(
    '%',
    '%%',
    );
  return `"${escaped}"`;
}

//endregion Quoting

//region Autostart entry

/**
 Build the autostart entry that runs the daemon and sends its output to the
 journal.

 The entry's `Exec` value is `/bin/sh -c` with one argument: a script that
 `exec`s the binary, so the shell is replaced by the daemon rather than wrapping
 it. Both the binary path and the whole script are encoded for the grammar they
 cross, at the point they are interpolated, which is what makes a checkout path
 containing a space, a quote or a percentage character work.

 @param binPath - Absolute path of the built daemon binary
 
 @returns The desktop file's text, newline-terminated
 
 @example
 ```ts
 await writeFile(entryPath, buildAutostartDesktopEntry({ binPath }));
 ```
 */
export function buildAutostartDesktopEntry({ binPath }: {
  readonly binPath: string;
}): string {
  /**
   Shell script the entry runs: replace the shell with the daemon, and send
   everything the daemon writes to the journal under one tag.
   */
  const script = `exec ${shellQuote(binPath)} 2>&1 | systemd-cat -t ${SYSTEMD_CAT_TAG} -p ${SYSTEMD_CAT_PRIORITY}`;
  return [
    '[Desktop Entry]',
    'Type=Application',
    'Name=Key Helper D-Bus Service',
    'Comment=Bridges KWin script commands to Neovim RPC and ydotool',
    `Exec=/bin/sh -c ${desktopEntryQuote({ value: script, })}`,
    'Terminal=false',
    'X-KDE-autostart-phase=2',
    '',
  ].join('\n',);
}

/**
 Write the autostart entry into a home directory.
 
 @param homeDirectory - Home directory whose autostart folder receives the entry
 
 @param binPath - Absolute path of the built daemon binary
 
 @example
 ```ts
 await writeAutostartEntry({ homeDirectory, binPath });
 ```
 */
export async function writeAutostartEntry({
  homeDirectory,
  binPath,
}: {
  readonly homeDirectory: string;
  readonly binPath: string;
}): Promise<void> {
  /**
   Autostart folder the entry is written into.
   */
  const directory = join(
    homeDirectory,
    AUTOSTART_DIRECTORY,
  );
  await mkdir(
    directory,
    { recursive: true, },
  );
  /**
   Path of the entry itself.
   */
  const entryPath = join(
    directory,
    AUTOSTART_FILE_NAME,
  );
  await writeFile(
    entryPath,
    buildAutostartDesktopEntry({ binPath, }),
    'utf8',
  );
  tagged({
    tag: writeAutostartEntry.name,
    l: installLogger,
  },)
    .info(`wrote ${entryPath}`,);
}

//endregion Autostart entry

//region KWin script link

/**
 Point the KWin script package's symlink at this checkout, replacing whatever
 was there.

 The link is the install's idempotent step: an entry that is already the right
 symlink is left alone, anything else at that path is replaced, and a missing
 entry is simply created. A failure to inspect the path for any reason other
 than absence is reported with the caught value, because recreating the entry
 over a permission problem or a broken path would hide the cause.
 
 @param linkPath - Path the symlink must occupy
 
 @param kwinTarget - Absolute path of the KWin script package in this checkout
 
 @throws {@link InstallError} when the path cannot be inspected for a reason
 
 @example
 ```ts
 await reconcileKwinScriptLink({ linkPath, kwinTarget });
 ```
 */
export async function reconcileKwinScriptLink({
  linkPath,
  kwinTarget,
}: {
  readonly linkPath: string;
  readonly kwinTarget: string;
}): Promise<void> {
  try {
    /**
     Entry already at the link path.
     */
    const existing = await lstat(linkPath);
    if (existing.isSymbolicLink() && ((await readlink(linkPath)) === kwinTarget)) {
      tagged({
        tag: reconcileKwinScriptLink.name,
        l: installLogger,
      },)
        .info(`link already correct: ${linkPath}`,);
      return;
    }
    await rm(
      linkPath,
      {
        recursive: true,
        force: true,
      },
    );
  } catch (error) {
    if (!isMissingEntry(error)) {
      throw new InstallError(`cannot inspect ${linkPath}: ${caughtValueText(error,)}`,);
    }
  }
  await symlink(
    kwinTarget,
    linkPath,
  );
  tagged({
    tag: reconcileKwinScriptLink.name,
    l: installLogger,
  },)
    .info(`linked ${linkPath} -> ${kwinTarget}`,);
}

//endregion KWin script link

//region Install

/**
 Install everything the daemon needs to start at login: the KWin script
 symlink, the autostart entry, and the KWin script's enablement.

 @param homeDirectory - Home directory the link and entry are installed into
 
 @param packageDirectory - This package's directory, whose built binary and
 script package are linked
 
 @param enableKwinScript - Enables the script in the desktop environment's
 configuration, which the task does with `kwriteconfig6`
 
 @example
 ```ts
 await installKeyHelper({ homeDirectory, packageDirectory, enableKwinScript });
 ```
 */
export async function installKeyHelper({
  homeDirectory,
  packageDirectory,
  enableKwinScript,
}: {
  readonly homeDirectory: string;
  readonly packageDirectory: string;
  readonly enableKwinScript: () => void;
}): Promise<void> {
  /**
   Folder KWin finds script packages in.
   */
  const scriptsDirectory = join(
    homeDirectory,
    KWIN_SCRIPTS_DIRECTORY,
  );
  await mkdir(
    scriptsDirectory,
    { recursive: true, },
  );
  await reconcileKwinScriptLink({
    linkPath: join(
      scriptsDirectory,
      KWIN_SCRIPT_LINK_NAME,
    ),
    kwinTarget: resolve(
      packageDirectory,
      KWIN_SCRIPT_SOURCE_DIRECTORY,
    ),
  });
  await writeAutostartEntry({
    homeDirectory,
    binPath: resolve(
      packageDirectory,
      DAEMON_BINARY_NAME,
    ),
  });
  enableKwinScript();
  tagged({
    tag: installKeyHelper.name,
    l: installLogger,
  },)
    .info('enabled the KWin script in kwinrc',);
}

//endregion Install
