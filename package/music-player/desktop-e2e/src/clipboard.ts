/**
 The clipboard of the nested Wayland display and the stub that stands in for the file manager.
 
 @example
 ```ts
 import { readClipboard, } from './clipboard.ts';
 
 const text = await readClipboard('wayland-1',);
 ```
 
 @packageDocumentation
 */

import {
  chmod,
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { runCommand, } from './command.ts';

/**
 How long `wl-paste` gets to answer.
 */
const CLIPBOARD_TIMEOUT_MS = 4_000;

/**
 Mode of the stub script: owner may run it.
 */
const EXECUTABLE_MODE = 0o755;

/**
 Reads the clipboard of a Wayland display, never the user's own display.
 
 @param display - the nested session's display name, such as `wayland-1`
 
 @returns the clipboard text, or a note that starts with `<wl-paste failed:` when `wl-paste` fails
 
 @example
 ```ts
 const text = await readClipboard('wayland-1',);
 ```
 */
export async function readClipboard(display: string,): Promise<string> {
  /**
   Result of `wl-paste`.
   */
  const result = await runCommand({
    command: 'wl-paste',
    args: ['--no-newline',],
    env: { WAYLAND_DISPLAY: display, },
    timeoutMs: CLIPBOARD_TIMEOUT_MS,
  },);
  return result.status === 0 ? result.stdout : `<wl-paste failed: ${result.stderr
    .trim()}>`;
}

/**
 Builds the text of the stub script: it appends its arguments, joined by spaces, as one line to a record file.
 
 @param record - file that receives the arguments
 
 @returns the script text, starting with a Node shebang
 
 @example
 ```ts
 stubScript('/work/xdg-open.record',);
 ```
 */
export function stubScript(record: string,): string {
  return `#!/usr/bin/env node\nrequire('node:fs').appendFileSync(${JSON.stringify(record,)}, process.argv.slice(2).join(' ') + '\\n');\n`;
}

/**
 Writes the file-manager stub: the app's "Show in file manager" runs `xdg-open`, and this one records its arguments.
 
 @param binDir - directory that comes first on the app's search path
 
 @param record - file that receives the arguments
 
 @example
 ```ts
 await writeFileManagerStub({ binDir, record, },);
 ```
 */
export async function writeFileManagerStub(
  {
    binDir,
    record,
  }: {
    readonly binDir: string;
    readonly record: string;
  },
): Promise<void> {
  await mkdir(
    binDir,
    { recursive: true, },
  );

  /**
   Path of the stub.
   */
  const stub = join(
    binDir,
    'xdg-open',
  );
  await writeFile(
    stub,
    stubScript(record,),
  );
  await chmod(
    stub,
    EXECUTABLE_MODE,
  );
}
