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
      name: 'checks the repository link before it starts, with the same node as the daemon and a bounded wait, and retries without ending in a failed state',
      fn: async function startsOnlyWhenTheRepositoryIsThere() {
        const service = sectionLines({ text: await readFile(UNIT, 'utf8',), section: 'Service', },);
        expect(service,).toContain(
          'ExecStartPre=%h/.local/share/mise/installs/node/latest/bin/node %h/.local/share/monochromatic-dev/repository-link/check.mjs',
        );
        expect(service,).toContain('TimeoutStartSec=20s',);
        expect(service,).toContain('Restart=on-failure',);
        expect(service,).toContain('RestartSec=30',);
        expect(service.some((line,) => line.startsWith('ExecStart=%h/.local/share/mise/installs/node/latest/bin/node ',),),).toBe(true,);
      },
    },),
    it({
      name: 'never reaches a permanent failed state: no start limit in the unit section, and the check comes before the daemon',
      fn: async function neverGivesUp() {
        const text = await readFile(UNIT, 'utf8',);
        expect(sectionLines({ text, section: 'Unit', },),).toContain('StartLimitIntervalSec=0',);
        const service = sectionLines({ text, section: 'Service', },);
        const pre = service.findIndex((line,) => line.startsWith('ExecStartPre=',),);
        const start = service.findIndex((line,) => line.startsWith('ExecStart=',),);
        expect(pre,).toBeGreaterThanOrEqual(0,);
        expect(pre,).toBeLessThan(start,);
        expect(service,).not.toContain('Restart=always',);
      },
    },),
  ],
},);
