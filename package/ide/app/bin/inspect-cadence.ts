#!/usr/bin/env node
// Record how the IDE's picture changes, frame by frame, while a scroll or a held minimap drag runs.
//
// The IDE binary of the chosen profile runs in the repository's nested compositor over a disposable copy of a project.
// The compositor records its screen at a steady 60 Hz whether or not the application drew (`record DIR 60 bmp`), so a
// recorded frame equal to the one before it during the input means the picture did not move for that tick. This script
// sends the input, hashes every recorded frame, and prints how many frames in the input window changed or stayed
// identical, the longest run of identical frames, and every run. The pattern analysis is in cadence.ts (tested in
// cadence.test.ts); this file only drives the compositor.
//
// A debug build of the dependencies draws too slowly to keep the cadence, a release build keeps it: that pair is the
// positive control of this check. Read a difference only when it exceeds the spread of repeated runs of one build.
import { spawn } from 'node:child_process';
import { createConnection } from 'node:net';
import { closeSync, existsSync, mkdirSync, mkdtempSync, openSync, readdirSync, readFileSync, realpathSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { homedir, tmpdir } from 'node:os';
import { join, resolve, sep } from 'node:path';
import { setTimeout as wait } from 'node:timers/promises';
import { compositor, demandCurrentCompositor } from './nested-compositor.ts';
import { analyzeCadence, formatCadence, frameHash, inputWindow, recordFps } from './cadence.ts';

// region arguments
// The task passes its arguments as usage_* variables; optional tuning comes from IDE_CADENCE_* variables.
const scenario = process.env.usage_scenario;
if (scenario !== 'scroll' && scenario !== 'drag') throw new Error('scenario must be scroll or drag, not ' + String(scenario));
const profile = process.env.usage_profile ?? 'debug';
if (profile !== 'debug' && profile !== 'release') throw new Error('profile must be debug or release, not ' + profile);
const size = process.env.usage_size ?? '1100x660';
if (!/^[1-9][0-9]*x[1-9][0-9]*$/.test(size)) throw new Error('size must be WIDTHxHEIGHT');
const [widthText = '', heightText = ''] = size.split('x');
const width = Number(widthText);
const height = Number(heightText);
const whole = ({ name, text, minimum = 1 }: { name: string; text: string; minimum?: number }): number => {
  const value = Number(text);
  if (!Number.isInteger(value) || value < minimum) throw new Error(name + ' must be a whole number of at least ' + String(minimum) + ', not ' + String(text));
  return value;
};
const steps = whole({ name: 'steps', text: process.env.usage_steps ?? '120' });
const stepMs = whole({ name: 'IDE_CADENCE_STEP_MS', text: process.env.IDE_CADENCE_STEP_MS ?? '16' });
const settleMs = whole({ name: 'IDE_CADENCE_SETTLE', text: process.env.IDE_CADENCE_SETTLE ?? '10000', minimum: 0 });
// The pointer grabs the minimap viewport box 55 pixels from the right edge, inside the strip the minimap fills.
const grab = whole({ name: 'IDE_CADENCE_GRAB', text: process.env.IDE_CADENCE_GRAB ?? '60', minimum: 0 });
const dragEnd = Math.min(grab + whole({ name: 'IDE_CADENCE_DISTANCE', text: process.env.IDE_CADENCE_DISTANCE ?? '400' }), height - 1);
const scrollDelta = whole({ name: 'IDE_CADENCE_SCROLL', text: process.env.IDE_CADENCE_SCROLL ?? '3000' });
if (!process.env.usage_project || !process.env.usage_file) throw new Error('Name the project directory and the file in it');
// endregion

// region disposable inputs
// As inspect:native: language servers run confined and cannot write into the project, and a directory below the
// system temporary directory is required so that a session never points real servers at a real project.
const project = realpathSync(process.env.usage_project);
if (!project.startsWith(realpathSync(tmpdir()) + sep)) throw new Error('The project must be a disposable directory below ' + tmpdir());
const sourceFile = realpathSync(resolve(project, process.env.usage_file));
if (!sourceFile.startsWith(project + sep)) throw new Error('The file must be inside the project');
const binary = resolve('target', profile, 'monochromatic-ide');
if (!existsSync(binary)) throw new Error('Missing ' + binary + '; the inspect:cadence task builds it');
demandCurrentCompositor();
const privateRoot = join(homedir(), 'temp', 'agent');
mkdirSync(privateRoot, { recursive: true, mode: 0o700 });
if ((statSync(privateRoot).mode & 0o077) !== 0) throw new Error('Scratch root must exclude group and other permissions: ' + privateRoot);
const artifact = mkdtempSync(join(privateRoot, 'ide-cadence-'));
for (const name of ['config', 'cache', 'data']) mkdirSync(join(artifact, name));
const fontConfig = join(artifact, 'fonts.conf');
writeFileSync(fontConfig, '<fontconfig><dir>/usr/share/fonts</dir><cachedir prefix="xdg">fontconfig</cachedir></fontconfig>');
const socket = join(artifact, 'nws.sock');
const framesDirectory = join(artifact, 'frames');
// endregion

// region session
const control = (line: string): Promise<string> => new Promise((resolveReply, reject) => {
  const connection = createConnection(socket);
  let reply = '';
  connection.on('error', reject);
  connection.on('close', () => reject(new Error('closed without a reply to ' + line)));
  connection.on('data', chunk => {
    reply += chunk;
    if (reply.includes('\n')) {
      connection.end();
      resolveReply(reply.trim());
    }
  });
  connection.write(line + '\n');
});
const log = openSync(join(artifact, 'run.log'), 'w');
const environment = {
  ...process.env,
  HELIX_RUNTIME: resolve('target', profile, 'runtime'),
  SLINT_BACKEND: 'winit',
  RUST_LOG: process.env.RUST_LOG ?? 'warn',
  FONTCONFIG_FILE: fontConfig,
  XDG_CONFIG_HOME: join(artifact, 'config'),
  XDG_CACHE_HOME: join(artifact, 'cache'),
  XDG_DATA_HOME: join(artifact, 'data'),
};
const host = spawn(compositor, ['--socket', socket, '--size', size, '--scale', '1', '--color-scheme', 'dark', '--', binary, project, '--file', sourceFile], { env: environment, stdio: ['ignore', log, log] });
const exited = new Promise(done => host.on('exit', done));
// What the compositor reported and when, collected while the input runs and read by the report.
interface CadenceRun {
  record: string;
  command: string;
  commandReply: string;
  inputMs: number;
  offsetMs: number;
  recordStop: string;
}
let run: CadenceRun | undefined;
try {
  const deadline = Date.now() + 120000;
  while (true) {
    if (existsSync(socket) && (await control('ping').catch(() => 'no')).startsWith('ok')) break;
    if (Date.now() > deadline) throw new Error('The compositor never answered ping');
    if (host.exitCode !== null) throw new Error('The compositor ended early; read ' + join(artifact, 'run.log'));
    await wait(300);
  }
  await wait(settleMs);
  await control('screenshot ' + join(artifact, 'before.png'));
  const record = await control('record ' + framesDirectory + ' ' + String(recordFps) + ' bmp');
  const recordStartedAt = Date.now();
  await wait(500);
  const command = scenario === 'scroll'
    ? 'scroll ' + String(Math.round(width / 3)) + ' ' + String(Math.round(height / 2)) + ' 0 ' + String(scrollDelta) + ' steps=' + String(steps) + ' step-ms=' + String(stepMs)
    : 'drag ' + String(width - 55) + ' ' + String(grab) + ' ' + String(width - 55) + ' ' + String(dragEnd) + ' left steps=' + String(steps) + ' step-ms=' + String(stepMs);
  const sentAt = Date.now();
  const commandReply = await control(command);
  const inputMs = Date.now() - sentAt;
  const offsetMs = sentAt - recordStartedAt;
  await wait(700);
  const recordStop = await control('record stop');
  run = { record, command, commandReply, inputMs, offsetMs, recordStop };
  await control('screenshot ' + join(artifact, 'after.png'));
  await control('quit');
} finally {
  const ended = await Promise.race([exited.then(() => true), wait(10000).then(() => false)]);
  if (!ended) host.kill('SIGTERM');
  closeSync(log);
}
// endregion

// region report
// The try block above assigns the run; a failure inside it ends the script before this point.
if (run === undefined) throw new Error('The compositor session ended without a recorded run');
const names = readdirSync(framesDirectory).filter(name => name.endsWith('.bmp')).sort();
const hashes = names.map(name => frameHash(readFileSync(join(framesDirectory, name))));
const window = inputWindow({ offsetMs: run.offsetMs, inputMs: run.inputMs });
const cadence = analyzeCadence({ hashes, window });
if (!process.env.IDE_CADENCE_KEEP_FRAMES) rmSync(framesDirectory, { recursive: true, force: true });
writeFileSync(join(artifact, 'hashes.json'), JSON.stringify(hashes));
console.log('scenario ' + scenario + ', ' + profile + ' build, window ' + size + ', file ' + sourceFile);
console.log('command: ' + run.command + ' -> ' + run.commandReply + ' (' + String(run.inputMs) + ' ms)');
console.log('record: ' + run.record + '; stop: ' + run.recordStop);
for (const line of formatCadence(cadence)) console.log(line);
console.log('artifact: ' + artifact);
console.log('CADENCE ' + JSON.stringify({ scenario, profile, ...cadence, pattern: undefined, command: run.command, inputMs: run.inputMs, recordStop: run.recordStop }));
if (hashes.length === 0) throw new Error('The recording holds no frame; read ' + join(artifact, 'run.log'));
// endregion
