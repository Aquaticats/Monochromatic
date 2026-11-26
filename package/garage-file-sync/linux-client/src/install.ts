/**
 The install task's entry: `mise run //package/garage-file-sync/linux-client:install` and
 `... :check:installed`.

 `--home` installs into another home folder, which is how the tests run it without
 touching the real `~/.config`.

 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';
import {
  checkInstalled,
  installFiles,
} from './install-files.ts';

/**
 Exit status of a command line the entry cannot use.
 */
const USAGE_EXIT_STATUS = 2;

/**
 Exit status of a check that found a drifted or missing copy.
 */
const DRIFT_EXIT_STATUS = 1;

/**
 The package folder, whose `systemd/` and `config/` folders are the sources.
 */
const PACKAGE_ROOT = join(
  import.meta.dirname,
  '..',
);

/**
 What the command line asked for.
 */
type InstallCommand = {
  readonly check: boolean;
  readonly home: string;
};

/**
 Reads the command line.

 @param argv - arguments after the script name

 @returns whether to check instead of install, and the home folder to use

 @throws Error when an argument is not understood

 @example
 ```ts
 parseCommand(['--check', '--home=/home/example',],); // { check: true, home: '/home/example' }
 ```
 */
function parseCommand(argv: readonly string[],): InstallCommand {
  /**
   Option prefix that names the home folder.
   */
  const homePrefix = '--home=';
  /**
   Parsed command line, filled as arguments are read.
   */
  const command: {
    check: boolean;
    home: string
  } = {
    check: false,
    home: homedir(),
  };
  for (const argument of argv) {
    if (argument === '--check') {
      command.check = true;
      continue;
    }
    if (argument.startsWith(homePrefix,)) {
      /**
       Home folder as written after the prefix.
       */
      const home = argument.slice(homePrefix.length,);
      if (home === '') throw new Error('--home needs a folder');
      command.home = home;
      continue;
    }
    throw new Error(`unknown option "${argument}"`);
  }
  return command;
}

try {
  /**
   What this run does and where.
   */
  const command = parseCommand(process.argv
    .slice(2,),);
  if (command.check) {
    /**
     Installed files that are not a copy of their source.
     */
    const problems = await checkInstalled({
      sourceRoot: PACKAGE_ROOT,
      home: command.home,
    },);
    for (const problem of problems) {
      console.error(problem.problem,);
    }
    if (problems.length > 0) {
      process.exitCode = DRIFT_EXIT_STATUS;
    } else {
      console.log(`every installed copy under ${command.home} matches this package`,);
    }
  } else {
    /**
     Targets written by this install.
     */
    const written = await installFiles({
      sourceRoot: PACKAGE_ROOT,
      home: command.home,
    },);
    for (const target of written) {
      console.log(`installed ${target}`,);
    }
    console.log(
      `copied ${String(written.length,)} files; run systemctl --user daemon-reload before starting the units`,
    );
  }
} catch (error) {
  console.error((String(error,)),);
  console.error(
    `usage: node ${relativeUserPath(join(
      PACKAGE_ROOT,
      'src',
      'install.ts',
    ),)} [--check] [--home=<folder>]`,
  );
  process.exitCode = USAGE_EXIT_STATUS;
}

/**
 Shortens the package's own path to something a reader can type.

 @param path - absolute path inside this checkout

 @returns the path relative to the home folder when it is inside it, else unchanged
 */
function relativeUserPath(path: string,): string {
  /**
   Home folder of the user who is reading the usage line.
   */
  const home = homedir();
  if (path.startsWith(`${home}/`,)) return `~/${path.slice(home.length + 1,)}`;
  return path;
}
