/**
 Log severity levels ordered from least to most severe.
 */
export type Level = 'debug' | 'error' | 'fatal' | 'info' | 'trace' | 'warn';

/**
 Structured log record written to sinks.
 */
export type LogRecord = {
  readonly level: Level;
  readonly message: string;
  readonly timestamp: number;
};

/**
 Optional drain hook for sinks that buffer records internally.
 Called via logger-level {@link Logger.flush} to force buffered work
 through before a process exit, critical error boundary, or assertion.
 
 Always async: sinks whose drain is synchronous return an
 already-resolved promise so callers `await` uniformly. A `void` arm is
 not used; under the `no-optional-escape` rule `T | void` is a banned
 fake-optional encoding, and there is no real synchronous value to carry.
 */
export type SinkFlush = () => Promise<void>;

/**
 Verification function that checks if a sink backend is available.
 May run setup side effects (resolving a log path, opening a writable
 stream) and reports whether the backend is usable. A sink whose
 verification resolves `false` (or rejects) is dropped by the logger and
 receives no further records.
 
 Always async, matching `write` and `flush`: a synchronous check returns an
 already-resolved promise (`Promise.resolve(check)`) so the logger awaits
 verification uniformly with no sync/async branch.
 */
export type Verify = () => Promise<boolean>;

/**
 Sink that receives log records. A sink is a self-describing adapter: it
 carries everything the logger must know to use it, namely how to
 `verify` its backend is available, how to `write` a record, and
 optionally how to `flush` buffered work. Holding `verify` on the sink
 (rather than as a sibling export the logger pairs by hand) lets the
 logger treat a registry as a plain `Sink[]` and lets a test supply one
 self-contained fake.
 
 Sinks that buffer records (e.g. microtask-batched console) may
 expose a `flush` hook so callers can force emission on demand.
 
 `write` is always async: a synchronous sink does its work eagerly and
 returns an already-resolved promise, so the logger observes a uniform
 `Promise<void>`. A rejected write is handled per sink and does not
 disable the backend; only a failed `verify` drops a sink. A `void` arm
 is not used, for the reason stated on {@link SinkFlush}.
 */
export type Sink = {
  readonly flush?: SinkFlush;
  readonly verify: Verify;
  readonly write: (record: LogRecord,) => Promise<void>;
};

/**
 Logger interface with 6 log levels plus `flush` for startup and sink drains.
 `flush()` resolves once startup verification has completed, tracked sink
 writes have settled, and every available sink's own {@link SinkFlush} hook
 has settled. Safe to call even when no sink buffers.
 */
export type Logger = {
  readonly debug: (message: string,) => void;
  readonly error: (message: string,) => void;
  readonly fatal: (message: string,) => void;
  readonly flush: () => Promise<void>;
  readonly info: (message: string,) => void;
  readonly trace: (message: string,) => void;
  readonly warn: (message: string,) => void;
};
