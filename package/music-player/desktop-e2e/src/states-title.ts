/**
 The window title of the playing track: what a good title looks like, and how to wait for the window to show it.

 The app writes the state line for playback before the window shows the title that goes with it, because the window
 takes a new title when it draws its next frame. A title read right after the state line can therefore be one frame
 old: measured on the release binary, about one read in eight right after Play still answers with the idle title,
 and the new one follows within a few milliseconds. A check of the title waits for the title itself.

 @example
 ```ts
 import { waitForCleanedTitle, } from './states-title.ts';

 const title = await waitForCleanedTitle({ session, },);
 ```

 @packageDocumentation
 */

import { CheckFailureError, } from './errors.ts';
import { pollUntilDeadline, } from './poll.ts';
import type { Session, } from './session.ts';

/**
 How long the window may take to show the title of the track that plays, in milliseconds. A frame takes milliseconds;
 the bound stays below the length of the first track of the main states library (three seconds), because the title
 changes again when that track ends.
 */
export const TITLE_FOLLOWS_MS = 2_000;

/**
 Pause between two questions to the compositor about the title, in milliseconds.
 */
export const TITLE_POLL_MS = 10;

/**
 Condition: the compositor's answer is the cleaned title of the first track of the main states library.

 @param answer - the compositor's answer to the title command

 @returns whether the title names the track without its folder or extension

 @example
 ```ts
 isCleanedFirstTrackTitle('ok One',); // true
 isCleanedFirstTrackTitle('ok Music Player',); // false
 ```
 */
export function isCleanedFirstTrackTitle(answer: string,): boolean {
  return answer.includes('One',)
    && (!answer.includes('.wav',))
    && (!answer.includes('Aaa Folder',));
}

/**
 Asks the compositor for the window title until it is the cleaned title of the first track, or the time is up.

 @param session - the session whose window to ask about

 @returns the compositor's answer that satisfied the condition

 @throws CheckFailureError when the window never shows such a title, with the last answer in the message

 @example
 ```ts
 const title = await waitForCleanedTitle({ session, },);
 ```
 */
export async function waitForCleanedTitle(
  {
    session,
  }: {
    readonly session: Session;
  },
): Promise<string> {
  /**
   The latest answer, kept for the message.
   */
  const latest = { answer: '', };
  /**
   Whether the window showed the cleaned title in time.
   */
  const arrived = await pollUntilDeadline({
    timeoutMs: TITLE_FOLLOWS_MS,
    intervalMs: TITLE_POLL_MS,
    check: async function titleIsCleaned(): Promise<boolean> {
      latest.answer = await session.send('title',);
      return isCleanedFirstTrackTitle(latest.answer,);
    },
  },);
  if (!arrived)
    throw new CheckFailureError(
      `the window title was "${latest.answer}" for ${TITLE_FOLLOWS_MS} ms after the track began to play`,
    );
  return latest.answer;
}
