import { chmod, lstat, mkdir, writeFile } from 'node:fs/promises';
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
// Chromium 156 and 157 initialize this mapping before feature flags are available.
// Helium deletes an empty BrowserMetrics directory within seconds of startup.
// The file inside keeps the read-only directory in place, so later starts that skip
// this launcher (an in-browser relaunch, web-app shortcuts, a terminal `helium-browser`)
// still take the fallback.
const keepFile = join(metrics, 'read-only-so-helium-keeps-diagnostic-metrics-in-memory.txt');
const keepFileText = `Helium writes diagnostic histograms into a memory-mapped file in this folder.
Writes to that mapping can stall the browser's main thread while the disk catches up.
This folder is read-only on purpose, so Chromium keeps those histograms in memory instead.
This file stops Helium from deleting the empty folder at startup.
Set up by ${join(homedir(), '.local', 'lib', 'helium', 'launcher.ts')}.
Details: doc/troubleshooting/helium-window-freezes.md in the Monochromatic repository.
To undo: make this folder writable again (chmod u+w), delete it, and start Helium normally.
`;
await mkdir(profile, { recursive: true, mode: 0o700 });
await mkdir(metrics, { recursive: true, mode: 0o700 });
const metadata = await lstat(metrics);
if (!metadata.isDirectory() || metadata.isSymbolicLink()) {
  throw new Error(`Helium metrics workaround requires an ordinary directory: ${metrics}`);
}
await chmod(metrics, 0o700);
await writeFile(keepFile, keepFileText);
await chmod(metrics, 0o500);
//endregion

//region Replace the helper so Helium keeps normal process and signal ownership
// The Arch `helium-browser-bin` wrapper, not `/opt/helium-browser-bin/helium` itself:
// the wrapper still reads `helium-browser-flags.conf` and sets `LD_LIBRARY_PATH`.
const executable = '/usr/bin/helium-browser';
if (typeof process.execve !== 'function') {
  throw new Error('Helium metrics workaround requires the installed Node runtime with process.execve.');
}
process.execve(executable, [executable, ...args], process.env);
//endregion
