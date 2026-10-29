#!/usr/bin/env node
// Check one single application executable: the `bundle` task's output, or any copy of it.
// The checked file is only read. It is compared with the runtime directory it was built from, and its startup
// checks run copies of it, each alone in a folder below the private agent scratch root, hosted in the repository's
// nested compositor, on a one-file project whose language configures no server.
import { spawn, spawnSync } from 'node:child_process';
import { once } from 'node:events';
import { chmodSync, copyFileSync, existsSync, lstatSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, readlinkSync, realpathSync, rmSync, statSync, utimesSync, writeFileSync } from 'node:fs';
import { createConnection } from 'node:net';
import { inflateSync } from 'node:zlib';
import { homedir, tmpdir } from 'node:os';
import { join, resolve, sep } from 'node:path';
import { setTimeout as wait } from 'node:timers/promises';
import { compositor, demandCurrentCompositor } from './nested-compositor.ts';

const names = ['inventory', 'license-texts', 'lone-copy-highlights', 'later-start-reuses-cache', 'damaged-cache-rebuilt', 'shadowing-query-ignored', 'concurrent-first-starts', 'damaged-embedded-part-reported', 'old-cache-folders-removed', 'crate-licenses'];
const only = process.env.usage_only ? new Set(process.env.usage_only.split(',')) : undefined;
for (const name of only ?? []) if (!names.includes(name)) throw new Error('Unknown check name ' + name + '; the checks are ' + names.join(', '));
const executable = realpathSync(resolve(process.env.usage_file || 'dist/monochromatic-ide'));
const runtime = realpathSync(resolve(process.env.usage_runtime || 'target/release/runtime'));
const source = realpathSync(process.cwd());
// The notices task writes the crate license list and the linked-crates task the linked crate list beside the
// profile folders, so both sit two levels above the runtime directory.
const targetDirectory = resolve(runtime, '..', '..');
const crateLicensesFile = join(targetDirectory, 'crate-licenses.json');
const linkedCratesFile = join(targetDirectory, 'linked-crates.txt');
const privateRoot = join(homedir(), 'temp', 'agent');
if ((statSync(privateRoot).mode & 0o077) !== 0) throw new Error('Scratch root must exclude group and other permissions: ' + privateRoot);
const artifact = realpathSync(mkdtempSync(join(privateRoot, 'ide-bundle-check-')));
console.log('BUNDLE_CHECK_ARTIFACT=' + artifact);
console.log('BUNDLE_CHECK_FILE=' + executable);
console.log('BUNDLE_CHECK_RUNTIME=' + runtime);

const demand = ({ condition, message }: { condition: unknown; message: string | undefined }): void => { if (!condition) throw new Error(message); };
const isFile = (path: string): boolean => { try { return lstatSync(path).isFile(); } catch { return false; } };
const isDirectory = (path: string): boolean => { try { return lstatSync(path).isDirectory(); } catch { return false; } };
// Every regular file below a directory, as paths relative to it, sorted.
const filesBelow = (directory: string): string[] => readdirSync(directory, { withFileTypes: true, recursive: true })
  .filter(entry => entry.isFile()).map(entry => join(entry.parentPath, entry.name).slice(directory.length + 1)).sort();
// The grammar library names the manifest lists, validated the way the build script reads them.
const listedGrammars = (): { revision: unknown; libraries: string[] } => {
  const manifest = JSON.parse(readFileSync(join(runtime, 'manifest.json'), 'utf8'));
  demand({ condition: Array.isArray(manifest.grammars) && manifest.grammars.length > 0, message: 'runtime/manifest.json lists no grammar' });
  for (const library of manifest.grammars) demand({ condition: typeof library === 'string' && /^[A-Za-z0-9_-]+\.so$/.test(library), message: 'runtime/manifest.json entry is not a grammar library name: ' + library });
  return { revision: manifest.helixRevision, libraries: manifest.grammars };
};
// Every file the build script embeds, as [path in the embedded layout, file on disk]: the same list as build.rs.
const expectedFiles = (): [string, string][] => {
  const { libraries } = listedGrammars();
  const files: [string, string][] = [['runtime/manifest.json', join(runtime, 'manifest.json')], ['runtime/Helix-LICENSE', join(runtime, 'Helix-LICENSE')]];
  for (const relative of filesBelow(join(runtime, 'queries'))) files.push(['runtime/queries/' + relative, join(runtime, 'queries', relative)]);
  for (const library of libraries) {
    const name = library.slice(0, -3);
    files.push(['runtime/grammars/' + library, join(runtime, 'grammars', library)]);
    for (const relative of filesBelow(join(runtime, 'licenses', name))) files.push(['runtime/licenses/' + name + '/' + relative, join(runtime, 'licenses', name, relative)]);
  }
  for (const relative of filesBelow(join(source, 'LICENSES'))) files.push(['LICENSES/' + relative, join(source, 'LICENSES', relative)]);
  for (const notice of ['Inter-LICENSE.txt', 'JetBrainsMono-OFL.txt']) files.push(['LICENSES/font/' + notice, join(source, 'asset', 'font', notice)]);
  files.push(['LICENSES/crates.json', crateLicensesFile]);
  return files;
};
let binaryBytes: Buffer | undefined;
const binary = (): Buffer => (binaryBytes ??= readFileSync(executable));
// The embedded paths whose bytes do not occur in the executable exactly as in the runtime directory.
const notEmbedded = (paths: string[]): string[] => {
  const expected = new Map(expectedFiles());
  return paths.filter(path => {
    const file = expected.get(path);
    if (file === undefined) throw new Error('No build input is expected for the embedded path ' + path);
    const bytes = readFileSync(file);
    return bytes.length > 0 && binary().indexOf(bytes) < 0;
  });
};

