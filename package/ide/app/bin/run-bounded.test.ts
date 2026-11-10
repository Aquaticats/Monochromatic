// Checks of the bounded podman argv every IDE task hands to podman.
// What: one argv builds from the container bound, the read-only clippy config, the work directory, and the image; the
// network and gpu switches change exactly their own arguments; a cargo run keeps the bounded argv.
// Why: the argv used to be copied into every task body, so a missing clippy mount or a limit change had to be found in
// thirty-six places, and a missed copy showed up only when a task ran.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { boundedArguments, createRunBounded } from './run-bounded.ts';
import type { ContainerBounds } from './container-bounds.ts';
import type { ExecFileSyncOptions } from 'node:child_process';

const containerLimits: ContainerBounds = {
  limitArguments: ['--cpus=2', '--memory=4g'],
  environmentArguments: ['CARGO_BUILD_JOBS=2', 'RUST_TEST_THREADS=2'],
  imageBuildArguments: [],
  summary: 'fixture bound',
};
const rootClippyConfig = '/fixture/clippy.toml';
const workDirectory = '/fixture/package/ide/app';
const image = 'localhost/monochromatic/ide';
const fixture = { image, containerLimits, rootClippyConfig, workDirectory };

test('the argv carries the mounts, the read-only clippy config, and the bound', () => {
  const args = boundedArguments(fixture);
  assert.ok(args.includes(`${workDirectory}:/work`));
  assert.ok(args.includes('ide-cargo:/cargo'));
  assert.ok(args.includes(`${rootClippyConfig}:/clippy-conf/clippy.toml:ro`));
  assert.ok(args.includes('CLIPPY_CONF_DIR=/clippy-conf'));
  assert.equal(args[args.indexOf('--workdir') + 1], '/work');
  assert.ok(args.includes('SLINT_EMIT_DEBUG_INFO=1'));
  assert.ok(args.includes('HELIX_RUNTIME=/work/target/debug/runtime'));
  for (const argument of containerLimits.limitArguments) assert.ok(args.includes(argument));
  for (const argument of containerLimits.environmentArguments) assert.ok(args.includes(argument));
  assert.equal(args.at(-1), image);
});

test('a sealed run keeps the network cut, and network or gpu change only their own arguments', () => {
  const sealed = boundedArguments(fixture);
  const networked = boundedArguments({ ...fixture, network: true });
  const gpu = boundedArguments({ ...fixture, gpu: true });
  assert.ok(sealed.includes('--network=none'));
  assert.ok(!networked.includes('--network=none'));
  assert.ok(!sealed.includes('/dev/dri'));
  assert.ok(gpu.includes('/dev/dri'));
  assert.ok(gpu.includes('VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json'));
  // A gpu run is still sealed: the device arguments do not open the network.
  assert.ok(gpu.includes('--network=none'));
  assert.ok(!boundedArguments({ ...fixture, network: true, gpu: true })?.includes('--network=none'));
});

test('a cargo run is the bounded podman command with cargo in front', () => {
  const calls: {
    command: string;
    args: readonly string[];
    options: ExecFileSyncOptions;
  }[] = [];
  const { runCargo } = createRunBounded({
    ...fixture,
    execute: (command, args, options) => {
      calls.push({ command, args, options });
    },
  });
  runCargo(['build'], { network: true });
  assert.equal(calls.length, 1);
  assert.equal(calls[0]?.command, 'podman');
  assert.deepEqual(calls[0]?.args.slice(-2), ['cargo', 'build']);
  assert.ok(!calls[0]?.args.includes('--network=none'));
  assert.deepEqual(calls[0]?.options, { stdio: 'inherit' });
});
