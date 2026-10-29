import spawn from 'nano-spawn';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import { log, } from './syslog.ts';

/**
 Sends a critical desktop notification via `notify-send` to alert the user
 about sustained unproductive activity.
 
 @param summary - LLM-generated analysis summary shown in the notification body
 
 @example
 ```ts
 await sendNotification("User has been browsing Reddit for 25 minutes.");
 ```
 */
export async function sendNotification(summary: string,): Promise<void> {
  try {
    await spawn(
      'notify-send',
      [
        '--urgency=critical',
        '--app-name=Hall Monitor',
        "Hall Monitor: You've been unproductive!",
        summary,
      ],
    );
    log.info('[notify] Desktop notification sent.',);
  }
  catch (err: unknown) {
    /**
     Caught error rendered as a string; preserves `err.message` for `Error` instances, otherwise coerces via `String(err)`.
     */
    const message = caughtValueText(err,);
    console.error(`[notify] Failed to send notification: ${message}`,);
    log.error(`[notify] Failed to send notification: ${message}`,);
  }
}
