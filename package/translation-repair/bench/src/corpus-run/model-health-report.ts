import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { askModelHealth, } from './model-health-probe.ts';

//region Model health report
// Asks every model of a roster the health question, one at a time, and says
// how many answered.

/**
 Exit code left behind when some model could not be reached at all.

 A ROSTER THAT CANNOT BE FULLY PROBED IS A FINDING, and the caller of a
 diagnostic reads its exit code. Preserved from the behaviour this replaced,
 where an unreachable model crashed the probe and produced a non-zero exit as
 a side effect of dying.
 */
const ROSTER_INCOMPLETE = 1;

/**
 Asks every roster model the health question and logs what returned.

 @param client - client every question goes through

 @param roster - models asked, in order

 @param timeoutMs - how long each call may run before it is abandoned

 @param l - logger the lines are written to, tagged here for this probe

 @returns Exit code the command leaves behind: zero when every model could
 be asked, and one when any could not

 @example
 ```ts
 const exitCode = await reportModelHealth({ client, roster, timeoutMs: 360_000, l, },);
 ```
 */
export async function reportModelHealth(
  {
    client,
    roster,
    timeoutMs,
    l: parent,
  }: {
    readonly client: ModelClient;
    readonly roster: readonly RosterModelId[];
    readonly timeoutMs: number;
    readonly l: Logger;
  },
): Promise<number> {
  /**
   Logger tagged for this probe.
   */
  const l = tagged({
    tag: reportModelHealth.name,
    l: parent,
  },);

  /**
   Models whose probe threw before any outcome could be read.

   SEPARATE FROM AN UNHEALTHY REPLY, which is the distinction this whole probe
   exists to draw. A model that answered badly is evidence about the model; a
   model that could not be asked is evidence about the provider, and reporting
   the second as the first would send a reader looking in the wrong place.
   */
  const unreachable: string[] = [];

  for (const modelId of roster) {
    try {
      /* oxlint-disable no-await-in-loop -- one model at a time on purpose: this is a diagnostic, and concurrent calls would let a provider rate limit read as a model fault */
      await askModelHealth({
        client,
        modelId,
        timeoutMs,
        l,
      },);
      /* oxlint-enable no-await-in-loop */
    } catch (error) {
      // A THROW HERE IS A REPORT, not the end of the probe. Before this, the
      // first model whose provider was out of budget ended the run and every
      // model after it went unreported, which is precisely the moment someone
      // is running this.
      /**
       What was thrown, by class and HTTP status where the provider stated
       one, then the provider's own words labelled as its own (a probe exists
       to show them), never by a message: a runtime rejection for an
       unsendable header quotes the key. Not cut to a preview length, since a
       cut would take the closing quote the words are delimited by; the words
       are at most the excerpt bound the status failure was built with.
       */
      const detail = exchangeFailureLogText({ error, },);

      unreachable.push(modelId,);
      // THE CLASS IS NAMED ONCE. `detail` already names it for every class
      // that does not write its own sentence, so a second naming in front
      // read `UNREACHABLE (TypeError): refused by TypeError`.
      l.warn(`${modelId}: UNREACHABLE: ${detail}`,);
    }
  }

  l.info(
    `ROSTER ${String(roster.length - unreachable.length,)} of `
      + `${String(roster.length,)} ${
        wordForCount({
          count: roster.length,
          one: 'model',
          many: 'models',
        },)
      } answered${
        (unreachable.length === 0)
          ? ''
          : `; unreachable: ${unreachable.join(', ',)}`
      }`,
  );

  return (unreachable.length > 0) ? ROSTER_INCOMPLETE : 0;
}

//endregion Model health report
