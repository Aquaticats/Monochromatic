/**
 The typed toolchain plan data: every tool from `mise.no-env.toml` `[tools]`
 (lines 29 to 308), in that file's order, plus the pinned `mpm` bootstrap entry.

 Each entry records the request intent from `mise.no-env.toml`, the resolved
 version `mise.lock` carried on 2026-10-26, the install route, the MPM manager
 identity, the exposure bins measured from the Mise-installed trees on that
 date, OS filters, provider options, tool environment exports, and the
 per-platform artifact provenance from `plan.artifacts.data.ts`.
 */

import { artifacts, } from './plan.artifacts.data.ts';
import type { ToolPlanEntry, } from './types.ts';

//region Tools

/**
 Every tool from `mise.no-env.toml` `[tools]`, in declaration order.
 */
export const TOOLS: readonly ToolPlanEntry[] = [
  // Pinned to 1.3.14 because the Rust rewrite's robustness invites serious doubts.
  {
    id: 'bun',
    request: { pinned: '1.3.14' },
    version: '1.3.14',
    route: 'bin-sourced',
    bins: [
      'bun',
      'bunx'
    ],
    options: { binAliases: { bunx: 'bun' } },
    env: {},
    artifacts: artifacts('bun'),
  },
  {
    id: 'pnpm',
    request: { track: 'latest' },
    version: '12.16.1',
    route: 'bin-sourced',
    bins: [
      'pn',
      'pnpm',
      'pnpx',
      'pnx'
    ],
    options: { binAliases: {
      pn: 'pnpm',
      pnpx: 'pnpm',
      pnx: 'pnpm'
    } },
    env: {},
    artifacts: artifacts('pnpm'),
  },
  // aqua:sharkdp/fd broke in 2026-03-07; cargo-binstall is the fallback route.
  {
    id: 'cargo:fd-find',
    request: { track: 'latest' },
    version: '10.5.0',
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: 'fd-find',
    bins: ['fd'],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'ripgrep',
    request: { track: 'latest' },
    version: '15.2.0',
    route: 'bin-sourced',
    bins: ['rg'],
    options: {},
    env: {},
    artifacts: artifacts('ripgrep'),
  },
  {
    id: 'cargo:fastmod',
    request: { track: 'latest' },
    version: '0.4.4',
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: 'fastmod',
    bins: ['fastmod'],
    options: {},
    env: {},
    artifacts: {},
  },
  // Pinned to 0.13.2: the --fuzzing-workspace and libfuzzer-sys integration
  // shifted across point releases; this is the verified rollout version.
  {
    id: 'cargo:cargo-fuzz',
    request: { pinned: '0.13.2' },
    version: '0.13.2',
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: 'cargo-fuzz',
    bins: ['cargo-fuzz'],
    options: {},
    env: {},
    artifacts: {},
  },
  // `bin/ocr` in the Mise tree is a stray link into a global npm package,
  // not a Node binary; the exposure links only Node's own binaries.
  {
    id: 'node',
    request: { track: 'latest' },
    version: '26.13.0',
    route: 'archive',
    bins: [
      'node',
      'npm',
      'npx'
    ],
    options: {},
    env: {},
    artifacts: artifacts('node'),
  },
  {
    id: 'harper-cli',
    request: { track: 'latest' },
    version: '3.0.0',
    route: 'bin-sourced',
    bins: ['harper-cli'],
    options: {},
    env: {},
    artifacts: artifacts('harper-cli'),
  },
  {
    id: 'dprint',
    request: { track: 'latest' },
    version: '0.64.3',
    route: 'bin-sourced',
    bins: ['dprint'],
    options: {},
    env: {},
    artifacts: artifacts('dprint'),
  },
  {
    id: 'uv',
    request: { track: 'latest' },
    version: '0.13.6',
    route: 'archive',
    bins: [
      'uv',
      'uvx'
    ],
    options: {},
    env: {},
    artifacts: artifacts('uv'),
  },
  {
    id: 'hyperfine',
    request: { track: 'latest' },
    version: '2.0.0',
    route: 'bin-sourced',
    bins: ['hyperfine'],
    options: {},
    env: {},
    artifacts: artifacts('hyperfine'),
  },
  {
    id: 'caddy',
    request: { track: 'latest' },
    version: '2.11.7',
    route: 'bin-sourced',
    bins: ['caddy'],
    options: {},
    env: {},
    artifacts: artifacts('caddy'),
  },
  {
    id: 'watchexec',
    request: { track: 'latest' },
    version: '2.8.0',
    route: 'bin-sourced',
    bins: ['watchexec'],
    options: {},
    env: {},
    artifacts: artifacts('watchexec'),
  },
  {
    id: 'sops',
    request: { track: 'latest' },
    version: '3.13.3',
    route: 'bin-sourced',
    bins: ['sops'],
    options: {},
    env: {},
    artifacts: artifacts('sops'),
  },
  {
    id: 'age',
    request: { track: 'latest' },
    version: '1.3.2',
    route: 'archive',
    bins: [
      'age',
      'age-inspect',
      'age-keygen',
      'age-plugin-batchpass',
      'age-plugin-pq',
      'age-plugin-tag',
      'age-plugin-tagpq',
    ],
    options: {},
    env: {},
    artifacts: artifacts('age'),
  },
  // Floating nightly channel, resolved to the dated toolchain string; the pin
  // rides in the MPM rustup package id because that adapter keeps only the id.
  {
    id: 'rust',
    request: { channel: 'nightly' },
    version: 'nightly-2026-10-29',
    route: 'rustup-toolchain',
    manager: 'rustup',
    packageId: 'nightly-2026-10-29',
    bins: [
      'cargo',
      'cargo-clippy',
      'cargo-fmt',
      'clippy-driver',
      'rust-gdb',
      'rust-gdbgui',
      'rust-lldb',
      'rustc',
      'rustdoc',
      'rustfmt',
      'rustup',
    ],
    options: {
      rustComponents: [
        'clippy',
        'rust-src',
        'llvm-tools',
        'rustc-dev'
      ],
      rustTargets: [
        'aarch64-linux-android',
        'x86_64-linux-android'
      ],
    },
    env: { RUSTUP_TOOLCHAIN: 'nightly-2026-10-29' },
    artifacts: {},
  },
  {
    id: 'cargo-binstall',
    request: { track: 'latest' },
    version: '1.26.1',
    route: 'bin-sourced',
    bins: ['cargo-binstall'],
    options: {},
    env: {},
    artifacts: artifacts('cargo-binstall'),
  },
  {
    id: 'cargo:timeout-cli',
    request: { track: 'latest' },
    version: '0.1.0',
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: 'timeout-cli',
    bins: ['timeout'],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'cargo:cargo-dylint',
    request: { track: 'latest' },
    version: '6.3.0',
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: 'cargo-dylint',
    bins: ['cargo-dylint'],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'cargo:dylint-link',
    request: { track: 'latest' },
    version: '6.3.0',
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: 'dylint-link',
    bins: ['dylint-link'],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'cargo:cargo-nextest',
    request: { track: 'latest' },
    version: '0.9.150',
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: 'cargo-nextest',
    bins: ['cargo-nextest'],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'cargo:cargo-about',
    request: { track: 'latest' },
    version: '0.9.3',
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: 'cargo-about',
    bins: ['cargo-about'],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'cargo:monochromatic-lint',
    request: { track: 'latest' },
    version: '0.3.0',
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: 'monochromatic-lint',
    bins: ['monochromatic-lint'],
    options: {},
    env: {},
    artifacts: {},
  },
  // The postinstall links `cli-git-native` as `git` under `.cache/cli-git-bin`
  // with a temporary name plus rename (doc/planning/cli-git-native-cutover.md).
  {
    id: 'cargo:monochromatic-git-policy-cli',
    request: { track: 'latest' },
    version: '0.2.12',
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: 'monochromatic-git-policy-cli',
    bins: ['cli-git-native'],
    os: ['linux'],
    options: {
      postinstall: {
        when: 'always',
        link: {
          sourceBin: 'cli-git-native',
          linkName: 'git',
          linkDir: '.cache/cli-git-bin',
        },
      },
    },
    env: {},
    artifacts: {},
  },
  // The postinstall runs `search-shadow link .cache/search-shadow-bin` after the exposure directory
  // is rebuilt: the crate links itself under the name of every search, find and count tool whose real
  // binary it finds on PATH (rg, grep, find, fd, du and others; the crate owns the list). That directory
  // precedes `.cache/toolchain-bin` on PATH, so those tools in the mise environment run behind the
  // shadow (doc/planning/agent-search-io-throttle.md). Linux only: the crate uses `ioprio_set` and `/proc`.
  {
    id: 'cargo:monochromatic-search-shadow',
    request: { track: 'latest' },
    version: '0.2.0',
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: 'monochromatic-search-shadow',
    bins: ['search-shadow'],
    os: ['linux'],
    options: {
      postinstall: {
        when: 'always',
        run: {
          sourceBin: 'search-shadow',
          args: ['link', '.cache/search-shadow-bin',],
        },
      },
    },
    env: {},
    artifacts: {},
  },
  {
    id: 'cargo:monochromatic-nested-wayland-session',
    request: { track: 'latest' },
    version: '0.6.2',
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: 'monochromatic-nested-wayland-session',
    bins: ['monochromatic-nested-wayland-session'],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'cmake',
    request: { track: 'latest' },
    version: '4.4.5',
    route: 'archive',
    bins: [
      'ccmake',
      'cmake',
      'cmake-gui',
      'cpack',
      'ctest'
    ],
    options: {},
    env: {},
    artifacts: artifacts('cmake'),
  },
  {
    id: 'cargo:apple-codesign',
    request: { track: 'latest' },
    version: '0.29.0',
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: 'apple-codesign',
    bins: ['rcodesign'],
    options: {},
    env: {},
    artifacts: {},
  },
  // Pinned to 0.15.2 to match the terminal Containerfile's ZIG_VERSION
  // (doc/decision/desktop-app-podman-build.md); bump both together.
  {
    id: 'zig',
    request: { pinned: '0.15.2' },
    version: '0.15.2',
    route: 'archive',
    bins: ['zig'],
    options: {},
    env: {},
    artifacts: artifacts('zig'),
  },
  {
    id: 'cargo:slint-lsp',
    request: { track: 'latest' },
    version: '1.18.1',
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: 'slint-lsp',
    bins: ['slint-lsp'],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'cargo:slint-viewer',
    request: { track: 'latest' },
    version: '1.18.1',
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: 'slint-viewer',
    bins: ['slint-viewer'],
    options: {},
    env: {},
    artifacts: {},
  },
  // conda-forge artifact fetched directly: no MPM conda restore here, because
  // the per-tool OS filter is unexpressible and conda itself is not provisioned.
  {
    id: 'conda:bubblewrap',
    request: { track: 'latest' },
    version: '0.13.1',
    route: 'archive',
    manager: 'conda',
    packageId: 'bubblewrap',
    bins: ['bwrap'],
    os: ['linux'],
    options: {},
    env: {},
    artifacts: artifacts('conda:bubblewrap'),
  },
  {
    id: 'taplo',
    request: { track: 'latest' },
    version: '0.10.0',
    route: 'archive',
    bins: [
      'taplo',
      'taplo-linux-x86_64'
    ],
    options: {
      // The gzip payload installs as `taplo`; the asset-name bin resolves
      // to the same file through the alias.
      binAliases: { 'taplo-linux-x86_64': 'taplo' },
    },
    env: {},
    artifacts: artifacts('taplo'),
  },
  {
    id: 'tombi',
    request: { track: 'latest' },
    version: '1.9.7',
    route: 'bin-sourced',
    bins: ['tombi'],
    options: {},
    env: {},
    artifacts: artifacts('tombi'),
  },
  {
    id: 'marksman',
    request: { track: 'latest' },
    version: '2026-02-08',
    route: 'bin-sourced',
    bins: ['marksman'],
    options: {},
    env: {},
    artifacts: artifacts('marksman'),
  },
  {
    id: 'rumdl',
    request: { track: 'latest' },
    version: '0.3.2',
    route: 'bin-sourced',
    bins: ['rumdl'],
    options: {},
    env: {},
    artifacts: artifacts('rumdl'),
  },
  {
    id: 'github:Feel-ix-343/markdown-oxide',
    request: { track: 'latest' },
    version: '0.25.12',
    route: 'bin-sourced',
    bins: ['markdown-oxide'],
    options: {},
    env: {},
    artifacts: artifacts('github:Feel-ix-343/markdown-oxide'),
  },
  {
    id: 'zizmor',
    request: { track: 'latest' },
    version: '1.31.1',
    route: 'bin-sourced',
    bins: ['zizmor'],
    options: {},
    env: {},
    artifacts: artifacts('zizmor'),
  },
  {
    id: 'npm:yaml-language-server',
    request: { track: 'latest' },
    version: '2.0.0',
    route: 'mpm-pnpm',
    manager: 'pnpm',
    packageId: 'yaml-language-server',
    bins: ['yaml-language-server'],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'npm:@actions/languageserver',
    request: { track: 'latest' },
    version: '0.3.61',
    route: 'mpm-pnpm',
    manager: 'pnpm',
    packageId: '@actions/languageserver',
    bins: ['actions-languageserver'],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'npm:@ansible/ansible-language-server',
    request: { track: 'latest' },
    version: '26.6.0',
    route: 'mpm-pnpm',
    manager: 'pnpm',
    packageId: '@ansible/ansible-language-server',
    bins: ['ansible-language-server'],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'npm:vscode-langservers-extracted',
    request: { track: 'latest' },
    version: '4.10.0',
    route: 'mpm-pnpm',
    manager: 'pnpm',
    packageId: 'vscode-langservers-extracted',
    bins: [
      'vscode-css-language-server',
      'vscode-eslint-language-server',
      'vscode-html-language-server',
      'vscode-json-language-server',
      'vscode-markdown-language-server',
    ],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'superhtml',
    request: { track: 'latest' },
    version: '0.7.0',
    route: 'bin-sourced',
    bins: ['superhtml'],
    options: {},
    env: {},
    artifacts: artifacts('superhtml'),
  },
  {
    id: 'npm:bash-language-server',
    request: { track: 'latest' },
    version: '5.8.1',
    route: 'mpm-pnpm',
    manager: 'pnpm',
    packageId: 'bash-language-server',
    bins: ['bash-language-server'],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'npm:dockerfile-language-server-nodejs',
    request: { track: 'latest' },
    version: '0.15.0',
    route: 'mpm-pnpm',
    manager: 'pnpm',
    packageId: 'dockerfile-language-server-nodejs',
    bins: ['docker-langserver'],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'npm:@microsoft/compose-language-service',
    request: { track: 'latest' },
    version: '1.0.0',
    route: 'mpm-pnpm',
    manager: 'pnpm',
    packageId: '@microsoft/compose-language-service',
    bins: ['docker-compose-langserver'],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'github:docker/docker-language-server',
    request: { track: 'latest' },
    version: '0.20.1',
    route: 'bin-sourced',
    bins: ['docker-language-server'],
    options: {},
    env: {},
    artifacts: artifacts('github:docker/docker-language-server'),
  },
  // Pinned to b9829: llama.cpp tags before the Linux build finishes uploading,
  // so a floating install can miss the ubuntu-x64 asset entirely.
  {
    id: 'llama.cpp',
    request: { pinned: 'b9829' },
    version: 'b9829',
    route: 'archive',
    bins: [
      'ggml-rpc-server',
      'llama',
      'llama-batched-bench',
      'llama-bench',
      'llama-cli',
      'llama-completion',
      'llama-debug-template-parser',
      'llama-fit-params',
      'llama-gemma3-cli',
      'llama-gguf-split',
      'llama-imatrix',
      'llama-llava-cli',
      'llama-minicpmv-cli',
      'llama-mtmd-cli',
      'llama-mtmd-debug',
      'llama-perplexity',
      'llama-quantize',
      'llama-qwen2vl-cli',
      'llama-results',
      'llama-server',
      'llama-template-analysis',
      'llama-tokenize',
      'llama-tts',
    ],
    options: {},
    env: {},
    artifacts: artifacts('llama.cpp'),
  },
  {
    id: 'opentofu',
    request: { track: 'latest' },
    version: '1.13.2',
    route: 'bin-sourced',
    bins: ['tofu'],
    options: {},
    env: {},
    artifacts: artifacts('opentofu'),
  },
  {
    id: 'hcloud',
    request: { track: 'latest' },
    version: '1.70.2',
    route: 'bin-sourced',
    bins: ['hcloud'],
    options: {},
    env: {},
    artifacts: artifacts('hcloud'),
  },
  {
    id: 'npm:typescript-language-server',
    request: { track: 'latest' },
    version: '6.0.2',
    route: 'mpm-pnpm',
    manager: 'pnpm',
    packageId: 'typescript-language-server',
    bins: ['typescript-language-server'],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'cargo:coreutils',
    request: { track: 'latest' },
    version: '0.12.0',
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: 'coreutils',
    bins: ['coreutils'],
    options: { cargoFeatures: ['unix'] },
    env: {},
    artifacts: {},
  },
  {
    id: 'npm:socket',
    request: { track: 'latest' },
    version: '1.11.2',
    route: 'mpm-pnpm',
    manager: 'pnpm',
    packageId: 'socket',
    bins: [
      'socket',
      'socket-npm',
      'socket-npx',
      'socket-pnpm',
      'socket-yarn'
    ],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'npm:wrangler',
    request: { track: 'latest' },
    version: '4.155.0',
    route: 'mpm-pnpm',
    manager: 'pnpm',
    packageId: 'wrangler',
    bins: [
      'cf-wrangler',
      'wrangler',
      'wrangler2'
    ],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'npm:pagefind',
    request: { track: 'latest' },
    version: '1.5.2',
    route: 'mpm-pnpm',
    manager: 'pnpm',
    packageId: 'pagefind',
    bins: ['pagefind'],
    options: {},
    env: {},
    artifacts: {},
  },
  {
    id: 'betterleaks',
    request: { track: 'latest' },
    version: '1.9.0',
    route: 'bin-sourced',
    bins: ['betterleaks'],
    options: {},
    env: {},
    artifacts: artifacts('betterleaks'),
  },
  // Temurin 21: AGP 9 / Gradle 9 need a JDK in the 17 to 21 range.
  {
    id: 'java',
    request: { channel: 'temurin-21' },
    version: 'temurin-21.0.13+9.0.LTS',
    route: 'archive',
    bins: [
      'jar',
      'jarsigner',
      'java',
      'javac',
      'javadoc',
      'javap',
      'jcmd',
      'jconsole',
      'jdb',
      'jdeprscan',
      'jdeps',
      'jfr',
      'jhsdb',
      'jimage',
      'jinfo',
      'jlink',
      'jmap',
      'jmod',
      'jpackage',
      'jps',
      'jrunscript',
      'jshell',
      'jstack',
      'jstat',
      'jstatd',
      'jwebserver',
      'keytool',
      'rmiregistry',
      'serialver',
    ],
    options: {},
    env: { JAVA_HOME: 'java/temurin-21.0.13+9.0.LTS' },
    artifacts: artifacts('java'),
  },
  // cmdline-tools from the locked archive; the SDK components below are the
  // values `prepare:android` installed (compileSdk 37, NDK 29).
  {
    id: 'android-sdk',
    request: { track: 'latest' },
    version: '23.0',
    route: 'android-sdk',
    bins: [
      'android',
      'apkanalyzer',
      'avdmanager',
      'd8',
      'lint',
      'profgen',
      'r8',
      'resourceshrinker',
      'retrace',
      'screenshot2',
      'adb',
      'etc1tool',
      'fastboot',
      'hprof-conv',
      'make_f2fs',
      'make_f2fs_casefold',
      'mke2fs',
    ],
    options: {
      androidPackages: [
        'platforms/android-37.0',
        'build-tools/37.0.0',
        'ndk/29.0.13846066',
        'platform-tools',
      ],
    },
    env: {
      ANDROID_HOME: 'android-sdk/23.0',
      ANDROID_SDK_ROOT: 'android-sdk/23.0',
    },
    artifacts: artifacts('android-sdk'),
  },
  {
    id: 'cargo:cargo-ndk',
    request: { track: 'latest' },
    version: '4.1.2',
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: 'cargo-ndk',
    bins: [
      'cargo-ndk',
      'cargo-ndk-env',
      'cargo-ndk-runner',
      'cargo-ndk-test'
    ],
    options: {},
    env: {},
    artifacts: {},
  },
  // Installed through `uv tool install slopo==<locked>`: the uvx adapter (and
  // this provider) honor `==` pins, unlike the pipx adapter.
  {
    id: 'pipx:slopo',
    request: { track: 'latest' },
    version: '0.9.0',
    route: 'mpm-uvx',
    manager: 'uvx',
    packageId: 'slopo',
    bins: ['slopo'],
    options: {},
    env: {},
    artifacts: {},
  },
] as const;

