import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Run configuration error
// The refusal a run raises before any model call when its environment lacks
// every provider key. Its own file since 2026-09-07 so that the provider
// construction, split out of `run-config.ts` under the line budget, can raise
// it without importing the file that imports it.

/**
 Raised when a setting a run depends on is absent from its environment.

 @example
 ```ts
 throw new RunConfigError({ variable: 'TRANSLATION_REPAIR_SYNTHETIC_API_KEY', },);
 ```
 */
export class RunConfigError extends StatedRefusalError {
  /**
   Declared here as well as inherited, so the source scan that keeps the
   marked-class inventory sees it: the message names a variable and a fix.
   */
  override readonly messageNamesOnly: true = true;

  /**
   Builds refusal naming the variable that could not be read.

   @param variable - environment variable name a run cannot start without

   @example
   ```ts
   throw new RunConfigError({ variable: 'TRANSLATION_REPAIR_SYNTHETIC_API_KEY', },);
   ```
   */
  public constructor({ variable, }: { readonly variable: string; },) {
    super({ says: `${variable} is not set; run under mise so sops injects it`, },);
    this.name = 'RunConfigError';
  }
}

//endregion Run configuration error
