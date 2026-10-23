import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { EveryProviderDryError, } from '@monochromatic-dev/translation-repair-provider/ts';

//region Stage run end
// THE ONE QUESTION "DOES THIS FAILURE END THE RUN?", asked by every layer that would otherwise turn a failed call into
// a lost voice (`attemptStageCall`, `attemptDecisionCall`, `askOnce` and the grace-window wait in `runGatherRound`,
// the settle in `readImagePair`, the failure choice in `mapOverlapped`) and by the entry scheduler that files a caught
// failure (`entryErrorOutcome`). The pass guard (`PassDryStop.guard`) is what covers every other layer: it aborts the
// shared signal before the error reaches any catch, and every layer that checks `signal.aborted` rethrows. A caller
// that builds a client without the guard is protected only by the layers named here. The owner ruled that `EveryProviderDryError` "ENDS THE RUN":
// every other budget state leaves something buyable, and this one leaves nothing, so no seat's voice can be heard
// either. XingZ6018 (2026-10-22) lost it as one seat's voice in every round for 77 s, until the bench floor stopped
// the run mid-entry. The scheduler reaches this through the stage package because it holds no runtime dependency on
// the provider package of its own.

/**
 Whether a raised value is the refusal that ends the run, narrowing it to its class so a caller can keep and rethrow it.

 @param error - what the call raised

 @returns Whether the router found every provider out of budget

 @example
 ```ts
 if (isDryRefusal(error))
   throw error;
 ```
 */
export function isDryRefusal(error: unknown,): error is EveryProviderDryError {
  return error instanceof EveryProviderDryError;
}

/**
 Whether a failed call's error ends the run instead of costing one seat its voice.

 @param error - what the call raised

 @returns Whether the router found every provider out of budget

 @example
 ```ts
 if (endsTheRun({ error, },))
   throw error;
 ```
 */
export function endsTheRun({ error, }: { readonly error: unknown; },): boolean {
  return isDryRefusal(error,);
}

/**
 Whether a failure carries the distance to a NeuralWatt return, which the pass may park on: the router's refusal that
 names one, or a writing bench error whose bench the return would lift. Read by shape, since this package cannot import
 the run package that makes the bench error; the shape is the field the gate reads.

 @param error - what the call raised

 @returns Whether the failure names a return the pass may wait for

 @example
 ```ts
 if (parksTheRun({ error, },))
   throw error;
 ```
 */
export function parksTheRun({ error, }: { readonly error: unknown; },): boolean {
  return isJsonRecord(error,) && ('neuralwattReturnsInMs' in error)
    && ((typeof error.neuralwattReturnsInMs) === 'number');
}

//endregion Stage run end
