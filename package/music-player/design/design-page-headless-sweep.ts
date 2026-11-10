/**
 Serves the design tree on loopback, visits every page in one headless browser session, and reads back what each page
 mounted, how much text it shows, whether each typeface it sets loaded, what the session recorded as page errors, and
 every request the session made.
 */

import { spawn } from 'node:child_process';
import {
  readFileSync,
  statSync,
} from 'node:fs';
import { createServer } from 'node:http';
import type { Server } from 'node:http';
import { extname, join } from 'node:path';

import {
  SHIPPED_FONT_FAMILIES,
  designRoot,
  familyNames,
  fontFamilyValues,
  readText,
} from './design-page-dependencies.ts';

/** Content type of each file extension a page loads. */
const CONTENT_TYPES = new Map<string, string>([
  ['.css', 'text/css; charset=utf-8'],
  ['.html', 'text/html; charset=utf-8'],
  ['.js', 'text/javascript; charset=utf-8'],
  ['.json', 'application/json; charset=utf-8'],
  ['.png', 'image/png'],
  ['.svg', 'image/svg+xml'],
  ['.ttf', 'font/ttf'],
  ['.woff2', 'font/woff2'],
]);

/** Longest the whole sweep may take, in milliseconds. A visit measured about 1.2 s per page on this machine. */
const SWEEP_DEADLINE_MS = 600_000;

/** Longest closing the browser session may take, in milliseconds. */
const CLOSE_DEADLINE_MS = 60_000;

/** Path prefix answering the control fixture instead of the committed tree. */
export const CONTROL_PREFIX = '/__control__/';

/** What one page told the browser, as the page itself reports it. */
export type Probe = {
  children: number;
  text: number;
  fonts: Record<string, readonly string[]>;
};

/** One page the sweep visits, and the shipped typefaces it sets. */
export type Visit = {
  families: readonly string[];
  page: string;
};

/** One answer the browser gave in the sweep, as the batch reports it. */
export type BatchEntry = {
  error?: unknown;
  result?: {
    errors?: {
      text?: string;
    }[];
    requests?: {
      status?: number;
      url?: string;
    }[];
    result?: unknown;
  };
};

/** File the loopback server answers with, or `null` when the path is outside the tree or is not a file. */
function servedFile({ root, path, }: { root: string; path: string; },): string | null {
  const target = join(
    root,
    path,
  );
  if (!target.startsWith(root,)) {
    return null;
  }
  try {
    return statSync(target,).isFile() ? target : null;
  }
  catch {
    return null;
  }
}

/** Starts the loopback site over the committed tree, and over `controlRoot` for the control prefix. */
export async function startSite({ controlRoot, }: { controlRoot: string; },): Promise<{
  baseUrl: string;
  server: Server;
}> {
  const server = createServer(function answer(request, response,): void {
    const pathname = new URL(
      request.url ?? '/',
      'http://127.0.0.1',
    ).pathname;
    const control = pathname.startsWith(CONTROL_PREFIX,);
    const file = servedFile({
      path: control ? pathname.slice(CONTROL_PREFIX.length - 1,) : pathname,
      root: control ? controlRoot : designRoot,
    },);
    if (file === null) {
      response.writeHead(
        404,
        { 'content-type': 'text/plain; charset=utf-8' },
      );
      response.end('not found',);
      return;
    }
    const body = readFileSync(file,);
    response.writeHead(
      200,
      {
        'content-length': String(body.byteLength,),
        'content-type': CONTENT_TYPES.get(extname(file,),) ?? 'application/octet-stream',
      },
    );
    response.end(body,);
  },);
  await new Promise<void>(function listen(done,): void {
    server.listen(
      0,
      '127.0.0.1',
      function ready(): void {
        done();
      },
    );
  },);
  const address = server.address();
  const port = typeof address === 'object' && address !== null ? address.port : 0;
  return {
    baseUrl: `http://127.0.0.1:${port}`,
    server,
  };
}

/** Script a page runs to report its mounted tree, its visible text and whether each typeface it sets loads. */
function probeScript(families: readonly string[],): string {
  return `(async () => {
  const families = ${JSON.stringify(families,)};
  const mounted = document.getElementById("dc-root");
  const fonts = {};
  for (const family of families) {
    try {
      const faces = await document.fonts.load('16px "' + family + '"');
      fonts[family] = faces.map(face => face.status);
    } catch (error) {
      fonts[family] = [String(error)];
    }
  }
  return JSON.stringify({
    children: mounted === null ? -1 : mounted.children.length,
    text: (document.body.innerText || "").trim().length,
    fonts
  });
})()`;
}

