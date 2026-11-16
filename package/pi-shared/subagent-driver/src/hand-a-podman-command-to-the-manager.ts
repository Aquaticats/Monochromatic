#!/usr/bin/env node
/**
 `hand-a-podman-command-to-the-manager`: the program a run's unit has first on its `PATH` under the name `podman`, which hands one
 `podman` command to a transient service of the account's manager.

 Why: a unit with any path property runs in a user namespace of its own that maps one uid, and rootless containers need a namespace
 range that one cannot give, so `podman` started inside such a unit ends with `write to uid_map failed: Operation not permitted`. A
 service the manager starts for the run is outside that namespace and outside the unit's mounts, so `podman` there works as it does
 for any session. Two reviews measured both halves: the run's unit in a namespace of its own, and a service it asked for in the
 account's.

 What stays as it was: the checkout is still read-only for the run's own processes, and this program loosens no path property.

 What it deliberately does not do: build a shell line out of the arguments. Every argument is handed to the manager as one argument,
 so a path with a space, a quote or a `$` reaches `podman` as the run wrote it.

 What ends the container: not this service. A container's `conmon` is a child of the manager's pause process and the container's own
 process runs outside the unit's cgroup, so binding the service to the run's unit kills `conmon` and leaves a container nothing can
 stop. Every container this program starts carries a label naming the run's unit, and the launcher removes what a run left once that
 unit has ended (`containers-of-a-run-that-ended.ts`).

 @module
 */

import {
  spawn,
} from 'node:child_process';
import { once } from 'node:events';
import {
  access,
  constants,
  readFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
  resolve,
} from 'node:path';
import { randomUUID, } from 'node:crypto';
import { errorCodeOf, } from './error-code-of.ts';
import {
  containerNameOf,
  runNameOf,
  unitOfThisProcess,
} from './a-container-name-of-a-run.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 How many characters of a fresh identifier the service's name carries, which is enough to tell two of them apart.
 */
const NAME_IDENTIFIER_CHARACTERS = 8;



/**
 Minutes in an hour, which sizes the service's bound.
 */
const MINUTES_PER_HOUR = 60;

/**
 Hours in a day, which sizes the service's bound.
 */
const HOURS_PER_DAY = 24;

/**
 How long a service this program asks for may live, in seconds.

 A bound rather than a measurement: the service ends when the command it runs ends, and a run may not last longer than a day; a
 service that outlived its run would be one nothing can find, so it is ended here.
 */
export const CONTAINER_SERVICE_SECONDS: number = HOURS_PER_DAY * MINUTES_PER_HOUR
  * MINUTES_PER_HOUR;

/**
 The label every container this program starts carries, whose value is the unit it was started for.
 */
export const CONTAINER_LABEL_OF_A_RUN: string = 'pi-container-of';

/**
 The names of the run's own variables a service is given.

 Why only these: a service inherits the manager's environment, and handing it more of the run's would widen what a container reads
 beyond what a container started for this session reads. These four say where the run's home, its search path, its temporary folder
 and its runtime folder are, which is what `podman` needs to find the same storage and the same sockets.
 */
export const VARIABLES_A_SERVICE_GETS: readonly string[] = [
  'HOME',
  'PATH',
  'TMPDIR',
  'XDG_RUNTIME_DIR',
];

/**
 What the account's manager is asked for one `podman` command.
 */
export type ContainerServiceRequest = {
  /**
   The unit this program runs in, which the container's label names, and nothing when this program is run outside a unit.
   */
  readonly ownUnit: string;

  /**
   The arguments the run gave, one for one.
   */
  readonly words: readonly string[];

  /**
   The folder the command runs in, which is where the run asked for it.
   */
  readonly workingFolder: string;

  /**
   Variables of the run's own environment that the service is given, by name.
   */
  readonly environment: Readonly<NodeJS.ProcessEnv>;

  /**
   The unit name the service gets, which the caller makes unique.
   */
  readonly unitName: string;

  /**
   The account's own `podman`, by absolute path: the service must not find this program again, which would ask for a service from
   inside a service for ever.
   */
  readonly podman: string;
};

/**
 What one hand-over is: the words of the service, and what the program prints before it.
 */
export type ContainerHandOver = {
  /**
   The program to run.
   */
  readonly command: string;

  /**
   Its arguments, each one the run wrote.
   */
  readonly words: readonly string[];

  /**
   Whether the program waits for the service, which a detached container does not.
   */
  readonly waits: boolean;

  /**
   The container's name, which a detached `podman run` prints instead of an id, and nothing when it is not detached.
   */
  readonly containerName: string;
};

