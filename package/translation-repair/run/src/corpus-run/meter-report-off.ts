import {
  PROVIDER_ORDER,
  type ProviderName,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Meter report off
// A PROVIDER THE RUN LEFT OFF IS SAID TO BE LEFT OFF (B1209). The launch and the budget sample both write a
// `PROVIDER OFF <provider>: <reason>` line for each provider they leave off (`providers-off.ts`), and a provider left off
// has no client, so its meter reads dry with nothing to report on every reading. The report once counted those readings
// like any other and printed `wet=0 dry=N`, `spendable on 0.0%` and a longest outage for a provider nothing ever asked,
// which reads as an outage and was a decision. It reads the line from the same logs as the readings and says so instead.

/**
 Marker the line a provider left off opens its message with.
 */
const OFF_MARKER = 'PROVIDER OFF ';

/**
 One provider a log says the run left off.

 @example
 ```ts
 const off: ProviderLeftOff = { provider: 'bedrock', reason: 'the owner left it off', };
 ```
 */
export type ProviderLeftOff = {
  /**
   Provider the run left off.
   */
  readonly provider: ProviderName;

  /**
   Why, as the line gave it.
   */
  readonly reason: string;
};

/**
 Reads the providers every log says were left off, each once.

 @param texts - whole text of each log

 @returns Providers left off in the order the logs first named them, with the reason of the first line

 @example
 ```ts
 const off = providersLeftOffIn({ texts: ['[info] PROVIDER OFF bedrock: the owner left it off',], },);
 ```
 */
export function providersLeftOffIn(
  { texts, }: { readonly texts: readonly string[]; },
): readonly ProviderLeftOff[] {
  /**
   Providers named so far, each beside the reason of its first line.
   */
  const named = new Map<ProviderName, string>();
  for (const text of texts) {
    for (const line of text.split('\n',)) {
      /**
       Where the marker sits, or that this is an ordinary line.
       */
      const markerAt = line.indexOf(OFF_MARKER,);
      if (markerAt === (-1))
        continue;
      /**
       Everything written after the marker: the provider, a colon and the reason.
       */
      const tail = line.slice(markerAt + OFF_MARKER.length,);
      /**
       Where the provider's name ends.
       */
      const colonAt = tail.indexOf(':',);
      if (colonAt === (-1))
        continue;
      /**
       The word the line names as left off.
       */
      const word = tail.slice(
        0,
        colonAt,
      );
      /**
       The provider it names, absent for a word that is none of them.
       */
      const provider = PROVIDER_ORDER.find(function isNamed(candidate,): boolean {
        return candidate === word;
      },);
      if ((provider !== undefined) && (!named.has(provider,))) {
        /**
         Why the line says the provider is left off.
         */
        const reason = tail
          .slice(colonAt + 1,)
          .trim();
        named.set(
          provider,
          reason,
        );
      }
    }
  }
  /**
   The providers named, each with its reason, in the order they were first named.
   */
  const found: ProviderLeftOff[] = [];
  for (const [provider, reason,] of named) {
    found.push({
      provider,
      reason,
    },);
  }
  return found;
}

/**
 The lines the report prints for a provider the run left off, in place of its counts.

 @param off - provider and the reason its line gave

 @returns Lines naming the provider as left off and saying why it has no figures

 @example
 ```ts
 for (const line of leftOffLines({ off, },)) console.log(line,);
 ```
 */
export function leftOffLines({ off, }: { readonly off: ProviderLeftOff; },): readonly string[] {
  return [
    `\n${off.provider}: LEFT OFF, not asked (PROVIDER OFF line: ${off.reason})`,
    '  its meter reads dry with nothing to report, so no wet or dry count, spendable share or outage is given for it',
  ];
}

//endregion Meter report off
