/**
 Tests for the session launch: unit names, environment, the `systemd-run` command line, and the two guards that keep a
 run away from processes that are not its own.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  cgroupBelongsToUnit,
  DEFAULT_UNIT_LIMITS,
  type DisplayMode,
  HOST_DISPLAY_VARIABLES,
  hostEnvironment,
  runRefusal,
  SESSION_IO_WEIGHT,
  sessionEnvironment,
  sessionLaunchArguments,
  type SessionOptions,
  systemdRunArguments,
  unitNameOf,
  UNIT_PREFIX,
} from './session-launch.ts';

/**
 Window and appearance used by the tests.
 */
const OPTIONS: SessionOptions = {
  width: 1_000,
  height: 800,
  scheme: 'dark',
  accent: '3B82F6',
  library: '/lib',
  label: 'wide',
};

/**
 The display mode of a run that asked for a visible window.
 */
const VISIBLE: DisplayMode = { kind: 'visible', };

/**
 The display mode of a run that asked for nothing: headless on a named render node.
 */
const HEADLESS: DisplayMode = { kind: 'headless', gpu: '/dev/dri/renderD129', };

/**
 Highest `IOWeight` a per-command cgroup may carry: the owner's rule in `CLAUDE.md` (RXS) says never above 10.
 The test states it itself so a change to the implementation's own constant cannot move it.
 */
const OWNER_MAXIMUM_IO_WEIGHT = 10;

/**
 Values a unit's arguments give one `--property` name, in order.

 @param args - arguments after `systemd-run`
 @param property - systemd property name such as `TasksMax`
 @returns the text after `--property=<name>=` of every argument that sets the property
 */
function propertyValues({ args, property, }: { readonly args: readonly string[]; readonly property: string; },): readonly string[] {
  const prefix = `--property=${property}=`;
  return args
    .filter((argument,) => argument.startsWith(prefix,),)
    .map((argument,) => argument.slice(prefix.length,),);
}

await describe({
  name: unitNameOf.name,
  children: [
    it({
      name: 'joins the prefix, the run id and the label',
      fn: async ({ expect, }) => {
        expect(unitNameOf({ runId: 4_242, label: 'wide', },),).toBe('music-player-e2e-4242-wide',);
        expect(unitNameOf({ runId: 1, label: 'visual-dark-3B82F6', },).startsWith(UNIT_PREFIX,),).toBe(true,);
      },
    },),
  ],
},);

await describe({
  name: hostEnvironment.name,
  concurrency: 1,
  children: [
    it({
      name: 'reads display, runtime directory and path, and falls back for the first two',
      fn: async ({ expect, }) => {
        const saved = { ...process.env, };
        process.env.WAYLAND_DISPLAY = 'wayland-9';
        process.env.XDG_RUNTIME_DIR = '/run/test';
        process.env.PATH = '/usr/bin';
        expect(hostEnvironment(1_234,),).toEqual({ waylandDisplay: 'wayland-9', runtimeDir: '/run/test', path: '/usr/bin', },);
        delete process.env.WAYLAND_DISPLAY;
        delete process.env.XDG_RUNTIME_DIR;
        expect(hostEnvironment(1_234,),).toEqual({
          waylandDisplay: 'wayland-0',
          runtimeDir: '/run/user/1234',
          path: '/usr/bin',
        },);
        process.env.WAYLAND_DISPLAY = saved.WAYLAND_DISPLAY ?? '';
        process.env.XDG_RUNTIME_DIR = saved.XDG_RUNTIME_DIR ?? '';
        process.env.PATH = saved.PATH ?? '';
        if (saved.WAYLAND_DISPLAY === undefined)
          delete process.env.WAYLAND_DISPLAY;
        if (saved.XDG_RUNTIME_DIR === undefined)
          delete process.env.XDG_RUNTIME_DIR;
      },
    },),
  ],
},);