/**
 The `--setenv=` words that give a service the variables the run's own environment holds.

 @param environment - the variables, by name

 @returns one word per variable that is set, in the order they are named

 @example
 ```ts
 const words = setenvWordsOf({ environment: { HOME: '/home/a', }, });
 ```
 */
function setenvWordsOf({ environment }: { readonly environment: Readonly<NodeJS.ProcessEnv> },): readonly string[] {
  /**
   One word per variable that is set.
   */
  const words: string[] = [];
  for (const name of VARIABLES_A_SERVICE_GETS) {
    /**
     What that variable holds, when it is set.
     */
    const value = environment[name];
    if (value !== undefined) {
      words.push(`--setenv=${name}=${value}`,);
    }
  }
  return words;
}

/**
 Builds what one `podman` command becomes: a service of the account's manager, which runs the account's own `podman`.

 @param request - the run's unit, its arguments, the folder, the variables and the service's name

 @returns the program and its words, whether it waits, and the container's name when the run asked for a detached one

 @throws Error when the run's unit cannot be read, because a container that is labelled with no unit belongs to no run

 @example
 ```ts
 const handOver = handOverOf({ ownUnit: 'pi-a.service', words: ['run', '--rm', 'alpine', '/bin/true'], workingFolder: '/repo/.claude/worktrees/pi-a', environment: { HOME: '/home/a', }, unitName: 'a-container-1', podman: '/usr/bin/podman', });
 ```
 */
export function handOverOf(request: ContainerServiceRequest,): ContainerHandOver {
  if (request.ownUnit === '') {
    throw new Error(
      'this program hands a container to the account\'s manager, and it cannot say which unit of a run to label that container with: it was run outside any unit of a run. Start the run in its unit, or run podman itself.',
    );
  }
  /**
   Whether the run asked for a container that outlives the command.
   */
  const detached = (request.words
    .some(function isDetach(word,) {
      return (word === '--detach') || (word === '-d');
    },))
    && ((request.words[0] === 'run') || (request.words[0] === 'create'));
  /**
   The arguments as the run wrote them, without the flag this program replaces.
   */
  const words = request.words
    .filter(function keepsEverythingButDetach(word,) {
      return (word !== '--detach') && (word !== '-d');
    },);
  /**
   Whether the run named the container itself, in which case its name is its own business.
   */
  const named = request.words
    .some(function namesTheContainer(word,) {
      return (word === '--name') || (word.startsWith('--name=',));
    },);
  /**
   The container's name, which a detached run is told; empty when this command starts no detached container.
   */
  /**
   The container's name, which a detached run is told.
   */
  const containerName = ((detached && (!named))
    ? containerNameOf({
      ownUnit: request.ownUnit,
      words: request.words,
    },)
    : '');
  /**
   Whether these words start a container at all, which is what the label belongs on.
   */
  const startsAContainer = [
    'run',
    'create',
  ].includes(words[0] ?? '',);
  return {
    command: 'systemd-run',
    words: [
      '--user',
      '--quiet',
      '--collect',
      `--unit=${request.unitName}`,
      `--property=RuntimeMaxSec=${String(CONTAINER_SERVICE_SECONDS,)}`,
      '--property=Type=exec',
      ...(detached ? [] : [
        '--wait',
        '--pipe',
      ]),
      `--working-directory=${request.workingFolder}`,
      ...setenvWordsOf({ environment: request.environment, },),
      request.podman,
      ...words.slice(
        0,
        1,
      ),
      ...(startsAContainer ? [`--label=${CONTAINER_LABEL_OF_A_RUN}=${request.ownUnit}`,] : []),
      ...words.slice(1,),
    ],
    waits: !detached,
    containerName,
  };
}

/**
 Finds the account's own `podman` on a search path, skipping the folder this program's own link is in.

 Why: the run's `podman` is this program, so a service that searched the run's path would run this program again, which would ask for
 another service from inside a service, for ever.

 @param request - the search path to look through, and the folder this program's own link is in

 @returns the absolute path of the account's `podman`, and the search path without this program's folder

 @throws Error when no other `podman` is on the path, naming the path it looked through

 @example
 ```ts
 const account = await accountPodmanOf({ path: '/runs/a-1/bin:/usr/bin:/bin', ownFolder: '/runs/a-1/bin', });
 ```
 */
