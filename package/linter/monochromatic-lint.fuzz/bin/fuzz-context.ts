/**
 The build context of the instrumented fuzz build: package inputs, vendored dependencies and the fuzz tools.
 */
import { constants } from 'node:fs';
import {
  access,
  copyFile,
  cp,
  mkdir,
  stat,
  writeFile,
} from 'node:fs/promises';
import {
  delimiter,
  join,
  resolve,
} from 'node:path';

import { mapInSequence } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

// The generator controls run in the build image with the runner the git-policy/cli gate pins by version and checksum.
import {
  copyNextest,
  nextestCopyLines,
} from '../../../git-policy/cli/bin/native-nextest-snapshot.ts';
import { runCommand } from '../../../git-policy/cli/bin/native-verification-process.ts';
import {
  baseImage,
  FuzzVerificationError,
} from './fuzz-constants.ts';

/**
 A package whose inputs the build context carries.
 */
type PackageCopy = {
  readonly source: string;
  readonly destination: string;
  readonly fuzz: boolean;
};

/**
 Error codes that mean a PATH directory does not hold the executable.
 */
const missingCodes: ReadonlySet<string> = new Set([
  'ENOENT',
  'ENOTDIR',
  'EACCES',
]);

/**
 Check whether a failure only says that the directory holds no usable entry of that name.

 @param error - what a lookup threw

 @returns true for the error codes of a missing, mistyped or unreadable entry

 @example
 ```ts
 isMissingEntry(new Error('x'));
 ```
 */
function isMissingEntry(error: unknown): boolean {
  if (!Error.isError(error)) return false;
  return ('code' in error) && missingCodes.has(String(error.code));
}

/**
 Find the executable in one directory of PATH.

 @param directory - a directory of PATH

 @param name - the executable name

 @returns the path as the only item, or no item when the directory holds no executable file of that name

 @throws Error when the lookup fails for a reason other than a missing or unreadable entry

 @example
 ```ts
 const found = await findIn({ directory: '/usr/bin', name: 'cargo-fuzz' });
 ```
 */
async function findIn({
  directory,
  name,
}: {
  readonly directory: string;
  readonly name: string;
}): Promise<readonly string[]> {
  /**
   Where the executable would be.
   */
  const candidate = resolve(
    directory,
    name,
  );
  try {
    await access(
      candidate,
      constants.X_OK,
    );
    return (await stat(candidate)).isFile() ? [candidate] : [];
  } catch (error) {
    if (isMissingEntry(error)) return [];
    throw error;
  }
}

/**
 Resolve the actual PATH-selected tool rather than an older unrelated Cargo-home installation.

 @param name - the executable name

 @returns the first executable file of that name on PATH

 @throws FuzzVerificationError when no directory of PATH holds it

 @example
 ```ts
 const cargoFuzz = await executable('cargo-fuzz');
 ```
 */
async function executable(name: string): Promise<string> {
  /**
   The matches in PATH order.
   */
  const found = (await mapInSequence({
    items: (process.env
      .PATH
      ?? '').split(delimiter),
    run: function lookup(directory) {
      return findIn({
        directory,
        name,
      });
    },
  })).flat();
  /**
   The first match.
   */
  const [first] = found;
  if (first === undefined) throw new FuzzVerificationError(`Executable ${name} was not found on PATH.`);
  return first;
}

/**
 Copy only package inputs, never its caches, dependencies, or private machine-local files.

 @param source - the package directory

 @param destination - where the context receives it

 @param fuzz - true to copy the fuzz targets too

 @example
 ```ts
 await copyPackage({ source, destination, fuzz: false });
 ```
 */
async function copyPackage({
  source,
  destination,
  fuzz,
}: PackageCopy): Promise<void> {
  await mapInSequence({
    items: [
      'Cargo.toml',
      'Cargo.lock',
      'src',
      ...(fuzz ? ['fuzz_targets'] : []),
    ],
    run: function copyOf(name) {
      return cp(
        join(
          source,
          name,
        ),
        join(
          destination,
          name,
        ),
        { recursive: true },
      );
    },
  });
}

/**
 Copy the subject, the sibling crate and their fuzz sidecars, plus the fixtures their tests read.

 @param source - the fuzz sidecar directory, which is the working directory

 @param context - the build context

 @example
 ```ts
 await copyPackages({ source, context });
 ```
 */
