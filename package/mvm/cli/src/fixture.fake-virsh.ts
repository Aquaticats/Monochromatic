/**
 Stand-in for `virsh` used by this package's tests.

 Run as `node fixture.fake-virsh.ts --connect <uri> <subcommand> ...`.
 It simulates the parts of libvirt and the QEMU guest agent that mvm talks to,
 keeping its state in the directory named by `MVM_FAKE_VIRSH_DIR`:

 - `scenario.json`: what the test wants to happen (see {@link Scenario}).
 - `state.json`: the simulated guest agent's process table and counters.
 - `calls.jsonl`: one JSON array of arguments per invocation.

 The guest agent's process table follows the real agent: a finished process
 stays in the table until its status is read, entries are found by process ID
 alone, oldest first, and reading a finished entry removes it.
 A launched Linux command really runs on the host through its shell, so the
 test observes real shell behavior; `powershell.exe` is emulated for
 statements that are single-quoted string literals.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  appendFile,
  mkdir,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { constants, } from 'node:os';
import { join, } from 'node:path';
import { text, } from 'node:stream/consumers';

import {
  guestFileCommand,
  isFileCommand,
} from './fixture.fake-guest-files.ts';

//region Shapes

/**
 One process the simulated agent remembers.
 */
type ProcessEntry = {
  readonly pid: number;
  /**
   Exit status; absent when a signal ended the process, as the real agent reports it.
   */
  readonly exitcode?: number;
  /**
   Number of the signal that ended the process, when one did.
   */
  readonly signal?: number;
  readonly stdout: string;
  readonly stderr: string;
  /**
   Status reads that still answer "not exited" before the result is released.
   */
  readonly runningPolls: number;
};

/**
 What a test asks the fake to do; every key is optional.

 - `staleEntries`: results left behind by earlier commands whose status was never read.
 - `pids`: process IDs handed to successive launches; counts up from 1000 when exhausted.
 - `runningPolls`: status reads answered "not exited" before a launched command's result is released.
 - `statusFailures`: standard-error texts for status reads that fail, consumed one per read before any succeeds.
 - `launchFailure`: standard-error text making every launch fail.
 - `loseLaunchedEntry`: when true a launch reports a process ID but the agent keeps no entry for it.
 - `domstate`: state reported by `domstate`; an object with a `failure` text makes it fail.
 - `domains`: list of `{ name, state }` printed by `list --all`.
 - `connectFailure`: when true every command fails the way virsh does without a session daemon.
 - `virtiofsd`: when true libvirt can serve a virtiofs share; otherwise `start` and `domxml-to-native` fail for a domain that has one.
 - `probeFailure`: standard-error text making `domxml-to-native` fail for another reason.

 The guest agent's file commands and their scenario keys are in `fixture.fake-guest-files.ts`.
 */
type Scenario = Readonly<Record<string, unknown>>;

/**
 Simulator state persisted between invocations.
 */
type State = {
  readonly processes: readonly ProcessEntry[];
  readonly launches: number;
  readonly statusReads: number;
};

/**
 Result of running one launched command.
 */
type Outcome = {
  readonly exitcode?: number;
  readonly signal?: number;
  readonly stdout: string;
  readonly stderr: string;
};

//endregion Shapes

//region Reading untyped JSON

/**
 Checks that a parsed JSON value is an object with string keys.

 @param value - Parsed JSON value

 @returns Whether `value` is a non-array object
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Checks that a parsed JSON value is a number or absent.

 @param value - Parsed JSON value

 @returns Whether `value` is a number or `undefined`
 */
function isOptionalNumber(value: unknown,): boolean {
  return (value === undefined) || ((typeof value) === 'number');
}

/**
 Checks that a parsed JSON value has the shape of a process table entry.

 @param value - Parsed JSON value

 @returns Whether `value` is a {@link ProcessEntry}
 */
function isProcessEntry(value: unknown,): value is ProcessEntry {
  return isRecord(value,)
    && ((typeof value.pid) === 'number')
    && ((typeof value.stdout) === 'string')
    && ((typeof value.stderr) === 'string')
    && ((typeof value.runningPolls) === 'number')
    && isOptionalNumber(value.exitcode,)
    && isOptionalNumber(value.signal,);
}

