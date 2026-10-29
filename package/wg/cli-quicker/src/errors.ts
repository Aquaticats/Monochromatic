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
