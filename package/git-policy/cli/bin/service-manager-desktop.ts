/**
 The stand-in for a desktop session and the reads of it that a reload proof judges: a service that owns a name on the
 machine's session bus, a target bound to it as the session's own units are bound to the window manager, and the two
 questions the proof asks about both.

 The harm the incident showed is this service and its target ending, not a log line, so every step of the proof reads them.
 */
import {
  containerRuntimeDirectory,
  inside,
  managerLogPath,
  type Machine,
} from './service-manager-machine.ts';

/**
 What:
  The state of the unrelated desktop service, its bound target, and the bus name it owns.
 */
export type DesktopState = {
  /**
   The service's `ActiveState`.
   */
  readonly service: string;
  /**
   The target's `ActiveState`.
   */
  readonly target: string;
  /**
   The owner of the bus name, or `none`.
   */
  readonly owner: string;
};

/**
 What:
  Starts the stand-in for a desktop: a service that owns a name on the machine's session bus, and a target bound to it as
  the session's own units are bound to the window manager.

 Why:
  The harm the incident showed is this program ending, so it has to exist before the steps that would end it, and it is
  started after any seeding of the machine, so the seeding cannot be mistaken for what an arm did.

 @param machine - the machine to give a desktop stand-in to

 @param probeDeadline - how long the stand-in's own start may take, which is the deadline a reload outlives

 @returns the desktop's state right after it came up

 @example
 ```ts
 await startDesktop({ machine, probeDeadline: '200ms' });
 ```
 */
export async function startDesktop({
  machine,
  probeDeadline,
}: {
  readonly machine: Machine;
  readonly probeDeadline: string;
}): Promise<DesktopState> {
  /**
   The unit directory the manager reads, where the stand-in's two units go.
   */
  const unitDirectory = `${containerRuntimeDirectory}/systemd/user`;
  await inside({
    machine,
    script: [
      `cat > ${unitDirectory}/desktop-probe.service <<'EOF'`,
      '[Unit]',
      'Description=Disposable desktop lifetime probe',
      '[Service]',
      'Type=dbus',
      'BusName=org.example.DesktopProbe',
      'ExecStart=/usr/sbin/dbus-test-tool echo --session --name=org.example.DesktopProbe',
      `TimeoutStartSec=${probeDeadline}`,
      'Restart=no',
      'EOF',
      `cat > ${unitDirectory}/desktop-probe.target <<'EOF'`,
      '[Unit]',
      'Description=Disposable target bound to the desktop probe',
      'Requires=desktop-probe.service',
      'BindsTo=desktop-probe.service',
      'EOF',
      'systemctl --user start desktop-probe.target',
    ].join('\n',),
  });
  /**
   The stand-in's state after it came up.
   */
  const state = await desktop({ machine, });
  console.log(`desktop stand-in: ${describe({ state, })}`);
  return state;
}

/**
 Reads the stand-in desktop's state.

 @param machine - the machine to ask

 @returns the service's state, the target's state, and the bus name's owner, or `none`

 @example
 ```ts
 await desktop({ machine }); // { service: 'active', target: 'active', owner: ':1.4' }
 ```
 */
export async function desktop({ machine, }: { readonly machine: Machine; }): Promise<DesktopState> {
  /**
   The three answers, one per line: the service's state, the target's state, and the name's owner.
   */
  const answer = await inside({
    machine,
    script: [
      'systemctl --user show --property=ActiveState --value desktop-probe.service',
      'systemctl --user show --property=ActiveState --value desktop-probe.target',
      'busctl --user call org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus GetNameOwner s org.example.DesktopProbe 2>/dev/null || echo none',
    ].join('\n',),
  });
  /**
   The answers as fields, each empty when the script printed fewer lines than expected.
   */
  const [service = '', target = '', owner = ''] = answer.split('\n',);
  return {
    service,
    target,
    owner,
  };
}

/**
 The desktop's state in one line.

 @param state - the state to describe

 @returns one line naming the service, the target and the bus name's owner

 @example
 ```ts
 describe({ state: { service: 'active', target: 'active', owner: ':1.4' } });
 ```
 */
export function describe({ state, }: { readonly state: DesktopState; }): string {
  return `service=${state.service} target=${state.target} owner=${state.owner}`;
}

/**
 Whether the desktop stand-in is alive: its service active, its target active, and a bus name owner other than `none`.

 @param state - the state to judge

 @returns true while the stand-in is up

 @example
 ```ts
 isAlive({ service: 'active', target: 'active', owner: ':1.4' }); // true
 ```
 */
export function isAlive(state: DesktopState,): boolean {
  return (state.service === 'active') && (state.target === 'active')
    && (state.owner !== 'none');
}

/**
 What:
  Every reload the fixture's manager ran, each with the time it took.

 Why:
  This is the count the arm from `main` has to show and the arm from this worktree has to show as zero: the number of
  manager-wide operations a commit asked for.

 @param machine - the machine to read

 @returns one line per reload, as the manager wrote it

 @example
 ```ts
 await reloads({ machine });
 ```
 */
export async function reloads({ machine, }: { readonly machine: Machine; }): Promise<readonly string[]> {
  /**
   The manager's own record of every reload it ran, which it writes to its log file.
   */
  const log = await inside({
    machine,
    script: `cat ${managerLogPath} 2>/dev/null || true`,
  });
  /**
   The log's lines, one per record.
   */
  const lines = log.split('\n',);
  /**
   The lines that record a finished reload.
   */
  const finished = lines.filter(recordsAReload,);
  return finished;
}

/**
 Whether one log line is the manager's record of a finished reload.

 @param line - the line to judge

 @returns true when the line records a reload the manager finished

 @example
 ```ts
 recordsAReload('Reloading finished in 487 ms.'); // true
 ```
 */
function recordsAReload(line: string,): boolean {
  return line.includes('Reloading finished in',);
}

/**
 What:
  The part of one reload record a report prints: the time it took, without the manager's own sentence around it.

 Why:
  A report lists the reloads an arm asked for, and the duration is the fact worth reading.

 @param line - the record as the manager wrote it

 @returns the record's tail from the phrase that names the duration, or the whole line when it is not there

 @example
 ```ts
 reloadDuration({ line: 'Reloading finished in 487 ms.' }); // 'reload 487 ms.'
 ```
 */
export function reloadDuration({ line, }: { readonly line: string; }): string {
  /**
   Where the manager's sentence about the duration begins.
   */
  const phrase = 'Reloading finished in ';
  /**
   Where the phrase sits in the line, or -1 when this line is not a reload record.
   */
  const at = line.lastIndexOf(phrase,);
  if (at === (-1)) return line;
  return `reload ${line.slice(at + phrase.length,)}`;
}
