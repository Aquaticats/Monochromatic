type LockSyncOptions = {
  readonly realpath?: boolean;
};

type RetriesObject = {
  readonly retries?: number;
  readonly factor?: number;
  readonly minTimeout?: number;
  readonly maxTimeout?: number;
  readonly randomize?: boolean;
};

type LockOptions = {
  readonly retries?: number | RetriesObject;
  readonly stale?: number;
  readonly onCompromised?: (err: Error,) => void;
  readonly realpath?: boolean;
};

/**
 Release callback. Removes the lock directory; tolerates that directory already being gone, rethrows any other
 removal failure, and remains retryable after such a failure.
 */
type Release = () => void;

declare function lock(
  file: string,
  options?: LockOptions,
): Promise<Release>;

declare namespace lock {
  function lock(
    file: string,
    options?: LockOptions,
  ): Promise<Release>;
  function lockSync(
    file: string,
    options?: LockSyncOptions,
  ): Release;
}

export = lock;
