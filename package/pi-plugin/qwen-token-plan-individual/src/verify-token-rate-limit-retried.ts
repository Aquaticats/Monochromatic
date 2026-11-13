/**
 Proof, with a real pi, that this provider's passing token-rate limit is asked
 again by pi's own automatic retry,
 and that the unchanged package ends the run on that answer.

 The proof runs pi twice, each in its own throwaway home,
 against a stand-in endpoint on the loopback interface:
 the first chat completion it is asked for is answered with the recorded `429`
 body, word for word, and every later one with a reply.
 One run loads the bundle this checkout builds;
 the other loads a bundle built from the package's last commit before this
 translation existed,
 which handed pi's OpenAI-completions stream on unchanged.
 Each run's feed and error output are kept beside a summary,
 so a reader checks the claim against what pi printed instead of trusting it.

 No request reaches the real provider:
 the throwaway `models.json` points the route at the loopback stand-in,
 which also answers the catalog refresh the extension asks for at session
 start,
 and both runs use pi's `--offline`.

 Run it with
 `mise run //package/pi-plugin/qwen-token-plan-individual:verify:token-rate-limit-retried`.

 @module
 */

import {
  execFile,
  spawn,
} from 'node:child_process';
import { existsSync, } from 'node:fs';
import {
  mkdir,
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import {
  createServer,
  type Server,
} from 'node:http';
import type { AddressInfo, } from 'node:net';
import {
  dirname,
  join,
  resolve,
} from 'node:path';
import { fileURLToPath, } from 'node:url';
import {
  promisify,
} from 'node:util';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { piCliPath, } from './disposable-host.ts';

//region Constants

/**
 Folder name prefix every proof folder starts with,
 so a reader of the scratch root knows what filled it.
 */
const PROOF_FOLDER_PREFIX = 'qwen-token-plan-individual-token-rate-limit-retried-';

/**
 Provider route both runs answer for.
 */
const PROVIDER = 'qwen-token-plan-individual';

/**
 Model both runs ask for.
 */
const MODEL = 'qwen4-max-preview';

/**
 Prompt both runs are given.
 */
const PROMPT = 'Answer with the single word pong.';

/**
 Reply the stand-in gives every chat completion after the first.
 */
const REPLY_TEXT = 'pong';

/**
 Credential the throwaway home stores,
 so pi treats the route as configured and no real secret reaches the proof.
 */
const FIXTURE_KEY = 'fixture-key';

/**
 The body fifteen review runs ended on 2026-11-13,
 with its request id replaced by a placeholder.
 */
const RECORDED_PASSING_LIMIT_BODY: Readonly<Record<string, unknown>> = {
  message:
    'Allocated quota exceeded, please increase your quota limit. For details, see: https://www.alibabacloud.com/help/en/model-studio/error-code#token-limit',
  id: '00000000-0000-4000-8000-000000000000',
  type: 'insufficient_quota',
  code: 'insufficient_quota',
};

/**
 Retry budget the throwaway home gives pi.

 It is pi's own retry either way: the proof sets a tenth of the wait this
 machine's settings allow and two attempts rather than eight,
 because what is measured is that pi retries this answer at all.
 */
const PROOF_RETRY_SETTINGS: Readonly<Record<string, unknown>> = {
  enabled: true,
  maxRetries: 2,
  baseDelayMs: 100,
};

/**
 Bound on one pi run, so a run that never settles fails the proof instead of
 holding it.
 */
const RUN_TIMEOUT_MILLISECONDS = 120_000;

/**
 Creation time the stand-in reports for the model it lists,
 copied from the listing the package pins.
 */
const LISTING_CREATED_SECONDS = 1_794_362_561;

/**
 Command runner both git and the bundler are called through.
 */
const RUN_COMMAND = promisify(execFile,);

/**
 Package directory name both the archive and the build name.
 */
const PACKAGE_ARCHIVE_PATH = 'package/pi-plugin/qwen-token-plan-individual';

/**
 Subcommand sequence that names the commit which introduced this translation
 into the provider dispatch.
 */
const TRANSLATION_COMMIT_SEARCH: readonly string[] = [
  'log',
  '--reverse',
  '--format=%H',
  '-S',
  'streamWithPassingTokenRateLimitRetried',
  '--',
  `${PACKAGE_ARCHIVE_PATH}/src/provider.ts`,
];

//endregion Constants

//region Types

/**
 What one pi run left behind.
 */
type PiRun = {
  /**
   Session events the run printed, in order.
   */
  readonly events: readonly Readonly<Record<string, unknown>>[];
  /**
   Everything the run wrote to standard error.
   */
  readonly stderr: string;
  /**
   Exit status the run ended with.
   */
  readonly exitCode: number;
  /**
   Whether pi had to be killed at the run's bound.
   */
  readonly timedOut: boolean;
};

/**
 The stand-in endpoint, its port, and how many chat completions it answered.
 */
type StandIn = {
  /**
   Port the loopback endpoint listens on.
   */
  readonly port: number;
  /**
   Server object the proof closes when both runs are done.
   */
  readonly server: Server;
  /**
   How many chat completions the endpoint has answered so far.
   *
   @returns the count at the moment it is read
   */
  chatRequests: () => number;
  /**
   Start counting a new run's requests, so that run meets the recorded answer
   first even though an earlier run already used the endpoint.
   */
  beginRun: () => void;
};

//endregion Types

//region Small helpers

/**
 Whether a parsed value is a plain object a session event can be read by key.

 @param value - parsed JSON value

 @returns `true` for an object that is not an array

 @example
 ```typescript
 if (isRecord(parsed)) {
   // parsed's fields can be read here
 }
 ```
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  if ((typeof value) !== 'object')
    return false;
  if (value === null)
    return false;
  return !Array.isArray(value,);
}

/**
 Read one field of a record as a plain object.

 @param record - record to read

 @param field - field name to read

 @returns the field's object, or nothing when it holds something else

 @example
 ```typescript
 const message = recordField({ record: event, field: 'message' });
 ```
 */
function recordField({
  record,
  field,
}: {
  readonly record: Readonly<Record<string, unknown>>;
  readonly field: string;
},): Readonly<Record<string, unknown>> | undefined {
  /**
   Value the field holds.
   */
  const value = record[field];
  return isRecord(value,) ? value : undefined;
}

/**
 Read one field of a record as a string.

 @param record - record to read

 @param field - field name to read

 @returns the field's string, or nothing when it holds something else

 @example
 ```typescript
 const type = stringField({ record: event, field: 'type' });
 ```
 */
function stringField({
  record,
  field,
}: {
  readonly record: Readonly<Record<string, unknown>>;
  readonly field: string;
},): string | undefined {
  /**
   Value the field holds.
   */
  const value = record[field];
  return (typeof value) === 'string' ? value : undefined;
}

/**
 Read one assistant message's text, so the reply a run ended with can be
 compared with what the stand-in sent.

 @param message - assistant message a session event carried

 @returns every text block the message holds, joined

 @example
 ```typescript
 const text = assistantText({ message });
 ```
 */
function assistantText({
  message,
}: {
  readonly message: Readonly<Record<string, unknown>>;
},): string {
  /**
   Content blocks the message carries, which it may hold as anything.
   */
  const content = message['content'];
  if (!Array.isArray(content,))
    return '';
  return content.flatMap(function textOf(block: unknown,) {
    if (!isRecord(block,))
      return [];
    /**
     Text the block carries, when it is a text block.
     */
    const text = block['text'];
    return (typeof text) === 'string' ? [text,] : [];
  },)
    .join('',);
}

/**
 The checkout this proof runs inside,
 found by walking up to the directory that holds the repository's `.git`
 entry.

 The repository's own `git` refuses to run from anywhere but the root,
 and this script starts in its package directory.

 @param start - directory to walk up from

 @returns absolute path of the repository root

 @throws naming the directory the walk ended at, which holds no `.git` entry

 @example
 ```typescript
 const repository = repositoryRootOf({ start: process.cwd() });
 ```
 */
function repositoryRootOf({
  start,
}: {
  readonly start: string;
},): string {
  if (existsSync(join(
    start,
    '.git',
  ),))
    return start;
  /**
   Directory above the one just read, or the same directory at the filesystem
   root.
   */
  const above = dirname(start,);
  if (above === start)
    throw new Error(
      `no .git entry was found above ${start}, so this proof cannot find the checkout it runs inside`,
    );
  return repositoryRootOf({ start: above, },);
}

/**
 An output stream a spawned child is guaranteed to have,
 refused loudly when the child reports none.

 @param stream - stream the child was given

 @returns the same stream, typed as present

 @example
 ```typescript
 const stdout = presentStream({ stream: child.stdout });
 ```
 */
function presentStream<T>({
  stream,
}: {
  readonly stream: T | null;
},): T {
  if (stream === null)
    throw new Error('the child process was started without the output stream this proof reads',);
  return stream;
}

//endregion Small helpers

//region The proof folder

/**
 Create the folder this proof keeps everything in.

 @returns the folder's absolute path

 @example
 ```typescript
 const root = await proofFolder();
 ```
 */
async function proofFolder(): Promise<string> {
  /**
   Fresh folder under the module-test scratch root,
   which no checkout and no project configuration file sits above.
   */
  const folder = await mkdtemp(join(
    scratchRoot(),
    PROOF_FOLDER_PREFIX,
  ),);
  await mkdir(
    join(
      folder,
      'runs',
    ),
    { recursive: true, },
  );
  return folder;
}

//endregion The proof folder

//region The throwaway home

/**
 Write one throwaway home whose agent directory points the route at the
 stand-in and gives pi a small retry budget.

 @param root - folder the home lives in

 @param port - loopback port the stand-in listens on

 @returns the home and the agent directory pi reads

 @example
 ```typescript
 const home = await throwawayHome({ root, port: standIn.port });
 ```
 */
async function throwawayHome({
  root,
  port,
}: {
  readonly root: string;
  readonly port: number;
},): Promise<{
  readonly home: string;
  readonly agentDir: string;
}> {
  /**
   Home the pi run sees,
   so nothing of this machine's `~/.pi` is read or written.
   */
  const home = join(
    root,
    'home',
  );
  /**
   Agent directory pi reads its settings, models and credential from.
   */
  const agentDir = join(
    home,
    '.pi',
    'agent',
  );
  await mkdir(
    agentDir,
    { recursive: true, },
  );
  await Promise.all([
    writeFile(
      join(
        agentDir,
        'settings.json',
      ),
      `${JSON.stringify({ retry: PROOF_RETRY_SETTINGS, }, null, 1,)}\n`,
      'utf8',
    ),
    writeFile(
      join(
        agentDir,
        'models.json',
      ),
      `${JSON.stringify({
        providers: {
          [PROVIDER]: { baseUrl: `http://127.0.0.1:${String(port,)}/v1`, },
        },
      }, null, 1,)}\n`,
      'utf8',
    ),
    writeFile(
      join(
        agentDir,
        'auth.json',
      ),
      `${JSON.stringify({
        [PROVIDER]: {
          type: 'api_key',
          key: FIXTURE_KEY,
        },
      }, null, 1,)}\n`,
      'utf8',
    ),
  ],);
  return {
    home,
    agentDir,
  };
}

//endregion The throwaway home

//region The stand-in endpoint

/**
 Build the reply body the stand-in answers a chat completion with.

 @returns the server-sent events one reply consists of

 @example
 ```typescript
 const body = replySseBody();
 ```
 */
function replySseBody(): string {
  /**
   Chunk lines the answer carries.
   */
  const chunkLines = [
    JSON.stringify({
      id: 'stand-in',
      object: 'chat.completion.chunk',
      choices: [{
        index: 0,
        delta: {
          role: 'assistant',
          content: REPLY_TEXT,
        },
        finish_reason: null,
      },],
    },),
    JSON.stringify({
      id: 'stand-in',
      object: 'chat.completion.chunk',
      choices: [{
        index: 0,
        delta: {},
        finish_reason: 'stop',
      },],
    },),
  ].map(function toChunkLine(payload,) {
    return `data: ${payload}\n\n`;
  },);
  return `${chunkLines.join('',)}data: [DONE]\n\n`;
}

/**
 Start the loopback endpoint both pi runs talk to.

 The first chat completion it answers is the recorded passing limit, with the
 status the provider sent it under, and every later one is a reply.
 The catalog listing the extension asks for at session start is answered too,
 so no refresh reaches the real provider.

 @returns the listening endpoint

 @example
 ```typescript
 const standIn = await startStandIn();
 ```
 */
async function startStandIn(): Promise<StandIn> {
  /**
   Chat completions the endpoint has answered, kept in an object so the
   handler and the reader share one count:
   `total` over the whole proof, and `thisRun` since the last
   {@link StandIn.beginRun}, where the first answer is the recorded one.
   */
  const counts: {
    total: number;
    thisRun: number;
  } = {
    total: 0,
    thisRun: 0,
  };
  /**
   Endpoint itself.
   */
  const server = createServer(function answerStandInRequest(
    request,
    response,
  ): void {
    if (request.method === 'GET') {
      response.writeHead(
        200,
        { 'content-type': 'application/json', },
      );
      response.end(
        JSON.stringify({
          data: [{
            id: MODEL,
            created: LISTING_CREATED_SECONDS,
          },],
        },),
      );
      return;
    }
    counts.total += 1;
    counts.thisRun += 1;
    if (counts.thisRun === 1) {
      response.writeHead(
        429,
        { 'content-type': 'application/json', },
      );
      response.end(JSON.stringify(RECORDED_PASSING_LIMIT_BODY,),);
      return;
    }
    response.writeHead(
      200,
      { 'content-type': 'text/event-stream', },
    );
    response.end(replySseBody(),);
  },);
  await new Promise<void>(function listen(resolveListen,) {
    server.listen(
      0,
      '127.0.0.1',
      resolveListen,
    );
  },);
  /**
   Address the endpoint was given, narrowed to the shape a listening server
   reports.
   */
  const address = server.address() as AddressInfo;
  return {
    port: address.port,
    server,
    chatRequests: function chatRequests(): number {
      return counts.total;
    },
    beginRun: function beginRun(): void {
      counts.thisRun = 0;
    },
  };
}

//endregion The stand-in endpoint

//region The unchanged package

/**
 Build the bundle the package produced before this translation existed,
 from the commit that introduced it.

 The package's source is copied out of that commit and built with this
 checkout's toolchain,
 so the two runs compare bundles built by the same compiler from the same pi
 and differ only in the package's own source.

 @param root - folder the copy lives in

 @returns absolute path of the unchanged bundle

 @throws naming the search that found no such commit, so a reader can run the
 proof where that history is present

 @example
 ```typescript
 const bundle = await unchangedBundle({ root });
 ```
 */
async function unchangedBundle({
  root,
}: {
  readonly root: string;
},): Promise<string> {
  /**
   Repository root this proof runs inside.
   */
  const repository = repositoryRootOf({ start: process.cwd(), },);
  /**
   Commit that first handed the provider dispatch to this translation.
   */
  const introducedIn = (await RUN_COMMAND(
    'git',
    TRANSLATION_COMMIT_SEARCH,
    { cwd: repository, },
  )).stdout.split('\n',)
    .find(function firstLine(line,) {
      return line.length > 0;
    },);
  if (introducedIn === undefined)
    throw new Error(
      'no commit introduced this translation into src/provider.ts, so the unchanged package cannot be named; run this proof where that history is present',
    );
  /**
   Copy of the package's source at the commit before that one.
   */
  const source = join(
    root,
    'unchanged-source',
  );
  await mkdir(
    source,
    { recursive: true, },
  );
  /**
   Archive file the package directory is unpacked from.
   */
  const archivePath = join(
    root,
    'unchanged-package.tar',
  );
  await RUN_COMMAND(
    'git',
    [
      'archive',
      `--format=tar`,
      `--output=${archivePath}`,
      `${introducedIn}^`,
      PACKAGE_ARCHIVE_PATH,
    ],
    { cwd: repository, },
  );
  await RUN_COMMAND(
    'tar',
    [
      '--extract',
      `--file=${archivePath}`,
      `--directory=${source}`,
    ],
  );
  /**
   Directory the unchanged bundle is built in.
   */
  const packageDirectory = join(
    source,
    PACKAGE_ARCHIVE_PATH,
  );
  // The copy stands outside the repository, so it borrows the dependency trees
  // the package and the repository root already hold:
  // its own peers and workspace packages, and the tsconfig chain the root
  // hoists.
  await RUN_COMMAND(
    'ln',
    [
      '--symbolic',
      join(
        repository,
        PACKAGE_ARCHIVE_PATH,
        'node_modules',
      ),
      join(
        packageDirectory,
        'node_modules',
      ),
    ],
  );
  await RUN_COMMAND(
    'ln',
    [
      '--symbolic',
      join(
        repository,
        'node_modules',
      ),
      join(
        source,
        'node_modules',
      ),
    ],
  );
  await RUN_COMMAND(
    'rolldown',
    [
      '--configLoader',
      'native',
      '--config',
      'rolldown.node.config.ts',
    ],
    { cwd: packageDirectory, },
  );
  return join(
    packageDirectory,
    'dist',
    'final',
    'node',
    'index.mjs',
  );
}

//endregion The unchanged package

//region One pi run

/**
 Parse the events one pi run printed, and keep the raw feed for a reader.

 @param stdout - everything the run wrote to standard output

 @param root - folder the raw feed is kept in

 @param name - run name the feed file is named for

 @returns the parsed session events, and the lines that were not events

 @example
 ```typescript
 const parsed = await parseFeed({ stdout, root, name: 'retried' });
 ```
 */
async function parseFeed({
  stdout,
  root,
  name,
}: {
  readonly stdout: string;
  readonly root: string;
  readonly name: string;
},): Promise<{
  readonly events: readonly Readonly<Record<string, unknown>>[];
  readonly unparsed: readonly string[];
}> {
  await writeFile(
    join(
      root,
      'runs',
      `${name}-feed.jsonl`,
    ),
    stdout,
    'utf8',
  );
  /**
   Parsed events, in the order pi printed them.
   */
  const events: Readonly<Record<string, unknown>>[] = [];
  /**
   Lines that were not events, kept so a broken feed is reported rather than
   hidden.
   */
  const unparsed: string[] = [];
  for (const line of stdout.split('\n',)) {
    if (line.trim().length === 0)
      continue;
    try {
      const parsed: unknown = JSON.parse(line,);
      if (isRecord(parsed,))
        events.push(parsed,);
      else
        unparsed.push(line,);
    }
    catch (error) {
      unparsed.push(`${line} (${caughtValueText(error,)})`,);
    }
  }
  return {
    events,
    unparsed,
  };
}

/**
 Run pi once against the stand-in, keeping its feed and its error output.

 @param root - folder the run lives in

 @param name - run name its files are named for

 @param extension - bundle the run loads

 @param home - throwaway home the run sees

 @param agentDir - agent directory inside that home

 @returns what the run printed, and how it ended

 @example
 ```typescript
 const run = await runPi({ root, name: 'retried', extension, home, agentDir });
 ```
 */
async function runPi({
  root,
  name,
  extension,
  home,
  agentDir,
}: {
  readonly root: string;
  readonly name: string;
  readonly extension: string;
  readonly home: string;
  readonly agentDir: string;
},): Promise<PiRun> {
  /**
   Directory the run works in, outside every checkout,
   so no project file is loaded beside the prompt.
   */
  const cwd = join(
    root,
    'runs',
    `${name}-working-directory`,
  );
  await mkdir(
    cwd,
    { recursive: true, },
  );
  /**
   Child process for this run.
   */
  const child = spawn(
    process.execPath,
    [
      piCliPath(),
      '--mode',
      'json',
      '--offline',
      '--no-session',
      '--no-extensions',
      '--extension',
      extension,
      '--no-tools',
      '--no-skills',
      '--no-prompt-templates',
      '--no-context-files',
      '--no-mcp',
      '--model',
      `${PROVIDER}/${MODEL}`,
      '--print',
      PROMPT,
    ],
    {
      cwd,
      // Standard input is refused rather than piped: pi reads a piped standard
      // input as the prompt, and a pipe nobody closes would hold the run open.
      stdio: ['ignore', 'pipe', 'pipe',],
      env: {
        PATH: process.env['PATH'] ?? '',
        HOME: home,
        PI_CODING_AGENT_DIR: agentDir,
        PI_OFFLINE: '1',
        PI_TELEMETRY: '0',
        PI_SKIP_VERSION_CHECK: '1',
      },
      signal: AbortSignal.timeout(RUN_TIMEOUT_MILLISECONDS,),
    },
  );
  /**
   Standard output the run prints, in order.
   */
  const stdoutChunks: Buffer[] = [];
  /**
   Standard error the run prints, in order.
   */
  const stderrChunks: Buffer[] = [];
  presentStream({ stream: child.stdout, },)
    .on('data', function collectStdout(chunk: Buffer,) {
      stdoutChunks.push(chunk,);
    },);
  presentStream({ stream: child.stderr, },)
    .on('data', function collectStderr(chunk: Buffer,) {
      stderrChunks.push(chunk,);
    },);
  /**
   How the child ended, or which error killed it.
   */
  const ending = await new Promise<{
    readonly code: number | null;
    readonly timedOut: boolean;
  }>(function waitForExit(resolveExit,) {
    child.on('error', function onError(error: Error,) {
      resolveExit({
        code: null,
        timedOut: error.name === 'AbortError',
      },);
    },);
    child.on('exit', function onExit(code: number | null,) {
      resolveExit({
        code,
        timedOut: false,
      },);
    },);
  },);
  /**
   Everything the run wrote to standard error.
   */
  const stderr = Buffer.concat(stderrChunks,)
    .toString('utf8',);
  await writeFile(
    join(
      root,
      'runs',
      `${name}-stderr.log`,
    ),
    stderr,
    'utf8',
  );
  /**
   Session events the run printed.
   */
  const parsed = await parseFeed({
    stdout: Buffer.concat(stdoutChunks,)
      .toString('utf8',),
    root,
    name,
  },);
  return {
    events: parsed.events,
    stderr: parsed.unparsed.length === 0
      ? stderr
      : `${stderr}\nunparsed feed lines:\n${parsed.unparsed.join('\n',)}\n`,
    exitCode: ending.code ?? -1,
    timedOut: ending.timedOut,
  };
}

//endregion One pi run

//region Reading a run

/**
 The assistant messages one run reported.

 @param run - run to read

 @returns each assistant message the run reported, in order

 @example
 ```typescript
 const messages = assistantMessagesOf({ run });
 ```
 */
function assistantMessagesOf({
  run,
}: {
  readonly run: PiRun;
},): Readonly<Record<string, unknown>>[] {
  return run.events.flatMap(function messageOf(event,) {
    if (stringField({
      record: event,
      field: 'type',
    },) !== 'message_end')
      return [];
    /**
     Message the event carried.
     */
    const message = recordField({
      record: event,
      field: 'message',
    },);
    if (message === undefined)
      return [];
    if (stringField({
      record: message,
      field: 'role',
    },) !== 'assistant')
      return [];
    return [message,];
  },);
}

/**
 The waits pi announced before its own automatic retry attempts.

 @param run - run to read

 @returns the announced delays, in order

 @example
 ```typescript
 const delays = retryDelaysOf({ run });
 ```
 */
function retryDelaysOf({
  run,
}: {
  readonly run: PiRun;
},): number[] {
  return run.events.flatMap(function delayOf(event,) {
    if (stringField({
      record: event,
      field: 'type',
    },) !== 'auto_retry_start')
      return [];
    /**
     Wait pi announced before the attempt.
     */
    const delay = event['delayMs'];
    return (typeof delay) === 'number' ? [delay,] : [];
  },);
}

//endregion Reading a run

//region The proof

/**
 Run both hosts and check what each one did.

 @returns one line per check that passed

 @throws listing every check that failed, so one run reports the whole picture

 @example
 ```typescript
 const lines = await proveRetriedByPi();
 ```
 */
async function proveRetriedByPi(): Promise<string[]> {
  /**
   Folder everything this proof writes lives in.
   */
  const root = await proofFolder();
  /**
   Endpoint both runs talk to.
   */
  const standIn = await startStandIn();
  using standInShutdown = {
    [Symbol.dispose](): void {
      standIn.server
        .close();
    },
  };
  /**
   Throwaway home the retried run sees.
   */
  const retriedHome = await throwawayHome({
    root,
    port: standIn.port,
  },);
  /**
   Throwaway home the unchanged run sees,
   so neither run reads the other's catalog store.
   */
  const unchangedHome = await throwawayHome({
    root: join(
      root,
      'unchanged',
    ),
    port: standIn.port,
  },);
  /**
   Bundle this checkout builds.
   */
  const currentBundle = resolve(
    dirname(fileURLToPath(import.meta.url,),),
    '..',
    'dist',
    'final',
    'node',
    'index.mjs',
  );
  /**
   Run loading this checkout's bundle, which meets the recorded answer first.
   */
  standIn.beginRun();
  const retried = await runPi({
    root,
    name: 'retried',
    extension: currentBundle,
    home: retriedHome.home,
    agentDir: retriedHome.agentDir,
  },);
  /**
   Run loading the unchanged package's bundle, built after the first run so
   the two builds cannot interleave.
   */
  standIn.beginRun();
  const unchanged = await runPi({
    root,
    name: 'unchanged',
    extension: await unchangedBundle({ root, },),
    home: unchangedHome.home,
    agentDir: unchangedHome.agentDir,
  },);
  /**
   Checks that passed.
   */
  const passed: string[] = [];
  /**
   Checks that failed.
   */
  const failed: string[] = [];
  /**
   Record one check's verdict.

   @param ok - whether the check passed

   @param line - what it measured
   */
  function check({
    ok,
    line,
  }: {
    readonly ok: boolean;
    readonly line: string;
  },): void {
    (ok ? passed : failed).push(line,);
  }
  /**
   Delays pi announced before its own attempts in the retried run.
   */
  const retriedDelays = retryDelaysOf({ run: retried, },);
  /**
   Assistant messages the retried run reported.
   */
  const retriedMessages = assistantMessagesOf({ run: retried, },);
  /**
   Assistant messages the unchanged run reported.
   */
  const unchangedMessages = assistantMessagesOf({ run: unchanged, },);
  check({
    ok: (retried.exitCode === 0) && (!retried.timedOut),
    line: `the retried run exited ${String(retried.exitCode,)} (timed out: ${String(retried.timedOut,)})`,
  },);
  check({
    ok: retriedDelays.length === 1,
    line: `the retried run's feed carries ${String(retriedDelays.length,)} auto_retry_start line(s), waits ${retriedDelays.map(String,)
      .join(', ') || 'none'}`,
  },);
  check({
    ok: retriedMessages.some(function retriedThenAnswered(message,) {
      return (stringField({
        record: message,
        field: 'stopReason',
      },) === 'error')
        && (stringField({
          record: message,
          field: 'errorMessage',
        },) ?? '')
          .startsWith('429: token rate limit reached',);
    },),
    line: `the retried run's failed answer carries this package's retried text: ${JSON.stringify(retriedMessages[0]?.['errorMessage'],)}`,
  },);
  check({
    ok: retriedMessages.filter(function answered(message,) {
      return stringField({
        record: message,
        field: 'stopReason',
      },) === 'stop';
    },)
      .map(function textOf(message,) {
        return assistantText({ message, },);
      },)
      .includes(REPLY_TEXT,),
    line: `the retried run ended with the stand-in's reply: ${JSON.stringify(retriedMessages.map(function textOf(message,) {
      return assistantText({ message, },);
    },),)}`,
  },);
  check({
    ok: retryDelaysOf({ run: unchanged, },).length === 0,
    line: 'the unchanged run\'s feed carries no auto_retry_start line',
  },);
  check({
    ok: unchangedMessages.length === 1,
    line: `the unchanged run settled on ${String(unchangedMessages.length,)} assistant message(s)`,
  },);
  check({
    ok: (stringField({
      record: unchangedMessages[0] ?? {},
      field: 'errorMessage',
    },) ?? '')
      .includes('insufficient_quota',),
    line: `the unchanged run ended on the provider's own body: ${JSON.stringify(unchangedMessages[0]?.['errorMessage'],)}`,
  },);
  check({
    ok: standIn.chatRequests() >= 2,
    line: `the stand-in answered ${String(standIn.chatRequests(),)} chat completion(s)`,
  },);
  await writeFile(
    join(
      root,
      'summary.txt',
    ),
    [
      ...passed.map(function passing(line,) {
        return `PASS ${line}`;
      },),
      ...failed.map(function failing(line,) {
        return `FAIL ${line}`;
      },),
      `proof folder: ${root}`,
    ].join('\n',) + '\n',
    'utf8',
  );
  if (failed.length > 0)
    throw new Error(
      `the proof failed ${String(failed.length,)} of ${String(passed.length + failed.length,)} checks:\n${failed.join('\n',)}\nproof folder: ${root}`,
    );
  return [
    ...passed,
    `proof folder kept at ${root}`,
  ];
}

//endregion The proof

/**
 Print one line per check this proof made, and fail the task when a check
 failed.

 @example
 ```typescript
 // mise run //package/pi-plugin/qwen-token-plan-individual:verify:token-rate-limit-retried
 ```
 */
async function main(): Promise<void> {
  /**
   Checks the proof made, in order.
   */
  const lines = await proveRetriedByPi();
  for (const line of lines)
    console.log(`PASS ${line}`,);
}

await main();
