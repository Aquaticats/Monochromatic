#!/usr/bin/env node
/**
 Music player that finds files by case-insensitive name via ripgrep and plays them with ffplay.
 
 Builds a glob pattern from positional arguments where each word's first letter
 becomes a case-insensitive bracket expression, then searches `$XDG_MUSIC_DIR`
 (or the output of `xdg-user-dir MUSIC`) for matching files.
 
 @example
 ```sh
 rgffplay sweet devil
 # runs: rg --files -g '*[Ss]weet*[Dd]evil*' ~/Music --null
 # then: ffplay -loop 0 -nodisp <matched files>
 ```
 
 @module
 */

import spawn from 'nano-spawn';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

/**
 Logger root for rgffplay after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'rgffplay', },);



//region Glob pattern construction: case-insensitive first letter per word

/**
 Converts a word's first character to a case-insensitive bracket expression.
 
 @param word - Single word from the search query.
 
 @returns Glob fragment with bracketed first letter, e.g. `sweet` becomes `[Ss]weet`.
 
 @example
 ```ts
 bracketFirst('devil') // '[Dd]evil'
 ```
 */
function bracketFirst(word: string,): string {
  /**
   First character isolated so it can be wrapped in a case-insensitive bracket; empty string when word is empty.
   */
  const [first,] = word;
  if (first === undefined)
    return '';
  return `[${first.toUpperCase()}${first.toLowerCase()}]${word.slice(1,)}`;
}

/**
 Builds a ripgrep glob pattern from name words.
 
 Joins words bracketed via {@link bracketFirst} with `*` wildcards and
 wraps in leading/trailing `*`.
 
 @param words - Name words from CLI arguments.
 
 @returns Glob pattern, e.g. `['sweet', 'devil']` becomes `*[Ss]weet*[Dd]evil*`.
 
 @example
 ```ts
 buildGlob(['sweet', 'devil']) // '*[Ss]weet*[Dd]evil*'
 ```
 */
function buildGlob(words: readonly string[],): string {
  return `*${
    words
      .map(function toBracket(w,) {
        return bracketFirst(w,);
      },)
      .join('*',)
  }*`;
}

//endregion Glob pattern construction

//region Music directory resolution: XDG_MUSIC_DIR or xdg-user-dir fallback

/**
 Tagged logger for music directory resolution.
 */
const rlMusicDir = tagged({
  tag: resolveMusicDir.name,
  l,
},);

/**
 Resolves the user's music directory.
 
 Checks `XDG_MUSIC_DIR` environment variable first,
 falls back to invoking `xdg-user-dir MUSIC`.
 
 @returns Absolute path to the music directory.
 
 @throws When `xdg-user-dir` is not available and `XDG_MUSIC_DIR` is unset.
 */
async function resolveMusicDir(): Promise<string> {
  /**
   User-set XDG override; preferred path when present so callers can point at any directory.
   */
  const envDir = process.env
    .XDG_MUSIC_DIR;
  if ((envDir !== undefined) && (envDir.length
    > 0)) {
    rlMusicDir.info(`using XDG_MUSIC_DIR="${envDir}"`,);
    return envDir;
  }

  rlMusicDir.info('XDG_MUSIC_DIR unset, falling back to xdg-user-dir',);
  /**
   Raw stdout from `xdg-user-dir MUSIC`; trimmed below since the helper appends a newline.
   */
  const { stdout, } = await spawn(
    'xdg-user-dir',
    ['MUSIC',],
  );
  /**
   Trimmed music directory path; stripping the trailing newline so the value is a valid filesystem path.
   */
  const dir = stdout.trim();
  rlMusicDir.info(`xdg-user-dir resolved to "${dir}"`,);
  return dir;
}

//endregion Music directory resolution

//region File search; rg --files with glob pattern

/**
 Tagged logger for the file search phase.
 */
const rlSearch = tagged({
  tag: findFiles.name,
  l,
},);

/**
 Searches for music files matching the glob pattern.
 
 Uses `rg --files -g <glob> <dir> --null` for null-separated output,
 then splits on null bytes to produce the file list.
 
 @param glob - Ripgrep glob pattern.
 
 @param musicDir - Absolute path to search in.
 
 @returns Single-element array containing the matched file path.
 
 @throws When no files match the glob pattern.
 
 @throws When more than one file matches (ambiguous query).
 */
async function findFiles({
  glob,
  musicDir,
}: {
  readonly glob: string;
  readonly musicDir: string;
},): Promise<readonly string[]> {
  rlSearch.info(`searching "${musicDir}" with glob "${glob}"`,);

  /**
   Raw stdout from rg, null-byte-separated.
   */
  const rgOutput = await spawn(
    'rg',
    [
      '--files',
      '-g',
      glob,
      musicDir,
      '--null',
    ],
  )
    .then(
    function extractStdout({ stdout, }: Readonly<{ stdout: string; }>,): string {
      return stdout;
    },
    function handleRgError(err: unknown,) {
        // rg exits 1 when no files match the glob
      if ((err !== null)
        && (err !== undefined)
        && ((typeof err) === 'object')
        && ('exitCode' in err))
        {
          /**
           Process exit code pulled off the spawn error so the no-match case (1) can be rethrown with a clearer message.
           */
        const { exitCode, } = err;
        if (exitCode === 1) {
          throw new Error(
            `No files matching glob "${glob}" in "${musicDir}"`,
            { cause: err, },
          );
        }
      }
      throw err;
    },
    );

  /**
   Matched file paths split from the null-separated rg output; empty fragments dropped so the count reflects real matches.
   */
  const files = rgOutput.split('\0',)
    .filter(function nonEmpty(f,) {
      return f.length
        > 0;
    },);

  if (files.length
    === 0)
    throw new Error(`No files matching glob "${glob}" in "${musicDir}"`,);

  if (files.length
    > 1) {
    throw new Error(
      `Ambiguous match: ${
        String(files.length,)
      } files found for glob "${glob}" in "${musicDir}":\n${files.join('\n',)}`,
    );
  }

  rlSearch.info(`matched: ${files[0]}`,);

  return files;
}

//endregion File search

//region Playback: ffplay with matched files

/**
 Tagged logger for the playback phase.
 */
const rlPlay = tagged({
  tag: 'playback',
  l,
},);

//endregion Playback

//region Main execution: parse args, find files, play

/**
 Tagged logger for the main execution flow.
 */
const rl = tagged({
  tag: 'main',
  l,
},);

/**
 Positional arguments forming the search query.
 */
const args = process.argv
  .slice(2,);

if (args.length
  === 0) {
  console.error('Usage: rgffplay <name...>',);
  console.error('Example: rgffplay sweet devil',);
  throw new Error('No arguments provided',);
}

rl.info(`query: "${args.join(' ',)}"`,);

/**
 Ripgrep glob pattern built from the query words via {@link buildGlob}.
 */
const glob = buildGlob(args,);
rl.info(`glob: "${glob}"`,);

/**
 Resolved absolute path to the music directory, via {@link resolveMusicDir}.
 */
const musicDir = await resolveMusicDir();

/**
 Matched music file paths from ripgrep, via {@link findFiles}.
 */
const files = await findFiles({
  glob,
  musicDir,
},);

rlPlay.info(`playing ${String(files.length,)} file(s) with ffplay -loop 0 -nodisp`,);

await spawn(
  'ffplay',
  [
    '-loop',
    '0',
    '-nodisp',
    ...files,
  ],
  {
    stdout: 'inherit',
    stderr: 'inherit',
    stdin: 'inherit',
  },
);

//endregion Main execution
