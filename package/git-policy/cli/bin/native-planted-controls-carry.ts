/**
 What an obligation becomes when a campaign writes the manifest that replaces the one holding it: carried forward, unless
 this attempt's proof or local pass pays it by the obligation's own keys.
 */
import { join } from 'node:path';

import { passOneDirectoryName } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/campaign-passes.ts';

import {
  type ExecutionLimits,
  limitsAtLeast,
} from './native-gate-proof.ts';
import type { PlantedDecision } from './native-planted-controls-ensure.ts';
import {
  manifested,
  obligationIdentity,
  type PlantedObligation,
} from './native-planted-controls-manifest.ts';
import { provesControls } from './native-planted-controls-proof.ts';
import type { ControlsKey } from './native-planted-controls-record.ts';
import { NativeVerificationError } from './native-verification-process.ts';

/**
 The obligations the manifests a campaign is about to replace hold.

 @param evidence - the campaign's evidence directory, which a resumed campaign wrote before, and whose `pass-1` directory
 may hold the first pass's

 @returns the obligations of both manifests, an obligation held by both once

 @throws NativeVerificationError when a manifest cannot be read or holds an incomplete obligation, because replacing it
 would erase what it owed

 @example
 ```ts
 await carriedObligations(evidence);
 ```
 */
export async function carriedObligations(evidence: string): Promise<readonly PlantedObligation[]> {
  /**
   What each directory's manifest holds.
   */
  const reads = await Promise.all([
    evidence,
    join(
      evidence,
      passOneDirectoryName,
    ),
  ].map(function manifestedIn(directory) {
    return manifested(directory);
  }));
  /**
   Reads that cannot be trusted.
   */
  const invalid = reads.flatMap(function reasonOf(read) {
    return read.kind === 'invalid' ? [read.reason] : [];
  });
  if (invalid.length > 0)
    throw new NativeVerificationError(`A manifest this campaign would replace cannot be read, so what it deferred cannot be carried forward: ${invalid.join('; ')} Move the campaign directory away to start it over.`);
  return reads.flatMap(function listOf(read) {
    return read.kind === 'obligations' ? read.list : [];
  })
    .filter(function isFirst(
      obligation,
      index,
      all
    ) {
      return all.findIndex(function isSame(other) {
        return obligationIdentity(other) === obligationIdentity(obligation);
      }) === index;
    });
}

/**
 Whether this attempt's proof or local pass satisfies an obligation, by the obligation's own keys.

 @param obligation - what an earlier attempt deferred under

 @param decision - what this attempt decided

 @param observed - the image, limits, controls key and runner identity this attempt ran or proved under; absent limits
 mean they could not be read, so nothing is satisfied by a local pass

 @returns true when a proof record names the obligation's image, controls, harness and limits, or this attempt ran the
 same controls on the same image under limits the obligation's campaign had at least

 @example
 ```ts
 await isSatisfied({ obligation, decision, observed });
 ```
 */
async function isSatisfied({
  obligation,
  decision,
  observed,
}: {
  readonly obligation: PlantedObligation;
  readonly decision: PlantedDecision;
  readonly observed: {
    readonly image: string;
    readonly limits?: ExecutionLimits;
    readonly key: ControlsKey;
    readonly harness: string;
  };
}): Promise<boolean> {
  if (decision.kind === 'proved')
    return await provesControls({
      file: decision.record,
      image: obligation.image,
      limits: obligation.limits,
      key: obligation.controls,
      harness: obligation.harness,
    });
  if ((decision.kind !== 'ran') || (observed.limits === undefined))
    return false;
  return (obligation.image === observed.image) && (obligation.controls
    .sha256
    === observed.key
      .sha256)
    && (JSON.stringify(obligation.controls
      .names) === JSON.stringify(observed.key
        .names))
    && (obligation.harness === observed.harness)
    && limitsAtLeast({
      campaign: obligation.limits,
      recorded: observed.limits,
    });
}

/**
 The obligations a campaign has after an attempt's decision, for the manifest it writes next.

 A deferral adds its obligation to those carried. A proof or a local pass removes the carried ones it satisfies by their own
 keys, so a resumed campaign cannot pay an old debt with a proof for other controls, other code or smaller limits. The
 person's explicit skip is a deliberate exception that removes all of them.

 @param evidence - the campaign's evidence directory

 @param decision - what this attempt decided

 @param observed - the image, limits, controls key and runner identity this attempt ran or proved under

 @returns the obligations to write

 @throws NativeVerificationError when a manifest to be replaced cannot be read

 @example
 ```ts
 await nextObligations({ evidence, decision, observed });
 ```
 */
export async function nextObligations({
  evidence,
  decision,
  observed,
}: {
  readonly evidence: string;
  readonly decision: PlantedDecision;
  readonly observed: {
    readonly image: string;
    readonly limits?: ExecutionLimits;
    readonly key: ControlsKey;
    readonly harness: string;
  };
}): Promise<readonly PlantedObligation[]> {
  if (decision.kind === 'requested')
    return [];
  /**
   The obligations of the manifests about to be replaced.
   */
  const carried = await carriedObligations(evidence);
  /**
   Whether each carried obligation is satisfied by this attempt.
   */
  const satisfied = await Promise.all(carried.map(function satisfies(obligation) {
    return isSatisfied({
      obligation,
      decision,
      observed,
    });
  }));
  /**
   The outstanding obligations followed by the new deferral, if any.
   */
  const next = [
    ...carried.filter(function isOutstanding(
      _obligation,
      index
    ) {
      return satisfied[index] !== true;
    }),
    ...(decision.kind === 'deferred' ? [decision.obligation] : []),
  ];
  // The same obligation, carried and decided again, is owed once.
  return next.filter(function isFirst(
    obligation,
    index
  ) {
    return next.findIndex(function isSame(other) {
      return obligationIdentity(other) === obligationIdentity(obligation);
    }) === index;
  });
}
