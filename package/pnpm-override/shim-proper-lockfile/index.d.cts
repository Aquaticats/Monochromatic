type LockSyncOptions = {
  readonly realpath?: boolean;
  readonly stale?: number;
  readonly onCompromised?: (err: Error,) => void;
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
 Release callback.
 Removes the lock only while the record in it names this process, tolerates the lock already being gone, and
 rethrows any other removal failure so a later call retries.
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
