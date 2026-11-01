import spawn from 'nano-spawn';

/**
 Emits a syslog message via the `logger` CLI utility.
 Fire-and-forget: the spawn promise is discarded so logging never blocks
 the main flow.
 
 @param priority - syslog priority level (e.g. "debug", "info", "warning", "err")
 
 @param message - log message body
 
 @example
 ```ts
 emit({ priority: "info", message: "capture cycle complete" });
 ```
 */
function emit(
  {
    priority,
    message,
  }: {
    readonly priority: string;
    readonly message: string;
  },
): void {
  // Fire-and-forget: ignore spawn errors to avoid blocking the main flow
  void spawn(
    'logger',
    [
      '-t',
      'hall-monitor',
      '-p',
      `user.${priority}`,
      '--',
      message,
    ],
  );
}

/**
 Logs a message at debug priority.
 
 @param msg - log message body
 */
function logDebug(msg: string,): void {
  emit({
    priority: 'debug',
    message: msg,
  },);
}

/**
 Logs a message at info priority.
 
 @param msg - log message body
 */
function logInfo(msg: string,): void {
  emit({
    priority: 'info',
    message: msg,
  },);
}

/**
 Logs a message at warning priority.
 
 @param msg - log message body
 */
function logWarn(msg: string,): void {
  emit({
    priority: 'warning',
    message: msg,
  },);
}

/**
 Logs a message at err priority.
 
 @param msg - log message body
 */
function logError(msg: string,): void {
  emit({
    priority: 'err',
    message: msg,
  },);
}

/**
 Syslog-backed logger for hall-monitor.
 All output is tagged with `hall-monitor` and routed through `logger(1)`,
 viewable via `journalctl -t hall-monitor -f`.
 
 @example
 ```ts
 log.info("[cycle] capture complete");
 log.error("[cycle] ffmpeg failed");
 ```
 */
export const log: {
  /**
   Log at debug priority.
   */
  debug: (msg: string,) => void;
  /**
   Log at info priority.
   */
  info: (msg: string,) => void;
  /**
   Log at warning priority.
   */
  warn: (msg: string,) => void;
  /**
   Log at err priority.
   */
  error: (msg: string,) => void;
} = {
  debug: logDebug,
  info: logInfo,
  warn: logWarn,
  error: logError,
};
