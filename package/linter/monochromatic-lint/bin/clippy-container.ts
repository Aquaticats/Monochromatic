#!/usr/bin/env node
/**
 Check the already tested snapshot, with a read-only compiler and bounded container resources.
 */
import { runCaptured } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

/**
 Run Clippy over the tested snapshot with the host's Rust toolchain mounted read-only.

 @throws Error when the toolchain root cannot be read or Clippy exits nonzero

 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  /**
   The tested image; concurrent worktrees set MONOCHROMATIC_LINT_IMAGE_TAG so one snapshot never runs another's image.
   */
  const testImage = `localhost/monochromatic-lint-test:${process.env
    .MONOCHROMATIC_LINT_IMAGE_TAG
    ?? 'development'}`;
  /**
   The host compiler's sysroot query.
   */
  const toolchain = await runCaptured({
    program: 'rustc',
    args: [
      '--print',
      'sysroot',
    ],
    capture: true,
  });
  if (toolchain.status !== 0) throw new Error(toolchain.stderr);
  /**
   The toolchain directory mounted into the container.
   */
  const root = toolchain.stdout
    .trim();
  /**
   What the Clippy container left behind.
   */
  const child = await runCaptured({
    program: 'podman',
    args: [
      'run',
      '--rm',
      '--init',
      '--network=none',
      '--memory=2g',
      '--cpus=2',
      '--pids-limit=128',
      '--security-opt',
      'label=disable',
      '--volume',
      `${root}:/toolchain:ro`,
      '--env',
      'PATH=/toolchain/bin:/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin',
      '--env',
      'RUSTC=/toolchain/bin/rustc',
      '--env',
      'RUSTDOC=/toolchain/bin/rustdoc',
      testImage,
      '/toolchain/bin/cargo',
      'clippy',
      '--offline',
      '--locked',
      '--all-targets',
      '--',
      '-D',
      'warnings',
    ],
    capture: false,
  });
  if (child.status !== 0) throw new Error(`Container Clippy failed: ${child.status}`);
}

await main();
