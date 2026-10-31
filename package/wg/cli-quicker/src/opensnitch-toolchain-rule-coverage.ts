/**
 Segment count of a rustup toolchain binary path below `toolchains/`:
 `<toolchain>/bin/<binary>`.
 */
const RUSTUP_TOOLCHAIN_PATH_SEGMENTS = 3;

/**
 Segment count of an exposure link path below its checkout or worktree root:
 `.cache/<directory>/<link>`.
 */
const EXPOSURE_PATH_SEGMENTS = 3;

/**
 Segment count of the prefix naming a linked worktree below the main checkout:
 `.claude/worktrees/<id>`.
 */
const WORKTREE_PATH_SEGMENTS = 3;

/**
 Binaries of the Cargo home's `bin` directory that update traffic runs through.
 */
const CARGO_HOME_BINARIES: ReadonlySet<string> = new Set([
  'cargo',
  'cargo-binstall',
  'rustup',
],);

/**
 Cache directories holding exposure links and launchers.
 */
const EXPOSURE_DIRECTORIES: ReadonlySet<string> = new Set([
  'toolchain-bin',
  'toolchain-launchers',
],);

/**
 Reports whether a path lies below `monochromatic-dev/toolchain/` or `monochromatic-dev/toolchain-global/`
 in any spelling of the data home.

 This is the plain-string twin of the install-roots daemon expression,
 so superseded rules are found without compiling a pattern.

 @param path - Executable path the daemon would compare.

 @param dataHomes - Spellings of the XDG data home.

 @returns Whether the install-roots rule covers the path.

 @example
 ```ts
 coversInstallRoot({ path: '/home/me/.local/share/monochromatic-dev/toolchain/bin/pnpm', dataHomes: ['/home/me/.local/share',] });
 ```
 */
export function coversInstallRoot(
  {
    path,
    dataHomes,
  }: {
    readonly path: string;
    readonly dataHomes: readonly string[];
  },
): boolean {
  return dataHomes.some(function underDataHome(dataHome,): boolean {
    return path.startsWith(`${dataHome}/monochromatic-dev/toolchain/`,)
      || path.startsWith(`${dataHome}/monochromatic-dev/toolchain-global/`,);
  },);
}

/**
 Reports whether a path is a file directly in `.cache/toolchain-bin` or `.cache/toolchain-launchers`
 of a checkout or of one of its linked worktrees.

 @param path - Executable path the daemon would compare.

 @param repositoryRoots - Spellings of the main checkout.

 @returns Whether the exposure-links rule covers the path.

 @example
 ```ts
 coversExposureLink({ path: '/home/me/repo/.cache/toolchain-bin/node', repositoryRoots: ['/home/me/repo',] });
 ```
 */
export function coversExposureLink(
  {
    path,
    repositoryRoots,
  }: {
    readonly path: string;
    readonly repositoryRoots: readonly string[];
  },
): boolean {
  return repositoryRoots.some(function underRoot(root,): boolean {
    if (!path.startsWith(`${root}/`,))
      return false;
    /**
     Segments below the checkout root.
     */
    const segments = path
      .slice(root.length + 1,)
      .split('/',);
    /**
     Whether the path starts inside a linked worktree directory.
     */
    const inWorktree = (segments[0] === '.claude')
      && (segments[1] === 'worktrees')
      && ((segments[2] ?? '') !== '');
    /**
     Segments below the checkout or worktree root.
     */
    const inner = inWorktree ? segments.slice(WORKTREE_PATH_SEGMENTS,) : segments;
    return (inner.length === EXPOSURE_PATH_SEGMENTS)
      && (inner[0] === '.cache')
      && EXPOSURE_DIRECTORIES.has(inner[1] ?? '',)
      && ((inner[2] ?? '') !== '');
  },);
}

/**
 Reports whether a path is a Cargo-home proxy or a rustup toolchain binary.

 @param path - Executable path the daemon would compare.

 @param cargoHomes - Spellings of the Cargo home.

 @param rustupHomes - Spellings of the rustup home.

 @returns Whether the Rust-tools rule covers the path.

 @example
 ```ts
 coversRustTool({ path: '/home/me/.cargo/bin/rustup', cargoHomes: ['/home/me/.cargo',], rustupHomes: [] });
 ```
 */
export function coversRustTool(
  {
    path,
    cargoHomes,
    rustupHomes,
  }: {
    readonly path: string;
    readonly cargoHomes: readonly string[];
    readonly rustupHomes: readonly string[];
  },
): boolean {
  if (cargoHomes.some(function isCargoBinary(cargoHome,): boolean {
    if (!path.startsWith(`${cargoHome}/bin/`,))
      return false;
    return CARGO_HOME_BINARIES.has(path.slice(cargoHome.length + '/bin/'.length,),);
  },))
    return true;
  return rustupHomes.some(function isToolchainBinary(rustupHome,): boolean {
    if (!path.startsWith(`${rustupHome}/toolchains/`,))
      return false;
    /**
     Segments below `toolchains/`.
     */
    const segments = path
      .slice(rustupHome.length + '/toolchains/'.length,)
      .split('/',);
    return (segments.length === RUSTUP_TOOLCHAIN_PATH_SEGMENTS)
      && ((segments[0] ?? '') !== '')
      && (segments[1] === 'bin')
      && ((segments[2] ?? '') !== '');
  },);
}
