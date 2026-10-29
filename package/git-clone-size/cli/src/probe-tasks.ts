import type { Signals, } from './assemble.ts';
import {
  hostCommitCount,
  hostStorageBytes,
  lsRemote,
  NO_HOST_COMMITS,
  NO_REFS,
  NO_STORAGE,
  type HostCommitCountResult,
  type LsRemoteResult,
} from './host-api.ts';
import {
  NO_DEEPEN,
  probeDeepen,
  type DeepenResult,
} from './probe-deepen.ts';
import {
  NO_CHURN,
  NO_TREE0,
  partialChurn,
  partialCommitCount,
  type ChurnResult,
  type CommitCountResult,
} from './probe-partial.ts';
import type { ShallowResult, } from './probe-shallow.ts';
import type { RemoteSource, } from './source.ts';

/**
 One settled probe signal, discriminated by `kind`. `none` marks a probe that
 yielded nothing (unsupported, failed, or budget-aborted), so the snapshot
 still advances without recording a value.
 */
export type Signal =
  | {
    readonly kind: 'shallow';
    readonly shallowBytes: number
  }
    | {
      readonly kind: 'deepen';
      readonly deepen: DeepenResult
    }
    | {
      readonly kind: 'refs';
      readonly refs: LsRemoteResult
    }
    | {
      readonly kind: 'storage';
      readonly storageBytes: number
    }
    | {
      readonly kind: 'commit';
      readonly commit: HostCommitCountResult
    }
    | {
      readonly kind: 'tree0';
      readonly tree0: CommitCountResult
    }
    | {
      readonly kind: 'churn';
      readonly churn: ChurnResult
    }
    | {
      readonly kind: 'none';
      readonly which: string
    };

/**
 Resolves the shared shallow-clone promise into a shallow signal.
 
 @param shallowPromise - the single in-flight shallow clone
 
 @returns a `shallow` signal, or `none` when the clone failed
 
 @example
 ```ts
 const sig = await shallowSignal({ shallowPromise });
 ```
 */
export async function shallowSignal(
  { shallowPromise, }: { readonly shallowPromise: Promise<ShallowResult>; },
): Promise<Signal> {
  /**
   Resolved shallow clone result.
   */
  const shallow = await shallowPromise;
  if ((!shallow.ok) || (shallow.shallowBytes === undefined)
    || (shallow.shallowBytes <= 0))
    return {
      kind: 'none',
      which: 'shallow',
    };
  return {
    kind: 'shallow',
    shallowBytes: shallow.shallowBytes,
  };
}

/**
 Runs the deepen probe on the shared shallow clone once it exists.
 
 @param shallowPromise - the single in-flight shallow clone
 
 @param maxDeepenCommits - cap on commits walked
 
 @param signal - wall-clock budget signal
 
 @returns a `deepen` signal, or `none` when no usable delta was observed
 
 @example
 ```ts
 const sig = await deepenSignal({ shallowPromise, maxDeepenCommits: 256 });
 ```
 */
export async function deepenSignal(
  {
    shallowPromise,
    maxDeepenCommits,
    signal,
  }: {
    readonly shallowPromise: Promise<ShallowResult>;
    readonly maxDeepenCommits: number;
    readonly signal: AbortSignal;
  },
): Promise<Signal> {
  /**
   Resolved shallow clone result, reused as the deepen base.
   */
  const shallow = await shallowPromise;
  if (!shallow.ok)
    return {
      kind: 'none',
      which: 'deepen',
    };
  /**
   Deepen probe outcome.
   */
  const deepen = await probeDeepen({
    clonePath: shallow.clonePath,
    maxDeepenCommits,
    signal,
  },);
  if (deepen === NO_DEEPEN)
    return {
      kind: 'none',
      which: 'deepen',
    };
  return {
    kind: 'deepen',
    deepen,
  };
}

/**
 Counts refs via `git ls-remote`.
 
 @param url - remote clone URL
 
 @returns a `refs` signal, or `none` on failure
 
 @example
 ```ts
 const sig = await refsSignal({ url });
 ```
 */
export async function refsSignal({ url, }: { readonly url: string; },): Promise<Signal> {
  /**
   Ref inventory, or absent on failure.
   */
  const refs = await lsRemote({ url, },);
  if (refs === NO_REFS)
    return {
      kind: 'none',
      which: 'refs',
    };
  return {
    kind: 'refs',
    refs,
  };
}

