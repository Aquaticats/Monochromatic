import {
  lstat,
  mkdtemp,
  readdir,
  readFile,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  confirmSentinelRunning,
  sentinelUnitName,
  startSentinelOrReport,
  systemdRunArguments,
  type SentinelLaunch,
} from '../dist/final/node/bypass-watch-sentinel-launch.mjs';
import {
  pathBreaksEnv,
  unitStateMeansGone,
} from '../dist/final/node/bypass-watch-sentinel-service.mjs';
import {
  readSentinelMarker,
  sentinelMarkerPath,
  writeSentinelMarker,
  type SentinelMarkerRead,
} from '../dist/final/node/bypass-watch-sentinel-marker.mjs';

/**
 A read that found nothing.
 */
const NOT_FOUND: SentinelMarkerRead = { found: false, };

/**
 A read that found a marker of the watcher under test.
 */
const FOUND: SentinelMarkerRead = {
  found: true,
  marker: {
    watcherPid: 4_242,
    pid: 777,
    startTime: '5',
  },
};

/**
 Identity of the watcher the fake launches name.
 */
const IDENTITY = {
  ownerId: 'owner-1',
  pid: 4_242,
  startTime: '9001',
} as const;

/**
 Path of the persisted state in the fake launches.
 */
const STATE_PATH = '/run/wg-quicker/wgtest.json';

/**
 Builds recording launch steps.

 @param serviceFails - Whether starting as a service fails.

 @param detachedFails - Whether starting detached fails.

 @param markers - What successive reads of the marker answer,
 the last one repeating.

 @param detachedMarkers - What reads answer after the sentinel started detached,
 the same as `markers` when absent.

 @param stopError - Why the stopped unit cannot be shown to be gone,
 or nothing when stopping works.

 @param alive - Whether the sentinel process is alive.

 @returns The steps,
 the events they recorded in order,
 and the reports said.

 @example
 ```ts
 const { launch, events } = fakeLaunch({ serviceFails: false, detachedFails: false, markers: [], alive: true });
 ```
 */
function fakeLaunch(
  {
    serviceFails,
    detachedFails,
    markers,
    detachedMarkers = markers,
    stopError,
    alive,
  }: {
    readonly serviceFails: boolean;
    readonly detachedFails: boolean;
    readonly markers: readonly SentinelMarkerRead[];
    readonly detachedMarkers?: readonly SentinelMarkerRead[];
    readonly stopError?: string;
    readonly alive: boolean;
  },
) {
  /**
   Events in the order they happened.
   */
  const events: string[] = [];
  /**
   Reports said.
   */
  const reports: string[] = [];
  /**
   Marker reads so far.
   */
  const reads = {
    count: 0,
    sinceDetached: 0,
    detached: false,
  };
  const launch: SentinelLaunch = {
    startAsService: ({ args, },) => {
      events.push(`service ${args.join(' ',)}`,);
      return serviceFails ? Promise.reject(new Error('systemd-run exited 1',),) : Promise.resolve();
    },
    stopService: ({ unit, },) => {
      events.push(`stop ${unit}`,);
      return stopError === undefined ? Promise.resolve() : Promise.reject(new Error(stopError,),);
    },
    startDetached: ({ args, },) => {
      events.push(`detached ${args.join(' ',)}`,);
      reads.detached = true;
      reads.sinceDetached = 0;
      return detachedFails ? Promise.reject(new Error('spawn failed',),) : Promise.resolve();
    },
    readMarker: () => {
      reads.count += 1;
      if (reads.detached) {
        reads.sinceDetached += 1;
        return Promise.resolve(detachedMarkers[Math.min(reads.sinceDetached - 1, detachedMarkers.length - 1,)] ?? NOT_FOUND,);
      }
      return Promise.resolve(markers[Math.min(reads.count - 1, markers.length - 1,)] ?? NOT_FOUND,);
    },
    removeMarker: () => {
      events.push('removeMarker',);
      return Promise.resolve();
    },
    isAlive: () => Promise.resolve(alive,),
    pause: () => Promise.resolve(),
    report: (message,) => {
      reports.push(message,);
    },
  };
  return {
    launch,
    events,
    reports,
    reads,
  };
}

