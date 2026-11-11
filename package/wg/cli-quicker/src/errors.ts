/**
 Error raised when command-line arguments do not satisfy the command contract.
 
 @example
 ```ts
 throw new CliUsageError('Missing required subcommand: up');
 ```
 */
export class CliUsageError extends Error {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'CliUsageError';
}

/**
 Error raised when a config file cannot represent a WireGuard interface.
 
 @example
 ```ts
 throw new ConfigError('Config file does not exist: /etc/wireguard/mx.conf');
 ```
 */
export class ConfigError extends Error {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'ConfigError';
}

/**
 Error raised when CLI cannot enter or complete privileged execution.
 
 @example
 ```ts
 throw new PrivilegeError('Unable to start sudo.');
 ```
 */
export class PrivilegeError extends Error {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'PrivilegeError';
}

/**
 Error raised when application bypass state is missing,
 corrupt,
 or unsafe to mutate.
 
 @example
 ```ts
 throw new BypassStateError('Bypass state does not match interface.');
 ```
 */
export class BypassStateError extends Error {
  /**
   Stable error type name.
   */
  override name = 'BypassStateError';
}

/**
 Error raised when no physical path can carry exempt traffic.
 
 @example
 ```ts
 throw new BypassRouteError('No physical default route exists.');
 ```
 */
export class BypassRouteError extends Error {
  /**
   Stable error type name.
   */
  override name = 'BypassRouteError';
}

/**
 Error raised when OpenSnitch system-firewall configuration cannot be reconciled safely.
 
 @example
 ```ts
 throw new OpenSnitchConfigError('OpenSnitch firewall config is malformed.');
 ```
 */
export class OpenSnitchConfigError extends Error {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'OpenSnitchConfigError';
}

/**
 Error raised when active OpenSnitch daemon does not converge to persisted rules.
 
 @example
 ```ts
 throw new OpenSnitchLiveReloadError('OpenSnitch live chain is incomplete.');
 ```
 */
export class OpenSnitchLiveReloadError extends OpenSnitchConfigError {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'OpenSnitchLiveReloadError';
}

/**
 Error raised when existing policy routing would override tunnel selection.
 
 @example
 ```ts
 throw new PolicyRoutingConflictError('Conflicting packet mark is active.');
 ```
 */
export class PolicyRoutingConflictError extends Error {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'PolicyRoutingConflictError';
}

/**
 Error raised when a config sets `ExemptMark` and the `wg-quicker-exempt` program cannot be found
 in any root-controlled location,
 so the start stops and is rolled back.

 @example
 ```ts
 throw new WatcherMissingError({ message: 'not found', searched: ['/usr/bin/wg-quicker-exempt'] });
 ```
 */
export class WatcherMissingError extends Error {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'WatcherMissingError';

  /**
   Every root-controlled location that was looked at.
   */
  readonly searched: readonly string[];

  /**
   Creates the error with the words for the person and the places searched.

   @param message - What was not found,
   what was done about it,
   and how to install the program.

   @param searched - Every location that was looked at.

   @example
   ```ts
   new WatcherMissingError({ message: 'not found', searched: [] });
   ```
   */
  public constructor(
    {
      message,
      searched,
    }: {
      readonly message: string;
      readonly searched: readonly string[];
    },
  ) {
    super(message,);
    this.searched = searched;
  }
}

/**
 Error raised when an external command's program could not be started,
 for example because it is not installed.
 The command never ran,
 so there is no exit code to report.

 @example
 ```ts
 throw new CommandSpawnError({ command: 'wg', args: ['addconf'], reason: 'spawn wg ENOENT' });
 ```
 */
export class CommandSpawnError extends Error {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'CommandSpawnError';

  /**
   Executable whose program could not be started.
   */
  readonly command: string;

  /**
   Arguments the executable would have received.
   */
  readonly args: readonly string[];

