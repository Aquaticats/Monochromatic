/**
 Guest agent file commands of the stand-in `virsh` (`fixture.fake-virsh.ts`).

 The simulated guest's files live under `guest/` in the fake's state
 directory: `/tmp/a b.txt` is `guest/tmp/a b.txt`, and `C:\Temp\a.txt` is
 `guest/C/Temp/a.txt`. Each open handle is one file under `handles/`, so
 invocations that touch different handles never share state.

 Behavior follows the real agent where mvm depends on it: `wb` creates or
 empties, a file opened for writing cannot be opened for writing again (the
 Windows agent opens with read sharing only), a read at the end of the file
 returns no data, and a handle keeps its position between commands.

 Scenario keys read here, all optional:

 - `readEof`: `'windows'` reports the end of the file only on an empty read; anything else reports it on a short read too, as the Linux agent does.
 - `writeUnansweredAtOffset`: the first write at this offset is applied, then fails like a timed-out agent command.
 - `readUnansweredAtOffset`: the first read at this offset advances the position, then fails like a timed-out agent command.
 - `writeErrorAtOffset`: every write at this offset is refused by the agent.
 - `shortWriteAtOffset`: a write at this offset stores only half of its data and says so.
 - `maxReadCount`: reads asking for more bytes fail the way an oversized answer does.
 - `silentCommandsAfterUnanswered`: after the scripted unanswered write or read, this many following file commands fail the way a guest agent that stopped answering does, and are not applied.

 @module
 */

import { randomInt, } from 'node:crypto';
import {
  access,
  mkdir,
  open,
  readdir,
  readFile,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

//region Shapes

/**
 What handling one file command produced: the agent's `return` value, or the
 standard-error text of a failed virsh call.
 */
export type FileCommandOutcome =
  | { readonly failure: string; }
    | { readonly reply: unknown; };

/**
 One open handle of the simulated agent.
 */
type Handle = {
  readonly hostPath: string;
  readonly position: number;
  readonly writable: boolean;
};

/**
 Text libvirt printed when the agent did not answer within the timeout, recorded on this host.
 */
const UNANSWERED =
  'error: guest agent command timed out: guest agent didn\'t respond to command within \'60\' seconds';

/**
 Text libvirt printed for every command while the Windows agent had stopped answering, recorded on this host.
 */
const SILENT =
  'error: Guest agent is not responding: guest agent didn\'t respond to synchronization within \'5\' seconds';

/**
 Smallest handle number handed out; the real agent's numbers are large too.
 */
const FIRST_HANDLE = 1_000;

/**
 End of the range handle numbers are drawn from.
 */
const HANDLE_RANGE_END = 2_000_000_000;

//endregion Shapes

//region Helpers

/**
 Builds the failure of an agent command the agent itself refused.

 @param commandName - Agent command name

 @param reason - Agent's error text

 @returns Outcome carrying libvirt's wording of an agent error
 */
function refused({
  commandName,
  reason,
}: {
  readonly commandName: string;
  readonly reason: string;
},): FileCommandOutcome {
  return {
    failure: `error: guest agent command failed: unable to execute QEMU agent command '${commandName}': ${reason}`,
  };
}

/**
 Maps a guest path to the file that stands for it on the host.

 @param guestPath - Path as the guest names it, Linux or Windows form

 @param root - Directory standing for the guest's file system

 @returns Host path of the stand-in file
 */
function hostPathOf({
  guestPath,
  root,
}: {
  readonly guestPath: string;
  readonly root: string;
},): string {
  /**
   Path segments; a Windows drive letter loses its colon and becomes the first segment.
   */
  const segments = guestPath
    .replaceAll(
    '\\',
    '/',
    )
    .split('/',)
    .filter(function isNamed(segment,) {
      return segment !== '';
    },)
    .map(function withoutColon(segment,) {
      return segment.endsWith(':',)
        ? segment.slice(
          0,
          -1,
        )
        : segment;
    },);
  return join(
    root,
    ...segments,
  );
}

/**
 Checks whether a path exists.

 @param path - Path to look for

 @returns Whether the path can be reached
 */
async function exists(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    return false;
  }
}

