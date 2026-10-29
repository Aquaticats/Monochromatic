// Screenshots of hints and diagnostic messages on wrapped lines in the nested compositor, light and dark.
// What: Opens a sample file whose lines wrap in a debug build of the IDE, with hints and diagnostics injected through
//       the debug-only inspection file (`IDE_INSPECT_ANNOTATIONS`), at a wide and a narrow window.
// Why: The frames in `screenshots/2026-10-12-wrap-annotations-applied/` come from this script.
// Usage: node design/wrap-annotation-shots.ts <debug application> <output directory> <runtime directory> [compositor]
// The application must be a debug build (`mise run //package/ide/app:build:debug`): release builds have no
// inspection path. `runtime directory` is `target/debug/runtime` of that build.
// Needs `WAYLAND_DISPLAY` and `XDG_RUNTIME_DIR` of a running Wayland session.
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { createConnection } from 'node:net';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { inflateSync } from 'node:zlib';

const [application, output, runtime, compositorArg] = process.argv.slice(2);
if (application === undefined || output === undefined || runtime === undefined) {
  throw new Error('Usage: node design/wrap-annotation-shots.ts <debug application> <output directory> <runtime directory> [compositor]');
}
const compositor = compositorArg ?? resolve(import.meta.dirname, '../../../nested-wayland-session/cli/target/release/monochromatic-nested-wayland-session');
// An undrawn window is one flat color; sampled colors of a PNG tell it from a drawn one.
const flat = (png: Buffer): boolean => {
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
  return colors.size <= 8;
};
const wait = (ms: number): Promise<void> => new Promise<void>(done => setTimeout(done, ms));
mkdirSync(output, { recursive: true });
const lines = [
  '// A comment line with a hint and a message that both wrap, to show where blocks stand over a wrapped line.',
  'fn main() {',
  '    let message = format!("a long string literal that wraps because it does not fit the width of the pane {} {}", first_value, second_value);',
  '    short();',
  '        let deeply_indented = compute_total(first_item, second_item, third_item, fourth_item, fifth_item, sixth_item); // a trailing comment',
  '    let total = area(2, 3);',
  '}',
  '',
];
const sample = lines.join('\n');
// Character offsets of the injected places, from the sample itself.
const at = (needle: string): number => {
  const index = sample.indexOf(needle);
  if (index < 0) throw new Error('missing ' + needle);
  return [...sample.slice(0, index)].length;
};
const longMessage = 'expected `String`, found `&str`: the argument named second_value is a string slice here, so it has to be converted with `to_string` before it can be formatted into the message of the macro';
const injected = {
  hints: [
    { position: at('message ='), label: ': String', kind: 'type' },
    { position: at('first_value,'), label: 'first:', kind: 'parameter' },
    { position: at('second_value)'), label: 'second:', kind: 'parameter' },
    { position: at('compute_total(') + 'compute_total('.length, label: 'first_item:', kind: 'parameter' },
    { position: at('sixth_item)'), label: 'sixth_item:', kind: 'parameter' },
    { position: at('area(2') + 'area('.length, label: 'width:', kind: 'parameter' },
  ],
  diagnostics: [
    { source: 'rustc', start: at('second_value)'), end: at('second_value)') + 'second_value'.length, severity: 'error', code: 'E0308', message: longMessage },
    { source: 'rustc', start: at('short();'), end: at('short();') + 5, severity: 'warning', code: 'W0001', message: 'unused result' },
    { source: 'clippy', start: at('deeply_indented'), end: at('deeply_indented') + 'deeply_indented'.length, severity: 'warning', code: 'clippy::needless_late_init', message: 'this binding is only used in the next call, so it could be written inline' },
  ],
};
const sizes = [['wide', '1100x660'], ['narrow', '760x660']] as const;
for (const scheme of ['light', 'dark']) {
  for (const [name, size] of sizes) {
    const directory = join(output, 'work-' + scheme + '-' + name);
    const project = join(directory, 'project');
    mkdirSync(project, { recursive: true });
    mkdirSync(join(directory, 'data'), { recursive: true });
    writeFileSync(join(project, 'sample.rs'), sample);
    const inspection = join(directory, 'annotations.json');
    writeFileSync(inspection, JSON.stringify(injected, null, 2));
    const env: NodeJS.ProcessEnv = { ...process.env, HELIX_RUNTIME: resolve(runtime), IDE_INSPECT_ANNOTATIONS: inspection, XDG_CONFIG_HOME: join(directory, 'config'), XDG_CACHE_HOME: join(directory, 'cache'), XDG_DATA_HOME: join(directory, 'data') };
    for (const key of ['CARGO_MANIFEST_DIR', 'SLINT_BACKEND', 'SLINT_SCALE_FACTOR']) delete env[key];
    const socket = join(directory, 'control.sock');
    const child = spawn(compositor, ['--socket', socket, '--size', size, '--color-scheme', scheme, '--', application, project, '--file', 'sample.rs'], { cwd: '/', env, stdio: ['ignore', 'pipe', 'pipe'] });
    const chunks: Buffer[] = [];
    child.stdout.on('data', chunk => chunks.push(chunk));
    child.stderr.on('data', chunk => chunks.push(chunk));
    const exited = once(child, 'exit');
    const control = (line: string): Promise<string> => new Promise((resolveReply, reject) => {
      const connection = createConnection(socket);
      let reply = '';
      connection.on('error', reject);
      connection.on('close', () => reject(new Error('closed without a reply to ' + line)));
      connection.on('data', chunk => { reply += chunk; if (reply.includes('\n')) { connection.end(); resolveReply(reply.trim()); } });
      connection.write(line + '\n');
    });
    try {
      const deadline = Date.now() + 120000;
      while (!(await control('ping').then(reply => reply.startsWith('ok'), () => false))) {
        if (Date.now() > deadline) throw new Error('compositor did not answer');
        await wait(100);
      }
      await wait(5000);
      const path = join(output, 'wrap-annotations-' + scheme + '-' + name + '.png');
      let previous: Buffer | undefined;
      let settled = false;
      for (let attempt = 0; attempt < 120 && !settled; attempt++) {
        await control('screenshot ' + path);
        const bytes = readFileSync(path);
        settled = previous?.equals(bytes) === true && attempt > 4 && !flat(bytes);
        previous = bytes;
        if (!settled) await wait(300);
      }
      if (!settled) throw new Error('frame never settled: ' + path);
      await control('quit');
      await exited;
    } finally {
      if (child.exitCode === null) { await control('quit').catch(() => undefined); await wait(2000); child.kill('SIGKILL'); }
      writeFileSync(join(output, 'session-' + scheme + '-' + name + '.log'), Buffer.concat(chunks).toString());
    }
  }
}
console.log('done');