  /**
   Creates the error naming the program and the system's reason.

   @param command - Executable that could not be started.

   @param args - Arguments the executable would have received.

   @param reason - What the system said, such as `spawn wg ENOENT`.

   @example
   ```ts
   new CommandSpawnError({ command: 'wg', args: [], reason: 'spawn wg ENOENT' });
   ```
   */
  public constructor(
    {
      command,
      args,
      reason,
    }: {
      readonly command: string;
      readonly args: readonly string[];
      readonly reason: string;
    },
  ) {
    /**
     Fresh copy of the argument list so joining never reads a caller-owned array.
     */
    const rendered: readonly string[] = [
      ...args,
    ];
    super(`Command could not be started: ${command} ${rendered.join(' ',)}\n${reason}`,);
    this.command = command;
    this.args = rendered;
  }
}

/**
 Error raised when an external command was still running at its deadline and was ended,
 so the caller can report the call as unfinished instead of waiting for ever.

 @example
 ```ts
 throw new CommandTimeoutError({ command: 'nft', args: ['list', 'ruleset'], deadlineMilliseconds: 60_000 });
 ```
 */
export class CommandTimeoutError extends Error {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'CommandTimeoutError';

  /**
   Executable that was ended.
   */
  readonly command: string;

  /**
   Arguments the executable received.
   */
  readonly args: readonly string[];

  /**
   The deadline that passed, in milliseconds.
   */
  readonly deadlineMilliseconds: number;

  /**
   Creates the error naming the command, its arguments and the deadline it passed.

   @param command - Executable that was ended.

   @param args - Arguments the executable received.

   @param deadlineMilliseconds - The deadline that passed.

   @example
   ```ts
   new CommandTimeoutError({ command: 'nft', args: [], deadlineMilliseconds: 60_000 });
   ```
   */
  public constructor(
    {
      command,
      args,
      deadlineMilliseconds,
    }: {
      readonly command: string;
      readonly args: readonly string[];
      readonly deadlineMilliseconds: number;
    },
  ) {
    /**
     Fresh copy of the argument list so joining never reads a caller-owned array.
     */
    const rendered: readonly string[] = [
      ...args,
    ];
    super(
      `Command was still running after ${String(deadlineMilliseconds,)} ms and was ended: ${command} ${rendered.join(' ',)}`,
    );
    this.command = command;
    this.args = rendered;
    this.deadlineMilliseconds = deadlineMilliseconds;
  }
}

/**
 Error raised when an external `ip`, `wg`, or `resolvectl` command exits non-zero.
 
 @example
 ```ts
 throw new CommandError({
   command: 'ip',
   args: ['link', 'add'],
   exitCode: 2,
   stderr: 'RTNETLINK answers: File exists',
 });
 ```
 */
export class CommandError extends Error {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'CommandError';

  /**
   Numeric exit code reported by the failed command.
   */
  readonly exitCode: number;

  /**
   Standard error emitted by the failed command.
   */
  readonly stderr: string;

  /**
   Creates a command failure carrying exit code and captured stderr.
   
   @param command - Executable that failed.
   
   @param args - Arguments the executable received.
   
   @param exitCode - Numeric exit code the executable reported.
   
   @param stderr - Standard error the executable emitted.
   
   @example
   ```ts
   new CommandError({ command: 'ip', args: ['link'], exitCode: 2, stderr: 'x' });
   ```
   */
  public constructor(
    {
      command,
      args,
      exitCode,
      stderr,
    }: {
      readonly command: string;
      readonly args: readonly string[];
      readonly exitCode: number;
      readonly stderr: string;
    },
  ) {
    /**
     Fresh copy of the argument list so joining never reads a caller-owned array.
     */
    const rendered: readonly string[] = [
      ...args,
    ];
    super(
      `Command failed (${String(exitCode,)}): ${command} ${rendered.join(' ',)}\n${stderr}`,
    );
    this.exitCode = exitCode;
    this.stderr = stderr;
  }
}

