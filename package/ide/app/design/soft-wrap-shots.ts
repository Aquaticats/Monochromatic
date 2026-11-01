// Screenshots of the wrapped source view in the nested compositor, light and dark.
// What: Opens a sample file whose lines wrap in the built IDE inside the nested compositor and captures it.
// Why: The soft-wrap frames in `screenshots/2026-10-12-soft-wrap-applied/` come from this script.
// Usage: node design/soft-wrap-shots.ts <application> <output directory> [compositor]
// Needs `WAYLAND_DISPLAY` and `XDG_RUNTIME_DIR` of a running Wayland session.
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { createConnection } from 'node:net';
import { mkdirSync, writeFileSync, readFileSync, copyFileSync, existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { inflateSync } from 'node:zlib';

const [application, output, compositorArg] = process.argv.slice(2);
if (application === undefined || output === undefined) throw new Error('Usage: node design/soft-wrap-shots.ts <application> <output directory> [compositor]');
const compositor = compositorArg ?? resolve(import.meta.dirname, '../../../nested-wayland-session/cli/target/release/monochromatic-nested-wayland-session');
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
const sample = [
  '// A long comment line that goes on and on to show how the text wraps at the right edge of the text area, with the number only on its first row.',
  'fn main() {',
  '    let message = format!("a long string literal that wraps because it does not fit the width of the pane {} {}", first_value, second_value);',
  '        let deeply_indented = compute_total(first_item, second_item, third_item, fourth_item, fifth_item, sixth_item, seventh_item); // a trailing comment',
  '    short();',
  '\tlet tabbed = values.iter().map(|value| value * 2).filter(|value| value % 3 == 0).collect::<Vec<u32>>().len(); // tab indented and long',
  '    let unbroken = "' + 'x'.repeat(190) + '";',
  '}',
  '',
].join('\n');
for (const scheme of ['light', 'dark']) {
  const directory = join(output, 'work-' + scheme);
  const project = join(directory, 'project');
  mkdirSync(project, { recursive: true });
  mkdirSync(join(directory, 'data'), { recursive: true });
  writeFileSync(join(project, 'sample.rs'), sample);
  const env: NodeJS.ProcessEnv = { ...process.env, XDG_CONFIG_HOME: join(directory, 'config'), XDG_CACHE_HOME: join(directory, 'cache'), XDG_DATA_HOME: join(directory, 'data') };
  for (const key of ['HELIX_RUNTIME', 'CARGO_MANIFEST_DIR', 'SLINT_BACKEND', 'SLINT_SCALE_FACTOR']) delete env[key];
  const socket = join(directory, 'control.sock');
  const child = spawn(compositor, ['--socket', socket, '--size', '1100x660', '--color-scheme', scheme, '--', application, project, '--file', 'sample.rs'], { cwd: '/', env, stdio: ['ignore', 'pipe', 'pipe'] });
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
    const shot = async (name: string): Promise<void> => {
      const path = join(output, name);
      let previous: Buffer | undefined;
      for (let attempt = 0; attempt < 120; attempt++) {
        await control('screenshot ' + path);
        const bytes = readFileSync(path);
        if (previous?.equals(bytes) && attempt > 4 && !flat(bytes)) return;
        previous = bytes;
        await wait(300);
      }
      throw new Error('frame never settled: ' + name);
    };
    await wait(4000);
    await shot('wrap-' + scheme + '.png');
    // Drag a selection from the middle of the first long row to the middle of the third long line, across wrapped rows.
    console.log(await control('click 400 56'));
    await wait(700);
    await shot('wrap-' + scheme + '-click.png');
    await control('key shift press');
    await wait(200);
    console.log(await control('click 520 230'));
    await wait(300);
    await control('key shift release');
    await wait(500);
    await shot('wrap-' + scheme + '-selection.png');
    await control('quit');
    await exited;
  } finally {
    if (child.exitCode === null) { await control('quit').catch(() => undefined); await wait(2000); child.kill('SIGKILL'); }
    writeFileSync(join(output, 'session-' + scheme + '.log'), Buffer.concat(chunks).toString());
  }
}
console.log('done');
