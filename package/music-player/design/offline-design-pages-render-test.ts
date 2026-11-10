/**
 Renders every music player design page in a headless browser served from this tree, and proves nothing a page shows
 came from outside it: no request leaves the loopback server, no response is missing, no page raises an error, and
 every shipped typeface a page sets actually loads from this tree.
 
 Needs `agent-browser` on `PATH`. The static half of the same promise, which runs without a browser, lives in
 `offline-design-pages-test.ts`.
 */

import {
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import type { Server } from 'node:http';
import { join } from 'node:path';

import { designPages, designRoot } from './design-page-dependencies.ts';
import {
  type Probe,
  type Visit,
  runSweep,
  shippedFamiliesOfPage,
  startSite,
  sweepCommands,
} from './design-page-headless-sweep.ts';

//region Findings

/** Fewest characters of text a rendered page shows before the render guard calls it empty. */
const MINIMUM_PAGE_TEXT = 40;

/** Browser-chosen paths that a missing file still shows as a failed response, which no page asked for. */
const IGNORED_REQUEST_PATHS = new Set<string>(['/favicon.ico'],);

/** Findings for one page, given what its probe said. */
function pageFindings({ probe, visit, }: { probe: Probe; visit: Visit; },): readonly string[] {
  const findings: string[] = [];
  if (probe.children < 1) {
    findings.push(`${visit.page}: mounted ${probe.children} elements, so the page rendered nothing`);
  }
  if (probe.text < MINIMUM_PAGE_TEXT) {
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

/** Findings for the requests one session made, given the origin this tree answers on. */
function requestFindings({ origin, recorded, }: {
  origin: string;
  recorded: readonly {
    status?: number;
    url?: string;
  }[];
},): readonly string[] {
  const findings: string[] = [];
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
  return findings;
}

//endregion Findings

//region Report

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
  const controlPage = `${'__control__'}/control.dc.html`;
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

  const origin = new URL(site.baseUrl,).origin;
  const recorded = entries[sweep.pages.length * 2 + 1]?.result?.requests ?? [];
  findings.push(...requestFindings({
    origin,
    recorded,
  },),);

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
    failures.push('control: no finding named the request the fixture makes outside this tree');
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
