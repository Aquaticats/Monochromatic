/**
 Stand-in programs for the tests of the workflow step programs: a `curl` that answers from a scenario file, a `podman`
 that records what it is asked, and the `cargo-binstall` and git policy CLI that the catalog-specifiers steps run.

 Each is the text of a Node script written into a folder that comes first on `PATH`, so a step program started with this
 `PATH` cannot reach a registry, a release or a container engine. They are shared with the old-versus-new comparison of the
 shell steps these programs replaced, so both run against the same answers.

 @module
 */

/**
 Name of the environment variable that holds the folder the stand-ins write their records into.
 */
export const FAKE_RECORD_DIRECTORY_VARIABLE = 'FAKE_RECORD_DIRECTORY';

/**
 Name of the environment variable that holds the path of the JSON file with the answers `curl` gives.
 */
export const FAKE_CURL_SCENARIO_VARIABLE = 'FAKE_CURL_SCENARIO';

/**
 File, in the record folder, with one JSON line per `curl` run holding its arguments.
 */
export const CURL_CALLS_FILE = 'curl-calls.jsonl';

/**
 File, in the record folder, with what `curl --config -` read on standard input.
 */
export const CURL_CONFIG_FILE = 'curl-config-input.log';

/**
 File, in the record folder, with one JSON line per `podman` run holding its arguments.
 */
export const PODMAN_CALLS_FILE = 'podman-calls.jsonl';

/**
 File, in the record folder, with what `podman login --password-stdin` read.
 */
export const PODMAN_LOGIN_INPUT_FILE = 'podman-login-input.log';

/**
 Source of the stand-in `curl`.

 It reads `--output`, `--write-out`, `--head`, `--fail`, `--show-error`, `--config -` and the address, ignores the other
 options, and answers with the first scenario rule whose `urlIncludes` text the address contains: its `status`, and its
 `body` or the content of its `bodyFile`. A rule with `exit` ends with that status; no matching rule is a host that cannot
 be resolved (status 6).
 */
export const FAKE_CURL_SOURCE: string = `
const fs = require('node:fs');
const path = require('node:path');
const argv = process.argv.slice(2);
const options = { output: '', writeOut: '', head: false, fail: false, showError: false, config: false, url: '' };
for (let index = 0; index < argv.length; index += 1) {
  const argument = argv[index];
  if (argument === '--output') { index += 1; options.output = argv[index]; }
  else if (argument === '--write-out') { index += 1; options.writeOut = argv[index]; }
  else if ((argument === '--max-time') || (argument === '--retry') || (argument === '--retry-delay')) { index += 1; }
  else if (argument === '--config') { index += 1; options.config = argv[index] === '-'; }
  else if (argument === '--head') { options.head = true; }
  else if (argument === '--fail') { options.fail = true; }
  else if (argument === '--show-error') { options.showError = true; }
  else if ((argument === '--silent') || (argument === '--location')) { /* accepted, nothing to do */ }
  else { options.url = argument; }
}
const records = process.env.${FAKE_RECORD_DIRECTORY_VARIABLE};
fs.appendFileSync(path.join(records, '${CURL_CALLS_FILE}'), JSON.stringify(argv) + '\\n');
if (options.config) {
  fs.appendFileSync(path.join(records, '${CURL_CONFIG_FILE}'), fs.readFileSync(0, 'utf8') + '---\\n');
}
const scenario = JSON.parse(fs.readFileSync(process.env.${FAKE_CURL_SCENARIO_VARIABLE}, 'utf8'));
const rule = scenario.rules.find((candidate) => options.url.includes(candidate.urlIncludes));
function finish(status, code) {
  process.stdout.write(options.writeOut.replace('%{http_code}', String(status)));
  process.exit(code);
}
if (rule === undefined) {
  if (options.showError) { process.stderr.write('curl: (6) Could not resolve host: fake\\n'); }
  finish(0, 6);
}
if (rule.exit !== undefined) {
  if (options.showError) { process.stderr.write('curl: (' + rule.exit + ') fake transport failure\\n'); }
  finish(0, rule.exit);
}
if (options.fail && (rule.status >= 400)) {
  if (options.showError) { process.stderr.write('curl: (22) The requested URL returned error: ' + rule.status + '\\n'); }
  finish(rule.status, 22);
}
const body = rule.bodyFile === undefined ? Buffer.from(rule.body ?? '') : fs.readFileSync(rule.bodyFile);
if ((options.output !== '') && !options.head) { fs.writeFileSync(options.output, body); }
finish(rule.status, 0);
`;