/**
 Error raised when `down` stopped before it changed anything,
 because a step that can fail (stopping a watcher, reading state) failed first.
 The tunnel is still fully up.

 @example
 ```ts
 throw new TeardownRefusedError('down wg0 stopped before changing anything');
 ```
 */
export class TeardownRefusedError extends Error {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'TeardownRefusedError';
}

/**
 Error raised when `down` failed after it started removing routing.
 Traffic is blocked when the block could be installed,
 and the message says how to finish.

 @example
 ```ts
 throw new TeardownIncompleteError('down wg0 failed part way');
 ```
 */
export class TeardownIncompleteError extends Error {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'TeardownIncompleteError';
}

/**
 Error raised when the traffic block of an unfinished `down` is still in place, or cannot be shown gone,
 after the attempt to remove it.
 While the block stands,
 everything that leaves the machine is dropped,
 except the clearnet traffic it was installed to let through,
 so the message names the nft table and the command that removes it.

 @example
 ```ts
 throw new TrafficBlockError('the traffic block is still in place');
 ```
 */
export class TrafficBlockError extends Error {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'TrafficBlockError';
}

/**
 Error raised when the traffic block of a `down` cannot learn what the routing sends around the tunnel,
 because the state of the application bypass or the kernel's rules cannot be read.
 A block built without that knowledge would cut traffic the routing sends around the tunnel by design,
 so the `down` stops with the tunnel still up.
 It carries a marker property instead of relying on `instanceof`,
 so it is recognized across separately built bundles.

 @example
 ```ts
 throw new ClearnetUnreadableError('the state file is damaged');
 ```
 */
export class ClearnetUnreadableError extends Error {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'ClearnetUnreadableError';

  /**
   Marker read by {@link isClearnetUnreadable}.
   */
  readonly clearnetUnreadable = true as const;
}

/**
 Tells whether a caught value is the error of a block that could not read what to let through.

 @param error - Caught value.

 @returns Whether it carries the marker.

 @example
 ```ts
 isClearnetUnreadable({ error: new ClearnetUnreadableError('x') }); // true
 ```
 */
export function isClearnetUnreadable({ error, }: { readonly error: unknown; },): boolean {
  return ((typeof error) === 'object')
    && (error !== null)
    && ('clearnetUnreadable' in error)
    && (error.clearnetUnreadable === true);
}

/**
 Error raised when the resolver does not finish answering within the time limit.
 It carries a marker property instead of relying on `instanceof`,
 so it is recognized across separately built bundles.

 @example
 ```ts
 throw new ResolverTimeoutError('the resolver did not answer');
 ```
 */
export class ResolverTimeoutError extends Error {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'ResolverTimeoutError';

  /**
   Marker read by {@link isResolverTimeout}.
   */
  readonly resolverTimeout = true as const;
}

/**
 Tells whether a caught value is a resolver timeout.

 @param error - Caught value.

 @returns Whether it carries the timeout marker.

 @example
 ```ts
 isResolverTimeout({ error: new ResolverTimeoutError('x') }); // true
 ```
 */
export function isResolverTimeout({ error, }: { readonly error: unknown; },): boolean {
  return ((typeof error) === 'object')
    && (error !== null)
    && ('resolverTimeout' in error)
    && (error.resolverTimeout === true);
}

/**
 Narrows a caught value to a Node error carrying an errno code.

 Node's filesystem and process functions throw real `Error` instances whose
 `code` names the condition, so the code is read only from an error: a bare
 object shaped like an errno comes from some other thrower, and reading its
 `code` as a missing file would swallow it.

 @param error - Caught value.

 @returns Whether value is an error carrying a code.

 @example
 ```ts
 isErrnoException(Object.assign(new Error('missing'), { code: 'ENOENT' })); // true
 ```
 */
export function isErrnoException(error: unknown,): error is NodeJS.ErrnoException {
  return Error.isError(error,)
    && ('code' in error);
}