// One check returns the numbers it measured, or throws what was wrong.
type Check = () => Promise<Record<string, unknown>>;
const checks: Record<string, Check> = {
  // A regular ELF executable that carries every runtime file and license text byte for byte.
  inventory: async () => {
    demand({ condition: isFile(executable), message: executable + ' is missing or not a regular file' });
    demand({ condition: (lstatSync(executable).mode & 0o100) !== 0, message: executable + ' is not executable' });
    demand({ condition: binary().subarray(0, 4).equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46])), message: executable + ' is not an ELF executable' });
    const { revision, libraries } = listedGrammars();
    demand({ condition: typeof revision === 'string' && /^[0-9a-f]{40}$/.test(revision), message: 'runtime/manifest.json does not name the Helix revision' });
    const files = expectedFiles();
    const missing = notEmbedded(files.map(([path]) => path));
    demand({ condition: missing.length === 0, message: 'the executable does not embed ' + missing.length + ' of ' + files.length + ' runtime files byte for byte: ' + missing.slice(0, 8).join(', ') });
    const embeddedBytes = files.reduce((sum, [, path]) => sum + statSync(path).size, 0);
    const highlightRules = files.filter(([path]) => path.endsWith('/highlights.scm')).length;
    return { size: statSync(executable).size, embeddedFiles: files.length, embeddedBytes, grammars: libraries.length, highlightRules, helixRevision: revision };
  },
  // The application's licenses, the font notices, Helix's license, and one notice per grammar, each embedded.
  'license-texts': async () => {
    const expected: [string, string][] = [
      ['LICENSES/LGPL-3.0-or-later.txt', 'GNU LESSER GENERAL PUBLIC LICENSE'],
      ['LICENSES/GPL-3.0-or-later.txt', 'GNU GENERAL PUBLIC LICENSE'],
      ['LICENSES/font/Inter-LICENSE.txt', 'SIL OPEN FONT LICENSE'],
      ['LICENSES/font/JetBrainsMono-OFL.txt', 'SIL OPEN FONT LICENSE'],
      ['runtime/Helix-LICENSE', 'Mozilla Public License Version 2.0'],
    ];
    const files = new Map(expectedFiles());
    // The build input file behind an embedded path; a path the build does not list is as good as a missing file.
    const inputFile = (relative: string): string => {
      const file = files.get(relative);
      if (file === undefined) throw new Error(relative + ' is missing from the build inputs');
      return file;
    };
    for (const [relative, phrase] of expected) {
      demand({ condition: files.has(relative) && isFile(inputFile(relative)), message: relative + ' is missing from the build inputs' });
      demand({ condition: readFileSync(inputFile(relative), 'utf8').includes(phrase), message: relative + ' does not contain "' + phrase + '"' });
    }
    for (const relative of ['LICENSES/font/Inter-LICENSE.txt', 'LICENSES/font/JetBrainsMono-OFL.txt']) {
      demand({ condition: /^Copyright .*Project Authors/m.test(readFileSync(inputFile(relative), 'utf8')), message: relative + ' lacks its copyright line' });
    }
    const { libraries } = listedGrammars();
    const notices: string[] = [];
    for (const library of libraries) {
      const name = library.slice(0, -3);
      const directory = join(runtime, 'licenses', name);
      demand({ condition: isDirectory(directory), message: 'runtime/licenses/' + name + ' is missing' });
      const texts = filesBelow(directory).filter(file => statSync(join(directory, file)).size > 0);
      demand({ condition: texts.length > 0, message: 'runtime/licenses/' + name + ' holds no notice' });
      demand({ condition: texts.some(file => /copyright/i.test(readFileSync(join(directory, file), 'utf8'))), message: 'runtime/licenses/' + name + ' holds no copyright notice' });
      // A REUSE-layout source keeps its copyright line only in source headers, which the runtime task extracts.
      if (isDirectory(join(directory, 'LICENSES'))) {
        const headers = join(directory, 'REUSE-headers.txt');
        demand({ condition: isFile(headers) && /Copyright/.test(readFileSync(headers, 'utf8')), message: 'runtime/licenses/' + name + ' ships license texts without REUSE-headers.txt copyright lines' });
      }
      notices.push(...texts.map(file => 'runtime/licenses/' + name + '/' + file));
    }
    // `--licenses` with an empty environment (no display, no home) prints every license and notice text in full:
    // the files below LICENSES/ and runtime/licenses/, and every other embedded file named like a license.
    const licenseNamed = (relative: string): boolean => {
      const name = (relative.split('/').at(-1) ?? '').toUpperCase();
      return name.includes('LICENSE') || name.includes('LICENCE') || name.startsWith('COPYING') || name.startsWith('NOTICE');
    };
    // The crate license list is printed entry by entry; the crate-licenses check reads those entries.
    const listed = expectedFiles().filter(([relative]) => relative !== 'LICENSES/crates.json').filter(([relative]) => relative.startsWith('LICENSES/') || relative.startsWith('runtime/licenses/') || licenseNamed(relative));
    const listing = spawnSync(executable, ['--licenses'], { env: {}, cwd: artifact, maxBuffer: 64 * 1024 * 1024, timeout: 60_000 });
    demand({ condition: !listing.error, message: 'could not run ' + executable + ' --licenses: ' + listing.error?.message });
    demand({ condition: listing.status === 0, message: executable + ' --licenses exited with status ' + listing.status + ': ' + listing.stderr.toString('utf8').slice(0, 600) });
    demand({ condition: listing.stderr.length === 0, message: executable + ' --licenses wrote to standard error: ' + listing.stderr.toString('utf8').slice(0, 600) });
    const printed = listing.stdout.toString('utf8');
    const headings = printed.split('\n').filter(line => line.startsWith('Embedded as ')).length;
    demand({ condition: headings === listed.length, message: '--licenses printed ' + headings + ' texts, expected ' + listed.length });
    const rule = '='.repeat(78);
    const unprinted = listed.filter(([relative, file]) => !printed.includes('\nEmbedded as ' + relative + '\n' + rule + '\n\n' + readFileSync(file, 'utf8')));
    demand({ condition: unprinted.length === 0, message: '--licenses does not print in full: ' + unprinted.map(([relative]) => relative).join(', ') });
    const missing = notEmbedded([...expected.map(([relative]) => relative), ...notices]);
    demand({ condition: missing.length === 0, message: 'license texts not embedded byte for byte: ' + missing.join(', ') });
    return { texts: expected.length, grammarNotices: notices.length, listedByLicensesFlag: headings, listingBytes: listing.stdout.length };
  },
};