//endregion Tools

//region Tool-managed bootstraps

/**
 The pinned `bin` CLI itself
 (marcosnils/bin, the `bin-sourced` installer).

 Downloaded as a standalone release binary and SHA-256 checked against the
 release's own `checksums.txt` values (verified 2026-10-26). It has no
 `bootstrap.requiresTool`: it is the first thing `toolchain:sync` places.
 */
export const BIN_BOOTSTRAP: ToolPlanEntry = {
  id: 'bin',
  request: { pinned: '0.29.4' },
  version: '0.29.4',
  route: 'archive',
  bins: ['bin'],
  options: {
    bootstrap: {
      distribution: 'standalone-binary',
    },
  },
  env: {},
  artifacts: {
    'linux-arm64': {
      url: 'https://github.com/marcosnils/bin/releases/download/v0.29.4/bin_0.29.4_linux_arm64',
      sha256: '706d5576d64f4ce66136f866cf51b92b2d0c2cc087e2cd17f32cd390b68e52cf',
    },
    'linux-x64': {
      url: 'https://github.com/marcosnils/bin/releases/download/v0.29.4/bin_0.29.4_linux_amd64',
      sha256: 'f7a179a813675bde2d8cf0ad3b4ae1d7bb1c6732daf623fb8deeba9f5f4782a7',
    },
    'macos-arm64': {
      url: 'https://github.com/marcosnils/bin/releases/download/v0.29.4/bin_0.29.4_darwin_arm64',
      sha256: '690ba4c3bbf8074a93da6b1ae86484bf43265f4b144d2b9de0ece7ae994414b9',
    },
    'macos-x64': {
      url: 'https://github.com/marcosnils/bin/releases/download/v0.29.4/bin_0.29.4_darwin_amd64',
      sha256: '4b71583e1227b911a0448be522cb09d359bb7160b24ff1f6a7dfac8595f741d4',
    },
    'windows-arm64': {
      url: 'https://github.com/marcosnils/bin/releases/download/v0.29.4/bin_0.29.4_windows_arm64.exe',
      sha256: '8521509a2991d07f07a57aa75b98505da1c12ed8145ff8b4c1412730de27b901',
    },
    'windows-x64': {
      url: 'https://github.com/marcosnils/bin/releases/download/v0.29.4/bin_0.29.4_windows_amd64.exe',
      sha256: 'd4606c4b310d222ae222286d98a305f2ac0678f80a91ffe695d4be9260689c76',
    },
  },
} as const;