/** Shipped typefaces a page names, which the browser must then load from this tree. */
export function shippedFamiliesOfPage(page: string,): readonly string[] {
  const html = readText(join(
    designRoot,
    page,
  ),) ?? '';
  const shipped: ReadonlySet<string> = new Set(SHIPPED_FONT_FAMILIES,);
  const named = new Set<string>();
  for (const value of fontFamilyValues(html,)) {
    for (const name of familyNames(value,)) {
      if (shipped.has(name,)) {
        named.add(name,);
      }
    }
  }
  return [...named].sort();
}

/** Commands that visit `visits` and read back what the session recorded. */
export function sweepCommands({ baseUrl, visits, }: {
  baseUrl: string;
  visits: readonly Visit[];
},): string[][] {
  const commands: string[][] = [];
  for (const visit of visits) {
    commands.push([
      'open',
      `${baseUrl}/${visit.page}`,
    ],);
    commands.push([
      'eval',
      probeScript(visit.families,),
    ],);
  }
  commands.push(['errors', '--json'],);
  commands.push([
    'network',
    'requests',
    '--json',
    '--filter',
    'http',
  ],);
  return commands;
}

/** Collects a child process's output and exit, killing it at `deadlineMs`. */
async function runChild({ args, deadlineMs, input, }: {
  args: readonly string[];
  deadlineMs: number;
  input?: string;
},): Promise<{
  code: number | null;
  stderr: string;
  stdout: string;
  timedOut: boolean;
}> {
  const child = spawn(
    'agent-browser',
    [...args],
    { stdio: ['pipe', 'pipe', 'pipe'], },
  );
  let stdout = '';
  let stderr = '';
  child.stdout.setEncoding('utf8',);
  child.stderr.setEncoding('utf8',);
  child.stdout.on('data', function collect(chunk: string,): void {
    stdout += chunk;
  },);
  child.stderr.on('data', function collect(chunk: string,): void {
    stderr += chunk;
  },);
  child.stdin.end(input ?? '',);
  let timedOut = false;
  const deadline = setTimeout(function stop(): void {
    timedOut = true;
    child.kill('SIGKILL',);
  }, deadlineMs,);
  const code = await new Promise<number | null>(function settle(settled,): void {
    child.on(
      'error',
      function failed(): void {
        settled(null,);
      },
    );
    child.on(
      'close',
      function closed(exit,) {
        settled(exit,);
      },
    );
  },);
  clearTimeout(deadline,);
  return {
    code,
    stderr,
    stdout,
    timedOut,
  };
}

/** Runs the whole sweep in one browser session and answers the batch entries, in command order. */
export async function runSweep({ commands, }: { commands: readonly string[][]; },): Promise<
  readonly BatchEntry[]
> {
  const session = `design-offline-${process.pid}`;
  const result = await runChild({
    args: [
      'batch',
      '--json',
      '--session',
      session,
    ],
    deadlineMs: SWEEP_DEADLINE_MS,
    input: JSON.stringify(commands,),
  },);
  await runChild({
    args: [
      'close',
      '--session',
      session,
    ],
    deadlineMs: CLOSE_DEADLINE_MS,
  },);
  if (result.timedOut) {
    throw new Error(`The browser sweep did not finish inside ${String(SWEEP_DEADLINE_MS)} ms`);
  }
  const start = result.stdout.indexOf('[',);
  const end = result.stdout.lastIndexOf(']',);
  if (start === -1 || end < start) {
    throw new Error(
      `The browser sweep printed no JSON (exit ${String(result.code)}): ${result.stdout}${result.stderr}`,
    );
  }
  const entries = JSON.parse(result.stdout.slice(
    start,
    end + 1,
  ),) as BatchEntry[];
  for (const [index, entry] of entries.entries()) {
    if (entry.error !== undefined && entry.error !== null) {
      throw new Error(`Browser command ${String(index)} failed: ${JSON.stringify(entry.error)}`);
    }
  }
  return entries;
}