async function copyPackages({
  source,
  context,
}: {
  readonly source: string;
  readonly context: string;
}): Promise<void> {
  /**
   The packages whose inputs the context carries.
   */
  const packages: readonly PackageCopy[] = [
    {
      source,
      destination: 'linter/monochromatic-lint.fuzz',
      fuzz: true,
    },
    {
      source: resolve(
        source,
        '../monochromatic-lint',
      ),
      destination: 'linter/monochromatic-lint',
      fuzz: false,
    },
    {
      source: resolve(
        source,
        '../../jsonc-edit/core-rust',
      ),
      destination: 'jsonc-edit/core-rust',
      fuzz: false,
    },
    {
      source: resolve(
        source,
        '../../jsonc-edit/core-rust.fuzz',
      ),
      destination: 'jsonc-edit/core-rust.fuzz',
      fuzz: true,
    },
  ];
  await mapInSequence({
    items: packages,
    run: function copyOf(item) {
      return copyPackage({
        source: item.source,
        destination: join(
          context,
          'package',
          item.destination,
        ),
        fuzz: item.fuzz,
      });
    },
  });
  await cp(
    resolve(
      source,
      '../monochromatic-lint/fixtures',
    ),
    join(
      context,
      'package/linter/monochromatic-lint/fixtures',
    ),
    { recursive: true },
  );
  await cp(
    resolve(
      source,
      '../../jsonc-edit/core-rust/fixtures',
    ),
    join(
      context,
      'package/jsonc-edit/core-rust/fixtures',
    ),
    { recursive: true },
  );
}

/**
 Vendor the locked dependencies offline and write the Cargo configuration that maps them into the image.

 @param context - the build context

 @throws FuzzVerificationError when Cargo does not print the expected source directory mapping

 @example
 ```ts
 await vendorDependencies(context);
 ```
 */
async function vendorDependencies(context: string): Promise<void> {
  /**
   Where Cargo places the vendored sources.
   */
  const vendorPath = join(
    context,
    'vendor',
  );
  /**
   What the offline vendoring printed.
   */
  const vendor = await runCommand({
    command: 'cargo',
    args: [
      'vendor',
      '--locked',
      '--offline',
      '--versioned-dirs',
      vendorPath,
    ],
    capture: true,
  });
  /**
   The Cargo configuration with the vendor path as the image sees it.
   */
  const config = vendor.stdout
    .replaceAll(
    JSON.stringify(vendorPath),
    '"/work/vendor"',
    );
  if (!config.includes('directory = "/work/vendor"'))
    throw new FuzzVerificationError('Cargo vendor did not emit the expected source directory mapping.');
  await mkdir(
    join(
      context,
      '.cargo',
    ),
    { recursive: true },
  );
  await writeFile(
    join(
      context,
      '.cargo/config.toml',
    ),
    config,
  );
}

/**
 Containerfile of the instrumented build image.

 @returns the text of the Containerfile

 @example
 ```ts
 const text = buildContainerfile();
 ```
 */
function buildContainerfile(): string {
  return [
    '# Same installed Rust runtime image as the subject container tests; compiler is a separate read-only input.',
    `FROM ${baseImage}`,
    'COPY package /work/package',
    'COPY vendor /work/vendor',
    'COPY .cargo /work/.cargo',
    'COPY cargo-fuzz /usr/local/cargo/bin/cargo-fuzz',
    'RUN ["cargo-fuzz", "--version"]',
    ...nextestCopyLines(),
    'WORKDIR /work/package/linter/monochromatic-lint.fuzz',
    // The optimized HIR ASAN build was SIGKILLed under the 2 GiB cap with two compiler jobs.
    'ENV CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0',
    'ENV PATH="/toolchain/bin:/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"',
    'ENV RUSTC="/toolchain/bin/rustc" RUSTDOC="/toolchain/bin/rustdoc"',
    '',
  ].join('\n');
}

/**
 Fill the build context: package inputs, vendored dependencies, cargo-fuzz, the nextest runner and the Containerfile.

 @param source - the fuzz sidecar directory, which is the working directory

 @param context - the build context to fill

 @throws FuzzVerificationError when Cargo vendoring or the tool lookup fails

 @example
 ```ts
 await prepareBuildContext({ source: process.cwd(), context });
 ```
 */
export async function prepareBuildContext({
  source,
  context,
}: {
  readonly source: string;
  readonly context: string;
}): Promise<void> {
  await copyPackages({
    source,
    context,
  });
  await vendorDependencies(context);
  await copyFile(
    await executable('cargo-fuzz'),
    join(
      context,
      'cargo-fuzz',
    ),
  );
  await copyNextest({ context });
  await writeFile(
    join(
      context,
      'Containerfile',
    ),
    buildContainerfile(),
  );
}
