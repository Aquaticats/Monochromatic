/**
 Git subcommand and option words the mutation decision reads.
 
 Words are listed rather than inferred:
 an option Git adds later reads as unknown,
 and an unknown subcommand counts as mutating.
 
 @module
 */

//region Global options

/**
 Git options that consume the next word as their value before the subcommand.
 */
const GIT_GLOBAL_OPTIONS_WITH_VALUE: ReadonlySet<string> = new Set([
  '-C',
  '-c',
  '--attr-source',
  '--config-env',
  '--exec-path',
  '--git-dir',
  '--namespace',
  '--super-prefix',
  '--work-tree',
],);

//endregion Global options

//region Read-only subcommands

/**
 Subcommands that only read the repository.
 
 Everything absent from this set and from the read-only argument shapes
 counts as mutating,
 so only a proven read may bypass the repository wrapper.
 */
const READ_ONLY_GIT_SUBCOMMANDS: ReadonlySet<string> = new Set([
  'annotate',
  'archive',
  'blame',
  'cat-file',
  'check-attr',
  'check-ignore',
  'check-mailmap',
  'check-ref-format',
  'cherry',
  'count-objects',
  'describe',
  'diff',
  'diff-files',
  'diff-index',
  'diff-tree',
  'for-each-ref',
  'format-patch',
  'fsck',
  'get-tar-commit-id',
  'grep',
  'help',
  'log',
  'ls-files',
  'ls-remote',
  'ls-tree',
  'merge-base',
  'name-rev',
  'patch-id',
  'range-diff',
  'rev-list',
  'rev-parse',
  'shortlog',
  'show',
  'show-branch',
  'show-index',
  'show-ref',
  'status',
  'var',
  'verify-commit',
  'verify-pack',
  'verify-tag',
  'version',
  'whatchanged',
],);

//endregion Read-only subcommands

//region Branch options

/**
 Options that consume a value in `git branch` listing forms.
 */
const BRANCH_OPTIONS_WITH_VALUE: ReadonlySet<string> = new Set([
  '--contains',
  '--format',
  '--merged',
  '--no-contains',
  '--no-merged',
  '--points-at',
  '--sort',
],);

/**
 `git branch` options that create, move, rename, or delete a branch.
 */
const BRANCH_WRITE_OPTIONS: ReadonlySet<string> = new Set([
  '-C',
  '-D',
  '-c',
  '-d',
  '-f',
  '-m',
  '-M',
  '-t',
  '-u',
  '--copy',
  '--delete',
  '--edit-description',
  '--force',
  '--move',
  '--no-track',
  '--set-upstream-to',
  '--track',
  '--unset-upstream',
],);

//endregion Branch options

//region Tag options

/**
 Options that consume a value in `git tag` listing forms.
 */
const TAG_OPTIONS_WITH_VALUE: ReadonlySet<string> = new Set([
  '--contains',
  '--format',
  '--merged',
  '--no-contains',
  '--no-merged',
  '--points-at',
  '--sort',
],);

/**
 `git tag` options that list tags.
 */
const TAG_READ_OPTIONS: ReadonlySet<string> = new Set([
  '-l',
  '-n',
  '--column',
  '--contains',
  '--format',
  '--list',
  '--merged',
  '--no-contains',
  '--no-merged',
  '--omit-empty',
  '--points-at',
  '--sort',
],);

/**
 `git tag` options that create, sign, or delete a tag.
 */
const TAG_WRITE_OPTIONS: ReadonlySet<string> = new Set([
  '-F',
  '-a',
  '-d',
  '-e',
  '-f',
  '-m',
  '-s',
  '-u',
  '--annotate',
  '--cleanup',
  '--create-reflog',
  '--delete',
  '--edit',
  '--file',
  '--force',
  '--local-user',
  '--message',
  '--sign',
],);

//endregion Tag options

//region Config options

/**
 Options that consume a value in `git config` invocations.
 */
const CONFIG_OPTIONS_WITH_VALUE: ReadonlySet<string> = new Set([
  '-f',
  '-t',
  '--blob',
  '--comment',
  '--default',
  '--file',
  '--type',
],);

/**
 `git config` options that only print stored values.
 */
const CONFIG_READ_OPTIONS: ReadonlySet<string> = new Set([
  '-l',
  '--get',
  '--get-all',
  '--get-regexp',
  '--get-urlmatch',
  '--list',
],);

/**
 `git config` options that change the configuration files.
 */
const CONFIG_WRITE_OPTIONS: ReadonlySet<string> = new Set([
  '-e',
  '--add',
  '--edit',
  '--remove-section',
  '--rename-section',
  '--replace-all',
  '--set',
  '--unset',
  '--unset-all',
],);

//endregion Config options

//region Reference options

/**
 `git symbolic-ref` options that write the reference.
 */
const SYMBOLIC_REF_WRITE_OPTIONS: ReadonlySet<string> = new Set([
  '-d',
  '-m',
  '--delete',
  '--message',
],);

//endregion Reference options

export {
  BRANCH_OPTIONS_WITH_VALUE,
  BRANCH_WRITE_OPTIONS,
  CONFIG_OPTIONS_WITH_VALUE,
  CONFIG_READ_OPTIONS,
  CONFIG_WRITE_OPTIONS,
  GIT_GLOBAL_OPTIONS_WITH_VALUE,
  READ_ONLY_GIT_SUBCOMMANDS,
  SYMBOLIC_REF_WRITE_OPTIONS,
  TAG_OPTIONS_WITH_VALUE,
  TAG_READ_OPTIONS,
  TAG_WRITE_OPTIONS,
};