export async function accountPodmanOf(request: {
  /**
   The search path to look through.
   */
  readonly path: string;

  /**
   The folder this program's own link is in, which is skipped.
   */
  readonly ownFolder: string;
},): Promise<{
  readonly podman: string;
  readonly path: string
}> {
  /**
   The search path and the folder to skip, each on its own line.
   */
  const {
    path,
    ownFolder,
  } = request;
  /**
   The folders of the search path, without the one holding this program.
   */
  const folders = path.split(':',)
    .filter(function isNotOurs(folder,) {
      return (folder !== '') && (resolve(folder,) !== resolve(ownFolder,));
    },);
  /**
   Each candidate, and whether it is an executable.
   */
  const checked = await Promise.all(folders.map(async function isExecutable(folder,) {
    /**
     A candidate this folder holds.
     */
    const candidate = join(
      folder,
      'podman',
    );
    try {
      await access(
        candidate,
        constants.X_OK,
      );
      return candidate;
    }
    catch (error) {
      // A folder without a podman is what the search is looking past; any other failure would hide what it meant to check.
      if (errorCodeOf(error,) !== MISSING_PATH_CODE) {
        throw error;
      }
      return '';
    }
  },),);
  /**
   The first candidate that is there.
   */
  const found = checked.find(function isThere(candidate,) {
    return candidate !== '';
  },);
  if (found !== undefined) {
    return {
      podman: found,
      path: folders.join(':',),
    };
  }
  throw new Error(
    `no podman of the account's own is on the search path ${folders.join(':',)}, so this program cannot hand the run's command to it: install podman, or put its folder on the path this run's unit gets.`,
  );
}

/**
 Hands the run's `podman` command to a service of the account's manager, and answers with what that command answered.

 @param request - the arguments the run gave, this program's own environment, its working folder and its cgroup file

 @returns the status the command ended with, and what a detached container is called

 @throws Error when the run's unit cannot be read, when the account's podman is not on the path, and when the manager cannot be asked

 @example
 ```ts
 const answered = await handTheCommandToTheManager({ words: ['run', '--rm', 'alpine', '/bin/true'], environment: process.env, workingFolder: process.cwd(), cgroup: '0::/pi-a.service', });
 ```
 */
export async function handTheCommandToTheManager(request: {
  /**
   The arguments the run gave, one for one.
   */
  readonly words: readonly string[];

  /**
   This program's own environment, from which four variables are passed on.
   */
  readonly environment: Readonly<NodeJS.ProcessEnv>;

  /**
   The folder the run asked for the command to run in.
   */
  readonly workingFolder: string;

  /**
   The text of this program's own cgroup file.
   */
  readonly cgroup: string;
},): Promise<{
  readonly status: number;
  readonly containerName: string
}> {
  /**
   The unit this program runs in, which the container's label names.
   */
  const ownUnit = unitOfThisProcess(request.cgroup,);
  /**
   The run's own name, which the service's name carries.
   */
  const runName = runNameOf({ ownUnit, },);
  /**
   The service's name, which says it carries a container of that run and is unique to this command.
   */
  const unitName = `container-of-${runName}-${randomUUID()
    .slice(
      0,
      NAME_IDENTIFIER_CHARACTERS,
    )}`;
  /**
   The account's own `podman`, and the search path that finds it rather than this program.
   */
  const account = await accountPodmanOf({
    path: request.environment
      .PATH
      ?? '',
    ownFolder: dirname(process.argv[1] ?? '',),
  },);
  /**
   What this command becomes.
   */
  const handOver = handOverOf({
    ownUnit,
    words: request.words,
    workingFolder: request.workingFolder,
    environment: {
      ...request.environment,
      PATH: account.path,
    },
    unitName,
    podman: account.podman,
  },);
  /**
   The program, with this process's own standard input, output and error, so no byte of them is copied or re-encoded.
   */
  const answered = spawn(
    handOver.command,
    [...handOver.words,],
    { stdio: 'inherit', },
  );
  // The manager is asked once; a question that never answers is not retried here, because the run's unit bounds this program.
  await once(
    answered,
    'exit',
  );
  if (handOver.containerName !== '') {
    // A detached container is run in the foreground inside its service, so the run learns its name rather than an id that would
    // name a container this program never created.
    if (answered.exitCode === 0) {
      process.stdout
        .write(`${handOver.containerName}\n`,);
    }
    return {
      status: (answered.exitCode ?? (-1)),
      containerName: handOver.containerName,
    };
  }
  return {
    status: (answered.exitCode ?? (-1)),
    containerName: '',
  };
}

// Running as a program: hand this command over, and leave with what it answered. When this file is imported, as the tests do, this
// block does nothing because the condition below is false.
if ((process.argv[1] ?? '').endsWith('/podman',)) {
  /**
   What the command answered, whose status is what a run reads.
   */
  const answered = await handTheCommandToTheManager({
    words: process.argv
      .slice(2,),
    environment: process.env,
    workingFolder: process.cwd(),
    cgroup: await readFile(
      '/proc/self/cgroup',
      'utf8',
    ),
  },);
  process.exitCode = answered.status;
}
