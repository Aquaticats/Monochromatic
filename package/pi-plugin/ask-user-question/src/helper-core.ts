import type { Socket, } from 'node:net';
import { once, } from 'node:events';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { runEditor, } from './editor-process.ts';
import {
  type HelperCompletion,
  serializeHelperCompletion,
} from './helper-protocol.ts';
import { readHelperRequest, } from './helper-request.ts';
import {
  AnswerRequestEndedError,
  authenticateHelper,
  createHelperConnection,
} from './helper-connection.ts';

//region Logger

/**
 Tagged logger for detached answer helper.
 */
const l = tagged({ tag: 'ask-user-question:helper-core', },);

//endregion Logger

//region Public helper

/**
 Authenticates with Pi,
 runs configured editor,
 and sends terminal completion frame.
 
 @param requestPath - private helper request path
 
 @throws when request or loopback connection cannot be established
 
 @example
 ```ts
 await runAnswerHelper({ requestPath: '/tmp/request.json' });
 ```
 */
export async function runAnswerHelper(
  { requestPath, }: { readonly requestPath: string; },
): Promise<void> {
  /**
   Private endpoint and answer file from Pi process.
   */
  const request = await readHelperRequest({ requestPath, },);
  /**
   Loopback connection kept open for editor lifetime.
   */
  using connection = createHelperConnection(request,);
  /**
   Request-owned channel and editor cancellation capabilities.
   */
  const {
    socket,
    signal,
  } = connection;
  try {
    await authenticateHelper({
      connection,
      token: request.token,
    },);
    /**
     Attached editor outcome mapped directly to protocol status.
     */
    const status = await runEditor({
      answerPath: request.answerPath,
      editorCommand: request.editorCommand,
      signal,
    },);
    await sendCompletion({
      socket,
      completion: { status, },
    },);
  }
  catch (error: unknown) {
    if (signal.aborted || (error instanceof AnswerRequestEndedError)) {
      l.debug(`answer request ended: ${String(error,)}`,);
      console.log('This question is no longer active. Return to Pi for the current question.',);
      return;
    }
    l.error(`answer helper failed: ${String(error,)}`,);
    await sendCompletion({
      socket,
      completion: {
        status: 'error',
        message: caughtValueText(error,),
      },
    },);
  }
}

//endregion Public helper

//region Protocol output

/**
 Sends one terminal completion frame and waits for socket settlement.
 
 @param socket - authenticated Pi socket
 
 @param completion - terminal editor completion status
 */
async function sendCompletion(
  {
    socket,
    completion,
  }: {
    readonly socket: Socket;
    readonly completion: HelperCompletion;
  },
): Promise<void> {
  /**
   Close event subscribed before ending stream to avoid race.
   */
  const closed = once(
    socket,
    'close',
  );
  socket.end(serializeHelperCompletion({ completion, }),);
  await closed;
}

//endregion Protocol output