await describe({
  name: systemdRunArguments.name,
  children: [
    it({
      name: 'starts the sentinel as its own collected service that systemd-oomd is asked to omit, behind /usr/bin/env, with the sudo identity and the script after a separator',
      fn: async () => {
        /**
         The environment of a sudo caller,
         with a variable the sentinel must not receive.
         A variable holds it because an object literal would be refused for the extra name.
         */
        const callerEnvironment = { SUDO_UID: '1000', SUDO_GID: '1001', PATH: '/should/not/travel', };
        /**
         Arguments for a watcher started through sudo.
         */
        const args = systemdRunArguments({
          identity: IDENTITY,
          scriptArguments: ['/app/bypass-watch-sentinel.mjs', STATE_PATH, 'owner-1', '4242', '9001',],
          environment: callerEnvironment,
        },);
        expect(args,).toContain('--collect',);
        expect(args,).toContain('--no-ask-password',);
        expect(args,).toContain('--unit=wg-quicker-sentinel-4242-9001',);
        expect(args,).toContain('--property=ManagedOOMPreference=omit',);
        expect(args,).toContain('--setenv=SUDO_UID=1000',);
        expect(args,).toContain('--setenv=SUDO_GID=1001',);
        expect(args.some((argument,) => argument.includes('/should/not/travel',),),).toBe(false,);
        /**
         Position of the separator.
         */
        const separator = args.indexOf('--',);
        expect(separator,).toBeGreaterThan(0,);
        expect(args.slice(separator + 1,),).toEqual(['/usr/bin/env', process.execPath, '/app/bypass-watch-sentinel.mjs', STATE_PATH, 'owner-1', '4242', '9001',],);
        expect(args.slice(0, separator,).every((argument,) => argument.startsWith('--',),),).toBe(true,);
        expect(sentinelUnitName({ identity: IDENTITY, },),).toBe('wg-quicker-sentinel-4242-9001',);
      },
    },),
    it({
      name: 'passes no sudo variable that the caller does not have',
      fn: async () => {
        /**
         Arguments for a watcher started without a sudo caller.
         */
        const args = systemdRunArguments({ identity: IDENTITY, scriptArguments: ['/app/s.mjs',], environment: {}, },);
        expect(args.some((argument,) => argument.startsWith('--setenv',),),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: pathBreaksEnv.name,
  children: [
    it({
      name: 'is true for any program path that contains an equals sign and false otherwise',
      fn: async () => {
        expect(pathBreaksEnv({ path: '/opt/a=b/node', },),).toBe(true,);
        expect(pathBreaksEnv({ path: '/opt/node=', },),).toBe(true,);
        expect(pathBreaksEnv({ path: '/opt/a b/node', },),).toBe(false,);
        expect(pathBreaksEnv({ path: '/opt/node', },),).toBe(false,);
      },
    },),
    it({
      name: 'puts the program path it is given behind env in the systemd-run arguments',
      fn: async () => {
        /**
         Arguments for a program at another path.
         */
        const args = systemdRunArguments({ identity: IDENTITY, scriptArguments: ['/app/s.mjs',], environment: {}, programPath: '/opt/other/node', },);
        expect(args.slice(args.indexOf('--',) + 1,),).toEqual(['/usr/bin/env', '/opt/other/node', '/app/s.mjs',],);
      },
    },),
  ],
},);

await describe({
  name: unitStateMeansGone.name,
  children: [
    it({
      name: 'means gone only for inactive and failed',
      fn: async () => {
        expect(unitStateMeansGone({ printed: 'inactive\n', },),).toBe(true,);
        expect(unitStateMeansGone({ printed: 'failed\n', },),).toBe(true,);
        expect(unitStateMeansGone({ printed: 'active\n', },),).toBe(false,);
        expect(unitStateMeansGone({ printed: 'activating\n', },),).toBe(false,);
        expect(unitStateMeansGone({ printed: 'deactivating\n', },),).toBe(false,);
        expect(unitStateMeansGone({ printed: '', },),).toBe(false,);
        expect(unitStateMeansGone({ printed: 'inactive active', },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: confirmSentinelRunning.name,
  children: [
    it({
      name: 'confirms a sentinel whose marker names this watcher and whose process runs, once the marker appears',
      fn: async () => {
        const marker = FOUND;
        const { launch, reads, } = fakeLaunch({ serviceFails: false, detachedFails: false, markers: [NOT_FOUND, NOT_FOUND, marker,], alive: true, },);
        expect(await confirmSentinelRunning({ statePath: STATE_PATH, watcherPid: 4_242, launch, },),).toBe(true,);
        expect(reads.count,).toBe(3,);
      },
    },),
    it({
      name: 'does not confirm when there is no marker, a marker of another watcher, or a marker whose process is gone',
      fn: async () => {
        const marker = FOUND;
        const other = { found: true, marker: { watcherPid: 1, pid: 777, startTime: '5', }, } as const;
        expect(await confirmSentinelRunning({ statePath: STATE_PATH, watcherPid: 4_242, launch: fakeLaunch({ serviceFails: false, detachedFails: false, markers: [NOT_FOUND,], alive: true, },).launch, },),).toBe(false,);
        expect(await confirmSentinelRunning({ statePath: STATE_PATH, watcherPid: 4_242, launch: fakeLaunch({ serviceFails: false, detachedFails: false, markers: [other,], alive: true, },).launch, },),).toBe(false,);
        expect(await confirmSentinelRunning({ statePath: STATE_PATH, watcherPid: 4_242, launch: fakeLaunch({ serviceFails: false, detachedFails: false, markers: [marker,], alive: false, },).launch, },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: startSentinelOrReport.name,
  children: [
    it({
      name: 'removes the old marker, starts the service, confirms it, and says nothing',
      fn: async () => {
        const marker = FOUND;
        const { launch, events, reports, } = fakeLaunch({ serviceFails: false, detachedFails: false, markers: [marker,], alive: true, },);
        const outcome = await startSentinelOrReport({ statePath: STATE_PATH, identity: IDENTITY, environment: { SUDO_UID: '1000', }, launch, },);
        expect(outcome,).toEqual({ placement: 'service', confirmed: true, },);
        expect(reports,).toEqual([],);
        expect(events[0],).toBe('removeMarker',);
        expect(events[1]?.startsWith('service --quiet --collect',),).toBe(true,);
        expect(events.some((event,) => event.startsWith('detached',),),).toBe(false,);
      },
    },),
    it({
      name: 'falls back to a detached sentinel when the service cannot start, and says that a kill of the watcher cgroup then ends both',
      fn: async () => {
        const marker = FOUND;
        const { launch, events, reports, } = fakeLaunch({ serviceFails: true, detachedFails: false, markers: [marker,], alive: true, },);
        const outcome = await startSentinelOrReport({ statePath: STATE_PATH, identity: IDENTITY, environment: {}, launch, },);
        expect(outcome,).toEqual({ placement: 'beside the watcher', confirmed: true, },);
        expect(events.some((event,) => event.startsWith('detached',),),).toBe(true,);
        expect(reports,).toHaveLength(1,);
        expect(reports[0],).toContain('POLICY NOT IN FORCE',);
        expect(reports[0],).toContain('could not start as its own service (Error: systemd-run exited 1)',);
        expect(reports[0],).toContain('systemd-oomd',);
        expect(reports[0],).toContain('ends both without a report',);
      },
    },),
    it({
      name: 'says it twice and does not wait for a marker when neither way starts a sentinel',
      fn: async () => {
        const { launch, reports, reads, } = fakeLaunch({ serviceFails: true, detachedFails: true, markers: [NOT_FOUND,], alive: true, },);
        const outcome = await startSentinelOrReport({ statePath: STATE_PATH, identity: IDENTITY, environment: {}, launch, },);
        expect(outcome,).toEqual({ confirmed: false, },);
        expect(reports,).toHaveLength(2,);
        expect(reports[1],).toContain('could not start at all (Error: spawn failed)',);
        expect(reads.count,).toBe(0,);
      },
    },),
    it({
      name: 'stops an accepted service that never confirms and starts the sentinel beside the watcher, saying where the failure is in the journal',
      fn: async () => {
        const { launch, reports, events, } = fakeLaunch({ serviceFails: false, detachedFails: false, markers: [NOT_FOUND,], detachedMarkers: [FOUND,], alive: true, },);
        const outcome = await startSentinelOrReport({ statePath: STATE_PATH, identity: IDENTITY, environment: {}, launch, },);
        expect(outcome,).toEqual({ placement: 'beside the watcher', confirmed: true, },);
        expect(reports,).toHaveLength(1,);
        expect(reports[0],).toContain('accepted as a service',);
        expect(reports[0],).toContain('did not confirm that it runs',);
        expect(reports[0],).toContain('journalctl --unit=wg-quicker-sentinel-4242-9001',);
        expect(reports[0],).not.toContain('systemctl status',);
        expect(reports[0],).toContain('ends both without a report',);
        expect(events.map((event,) => event.split(' ',)[0],),).toEqual(['removeMarker', 'service', 'stop', 'removeMarker', 'detached',],);
        expect(events.includes('stop wg-quicker-sentinel-4242-9001',),).toBe(true,);
      },
    },),
    it({
      name: 'says that two sentinels may run and a death may be reported twice when the unit cannot be shown to be stopped, and still starts the one beside the watcher',
      fn: async () => {
        const { launch, reports, events, } = fakeLaunch({
          serviceFails: false,
          detachedFails: false,
          markers: [NOT_FOUND,],
          detachedMarkers: [FOUND,],
          stopError: 'the unit is active (systemctl exited 1: timed out)',
          alive: true,
        },);
        const outcome = await startSentinelOrReport({ statePath: STATE_PATH, identity: IDENTITY, environment: {}, launch, },);
        expect(outcome,).toEqual({ placement: 'beside the watcher', confirmed: true, },);
        expect(reports,).toHaveLength(2,);
        expect(reports[1],).toContain('wg-quicker-sentinel-4242-9001 could not be shown to be stopped',);
        expect(reports[1],).toContain('the unit is active (systemctl exited 1: timed out)',);
        expect(reports[1],).toContain('two sentinels may run',);
        expect(reports[1],).toContain('reported twice',);
        expect(events.some((event,) => event.startsWith('detached',),),).toBe(true,);
      },
    },),
    it({
      name: 'refuses a program path with an equals sign as a service, because env would read it as an assignment, and starts the sentinel beside the watcher',
      fn: async () => {
        const { launch, reports, events, } = fakeLaunch({ serviceFails: false, detachedFails: false, markers: [FOUND,], alive: true, },);
        const outcome = await startSentinelOrReport({ statePath: STATE_PATH, identity: IDENTITY, environment: {}, launch, programPath: '/opt/a=b/node', },);
        expect(outcome,).toEqual({ placement: 'beside the watcher', confirmed: true, },);
        expect(reports,).toHaveLength(1,);
        expect(reports[0],).toContain("contains '='",);
        expect(reports[0],).toContain('variable assignment',);
        expect(reports[0],).toContain('/opt/a=b/node',);
        expect(reports[0],).toContain('ends both without a report',);
        expect(events.some((event,) => event.startsWith('service',) || event.startsWith('stop',),),).toBe(false,);
        expect(events.some((event,) => event.startsWith('detached',),),).toBe(true,);
      },
    },),
    it({
      name: 'says it twice when the accepted service and the detached sentinel both never confirm',
      fn: async () => {
        const { launch, reports, } = fakeLaunch({ serviceFails: false, detachedFails: false, markers: [NOT_FOUND,], alive: true, },);
        const outcome = await startSentinelOrReport({ statePath: STATE_PATH, identity: IDENTITY, environment: {}, launch, },);
        expect(outcome,).toEqual({ placement: 'beside the watcher', confirmed: false, },);
        expect(reports,).toHaveLength(2,);
        expect(reports[1],).toContain('started beside the watcher but did not confirm',);
      },
    },),
  ],
},);

await describe({
  name: 'the sentinel marker',
  children: [
    it({
      name: 'is written with the pid and start time of the running process, read back, and absent when missing or malformed',
      fn: async () => {
        /**
         Scratch directory for the state path.
         */
        const directory = await mkdtemp(join(scratchRoot(), 'wgv-marker-',),);
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(directory, { force: true, recursive: true, },);
          },
        };
        /**
         State path whose marker is tested.
         */
        const statePath = join(directory, 'wgtest.json',);
        expect(await readSentinelMarker({ statePath, },),).toEqual({ found: false, },);
        await writeSentinelMarker({ statePath, watcherPid: 4_242, },);
        /**
         The marker as written.
         */
        const marker = await readSentinelMarker({ statePath, },);
        expect(marker.found,).toBe(true,);
        expect(marker.found ? marker.marker.watcherPid : -1,).toBe(4_242,);
        expect(marker.found ? marker.marker.pid : -1,).toBe(process.pid,);
        expect(marker.found ? marker.marker.startTime.length > 0 : false,).toBe(true,);
        expect(sentinelMarkerPath({ statePath, },),).toBe(`${statePath}.sentinel.json`,);
        await writeFile(sentinelMarkerPath({ statePath, },), '{ not json',);
        expect(await readSentinelMarker({ statePath, },),).toEqual({ found: false, },);
        await writeFile(sentinelMarkerPath({ statePath, },), JSON.stringify({ watcherPid: '4242', pid: 1, startTime: '2', },),);
        expect(await readSentinelMarker({ statePath, },),).toEqual({ found: false, },);
      },
    },),
    it({
      name: 'replaces a link planted at the marker path instead of following it, and leaves no temporary file',
      fn: async () => {
        /**
         Scratch directory for the state path.
         */
        const directory = await mkdtemp(join(scratchRoot(), 'wgv-marker-link-',),);
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(directory, { force: true, recursive: true, },);
          },
        };
        /**
         State path whose marker is tested.
         */
        const statePath = join(directory, 'wgtest.json',);
        /**
         A file the planted link points at.
         */
        const victim = join(directory, 'victim',);
        await writeFile(victim, 'precious',);
        await symlink(victim, sentinelMarkerPath({ statePath, },),);
        await writeSentinelMarker({ statePath, watcherPid: 4_242, },);
        expect(await readFile(victim, 'utf8',),).toBe('precious',);
        expect((await lstat(sentinelMarkerPath({ statePath, },),)).isSymbolicLink(),).toBe(false,);
        /**
         The marker as written.
         */
        const marker = await readSentinelMarker({ statePath, },);
        expect(marker.found ? marker.marker.watcherPid : -1,).toBe(4_242,);
        expect((await readdir(directory,)).filter((name,) => name.endsWith('.tmp',),),).toEqual([],);
      },
    },),
  ],
},);