/**
 Reads a list of process table entries from a parsed JSON value.

 @param value - Parsed JSON value

 @returns Entries of `value` that have the entry shape; empty when `value` is not a list
 */
function processEntries(value: unknown,): readonly ProcessEntry[] {
  return Array.isArray(value,) ? value.filter(isProcessEntry,) : [];
}

/**
 Reads a number from a parsed JSON value.

 @param fallback - Number used when `value` is not a number

 @param value - Parsed JSON value

 @returns `value` when it is a number, else `fallback`
 */
function numberOr({
  fallback,
  value,
}: {
  readonly fallback: number;
  readonly value: unknown;
},): number {
  return ((typeof value) === 'number') ? value : fallback;
}

/**
 Reads one element of a list held in a parsed JSON value.

 @param index - Position in the list

 @param value - Parsed JSON value expected to be a list

 @returns Element at `index`, or `undefined` when `value` is not a list or is shorter
 */
function elementAt({
  index,
  value,
}: {
  readonly index: number;
  readonly value: unknown;
},): unknown {
  if (!Array.isArray(value,)) {
    return undefined;
  }
  /**
   Element typed as unknown; lists parsed from JSON carry no element type.
   */
  const element: unknown = value[index];
  return element;
}

/**
 Reads and parses a JSON file of the state directory.

 @param name - File name inside the state directory

 @param within - State directory

 @returns Parsed value, or `undefined` when the directory has no such file
 */
async function readJson({
  name,
  within,
}: {
  readonly name: string;
  readonly within: string;
},): Promise<unknown> {
  if (!(await readdir(within,)).includes(name,)) {
    return undefined;
  }
  /**
   Parsed file content, typed as unknown until a guard narrows it.
   */
  const parsed: unknown = JSON.parse(
    await readFile(
      join(
        within,
        name,
      ),
      'utf8',
    ),
  );
  return parsed;
}

//endregion Reading untyped JSON

//region Files

/**
 Directory holding the scenario, the state and the call log.
 */
const directory = process.env
  .MVM_FAKE_VIRSH_DIR;
if (directory === undefined) {
  throw new Error('MVM_FAKE_VIRSH_DIR is not set; the fake virsh needs its state directory',);
}

/**
 Name of the state file inside the state directory.
 */
const STATE_FILE = 'state.json';

/**
 Path of the state file inside the state directory.
 */
const statePath = join(
  directory,
  STATE_FILE,
);

/**
 Reads the scenario the test wrote, or an empty one.

 @returns Scenario for this invocation
 */
async function readScenario(): Promise<Scenario> {
  /**
   Parsed scenario file, when the test wrote one.
   */
  const parsed = await readJson({
    name: 'scenario.json',
    within: String(directory,),
  },);
  return isRecord(parsed,) ? parsed : {};
}

/**
 Reads the simulator state, seeding it from the scenario on first use.

 @param scenario - Scenario whose stale entries seed a fresh state

 @returns Current simulator state
 */
async function readState(scenario: Scenario,): Promise<State> {
  /**
   Parsed state file; absent before the first agent command.
   */
  const parsed = await readJson({
    name: STATE_FILE,
    within: String(directory,),
  },);
  if (!isRecord(parsed,)) {
    return {
      launches: 0,
      processes: processEntries(scenario.staleEntries,),
      statusReads: 0,
    };
  }
  return {
    launches: numberOr({
      fallback: 0,
      value: parsed.launches,
    },),
    processes: processEntries(parsed.processes,),
    statusReads: numberOr({
      fallback: 0,
      value: parsed.statusReads,
    },),
  };
}

/**
 Persists the simulator state for the next invocation.

 @param state - State to store
 */
async function writeState(state: State,): Promise<void> {
  await writeFile(
    statePath,
    JSON.stringify(state,),
  );
}

//endregion Files

//region Output

/**
 Prints a guest agent reply the way `virsh qemu-agent-command` does.

 @param value - Value placed under the reply's `return` key
 */
function reply(value: unknown,): void {
  process.stdout
    .write(`${JSON.stringify({ return: value, },)}\n\n`,);
}

