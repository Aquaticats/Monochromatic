/**
 Decides whether one Git invocation can write to the repository.
 
 Read-only subcommands and read-only argument shapes pass;
 everything else, including unknown subcommands, counts as writing,
 because only a proven read may bypass the repository wrapper.
 
 @module
 */

import {
  GIT_VERB_FIRST_FORMS,
  READ_ONLY_GIT_SUBCOMMAND_FORMS,
  verbFirstReadsOnly,
} from './git-subcommand-read-forms.ts';
import {
  GIT_GLOBAL_OPTIONS_WITH_VALUE,
  READ_ONLY_GIT_SUBCOMMANDS,
} from './git-subcommand-option-sets.ts';

//region Constants

/**
 Sentinel returned when the words after the Git executable name carry no subcommand.
 */
const NO_GIT_SUBCOMMAND: unique symbol = Symbol('words after the program name are options only',);

//endregion Constants

//region Subcommand split

/**
 Subcommand and arguments parsed from the words after the Git executable.
 */
type GitSubcommandCall = {
  /**
   Subcommand name, or {@link NO_GIT_SUBCOMMAND} when only global options are present.
   */
  readonly subcommand: string | typeof NO_GIT_SUBCOMMAND;
  /**
   Subcommand arguments in source order.
   */
  readonly args: readonly string[];
};

/**
 Splits the words after the Git executable into subcommand and arguments.
 
 Global options before the subcommand are dropped with their values.
 
 @param words - words after the Git executable
 
 @returns subcommand call
 
 @example
 ```ts
 splitGitSubcommand(['-C', '/repo', 'status']);
// { subcommand: 'status', args: [] }
 ```
 */
function splitGitSubcommand(words: readonly string[],): GitSubcommandCall {
  for (let index = 0; index < words.length; index += 1) {
    /**
     Word inspected as global option or subcommand.
     */
    const word = words[index];
    if (word === undefined)
      break;
    if (!word.startsWith('-',)) {
      return {
        subcommand: word,
        args: words.slice(index + 1,),
      };
    }
    if (GIT_GLOBAL_OPTIONS_WITH_VALUE.has(word,))
      index += 1;
  }
  return {
    subcommand: NO_GIT_SUBCOMMAND,
    args: [],
  };
}

//endregion Subcommand split

//region Mutation decision

/**
 Reports whether one Git invocation can write to the repository.
 
 @param words - words after the Git executable
 
 @returns whether the invocation can mutate the repository
 
 @example
 ```ts
 mutatesRepository(['commit', '-m', 'x']); // true
 mutatesRepository(['status']); // false
 ```
 */
function mutatesRepository(words: readonly string[],): boolean {
  /**
   Subcommand and its arguments.
   */
  const {
    subcommand,
    args,
  } = splitGitSubcommand(words,);
  if (subcommand === NO_GIT_SUBCOMMAND)
    return false;
  if (READ_ONLY_GIT_SUBCOMMANDS.has(subcommand,))
    return false;

  /**
   Verb-first shape of the subcommand, when it has one.
   */
  const verbFirst = GIT_VERB_FIRST_FORMS.get(subcommand,);
  if (verbFirst !== undefined) {
    return !verbFirstReadsOnly({
      args,
      form: verbFirst,
    },);
  }

  /**
   Read-only test for subcommands that read in some argument shapes.
   */
  const readsOnly = READ_ONLY_GIT_SUBCOMMAND_FORMS.get(subcommand,);
  if (readsOnly !== undefined)
    return !readsOnly(args,);

  return true;
}

//endregion Mutation decision

export {
  mutatesRepository,
  NO_GIT_SUBCOMMAND,
};
