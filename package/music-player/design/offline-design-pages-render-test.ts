/**
 Renders every music player design page in a headless browser served from this tree, and proves nothing a page shows
 came from outside it: no request leaves the loopback server, no response is missing, no page raises an error, and
 every shipped typeface a page sets actually loads from this tree.
 
 Needs `agent-browser` on `PATH`. The static half of the same promise, which runs without a browser, lives in
 `offline-design-pages-test.ts`.
 */

import { spawn } from 'node:child_process';
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { createServer } from 'node:http';
import type { Server } from 'node:http';
import { extname, join } from 'node:path';

import {
  SHIPPED_FONT_FAMILIES,
  designPages,
  designRoot,
  familyNames,
  fontFamilyValues,
  readText,
} from './design-page-dependencies.ts';

//region Serving the tree

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
const CONTROL_PREFIX = '/__control__/';

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
async function startSite({ controlRoot, }: { controlRoot: string; },): Promise<{
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

//endregion Serving the tree

//region Probing a page

/** What one page told the browser, as the page itself reports it. */
type Probe = {
  children: number;
  text: number;
  fonts: Record<string, readonly string[]>;
};

/** One page the sweep visits, and the shipped typefaces it sets. */
type Visit = {
  families: readonly string[];
  page: string;
};

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
function shippedFamiliesOfPage(page: string,): readonly string[] {
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

/** One answer the browser gave in the sweep, as the batch reports it. */
type BatchEntry = {
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

/** Commands that visit every page and the control fixture, and read back what the session recorded. */
function sweepCommands({ baseUrl, controlPage, visits, }: {
  baseUrl: string;
  controlPage: string;
  visits: readonly Visit[];
},): {
  commands: string[][];
  pages: readonly Visit[];
} {
  const pages = [
    ...visits,
    {
      families: [],
      page: controlPage,
    },
  ];
  const commands: string[][] = [];
  for (const visit of pages) {
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
  return {
    commands,
    pages,
  };
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
async function runSweep({ commands, }: { commands: readonly string[][]; },): Promise<readonly BatchEntry[]> {
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

//endregion Probing a page

//region Report

/** Findings for one page, given what its probe said. */
function pageFindings({ probe, visit, }: { probe: Probe; visit: Visit; },): readonly string[] {
  const findings: string[] = [];
  if (probe.children < 1) {
    findings.push(`${visit.page}: mounted ${probe.children} elements, so the page rendered nothing`);
  }
  if (probe.text < 40) {
    findings.push(`${visit.page}: shows ${probe.text} characters of text`);
  }
  for (const family of visit.families) {
    const statuses = probe.fonts[family] ?? [];
    if (statuses.length === 0) {
      findings.push(`${visit.page}: sets ${family} and the browser found no such face to load`);
      continue;
    }
    const failed = statuses.filter(function loaded(status,): boolean {
      return status !== 'loaded';
    },);
    if (failed.length > 0) {
      findings.push(`${visit.page}: sets ${family} and its face is ${failed.join(', ')}`);
    }
  }
  return findings;
}

/** Browser-chosen paths a missing file still shows as a failed response, which no page asked for. */
const IGNORED_REQUEST_PATHS = new Set<string>(['/favicon.ico'],);

const scratchBase = join(
  designRoot,
  'node_modules',
  '.monochromatic',
  'music-player-design',
);
mkdirSync(
  scratchBase,
  { recursive: true, },
);
const scratch = mkdtempSync(join(
  scratchBase,
  'offline-render-',
),);
let site: {
  baseUrl: string;
  server: Server;
} | null = null;
try {
  //region Control fixture

  /**
   A page with no runtime mount that asks a host for a stylesheet stands for both defects this guard exists for: a
   page that mounts nothing, and a request leaving this tree.
   */
  const controlPage = '__control__/control.dc.html';
  const controlRoot = join(
    scratch,
    '__control__',
  );
  mkdirSync(
    controlRoot,
    { recursive: true, },
  );
  writeFileSync(
    join(
      controlRoot,
      'control.dc.html',
    ),
    `<!DOCTYPE html>
<html><head>
<link href="https://fonts.googleapis.com/css2?family=Roboto&display=swap" rel="stylesheet">
</head><body><x-dc><div style="font-family:sans-serif">control</div></x-dc></body></html>
`,
    'utf8',
  );

  //endregion Control fixture

  //region Sweep

  site = await startSite({ controlRoot, },);
  const visits: Visit[] = designPages(designRoot,).map(function visit(page,): Visit {
    return {
      families: shippedFamiliesOfPage(page,),
      page,
    };
  },);
  const sweep = sweepCommands({
    baseUrl: site.baseUrl,
    controlPage,
    visits,
  },);
  const entries = await runSweep({ commands: sweep.commands, },);

  //endregion Sweep

  //region Findings

  const failures: string[] = [];
  const findings: string[] = [];
  for (const [index, visit] of sweep.pages.entries()) {
    const answer = entries[index * 2 + 1]?.result?.result;
    if (typeof answer !== 'string') {
      findings.push(`${visit.page}: the browser answered ${JSON.stringify(answer,)}`);
      continue;
    }
    findings.push(...pageFindings({
      probe: JSON.parse(answer,) as Probe,
      visit,
    },),);
  }

  for (const error of entries[sweep.pages.length * 2]?.result?.errors ?? []) {
    findings.push(`a page raised ${String(error.text,)}`);
  }

  const recorded = entries[sweep.pages.length * 2 + 1]?.result?.requests ?? [];
  const origin = new URL(site.baseUrl,).origin;
  for (const request of recorded) {
    const url = request.url ?? '';
    if (!url.startsWith(origin,)) {
      findings.push(`a page asked ${url} outside this tree`);
      continue;
    }
    const path = new URL(url,).pathname;
    if ((request.status ?? 0) >= 400 && !IGNORED_REQUEST_PATHS.has(path,)) {
      findings.push(`${path} answered ${String(request.status,)}`);
    }
  }

  //region Controls

  const controlProbe = entries[(sweep.pages.length - 1) * 2 + 1]?.result?.result;
  const controlFindings = typeof controlProbe === 'string'
    ? pageFindings({
      probe: JSON.parse(controlProbe,) as Probe,
      visit: {
        families: [],
        page: controlPage,
      },
    },)
    : [];
  if (controlFindings.length < 2) {
    failures.push(`control: the fixture that mounts nothing reported ${controlFindings.length} findings`);
  }
  if (!findings.some(function remote(finding,): boolean {
    return finding.includes('fonts.googleapis.com',);
  },)) {
    failures.push('control: a request outside this tree was reported for no page, so the request record proves nothing');
  }
  if (!recorded.some(function vendored(request,): boolean {
    return (request.url ?? '').includes('/vendor/',);
  },)) {
    failures.push(`control: no request asked this tree for a vendored build in ${recorded.length} requests`);
  }

  //endregion Controls

  const lines = [
    ...failures,
    ...findings,
  ];
  if (lines.length > 0) {
    throw new Error(`Offline design pages in a browser: ${lines.length} findings\n${lines.join('\n')}`);
  }
  console.log(
    `Offline design pages in a browser: ${visits.length} pages rendered, ${recorded.length} requests all answered inside this tree, every shipped face loaded.`,
  );

  //endregion Findings
}
finally {
  site?.server.close();
  rmSync(
    scratch,
    { recursive: true, force: true, },
  );
}

//endregion Report