/**
 Fails the invocation the way virsh does: text on standard error, exit status 1.

 @param stderr - Text for standard error
 */
function fail(stderr: string,): void {
  process.stderr
    .write(`${stderr}\n`,);
  process.exitCode = 1;
}

//endregion Output

//region Guest command execution

/**
 Text that makes the emulated PowerShell treat the whole script as unparseable.
 */
const POWERSHELL_PARSE_FAILURE = '<<does-not-parse>>';

/**
 Emulates Windows PowerShell for a script made of single-quoted string literals
 separated by `; `: each literal is written on its own CRLF-terminated line.
 Any other statement fails the way an unknown command does.
 A script containing {@link POWERSHELL_PARSE_FAILURE} fails before any statement runs,
 as PowerShell does for text it cannot parse.

 @param script - Text passed after `-Command`

 @returns Emulated outcome
 */
function runPowerShell(script: string,): Outcome {
  if (script.includes(POWERSHELL_PARSE_FAILURE,)) {
    return {
      exitcode: 1,
      stderr: 'ParserError: fake powershell could not parse the command\r\n',
      stdout: '',
    };
  }
  /**
   Lines written so far; kept when a later statement fails.
   */
  const lines: string[] = [];
  for (const statement of script.split('; ',)) {
    if ((statement.length < 2) || (!statement.startsWith('\'',))
      || (!statement.endsWith('\'',))) {
      return {
        exitcode: 1,
        stderr: `fake powershell: unsupported statement: ${statement}\r\n`,
        stdout: lines.join('',),
      };
    }
    lines.push(`${
      statement
        .slice(
          1,
          -1,
        )
        .replaceAll(
          '\'\'',
          '\'',
        )
    }\r\n`,);
  }
  return {
    exitcode: 0,
    stderr: '',
    stdout: lines.join('',),
  };
}

/**
 Looks up the number of a signal by its name.

 @param name - Signal name such as `SIGKILL`

 @returns Signal number, or 0 when the name is not a signal of this platform
 */
function signalNumber(name: string,): number {
  /**
   Entry of the platform's signal table with this name, when one exists.
   */
  const entry = Object
    .entries(constants.signals,)
    .find(function hasName([candidate,],) {
      return candidate === name;
    },);
  return entry === undefined ? 0 : entry[1];
}

/**
 Runs a launched command: PowerShell is emulated, anything else runs on the host.

 @param arg - Arguments of the launched program

 @param path - Program the guest was asked to run

 @returns Outcome the simulated agent will report
 */
