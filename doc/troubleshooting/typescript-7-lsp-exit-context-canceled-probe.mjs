// Drive TypeScript 7's language server over standard input and output with several shutdown sequences
// and print what each leaves on standard error and as exit status.
// Usage, from the repository root: node doc/troubleshooting/typescript-7-lsp-exit-context-canceled-probe.mjs
// TSGO_BINARY=/path/to/a/built/server runs that build instead of the copied package's launcher.
// The server runs unconfined against a disposable project below the system temporary directory,
// with a disposable home and cache; the repository is read only to copy the TypeScript 7.0.2 packages.
import { spawn } from 'node:child_process';
import {
  cpSync,
  mkdirSync,
  mkdtempSync,
  realpathSync,
  writeFileSync
} from 'node:fs';
import { tmpdir } from 'node:os';
import {
  join,
  resolve
} from 'node:path';
import { setTimeout as wait } from 'node:timers/promises';

const store = resolve(
  import.meta.dirname,
  '../../node_modules/.pnpm'
);
const base = realpathSync(
  mkdtempSync(join(
    tmpdir(),
    'typescript-7-lsp-exit-'
  )),
);
const project = join(
  base,
  'project'
);
function write(
  relative,
  text
) {
  const target = join(
    project,
    relative
  );
  mkdirSync(
    join(
      target,
      '..'
    ),
    { recursive: true }
  );
  writeFileSync(
    target,
    text
  );
}
write(
  'package.json',
  `${JSON.stringify({
    name: 'probe',
    private: true,
    type: 'module'
  })  }\n`
);
write(
  'tsconfig.json',
  `${JSON.stringify({
    compilerOptions: {
      strict: true,
      noEmit: true
    },
    include: ['src']
  })  }\n`
);
write(
  'src/main.ts',
  'export const value: number = 1;\n'
);
cpSync(
  join(
    store,
    'typescript@7.0.2',
    'node_modules',
    'typescript'
  ),
  join(
    project,
    'node_modules',
    'typescript'
  ),
  { recursive: true }
);
cpSync(
  join(
    store,
    '@typescript+typescript-linux-x64@7.0.2',
    'node_modules',
    '@typescript',
    'typescript-linux-x64'
  ),
  join(
    project,
    'node_modules',
    '@typescript',
    'typescript-linux-x64'
  ),
  { recursive: true },
);
for (const name of [
  'home',
  'cache'
]) mkdirSync(join(
  base,
  name
));

function frame(message) {
  const body = JSON.stringify(message);
  return `Content-Length: ${  Buffer.byteLength(body)
     }\r\n\r\n${
     body}`;
}

// Start the server, initialize it, open a file, wait for one hover answer, then run `finish`.
async function run(
  name,
  finish
) {
  const child = spawn(
    process.env
      .TSGO_BINARY
      ?? join(
        project,
        'node_modules/typescript/bin/tsc'
      ),
    [
      '--lsp',
      '--stdio'
    ],
    {
      cwd: project,
      env: {
        PATH: process.env
          .PATH,
        HOME: join(
          base,
          'home'
        ),
        XDG_CACHE_HOME: join(
          base,
          'cache'
        )
      },
      stdio: [
        'pipe',
        'pipe',
        'pipe'
      ],
    }
  );
  let stderr = '';
  let buffer = Buffer.alloc(0);
  const waiters = new Map();
  child.stderr
    .on(
    'data',
    chunk => { stderr += chunk; }
    );
  child.stdout
    .on(
    'data',
    chunk => {
      buffer = Buffer.concat([
        buffer,
        chunk
      ]);
      for (;;) {
        const end = buffer.indexOf('\r\n\r\n');
        if (end === (-1)) return;
        const length = Number(/Content-Length: (\d+)/.exec(buffer.subarray(
          0,
          end
        )
          .toString())[1]);
        if (buffer.length < (end
          + 4
          + length)) return;
        const message = JSON.parse(buffer.subarray(
          end + 4,
          end + 4
            + length
        )
          .toString());
        buffer = buffer.subarray(end + 4
          + length);
        if ((message.id !== undefined) && (message.method === undefined)) waiters.get(message.id)?.(message);
      // Requests from the server get an empty success, so the server never waits for the probe.
        else if (message.id !== undefined) child.stdin
          .write(frame({
            jsonrpc: '2.0',
            id: message.id,
            result: null
          }));
      }
    }
    );
  let next = 0;
  function request(
    method,
    params
  ) {
    const id = ++next;
    const answered = new Promise(done => waiters.set(
      id,
      done
    ));
    child.stdin
      .write(frame({
        jsonrpc: '2.0',
        id,
        method,
        params
      }));
    return answered;
  }
  function notify(
    method,
    params
  ) { return child.stdin
    .write(frame({
      jsonrpc: '2.0',
      method,
      params
    })); }
  const exited = new Promise(done => child.on(
    'exit',
    (
      code,
      signal
    ) =>{  done({
      code,
      signal
    }); }
  ));
  const uri = `file://${  project}`;
  await request(
    'initialize',
    {
      processId: process.pid,
      rootUri: uri,
      capabilities: {},
      workspaceFolders: [{
        uri,
        name: 'project'
      }]
    }
  );
  notify(
    'initialized',
    {}
  );
  notify(
    'textDocument/didOpen',
    { textDocument: {
      uri: `${uri  }/src/main.ts`,
      languageId: 'typescript',
      version: 0,
      text: 'export const value: number = 1;\n'
    } }
  );
  const hover = await request(
    'textDocument/hover',
    {
      textDocument: { uri: `${uri  }/src/main.ts` },
      position: {
        line: 0,
        character: 14
      }
    }
  );
  await finish({
    child,
    request,
    notify
  });
  const outcome = await Promise.race([
    exited,
    wait(8_000)
      .then(() => ({ timedOut: true }))
  ]);
  if (outcome.timedOut) child.kill('SIGKILL');
  await wait(100);
  return {
    name,
    hoverAnswered: hover.result !== undefined,
    ...outcome,
    stderr
  };
}

const cases = [
  [
    'shutdown answered, then exit',
    async ({
      request,
      notify
    }) => { await request('shutdown');
      notify('exit'); }
  ],
  [
    'shutdown and exit back to back',
    async ({
      request,
      notify
    }) => { request('shutdown');
      notify('exit'); }
  ],
  [
    'shutdown answered, then standard input closed, no exit',
    async ({
      child,
      request
    }) => { await request('shutdown');
      child.stdin
        .end(); }
  ],
  [
    'standard input closed without shutdown',
    async ({ child }) => { child.stdin
      .end(); }
  ],
  [
    'exit without shutdown',
    async ({ notify }) => { notify('exit'); }
  ],
  [
    'SIGTERM',
    async ({ child }) => { child.kill('SIGTERM'); }
  ],
];
const results = [];
for (const [name, finish] of cases) {
  // Each sequence runs three times, to show that the outcome does not vary between runs.
  for (let attempt = 0; attempt < 3; attempt++) results.push(await run(
    name,
    finish
  ));
}
writeFileSync(
  join(
    base,
    'results.json'
  ),
  JSON.stringify(
    results,
    null,
    2
  )
);
for (const result of results) console.log(JSON.stringify(result));
console.log(`Results: ${  join(
  base,
  'results.json'
)}`);
