import { chmod, lstat, mkdir } from 'node:fs/promises';
import { homedir } from 'node:os';
import { join, resolve } from 'node:path';

//region Resolve the same profile and forward the caller's arguments unchanged
const args = process.argv.slice(2);
const inlineProfile = args.find(argument => argument.startsWith('--user-data-dir='));
const profileOption = args.indexOf('--user-data-dir');
const separateProfile = profileOption < 0 ? undefined : args[profileOption + 1];
if (profileOption >= 0 && !separateProfile) {
  throw new Error('Helium --user-data-dir needs a profile directory.');
}
const configDirectory = process.env.XDG_CONFIG_HOME || join(homedir(), '.config');
const profile = resolve(inlineProfile?.slice('--user-data-dir='.length)
  ?? separateProfile ?? join(configDirectory, 'net.imput.helium'));
const metrics = join(profile, 'BrowserMetrics');
//endregion

//region Force Chromium's verified local-memory fallback for diagnostic histograms
// Chromium 156 initializes this mapping before feature flags are available.
// Its cleanup removes this directory, so prepare it on every launcher invocation.
await mkdir(profile, { recursive: true, mode: 0o700 });
await mkdir(metrics, { recursive: true, mode: 0o500 });
const metadata = await lstat(metrics);
if (!metadata.isDirectory() || metadata.isSymbolicLink()) {
  throw new Error(`Helium metrics workaround requires an ordinary directory: ${metrics}`);
}
await chmod(metrics, 0o500);
//endregion

//region Replace the helper so Helium keeps normal process and signal ownership
const executable = join(homedir(), 'AppImages', 'helium.appimage');
if (typeof process.execve !== 'function') {
  throw new Error('Helium metrics workaround requires the installed Node runtime with process.execve.');
}
process.execve(executable, [executable, ...args], process.env);
//endregion