/**
 Reads a number from a request.

 @param key - Argument name

 @param request - Arguments of the agent command

 @returns The number, or `NaN` when the argument is missing or not a number
 */
function numberArgument({
  key,
  request,
}: {
  readonly key: string;
  readonly request: Readonly<Record<string, unknown>>;
},): number {
  /**
   Raw argument value.
   */
  const value = request[key];
  return ((typeof value) === 'number') ? value : Number.NaN;
}

//endregion Helpers

//region Handles

/**
 Ending of a handle's state file name; the rest of the name is the handle number.
 */
const HANDLE_FILE_SUFFIX = '.json';

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
 Path of the file holding one handle's state.

 @param directory - Fake's state directory

 @param handle - Handle number

 @returns Path of the handle's state file
 */
function handlePath({
  directory,
  handle,
}: {
  readonly directory: string;
  readonly handle: number;
},): string {
  return join(
    directory,
    'handles',
    `${String(handle,)}${HANDLE_FILE_SUFFIX}`,
  );
}

/**
 Reads all open handles.

 @param directory - Fake's state directory

 @returns Handle numbers with their state
 */
async function openHandles(
  directory: string,
): Promise<readonly (readonly [
  number,
  Handle
])[]> {
  /**
   Directory holding one file per open handle.
   */
  const handlesDirectory = join(
    directory,
    'handles',
  );
  await mkdir(
    handlesDirectory,
    { recursive: true, },
  );
  /**
   Names of the handle files.
   */
  const names = await readdir(handlesDirectory,);
  return await Promise.all(
    names.map(async function readOne(name,): Promise<readonly [
      number,
      Handle
    ]> {
      /**
       Stored state of one handle, typed as unknown until its properties are checked.
       */
      const stored: unknown = JSON.parse(
        await readFile(
          join(
            handlesDirectory,
            name,
          ),
          'utf8',
        ),
      );
      /**
       Stored properties; a file that does not hold an object reads as a handle with no usable state.
       */
      const properties = isRecord(stored,) ? stored : {};
      return [
        Number(name.slice(
          0,
          -HANDLE_FILE_SUFFIX.length,
        ),),
        {
          hostPath: ((typeof properties.hostPath) === 'string') ? properties.hostPath : '',
          position: ((typeof properties.position) === 'number') ? properties.position : 0,
          writable: properties.writable === true,
        },
      ];
    },),
  );
}

/**
 Stores one handle's state.

 @param directory - Fake's state directory

 @param handle - Handle number

 @param state - State to store
 */
async function storeHandle({
  directory,
  handle,
  state,
}: {
  readonly directory: string;
  readonly handle: number;
  readonly state: Handle;
},): Promise<void> {
  await writeFile(
    handlePath({
      directory,
      handle,
    },),
    JSON.stringify(state,),
  );
}

/**
 Records that a once-only scripted failure has happened.

 @param directory - Fake's state directory

 @param name - Name of the failure

 @returns Whether this call is the first to record it
 */
async function firstTime({
  directory,
  name,
}: {
  readonly directory: string;
  readonly name: string;
},): Promise<boolean> {
  /**
   Marker file whose presence means the failure already happened.
   */
  const marker = join(
    directory,
    `${name}.used`,
  );
  if (await exists(marker,)) {
    return false;
  }
  await writeFile(
    marker,
    '',
  );
  return true;
}

/**
 File holding how many following file commands the simulated agent leaves unanswered.

 @param directory - Fake's state directory

 @returns Path of the counter file
 */
function silencePath(directory: string,): string {
  return join(
    directory,
    'silent-commands.remaining',
  );
}

