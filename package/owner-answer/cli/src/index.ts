/**
 Records the owner's answers in the questions file this repository keeps open.

 One answers file holds the block to insert, the heading it goes before,
 and every part of the questions file it replaces.
 A name that matches nothing stops the run before a byte is written,
 so a questions file is never edited by guesswork.

 @module
 */

export {
  parseAnswers,
  type Answers,
} from './answers.ts';
export {
  EXIT_MISUSE,
  EXIT_OK,
  runCli,
  runRecord,
  type CliStreams,
  type RunCliRequest,
  type RunRecordRequest,
} from './cli-entry.ts';
export {
  parseArguments,
  USAGE,
  type Command,
  type HelpCommand,
  type ParsedArguments,
  type ParsedFlags,
  type RecordCommand,
} from './cli-args.ts';
export {
  AnswersError,
  CliInvocationError,
  OwnerAnswerError,
} from './errors.ts';
export {
  insertBefore,
  recordAnswers,
  removeBullet,
  removeRun,
  removeSection,
  type Insertion,
  type InsertBeforeRequest,
  type RecordRequest,
  type RecordResult,
  type RemoveBulletRequest,
  type RemoveRunRequest,
  type RemoveSectionRequest,
} from './record.ts';
