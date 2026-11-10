// The nested-compositor session both design capture scripts drive.
// What: the PNG flat-frame probe, one control-socket request, the readiness wait, the settled-screenshot loop,
//       the disposable project a session opens, and the session lifecycle: spawn with a scrubbed environment,
//       capture its own output, and quit-then-kill it.
// Why: two capture scripts take the same steps against the same compositor. A fix to the settle rule or to the
//      socket protocol applied to one copy would leave the other capturing a different frame set, and the two
//      copies had already drifted in shape while meaning the same thing.
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { createConnection } from 'node:net';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { inflateSync } from 'node:zlib';

/**
 The nested compositor of this repository's toolchain, for a script given no other.

 What: the build the compositor package of this checkout produces.
 Why: a capture script is run from this checkout, and the tool on PATH is the one the inspection tasks require.
 */
export const defaultCompositor = resolve(import.meta.dirname, '../../../nested-wayland-session/cli/target/release/monochromatic-nested-wayland-session');

/**
 How many attempts the settled-frame loop may spend before it gives up.
 */
export const SETTLE_ATTEMPTS = 120;

/**
 Milliseconds between two screenshots of the settled-frame loop.
 */
export const SETTLE_INTERVAL_MS = 300;

/**
 Attempts before which a frame may still be the first draw, so its being unchanged says nothing.
 */
export const SETTLE_DRAWN_ATTEMPTS = 5;

/**
 Milliseconds a compositor gets to answer its control socket before the session fails.
 */
export const READY_DEADLINE_MS = 120000;

/**
 Milliseconds between two readiness pings.
 */
export const READY_INTERVAL_MS = 100;

/**
 Milliseconds a session gets to end after its quit request before it is killed.
 */
export const QUIT_PATIENCE_MS = 2000;

/**
 What: a promise that resolves after `ms` milliseconds.
 Why: a compositor needs time to start, to draw, and to stop, and nothing waits on it otherwise.

 @param ms - milliseconds to wait
 @returns a promise no value resolves
 */
export const wait = (ms: number): Promise<void> => new Promise<void>(done => setTimeout(done, ms));

/**
 What: whether a PNG holds at most eight sampled colors, which is what an undrawn window looks like.
 Why: a window that has not drawn yet is one flat color, and a screenshot of it must not count as the frame.

 @param png - the encoded screenshot
 @returns true when the sampled pixels hold at most eight distinct colors
 */