/**
 Reads the host storage proxy.
 
 @param source - parsed remote with host/owner/repo
 
 @returns a `storage` signal, or `none` when unavailable
 
 @example
 ```ts
 const sig = await storageSignal({ source });
 ```
 */
export async function storageSignal({ source, }: { readonly source: RemoteSource; },): Promise<Signal> {
  /**
   Host storage proxy, or absent.
   */
  const storage = await hostStorageBytes({ source, },);
  if (storage === NO_STORAGE)
    return {
      kind: 'none',
      which: 'host-proxy',
    };
  return {
    kind: 'storage',
    storageBytes: storage.bytes,
  };
}

/**
 Reads the host commit count.
 
 @param source - parsed remote with host/owner/repo
 
 @returns a `commit` signal, or `none` when unavailable
 
 @example
 ```ts
 const sig = await commitSignal({ source });
 ```
 */
export async function commitSignal({ source, }: { readonly source: RemoteSource; },): Promise<Signal> {
  /**
   Host commit count, or absent.
   */
  const commit = await hostCommitCount({ source, },);
  if (commit === NO_HOST_COMMITS)
    return {
      kind: 'none',
      which: 'commit-count',
    };
  return {
    kind: 'commit',
    commit,
  };
}

/**
 Counts commits via a tree:0 partial clone.
 
 @param url - remote clone URL
 
 @param dest - temp directory for the clone
 
 @param signal - wall-clock budget signal
 
 @returns a `tree0` signal, or `none` when unsupported
 
 @example
 ```ts
 const sig = await tree0Signal({ url, dest });
 ```
 */
export async function tree0Signal(
  {
    url,
    dest,
    signal,
  }: {
    readonly url: string;
    readonly dest: string;
    readonly signal: AbortSignal;
  },
): Promise<Signal> {
  /**
   Commit count from the commits-only clone, or absent.
   */
  const tree0 = await partialCommitCount({
    url,
    dest,
    signal,
  },);
  if (tree0 === NO_TREE0)
    return {
      kind: 'none',
      which: 'commit-count',
    };
  return {
    kind: 'tree0',
    tree0,
  };
}

/**
 Measures churn via a blobless partial clone.
 
 @param url - remote clone URL
 
 @param dest - temp directory for the clone
 
 @param signal - wall-clock budget signal
 
 @returns a `churn` signal, or `none` when unsupported
 
 @example
 ```ts
 const sig = await churnSignal({ url, dest });
 ```
 */
export async function churnSignal(
  {
    url,
    dest,
    signal,
  }: {
    readonly url: string;
    readonly dest: string;
    readonly signal: AbortSignal;
  },
): Promise<Signal> {
  /**
   Churn counts from the blobless clone, or absent.
   */
  const churn = await partialChurn({
    url,
    dest,
    signal,
  },);
  if (churn === NO_CHURN)
    return {
      kind: 'none',
      which: 'churn',
    };
  return {
    kind: 'churn',
    churn,
  };
}

/**
 Folds a settled signal into the accumulator, returning a new {@link Signals}.
 A `none` signal yields the input unchanged, advancing the stream without a
 value.
 
 @param signals - current accumulator
 
 @param signal - the settled probe signal
 
 @returns a new accumulator including the signal, or the input for `none`
 
 @example
 ```ts
 const next = applySignal({ signals, signal: { kind: 'shallow', shallowBytes: 4_000_000 } });
 ```
 */
export function applySignal(
  {
    signals,
    signal,
  }: {
    readonly signals: Signals;
    readonly signal: Signal;
  },
): Signals {
  if (signal.kind === 'shallow')
    return {
      ...signals,
      shallowBytes: signal.shallowBytes,
    };
  if (signal.kind === 'deepen')
    return {
      ...signals,
      deepen: signal.deepen,
    };
  if (signal.kind === 'refs')
    return {
      ...signals,
      refs: signal.refs,
    };
  if (signal.kind === 'storage')
    return {
      ...signals,
      storageBytes: signal.storageBytes,
    };
  if (signal.kind === 'commit')
    return {
      ...signals,
      commit: signal.commit,
    };
  if (signal.kind === 'tree0')
    return {
      ...signals,
      tree0: signal.tree0,
    };
  if (signal.kind === 'churn')
    return {
      ...signals,
      churn: signal.churn,
    };
  return signals;
}
