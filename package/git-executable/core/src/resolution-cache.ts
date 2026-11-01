//region Bounded successful-resolution cache

/**
 Successful candidate sequences retained for process-lifetime reuse.
 Production callers normally use one environment;
 bound protects callers injecting changing environments.
 */
const MAX_CACHED_REAL_GIT_RESOLUTIONS = 16;

/**
 Successful and in-flight resolutions keyed by effective candidate sequence.
 Rejected resolutions are removed before their errors escape.
 */
const resolutionByCandidateSequence = new Map<string, Promise<string>>();

/**
 Successful keys in least-recently-used to most-recently-used order.
 In-flight keys remain only in {@link resolutionByCandidateSequence}.
 */
const successfulResolutionKeys = new Set<string>();

/**
 Marks successful key as most recently used and evicts oldest success beyond bound.
 
 @param cacheKey - Effective absolute candidate sequence identity.
 
 @example
 ```ts
 retainSuccessfulResolution('[/usr/bin/git]');
 ```
 */
function retainSuccessfulResolution(cacheKey: string,): void {
  successfulResolutionKeys.delete(cacheKey,);
  successfulResolutionKeys.add(cacheKey,);

  if (successfulResolutionKeys.size <= MAX_CACHED_REAL_GIT_RESOLUTIONS)
    return;

  /**
   Least recently used successful key at insertion-order head.
   */
  const oldestSuccessfulKey = successfulResolutionKeys
    .values()
    .next()
    .value;
  if (oldestSuccessfulKey === undefined)
    throw new Error('Successful real-Git cache exceeded bound without an eviction key.',);
  successfulResolutionKeys.delete(oldestSuccessfulKey,);
  resolutionByCandidateSequence.delete(oldestSuccessfulKey,);
}

/**
 Shares in-flight real-Git lookup and retains bounded successful results.
 
 @param cacheKey - Effective absolute candidate sequence identity.
 
 @param resolve - Fresh candidate scan invoked only on cache miss.
 
 @returns Cached or freshly resolved real-Git path.
 
 @throws Whatever fresh scan rejects with;
 rejection is not retained.
 
 @example
 ```ts
 await resolveCachedRealGit({
   cacheKey: '["/usr/bin/git"]',
   resolve: async () => '/usr/bin/git',
 });
 ```
 */
export async function resolveCachedRealGit({
  cacheKey,
  resolve,
}: {
  readonly cacheKey: string;
  readonly resolve: () => Promise<string>;
},): Promise<string> {
  /**
   In-flight or successful equal resolution when already known.
   */
  const cachedResolution = resolutionByCandidateSequence.get(cacheKey,);
  if (cachedResolution !== undefined) {
    if (successfulResolutionKeys.has(cacheKey,))
      retainSuccessfulResolution(cacheKey,);
    return await cachedResolution;
  }

  /**
   Fresh scan stored before awaiting so concurrent callers share it.
   */
  const resolution = resolve();
  resolutionByCandidateSequence.set(
    cacheKey,
    resolution,
  );

  try {
    /**
     Successful result retained before returning to caller.
     */
    const resolvedPath = await resolution;
    retainSuccessfulResolution(cacheKey,);
    return resolvedPath;
  }
  catch (error) {
    resolutionByCandidateSequence.delete(cacheKey,);
    throw error;
  }
}

//endregion Bounded successful-resolution cache