/**
 Source of the stand-in `podman`.

 `--version` prints a fixed version; `build` prints one line and ends with the status in `FAKE_PODMAN_BUILD_EXIT`;
 `login` records what it read on standard input; `push --digestfile <file>` writes `FAKE_PODMAN_DIGEST` (a well-formed
 digest when unset) into the file and ends with the status in `FAKE_PODMAN_PUSH_EXIT`.
 */
export const FAKE_PODMAN_SOURCE: string = `
const fs = require('node:fs');
const path = require('node:path');
const args = process.argv.slice(2);
const records = process.env.${FAKE_RECORD_DIRECTORY_VARIABLE};
fs.appendFileSync(path.join(records, '${PODMAN_CALLS_FILE}'), JSON.stringify(args) + '\\n');
if (args[0] === '--version') {
  console.log('podman version 9.9.9-fake');
} else if (args[0] === 'build') {
  console.log('STEP 1/1: FROM fake');
  process.exit(Number(process.env.FAKE_PODMAN_BUILD_EXIT ?? 0));
} else if (args[0] === 'login') {
  fs.appendFileSync(path.join(records, '${PODMAN_LOGIN_INPUT_FILE}'), fs.readFileSync(0, 'utf8') + '---\\n');
  console.log('Login Succeeded!');
} else if (args[0] === 'push') {
  const digestFile = args[args.indexOf('--digestfile') + 1];
  fs.writeFileSync(digestFile, process.env.FAKE_PODMAN_DIGEST ?? ('sha256:' + 'a'.repeat(64)));
  console.log('Writing manifest to image destination');
  process.exit(Number(process.env.FAKE_PODMAN_PUSH_EXIT ?? 0));
}
`;

/**
 Source of the stand-in `cargo-binstall`, packed into the archive the catalog-specifiers lock row points at.

 It records its arguments, then installs into the folder after `--root`: with `FAKE_BINSTALL_PROGRAM` set to
 `monochromatic-git-policy-cli` or `cli-git-native` it writes that program (a script that prints one version line) into
 `<root>/bin`; with `FAKE_BINSTALL_EXIT` set it prints a failure and ends with that status instead.
 */
export const FAKE_BINSTALL_SOURCE: string = `
const fs = require('node:fs');
const path = require('node:path');
const args = process.argv.slice(2);
fs.appendFileSync(path.join(process.env.${FAKE_RECORD_DIRECTORY_VARIABLE}, 'binstall-calls.jsonl'), JSON.stringify(args) + '\\n');
if (process.env.FAKE_BINSTALL_EXIT !== undefined) {
  console.error('cargo-binstall: ERROR Fatal error: fake failure');
  process.exit(Number(process.env.FAKE_BINSTALL_EXIT));
}
const bin = path.join(args[args.indexOf('--root') + 1], 'bin');
fs.mkdirSync(bin, { recursive: true });
const program = process.env.FAKE_BINSTALL_PROGRAM;
fs.writeFileSync(path.join(bin, program), '#!' + process.execPath + '\\nconsole.log("' + program + ' 9.9.9-fake");\\n', { mode: 0o755 });
`;

/**
 Source of the stand-in git policy CLI that the negative control of catalog-specifiers runs.

 It prints `FAKE_CLI_STDOUT` and ends with the status in `FAKE_CLI_EXIT` (0 when unset), and records its arguments and the
 folder it ran in.
 */
export const FAKE_POLICY_CLI_SOURCE: string = `
const fs = require('node:fs');
const path = require('node:path');
fs.appendFileSync(
  path.join(process.env.${FAKE_RECORD_DIRECTORY_VARIABLE}, 'policy-cli-calls.jsonl'),
  JSON.stringify({ args: process.argv.slice(2), cwd: process.cwd() }) + '\\n',
);
process.stdout.write(process.env.FAKE_CLI_STDOUT ?? '');
process.exit(Number(process.env.FAKE_CLI_EXIT ?? 0));
`;

/**
 Source of a program packed into a fake release archive, which prints its name and a fixed version for `--version`
 and ends with the status in `FAKE_TOOL_VERSION_EXIT` (0 when unset).
 */
export const FAKE_RELEASE_PROGRAM_SOURCE = `
console.log(require('node:path').basename(process.argv[1]) + ' 1.2.3-fake');
process.exit(Number(process.env.FAKE_TOOL_VERSION_EXIT ?? 0));
`;
