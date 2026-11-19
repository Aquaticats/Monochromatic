import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 Path of the unit file the repository keeps; the installed copy in `~/.config/systemd/user` is a copy of it.
 */
const UNIT = join(import.meta.dirname, '..', 'systemd', 'claude-cpu-watch.service',);

/**
 The `key=value` lines of one section of a unit file, in order.

 @param text - the unit file
 
 @param section - the section name without brackets
 
 @returns the lines of that section that hold an assignment
 */
function sectionLines(
  {
    text,
    section,
  }: {
    readonly text: string;
    readonly section: string;
  },
): readonly string[] {
  /**
   The text after the section header, up to the next header.
   */
  const afterHeader = text.split(`[${section}]\n`,)[1] ?? '';
  return (afterHeader.split('\n[',)[0] ?? '').split('\n',)
    .filter(function isAssignment(line,) {
      return line.includes('=',) && (!line.startsWith('#',));
    },);
}

await describe({
  name: 'claude-cpu-watch.service',
  children: [
    it({
      name: 'starts the daemon from this checkout with the node on the user manager\'s PATH, and restarts it thirty seconds after a failure',
      fn: async function startsWithTheNodeOnThePath() {
        const service = sectionLines({ text: await readFile(UNIT, 'utf8',), section: 'Service', },);
        expect(service,).toContain(
          'ExecStart=/usr/bin/env node %h/Monochromatic/package/claude-cpu-cap/daemon/src/cpu-watch.ts',
        );
        expect(service,).toContain('Restart=on-failure',);
        expect(service,).toContain('RestartSec=30',);
      },
    },),
    it({
      name: 'never reaches a permanent failed state: no start limit in the unit section, no check before the daemon, and no restart of a clean exit',
      fn: async function neverGivesUp() {
        const text = await readFile(UNIT, 'utf8',);
        expect(sectionLines({ text, section: 'Unit', },),).toContain('StartLimitIntervalSec=0',);
        const service = sectionLines({ text, section: 'Service', },);
        expect(service.some((line,) => line.startsWith('ExecStartPre=',),),).toBe(false,);
        expect(service,).not.toContain('Restart=always',);
      },
    },),
  ],
},);
