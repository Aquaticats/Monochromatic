import spawn from 'nano-spawn';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { log, } from './syslog.ts';

/**
 Module logger: the journal identity this package files every record under, then this module's
 subsystem tag.
 */
const l = tagged({
  l: log,
  tag: 'notify',
},);

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
  /**
   Notification logger, tagged with this function's name under the module's subsystem tag.
   */
  const fnLog = tagged({
    l,
    tag: sendNotification.name,
  },);

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
    fnLog.info('Desktop notification sent.',);
  }
  catch (err: unknown) {
    /**
     Caught error rendered as a string; preserves `err.message` for `Error` instances, otherwise coerces via `String(err)`.
     */
    const message = caughtValueText(err,);
    fnLog.error(`Failed to send notification: ${message}`,);
  }
}