/**
 Starts the scripted silence after an unanswered command, when the scenario asks for one.

 @param directory - Fake's state directory

 @param scenario - Scenario whose `silentCommandsAfterUnanswered` gives the number of silent commands
 */
async function startSilence({
  directory,
  scenario,
}: {
  readonly directory: string;
  readonly scenario: Readonly<Record<string, unknown>>;
},): Promise<void> {
  /**
   Number of following commands to leave unanswered.
   */
  const { silentCommandsAfterUnanswered, } = scenario;
  if (((typeof silentCommandsAfterUnanswered) === 'number') && (silentCommandsAfterUnanswered > 0)) {
    await writeFile(
      silencePath(directory,),
      String(silentCommandsAfterUnanswered,),
    );
  }
}

/**
 Consumes one command of the scripted silence.

 @param directory - Fake's state directory

 @returns Whether this command falls into the silence and must go unanswered
 */
async function consumeSilence(directory: string,): Promise<boolean> {
  /**
   Counter file of the silence.
   */
  const path = silencePath(directory,);
  if (!(await exists(path,))) {
    return false;
  }
  /**
   Silent commands left before this one.
   */
  const remaining = Number(await readFile(
    path,
    'utf8',
  ),);
  if (remaining <= 0) {
    return false;
  }
  await writeFile(
    path,
    String(remaining - 1,),
  );
  return true;
}

//endregion Handles

//region Commands

/**
 Handles `guest-file-open`.

 @param directory - Fake's state directory

 @param request - Arguments of the agent command

 @returns New handle number, or the agent's refusal
 */
async function fileOpen({
  directory,
  request,
}: {
  readonly directory: string;
  readonly request: Readonly<Record<string, unknown>>;
},): Promise<FileCommandOutcome> {
  /**
   Path as the guest names it.
   */
  const guestPath = String(request.path,);
  /**
   Open mode; the agent's default is reading.
   */
  const mode = ((typeof request.mode) === 'string') ? request.mode : 'r';
  /**
   Stand-in file on the host.
   */
  const hostPath = hostPathOf({
    guestPath,
    root: join(
      directory,
      'guest',
    ),
  },);
  /**
   Whether the mode writes.
   */
  const writable = mode.startsWith('w',);
  if ((!writable) && (!mode.startsWith('r',))) {
    return refused({
      commandName: 'guest-file-open',
      reason: 'invalid file open mode',
    },);
  }
  /**
   Handles open right now, to refuse a second writer as the Windows agent does.
   */
  const handles = await openHandles(directory,);
  if (writable
    && handles.some(function writesSameFile([, state,],) {
      return state.writable && (state.hostPath === hostPath);
    },))
    {
    return refused({
      commandName: 'guest-file-open',
      reason:
        `failed to open file '${guestPath}': The process cannot access the file because it is being used by another process.`,
    },);
  }
  if (writable ? (!(await exists(dirname(hostPath,),))) : (!(await exists(hostPath,)))) {
    return refused({
      commandName: 'guest-file-open',
      reason: `failed to open file '${guestPath}' (mode: '${mode}'): No such file or directory`,
    },);
  }
  if (writable) {
    await writeFile(
      hostPath,
      '',
    );
  }
  /**
   Number for the new handle.
   */
  const handle = randomInt(
    FIRST_HANDLE,
    HANDLE_RANGE_END,
  );
  await storeHandle({
    directory,
    handle,
    state: {
      hostPath,
      position: 0,
      writable,
    },
  },);
  return { reply: handle, };
}

/**
 Handles `guest-file-write`.

 @param directory - Fake's state directory

 @param handle - Handle number

 @param request - Arguments of the agent command

 @param scenario - Scenario with the scripted failures

 @param state - Handle state before the write

 @returns Bytes written, or a failure
 */
