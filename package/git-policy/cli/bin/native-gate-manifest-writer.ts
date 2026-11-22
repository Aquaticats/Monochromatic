/**
 One gate run's manifest writer: every write it is asked for lands after the one before it, so two containers' IDs read at
 the same time never interleave their writes into one file.

 A gate's suite and Clippy start together and each reports its own ID as soon as its client writes it, so without this
 order the two writes would overlap, and the one that finished last would not be the one holding both IDs.
 `writeGateManifest` moves each write into place with one rename (`native-gate-manifest.ts`), so a reader sees the manifest
 as it stood before or after a write and never a half-written one.
 */
import {
  type GateManifest,
  writeGateManifest,
} from './native-gate-manifest.ts';

/**
 Make one step run at a time: each call waits for the call before it, so two callers never run the step together.

 The step is not retried and an earlier failure is not repeated to a later caller: that caller's own step still runs,
 because it records the state as it stands then.

 @param step - the work one call does, run one call at a time

 @returns a function that runs the step after every call before it, answering once its own run ended

 @example
 ```ts
 const writeAfter = serialStepOf({ step: writeManifest });
 await writeAfter();
 ```
 */
function serialStepOf({ step }: { readonly step: () => Promise<void>; },): () => Promise<void> {
  /**
   The run asked for last, which the next call waits for.

   It never rejects: a failed run already answered the caller that asked for it, and a later caller's step must still
   run.
   */
  let inFlight: Promise<void> = Promise.resolve();
  /**
   Run this step after every run asked for before it, and settle this run's own slot when it ends either way.
   */
  async function stepAfterEarlier(): Promise<void> {
    /**
     The run this one waits for.
     */
    const earlier = inFlight;
    /**
     This run's own settlement, which the next call waits for.
     */
    const mine = Promise.withResolvers<void>();
    inFlight = mine.promise;
    try {
      await earlier;
      await step();
      mine.resolve();
    }
    catch (error) {
      // The next call takes this slot and records the state as it stands then, so it waits for this run to end and not
      // for it to succeed.
      mine.resolve();
      throw error;
    }
  }
  return stepAfterEarlier;
}

/**
 This gate run's manifest writer, whose writes land one after the other.

 @param manifestOf - builds the manifest as the run stands now, called once per write so it carries every ID read so far

 @param write - writes one manifest, which a test replaces with its own

 @returns a function that writes the manifest, answering once its own write is on disk

 @example
 ```ts
 const writeManifest = gateManifestWriterOf({ manifestOf });
 await writeManifest();
 ```
 */
export function gateManifestWriterOf({
  manifestOf,
  write = writeGateManifest,
}: {
  readonly manifestOf: () => GateManifest;
  readonly write?: (manifest: GateManifest) => Promise<void>;
},): () => Promise<void> {
  return serialStepOf({
    step: function writeCurrentManifest(): Promise<void> {
      return write(manifestOf(),);
    },
  },);
}