export const flat = (png: Buffer): boolean => {
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

/**
 What: one request to a compositor's control socket, resolved with its reply line.
 Why: the compositor answers each line of its socket with one line, and every step of a session is such a request.

 @param socket - the control socket path the compositor was started with
 @returns a function that sends one line and resolves with the trimmed reply
 */
export function controlTo(socket: string): (line: string) => Promise<string> {
  return (line: string): Promise<string> => new Promise((resolveReply, reject) => {
    const connection = createConnection(socket);
    let reply = '';
    connection.on('error', reject);
    connection.on('close', () => reject(new Error('the compositor closed the control connection without replying to ' + line)));
    connection.on('data', chunk => { reply += chunk; if (reply.includes('\n')) { connection.end(); resolveReply(reply.trim()); } });
    connection.write(line + '\n');
  });
}

/**
 What: wait until the compositor answers its control socket.
 Why: a session started before the compositor listens fails at its first request, and the compositor's own start
      (its EGL setup) can take tens of seconds on a busy host.

 @param control - the control-socket request of this session
 @param deadline - the instant after which the session is declared unstarted
 @throws {Error} when the compositor has not answered `ok` by `deadline`, naming the last transport failure
 */
export async function waitForCompositor(control: (line: string) => Promise<string>, deadline = Date.now() + READY_DEADLINE_MS): Promise<void> {
  let failure = 'the control socket has not answered yet';
  while (true) {
    try {
      if ((await control('ping')).startsWith('ok')) return;
      failure = 'the compositor answered its ping without ok';
    } catch (error) {
      // A compositor killed while it starts closes its socket instead of answering, and that is not a slow compositor:
      // keep the transport failure so the deadline report names what went wrong.
      failure = error instanceof Error ? error.message : String(error);
    }
    if (Date.now() > deadline) throw new Error('the compositor did not answer: ' + failure);
    await wait(READY_INTERVAL_MS);
  }
}

/**
 What: take screenshots until two in a row are identical and the frame is drawn.
 Why: a screenshot requested before the window settled shows a transitional frame, and the frame sets these scripts
      record are read as the settled state.

 @param control - the control-socket request of this session
 @param path - where the compositor writes each screenshot
 @param attempts - how many screenshots may be taken
 @param intervalMs - milliseconds between two screenshots
 @throws {Error} when no two screenshots agree within `attempts`
 */
export async function settleScreenshot({ control, path, attempts = SETTLE_ATTEMPTS, intervalMs = SETTLE_INTERVAL_MS }: {
  control: (line: string) => Promise<string>;
  path: string;
  attempts?: number;
  intervalMs?: number;
}): Promise<void> {
  let previous: Buffer | undefined;
  for (let attempt = 0; attempt < attempts; attempt++) {
    await control('screenshot ' + path);
    const bytes = readFileSync(path);
    if (previous?.equals(bytes) === true && attempt > SETTLE_DRAWN_ATTEMPTS && !flat(bytes)) return;
    previous = bytes;
    await wait(intervalMs);
  }
  throw new Error('frame never settled: ' + path);
}

/**
 What: the disposable project a capture session opens, holding one sample source file.
 Why: the window must open a file whose lines are known, and the compositor writes its own session data beside it.

 @param directory - the session's own directory
 @param sample - the source text written as `sample.rs`
 @returns the project directory
 */
export function disposableProject({ directory, sample }: { directory: string; sample: string }): string {
  const project = join(directory, 'project');
  mkdirSync(project, { recursive: true });
  mkdirSync(join(directory, 'data'), { recursive: true });
  writeFileSync(join(project, 'sample.rs'), sample);
  return project;
}

/**
 What: the environment one capture session runs with.
 Why: a window that reads the caller's own configuration, fonts, or backend setting does not show the frame the
      script means to capture, and the session's own directories must hold everything it writes.

 @param directory - the session's own directory
 @param helixRuntime - the runtime directory the application reads, or nothing to leave the variable unset
 @returns the environment for the spawned compositor
 */
export function sessionEnvironment({ directory, helixRuntime }: { directory: string; helixRuntime?: string }): NodeJS.ProcessEnv {
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    XDG_CONFIG_HOME: join(directory, 'config'),
    XDG_CACHE_HOME: join(directory, 'cache'),
    XDG_DATA_HOME: join(directory, 'data'),
  };
  // A capture must not inherit the build directory, the window backend, or a scale the caller chose.
  for (const key of ['CARGO_MANIFEST_DIR', 'SLINT_BACKEND', 'SLINT_SCALE_FACTOR']) delete env[key];
  if (helixRuntime === undefined) delete env['HELIX_RUNTIME'];
  else env['HELIX_RUNTIME'] = helixRuntime;
  return env;
}

/**
 What: run one nested-compositor session over a sample file and hand its control socket to `body`.
 Why: every capture script must start the same session, wait for it the same way, and end it the same way: asked to
      quit, then killed when it does not, with its own output kept for a failed run.

 @param options - the session's application, project, file, screen, and environment
 @param body - the requests this session makes, given the session's control-socket request
 @throws {Error} when the compositor does not answer, or when `body` fails
 */
export async function captureSession({ application, project, file, scheme, size, directory, env, logPath, compositor = defaultCompositor }: {
  application: string;
  project: string;
  file: string;
  scheme: string;
  size: string;
  directory: string;
  env: NodeJS.ProcessEnv;
  logPath: string;
  compositor?: string;
}, body: (control: (line: string) => Promise<string>) => Promise<void>): Promise<void> {
  const socket = join(directory, 'control.sock');
  const child = spawn(compositor, ['--socket', socket, '--size', size, '--color-scheme', scheme, '--', application, project, '--file', file], { cwd: '/', env, stdio: ['ignore', 'pipe', 'pipe'] });
  const chunks: Buffer[] = [];
  child.stdout.on('data', chunk => chunks.push(chunk));
  child.stderr.on('data', chunk => chunks.push(chunk));
  const exited = once(child, 'exit');
  const control = controlTo(socket);
  try {
    await waitForCompositor(control);
    await body(control);
    await control('quit');
    await exited;
  } finally {
    // A failed session is asked to quit first and stopped by a signal only when that does not end it.
    if (child.exitCode === null) {
      try {
        await control('quit');
      } catch (error) {
        console.error('the session did not accept its quit request: ' + (error instanceof Error ? error.message : String(error)));
      }
      await wait(QUIT_PATIENCE_MS);
      child.kill('SIGKILL');
    }
    writeFileSync(logPath, Buffer.concat(chunks).toString());
  }
}
