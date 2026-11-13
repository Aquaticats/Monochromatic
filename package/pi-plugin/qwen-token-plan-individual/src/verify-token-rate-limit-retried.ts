/**
 Proof, with a real pi, that this provider's passing token-rate limit is asked
 again by pi's own automatic retry,
 and that the unchanged package ends the run on that answer.

 The proof runs pi twice, each in its own throwaway home,
 against a stand-in endpoint on the loopback interface:
 the first chat completion of each run is answered with the recorded `429`
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
  mkdir,
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
  resolve,
} from 'node:path';
import { fileURLToPath, } from 'node:url';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { piCliPath, } from './disposable-host.ts';
import { runPi, } from './verify-token-rate-limit-retried-pi-run.ts';
import { checksOf, } from './verify-token-rate-limit-retried-checks.ts';
import {
  startStandIn,
  type StandIn,
} from './verify-token-rate-limit-retried-stand-in.ts';
import { unchangedBundleOf, } from './verify-token-rate-limit-retried-unchanged-bundle.ts';

//region Constants

/**
 Folder name prefix every proof folder starts with,
 so a reader of the scratch root knows what filled it.
 */
const PROOF_FOLDER_PREFIX =
  'qwen-token-plan-individual-token-rate-limit-retried-';

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
 Reply the stand-in gives every chat completion after the run's first.
 */
const REPLY_TEXT = 'pong';

/**
 Credential the throwaway home stores,
 so pi treats the route as configured and no real secret reaches the proof.
 */
const FIXTURE_KEY = 'fixture-key';

/**
 Creation time the stand-in reports for the model it lists,
 copied from the listing the package pins.
 */
const LISTING_CREATED_SECONDS = 1_794_362_561;

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
 Repository-relative path of the package directory, which both the archive and
 the build name.
 */
const PACKAGE_PATH = 'package/pi-plugin/qwen-token-plan-individual';

/**
 Text whose introducing commit names the package that predates this
 translation.
 */
const TRANSLATION_MARKER = 'streamWithPassingTokenRateLimitRetried';

//endregion Constants

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
      `${JSON.stringify(
        { retry: PROOF_RETRY_SETTINGS, },
        null,
        1,
      )}\n`,
      'utf8',
    ),
    writeFile(
      join(
        agentDir,
        'models.json',
      ),
      `${JSON.stringify(
        {
        providers: {
          [PROVIDER]: { baseUrl: `http://127.0.0.1:${String(port,)}/v1`, },
        },
      },
        null,
        1,
      )}\n`,
      'utf8',
    ),
    writeFile(
      join(
        agentDir,
        'auth.json',
      ),
      `${JSON.stringify(
        {
        [PROVIDER]: {
          type: 'api_key',
          key: FIXTURE_KEY,
        },
      },
        null,
        1,
      )}\n`,
      'utf8',
    ),
  ],);
  return {
    home,
    agentDir,
  };
}

//endregion The throwaway home




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
  const standIn: StandIn = await startStandIn({
    answers: {
      model: MODEL,
      listingCreatedSeconds: LISTING_CREATED_SECONDS,
      replyText: REPLY_TEXT,
    },
  },);
  /**
   Disposal that stops the stand-in however this proof ends.
   */
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
   Pi's CLI entry, which both runs start.
   */
  const piCli = piCliPath();
  /**
   Bundle this checkout builds.
   */
  const currentBundle = resolve(
    import.meta.dirname,
    '..',
    'dist',
    'final',
    'node',
    'index.mjs',
  );
  standIn.beginRun();
  /**
   Run loading this checkout's bundle, which meets the recorded answer first.
   */
  const retried = await runPi({
    root,
    name: 'retried',
    piCli,
    extension: currentBundle,
    home: retriedHome.home,
    agentDir: retriedHome.agentDir,
    provider: PROVIDER,
    model: MODEL,
    prompt: PROMPT,
  },);
  standIn.beginRun();
  /**
   Run loading the unchanged package's bundle,
   built after the first run so the two builds cannot interleave.
   */
  const unchanged = await runPi({
    root,
    name: 'unchanged',
    piCli,
    extension: await unchangedBundleOf({
      root,
      packagePath: PACKAGE_PATH,
      translationMarker: TRANSLATION_MARKER,
    },),
    home: unchangedHome.home,
    agentDir: unchangedHome.agentDir,
    provider: PROVIDER,
    model: MODEL,
    prompt: PROMPT,
  },);
  /**
   Every check the proof makes.
   */
  const checks = checksOf({
    runs: {
      retried,
      unchanged,
      chatRequests: standIn.chatRequests(),
    },
  },);
  /**
   Checks that failed, which end the proof.
   */
  const failed = checks.filter(function failedCheck(check,) {
    return !check.ok;
  },);
  await writeFile(
    join(
      root,
      'summary.txt',
    ),
    `${[
      ...checks.map(function reported(check,) {
        return `${check.ok ? 'PASS' : 'FAIL'} ${check.line}`;
      },),
      `proof folder: ${root}`,
    ].join('\n',)  }\n`,
    'utf8',
  );
  if (failed.length > 0)
    throw new Error(
      `the proof failed ${String(failed.length,)} of ${String(checks.length,)} checks:\n${failed.map(function lineOf(check,) {
        return `FAIL ${check.line}`;
      },)
        .join('\n',)}\nproof folder: ${root}`,
    );
  return [
    ...checks.map(function passingLine(check,) {
      return check.line;
    },),
    `proof folder kept at ${root}`,
  ];
}

//endregion The proof

/**
 The checks the proof made, in order, as the task prints them.

 A check that failed has already thrown by here, so this run only reports the
 ones that held.
 */
const proofLines = await proveRetriedByPi();
for (const line of proofLines)
  console.log(`PASS ${line}`,);
