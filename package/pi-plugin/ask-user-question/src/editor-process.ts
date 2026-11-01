import { spawn, } from 'node:child_process';
import {
  addAbortListener,
  once,
} from 'node:events';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

//region Logger

/**
 Tagged logger for attached editor process.
 */
const l = tagged({ tag: 'ask-user-question:editor-process', },);

//endregion Logger

//region Types

/**
 Editor exit classification sent to Pi process.
 */
export type EditorExit = 'submitted' | 'cancelled';

//endregion Types

//region Execution

/**
 Runs configured editor attached to detached answer terminal.
 
 @param answerPath - private answer file opened by editor
 
 @param editorCommand - effective executable and configured arguments from Pi process
 
 @param signal - cancellation signal from Pi channel
 
 @returns submitted for zero exit,
 cancelled for nonzero exit
 
 @throws when editor cannot spawn or Pi closes channel while editor is active
 
 @example
 ```ts
 await runEditor({ answerPath: '/tmp/ANSWER.md', editorCommand: ['nano'], signal: new AbortController().signal });
 ```
 */
export async function runEditor(
  {
    answerPath,
    editorCommand,
    signal,
  }: {
    readonly answerPath: string;
    readonly editorCommand: readonly string[];
    readonly signal: AbortSignal;
  },
): Promise<EditorExit> {
  /**
   Executable guaranteed by nonempty editor resolver result.
   */
  const [executable, ...configuredArgs] = editorCommand;
  if (executable === undefined)
    throw new Error('Resolved editor command did not contain an executable.',);
  signal.throwIfAborted();
  l.info(`launching configured editor: ${executable}`,);
  console.log('Write your answer, then save and exit to submit. Leave the file empty to cancel.',);
  /**
   Editor process inherits detached terminal streams.
   */
  const child = spawn(
    executable,
    [
      ...configuredArgs,
      answerPath,
    ],
    { stdio: 'inherit', },
  );
  /**
   Listener terminating editor when originating Pi request disappears.
   */
  using abortSubscription = addAbortListener(
    signal,
    function abortEditor(): void {
      child.kill();
    },
  );
  /**
   Exit tuple emitted after attached editor finishes.
   */
  const exit = await once(
    child,
    'exit',
  );
  signal.throwIfAborted();
  /**
   Numeric process exit code;
   null means signal termination and therefore cancellation.
   */
  const code: unknown = exit[0];
  return code === 0
    ? 'submitted'
    : 'cancelled';
}

//endregion Execution
