/**
 Guards that the root `.gitignore` still re-includes the two files the KWin
 script cannot ship without.

 The repository's blanket `*.js` and `metadata.json` patterns match both, so
 only a negating line keeps them addable. An already-tracked file that Git stops
 seeing is still committed, and the loss appears only once the file is deleted
 and recreated, so nothing fails visibly without this guard.

 Git is asked rather than the pattern text parsed: a moved package or an edited
 pattern fails here instead of in a fresh clone.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { execFileSync, } from 'node:child_process';
import { existsSync, } from 'node:fs';
import {
  dirname,
  join,
  relative,
} from 'node:path';

/** Raised when the walk from this file reaches the filesystem root without a `.git` entry. */
class RepositoryRootNotFoundError extends Error {
  /**
   @param startDirectory - directory the search started in
   */
  constructor(startDirectory: string,) {
    super(`no .git entry in any directory above ${startDirectory}`,);
    this.name = 'RepositoryRootNotFoundError';
  }
}

/**
 Finds the repository root by walking up to the nearest entry named `.git`.

 A plain filesystem walk, not `git rev-parse`: this repository's Git policy
 refuses a subdirectory as the working directory, so a Git call to find the root
 would need the root it is looking for.

 @param startDirectory - directory to search from, usually this test file's own

 @returns absolute path of the directory holding `.git`, a file in a worktree

 @throws {@link RepositoryRootNotFoundError} when no ancestor holds `.git`
 */
function repositoryRootFrom(startDirectory: string,): string {
  let current = startDirectory;
  while (true) {
    if (existsSync(join(current, '.git',),)) { return current }
    const parent = dirname(current,);
    if (parent === current) { throw new RepositoryRootNotFoundError(startDirectory,) }
    current = parent;
  }
}

/** Repository root, the directory the `.gitignore` under guard sits in. */
const repositoryRoot = repositoryRootFrom(import.meta.dirname,);

/** The KWin script package this test's package ships beside its daemon. */
const kwinScriptDirectory = join(import.meta.dirname, '..', 'kwin-script',);

/**
 The KWin script files that the blanket `*.js` and `metadata.json` patterns
 would hide without a negating line.

 `contents/code/main.js` is the `X-Plasma-MainScript` that `metadata.json` names,
 so a clone missing either one installs a script Plasma cannot run.
 */
const filesTheBlanketPatternsMatch = ['contents/code/main.js', 'metadata.json',] as const;

/**
 Reports the last `.gitignore` pattern Git applies to a path, tracked or not.

 `--no-index` makes Git report the pattern even though the path is tracked,
 which is exactly the state a broken negation hides.

 @param repositoryPath - path relative to the repository root

 @returns the pattern Git reports for the path

 @example
 ```ts
 expect(reportedPattern('package/kwin/key-helper/kwin-script/metadata.json')).toBe('!/package/kwin/key-helper/kwin-script/metadata.json');
 ```
 */
function reportedPattern(repositoryPath: string,): string {
  const [verdict,] = execFileSync(
    'git',
    ['check-ignore', '--no-index', '--verbose', '--', repositoryPath,],
    { cwd: repositoryRoot, encoding: 'utf8', },
  ).split('\t',);
  // `<source>:<line>:<pattern>`; a pattern may itself hold a colon, so only the
  // first two separators are consumed.
  const [, , ...patternParts] = (verdict ?? '').split(':',);
  return patternParts.join(':',);
}

await describe({
  name: 'the root .gitignore keeps the KWin script addable',
  children: filesTheBlanketPatternsMatch.map((file,) => it({
    name: `re-includes ${file}`,
    fn: async () => {
      const repositoryPath = relative(repositoryRoot, join(kwinScriptDirectory, file,),);
      expect(
        () => execFileSync(
          'git',
          ['ls-files', '--error-unmatch', '--', repositoryPath,],
          { cwd: repositoryRoot, stdio: 'ignore', },
        ),
        `${repositoryPath} is not tracked source any more`,
      ).not.toThrow();
      const pattern = reportedPattern(repositoryPath,);
      // The negation has to name this repository-relative path, so moving the
      // package fails here instead of silently re-ignoring the files.
      expect(pattern,).toBe(`!/${repositoryPath}`,);
    },
  },)),
},);
