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
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { shellQuote, } from '@monochromatic-dev/shell-argument-core/ts';

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
 `systemd-cat` tag the daemon's log lines carry in the journal.
 */
const SYSTEMD_CAT_TAG = 'key-helper';

/**
 `systemd-cat` priority the daemon's log lines carry in the journal.
 */
const SYSTEMD_CAT_PRIORITY = 'debug';

//endregion Constants

//region Quoting

/**
 Encode one value as a desktop-entry `Exec` argument.

 The Desktop Entry Specification reads an `Exec` value into an argument vector
 before anything runs it, and an argument holding a reserved character must be
 enclosed in double quotes with `"`, `` ` ``, `$` and `\` escaped by a
 backslash. The shell command this file builds contains spaces and pipes, so it
 always takes this form.

 @param value - One argument of the entry's command line
 
 @returns The argument quoted for the desktop-entry grammar
 
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
 containing a space or a quote work.

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