async function fileWrite({
  directory,
  handle,
  request,
  scenario,
  state,
}: {
  readonly directory: string;
  readonly handle: number;
  readonly request: Readonly<Record<string, unknown>>;
  readonly scenario: Readonly<Record<string, unknown>>;
  readonly state: Handle;
},): Promise<FileCommandOutcome> {
  if (scenario.writeErrorAtOffset === state.position) {
    return refused({
      commandName: 'guest-file-write',
      reason: 'failed to write to file: No space left on device',
    },);
  }
  /**
   Bytes mvm sent.
   */
  const given = Buffer.from(
    String(request['buf-b64'],),
    'base64',
  );
  /**
   Bytes actually stored: half of them when the scenario scripts a short write here.
   */
  const stored = (scenario.shortWriteAtOffset === state.position)
    ? given.subarray(
      0,
      Math.floor(given.length / 2,),
    )
    : given;
  /**
   Stand-in file, opened for update so writing at a position keeps the rest.
   */
  await using file = await open(
    state.hostPath,
    'r+',
  );
  await file.write(
    stored,
    0,
    stored.length,
    state.position,
  );
  await storeHandle({
    directory,
    handle,
    state: {
      ...state,
      position: state.position + stored.length,
    },
  },);
  if ((scenario.writeUnansweredAtOffset === state.position)
    && (await firstTime({
      directory,
      name: 'write-unanswered',
    },)))
    {
    await startSilence({
      directory,
      scenario,
    },);
    return { failure: UNANSWERED, };
  }
  return {
    reply: {
      count: stored.length,
      eof: false,
    },
  };
}

/**
 Handles `guest-file-read`.

 @param directory - Fake's state directory

 @param handle - Handle number

 @param request - Arguments of the agent command

 @param scenario - Scenario with the scripted failures

 @param state - Handle state before the read

 @returns Bytes read with the end-of-file flag, or a failure
 */
async function fileRead({
  directory,
  handle,
  request,
  scenario,
  state,
}: {
  readonly directory: string;
  readonly handle: number;
  readonly request: Readonly<Record<string, unknown>>;
  readonly scenario: Readonly<Record<string, unknown>>;
  readonly state: Handle;
},): Promise<FileCommandOutcome> {
  /**
   Bytes asked for; the agent's default is 4 KiB.
   */
  const count = ((typeof request.count) === 'number') ? request.count : 4_096;
  /**
   Largest read the scenario lets through, when it sets one.
   */
  const {maxReadCount} = scenario;
  if (((typeof maxReadCount) === 'number') && (count > maxReadCount)) {
    return { failure: 'error: internal error: fake virsh: the answer is larger than libvirt passes on', };
  }
  /**
   Buffer the read fills.
   */
  const buffer = Buffer.alloc(count,);
  /**
   Stand-in file.
   */
  await using file = await open(
    state.hostPath,
    'r',
  );
  /**
   Result of the read at the handle's position.
   */
  const { bytesRead, } = await file.read(
    buffer,
    0,
    count,
    state.position,
  );
  await storeHandle({
    directory,
    handle,
    state: {
      ...state,
      position: state.position + bytesRead,
    },
  },);
  if ((scenario.readUnansweredAtOffset === state.position)
    && (await firstTime({
      directory,
      name: 'read-unanswered',
    },)))
    {
    await startSilence({
      directory,
      scenario,
    },);
    return { failure: UNANSWERED, };
  }
  return {
    reply: {
      count: bytesRead,
      eof: scenario.readEof === 'windows' ? (bytesRead === 0) : (bytesRead < count),
      ...(bytesRead === 0
        ? {}
        : {
          'buf-b64': buffer
            .subarray(
            0,
            bytesRead,
            )
            .toString('base64',),
        }),
    },
  };
}

/**
 Handles `guest-file-seek`.

 @param directory - Fake's state directory

 @param handle - Handle number

 @param request - Arguments of the agent command

 @param state - Handle state before the seek

 @returns New position, or the agent's refusal
 */
