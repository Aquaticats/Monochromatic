import { AsyncLocalStorage, } from 'node:async_hooks';
import { resolve, } from 'node:path';

/**
 Glob expansion observed while a builder's lazy callback executed.
 */
export type TrackedGlob = {
  /**
   Glob pattern passed to {@link cat}.
   */
  readonly pattern: string;

  /**
   Paths matched by the glob.
   */
  readonly paths: readonly string[];
};

/**
 Function shape used by source capture.
 */
export type SourceCaptureCallback<TResult,> = () => TResult | Promise<TResult>;

/**
 Captured builder result and source dependencies.
 */
export type CapturedSources<TResult,> = {
  /**
   Value returned by the captured callback.
   */
  readonly value: TResult;

  /**
   Absolute file paths read while the callback executed.
   */
  readonly reads: readonly string[];

  /**
   Glob expansions observed while the callback executed.
   */
  readonly globs: readonly TrackedGlob[];
};

/**
 Mutable async-local capture state.
 */
type SourceCaptureState = {
  /**
   Absolute file paths captured in the current async context.
   */
  readonly reads: Set<string>;

  /**
   Glob expansions captured in the current async context, keyed by pattern.
   */
  readonly globs: Map<string, readonly string[]>;
};

/**
 Async-local capture store used by lazy staleness-cache builders.
 */
const sourceCaptureStorage: AsyncLocalStorage<SourceCaptureState> = new AsyncLocalStorage<SourceCaptureState>();

/**
 Records an already-resolved read path in the active source capture.
 
 @param absolutePath - Absolute path to add to the active capture.
 
 @example
 ```ts
 recordReadInActiveCapture('/repo/AGENTS.md');
 ```
 */
export function recordReadInActiveCapture(absolutePath: string,): void {
  sourceCaptureStorage.getStore()
    ?.reads
    .add(absolutePath,);
}

/**
 Records a glob expansion in the active source capture.
 
 @param pattern - Glob pattern passed to {@link cat}.
 
 @param paths - Paths matched by the glob.
 
 @example
 ```ts
 recordGlobInActiveCapture({ pattern: './src/*.ts', paths: ['./src/index.ts'] });
 ```
 */
export function recordGlobInActiveCapture(
  {
    pattern,
    paths,
  }: {
    readonly pattern: string;
    readonly paths: readonly string[];
  },
): void {
  sourceCaptureStorage.getStore()
    ?.globs
    .set(
    pattern,
    [...new Set(paths.map(function toAbsolutePath(path,): string {
      return resolve(path,);
    },),),].toSorted(),
    );
}

/**
 Returns captured globs in deterministic order.
 
 @param globs - Mutable glob capture map.
 
 @returns Sorted glob captures.
 
 @example
 ```ts
 const globs = capturedGlobs(capture.globs);
 ```
 */
function capturedGlobs(globs: ReadonlyMap<string, readonly string[]>,): readonly TrackedGlob[] {
  return [...globs.entries(),]
    .map(function toTrackedGlob(entry,): TrackedGlob {
      /**
       Glob pattern and paths tuple.
       */
      const [pattern, paths,] = entry;
      return {
        pattern,
        paths,
      };
    },)
    .toSorted(function compareGlobPatterns(
      leftGlob,
      rightGlob,
    ): number {
      return leftGlob
        .pattern
        .localeCompare(rightGlob.pattern,);
    },);
}

/**
 Captures reads and glob expansions performed by an async builder callback.
 
 @param fn - Callback whose calls to {@link cat} and {@link addWatchedPaths} should be captured.
 
 @mutates fn through sourceCaptureStorage.run callback invocation
 
 @returns Callback result plus captured sources.
 
 @example
 ```ts
 const captured = await captureTrackedSources({ fn: () => cat(['./AGENTS.md']) });
 ```
 */
export async function captureTrackedSources<TResult,>(
  {
    fn,
  }: {
    readonly fn: SourceCaptureCallback<TResult>;
  },
): Promise<CapturedSources<Awaited<TResult>>> {
  /**
   Mutable capture state scoped to this async call chain.
   */
  const captureState: SourceCaptureState = {
    reads: new Set<string>(),
    globs: new Map<string, readonly string[]>(),
  };
  /**
   Value returned by the captured callback.
   */
  const value = await sourceCaptureStorage.run(
    captureState,
    fn,
  );
  return {
    value,
    reads: [...captureState.reads,].toSorted(),
    globs: capturedGlobs(captureState.globs,),
  };
}