await describe({
  name: sessionEnvironment.name,
  children: [
    it({
      name: 'points the app at the private sink and into scratch XDG directories',
      fn: async ({ expect, }) => {
        const env = sessionEnvironment({
          display: VISIBLE,
          sinkName: 'music_player_e2e_7',
          xdg: '/scratch/wide-xdg',
          binDir: '/scratch/bin',
          host: { waylandDisplay: 'wayland-0', runtimeDir: '/run/user/1000', path: '/usr/bin', },
        },);
        expect(env.PIPEWIRE_NODE,).toBe('music_player_e2e_7',);
        expect(env.XDG_CONFIG_HOME,).toBe('/scratch/wide-xdg/config',);
        expect(env.XDG_DATA_HOME,).toBe('/scratch/wide-xdg/data',);
        expect(env.XDG_CACHE_HOME,).toBe('/scratch/wide-xdg/cache',);
        expect(env.XDG_STATE_HOME,).toBe('/scratch/wide-xdg/state',);
        expect(env.PATH,).toBe('/scratch/bin:/usr/bin',);
        expect(env.WAYLAND_DISPLAY,).toBe('wayland-0',);
        expect(env.XDG_RUNTIME_DIR,).toBe('/run/user/1000',);
        expect(env.MUSIC_PLAYER_ACCENT,).toBe('',);
        expect(env.MUSIC_PLAYER_COLOR_SCHEME,).toBe('',);
      },
    },),
    it({
      name: 'lets a session add and replace variables',
      fn: async ({ expect, }) => {
        const env = sessionEnvironment({
          display: VISIBLE,
          sinkName: 's',
          xdg: '/x',
          binDir: '/b',
          host: { waylandDisplay: 'wayland-0', runtimeDir: '/r', path: '/p', },
          extraEnv: { RUST_LOG: 'trace', EXTRA: '1', },
        },);
        expect(env.RUST_LOG,).toBe('trace',);
        expect(env.EXTRA,).toBe('1',);
      },
    },),
  ],
},);

