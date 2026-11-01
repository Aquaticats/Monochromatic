/**
 Vendoring of the native wrapper's locked registry dependencies, which the wrapper and the crates it links pull in.

 The container has no network, so registry dependencies are vendored on the host from the Cargo cache
 and baked into the image beside the copied path crates (`native-wrapper-snapshot.ts` lists those).
 */
import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

/**
 Image path of the vendored registry sources; the Containerfile copies `vendor` there.
 */
const imageVendorDirectory = '/work/vendor';

/**
 Vendor the copied wrapper's locked registry dependencies from the host's Cargo cache into the build context,
 and write the Cargo source mapping the image reads them through.
 Every archive the lockfile names must already be in that cache; Cargo fails the gate otherwise.
 Another crate of the snapshot, such as the linter the Markdown policy runs, passes its own manifest and output
 names, so each image stage reads only the sources its own lockfile names.

 @param context - build context holding the copied crates

 @param manifest - manifest under the context whose lockfile is vendored (the wrapper's by default)

 @param vendorName - directory name receiving the sources (`vendor` by default)

 @param configName - directory name receiving the mapping (`cargo-config` by default)

 @example
 ```ts
 await vendorLockedDependencies({ context });
 ```
 */
export async function vendorLockedDependencies({
  context,
  manifest = 'package/git-policy/cli/Cargo.toml',
  vendorName = 'vendor',
  configName = 'cargo-config',
}: {
  readonly context: string;
  readonly manifest?: string;
  readonly vendorName?: string;
  readonly configName?: string;
}): Promise<void> {
  /**
   Host directory that receives the vendored registry sources.
   */
  const vendor = join(
    context,
    vendorName,
  );
  /**
   Result of `cargo vendor`, whose standard output is the source mapping for the host path.
   */
  const vendored = await runCommand({
    command: 'cargo',
    args: [
      'vendor',
      '--manifest-path',
      join(
        context,
        manifest,
      ),
      '--offline',
      '--locked',
      '--versioned-dirs',
      vendor,
    ],
    capture: true,
  });
  // Cargo prints the mapping for the host path; the image holds the same directory elsewhere.
  /**
   The mapping rewritten to the image's vendor directory.
   */
  const mapping = vendored.stdout
    .replaceAll(
    JSON.stringify(vendor),
    JSON.stringify(imageVendorDirectory),
    );
  if (!mapping.includes(`directory = ${JSON.stringify(imageVendorDirectory)}`))
    throw new NativeVerificationError('Cargo vendor did not provide the expected offline source mapping.');
  /**
   Host directory that receives the Cargo configuration file holding the mapping.
   */
  const configuration = join(
    context,
    configName,
  );
  await mkdir(
    configuration,
    { recursive: true },
  );
  await writeFile(
    join(
      configuration,
      'config.toml',
    ),
    mapping,
  );
}
