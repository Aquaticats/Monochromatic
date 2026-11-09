/**
 Programs whose own options name a command they run.
 
 `find` runs a command listed among its own arguments,
 between one of its command options and that option's terminator,
 so `find . -exec /usr/bin/git commit {} \;` runs Git and its arguments
 rather than passing them to `find` alone.
 
 A program here is not a wrapper: its own name is still the program the shell runs,
 and the commands it was given are read on their own.
 
 @module
 */

//region Types

/**
 Program whose options name a command it runs.
 */
type NestedCommandProgram = {
  /**
   Executable names that introduce the program.
   */
  readonly names: ReadonlySet<string>;
  /**
   Options after which the words name a command and its arguments.
   */
  readonly commandOptions: ReadonlySet<string>;
  /**
   Options whose command may end with the batching word,
   as `find -exec COMMAND {} +` batches matches into one run.
   */
  readonly batchingCommandOptions: ReadonlySet<string>;
  /**
   Words that always end the command named by such an option.
   */
  readonly terminators: ReadonlySet<string>;
  /**
   Word that ends a batching command only when it follows the placeholder,
   since an ordinary argument may spell it too.
   */
  readonly batchingTerminator: string;
  /**
   Placeholder a batching command names last, which the batching word follows.
   */
  readonly batchingPlaceholder: string;
};

//endregion Types

//region Table

/**
 Programs that agent command lines use to run a command named in their arguments.
 */
const NESTED_COMMAND_PROGRAMS: readonly NestedCommandProgram[] = [
  {
    names: new Set(['find',],),
    commandOptions: new Set([
      '-exec',
      '-execdir',
      '-ok',
      '-okdir',
    ],),
    batchingCommandOptions: new Set([
      '-exec',
      '-execdir',
    ],),
    terminators: new Set([';',],),
    batchingTerminator: '+',
    batchingPlaceholder: '{}',
  },
];

//endregion Table

export type { NestedCommandProgram, };
export { NESTED_COMMAND_PROGRAMS, };