await describe({
  name: systemdRunArguments.name,
  children: [
    it({
      name: 'starts a user unit that holds the compositor and the app, with a memory cap and the log as output',
      fn: async ({ expect, }) => {
        const args = systemdRunArguments({
          display: VISIBLE,
          unit: 'music-player-e2e-1-wide',
          logPath: '/scratch/wide.log',
          environment: { PIPEWIRE_NODE: 'sink', },
          compositorPath: '/bin/compositor',
          socket: '/scratch/wide.sock',
          options: OPTIONS,
          releaseBinary: '/app/music-player',
          limits: DEFAULT_UNIT_LIMITS,
        },);
        expect(args.slice(0, 4,),).toEqual([
          '--user',
          '--unit=music-player-e2e-1-wide',
          '--collect',
          `--property=MemoryMax=${DEFAULT_UNIT_LIMITS.memoryMax}`,
        ],);
        expect(args,).toContain('--property=StandardOutput=append:/scratch/wide.log',);
        expect(args,).toContain('--property=StandardError=append:/scratch/wide.log',);
        expect(args,).toContain('--setenv=PIPEWIRE_NODE=sink',);
        expect(
          args.slice(args.indexOf('/bin/compositor',),),
        ).toEqual([
          '/bin/compositor',
          '--socket',
          '/scratch/wide.sock',
          '--size',
          '1000x800',
          '--color-scheme',
          'dark',
          '--accent-color',
          '3B82F6',
          '--',
          '/app/music-player',
          '/lib',
        ],);
      },
    },),
    it({
      name: 'sets memory, CPU, task and disk-weight limits on the unit, one of each',
      fn: async ({ expect, }) => {
        const args = systemdRunArguments({
          display: VISIBLE,
          unit: 'u',
          logPath: '/l',
          environment: {},
          compositorPath: '/c',
          socket: '/s',
          options: OPTIONS,
          releaseBinary: '/app',
          limits: DEFAULT_UNIT_LIMITS,
        },);
        for (const property of ['MemoryMax', 'CPUQuota', 'TasksMax', 'IOWeight',])
          expect(propertyValues({ args, property, },),).toHaveLength(1,);
      },
    },),
    it({
      name: 'never sets a disk weight above the owner limit',
      fn: async ({ expect, }) => {
        const args = systemdRunArguments({
          display: VISIBLE,
          unit: 'u',
          logPath: '/l',
          environment: {},
          compositorPath: '/c',
          socket: '/s',
          options: OPTIONS,
          releaseBinary: '/app',
          limits: DEFAULT_UNIT_LIMITS,
        },);
        const [weight,] = propertyValues({ args, property: 'IOWeight', },);
        expect(Number(weight,),).toBeGreaterThan(0,);
        expect(Number(weight,),).toBeLessThanOrEqual(OWNER_MAXIMUM_IO_WEIGHT,);
        expect(SESSION_IO_WEIGHT,).toBeLessThanOrEqual(OWNER_MAXIMUM_IO_WEIGHT,);
      },
    },),
    it({
      name: 'passes the limits it is given',
      fn: async ({ expect, }) => {
        const args = systemdRunArguments({
          display: VISIBLE,
          unit: 'u',
          logPath: '/l',
          environment: {},
          compositorPath: '/c',
          socket: '/s',
          options: OPTIONS,
          releaseBinary: '/app',
          limits: { memoryMax: '3G', cpuQuota: '700%', tasksMax: '77', },
        },);
        expect(propertyValues({ args, property: 'MemoryMax', },),).toEqual(['3G',],);
        expect(propertyValues({ args, property: 'CPUQuota', },),).toEqual(['700%',],);
        expect(propertyValues({ args, property: 'TasksMax', },),).toEqual(['77',],);
      },
    },),
    it({
      name: 'passes explicit app arguments instead of the library',
      fn: async ({ expect, }) => {
        const args = systemdRunArguments({
          display: VISIBLE,
          unit: 'u',
          logPath: '/l',
          environment: {},
          compositorPath: '/c',
          socket: '/s',
          options: { ...OPTIONS, appArgs: ['--flag', '/other',], },
          releaseBinary: '/app',
          limits: DEFAULT_UNIT_LIMITS,
        },);
        expect(args.slice(-3,),).toEqual(['/app', '--flag', '/other',],);
      },
    },),
    it({
      name: 'never carries a command that signals a process',
      fn: async ({ expect, }) => {
        const args = systemdRunArguments({
          display: VISIBLE,
          unit: 'u',
          logPath: '/l',
          environment: {},
          compositorPath: '/c',
          socket: '/s',
          options: OPTIONS,
          releaseBinary: '/app',
          limits: DEFAULT_UNIT_LIMITS,
        },);
        expect(['kill', 'pkill', 'signal',].some((word,) => args.join(' ',).includes(word,),),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: cgroupBelongsToUnit.name,
  children: [
    it({
      name: 'accepts a process in the unit service',
      fn: async ({ expect, }) => {
        const cgroup = '0::/user.slice/user-1000.slice/user@1000.service/app.slice/music-player-e2e-12-wide.service\n';
        expect(cgroupBelongsToUnit({ cgroup, unit: 'music-player-e2e-12-wide', },),).toBe(true,);
      },
    },),
    it({
      name: 'rejects the user own player, another run and a unit whose name only ends the same',
      fn: async ({ expect, }) => {
        expect(cgroupBelongsToUnit({ cgroup: '0::/user.slice/app-music-player-1234.scope\n', unit: 'music-player-e2e-12-wide', },),).toBe(false,);
        expect(cgroupBelongsToUnit({ cgroup: '0::/a/music-player-e2e-13-wide.service\n', unit: 'music-player-e2e-12-wide', },),).toBe(false,);
        expect(cgroupBelongsToUnit({ cgroup: '0::/a/music-player-e2e-112-wide.service\n', unit: 'music-player-e2e-12-wide', },),).toBe(false,);
        expect(cgroupBelongsToUnit({ cgroup: '', unit: 'music-player-e2e-12-wide', },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: runRefusal.name,
  children: [
    it({
      name: 'gives the empty string when no run is active',
      fn: async ({ expect, }) => {
        expect(runRefusal('',),).toBe('',);
        expect(runRefusal('  \n',),).toBe('',);
      },
    },),
    it({
      name: 'refuses while another run is listed, naming its units',
      fn: async ({ expect, }) => {
        const listing = 'music-player-e2e-99-wide.service loaded active running x\n';
        expect(runRefusal(listing,),).toBe(
          `another end-to-end run is active, and this one must run alone:\n${listing.trim()}`,
        );
      },
    },),
  ],
},);

/**
 What the display variables of the host are named in the environment of a process that would open a window on it.
 The test states them itself, so a change to the implementation's own list cannot shrink the claim.
 */
const HOST_DISPLAY_NAMES = ['WAYLAND_DISPLAY', 'DISPLAY', 'WAYLAND_SOCKET', 'XAUTHORITY',] as const;

/**
 Parts of a full launch that the tests below inspect.
 */
function launchOf(display: DisplayMode,): readonly string[] {
  return sessionLaunchArguments({
    display,
    unit: 'music-player-e2e-1-wide',
    logPath: '/scratch/wide.log',
    sinkName: 'sink',
    xdg: '/scratch/wide-xdg',
    binDir: '/scratch/bin',
    host: { waylandDisplay: 'wayland-0', runtimeDir: '/run/user/1000', path: '/usr/bin', },
    compositorPath: '/bin/compositor',
    socket: '/scratch/wide.sock',
    options: OPTIONS,
    releaseBinary: '/app/music-player',
    limits: DEFAULT_UNIT_LIMITS,
  },);
}

await describe({
  name: 'a headless launch',
  children: [
    it({
      name: 'starts the compositor headless on the chosen node, before the app command',
      fn: async ({ expect, }) => {
        const args = launchOf(HEADLESS,);
        const compositor = args.indexOf('/bin/compositor',);
        const separator = args.indexOf('--',);
        expect(args.slice(compositor + 1, compositor + 4,),).toEqual(['--headless', '--gpu', '/dev/dri/renderD129',],);
        expect(compositor,).toBeGreaterThan(-1,);
        expect(args.indexOf('--headless',),).toBeLessThan(separator,);
        expect(args.slice(separator + 1,),).toEqual(['/app/music-player', '/lib',],);
      },
    },),
    it({
      name: 'gives the unit no variable that names the display of the host, and removes the ones the user manager holds',
      fn: async ({ expect, }) => {
        const args = launchOf(HEADLESS,);
        for (const name of HOST_DISPLAY_NAMES)
          expect(args.some((argument,) => argument.startsWith(`--setenv=${name}=`,),),).toBe(false,);
        const [unset,] = propertyValues({ args, property: 'UnsetEnvironment', },);
        expect(unset,).toBeDefined();
        for (const name of HOST_DISPLAY_NAMES)
          expect((unset ?? '').split(' ',),).toContain(name,);
        expect([...HOST_DISPLAY_VARIABLES,].toSorted(),).toEqual([...HOST_DISPLAY_NAMES,].toSorted(),);
      },
    },),
    it({
      name: 'keeps the runtime directory, where the nested socket lives',
      fn: async ({ expect, }) => {
        expect(launchOf(HEADLESS,),).toContain('--setenv=XDG_RUNTIME_DIR=/run/user/1000',);
      },
    },),
  ],
},);

await describe({
  name: 'a visible launch, which only an explicit request gives',
  children: [
    it({
      name: 'names the display of the host and starts the compositor without --headless',
      fn: async ({ expect, }) => {
        const args = launchOf(VISIBLE,);
        expect(args,).toContain('--setenv=WAYLAND_DISPLAY=wayland-0',);
        expect(args,).not.toContain('--headless',);
        expect(args,).not.toContain('--gpu',);
        expect(propertyValues({ args, property: 'UnsetEnvironment', },),).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: `${sessionEnvironment.name} without a visible window`,
  children: [
    it({
      name: 'carries none of the display variables of the host, even when the host has them',
      fn: async ({ expect, }) => {
        const env = sessionEnvironment({
          display: HEADLESS,
          sinkName: 's',
          xdg: '/x',
          binDir: '/b',
          host: { waylandDisplay: 'wayland-0', runtimeDir: '/r', path: '/p', },
        },);
        for (const name of HOST_DISPLAY_NAMES)
          expect(Object.keys(env,),).not.toContain(name,);
        expect(env.XDG_RUNTIME_DIR,).toBe('/r',);
      },
    },),
    it({
      name: 'does not let a session variable bring the host display back',
      fn: async ({ expect, }) => {
        const env = sessionEnvironment({
          display: HEADLESS,
          sinkName: 's',
          xdg: '/x',
          binDir: '/b',
          host: { waylandDisplay: 'wayland-0', runtimeDir: '/r', path: '/p', },
          extraEnv: { WAYLAND_DISPLAY: 'wayland-5', DISPLAY: ':0', },
        },);
        for (const name of HOST_DISPLAY_NAMES)
          expect(Object.keys(env,),).not.toContain(name,);
      },
    },),
  ],
},);