async function runGuestProgram({
  arg,
  path,
}: {
  readonly arg: readonly string[];
  readonly path: string;
},): Promise<Outcome> {
  if (path === 'powershell.exe') {
    return runPowerShell(arg.at(-1,) ?? '',);
  }
  /**
   Real run of the program on the host, so shell semantics are the shell's own.
   */
  const child = spawn(
    path,
    [...arg,],
    {
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Exact output of both streams and the values of the child's `close` event: exit status, then signal name.
   */
  const [stdout, stderr, closed,] = await Promise.all([
    text(child.stdout,),
    text(child.stderr,),
    once(
      child,
      'close',
    ),
  ],);
  /**
   Exit status of the child, or `null` when a signal ended it.
   */
  const status: unknown = closed[0];
  /**
   Name of the signal that ended the child, or `null` on a normal exit.
   */
  const signalName: unknown = closed[1];
  if ((typeof signalName) === 'string') {
    return {
      signal: signalNumber(signalName,),
      stderr,
      stdout,
    };
  }
  return {
    exitcode: numberOr({
      fallback: 1,
      value: status,
    },),
    stderr,
    stdout,
  };
}

//endregion Guest command execution

//region Guest agent commands

/**
 Handles `guest-exec`: runs the command and remembers its result under a process ID.

 @param request - Arguments of the agent command

 @param scenario - Scenario for this invocation

 @param state - Simulator state before the launch
 */
async function guestExec({
  request,
  scenario,
  state,
}: {
  readonly request: Readonly<Record<string, unknown>>;
  readonly scenario: Scenario;
  readonly state: State;
},): Promise<void> {
  /**
   Scripted launch failure, when the scenario asks for one.
   */
  const {launchFailure} = scenario;
  if ((typeof launchFailure) === 'string') {
    fail(launchFailure,);
    return;
  }
  /**
   Process ID for this launch: scripted when the scenario lists one, else counting up.
   */
  const pid = numberOr({
    fallback: 1_000 + state.launches,
    value: elementAt({
      index: state.launches,
      value: scenario.pids,
    },),
  },);
  /**
   Arguments of the launched program; anything that is not text is ignored.
   */
  const arg = Array.isArray(request.arg,)
    ? request.arg
      .filter(function isText(value: unknown,): value is string {
        return (typeof value) === 'string';
      },)
    : [];
  /**
   Outcome the agent will report once the status is read.
   */
  const outcome = await runGuestProgram({
    arg,
    path: String(request.path,),
  },);
  await writeState({
    ...state,
    launches: state.launches + 1,
    processes: scenario.loseLaunchedEntry === true
      ? state.processes
      : [
        ...state.processes,
        {
          ...outcome,
          pid,
          runningPolls: numberOr({
            fallback: 0,
            value: scenario.runningPolls,
          },),
        },
      ],
  },);
  reply({ pid, },);
}

/**
 Handles `guest-exec-status`: oldest entry for the process ID, removed once it reports exit.

 @param pid - Process ID asked about

 @param scenario - Scenario for this invocation

 @param state - Simulator state before the read
 */
async function guestExecStatus({
  pid,
  scenario,
  state,
}: {
  readonly pid: number;
  readonly scenario: Scenario;
  readonly state: State;
},): Promise<void> {
  /**
   State with this read counted; every branch stores a variant of it.
   */
  const counted = {
    ...state,
    statusReads: state.statusReads + 1,
  };
  /**
   Scripted failure for this read, when the scenario still has one.
   */
  const failure = elementAt({
    index: state.statusReads,
    value: scenario.statusFailures,
  },);
  if ((typeof failure) === 'string') {
    await writeState(counted,);
    fail(failure,);
    return;
  }
  /**
   Position of the oldest entry with this process ID.
   */
  const index = state.processes
    .findIndex(function hasPid(entry,) {
      return entry.pid === pid;
    },);
  /**
   Oldest entry with this process ID, when one exists.
   */
  const entry = state.processes[index];
  if (entry === undefined) {
    await writeState(counted,);
    fail(
      `error: guest agent command failed: unable to execute QEMU agent command 'guest-exec-status': PID ${
        String(pid,)
      } does not exist`,
    );
    return;
  }
  if (entry.runningPolls > 0) {
    await writeState({
      ...counted,
      processes: state.processes
        .with(
        index,
        {
          ...entry,
          runningPolls: entry.runningPolls - 1,
        },
        ),
    },);
    reply({ exited: false, },);
    return;
  }
  await writeState({
    ...counted,
    processes: state.processes
      .toSpliced(
      index,
      1,
      ),
  },);
  reply({
    exited: true,
    ...(entry.exitcode === undefined ? {} : { exitcode: entry.exitcode, }),
    ...(entry.signal === undefined ? {} : { signal: entry.signal, }),
    ...(entry.stdout === ''
      ? {}
      : {
        'out-data': Buffer
          .from(entry.stdout,)
          .toString('base64',),
      }),
    ...(entry.stderr === ''
      ? {}
      : {
        'err-data': Buffer
          .from(entry.stderr,)
          .toString('base64',),
      }),
  },);
}

/**
 Handles `qemu-agent-command [--timeout N] <domain> <json>`.

 @param scenario - Scenario for this invocation

 @param tokens - Arguments after the subcommand name
 */
async function agentCommand({
  scenario,
  tokens,
}: {
  readonly scenario: Scenario;
  readonly tokens: readonly string[];
},): Promise<void> {
  /**
   Positional arguments with `--timeout <seconds>` removed: domain, then the JSON command.
   */
  const positional = tokens.filter(function isPositional(
    token,
    index,
  ) {
    return (token !== '--timeout') && (tokens[index - 1] !== '--timeout');
  },);
  /**
   Guest agent command as mvm wrote it.
   */
  const message: unknown = JSON.parse(positional[1] ?? '{}',);
  if (!isRecord(message,)) {
    fail('error: fake virsh: the agent command is not a JSON object',);
    return;
  }
  /**
   Name of the guest agent command.
   */
  const execute = String(message.execute,);
  /**
   Arguments of the guest agent command; empty when it takes none.
   */
  const request = isRecord(message.arguments,) ? message.arguments : {};
  /**
   Simulator state before this command.
   */
  const state = await readState(scenario,);
  if (execute === 'guest-ping') {
    reply({},);
    return;
  }
  if (execute === 'guest-exec') {
    await guestExec({
      request,
      scenario,
      state,
    },);
    return;
  }
  if (execute === 'guest-exec-status') {
    await guestExecStatus({
      pid: Number(request.pid,),
      scenario,
      state,
    },);
    return;
  }
  if (isFileCommand(execute,)) {
    /**
     Result of the simulated file command.
     */
    const outcome = await guestFileCommand({
      directory: String(directory,),
      execute,
      request,
      scenario,
    },);
    if ('failure' in outcome) {
      fail(outcome.failure,);
    }
    else {
      reply(outcome.reply,);
    }
    return;
  }
  fail(
    `error: guest agent command failed: unable to execute QEMU agent command '${execute}': The command ${execute} has not been found`,
  );
}

//endregion Guest agent commands

//region Domain definitions

/**
 Text of libvirt's error when a domain has a virtiofs share and no virtiofsd exists, recorded on this host.
 */
const NO_VIRTIOFSD = 'error: operation failed: Unable to find a satisfying virtiofsd';

/**
 Directory holding the XML of every domain defined so far.
 */
const definedDirectory = join(
  directory,
  'defined',
);

/**
 Handles `define <xml file>`: remembers the domain's XML under its name.

 @param xmlPath - File holding the domain XML
 */
async function defineDomain(xmlPath: string,): Promise<void> {
  /**
   Domain XML as mvm wrote it.
   */
  const xml = await readFile(
    xmlPath,
    'utf8',
  );
  /**
   Domain name: the text of the first `name` element.
   */
  const name = xml.slice(
    xml.indexOf('<name>',) + '<name>'.length,
    xml.indexOf('</name>',),
  );
  await mkdir(
    definedDirectory,
    { recursive: true, },
  );
  await writeFile(
    join(
      definedDirectory,
      `${name}.xml`,
    ),
    xml,
  );
  process.stdout
    .write(`Domain '${name}' defined from ${xmlPath}\n\n`,);
}

/**
 Handles `start <domain>`: fails the way libvirt does when the domain has a share nothing can serve.

 @param domain - Domain name

 @param scenario - Scenario saying whether a virtiofsd exists
 */
async function startDomain({
  domain,
  scenario,
}: {
  readonly domain: string;
  readonly scenario: Scenario;
},): Promise<void> {
  /**
   XML the domain was defined with.
   */
  const xml = await readFile(
    join(
      definedDirectory,
      `${domain}.xml`,
    ),
    'utf8',
  );
  if (xml.includes('virtiofs',) && (scenario.virtiofsd !== true)) {
    fail(`error: Failed to start domain '${domain}'\n${NO_VIRTIOFSD}`,);
    return;
  }
  process.stdout
    .write(`Domain '${domain}' started\n\n`,);
}

/**
 Handles `domxml-to-native qemu-argv --xml <file>`: converts without defining anything,
 and fails like a start would when the XML has a share nothing can serve.

 @param scenario - Scenario saying whether a virtiofsd exists; `probeFailure` makes the conversion fail with that text

 @param xmlPath - File holding the domain XML
 */
async function convertDomain({
  scenario,
  xmlPath,
}: {
  readonly scenario: Scenario;
  readonly xmlPath: string;
},): Promise<void> {
  /**
   Scripted failure of the conversion, when the scenario asks for one.
   */
  const { probeFailure, } = scenario;
  if ((typeof probeFailure) === 'string') {
    fail(probeFailure,);
    return;
  }
  /**
   Domain XML to convert.
   */
  const xml = await readFile(
    xmlPath,
    'utf8',
  );
  if (xml.includes('virtiofs',) && (scenario.virtiofsd !== true)) {
    fail(NO_VIRTIOFSD,);
    return;
  }
  process.stdout
    .write('qemu-system-x86_64 -name guest=converted\n\n',);
}

//endregion Domain definitions

//region Domain listing

/**
 Text virsh printed on this host with no session daemon and on-demand start disabled, recorded on 2026-10-05.
 */
const CANNOT_CONNECT = [
  'error: failed to connect to the hypervisor',
  'error: Failed to connect socket to \'/run/user/1000/libvirt/virtqemud-sock\': No such file or directory',
].join('\n',);

/**
 Handles `list --all` and `list --all --name` in virsh's output layout.

 @param namesOnly - Whether only names are printed, one per line

 @param scenario - Scenario whose `domains` are listed
 */
function listDomains({
  namesOnly,
  scenario,
}: {
  readonly namesOnly: boolean;
  readonly scenario: Scenario;
},): void {
  /**
   Domains the scenario defines; entries without a name are ignored.
   */
  const domains = Array.isArray(scenario.domains,)
    ? scenario.domains
      .filter(isRecord,)
    : [];
  /**
   Output lines: bare names, or virsh's table with its header and separator.
   */
  const lines = namesOnly
    ? domains.map(function nameOf(domain,) {
      return String(domain.name,);
    },)
    : [
      ' Id   Name       State',
      '--------------------------',
      ...domains.map(function rowOf(domain,) {
        return ` -    ${String(domain.name,)}   ${String(domain.state,)}`;
      },),
    ];
  process.stdout
    .write(`${lines.join('\n',)}\n\n`,);
}

//endregion Domain listing

//region Dispatch

/**
 Longest argument written to the call log in full.
 */
const LOGGED_ARGUMENT_CHARACTERS = 2_000;

/**
 Arguments after the `--connect <uri>` pair mvm always passes.
 */
const [subcommand, ...tokens] = process.argv
  .slice(
  process.argv
    .indexOf('--connect',)
    + 2,
  );

await appendFile(
  join(
    directory,
    'calls.jsonl',
  ),
  `${
    JSON.stringify(
      process.argv
        .slice(2,)
        .map(function shortenForLog(argument,) {
          // A file write carries its data in one argument; the log keeps only enough to recognize the command.
          return argument.length > LOGGED_ARGUMENT_CHARACTERS
            ? `${
              argument.slice(
                0,
                LOGGED_ARGUMENT_CHARACTERS,
              )
            }...`
            : argument;
        },),
    )
  }\n`,
);

/**
 Scenario for this invocation.
 */
const scenario = await readScenario();

if (scenario.connectFailure === true) {
  fail(CANNOT_CONNECT,);
}
else if (subcommand === 'list') {
  listDomains({
    namesOnly: tokens.includes('--name',),
    scenario,
  },);
}
else if (subcommand === 'define') {
  await defineDomain(String(tokens[0],),);
}
else if (subcommand === 'start') {
  await startDomain({
    domain: String(tokens[0],),
    scenario,
  },);
}
else if (subcommand === 'domxml-to-native') {
  await convertDomain({
    scenario,
    xmlPath: String(tokens.at(-1,),),
  },);
}
else if ((subcommand === 'destroy') || (subcommand === 'undefine')) {
  process.stdout
    .write(`Domain '${String(tokens[0],)}' ${subcommand}d\n\n`,);
}
else if (subcommand === 'qemu-agent-command') {
  await agentCommand({
    scenario,
    tokens,
  },);
}
else if (subcommand === 'domstate') {
  /**
   Scripted domain state: a state name, or an object whose `failure` text makes the query fail.
   */
  const {domstate} = scenario;
  if (isRecord(domstate,)) {
    fail(String(domstate.failure,),);
  }
  else {
    process.stdout
      .write(`${((typeof domstate) === 'string') ? domstate : 'running'}\n\n`,);
  }
}
else {
  fail(`error: unknown command: '${String(subcommand,)}'`,);
}

//endregion Dispatch