// One nested-compositor session of an application copy on the disposable project.
// Returns the combined compositor and application output once `settled` accepts it, a screenshot, the time from
// start to the settled output, and what was wrong with the way the session ended, if anything: the caller judges
// the output first, so a damaged file is reported by its damage and not by a slow exit.
const plain = (text: string): string => text.replace(/\u001b\[[0-9;]*m/g, '');
// Distinct colors among every 61st pixel of the compositor's RGBA PNG: a window not yet drawn is one flat color.
const sampledColors = (png: Buffer): number => {
  const chunks: Buffer[] = [];
  for (let offset = 8; offset < png.length;) {
    const length = png.readUInt32BE(offset);
    if (png.toString('latin1', offset + 4, offset + 8) === 'IDAT') chunks.push(png.subarray(offset + 8, offset + 8 + length));
    offset += 12 + length;
  }
  const width = png.readUInt32BE(16);
  const rows = inflateSync(Buffer.concat(chunks));
  const stride = 1 + width * 4;
  const colors = new Set<number>();
  for (let row = 0; row * stride < rows.length; row++) {
    for (let column = row % 61; column < width; column += 61) colors.add(rows.readUInt32BE(row * stride + 1 + column * 4));
  }
  return colors.size;
};
// Stop and remove what a session may leave behind, and return what was found.
// A compositor that ends by a signal leaves its hosted application, its private bus daemon, and that daemon's
// directory, which carries the compositor's process id; a compositor that ends on `quit` leaves none of them.
const sweep = ({ compositorProcess, application }: { compositorProcess: number | undefined; application: string }): string[] => {
  const directory = join(tmpdir(), 'monochromatic-nested-wayland-session-' + compositorProcess + '-0');
  const found: string[] = [];
  for (const entry of readdirSync('/proc')) {
    if (!/^\d+$/.test(entry)) continue;
    // A process may end between the listing and these reads; that is not a leftover.
    let exe: string;
    let command: string[];
    try {
      exe = readlinkSync('/proc/' + entry + '/exe');
      command = readFileSync('/proc/' + entry + '/cmdline', 'utf8').split('\0');
    } catch { continue; }
    if (exe !== application && !command.some(part => part.includes(directory + sep))) continue;
    found.push(command.filter(Boolean).join(' ').slice(0, 160));
    try { process.kill(Number(entry), 'SIGKILL'); } catch { continue; }
  }
  if (existsSync(directory)) { found.push(directory); rmSync(directory, { recursive: true, force: true }); }
  return found;
};
const prepared = /source syntax prepared path=\S*fixture\.sql spans=(\d+)/;
const unavailable = 'highlighting unavailable; retaining readable source';
const settledOnSyntax = (text: string): boolean => prepared.test(text) || text.includes(unavailable);
// `cache` and `config` may be shared between sessions; each session gets its own project and data folders.
interface SessionResult { output: string; frame: string; settledAfter: number; ending: string | undefined }
const session = async ({ name, application, cache, config, settled = settledOnSyntax }: {
  name: string;
  application: string;
  cache: string;
  config: string;
  settled?: (text: string) => boolean;
}): Promise<SessionResult> => {
  demand({ condition: process.env.WAYLAND_DISPLAY, message: 'a Wayland session is needed to host the nested compositor' });
  demandCurrentCompositor();
  const directory = join(artifact, name);
  const project = join(directory, 'project');
  for (const part of ['data', 'project']) mkdirSync(join(directory, part), { recursive: true });
  mkdirSync(cache, { recursive: true });
  mkdirSync(config, { recursive: true });
  // SQL has a bundled grammar and no configured language server, so nothing but the application starts.
  writeFileSync(join(project, 'fixture.sql'), ['-- A comment, a keyword, a string, and a number.', "select 'cat' as name, 42 as answer from pets where name = 'cat';", ''].join('\n'));
  // Nothing may point at the source tree: no runtime variable, no Cargo variable, private XDG homes.
  const env: NodeJS.ProcessEnv = { ...process.env, XDG_CONFIG_HOME: config, XDG_CACHE_HOME: cache, XDG_DATA_HOME: join(directory, 'data') };
  for (const key of ['HELIX_RUNTIME', 'CARGO_MANIFEST_DIR', 'SLINT_BACKEND', 'SLINT_SCALE_FACTOR', 'RUST_LOG']) delete env[key];
  const socket = join(directory, 'control.sock');
  const started = performance.now();
  // RUST_LOG asks the application for the debug records these checks read (a build whose default level is lower
  // shows them only through it); it is set for the application alone, because the compositor reads it too.
  const hosted = ['/usr/bin/env', 'RUST_LOG=ide_app=debug,monochromatic_ide=debug', application, project, '--file', 'fixture.sql'];
  const child = spawn(compositor, ['--socket', socket, '--size', '1100x660', '--color-scheme', 'dark', '--', ...hosted], { cwd: '/', env, stdio: ['ignore', 'pipe', 'pipe'] });
  const chunks: Buffer[] = [];
  child.stdout.on('data', chunk => chunks.push(chunk));
  child.stderr.on('data', chunk => chunks.push(chunk));
  const output = () => plain(Buffer.concat(chunks).toString());
  const exited = once(child, 'exit');
  const control = (line: string): Promise<string> => new Promise((resolveReply, reject) => {
    const connection = createConnection(socket);
    let reply = '';
    connection.on('error', reject);
    // A compositor that is going away may close the connection without a reply; that must not hang the check.
    connection.on('close', () => reject(new Error('the compositor closed the control connection without replying to ' + line)));
    connection.on('data', chunk => { reply += chunk; if (reply.includes('\n')) { connection.end(); resolveReply(reply.trim()); } });
    connection.write(line + '\n');
  });
  const log = join(directory, 'session.log');
  try {
    // The compositor's own start (its EGL setup) took 37 s at a host load of 77 on 2026-10-06, and it starts the
    // application only after that; the application's 60 s count from the compositor's first control reply.
    const compositorDeadline = performance.now() + 120000;
    while (!(await control('ping').then(reply => reply.startsWith('ok'), () => false))) {
      demand({ condition: child.exitCode === null && child.signalCode === null, message: 'the session ended early with status ' + child.exitCode + '; see ' + log });
      demand({ condition: performance.now() < compositorDeadline, message: 'the compositor did not answer its control socket within 120 s; see ' + log });
      await wait(100);
    }
    const deadline = performance.now() + 60000;
    while (!settled(output())) {
      demand({ condition: child.exitCode === null && child.signalCode === null, message: 'the session ended early with status ' + child.exitCode + '; see ' + log });
      demand({ condition: performance.now() < deadline, message: 'the expected application output did not appear within 60 s of the compositor answering; see ' + log });
      await wait(50);
    }
    const settledAfter = Math.round(performance.now() - started);
    // A frame counts once the window is drawn (more than eight sampled colors) and two screenshots 250 ms apart
    // are identical. The rows are filtered per line, but these screenshots use no filter, so pixels read directly.
    const frame = join(directory, 'frame.png');
    let previous: Buffer | undefined;
    let colors = 0;
    for (let attempt = 0; attempt < 240; attempt++) {
      demand({ condition: await control('screenshot ' + frame) === 'ok', message: 'the compositor refused a screenshot' });
      const bytes = readFileSync(frame);
      colors = sampledColors(bytes);
      if (colors > 8 && previous?.equals(bytes)) break;
      previous = bytes;
      await wait(250);
    }
    demand({ condition: colors > 8, message: 'the window was not drawn within 60 s of the settled output; see ' + frame });
    demand({ condition: await control('quit') === 'ok', message: 'the compositor refused quit' });
    const [code, signal] = await exited;
    let ending: string | undefined;
    // The compositor reports a client it had to kill as exit status 0, so the forced case is checked first.
    if (output().includes('forcing shutdown')) ending = 'the application did not exit within the compositor\'s 2 s after the close request and was stopped by force; see ' + log;
    else if (code !== 0 || signal !== null) ending = 'the session ended with status ' + code + ' and signal ' + signal + '; see ' + log;
    else if (!output().includes('hosted client exited with code 0')) ending = 'the application did not exit with status 0; see ' + log;
    // The application's process may outlive the compositor's report by a moment; a real leftover outlives this wait.
    await wait(300);
    const left = sweep({ compositorProcess: child.pid, application });
    if (ending === undefined && left.length > 0) ending = 'the session left behind: ' + left.join('; ') + '; see ' + log;
    return { output: output(), frame, settledAfter, ending };
  } finally {
    // A failed session is asked to quit first and stopped by a signal only when that does not end it.
    if (child.exitCode === null && child.signalCode === null) {
      const asked = await Promise.race([control('quit').catch(() => undefined), wait(2000)]);
      const force = setTimeout(() => child.kill('SIGKILL'), asked === 'ok' ? 5000 : 0);
      await exited;
      clearTimeout(force);
    }
    writeFileSync(log, output());
    const left = sweep({ compositorProcess: child.pid, application });
    if (left.length > 0) console.error('Removed what the session left behind: ' + left.join('; '));
  }
};
// A copy of the executable alone in its own folder below the scratch root, so nothing can lie beside it.
const loneCopy = (name: string): string => {
  const directory = join(artifact, name);
  mkdirSync(directory);
  const copy = join(directory, 'monochromatic-ide');
  copyFileSync(executable, copy);
  chmodSync(copy, 0o755);
  demand({ condition: !copy.startsWith(source + sep), message: 'the application copy must lie outside the source tree' });
  demand({ condition: readdirSync(directory).length === 1, message: 'the copy is not alone in its folder' });
  return copy;
};
const spansOf = ({ output, what }: { output: string; what: string }): number => {
  demand({ condition: !output.includes(unavailable), message: what + ' could not highlight: ' + output.split('\n').find(line => line.includes(unavailable)) });
  const preparedSpans = output.match(prepared)?.[1];
  if (preparedSpans === undefined) throw new Error(what + ' printed no record of prepared syntax for fixture.sql');
  const spans = Number(preparedSpans);
  demand({ condition: spans > 0, message: what + ' prepared no highlighted span for fixture.sql' });
  return spans;
};
// The one cache folder of this executable's embedded runtime, below a cache home.
const keyFolder = (cache: string): string => {
  const root = join(cache, 'monochromatic-ide', 'runtime');
  demand({ condition: isDirectory(root), message: 'no unpacked runtime below ' + root });
  const keys = readdirSync(root);
  const key = keys[0];
  if (keys.length !== 1 || key === undefined || !/^[0-9a-f]{16}$/.test(key)) throw new Error('expected one runtime key folder below ' + root + ', found ' + JSON.stringify(keys));
  return join(root, key);
};
const cachedSql = (cache: string): string => join(keyFolder(cache), 'grammars', 'sql.so');
const sqlLibrary = (): Buffer => readFileSync(join(runtime, 'grammars', 'sql.so'));
const partials = (cache: string): string[] => readdirSync(join(keyFolder(cache), 'grammars')).filter(name => !name.endsWith('.so'));
const elapsed = ({ output, phrase }: { output: string; phrase: string }): number => Number(output.split('\n').find(line => line.includes(phrase))?.match(/elapsed_us=(\d+)/)?.[1] ?? NaN);
const unpackedPhrase = 'language parser unpacked into the private cache';
const reusedPhrase = 'cached language parser matches the embedded one';
const rewrittenPhrase = 'cached language parser differs from the embedded one; writing it again';

// A lone copy reports its version with no environment at all, then highlights from what it carries, unpacking only
// the parser it needs into an empty private cache.
checks['lone-copy-highlights'] = async () => {
  const application = loneCopy('lone');
  const version = spawnSync(application, ['--version'], { cwd: '/', env: {}, encoding: 'utf8' });
  demand({ condition: version.status === 0 && /^monochromatic-ide \d+\.\d+\.\d+/.test(version.stdout), message: 'the copy did not report its version: ' + JSON.stringify([version.status, version.stdout, version.stderr]) });
  const cache = join(artifact, 'lone-home', 'cache');
  const { output, frame, settledAfter, ending } = await session({ name: 'lone-session', application, cache, config: join(artifact, 'lone-home', 'config') });
  const spans = spansOf({ output, what: 'the lone copy' });
  demand({ condition: isFile(cachedSql(cache)) && readFileSync(cachedSql(cache)).equals(sqlLibrary()), message: 'the unpacked sql.so differs from the runtime it was built from' });
  demand({ condition: partials(cache).length === 0, message: 'partial files remain in the cache: ' + partials(cache).join(', ') });
  demand({ condition: output.includes(unpackedPhrase), message: 'the first start did not report unpacking the parser' });
  demand({ condition: ending === undefined, message: ending });
  return { version: version.stdout.trim(), spans, key: keyFolder(cache).split(sep).at(-1), unpackedUs: elapsed({ output, phrase: unpackedPhrase }), settledAfterMs: settledAfter, frame };
};
// A second start with the same cache compares the cached parser and reuses it without writing.
checks['later-start-reuses-cache'] = async () => {
  const application = loneCopy('later');
  const home = join(artifact, 'later-home');
  const cache = join(home, 'cache');
  const first = await session({ name: 'later-first', application, cache, config: join(home, 'config') });
  spansOf({ output: first.output, what: 'the first start' });
  const before = statSync(cachedSql(cache));
  const second = await session({ name: 'later-second', application, cache, config: join(home, 'config') });
  const spans = spansOf({ output: second.output, what: 'the later start' });
  demand({ condition: second.output.includes(reusedPhrase), message: 'the later start did not report reusing the cached parser' });
  demand({ condition: !second.output.includes(unpackedPhrase), message: 'the later start unpacked the parser again' });
  const after = statSync(cachedSql(cache));
  demand({ condition: after.mtimeMs === before.mtimeMs && after.ino === before.ino, message: 'the later start rewrote the cached parser' });
  demand({ condition: first.ending === undefined, message: first.ending });
  demand({ condition: second.ending === undefined, message: second.ending });
  return { spans, firstUnpackedUs: elapsed({ output: first.output, phrase: unpackedPhrase }), laterReusedUs: elapsed({ output: second.output, phrase: reusedPhrase }), firstSettledAfterMs: first.settledAfter, laterSettledAfterMs: second.settledAfter, frame: second.frame };
};
// A cached parser with one changed byte is detected, written again from the embedded copy, and highlighting works.
checks['damaged-cache-rebuilt'] = async () => {
  const application = loneCopy('damaged-cache');
  const home = join(artifact, 'damaged-cache-home');
  const cache = join(home, 'cache');
  const first = await session({ name: 'damaged-cache-first', application, cache, config: join(home, 'config') });
  spansOf({ output: first.output, what: 'the first start' });
  const damaged = readFileSync(cachedSql(cache));
  const middle = Math.floor(damaged.length / 2);
  damaged[middle] = (damaged[middle] ?? 0) ^ 0x01;
  writeFileSync(cachedSql(cache), damaged);
  const second = await session({ name: 'damaged-cache-second', application, cache, config: join(home, 'config') });
  const spans = spansOf({ output: second.output, what: 'the start after the damage' });
  demand({ condition: second.output.includes(rewrittenPhrase), message: 'the damaged cached parser was not reported and written again' });
  demand({ condition: readFileSync(cachedSql(cache)).equals(sqlLibrary()), message: 'the cached parser was not restored' });
  demand({ condition: partials(cache).length === 0, message: 'partial files remain in the cache: ' + partials(cache).join(', ') });
  demand({ condition: second.ending === undefined, message: second.ending });
  return { spans, report: (second.output.split('\n').find(line => line.includes(rewrittenPhrase)) ?? '').slice(0, 300), frame: second.frame };
};
// An empty highlights.scm below $XDG_CONFIG_HOME/helix/runtime, which Helix's own lookup prefers, changes nothing.
checks['shadowing-query-ignored'] = async () => {
  const application = loneCopy('shadowing');
  const home = join(artifact, 'shadowing-home');
  const plainRun = await session({ name: 'shadowing-without', application, cache: join(home, 'cache'), config: join(home, 'config-empty') });
  const without = spansOf({ output: plainRun.output, what: 'the start without a user query' });
  const queries = join(home, 'config-shadowing', 'helix', 'runtime', 'queries', 'sql');
  mkdirSync(queries, { recursive: true });
  writeFileSync(join(queries, 'highlights.scm'), '');
  const shadowed = await session({ name: 'shadowing-with', application, cache: join(home, 'cache'), config: join(home, 'config-shadowing') });
  const withQuery = spansOf({ output: shadowed.output, what: 'the start with an empty user query' });
  demand({ condition: withQuery === without, message: 'the user query changed the highlighting: ' + without + ' spans without it, ' + withQuery + ' with it' });
  demand({ condition: plainRun.ending === undefined, message: plainRun.ending });
  demand({ condition: shadowed.ending === undefined, message: shadowed.ending });
  return { spansWithout: without, spansWithShadowingQuery: withQuery, frame: shadowed.frame };
};
// Two first starts at the same moment, sharing one empty cache, both highlight and leave one intact parser.
// Each runs its own identical copy, so the leftover sweep of one session cannot stop the other's application.
checks['concurrent-first-starts'] = async () => {
  const home = join(artifact, 'concurrent-home');
  const cache = join(home, 'cache');
  const start = (name: string): Promise<SessionResult> => session({ name, application: loneCopy(name + '-copy'), cache, config: join(home, 'config') });
  const [left, right] = await Promise.all([start('concurrent-left'), start('concurrent-right')]);
  const spans = [spansOf({ output: left.output, what: 'the first concurrent start' }), spansOf({ output: right.output, what: 'the second concurrent start' })];
  demand({ condition: readFileSync(cachedSql(cache)).equals(sqlLibrary()), message: 'the shared cached parser is not intact' });
  demand({ condition: partials(cache).length === 0, message: 'partial files remain in the cache: ' + partials(cache).join(', ') });
  demand({ condition: left.ending === undefined, message: left.ending });
  demand({ condition: right.ending === undefined, message: right.ending });
  const writers = [left, right].filter(run => run.output.includes(unpackedPhrase)).length;
  return { spans, sessionsThatWrote: writers, frame: left.frame };
};
// One changed byte inside the embedded sql.so: the copy keeps the source readable and names the damaged part and
// the remedy, and unpacks nothing for it.
checks['damaged-embedded-part-reported'] = async () => {
  const application = loneCopy('damaged-embedded');
  const bytes = readFileSync(application);
  const library = sqlLibrary();
  const at = bytes.indexOf(library);
  demand({ condition: at >= 0, message: 'the executable does not embed sql.so; the inventory check names what is missing' });
  demand({ condition: bytes.indexOf(library, at + 1) < 0, message: 'sql.so occurs twice in the executable, so the damaged copy would be ambiguous' });
  const middle = at + Math.floor(library.length / 2);
  bytes[middle] = (bytes[middle] ?? 0) ^ 0x01;
  writeFileSync(application, bytes);
  const home = join(artifact, 'damaged-embedded-home');
  const { output, frame, ending } = await session({ name: 'damaged-embedded-session', application, cache: join(home, 'cache'), config: join(home, 'config') });
  const report = output.split('\n').find(line => line.includes(unavailable));
  if (report === undefined) throw new Error('the damaged copy reported nothing; it would load or silently skip a damaged parser');
  demand({ condition: report.includes('runtime/grammars/sql.so embedded in ' + application + ' is damaged'), message: 'the report does not name the damaged part and the executable: ' + report });
  demand({ condition: report.includes('Replace the executable'), message: 'the report does not give the remedy: ' + report });
  const unpackedRoot = join(home, 'cache', 'monochromatic-ide', 'runtime');
  // Every start renews its key folder's `last-used` marker; anything else below the runtime cache was unpacked.
  const unpacked = isDirectory(unpackedRoot) ? filesBelow(unpackedRoot).filter(file => !/^[0-9a-f]{16}\/last-used$/.test(file)) : [];
  demand({ condition: unpacked.length === 0, message: 'files were unpacked although the embedded part is damaged: ' + unpacked.join(', ') });
  demand({ condition: ending === undefined, message: ending });
  return { report: report.slice(report.indexOf(unavailable)).slice(0, 400), frame };
};

// A start removes the cache folders of other builds unused for more than 30 days and the rest of a removal cut
// short, keeps a folder used a day ago, and renews its own folder's marker, all before any window: the display
// connection of this start fails on purpose, so no compositor is needed.
checks['old-cache-folders-removed'] = async () => {
  const base = mkdtempSync(join(artifact, 'old-cache-'));
  const project = join(base, 'project');
  mkdirSync(project);
  writeFileSync(join(project, 'fixture.sql'), 'select 1;\n');
  const runtimeCache = join(base, 'cache', 'monochromatic-ide', 'runtime');
  const now = Date.now();
  const seed = ({ name, daysAgo }: { name: string; daysAgo: number }): void => {
    mkdirSync(join(runtimeCache, name, 'grammars'), { recursive: true, mode: 0o700 });
    writeFileSync(join(runtimeCache, name, 'grammars', 'sql.so'), 'an older build');
    writeFileSync(join(runtimeCache, name, 'last-used'), '');
    const time = new Date(now - daysAgo * 86_400_000);
    utimesSync(join(runtimeCache, name, 'last-used'), time, time);
  };
  seed({ name: '0000000000000031', daysAgo: 31 });
  seed({ name: '0000000000000001', daysAgo: 1 });
  mkdirSync(join(runtimeCache, '.00000000000000aa.removing-1-0', 'grammars'), { recursive: true });
  const started = spawnSync(executable, [project], {
    cwd: base, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024, timeout: 60_000,
    env: { PATH: '/usr/bin', HOME: join(base, 'home'), XDG_CACHE_HOME: join(base, 'cache'), XDG_CONFIG_HOME: join(base, 'config'), XDG_DATA_HOME: join(base, 'data'), XDG_RUNTIME_DIR: base, WAYLAND_DISPLAY: join(base, 'absent-wayland.socket'), SLINT_BACKEND: 'winit', RUST_LOG: 'ide_app=debug,monochromatic_ide=debug' },
  });
  const output = plain((started.stdout ?? '') + (started.stderr ?? ''));
  writeFileSync(join(base, 'start.log'), output);
  demand({ condition: !started.error, message: 'could not start ' + executable + ': ' + started.error?.message });
  const left = readdirSync(runtimeCache).sort();
  demand({ condition: !left.includes('0000000000000031'), message: 'the folder unused for 31 days remains: ' + JSON.stringify(left) });
  demand({ condition: !left.some(name => name.startsWith('.')), message: 'the rest of a cut-short removal remains: ' + JSON.stringify(left) });
  demand({ condition: isFile(join(runtimeCache, '0000000000000001', 'grammars', 'sql.so')), message: 'the folder used a day ago was removed: ' + JSON.stringify(left) });
  const own = left.filter(name => name !== '0000000000000001');
  const ownFolder = own[0];
  if (own.length !== 1 || ownFolder === undefined || !/^[0-9a-f]{16}$/.test(ownFolder)) throw new Error('expected this build\'s key folder besides the kept one: ' + JSON.stringify(left));
  const renewed = lstatSync(join(runtimeCache, ownFolder, 'last-used')).mtimeMs;
  demand({ condition: renewed >= now - 1000, message: 'this build\'s marker was not renewed at start' });
  return { left, status: started.status, tidyElapsedUs: elapsed({ output, phrase: 'runtime cache tidied' }) };
};

// The Rust crate license list: `--licenses` prints one section per text of target/crate-licenses.json, with its
// heading, its crates, and the text in full; every crate `cargo tree --edges normal,no-proc-macro` lists as linked
// (the linked-crates task writes target/linked-crates.txt) and every registry crate whose source path the
// executable itself names is among the crates printed.
checks['crate-licenses'] = async () => {
  demand({ condition: isFile(crateLicensesFile), message: crateLicensesFile + ' is missing; run the notices task' });
  demand({ condition: isFile(linkedCratesFile), message: linkedCratesFile + ' is missing; run the linked-crates task' });
  const list: { licenses: { name: string; id: string; text: string; crates: { name: string; version: string }[] }[] } = JSON.parse(readFileSync(crateLicensesFile, 'utf8'));
  demand({ condition: Array.isArray(list.licenses) && list.licenses.length > 0, message: crateLicensesFile + ' lists no license text' });
  const listing = spawnSync(executable, ['--licenses'], { env: {}, cwd: artifact, maxBuffer: 64 * 1024 * 1024, timeout: 60_000 });
  demand({ condition: !listing.error, message: 'could not run ' + executable + ' --licenses: ' + listing.error?.message });
  demand({ condition: listing.status === 0, message: executable + ' --licenses exited with status ' + listing.status + ': ' + listing.stderr.toString('utf8').slice(0, 600) });
  const printed = listing.stdout.toString('utf8');
  const rule = '='.repeat(78);
  const section = /^Rust crates under (.+) \(([^()\n]+)\): (\d+) crates?\n((?:Used by: |         ).*\n(?:         .*\n)*)(Text from the crate file .+|Text: the standard text of this license \(no crate file was recognized\))\n={78}\n\n/gm;
  const sections = [...printed.matchAll(section)];
  demand({ condition: sections.length === list.licenses.length, message: '--licenses printed ' + sections.length + ' crate license sections, the list holds ' + list.licenses.length });
  const covered = new Set<string>();
  for (const [index, license] of list.licenses.entries()) {
    const found = sections[index];
    if (found === undefined) throw new Error('crate license section ' + index + ' was not printed');
    const [whole = '', name, id, count, usedBy = ''] = found;
    demand({ condition: name === license.name && id === license.id, message: 'crate license section ' + index + ' names ' + name + ' (' + id + '), the list ' + license.name + ' (' + license.id + ')' });
    const tokens = usedBy.replace(/^Used by: /, '').replaceAll(',', ' ').split(/\s+/).filter(Boolean);
    const crates: string[] = [];
    for (let at = 0; at + 1 < tokens.length; at += 2) crates.push(tokens[at] + ' ' + tokens[at + 1]);
    const expected = license.crates.map(item => item.name + ' ' + item.version);
    demand({ condition: Number(count) === expected.length && crates.join(',') === expected.join(','), message: 'crate license section ' + index + ' (' + id + ') lists ' + crates.join(', ') + ' instead of ' + expected.join(', ') });
    demand({ condition: printed.includes(whole + license.text), message: 'the ' + id + ' text for ' + expected[0] + ' is not printed in full under its heading' });
    for (const item of crates) covered.add(item);
  }
  const linked = readFileSync(linkedCratesFile, 'utf8').split('\n').filter(Boolean).filter(line => !line.startsWith('ide '));
  const uncovered = linked.filter(item => !covered.has(item));
  demand({ condition: uncovered.length === 0, message: uncovered.length + ' of ' + linked.length + ' linked crates have no license text: ' + uncovered.slice(0, 12).join(', ') });
  const named = new Set<string>();
  for (const match of binary().toString('latin1').matchAll(/\/registry\/src\/index\.crates\.io-[0-9a-f]+\/([A-Za-z0-9_.+-]+?)-(\d+\.\d+\.\d+(?:[-+][A-Za-z0-9.+-]*)?)\//g)) named.add(match[1] + ' ' + match[2]);
  const unnamed = [...named].filter(item => !covered.has(item));
  demand({ condition: unnamed.length === 0, message: 'crates whose sources the executable names have no license text: ' + unnamed.join(', ') });
  return { texts: list.licenses.length, cratesCovered: covered.size, linkedCrates: linked.length, cratesNamedInTheExecutable: named.size };
};

const results: { name: string; passed: boolean; detail: unknown }[] = [];
for (const name of names) {
  if (only && !only.has(name)) continue;
  try {
    const check = checks[name];
    if (check === undefined) throw new Error('No check is defined under the name ' + name);
    results.push({ name, passed: true, detail: await check() });
  } catch (error) {
    results.push({ name, passed: false, detail: error instanceof Error ? error.message : String(error) });
  }
  writeFileSync(join(artifact, 'results.json'), JSON.stringify(results, null, 2));
  console.log(JSON.stringify(results.at(-1)));
}
const failed = results.filter(result => !result.passed).map(result => result.name);
if (failed.length > 0) throw new Error('Bundle checks failed for ' + executable + ': ' + failed.join(', ') + '; inspect ' + artifact);
console.log('Bundle checks passed for ' + executable + ' (' + results.length + ' of ' + names.length + ' checks): ' + artifact);
