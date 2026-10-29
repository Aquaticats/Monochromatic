/**
 Options for the {@link matrix} function.
 
 Defines the axes of the cartesian product to execute:
 `files x os x user x runtime`, minus `exclude` matches.
 
 @example
 ```ts
 await matrix({
   os: ['container:ubuntu', 'container:fedora'],
   user: ['root', 'user'],
   runtime: ['bun'],
 });
 ```
 */
export type MatrixOptions = {
  /**
   Files to execute inside each environment.
   Self-executing scripts run with the selected runtime.
   Defaults to discovering `*.unit.matrix.test.ts` in the calling package.
   */
  readonly files?: readonly string[];

  /**
   Environments. Protocol prefix selects the backend.
   - `container:`: podman (implemented)
   - `host:`: run directly on the host (no container, runtime must be pre-installed)
   - `vm:`: mvm (not yet implemented, throws)
   
   @example `['container:ubuntu', 'container:fedora']`
   
   @example `['host:']`
   */
  readonly os: readonly string[];

  /**
   User contexts inside each environment.
   - `'root'`: run as root
   - `'user'`: create a non-root user with passwordless sudo
   
   Defaults to `['root']`.
   */
  readonly user?: readonly UserContext[];

  /**
   JS runtimes to install and execute files with.
   Defaults to `['bun']`.
   */
  readonly runtime?: readonly Runtime[];

  /**
   Exclude specific combinations from the cartesian product.
   Each entry is a partial match; all specified fields must match to exclude.
   
   @example
   ```ts
   exclude: [
     { os: 'container:alpine', user: 'user' },
   ]
   ```
   */
  readonly exclude?: readonly ExcludeEntry[];

  /**
   Maximum number of combinations to run concurrently.
   Defaults to `4`. Set to `1` for sequential execution.
   */
  readonly concurrency?: number;
};

/**
 User context inside a container.
 - `'root'`: run as root (uid 0)
 - `'user'`: create a non-root user (uid 1000) with passwordless sudo
 */
export type UserContext = 'root' | 'user';

/**
 JS runtime to install and execute test files with.
 - `'bun'`: installed via `curl -fsSL https://bun.sh/install | bash`
 - `'deno'`: installed via `curl -fsSL https://deno.land/install.sh | sh`
 - `'node'`: the release `toolchain.lock.json` pins, fetched and SHA-256
   checked against that lock and extracted under `$HOME/node` at run time
 */
export type Runtime = 'bun' | 'deno' | 'node';

/**
 Matcher for a single exclude field.
 - String: exact match against the combination value
 - Function: predicate that returns `true` when the combination should be excluded
 */
export type ExcludeMatcher<T,> = T | ((value: T,) => boolean);

/**
 Partial match for excluding combinations from the cartesian product.
 All specified fields must match a combination for it to be excluded.
 Each field accepts either an exact value or a predicate function.
 
 @example
 ```ts
 // Exact match
 { os: 'container:alpine', user: 'user' }
 
 // Predicate: exclude all non-root users on host
 {
   os: 'host:',
   user: function notRoot(user) { return user !== 'root'; },
 }
 ```
 */
export type ExcludeEntry = {
  readonly os?: ExcludeMatcher<string>;
  readonly user?: ExcludeMatcher<UserContext>;
  readonly runtime?: ExcludeMatcher<Runtime>;
  readonly file?: ExcludeMatcher<string>;
};

/**
 Single combination in the cartesian product, fully resolved.
 Represents one execution: one file, one OS, one user context, one runtime.
 */
export type Combination = {
  /**
   Absolute path to the test file on the host.
   */
  readonly file: string;
  /**
   OS specification with protocol prefix (e.g. `container:ubuntu`).
   */
  readonly os: string;
  /**
   User context inside the container.
   */
  readonly user: UserContext;
  /**
   JS runtime to execute the file with.
   */
  readonly runtime: Runtime;
};

/**
 Supported package managers, derived from distro names.
 - `apt`: Debian, Ubuntu
 - `dnf`: Fedora, RHEL, CentOS
 - `apk`: Alpine
 - `pacman`: Arch, CachyOS
 */
export type PackageManager = 'apt' | 'dnf' | 'apk' | 'pacman';

/**
 Parsed OS specification, split into protocol and distro name.
 */
export type ParsedOs = {
  /**
   Backend protocol: `'container'` (podman), `'host'` (direct execution), or `'vm'` (mvm, not implemented).
   */
  readonly protocol: 'container' | 'host' | 'vm';
  /**
   Distro name used as the container image tag (e.g. `'ubuntu'`, `'fedora'`). Empty string for `host:`.
   */
  readonly distro: string;
};
