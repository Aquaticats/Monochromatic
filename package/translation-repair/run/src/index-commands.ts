//region Command line exports
// The command lines and their refusals, the corpus pin setting and the entry filter.
// Re-exported by `index.ts`, which is the package's one public entry.

export { reportingRefusals, } from './corpus-run/cli-refusal.ts';
export {
  askedAmong,
  idListFlag,
  usdFlag,
  wholeNumberFlag,
  writtenOr,
} from './corpus-run/command-flags.ts';
export {
  type CommandLineFor,
  type CommandLineSpec,
  type FlagValue,
  type PositionalSpec,
  type ReadsFlag,
  type ReadsSwitch,
} from './corpus-run/command-line-types.ts';
export { readCommandLine, } from './corpus-run/command-line.ts';
export {
  COMMAND_OWNERS,
  taskPathOf,
} from './corpus-run/command-owners.ts';
export {
  COMMAND_LINES,
  type CommandLineOf,
  type CommandName,
} from './corpus-run/command-lines.ts';
export {
  CORPUS_CLONE_DIR_VAR,
  CORPUS_COMMIT_VAR,
  corpusPinOverrideNote,
  type CorpusPinSetting,
  type CorpusPinSource,
  readCorpusPinSetting,
} from './corpus-run/corpus-pin-override.ts';
export { readOnlyIds, } from './corpus-run/entry-filter.ts';
export {
  registerRunEndReport,
  type RunEndReport,
  runEndReports,
} from './corpus-run/run-end-report.ts';

//endregion Command line exports