async function fileSeek({
  directory,
  handle,
  request,
  state,
}: {
  readonly directory: string;
  readonly handle: number;
  readonly request: Readonly<Record<string, unknown>>;
  readonly state: Handle;
},): Promise<FileCommandOutcome> {
  /**
   Offset relative to the reference point.
   */
  const offset = numberArgument({
    key: 'offset',
    request,
  },);
  /**
   Reference point: a name, or the number the older protocol used.
   */
  const {whence} = request;
  /**
   Size of the stand-in file, for seeks relative to its end.
   */
  const { size, } = await stat(state.hostPath,);
  /**
   Position the reference point stands for.
   */
  const base = (function resolveBase(): number {
    if ((whence === 'set') || (whence === 0))
      return 0;
    if ((whence === 'cur') || (whence === 1))
      return state.position;
    if ((whence === 'end') || (whence === 2))
      return size;
    return Number.NaN;
  })();
  /**
   New position.
   */
  const position = base + offset;
  if (Number.isNaN(position,) || (position < 0)) {
    return refused({
      commandName: 'guest-file-seek',
      reason: 'failed to seek file: Invalid argument',
    },);
  }
  await storeHandle({
    directory,
    handle,
    state: {
      ...state,
      position,
    },
  },);
  return {
    reply: {
      eof: false,
      position,
    },
  };
}

/**
 Names of the agent commands handled here.
 */
const FILE_COMMANDS: ReadonlySet<string> = new Set([
  'guest-file-close',
  'guest-file-open',
  'guest-file-read',
  'guest-file-seek',
  'guest-file-write',
],);

/**
 Checks whether an agent command is one of the file commands.

 @param execute - Agent command name

 @returns Whether {@link guestFileCommand} handles it

 @example
 ```ts
 isFileCommand('guest-file-read'); // true
 ```
 */
export function isFileCommand(execute: string,): boolean {
  return FILE_COMMANDS.has(execute,);
}

/**
 Handles one guest agent file command.

 @param directory - Fake's state directory

 @param execute - Agent command name; one for which {@link isFileCommand} is true

 @param request - Arguments of the agent command

 @param scenario - Scenario with the scripted failures

 @returns The agent's `return` value, or the standard-error text of a failed virsh call

 @example
 ```ts
 const outcome = await guestFileCommand({ directory, execute: 'guest-file-open', request: { path: '/tmp/a' }, scenario: {} });
 ```
 */
export async function guestFileCommand({
  directory,
  execute,
  request,
  scenario,
}: {
  readonly directory: string;
  readonly execute: string;
  readonly request: Readonly<Record<string, unknown>>;
  readonly scenario: Readonly<Record<string, unknown>>;
},): Promise<FileCommandOutcome> {
  if (await consumeSilence(directory,)) {
    return { failure: SILENT, };
  }
  if (execute === 'guest-file-open') {
    return await fileOpen({
      directory,
      request,
    },);
  }
  /**
   Handle the command names.
   */
  const handle = numberArgument({
    key: 'handle',
    request,
  },);
  /**
   State of that handle, when it is open.
   */
  const entry = (await openHandles(directory,)).find(function hasNumber([number,],) {
    return number === handle;
  },);
  if (entry === undefined) {
    return refused({
      commandName: execute,
      reason: `handle '${String(handle,)}' has not been found`,
    },);
  }
  /**
   State of the handle before this command.
   */
  const [, state,] = entry;
  if (execute === 'guest-file-close') {
    await rm(handlePath({
      directory,
      handle,
    },),);
    return { reply: {}, };
  }
  if (execute === 'guest-file-write') {
    return await fileWrite({
      directory,
      handle,
      request,
      scenario,
      state,
    },);
  }
  if (execute === 'guest-file-read') {
    return await fileRead({
      directory,
      handle,
      request,
      scenario,
      state,
    },);
  }
  return await fileSeek({
    directory,
    handle,
    request,
    state,
  },);
}

//endregion Commands
