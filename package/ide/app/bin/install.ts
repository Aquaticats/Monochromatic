#!/usr/bin/env node
// Install the single application executable and its launcher entry for the current user:
//   $HOME/.local/bin/monochromatic-ide
//   ${XDG_DATA_HOME:-$HOME/.local/share}/applications/monochromatic.ide.desktop
// The `install` task runs this after `bundle`. Verify it with a disposable HOME (and XDG_DATA_HOME),
// never against the real home: `HOME=/disposable/home node bin/install.ts`.
import { spawnSync } from 'node:child_process';
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  renameSync,
  rmSync,
  statSync
} from 'node:fs';
import {
  isAbsolute,
  join,
  resolve
} from 'node:path';

const home = process.env
  .HOME
  ?? '';
if (!isAbsolute(home)) throw new Error(`HOME must name an absolute folder to install into; it is ${  JSON.stringify(home)}`);
// The XDG base directory specification ignores a relative XDG_DATA_HOME and falls back to ~/.local/share.
const configuredDataHome = process.env
  .XDG_DATA_HOME
  ?? '';
const dataHome = isAbsolute(configuredDataHome) ? configuredDataHome : join(
      home,
      '.local',
      'share'
    );
const executable = resolve(process.argv[2] ?? 'dist/monochromatic-ide');
const entry = resolve('share/applications/monochromatic.ide.desktop');
if ((!existsSync(executable)) || (!statSync(executable)
  .isFile())) throw new Error(`Missing ${  executable  }; run the bundle task first`);

/**
 Copies a file beside its destination and renames it over the destination, so a running copy keeps its file and a
 failed copy never leaves a half-written executable or entry at the final name.

 @param from - file to install

 @param directory - folder to install into, created when absent

 @param name - file name inside `directory`

 @param mode - permission bits of the installed file

 @returns the path of the installed file

 @throws {Error} when the copy or the rename fails
 */
function place({
  from,
  directory,
  name,
  mode
}: {
  from: string;
  directory: string;
  name: string;
  mode: number;
}): string {
  mkdirSync(
    directory,
    { recursive: true }
  );
  const destination = join(
    directory,
    name
  );
  const partial = join(
    directory,
    `.${  name  }.${  process.pid  }.partial`
  );
  try {
    copyFileSync(
      from,
      partial
    );
    chmodSync(
      partial,
      mode
    );
    renameSync(
      partial,
      destination
    );
  } catch (error) {
    rmSync(
      partial,
      { force: true }
    );
    throw new Error(
      `Cannot install ${  destination  }: ${  error instanceof Error ? error.message : String(error)}`,
      { cause: error }
    );
  }
  console.log(`Installed ${  destination}`);
  return destination;
}

const binDirectory = join(
  home,
  '.local',
  'bin'
);
place({
  from: executable,
  directory: binDirectory,
  name: 'monochromatic-ide',
  mode: 0o755
});
const applications = join(
  dataHome,
  'applications'
);
place({
  from: entry,
  directory: applications,
  name: 'monochromatic.ide.desktop',
  mode: 0o644
});

// GLib-based launchers read a folder's mimeinfo.cache when one exists; refresh it so "Open with" for
// folders (MimeType=inode/directory) sees the new entry. KDE rebuilds its own cache when the folder changes.
const refresh = spawnSync(
  'update-desktop-database',
  [applications],
  { encoding: 'utf8' }
);
if (refresh.error !== undefined && 'code' in refresh.error
  && refresh.error.code
  === 'ENOENT') console.log(`update-desktop-database is not installed; the MIME cache of ${  applications  } was not refreshed`);
else if (refresh.status !== 0) throw new Error(`update-desktop-database ${  applications  } failed: ${  refresh.stderr
  || refresh.error
    ?.message}`);
else console.log(`Refreshed the MIME cache of ${  applications}`);

const onPath = (process.env
  .PATH
  ?? '').split(':')
  .includes(binDirectory);
if (!onPath) console.log(`Note: ${  binDirectory  } is not on PATH in this shell; the launcher entry starts the bare name monochromatic-ide`);
