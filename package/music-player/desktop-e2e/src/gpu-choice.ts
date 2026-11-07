/**
 The display of a run and, for the headless default, the render node it renders on.

 A run is headless unless a person asked for a visible window, so nothing an agent starts appears on the desktop. A
 headless compositor needs a device, and the runner does not hard-code one: it runs the compositor's `--list-gpus`, which
 opens every render node Mesa offers and names the one the compositor itself would use (`auto uses:`), and passes the
 chosen node to `--gpu`. An explicit `--gpu` never falls back, so a node that cannot be used fails the start plainly.
 The software renderer is never chosen silently; `E2E_GPU=software` names it.

 @example
 ```ts
 import { resolveDisplay, } from './gpu-choice.ts';

 const display = await resolveDisplay({ compositorPath, visible: false, gpu: '', },);
 ```

 @packageDocumentation
 */

import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { runCommand, } from './command.ts';
import { CheckFailureError, } from './errors.ts';
import type { DisplayMode, } from './session-launch.ts';
import { COLD_START_BOUND_MS, } from './timing.ts';

/**
 Prefix of a render node path in the listing.
 */
const RENDER_NODE_PREFIX = '/dev/dri/';

/**
 Text a listing line carries when Mesa lists a device but cannot open it.
 */
const UNUSABLE_MARKER = ' unusable:';

/**
 Start of the line that names the device `auto` uses.
 */
const AUTO_PREFIX = 'auto uses: ';

/**
 First word of the software renderer's line.
 */
const SOFTWARE_WORD = 'software';

/**
 What `--list-gpus` printed, read into the parts the choice needs.
 */
export type GpuListing = {
  /**
   The render node `auto uses:` names, or the empty string when it names none or names the software renderer.
   */
  readonly autoUses: string;
  /**
   Whether the software renderer is listed.
   */
  readonly hasSoftware: boolean;
  /**
   Render node paths that Mesa opened, in the order listed.
   */
  readonly usableNodes: readonly string[];
};

/**
 Reads the text `--list-gpus` prints.

 @param output - standard output of the compositor's `--list-gpus`

 @returns the usable render nodes, the node `auto` uses and whether the software renderer is listed

 @example
 ```ts
 parseGpuListing('/dev/dri/renderD128 pci=1002:7480\nauto uses: /dev/dri/renderD128 (x)',).autoUses; // '/dev/dri/renderD128'
 ```
 */
export function parseGpuListing(output: string,): GpuListing {
  /**
   The non-empty lines of the output.
   */
  const lines = output.split('\n',)
    .map(function trimmed(line,): string {
      return line.trim();
    },)
    .filter(function isNotEmpty(line,): boolean {
      return line !== '';
    },);

  /**
   The line that names the device `auto` uses, without its prefix.
   */
  const autoLine = lines.find(function isAutoLine(line,): boolean {
    return line.startsWith(AUTO_PREFIX,);
  },)
    ?.slice(AUTO_PREFIX.length,)
    ?? '';

  /**
   The first word of that line.
   */
  const autoWord = autoLine.split(' ',)[0] ?? '';
  return {
    usableNodes: lines
      .filter(function isUsableNode(line,): boolean {
        return line.startsWith(RENDER_NODE_PREFIX,) && (!line.includes(UNUSABLE_MARKER,));
      },)
      .map(function nodeOf(line,): string {
        return line.split(' ',)[0] ?? '';
      },),
    autoUses: autoWord.startsWith(RENDER_NODE_PREFIX,) ? autoWord : '',
    hasSoftware: lines.some(function isSoftwareLine(line,): boolean {
      return line.startsWith(`${SOFTWARE_WORD} `,);
    },),
  };
}

/**
 Chooses the render node of a headless run from the compositor's listing: the node `auto` uses, else the first node
 Mesa could open.

 @param output - standard output of the compositor's `--list-gpus`

 @param compositorPath - the compositor that printed it, for the message

 @returns the render node path to pass to `--gpu`

 @throws CheckFailureError when no render node could be opened, naming the compositor, everything the listing said and
 the ways out; the software renderer is not a silent fallback

 @example
 ```ts
 chooseHeadlessGpu({ output: '/dev/dri/renderD128 pci=1\nauto uses: /dev/dri/renderD128 (x)', compositorPath: '/c', },);
 ```
 */
