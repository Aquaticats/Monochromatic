/**
 Library surface of the machine runner.

 The program itself lives in `./cli.ts`. What a caller can be handed is the
 table of tests this package runs, the marker a prepared guest holds, and the
 two renderers that decide what a guest is asked to run.
 */

export { machineName, } from './machine.ts';
export {
  parseArguments,
  renderPhaseCommand,
} from './run-request.ts';
export {
  quoteForPosixShell,
  renderCommandLine,
} from './shell-command.ts';
export {
  GUEST_CRATE_DIRECTORY,
  GUEST_MARKER_PATH,
  TEST_DEFINITIONS,
} from './test-definitions.ts';
export type {
  TestDefinition,
  TestPhase,
} from './test-definitions.ts';
