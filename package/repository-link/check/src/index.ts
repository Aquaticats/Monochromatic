/**
 Library surface of the repository link check: the check, the identity file, the writer, `openRepositoryRoot()`, and the
 real probe. The bins are `check.ts`, `run.ts` and `record.ts`.

 @module
 */

export { checkRepositoryLink, } from './check-link.ts';
export {
  EXIT_GIT_FATAL,
  EXIT_TEMPFAIL,
  ROOT_COMMIT_MARKER,
} from './constants.ts';
export {
  identityPathFor,
  readIdentity,
  writeIdentity,
} from './identity-file.ts';
export type {
  AnswerKind,
  EntryInfo,
  LinkAnswer,
  LinkIdentity,
  LinkPhase,
  Probe,
} from './link-types.ts';
export {
  mountHolding,
  parseMountInfo,
} from './mountinfo.ts';
export {
  openRepositoryRoot,
  RepositoryLinkNotReadyError,
} from './open-repository-root.ts';
export { realProbe, } from './real-probe.ts';
export {
  recordIdentity,
  RecordIdentityError,
} from './record-identity.ts';
export { resolveCommand, } from './resolve-command.ts';
export { parseBinOptions, } from './run-options.ts';