export function chooseHeadlessGpu(
  {
    output,
    compositorPath,
  }: {
    readonly compositorPath: string;
    readonly output: string;
  },
): string {
  /**
   The listing in parts.
   */
  const listing = parseGpuListing(output,);

  /**
   The node `auto` uses, or the first usable one.
   */
  const chosen = listing.autoUses !== '' ? listing.autoUses : listing.usableNodes[0] ?? '';
  if (chosen !== '')
    return chosen;

  /**
   What the compositor printed, quoted, or the word for no text.
   */
  const printed = output.trim() === '' ? 'nothing' : `"${output.trim()}"`;

  /**
   A sentence about the software renderer when the listing has it.
   */
  const softwareNote = listing.hasSoftware
    ? 'It lists the software renderer, which this run does not choose by itself because it starts slowly. '
    : '';
  throw new CheckFailureError(
    `a headless run needs a GPU render node, and \`${compositorPath} --list-gpus\` offered none it could open `
      + `(it printed ${printed}). ${softwareNote}Ways out: set E2E_GPU to a render node path or to software `
      + '(Mesa\'s CPU renderer), or pass --visible to open a nested window on the desktop instead.',
  );
}

/**
 Asks the compositor which render devices it can open, and chooses one for a headless run.

 @param compositorPath - the compositor executable

 @returns the render node path to pass to `--gpu`

 @throws CheckFailureError when the listing command fails or times out, or offers no render node

 @example
 ```ts
 const gpu = await probeHeadlessGpu({ compositorPath: '/c/nested', },);
 ```
 */
export async function probeHeadlessGpu({ compositorPath, }: { readonly compositorPath: string; },): Promise<string> {
  /**
   Tagged logger naming the probe.
   */
  const rl = tagged({
    tag: probeHeadlessGpu.name,
    l: logger,
  },);

  /**
   Moment the listing started, in milliseconds.
   */
  const started = performance.now();

  /**
   How the listing command ended.
   */
  const result = await runCommand({
    command: compositorPath,
    args: ['--list-gpus',],
    timeoutMs: COLD_START_BOUND_MS,
  },);

  /**
   The listing on one line, for the log.
   */
  const oneLine = result.stdout
    .split('\n',)
    .join(' | ',);
  rl.info(
    `${compositorPath} --list-gpus ended with status ${result.status} after ${
      Math.round(performance.now() - started,)
    } ms: ${oneLine}`,
  );
  if (result.status !== 0) {
    /**
     What the failed command printed on each stream.
     */
    const out = result.stdout
      .trim();

    /**
     What it printed on the error stream.
     */
    const err = result.stderr
      .trim();

    /**
     Both, in words.
     */
    const printed = `printing "${out}" and error text "${err}"`;
    throw new CheckFailureError(
      `\`${compositorPath} --list-gpus\` ended with status ${result.status} (a start that could not happen, a kill, `
        + `or no answer within ${COLD_START_BOUND_MS} ms), ${printed}; set E2E_GPU to a render node path or to `
        + 'software to skip the listing, or pass --visible',
    );
  }
  return chooseHeadlessGpu({
    output: result.stdout,
    compositorPath,
  },);
}

/**
 Decides where a run's compositor draws.

 A run is headless unless `visible` is true. A failed probe is an error, never a visible window: only the request of a
 person opens a window on the desktop.

 @param compositorPath - the compositor executable, asked for its devices when none is named

 @param visible - whether a person asked for a nested window

 @param gpu - the device a person named, or the empty string to probe

 @returns the display mode of every session of the run

 @throws CheckFailureError when a headless run finds no render node

 @example
 ```ts
 const display = await resolveDisplay({ compositorPath: '/c/nested', visible: false, gpu: '', },);
 ```
 */
export async function resolveDisplay(
  {
    compositorPath,
    visible,
    gpu,
  }: {
    readonly compositorPath: string;
    readonly gpu: string;
    readonly visible: boolean;
  },
): Promise<DisplayMode> {
  if (visible)
    return { kind: 'visible', };
  if (gpu !== '')
    return {
      kind: 'headless',
      gpu,
    };
  return {
    kind: 'headless',
    gpu: await probeHeadlessGpu({ compositorPath, },),
  };
}
