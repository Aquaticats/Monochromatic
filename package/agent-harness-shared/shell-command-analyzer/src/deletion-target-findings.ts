/**
 Decides which words of one deletion command break the literal-path rule.
 
 Three questions settle every command that deletes:
 does another program supply the paths,
 does the shell compute one,
 and does one collapse to a broader target than it looks like.
 Each answer names itself, so a refusal can say which check fired.
 
 The reader above this module hands it targets already stripped of the
 program's own options, so a deleted path is never an option value.
 
 @module
 */

import type { LiteralRmPathCheck, } from './literal-rm-path-finding.ts';
import {
  collapsedTargetReason,
  NOT_COLLAPSED,
} from './deletion-target-collapse.ts';
import type { DeletionTarget, } from './deletion-programs.ts';

//region Types

/**
 One finding before the reader adds where it was found.
 */
type DeletionFindingDraft = {
  /**
   Word whose spelling or value broke the rule.
   */
  readonly word: string;
  /**
   Which check fired.
   */
  readonly check: LiteralRmPathCheck;
  /**
   Why that check fired, in words a refusal can print.
   */
  readonly detail?: string;
};

//endregion Types

//region Reads

/**
 Lists the words of one deletion command that break the literal-path rule.
 
 @param programWord - word naming the deleting program
 
 @param targets - words the program would delete, options already dropped
 
 @param unwrittenPaths - whether another program supplies those paths
 
 @param homeFolder - the account's home folder, injected so a fixture need not run as that account
 
 @param carriesExpansion - decides whether one target's spelling is computed
 
 @returns one draft per offending word
 
 @example
 ```ts
 deletionFindingDrafts({
   programWord: 'rm',
   targets: [{ value: '$d', sourceText: '"$d"' }],
   unwrittenPaths: false,
   homeFolder: '/home/user',
   carriesExpansion: shellWordCarriesExpansion,
 });
 ```
 */
function deletionFindingDrafts({
  programWord,
  targets,
  unwrittenPaths,
  homeFolder,
  carriesExpansion,
}: {
  readonly programWord: string;
  readonly targets: readonly DeletionTarget[];
  readonly unwrittenPaths: boolean;
  readonly homeFolder: string;
  readonly carriesExpansion: (sourceText: string,) => boolean;
},): readonly DeletionFindingDraft[] {
  if (unwrittenPaths) {
    return [{
      word: programWord,
      check: 'another-program-supplies-the-paths',
    },];
  }
  return targets
    .flatMap(function draftsForTarget(target,): readonly DeletionFindingDraft[] {
      if (carriesExpansion(target.sourceText,)) {
        return [{
          word: target.sourceText,
          check: 'shell-computes-the-path',
        },];
      }
      /**
       Why this target names more than the path it looks like.
       */
      const reason = collapsedTargetReason({
        value: target.value,
        homeFolder,
      },);
      if (reason === NOT_COLLAPSED)
        return [];
      return [{
        word: target.sourceText,
        check: 'target-collapses-to-a-broader-path',
        detail: reason,
      },];
    },);
}

//endregion Reads

export { deletionFindingDrafts, };
export type { DeletionFindingDraft, };
