/**
 Per-index ledger of which loaded sources lost callables and which sources each one reached.

 A demand-driven index loads sources on request and keeps them for every later request, so the sources
 it holds are not the sources one callable's summary was built from. The ledger keeps the edges that
 expansion followed, which is what lets a lookup name the omissions of exactly the sources a summary
 depends on, whatever else the index loaded for other callables.

 @module
 */

import type { EffectSummaryOmissionReason, } from './effect-omission-reason.ts';
import {
  type ExternalOmissionRecord,
  mergeExternalOmissions,
} from './effect-external-omission.ts';

/**
 What the ledger keeps for one loaded source.
 */
type LedgerEntry = {
  readonly own: readonly ExternalOmissionRecord[];
  readonly external: readonly ExternalOmissionRecord[];
  readonly dependencies: readonly string[];
};

/**
 What a loaded source lost,
 in the shape both a restored cache hit and a pending store carry.
 */
export type LoadedSourceOmissions = {
  readonly omittedCallableKeys: readonly string[];
  readonly omissionReasons: readonly EffectSummaryOmissionReason[];
  readonly externalOmissions: readonly ExternalOmissionRecord[];
};

/**
 Mutable ledger of the sources one index has loaded.
 */
export type OmissionLedger = {
  /**
   Records one loaded source,
   whether scanned afresh or restored from the cache.
   */
  readonly record: (options: {
    readonly fileName: string;
    readonly source: LoadedSourceOmissions;
    readonly dependencies: readonly string[];
  }) => void;
  /**
   Lists the omissions of a source and every source reached from it.
   */
  readonly reachedFrom: (fileName: string) => readonly ExternalOmissionRecord[];
};

/**
 Creates an empty ledger for one index.

 @returns ledger recording loaded sources and answering reach queries.

 @example
 ```ts
 const ledger = createOmissionLedger();
 ```
 */
export function createOmissionLedger(): OmissionLedger {
  /**
   Entry per loaded source.
   */
  const entries = new Map<string, LedgerEntry>();
  return {
    record({
      fileName,
      source,
      dependencies,
    },): void {
      /**
       Callables of this source left out of the index.
       */
      const { omittedCallableKeys, } = source;
      entries.set(
        fileName,
        {
          own: omittedCallableKeys.length === 0
            ? []
            : [{
              sourceFileName: fileName,
              omittedCount: omittedCallableKeys.length,
              reasons: [...source.omissionReasons,].toSorted(),
            },],
          external: source.externalOmissions,
          dependencies,
        },
      );
    },
    reachedFrom(fileName,): readonly ExternalOmissionRecord[] {
      /**
       Sources already walked.
       */
      const visited = new Set<string>();
      /**
       Sources awaiting their records and edges.
       */
      const pending: string[] = [fileName,];
      /**
       Record lists gathered from every walked source.
       */
      const groups: (readonly ExternalOmissionRecord[])[] = [];
      while (pending.length > 0) {
        /**
         Next source to walk,
         absent only after unexpected stack mutation.
         */
        const current = pending.pop();
        if ((current === undefined) || visited.has(current,))
          continue;
        visited.add(current,);
        /**
         Entry of the source,
         absent when the index never loaded it.
         */
        const entry = entries.get(current,);
        if (entry === undefined)
          continue;
        groups.push(
          entry.own,
          entry.external,
        );
        pending.push(...entry.dependencies,);
      }
      return mergeExternalOmissions(groups,);
    },
  };
}
