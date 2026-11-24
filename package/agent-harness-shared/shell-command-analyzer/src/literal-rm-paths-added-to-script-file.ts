/**
 Reports the `rm` violations one file change would add to a shell script.
 
 Only a violation the file does not already hold is refused,
 so a change that repairs an old script is never blocked by the violation it
 removes.
 Two changes cannot split one violation between them,
 because the whole file the call would leave on disk is the text judged.
 
 @module
 */

import { homedir, } from 'node:os';

import { findLiteralRmPathsInScriptFile, } from './literal-rm-paths.ts';
import type { LiteralRmPathFinding, } from './literal-rm-path-finding.ts';
import { isShellScriptFile, } from './shell-script-file.ts';

//region Constants

/**
 Separator between the identity parts of one finding, which no command text carries.
 */
const FINDING_IDENTITY_SEPARATOR = '\u0000';

//endregion Constants

//region Comparison

/**
 Reads the identity of one finding, which ignores where in the file it sits.
 
 A line number moves when an unrelated edit adds a line above it,
 so the command and the offending word are what identify a violation.
 
 @param finding - finding to identify
 
 @returns identity two findings share when they are the same violation
 
 @example
 ```ts
 findingIdentity({ command: 'rm -rf "$d"', word: '"$d"', }); // 'rm -rf "$d"\u0000"$d"'
 ```
 */
function findingIdentity(finding: LiteralRmPathFinding,): string {
  return [
    finding.command,
    finding.word,
    finding.check,
  ]
    .join(FINDING_IDENTITY_SEPARATOR,);
}

/**
 Lists the findings one text holds that another text does not.
 
 A violation held twice stays new when it is held three times,
 because the comparison counts occurrences rather than shapes alone.
 
 @param before - findings the file holds now
 
 @param after - findings the file would hold after the change
 
 @returns findings of `after` left over once `before` is accounted for
 
 @example
 ```ts
 findingsNotPresentBefore({ before: [], after: [{ command: 'rm -rf "$d"', word: '"$d"', }] });
 ```
 */
function findingsNotPresentBefore({
  before,
  after,
}: {
  readonly before: readonly LiteralRmPathFinding[];
  readonly after: readonly LiteralRmPathFinding[];
},): readonly LiteralRmPathFinding[] {
  /**
   Occurrences of each violation in the earlier text, counted down as they are matched.
   */
  const remaining = new Map<string, number>();
  for (const finding of before) {
    /**
     Identity of the earlier finding.
     */
    const identity = findingIdentity(finding,);
    remaining.set(
      identity,
      (remaining.get(identity,) ?? 0) + 1,
    );
  }
  return after
    .filter(function isNewFinding(finding,): boolean {
      /**
       Identity of the later finding.
       */
      const identity = findingIdentity(finding,);
      /**
       Occurrences of that identity still unaccounted for.
       */
      const matched = remaining.get(identity,) ?? 0;
      if (matched === 0)
        return true;
      remaining.set(
        identity,
        matched - 1,
      );
      return false;
    },);
}

//endregion Comparison

//region Public reads

/**
 Reports the deletion violations a change would add to a shell script.
 
 A file is judged only when the change leaves it a shell script or found it one,
 so a change to a file a shell never reads costs nothing.
 
 @param path - file path the change targets
 
 @param before - file text before the change, or empty text for a file the change creates
 
 @param after - file text the change would leave on disk
 
 @param homeFolder - the account's home folder, defaulting to this process's
 
 @returns the violations the change adds, empty when it adds none
 
 @example
 ```ts
 literalRmPathFindingsAddedToScriptFile({
   path: '/repo/clean.sh',
   before: '',
   after: 'rm -rf "$d"\n',
 });
 ```
 */
function literalRmPathFindingsAddedToScriptFile({
  path,
  before,
  after,
  homeFolder = homedir(),
}: {
  readonly path: string;
  readonly before: string;
  readonly after: string;
  readonly homeFolder?: string;
},): readonly LiteralRmPathFinding[] {
  if ((!isShellScriptFile({
    path,
    text: after,
  },))
    && (!isShellScriptFile({
      path,
      text: before,
    },)))
    return [];

  return findingsNotPresentBefore({
    before: findLiteralRmPathsInScriptFile({
      path,
      source: before,
      homeFolder,
    },),
    after: findLiteralRmPathsInScriptFile({
      path,
      source: after,
      homeFolder,
    },),
  },);
}

//endregion Public reads

export {
  findingIdentity,
  findingsNotPresentBefore,
  literalRmPathFindingsAddedToScriptFile,
};
