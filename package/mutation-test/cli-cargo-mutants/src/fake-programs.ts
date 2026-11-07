import { fakeMutantsSimulationSource } from './fake-mutants-simulation.ts';

/**
 Length of a full content-addressed Podman image or container ID: a SHA-256 digest in hexadecimal.
 */
const hexIdLength = 64;

/**
 Separator between the arguments of one recorded call.
 */
export const argumentSeparator = '\u0001';

/**
 Terminator of one recorded call.
 */
export const callTerminator = '\u0002';

/**
 Image ID that the stand-in `podman image inspect` reports for the tested image.
 */
export const fakeTestedImageId: string = 'a'.repeat(hexIdLength);

/**
 Image ID that the stand-in `podman build --iidfile` writes for the campaign image.
 */
export const fakeCampaignImageId: string = 'b'.repeat(hexIdLength);

/**
 Container ID that the stand-in `podman create` prints.
 */
export const fakeContainerId: string = 'c'.repeat(hexIdLength);

/**
 Bytes of the stand-in `cargo-mutants` executable, copied into campaign images and hashed into manifests.
 */
export const fakeCargoMutantsBytes = 'fake cargo-mutants executable\n';

/**
 Stand-in for `podman`: records every call, answers the few calls a campaign makes, and starts nothing.
 It copies the Containerfile out of every build context before the caller removes the context.

 When `images.json` (image reference to image ID) exists in the fixture directory, `image inspect <reference>` and
 `tag <image> <reference>` read and write it, and an unknown reference fails with status 125 and `image not known`, as
 Podman does; without the file every inspection answers the tested image. `FAKE_TAG_LANDS_ON` makes `tag` point the new
 reference at that ID instead, to stand for a tag that moved meanwhile.
 */
export const fakePodmanSource: string = `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const directory = process.env.FAKE_DIR;
const args = process.argv.slice(2);
const calls = path.join(directory, 'calls.log');
fs.appendFileSync(calls, args.join('${argumentSeparator}') + '${callTerminator}');
const number = fs.readFileSync(calls, 'utf8').split('${callTerminator}').length - 1;
const campaignImageId = '${fakeCampaignImageId}';
${fakeMutantsSimulationSource}
const failing = process.env.FAKE_FAIL;
const fail = (name, status) => { if (failing === name) { process.stderr.write('fake podman failure: ' + name + '\\n'); process.exit(status); } };
const registryFile = path.join(directory, 'images.json');
const registry = fs.existsSync(registryFile) ? JSON.parse(fs.readFileSync(registryFile, 'utf8')) : undefined;
if (registry !== undefined && args[0] === 'image' && args[1] === 'inspect' && !args.includes('{{.Config.User}}')) {
  fail('inspect-image', 125);
  const known = registry[args[2]];
  if (known === undefined) { process.stderr.write('Error: ' + args[2] + ': image not known\\n'); process.exit(125); }
  process.stdout.write(known + '\\n');
} else if (registry !== undefined && args[0] === 'tag') {
  fail('tag', 125);
  const source = registry[args[1]] ?? (Object.values(registry).includes(args[1]) ? args[1] : undefined);
  if (source === undefined) { process.stderr.write('Error: ' + args[1] + ': image not known\\n'); process.exit(125); }
  registry[args[2]] = process.env.FAKE_TAG_LANDS_ON ?? source;
  fs.writeFileSync(registryFile, JSON.stringify(registry));
} else if (args[0] === 'image' && args[1] === 'inspect') {
  fail('inspect-image', 125);
  if (args.includes('{{.Config.User}}')) { process.stdout.write((process.env.FAKE_IMAGE_USER === undefined ? '1000:1000' : process.env.FAKE_IMAGE_USER) + '\\n'); process.exit(0); }
  const plain = failing === 'short-image-id' ? 'abc' : '${fakeTestedImageId}';
  process.stdout.write((args.includes('--format') ? plain : JSON.stringify([{ Id: plain }])) + '\\n');
} else if (args[0] === 'build') {
  fail('build', 1);
  const context = args[args.length - 1];
  const fileAt = args.indexOf('--file');
  fs.copyFileSync(fileAt >= 0 ? args[fileAt + 1] : path.join(context, 'Containerfile'), path.join(directory, 'containerfile.' + number));
  const at = args.indexOf('--iidfile');
  if (at >= 0) fs.writeFileSync(args[at + 1], failing === 'short-campaign-id' ? 'sha256:abc\\n' : 'sha256:${fakeCampaignImageId}\\n');
} else if (args[0] === 'create') {
  fs.writeFileSync(path.join(directory, 'created.json'), JSON.stringify(args));
  process.stdout.write((failing === 'short-container-id' ? 'abc' : '${fakeContainerId}') + '\\n');
} else if (args[0] === 'container' && args[1] === 'inspect') {
  const created = JSON.parse(fs.readFileSync(path.join(directory, 'created.json'), 'utf8'));
  const at = created.indexOf('${fakeCampaignImageId}');
  process.stdout.write(JSON.stringify(failing === 'wrong-command' ? ['other'] : created.slice(at + 1)) + '\\n');
} else if (args[0] === 'rm') {
  fail('rm', 7);
} else if (args[0] === 'start') {
  fail('start', 3);
  simulateCargoMutants();
}
`;

/**
 Stand-in for `cargo`: records its call with a leading marker and answers `mutants --list` with one line per `--file`
 scope, or two fixed lines for an unscoped listing.
 */
export const fakeCargoSource: string = `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const args = process.argv.slice(2);
fs.appendFileSync(path.join(process.env.FAKE_DIR, 'calls.log'), ['<cargo>', ...args].join('${argumentSeparator}') + '${callTerminator}');
const files = args.flatMap((argument, index) => (args[index - 1] === '--file' ? [argument] : []));
const lines = files.length === 0 ? ['zz:1:1: unscoped one', 'zz:2:1: unscoped two'] : files.map((file) => file + ':1:1: replace f in ' + file);
process.stdout.write(lines.join('\\n') + '\\n');
`;