/**
 The pinned `mpm` prerequisite itself
 (doc/decision/mpm-toolchain-adoption.md: pinned in the tool plan like any tool).

 The spec's route is the standalone release binary with a SHA-256 check. On
 2026-10-26 every GitHub route to those release assets returned 404
 (REST API, git transport, release download URLs), so no digest could be
 recorded or checked for them. The distribution below is the PyPI wheel
 `meta-package-manager` 8.2.0 publishes, whose SHA-256 PyPI's own metadata
 records; `toolchain:sync` downloads it, verifies the digest, and installs it
 with `uv tool install` (the `uv` tool is staged first for that reason).
 */
export const MPM_BOOTSTRAP: ToolPlanEntry = {
  id: 'mpm',
  request: { pinned: '8.2.0' },
  version: '8.2.0',
  route: 'mpm-uvx',
  manager: 'uvx',
  packageId: 'meta-package-manager',
  bins: [
    'mpm',
    'meta-package-manager'
  ],
  options: {
    bootstrap: {
      requiresTool: 'uv',
      distribution: 'wheel',
    },
  },
  env: {},
  artifacts: {
    'linux-arm64': {
      url: 'https://files.pythonhosted.org/packages/67/74/66f247a48fe22e95486ecf9a0e41aed03a643c2067c22e81e38d2d6ce7fd/meta_package_manager-8.2.0-py3-none-any.whl',
      sha256: '6b5cb8036f3f9dee5659de91786572a99ab332be80a82a8484bf86783400c292',
    },
    'linux-arm64-musl': {
      url: 'https://files.pythonhosted.org/packages/67/74/66f247a48fe22e95486ecf9a0e41aed03a643c2067c22e81e38d2d6ce7fd/meta_package_manager-8.2.0-py3-none-any.whl',
      sha256: '6b5cb8036f3f9dee5659de91786572a99ab332be80a82a8484bf86783400c292',
    },
    'linux-x64': {
      url: 'https://files.pythonhosted.org/packages/67/74/66f247a48fe22e95486ecf9a0e41aed03a643c2067c22e81e38d2d6ce7fd/meta_package_manager-8.2.0-py3-none-any.whl',
      sha256: '6b5cb8036f3f9dee5659de91786572a99ab332be80a82a8484bf86783400c292',
    },
    'linux-x64-musl': {
      url: 'https://files.pythonhosted.org/packages/67/74/66f247a48fe22e95486ecf9a0e41aed03a643c2067c22e81e38d2d6ce7fd/meta_package_manager-8.2.0-py3-none-any.whl',
      sha256: '6b5cb8036f3f9dee5659de91786572a99ab332be80a82a8484bf86783400c292',
    },
  },
} as const;

/**
 Every plan entry: the pinned `bin` and `mpm` bootstraps first
 (the sync algorithm's step 1), then the 59 `mise.no-env.toml` tools in
 declaration order.
 */
export const ALL_TOOLS: readonly ToolPlanEntry[] = [
  BIN_BOOTSTRAP,
  MPM_BOOTSTRAP,
  ...TOOLS,
];

//endregion Tool-managed bootstraps
