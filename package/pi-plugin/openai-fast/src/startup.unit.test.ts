/**
 Built extension discovery through the real pi CLI in disposable offline state.

 @module
 */
import { spawnSync, } from 'node:child_process';
import { existsSync, } from 'node:fs';
import { readFile, unlink, writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { CODEX_PROVIDER, OPENAI_PROVIDER, } from '../dist/final/node/index.mjs';
import { fixtureHome, } from './host-fixture-home.ts';
import { fixtureModel, } from './host-fixture-model.ts';
import { fixtureCredential, HOST_TOKEN, } from './host-fixture-provider.ts';

//region Startup discovery uses the built default factory, never manual registration.

/** Independent startup configuration and native authentication branches. */
const scenarios = [
  { configuration: 'both', authentication: 'models', },
  { configuration: 'native', authentication: 'models', },
  { configuration: 'legacy', authentication: 'models', },
  { configuration: 'none', authentication: 'models', },
  { configuration: 'both', authentication: 'oauth', },
  { configuration: 'native', authentication: 'oauth', },
  { configuration: 'legacy', authentication: 'oauth', },
  { configuration: 'native', authentication: 'stored-key', },
  { configuration: 'native', authentication: 'environment', },
  { configuration: 'native', authentication: 'command', },
] as const;

await describe({ name: 'built default extension startup', children: scenarios.map(function configurationCase({ configuration, authentication, },) {
  return it({ name: `shows fast providers only for configured sources: ${configuration}, ${authentication}`, fn: async function startupDiscovery() {
    /** Every child-created file stays in this fixture's independently disposable home. */
    await using home = await fixtureHome();
    /** Resolved host entry anchors the installed CLI and its native resource directory. */
    const hostEntry = new URL(import.meta.resolve('@earendil-works/pi-coding-agent'),);
    /** Only the consumer artifact is loaded, not package TypeScript implementation. */
    const extension = fileURLToPath(new URL('../dist/final/node/index.mjs', import.meta.url,),);
    /** A shared model ID detects missing registration for either independent namespace. */
    const model = fixtureModel({ id: 'startup-openai-fixture', },);
    /** Native and legacy configuration are independent inputs to availability. */
    const nativeConfigured = (configuration === 'both') || (configuration === 'native');
    /** Legacy readiness must not be inferred from the native provider's authentication. */
    const legacyConfigured = (configuration === 'both') || (configuration === 'legacy');
    /** Command-backed credentials must remain unevaluated during availability checks. */
    const marker = join(home.root, 'auth-command-ran',);
    /** Fixed command grammar passes the disposable path through the child environment. */
    const commandKey = '!node --eval \'require("node:fs").writeFileSync(process.env.PI_OPENAI_FAST_AVAILABILITY_MARKER,"executed"); process.stdout.write("sk-command-fixture")\'';
    if (authentication === 'command') {
      await writeFile(marker, 'positive control',);
      expect(existsSync(marker,),).toBe(true,);
      await unlink(marker,);
    }
    await writeFile(home.modelsPath, JSON.stringify({ providers: {
      [CODEX_PROVIDER]: { ...((authentication === 'models') && legacyConfigured ? { apiKey: HOST_TOKEN, } : {}), models: [model,], },
      [OPENAI_PROVIDER]: { ...(nativeConfigured && ((authentication === 'models') || (authentication === 'command'))
        ? { apiKey: authentication === 'command' ? commandKey : 'sk-startup-fixture', } : {}), models: [{ ...model, api: 'openai-responses',
          provider: OPENAI_PROVIDER, baseUrl: 'https://api.openai.com/v1', },], },
    }, },),);
    /** Expired synthetic OAuth still counts as configured without resolving or refreshing it. */
    const oauth = fixtureCredential({ expired: true, },);
    /** Stored credentials belong to original identities only, never the fast namespaces. */
    const stored = {
      ...((authentication === 'oauth') && legacyConfigured ? { [CODEX_PROVIDER]: oauth, } : {}),
      ...((authentication === 'oauth') && nativeConfigured ? { [OPENAI_PROVIDER]: oauth, } : {}),
      ...((authentication === 'stored-key') && nativeConfigured ? { [OPENAI_PROVIDER]: { type: 'api_key', key: 'sk-stored-fixture', }, } : {}),
    };
    /** Exact stored bytes detect any unintended native token refresh or fast-credential write. */
    const serializedAuth = JSON.stringify(stored,);
    /** Disposable native auth path is shared by the CLI and its startup readiness runtime. */
    const authPath = join(home.agentDir, 'auth.json',);
    await writeFile(authPath, serializedAuth,);
    await writeFile(join(home.agentDir, 'settings.json',), JSON.stringify({ packages: [],
      enableAnalytics: false, enableInstallTelemetry: false, },),);
    /** Listing initializes the real factory but sends no inference request. */
    const result = spawnSync(process.execPath, [fileURLToPath(new URL('cli.js', hostEntry,),),
      '--offline', '--no-approve', '--no-extensions', '--extension', extension,
      '--no-skills', '--no-themes', '--no-prompt-templates', '--no-context-files',
      '--list-models', model.id,], {
        cwd: home.cwd, timeout: 15_000, maxBuffer: 1_048_576, encoding: 'utf8',
        env: { PATH: process.env.PATH ?? '', HOME: home.root, PI_CODING_AGENT_DIR: home.agentDir,
          PI_PACKAGE_DIR: fileURLToPath(new URL('../', hostEntry,),), PI_OFFLINE: '1',
          PI_OPENAI_FAST_AVAILABILITY_MARKER: marker,
          ...((authentication === 'environment') && nativeConfigured ? { OPENAI_API_KEY: 'sk-environment-fixture', } : {}), },
      },);
    if (result.error !== undefined)
      throw result.error;
    expect(result.status,).toBe(0,);
    /** Real CLI rows expose precisely the available provider identities. */
    const providers = new Set(result.stdout.split('\n',).map(function providerColumn(line,) {
      return line.trim().split(' ',)[0];
    },));
    expect(providers.has(OPENAI_PROVIDER,),).toBe(nativeConfigured,);
    expect(providers.has(CODEX_PROVIDER,),).toBe(legacyConfigured,);
    expect(providers.has('openai-fast',),).toBe(nativeConfigured,);
    expect(providers.has('openai-codex-fast',),).toBe(legacyConfigured,);
    expect(result.stderr,).toBe('',);
    expect(await readFile(authPath, 'utf8',),).toBe(serializedAuth,);
    expect(existsSync(marker,),).toBe(false,);
  }, timeout: 30_000, },);
},), },);

//endregion Startup discovery.
