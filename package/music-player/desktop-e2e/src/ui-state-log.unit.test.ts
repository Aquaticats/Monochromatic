/**
 Tests for the log reader: following a file, decoding its state lines, counting shortcut uses, and waiting for states.
 
 @module
 */

import {
  appendFile,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { createUiStateLog, } from './ui-state-log.ts';

/**
 Builds a state line as the app writes it.
 
 @param seq - sequence number
 
 @param fields - fields that replace the defaults
 
 @returns the whole log line, with a color sequence in front like the app's logger writes
 */
function stateLine(seq: number, fields: Readonly<Record<string, unknown>> = {},): string {
  const state = {
    seq,
    reason: 'key',
    route: 'Player',
    zone: 'surface',
    focus: 'play',
    unfolded: true,
    picker_open: false,
    menu_open: false,
    menu_x: 0,
    menu_y: 0,
    menu_width: 0,
    menu_height: 0,
    folder_scroll: 0,
    track_scroll: 0,
    folder: 'Aaa Tones',
    ring_track: null,
    ring_track_title: null,
    ring_folder: null,
    menu_ring: null,
    search_ring: null,
    search_query: '',
    current_index: null,
    playing: false,
    mode: 1,
    position: 0,
    duration: 0,
    settings_selected: 0,
    ring_visible: false,
    focus_request: 0,
    deck_disabled: false,
    first_run_title: '',
    badges: [],
    auto_play_on_select: false,
  text_size_pane_title_px: 20,
  text_size_group_heading_px: 20,
  text_size_folder_name_px: 14,
  text_size_track_title_px: 14,
    ...fields,
  };
  return `\u001B[32m INFO\u001B[0m music_player::ui_state_log: ui_state ${JSON.stringify(state,)}\n`;
}

/**
 Makes a scratch directory with a log file in it.
 
 @param content - initial content of the log
 
 @returns the directory and the log path
 */
async function scratchLog(content: string,): Promise<{ readonly dir: string; readonly logPath: string; }> {
  const dir = await mkdtemp(join(scratchRoot(), 'e2e-log-',),);
  const logPath = join(dir, 'wide.log',);
  await writeFile(logPath, content,);
  return { dir, logPath, };
}

await describe({
  name: createUiStateLog.name,
  children: [
    it({
      name: 'reads the latest state and its sequence number, and fails when there is none',
      fn: async ({ expect, }) => {
        const { dir, logPath, } = await scratchLog('INFO unrelated\n',);
        const log = createUiStateLog({ logPath, },);
        expect(await log.seq(),).toBe(0,);
        let caught: unknown;
        try {
          await log.last();
        }
        catch (error: unknown) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('the app has written no ui_state line yet',);
        await appendFile(logPath, stateLine(1,) + stateLine(2, { focus: 'seek', },),);
        expect((await log.last()).focus,).toBe('seek',);
        expect(await log.seq(),).toBe(2,);
        expect(log.states(),).toHaveLength(2,);
        await rm(dir, { recursive: true, force: true, },);
      },
    },),
    it({
      name: 'keeps a line that arrives in two pieces until it is complete',
      fn: async ({ expect, }) => {
        const { dir, logPath, } = await scratchLog('',);
        const log = createUiStateLog({ logPath, },);
        const line = stateLine(5,);
        await appendFile(logPath, line.slice(0, 40,),);
        expect(await log.seq(),).toBe(0,);
        await appendFile(logPath, line.slice(40,),);
        expect(await log.seq(),).toBe(5,);
        await rm(dir, { recursive: true, force: true, },);
      },
    },),
    it({
      name: 'keeps a multi-byte character that a read boundary cuts in two',
      fn: async ({ expect, }) => {
        const { dir, logPath, } = await scratchLog('',);
        const log = createUiStateLog({ logPath, },);
        const bytes = Buffer.from(stateLine(1, { folder: 'Tönes', },),);
        const cut = bytes.indexOf(0xC3,) + 1;
        await appendFile(logPath, bytes.subarray(0, cut,),);
        await log.seq();
        await appendFile(logPath, bytes.subarray(cut,),);
        expect((await log.last()).folder,).toBe('Tönes',);
        await rm(dir, { recursive: true, force: true, },);
      },
    },),
    it({
      name: 'waits for a state newer than a sequence number that satisfies the predicate',
      fn: async ({ expect, }) => {
        const { dir, logPath, } = await scratchLog(stateLine(1, { focus: 'play', },),);
        const log = createUiStateLog({ logPath, },);
        setTimeout(() => {
          void appendFile(logPath, stateLine(2, { focus: 'seek', },),);
        }, 80,);
        const state = await log.waitState({
          predicate: (candidate,) => candidate.focus === 'seek',
          what: 'focus on seek',
          after: 1,
          timeoutMs: 3_000,
        },);
        expect(state.seq,).toBe(2,);
        await rm(dir, { recursive: true, force: true, },);
      },
    },),
    it({
      name: 'ignores states at or before the sequence number',
      fn: async ({ expect, }) => {
        const { dir, logPath, } = await scratchLog(stateLine(3, { focus: 'seek', },),);
        const log = createUiStateLog({ logPath, },);
        let caught: unknown;
        try {
          await log.waitState({
            predicate: (candidate,) => candidate.focus === 'seek',
            what: 'a new seek focus',
            after: 3,
            timeoutMs: 100,
          },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect((caught as Error).message,).toContain('timed out waiting for a new seek focus; last states:',);
        await rm(dir, { recursive: true, force: true, },);
      },
    },),
    it({
      name: 'shows the latest states in the timeout message',
      fn: async ({ expect, }) => {
        const { dir, logPath, } = await scratchLog(stateLine(1,) + stateLine(2,) + stateLine(3,) + stateLine(4,),);
        const log = createUiStateLog({ logPath, },);
        let caught: unknown;
        try {
          await log.waitState({ predicate: () => false, what: 'nothing', timeoutMs: 50, },);
        }
        catch (error: unknown) {
          caught = error;
        }
        const {message} = (caught as Error);
        expect(message,).toContain('"seq":4',);
        expect(message,).toContain('"seq":2',);
        expect(message,).not.toContain('"seq":1,',);
        await rm(dir, { recursive: true, force: true, },);
      },
    },),
    it({
      name: 'counts shortcut uses, all or by action, in lines without color sequences',
      fn: async ({ expect, }) => {
        const { dir, logPath, } = await scratchLog(
          '\u001B[32m INFO\u001B[0m shortcut used action="reveal"\nINFO shortcut used action="row-details"\n',
        );
        const log = createUiStateLog({ logPath, },);
        expect(await log.used(),).toBe(2,);
        expect(await log.used('reveal',),).toBe(1,);
        expect(await log.used('none',),).toBe(0,);
        await rm(dir, { recursive: true, force: true, },);
      },
    },),
    it({
      name: 'finds the nested display name and shows the tail of the log',
      fn: async ({ expect, }) => {
        const { dir, logPath, } = await scratchLog('\u001B[2m x\u001B[0m Created new socket name=Some("wayland-4")\nlast line',);
        const log = createUiStateLog({ logPath, },);
        expect(await log.waylandDisplay(),).toBe('wayland-4',);
        expect(await log.tail(9,),).toBe('last line',);
        await rm(dir, { recursive: true, force: true, },);
      },
    },),
    it({
      name: 'reads no lines when the file does not exist',
      fn: async ({ expect, }) => {
        const log = createUiStateLog({ logPath: join(scratchRoot(), 'e2e-no-such-log.log',), },);
        expect(await log.lines(),).toEqual([],);
        expect(await log.used(),).toBe(0,);
      },
    },),
  ],
},);
