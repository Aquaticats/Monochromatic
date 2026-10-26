/**
 * Per-platform artifact provenance for the toolchain plan.
 *
 * Seeded from the `mise.lock` resolutions extracted to
 * `${HOME}/temp/agent/mise-lock-extract-2026-10-26.json` on 2026-10-26.
 * Linux-only rows are carried over as-is
 * (doc/decision/mise-lock-linux-only-stopgap.md);
 * macOS and Windows artifact rows stay absent until they are resolved.
 *
 * Regenerate by hand from a fresh `mise.lock` extract when resolutions change.
 */

/** One downloadable artifact: its URL and the SHA-256 the lock records. */
export type ArtifactRow = {
  readonly url: string;
  readonly sha256: string;
};

/** Artifact rows keyed by platform id (`linux-x64`, `linux-arm64-musl`, ...). */
export type ToolArtifacts = Readonly<Record<string, ArtifactRow>>;

/**
 * Artifact provenance keyed by tool id, then by platform id.
 *
 * Keys are sorted for deterministic rendering.
 */
export const ARTIFACTS: Readonly<Record<string, ToolArtifacts>> = {
  'age': {
    'linux-arm64': { url: 'https://github.com/FiloSottile/age/releases/download/v1.3.2/age-v1.3.2-linux-arm64.tar.gz', sha256: '6b8dc4333c53a5a57c9e5834e3a48f92605d7154014cd07269ff3327db5d37f4' },
    'linux-arm64-musl': { url: 'https://github.com/FiloSottile/age/releases/download/v1.3.2/age-v1.3.2-linux-arm64.tar.gz', sha256: '6b8dc4333c53a5a57c9e5834e3a48f92605d7154014cd07269ff3327db5d37f4' },
    'linux-x64': { url: 'https://github.com/FiloSottile/age/releases/download/v1.3.2/age-v1.3.2-linux-amd64.tar.gz', sha256: 'cbe24006683f8eb669266162894b9a522a1af52f2665fbc63a4bb032ed26ac10' },
    'linux-x64-musl': { url: 'https://github.com/FiloSottile/age/releases/download/v1.3.2/age-v1.3.2-linux-amd64.tar.gz', sha256: 'cbe24006683f8eb669266162894b9a522a1af52f2665fbc63a4bb032ed26ac10' },
  },
  'android-sdk': {
    'linux-arm64': { url: 'https://dl.google.com/android/repository/commandlinetools-linux-16111833_latest.zip', sha256: '0877a1d048fe4a24efe2eff536ca4223f7adeb58648bb81909d33c446918cfa8' },
    'linux-arm64-musl': { url: 'https://dl.google.com/android/repository/commandlinetools-linux-16111833_latest.zip', sha256: '0877a1d048fe4a24efe2eff536ca4223f7adeb58648bb81909d33c446918cfa8' },
    'linux-x64': { url: 'https://dl.google.com/android/repository/commandlinetools-linux-16111833_latest.zip', sha256: '0877a1d048fe4a24efe2eff536ca4223f7adeb58648bb81909d33c446918cfa8' },
    'linux-x64-musl': { url: 'https://dl.google.com/android/repository/commandlinetools-linux-16111833_latest.zip', sha256: '0877a1d048fe4a24efe2eff536ca4223f7adeb58648bb81909d33c446918cfa8' },
  },
  'betterleaks': {
    'linux-arm64': { url: 'https://github.com/betterleaks/betterleaks/releases/download/v1.9.0/betterleaks_1.9.0_linux_arm64.tar.gz', sha256: '1d39116e0a58dc94574715e2aa12a2dbd5062f193eee3fec011fef6ba06bd13b' },
    'linux-arm64-musl': { url: 'https://github.com/betterleaks/betterleaks/releases/download/v1.9.0/betterleaks_1.9.0_linux_arm64.tar.gz', sha256: '1d39116e0a58dc94574715e2aa12a2dbd5062f193eee3fec011fef6ba06bd13b' },
    'linux-x64': { url: 'https://github.com/betterleaks/betterleaks/releases/download/v1.9.0/betterleaks_1.9.0_linux_x64.tar.gz', sha256: 'f8b185a39ffcece2a1ca82bf3a4e7435cd81963ffd16b7a9128daf75f35f6de7' },
    'linux-x64-musl': { url: 'https://github.com/betterleaks/betterleaks/releases/download/v1.9.0/betterleaks_1.9.0_linux_x64.tar.gz', sha256: 'f8b185a39ffcece2a1ca82bf3a4e7435cd81963ffd16b7a9128daf75f35f6de7' },
  },
  'bun': {
    'linux-arm64': { url: 'https://github.com/oven-sh/bun/releases/download/bun-v1.3.14/bun-linux-aarch64.zip', sha256: 'a27ffb63a8310375836e0d6f668ae17fa8d8d18b88c37c821c65331973a19a3b' },
    'linux-arm64-musl': { url: 'https://github.com/oven-sh/bun/releases/download/bun-v1.3.14/bun-linux-aarch64-musl.zip', sha256: 'b98e0ad3625c5c00d1d5b5ff55605c7adddbfae151861e68ade57b2d3b8703bb' },
    'linux-x64': { url: 'https://github.com/oven-sh/bun/releases/download/bun-v1.3.14/bun-linux-x64.zip', sha256: '951ee2aee855f08595aeec6225226a298d3fea83a3dcd6465c09cbccdf7e848f' },
    'linux-x64-baseline': { url: 'https://github.com/oven-sh/bun/releases/download/bun-v1.3.14/bun-linux-x64-baseline.zip', sha256: 'a063908ae08b7852ca10939bbdc6ceed3ddabce8fb9402dce83d65d73b36e6c7' },
    'linux-x64-musl': { url: 'https://github.com/oven-sh/bun/releases/download/bun-v1.3.14/bun-linux-x64-musl.zip', sha256: '14bd9aedeebf1dba67e8def9531c89bc989ecfdf1de42e5bfcaf1b8cd9294719' },
    'linux-x64-musl-baseline': { url: 'https://github.com/oven-sh/bun/releases/download/bun-v1.3.14/bun-linux-x64-musl-baseline.zip', sha256: '56a7d6806cf155536c0178f0ea5fbd098e684fa509ebdb4fc0a7e19fb65382dc' },
  },
  'caddy': {
    'linux-arm64': { url: 'https://github.com/caddyserver/caddy/releases/download/v2.11.7/caddy_2.11.7_linux_arm64.tar.gz', sha256: 'd8fc6d179a5d283028a472a5618564f6ad8a86fed513e64f032b3b0b7cc45e42' },
    'linux-arm64-musl': { url: 'https://github.com/caddyserver/caddy/releases/download/v2.11.7/caddy_2.11.7_linux_arm64.tar.gz', sha256: 'd8fc6d179a5d283028a472a5618564f6ad8a86fed513e64f032b3b0b7cc45e42' },
    'linux-x64': { url: 'https://github.com/caddyserver/caddy/releases/download/v2.11.7/caddy_2.11.7_linux_amd64.tar.gz', sha256: '727b91701a392de6ebc5027509f548bf39979e5216340d0faed8fa5e69c84f8b' },
    'linux-x64-musl': { url: 'https://github.com/caddyserver/caddy/releases/download/v2.11.7/caddy_2.11.7_linux_amd64.tar.gz', sha256: '727b91701a392de6ebc5027509f548bf39979e5216340d0faed8fa5e69c84f8b' },
  },
  'cargo-binstall': {
    'linux-arm64': { url: 'https://github.com/cargo-bins/cargo-binstall/releases/download/v1.26.0/cargo-binstall-aarch64-unknown-linux-gnu.tgz', sha256: '255fd8d52e3a977ddb5e74fee58e7ef14b31c09d4eab5ad678e9967baedefced' },
    'linux-arm64-musl': { url: 'https://github.com/cargo-bins/cargo-binstall/releases/download/v1.26.0/cargo-binstall-aarch64-unknown-linux-musl.tgz', sha256: '4c7fdaa9fb6803ef66d282dfb671dc81f36fc03d469fb9a177b4aa4fade6b91b' },
    'linux-x64': { url: 'https://github.com/cargo-bins/cargo-binstall/releases/download/v1.26.0/cargo-binstall-x86_64-unknown-linux-gnu.tgz', sha256: 'e59aec44d811ab53b1bb4c5e0b57e433b017999c37e24ca577a3afbefd49dabd' },
    'linux-x64-musl': { url: 'https://github.com/cargo-bins/cargo-binstall/releases/download/v1.26.0/cargo-binstall-x86_64-unknown-linux-musl.tgz', sha256: 'dd7ff9451d8ba2e437a8d861889b630746f82f09d931df2d893b0c3f28656f87' },
  },
  'cmake': {
    'linux-arm64': { url: 'https://github.com/Kitware/CMake/releases/download/v4.4.4/cmake-4.4.4-linux-aarch64.tar.gz', sha256: 'a1b6cc63636a0e55c63257cf3315a8a5f129e42fade25db1afea4ff8ab06f25e' },
    'linux-arm64-musl': { url: 'https://github.com/Kitware/CMake/releases/download/v4.4.4/cmake-4.4.4-linux-aarch64.tar.gz', sha256: 'a1b6cc63636a0e55c63257cf3315a8a5f129e42fade25db1afea4ff8ab06f25e' },
    'linux-x64': { url: 'https://github.com/Kitware/CMake/releases/download/v4.4.4/cmake-4.4.4-linux-x86_64.tar.gz', sha256: 'e5bb807f7728cb60cd8b27ebc97a2edb469b68655f21e844a600c3575b76f5bb' },
    'linux-x64-musl': { url: 'https://github.com/Kitware/CMake/releases/download/v4.4.4/cmake-4.4.4-linux-x86_64.tar.gz', sha256: 'e5bb807f7728cb60cd8b27ebc97a2edb469b68655f21e844a600c3575b76f5bb' },
  },
  'conda:bubblewrap': {
    'linux-arm64': { url: 'https://conda.anaconda.org/conda-forge/linux-aarch64/bubblewrap-0.13.1-hfe774f5_0.conda', sha256: '4290eb42ec2a0353d6e66096565e3c35d181d1f7f1e98ad729308cbf18a643f8' },
    'linux-arm64-musl': { url: 'https://conda.anaconda.org/conda-forge/linux-aarch64/bubblewrap-0.13.1-hfe774f5_0.conda', sha256: '4290eb42ec2a0353d6e66096565e3c35d181d1f7f1e98ad729308cbf18a643f8' },
    'linux-x64': { url: 'https://conda.anaconda.org/conda-forge/linux-64/bubblewrap-0.13.1-haf0a511_0.conda', sha256: '32ceb8953a90ef961d6111a097ef23b5c9fda051c2e03e2735f40ced2fde7357' },
    'linux-x64-musl': { url: 'https://conda.anaconda.org/conda-forge/linux-64/bubblewrap-0.13.1-haf0a511_0.conda', sha256: '32ceb8953a90ef961d6111a097ef23b5c9fda051c2e03e2735f40ced2fde7357' },
  },
  'dprint': {
    'linux-arm64': { url: 'https://github.com/dprint/dprint/releases/download/0.64.0/dprint-aarch64-unknown-linux-gnu.zip', sha256: 'f9308f92d90ad8c5487730b5d8f90757e0e51070f075113d0e654f97f5d72a45' },
    'linux-arm64-musl': { url: 'https://github.com/dprint/dprint/releases/download/0.64.0/dprint-aarch64-unknown-linux-musl.zip', sha256: '6366cda97e81de1bbc54d91d461e857afa595feb568bd1bec94ab2122e13d969' },
    'linux-x64': { url: 'https://github.com/dprint/dprint/releases/download/0.64.0/dprint-x86_64-unknown-linux-gnu.zip', sha256: '5102aa940a060e48c33a5ceefce1273c701c6618503b7514323caada3b3649bb' },
    'linux-x64-musl': { url: 'https://github.com/dprint/dprint/releases/download/0.64.0/dprint-x86_64-unknown-linux-musl.zip', sha256: '21ac931303bb420bbdf3e0589506e5c32611c9fcff4ed2eb9802db657c499e9a' },
  },
  'github:Feel-ix-343/markdown-oxide': {
    'linux-arm64': { url: 'https://github.com/Feel-ix-343/markdown-oxide/releases/download/v0.25.12/markdown-oxide-v0.25.12-aarch64-unknown-linux-gnu.tar.gz', sha256: '6634b644ff0bcc1b8fb27cc518f82618547ec43fe19d515a890e8e8756950020' },
    'linux-arm64-musl': { url: 'https://github.com/Feel-ix-343/markdown-oxide/releases/download/v0.25.12/markdown-oxide-v0.25.12-aarch64-unknown-linux-gnu.tar.gz', sha256: '6634b644ff0bcc1b8fb27cc518f82618547ec43fe19d515a890e8e8756950020' },
    'linux-x64': { url: 'https://github.com/Feel-ix-343/markdown-oxide/releases/download/v0.25.12/markdown-oxide-v0.25.12-x86_64-unknown-linux-gnu.tar.gz', sha256: 'ad4248cf5d3f0e9d9f120b579501c45dad9c46bfcb4ddec36d2cb85d68a58828' },
    'linux-x64-musl': { url: 'https://github.com/Feel-ix-343/markdown-oxide/releases/download/v0.25.12/markdown-oxide-v0.25.12-x86_64-unknown-linux-gnu.tar.gz', sha256: 'ad4248cf5d3f0e9d9f120b579501c45dad9c46bfcb4ddec36d2cb85d68a58828' },
  },
  'github:docker/docker-language-server': {
    'linux-arm64': { url: 'https://github.com/docker/docker-language-server/releases/download/v0.20.1/docker-language-server-linux-arm64-v0.20.1', sha256: 'bd56c7815e0a22cfb708669f3d5e817de91d9b54039ff7e52867142a132ad8d7' },
    'linux-arm64-musl': { url: 'https://github.com/docker/docker-language-server/releases/download/v0.20.1/docker-language-server-linux-arm64-v0.20.1', sha256: 'bd56c7815e0a22cfb708669f3d5e817de91d9b54039ff7e52867142a132ad8d7' },
    'linux-x64': { url: 'https://github.com/docker/docker-language-server/releases/download/v0.20.1/docker-language-server-linux-amd64-v0.20.1', sha256: '01907aa5b0eae11e44cffea0a993d08aa155542a9af570295dd1dff39e67692a' },
    'linux-x64-musl': { url: 'https://github.com/docker/docker-language-server/releases/download/v0.20.1/docker-language-server-linux-amd64-v0.20.1', sha256: '01907aa5b0eae11e44cffea0a993d08aa155542a9af570295dd1dff39e67692a' },
  },
  'harper-cli': {
    'linux-arm64': { url: 'https://github.com/Automattic/harper/releases/download/v2.13.0/harper-cli-aarch64-unknown-linux-gnu.tar.gz', sha256: '37b56e29c16033a145ee02bb0437728be24a1e5af0d5a28f9f6e46e9648bd090' },
    'linux-arm64-musl': { url: 'https://github.com/Automattic/harper/releases/download/v2.13.0/harper-cli-aarch64-unknown-linux-gnu.tar.gz', sha256: '37b56e29c16033a145ee02bb0437728be24a1e5af0d5a28f9f6e46e9648bd090' },
    'linux-x64': { url: 'https://github.com/Automattic/harper/releases/download/v2.13.0/harper-cli-x86_64-unknown-linux-gnu.tar.gz', sha256: '3c2db5f53d374f7b3fe70200a6675f5ba43f01b3995974fb32a39e20237e914a' },
    'linux-x64-musl': { url: 'https://github.com/Automattic/harper/releases/download/v2.13.0/harper-cli-x86_64-unknown-linux-gnu.tar.gz', sha256: '3c2db5f53d374f7b3fe70200a6675f5ba43f01b3995974fb32a39e20237e914a' },
  },
  'hcloud': {
    'linux-arm64': { url: 'https://github.com/hetznercloud/cli/releases/download/v1.70.1/hcloud-linux-arm64.tar.gz', sha256: 'c9dbc00158d31131527e9ac5cb59b835fe8275ab7922f627d81b4e9bc2a2cebb' },
    'linux-arm64-musl': { url: 'https://github.com/hetznercloud/cli/releases/download/v1.70.1/hcloud-linux-arm64.tar.gz', sha256: 'c9dbc00158d31131527e9ac5cb59b835fe8275ab7922f627d81b4e9bc2a2cebb' },
    'linux-x64': { url: 'https://github.com/hetznercloud/cli/releases/download/v1.70.1/hcloud-linux-amd64.tar.gz', sha256: '85182e0d51f9a26abeefd79b37de8a7fb51215d54c59ee290c3c9141e8aa887e' },
    'linux-x64-musl': { url: 'https://github.com/hetznercloud/cli/releases/download/v1.70.1/hcloud-linux-amd64.tar.gz', sha256: '85182e0d51f9a26abeefd79b37de8a7fb51215d54c59ee290c3c9141e8aa887e' },
  },
  'hyperfine': {
    'linux-arm64': { url: 'https://github.com/sharkdp/hyperfine/releases/download/v2.0.0/hyperfine-v2.0.0-aarch64-unknown-linux-gnu.tar.gz', sha256: 'f8244d1e17f8da23ee6b3ae2cdc04a6a185a450f6b74e09bff12e498038fcbc9' },
    'linux-arm64-musl': { url: 'https://github.com/sharkdp/hyperfine/releases/download/v2.0.0/hyperfine-v2.0.0-aarch64-unknown-linux-gnu.tar.gz', sha256: 'f8244d1e17f8da23ee6b3ae2cdc04a6a185a450f6b74e09bff12e498038fcbc9' },
    'linux-x64': { url: 'https://github.com/sharkdp/hyperfine/releases/download/v2.0.0/hyperfine-v2.0.0-x86_64-unknown-linux-gnu.tar.gz', sha256: 'ae2beda2ac99c098427e4f755244552daf20e514422889cca6d44421e2d93c87' },
    'linux-x64-musl': { url: 'https://github.com/sharkdp/hyperfine/releases/download/v2.0.0/hyperfine-v2.0.0-x86_64-unknown-linux-musl.tar.gz', sha256: '38e59087bd65520930a13cf513d8a493b7043741e8ad62349a8a9929eb61bc5c' },
  },
  'java': {
    'linux-arm64': { url: 'https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.12%2B8/OpenJDK21U-jdk_aarch64_linux_hotspot_21.0.12_8.tar.gz', sha256: 'eba38e871b02d407897bfe017ea35352dfc1420ef6d2112425b0c67325ca509d' },
    'linux-arm64-musl': { url: 'https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.12%2B8/OpenJDK21U-jdk_aarch64_alpine-linux_hotspot_21.0.12_8.tar.gz', sha256: 'a32f7655011640ed8fb4347fffa2588cb7538de2f2e82027845816740560e2b1' },
    'linux-x64': { url: 'https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.12%2B8/OpenJDK21U-jdk_x64_linux_hotspot_21.0.12_8.tar.gz', sha256: 'e4446ff06a276155697597cc0f1b15da004ff083f4964a35271ecee567177370' },
    'linux-x64-musl': { url: 'https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.12%2B8/OpenJDK21U-jdk_x64_alpine-linux_hotspot_21.0.12_8.tar.gz', sha256: '073a48cb87f788a0cb283d81cc6d96cbc9d671d6a60a3ba6de62e66f0ea9dd47' },
  },
  'llama.cpp': {
    'linux-arm64': { url: 'https://github.com/ggml-org/llama.cpp/releases/download/b9829/llama-b9829-bin-ubuntu-arm64.tar.gz', sha256: '684f2f4c3e449cf65b560f866905ff38c27e0645615aa36a217e8d1d86df0e55' },
    'linux-arm64-musl': { url: 'https://github.com/ggml-org/llama.cpp/releases/download/b9829/llama-b9829-bin-ubuntu-arm64.tar.gz', sha256: '684f2f4c3e449cf65b560f866905ff38c27e0645615aa36a217e8d1d86df0e55' },
    'linux-x64': { url: 'https://github.com/ggml-org/llama.cpp/releases/download/b9829/llama-b9829-bin-ubuntu-x64.tar.gz', sha256: 'be52668d34106d1fccf4ddff63ad7be7c938c3cb3bdaf565eaeff01200810e5f' },
    'linux-x64-musl': { url: 'https://github.com/ggml-org/llama.cpp/releases/download/b9829/llama-b9829-bin-ubuntu-x64.tar.gz', sha256: 'be52668d34106d1fccf4ddff63ad7be7c938c3cb3bdaf565eaeff01200810e5f' },
  },
  'marksman': {
    'linux-arm64': { url: 'https://github.com/artempyanykh/marksman/releases/download/2026-02-08/marksman-linux-arm64', sha256: 'db8e124527f7f8048e3e6c91821b9c52ef173d92c01e47d221bf1337afd962fb' },
    'linux-arm64-musl': { url: 'https://github.com/artempyanykh/marksman/releases/download/2026-02-08/marksman-linux-arm64', sha256: 'db8e124527f7f8048e3e6c91821b9c52ef173d92c01e47d221bf1337afd962fb' },
    'linux-x64': { url: 'https://github.com/artempyanykh/marksman/releases/download/2026-02-08/marksman-linux-x64', sha256: 'be5098e8213219269c47fc0d916a66fa31ce0602ec967475c722260aabf26087' },
    'linux-x64-musl': { url: 'https://github.com/artempyanykh/marksman/releases/download/2026-02-08/marksman-linux-x64', sha256: 'be5098e8213219269c47fc0d916a66fa31ce0602ec967475c722260aabf26087' },
  },
  'node': {
    'linux-arm64': { url: 'https://nodejs.org/dist/v26.12.0/node-v26.12.0-linux-arm64.tar.gz', sha256: 'ec53c915c3fdd8786c5685abd25ae3fe9fba0692307f9811d7caaadea1bcaf03' },
    'linux-arm64-musl': { url: 'https://unofficial-builds.nodejs.org/download/release/v26.12.0/node-v26.12.0-linux-arm64-musl.tar.gz', sha256: '2e70b6d5adbae94546c4d7e01de7b64c50bf8d3dbfaad2bb0774942e9da902b8' },
    'linux-x64': { url: 'https://nodejs.org/dist/v26.12.0/node-v26.12.0-linux-x64.tar.gz', sha256: 'd9f88af72ad34f7ddf8e67289829f1698f54b6254f7d4f54b7453117ff1a3d2c' },
    'linux-x64-musl': { url: 'https://unofficial-builds.nodejs.org/download/release/v26.12.0/node-v26.12.0-linux-x64-musl.tar.gz', sha256: '48ba1cdf21e684f13f38b6295f43bfea951f1ffd008b4cf93361c966b33f6938' },
  },
  'opentofu': {
    'linux-arm64': { url: 'https://github.com/opentofu/opentofu/releases/download/v1.13.1/tofu_1.13.1_linux_arm64.tar.gz', sha256: '9c1ef375aa1852db0b2888aa921b640c71f8140d4682aa4fec99378a64fa7dc3' },
    'linux-arm64-musl': { url: 'https://github.com/opentofu/opentofu/releases/download/v1.13.1/tofu_1.13.1_linux_arm64.tar.gz', sha256: '9c1ef375aa1852db0b2888aa921b640c71f8140d4682aa4fec99378a64fa7dc3' },
    'linux-x64': { url: 'https://github.com/opentofu/opentofu/releases/download/v1.13.1/tofu_1.13.1_linux_amd64.tar.gz', sha256: '378ada19d4bc70c43732004e8159be771b23b9a5afdf059e5f8a2b3fa2c70a69' },
    'linux-x64-musl': { url: 'https://github.com/opentofu/opentofu/releases/download/v1.13.1/tofu_1.13.1_linux_amd64.tar.gz', sha256: '378ada19d4bc70c43732004e8159be771b23b9a5afdf059e5f8a2b3fa2c70a69' },
  },
  'pnpm': {
    'linux-arm64': { url: 'https://github.com/pnpm/pnpm/releases/download/v12.14.0/pnpm-linux-arm64.tar.gz', sha256: 'cfc780b9e690549e6633e1045fb1fb6ef84e390711e2099e6ccbae91c32456ea' },
    'linux-arm64-musl': { url: 'https://github.com/pnpm/pnpm/releases/download/v12.14.0/pnpm-linux-arm64-musl.tar.gz', sha256: '511a54809bdfd28270d30011d1697f582c8130d555b4fa0e2856b1a7534c98bf' },
    'linux-x64': { url: 'https://github.com/pnpm/pnpm/releases/download/v12.14.0/pnpm-linux-x64.tar.gz', sha256: '3910c3852136bed5bc1511b9137a45fd4357ed6b3970d9d08893641e9005867e' },
    'linux-x64-musl': { url: 'https://github.com/pnpm/pnpm/releases/download/v12.14.0/pnpm-linux-x64-musl.tar.gz', sha256: 'd7115e2f2791d79bf9e7e448609537595bb7787de1ef5292ce91ff3d1be2629e' },
  },
  'ripgrep': {
    'linux-arm64': { url: 'https://github.com/BurntSushi/ripgrep/releases/download/15.2.0/ripgrep-15.2.0-aarch64-unknown-linux-gnu.tar.gz', sha256: 'a740b91c82eaf9914cfedd353572f2791cbe0162c84101ee0951058f4dcbc90d' },
    'linux-arm64-musl': { url: 'https://github.com/BurntSushi/ripgrep/releases/download/15.2.0/ripgrep-15.2.0-aarch64-unknown-linux-gnu.tar.gz', sha256: 'a740b91c82eaf9914cfedd353572f2791cbe0162c84101ee0951058f4dcbc90d' },
    'linux-x64': { url: 'https://github.com/BurntSushi/ripgrep/releases/download/15.2.0/ripgrep-15.2.0-x86_64-unknown-linux-musl.tar.gz', sha256: '33e15bcf1624b25cdd2a55813a47a2f95dbe126268203e76aa6a585d1e7b149c' },
    'linux-x64-musl': { url: 'https://github.com/BurntSushi/ripgrep/releases/download/15.2.0/ripgrep-15.2.0-x86_64-unknown-linux-musl.tar.gz', sha256: '33e15bcf1624b25cdd2a55813a47a2f95dbe126268203e76aa6a585d1e7b149c' },
  },
  'rumdl': {
    'linux-arm64': { url: 'https://github.com/rvben/rumdl/releases/download/v0.3.0/rumdl-v0.3.0-aarch64-unknown-linux-gnu.tar.gz', sha256: 'b0ea55fdbabcc2b86e7ef594c3fd86cfee620844c2d8cea4a3135b9468d2845c' },
    'linux-arm64-musl': { url: 'https://github.com/rvben/rumdl/releases/download/v0.3.0/rumdl-v0.3.0-aarch64-unknown-linux-musl.tar.gz', sha256: '5a6274e69bf22164ee6adb47381c870f61ece4648a89fa8b178f0ed24857154a' },
    'linux-x64': { url: 'https://github.com/rvben/rumdl/releases/download/v0.3.0/rumdl-v0.3.0-x86_64-unknown-linux-gnu.tar.gz', sha256: '1d23c7f8c10d534e53c76b34a22f7dea7f5e5ebe493adbb7e25a401f88620c87' },
    'linux-x64-musl': { url: 'https://github.com/rvben/rumdl/releases/download/v0.3.0/rumdl-v0.3.0-x86_64-unknown-linux-musl.tar.gz', sha256: '6c90c398f04c11a09dcbd2da08bdaa6d53d9b14393e3cd32dbdf4da5620eec5c' },
  },
  'sops': {
    'linux-arm64': { url: 'https://github.com/getsops/sops/releases/download/v3.13.3/sops-v3.13.3.linux.arm64', sha256: '53b0abacd38ef1b12a66d6c100956691b9cefce018d91f81e73ddf7438b94d77' },
    'linux-arm64-musl': { url: 'https://github.com/getsops/sops/releases/download/v3.13.3/sops-v3.13.3.linux.arm64', sha256: '53b0abacd38ef1b12a66d6c100956691b9cefce018d91f81e73ddf7438b94d77' },
    'linux-x64': { url: 'https://github.com/getsops/sops/releases/download/v3.13.3/sops-v3.13.3.linux.amd64', sha256: 'e5bec3346a873ae91d871550f3e698c1aad962aff462a080e40f25fde17fef6b' },
    'linux-x64-musl': { url: 'https://github.com/getsops/sops/releases/download/v3.13.3/sops-v3.13.3.linux.amd64', sha256: 'e5bec3346a873ae91d871550f3e698c1aad962aff462a080e40f25fde17fef6b' },
  },
  'superhtml': {
    'linux-arm64': { url: 'https://github.com/kristoff-it/superhtml/releases/download/v0.7.0/aarch64-linux.tar.xz', sha256: '9fa2ed1ec830464c38929531693e7129ae6df3ccb1bb3f01fec3322ed5759fc5' },
    'linux-arm64-musl': { url: 'https://github.com/kristoff-it/superhtml/releases/download/v0.7.0/aarch64-linux.tar.xz', sha256: '9fa2ed1ec830464c38929531693e7129ae6df3ccb1bb3f01fec3322ed5759fc5' },
    'linux-x64': { url: 'https://github.com/kristoff-it/superhtml/releases/download/v0.7.0/x86_64-linux-musl.tar.xz', sha256: 'b75c6eeef539416096eac38729ee54e5f3b248f039cab4f57660f29a88742f68' },
    'linux-x64-musl': { url: 'https://github.com/kristoff-it/superhtml/releases/download/v0.7.0/x86_64-linux-musl.tar.xz', sha256: 'b75c6eeef539416096eac38729ee54e5f3b248f039cab4f57660f29a88742f68' },
  },
  'taplo': {
    'linux-arm64': { url: 'https://github.com/tamasfe/taplo/releases/download/0.10.0/taplo-linux-aarch64.gz', sha256: '033681d01eec8376c3fd38fa3703c79316f5e14bb013d859943b60a07bccdcc3' },
    'linux-arm64-musl': { url: 'https://github.com/tamasfe/taplo/releases/download/0.10.0/taplo-linux-aarch64.gz', sha256: '033681d01eec8376c3fd38fa3703c79316f5e14bb013d859943b60a07bccdcc3' },
    'linux-x64': { url: 'https://github.com/tamasfe/taplo/releases/download/0.10.0/taplo-linux-x86_64.gz', sha256: 'blake3:4871fab0e60275a1eb46e7190726e144f56c9a9527f59b0d1da5a042baead8e2' },
    'linux-x64-musl': { url: 'https://github.com/tamasfe/taplo/releases/download/0.10.0/taplo-linux-x86_64.gz', sha256: '8fe196b894ccf9072f98d4e1013a180306e17d244830b03986ee5e8eabeb6156' },
  },
  'tombi': {
    'linux-arm64': { url: 'https://github.com/tombi-toml/tombi/releases/download/v1.8.9/tombi-cli-1.8.9-aarch64-unknown-linux-musl.tar.gz', sha256: '4862f4fea963536bfb07678bed3d005735408e700701e979e6b68b9571084de6' },
    'linux-arm64-musl': { url: 'https://github.com/tombi-toml/tombi/releases/download/v1.8.9/tombi-cli-1.8.9-aarch64-unknown-linux-musl.tar.gz', sha256: '4862f4fea963536bfb07678bed3d005735408e700701e979e6b68b9571084de6' },
    'linux-x64': { url: 'https://github.com/tombi-toml/tombi/releases/download/v1.8.9/tombi-cli-1.8.9-x86_64-unknown-linux-musl.tar.gz', sha256: '26d7207a4de89e5d846d5a9e5d3bc9c407f8b035f06adfdb774564bd7e440c15' },
    'linux-x64-musl': { url: 'https://github.com/tombi-toml/tombi/releases/download/v1.8.9/tombi-cli-1.8.9-x86_64-unknown-linux-musl.tar.gz', sha256: '26d7207a4de89e5d846d5a9e5d3bc9c407f8b035f06adfdb774564bd7e440c15' },
  },
  'uv': {
    'linux-arm64': { url: 'https://github.com/astral-sh/uv/releases/download/0.13.4/uv-aarch64-unknown-linux-gnu.tar.gz', sha256: '1ae72b4718729e2ccc4f1ae2c6d1e531ca8467800e0546e9130b3886023a60ab' },
    'linux-arm64-musl': { url: 'https://github.com/astral-sh/uv/releases/download/0.13.4/uv-aarch64-unknown-linux-musl.tar.gz', sha256: '253a17cadcb659ae77a0fe3b9a2f1c14297528d480c421899e61e64d64f24e8f' },
    'linux-x64': { url: 'https://github.com/astral-sh/uv/releases/download/0.13.4/uv-x86_64-unknown-linux-gnu.tar.gz', sha256: '2b428396f12572d4429cf210ca234dd599a33b3af0958a4d5ed569d63e3f344f' },
    'linux-x64-musl': { url: 'https://github.com/astral-sh/uv/releases/download/0.13.4/uv-x86_64-unknown-linux-musl.tar.gz', sha256: '28d4385f9f9d1d1c7654d85b3855e0e2615b2f6cc13136a2edff858bbdc16ff1' },
  },
  'watchexec': {
    'linux-arm64': { url: 'https://github.com/watchexec/watchexec/releases/download/v2.8.0/watchexec-2.8.0-aarch64-unknown-linux-gnu.tar.xz', sha256: 'aaa5eb7e62e9ce99cad7f14260265aaa9df8884e195c0661ab011825a6d61634' },
    'linux-arm64-musl': { url: 'https://github.com/watchexec/watchexec/releases/download/v2.8.0/watchexec-2.8.0-aarch64-unknown-linux-musl.tar.xz', sha256: '3961878b21e3c1729279322f4007dd5bbf0032110fbcb5e7b983946a9ceb3bf6' },
    'linux-x64': { url: 'https://github.com/watchexec/watchexec/releases/download/v2.8.0/watchexec-2.8.0-x86_64-unknown-linux-gnu.tar.xz', sha256: '21839e9840ef592b4d571885ccbd692f54ae5bd4231f3caf29fcf31fa7723afe' },
    'linux-x64-musl': { url: 'https://github.com/watchexec/watchexec/releases/download/v2.8.0/watchexec-2.8.0-x86_64-unknown-linux-musl.tar.xz', sha256: '35ea667cbe4bc96c23ba2a1f46c94c46bc385b67165e3bf621c0e7e4959aa99b' },
  },
  'zig': {
    'linux-arm64': { url: 'https://ziglang.org/download/0.15.2/zig-aarch64-linux-0.15.2.tar.xz', sha256: '958ed7d1e00d0ea76590d27666efbf7a932281b3d7ba0c6b01b0ff26498f667f' },
    'linux-arm64-musl': { url: 'https://ziglang.org/download/0.15.2/zig-aarch64-linux-0.15.2.tar.xz', sha256: '958ed7d1e00d0ea76590d27666efbf7a932281b3d7ba0c6b01b0ff26498f667f' },
    'linux-x64': { url: 'https://ziglang.org/download/0.15.2/zig-x86_64-linux-0.15.2.tar.xz', sha256: '02aa270f183da276e5b5920b1dac44a63f1a49e55050ebde3aecc9eb82f93239' },
    'linux-x64-musl': { url: 'https://ziglang.org/download/0.15.2/zig-x86_64-linux-0.15.2.tar.xz', sha256: '02aa270f183da276e5b5920b1dac44a63f1a49e55050ebde3aecc9eb82f93239' },
  },
  'zizmor': {
    'linux-arm64': { url: 'https://github.com/zizmorcore/zizmor/releases/download/v1.31.1/zizmor-aarch64-unknown-linux-gnu.tar.gz', sha256: '37eebbd5762a44f262d94875431b14a6d2402991e90a685113ea97b8b41be833' },
    'linux-arm64-musl': { url: 'https://github.com/zizmorcore/zizmor/releases/download/v1.31.1/zizmor-aarch64-unknown-linux-gnu.tar.gz', sha256: '37eebbd5762a44f262d94875431b14a6d2402991e90a685113ea97b8b41be833' },
    'linux-x64': { url: 'https://github.com/zizmorcore/zizmor/releases/download/v1.31.1/zizmor-x86_64-unknown-linux-gnu.tar.gz', sha256: 'd185f726bdaf92df78273217dfdd24ebcd6b0ef48cf3ce776ff5762b6e138c99' },
    'linux-x64-musl': { url: 'https://github.com/zizmorcore/zizmor/releases/download/v1.31.1/zizmor-x86_64-unknown-linux-gnu.tar.gz', sha256: 'd185f726bdaf92df78273217dfdd24ebcd6b0ef48cf3ce776ff5762b6e138c99' },
  },
};

/**
 Looks one tool's artifact table up by tool id.
 
 @param id - Stable tool id
 
 @returns Artifact table recorded for the tool
 
 @throws {Error} when the id has no artifact table
 */
export function artifacts(id: string,): ToolArtifacts {
  const table = ARTIFACTS[id];
  if (table === undefined)
    throw new Error(`no artifact provenance recorded for ${JSON.stringify(id,)}`);
  return table;
}
