/**
 What a scenario may do to its commit service.

 @module
 */

//region Types

/**
 One scenario's commit service and what a scenario may do to it.
 */
export type ScenarioService = Readonly<{
  /**
   Runtime directory the wrappers of the scenario find the service in (`XDG_RUNTIME_DIR`).
   */
  runtime: string;
  /**
   Process ID of the running service.
   */
  pid: () => number;
  /**
   Whether the service process has ended.
   */
  hasExited: () => boolean;
  /**
   Everything every service process of the scenario wrote to its standard error, oldest first.
   */
  log: () => string;
  /**
   Freezes the service with `SIGSTOP`.
   A client that connects meanwhile is accepted by the kernel but receives nothing, so its prepared commit waits.
   */
  stall: () => void;
  /**
   Lets a frozen service run again.
   */
  resume: () => void;
  /**
   Kills the service with `SIGKILL` and waits until it is gone.
   */
  kill: () => Promise<void>;
  /**
   Starts a service after the previous one ended, the way the service manager restarts a crashed unit:
   the stale socket file is removed first, because the service itself does not replace one.
   The new process is clean unless a fault point arms it.
   */
  restart: (fault?: string) => Promise<void>;
  /**
   Waits until the current service ends, then restarts it after a short outage.
   */
  restartAfterExit: () => Promise<void>;
  /**
   Restarts the service when it has ended, so the observation finds one to finish the after-commit work.
   */
  ensureRunning: () => Promise<void>;
}>;

//endregion Types
