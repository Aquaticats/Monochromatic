# @aliou Pi extensions pin Pi 0.74.0 peers and npm blocks Pi 0.75.3 installs

Note 2026-11-09T16:56:27-05:00:
`.pi/npm/` is not in the tree;
its files were deleted on 2026-06-15 in `32b6cd845f`.

## Symptom

Running npm resolution for a Pi 0.75.3 extension set fails with `ERESOLVE` while npm resolves
`@aliou/pi-linkup@0.11.0`:

```text
While resolving: @aliou/pi-linkup@0.11.0
Found: @earendil-works/pi-coding-agent@0.75.3
Could not resolve dependency:
peerOptional @earendil-works/pi-coding-agent@"0.74.0" from @aliou/pi-linkup@0.11.0
Conflicting peer dependency: @earendil-works/pi-coding-agent@0.74.0
```

The full npm report at `/var/home/user/.npm/_logs/2026-05-20T10_44_05_546Z-eresolve-report.txt`
also shows the surrounding extension set:

```text
peerOptional @earendil-works/pi-coding-agent@"^0.75.3" from @aliou/pi-processes@0.9.1
peer @earendil-works/pi-coding-agent@"*" from @diegopetrucci/pi-openai-fast@0.1.1
peerOptional @earendil-works/pi-coding-agent@">=0.74.0 <1" from @aliou/pi-utils-ui@0.4.1
@aliou/pi-synthetic@0.17.2
```

## Root cause

npm 11 enforces peer dependency compatibility during tree construction.
 The npm 11 `package.json` docs at
<https://docs.npmjs.com/cli/v11/configuring-npm/package-json#peerdependencies> say peer dependencies are installed
by default as of npm 7,
 and that conflicting plugin requirements can make resolution fail:

```text
As of npm v7, peerDependencies are installed by default.
Trying to install another plugin with a conflicting requirement may cause an error if the tree cannot be resolved
correctly.
```

The same docs at
<https://docs.npmjs.com/cli/v11/configuring-npm/package-json#peerdependenciesmeta> state that optional peer metadata
prevents automatic installation of absent peers:

```text
Specifically, it allows peer dependencies to be marked as optional.
Npm will not automatically install optional peer dependencies.
```

That optional marker does not make an incompatible peer version acceptable once the peer is present in the tree.

`@aliou/pi-linkup@0.11.0` pins three Pi host peers to exactly `0.74.0`.
 Source clone:
`/tmp/pi-linkup-source-20260520`,
 commit `c41348b0ba833630a3480aa864f58a730da978af`.

`/tmp/pi-linkup-source-20260520/package.json:39-42`:

```json
"peerDependencies": {
  "@earendil-works/pi-ai": "0.74.0",
  "@earendil-works/pi-coding-agent": "0.74.0",
  "@earendil-works/pi-tui": "0.74.0"
},
```

The same package marks those peers optional,
 but the exact version range still applies when Pi packages are already
present.

`/tmp/pi-linkup-source-20260520/package.json:71-80`:

```json
"peerDependenciesMeta": {
  "@earendil-works/pi-coding-agent": {
    "optional": true
  },
  "@earendil-works/pi-ai": {
    "optional": true
  },
  "@earendil-works/pi-tui": {
    "optional": true
  }
},
```

`@aliou/pi-synthetic@0.17.2` has the same Pi 0.74.0 pin for `pi-coding-agent` and `pi-tui`.
 Source clone:
`/tmp/pi-synthetic-source-20260520`,
 commit `7ebe180f23f441bf2e5338623a6192157ddc47e0`.

`/tmp/pi-synthetic-source-20260520/package.json:34-36`:

```json
"peerDependencies": {
  "@earendil-works/pi-coding-agent": "0.74.0",
  "@earendil-works/pi-tui": "0.74.0"
},
```

`@aliou/pi-processes@0.9.1` requires the opposite side of the conflict:
 Pi `^0.75.3`.
 Source clone:
`/tmp/pi-processes-source-20260520`,
 commit `0958ef30d7a1c7872d0fccb9b2bc3acecdff5099`.

`/tmp/pi-processes-source-20260520/package.json:41-43`:

```json
"peerDependencies": {
  "@earendil-works/pi-coding-agent": "^0.75.3",
  "@earendil-works/pi-tui": "^0.75.3"
},
```

`@aliou/pi-utils-ui@0.4.1` is not the blocking peer.
 Its host ranges already span Pi 0.74.0 to below 1.0.0.
Source clone:
 `/tmp/pi-utils-ui-source-20260520`,
 commit `d8b4f166fe167e852b26096ad693e9c7f7206a08`.

`/tmp/pi-utils-ui-source-20260520/package.json:34-36`:

```json
"peerDependencies": {
  "@earendil-works/pi-coding-agent": ">=0.74.0 <1",
  "@earendil-works/pi-tui": ">=0.74.0 <1"
},
```

## Verification

Version context:

- Node:
   `v26.1.0`
- npm:
   `11.13.0`
- `@aliou/pi-linkup@0.11.0` npm integrity:
  `sha512-ITYuZOzTcEimMUQ3KuBrW90fblfrq3pciz9Gv21miVOz2MmQih8XfD3dkPf3QjsOdUXZsxxE9KHGQyCOp591OA==`
- `@aliou/pi-synthetic@0.17.2` npm integrity:
  `sha512-TorNGHUTFzoUlprl5MBoQ7EdKkvI3pkFy4q4mal9quqB0/bzlOKjbAYYAAIZNblv1k6dmbbK/gxv5p3bC7p8vQ==`
- `@aliou/pi-processes@0.9.1` npm integrity:
  `sha512-PA/T3DeOs3rTSfrSMNgcdDxwNnqS6ycEn81zFpS2gUpisgsTpEp3FAR6p2LaZDqgnGxdacT+hUazS6q9OQlYcA==`
- `@aliou/pi-utils-ui@0.4.1` npm integrity:
  `sha512-1oJraVjjlZD8UM41472MF1O8a41/4OvAxdH/HlQsahZH/gBkhahGw/EjlcVqXTWnCQfo+4X6PksRz63erNsPwQ==`

### Verification scope

The harnesses below verify package-manager resolver behaviour only.
 They use lockfile-only installs with ignored scripts
to avoid running third-party package scripts.
 They do not prove full runtime compatibility of every extension under
Pi 0.75.3.

The npm `ERESOLVE` report came from npm,
 not from the Monochromatic pnpm workspace.
 Pi package docs say npm sources
install under `~/.pi/agent/npm/` or `.pi/npm/`,
 and `settings.md` says `npmCommand` defaults to npm unless configured.
Pi 0.75.3 source confirms the default command is `npm`:
`node_modules/.pnpm/@earendil-works+pi-coding-agent@0.75.3_ws@8.20.0_zod@4.3.6/node_modules/@earendil-works/pi-coding-agent/dist/core/package-manager.js:1351-1357`.

```js
getNpmCommand() {
    const configuredCommand = this.settingsManager.getNpmCommand();
    if (!configuredCommand || configuredCommand.length === 0) {
        return { command: "npm", args: [] };
    }
```

The same source builds package install arguments from the configured package-manager name,
 using npm by default and
pnpm only when `npmCommand` resolves to `pnpm`:
`node_modules/.pnpm/@earendil-works+pi-coding-agent@0.75.3_ws@8.20.0_zod@4.3.6/node_modules/@earendil-works/pi-coding-agent/dist/core/package-manager.js:1385-1393`.

```js
getNpmInstallArgs(specs, installRoot) {
    const packageManagerName = this.getPackageManagerName();
    if (packageManagerName === "bun") {
        return ["install", ...specs, "--cwd", installRoot];
    }
    if (packageManagerName === "pnpm") {
        return ["install", ...specs, "--prefix", installRoot, "--config.strict-dep-builds=false"];
    }
    return ["install", ...specs, "--prefix", installRoot];
}
```

Apply npm `overrides` only to the Pi-managed npm package root that produced the npm error.
 For the Monochromatic pnpm
workspace,
 use `pnpm-workspace.yaml` guidance below instead.

### Failing catalogue

#### Published Pi 0.75.3 graph plus linkup fails at linkup

Harness:

```sh
mkdir -p /tmp/pi-linkup-eresolve-repro-20260520
cd /tmp/pi-linkup-eresolve-repro-20260520
npm init -y >/dev/null
npm pkg set type=module \
  dependencies.'@earendil-works/pi-coding-agent'=0.75.3 \
  dependencies.'@aliou/pi-processes'=0.9.1 \
  dependencies.'@diegopetrucci/pi-openai-fast'=0.1.1 \
  dependencies.'@aliou/pi-linkup'=0.11.0 >/dev/null
npm install --package-lock-only --ignore-scripts
```

Observed output excerpt:

```text
npm error While resolving: @aliou/pi-linkup@0.11.0
npm error Found: @earendil-works/pi-coding-agent@0.75.3
npm error Could not resolve dependency:
npm error peerOptional @earendil-works/pi-coding-agent@"0.74.0" from @aliou/pi-linkup@0.11.0
```

#### Downgrading Pi to 0.74.0 fails at pi-processes

Harness change:

```sh
cd /tmp/pi-linkup-eresolve-repro-20260520
npm pkg set dependencies.'@earendil-works/pi-coding-agent'=0.74.0 >/dev/null
rm -f package-lock.json
npm install --package-lock-only --ignore-scripts
```

Observed output excerpt:

```text
npm error While resolving: @aliou/pi-processes@0.9.1
npm error Found: @earendil-works/pi-coding-agent@0.74.0
npm error Could not resolve dependency:
npm error peerOptional @earendil-works/pi-coding-agent@"^0.75.3" from @aliou/pi-processes@0.9.1
```

#### Relaxing only linkup exposes the next failure at synthetic

Harness:

```sh
mkdir -p /tmp/pi-linkup-relaxed-full-20260520
cd /tmp/pi-linkup-relaxed-full-20260520
npm init -y >/dev/null
npm pkg set type=module \
  dependencies.'@earendil-works/pi-coding-agent'=0.75.3 \
  dependencies.'@aliou/pi-processes'=0.9.1 \
  dependencies.'@diegopetrucci/pi-openai-fast'=0.1.1 \
  dependencies.'@aliou/pi-linkup'='file:../pi-linkup-relaxed-metadata-20260520' \
  dependencies.'@aliou/pi-synthetic'=0.17.2 >/dev/null
npm install --package-lock-only --ignore-scripts
```

Observed output excerpt:

```text
npm error While resolving: @aliou/pi-synthetic@0.17.2
npm error Found: @earendil-works/pi-coding-agent@0.75.3
npm error Could not resolve dependency:
npm error peerOptional @earendil-works/pi-coding-agent@"0.74.0" from @aliou/pi-synthetic@0.17.2
```

### Passing catalogue

#### Root overrides unblock the published packages

Harness:

```sh
cd /tmp/pi-linkup-eresolve-full-20260520
npm pkg set \
  overrides.'@earendil-works/pi-coding-agent'=0.75.3 \
  overrides.'@earendil-works/pi-tui'=0.75.3 \
  overrides.'@earendil-works/pi-ai'=0.75.3 >/dev/null
rm -f package-lock.json
npm install --package-lock-only --ignore-scripts
```

Observed output:

```text
up to date, audited 139 packages in 836ms

19 packages are looking for funding
  run `npm fund` for details

found 0 vulnerabilities
```

#### Relaxing both Aliou packages unblocks the graph without root overrides

Harness:

```sh
mkdir -p /tmp/pi-aliou-relaxed-full-20260520
cd /tmp/pi-aliou-relaxed-full-20260520
npm init -y >/dev/null
npm pkg set type=module \
  dependencies.'@earendil-works/pi-coding-agent'=0.75.3 \
  dependencies.'@aliou/pi-processes'=0.9.1 \
  dependencies.'@diegopetrucci/pi-openai-fast'=0.1.1 \
  dependencies.'@aliou/pi-linkup'='file:../pi-linkup-relaxed-metadata-20260520' \
  dependencies.'@aliou/pi-synthetic'='file:../pi-synthetic-relaxed-metadata-20260520' >/dev/null
npm install --package-lock-only --ignore-scripts
```

Observed output:

```text
up to date, audited 141 packages in 6s

19 packages are looking for funding
  run `npm fund` for details

found 0 vulnerabilities
```

#### `--legacy-peer-deps` unblocks the published graph by disabling strict peer resolution

Harness:

```sh
cd /tmp/pi-linkup-eresolve-repro-20260520
npm pkg set dependencies.'@earendil-works/pi-coding-agent'=0.75.3 >/dev/null
rm -f package-lock.json
npm install --package-lock-only --ignore-scripts --legacy-peer-deps
```

Observed output:

```text
up to date, audited 138 packages in 1s

19 packages are looking for funding
  run `npm fund` for details

found 0 vulnerabilities
```

#### pnpm workspace install warns, and strict peer checks fail without pnpm rules

Harness:

```sh
mkdir -p /tmp/pi-peer-pnpm-repro-20260520
cd /tmp/pi-peer-pnpm-repro-20260520
cat > package.json <<'JSON'
{
  "name": "pi-peer-pnpm-repro",
  "private": true,
  "type": "module",
  "dependencies": {
    "@earendil-works/pi-coding-agent": "0.75.3",
    "@aliou/pi-processes": "0.9.1",
    "@diegopetrucci/pi-openai-fast": "0.1.1",
    "@aliou/pi-linkup": "0.11.0",
    "@aliou/pi-synthetic": "0.17.2"
  }
}
JSON
cat > pnpm-workspace.yaml <<'YAML'
packages:
- .
autoInstallPeers: false
nodeLinker: isolated
YAML
pnpm install --lockfile-only --ignore-scripts
pnpm peers check
```

Observed output excerpts:

```text
[WARN] Issues with peer dependencies found. Run "pnpm peers check" to list them.
```

```text
Issues with peer dependencies found

✕ unmet peer @earendil-works/pi-coding-agent
  Installed: 0.75.3
  Wanted:
    0.74.0:
      @aliou/pi-linkup@0.11.0
      @aliou/pi-synthetic@0.17.2
```

With `strictPeerDependencies: true`,
 `pnpm install --lockfile-only --ignore-scripts` also fails with
`ERR_PNPM_PEER_DEP_ISSUES`.

#### pnpm workspace rules unblock strict peer checks

Harness:

```sh
cd /tmp/pi-peer-pnpm-repro-20260520
cat > pnpm-workspace.yaml <<'YAML'
packages:
- .
autoInstallPeers: false
nodeLinker: isolated
strictPeerDependencies: true
peerDependencyRules:
  allowedVersions:
    "@aliou/pi-linkup>@earendil-works/pi-ai": "0.75.3"
    "@aliou/pi-linkup>@earendil-works/pi-coding-agent": "0.75.3"
    "@aliou/pi-linkup>@earendil-works/pi-tui": "0.75.3"
    "@aliou/pi-synthetic>@earendil-works/pi-coding-agent": "0.75.3"
    "@aliou/pi-synthetic>@earendil-works/pi-tui": "0.75.3"
YAML
rm -f pnpm-lock.yaml
pnpm install --lockfile-only --ignore-scripts
pnpm peers check
```

Observed output excerpt:

```text
Done in 561ms using pnpm v11.1.3
No peer dependency issues found
```

pnpm root `overrides` also work when scoped to the problematic package peers.
 pnpm docs at
<https://pnpm.io/settings#overriding-peer-dependencies> state that overrides apply to `peerDependencies`.

## Verified workarounds

### Add root pnpm peer rules for the Monochromatic workspace

If this graph is represented in the Monochromatic pnpm workspace,
 patch `pnpm-workspace.yaml`,
 not npm `package.json`.
The least tree-changing pnpm workaround is `peerDependencyRules.allowedVersions`:

```yaml
peerDependencyRules:
  allowedVersions:
    '@aliou/pi-linkup>@earendil-works/pi-ai': '0.75.3'
    '@aliou/pi-linkup>@earendil-works/pi-coding-agent': '0.75.3'
    '@aliou/pi-linkup>@earendil-works/pi-tui': '0.75.3'
    '@aliou/pi-synthetic>@earendil-works/pi-coding-agent': '0.75.3'
    '@aliou/pi-synthetic>@earendil-works/pi-tui': '0.75.3'
```

This passed `pnpm install --lockfile-only --ignore-scripts --strict-peer-dependencies` and `pnpm peers check` in the
reproduction harness.
 Tradeoff:
 it asserts local compatibility;
 it does not verify runtime compatibility.

Root pnpm `overrides` also pass and actually rewrite the peer ranges during pnpm resolution:

```yaml
overrides:
  '@aliou/pi-linkup>@earendil-works/pi-ai': '0.75.3'
  '@aliou/pi-linkup>@earendil-works/pi-coding-agent': '0.75.3'
  '@aliou/pi-linkup>@earendil-works/pi-tui': '0.75.3'
  '@aliou/pi-synthetic>@earendil-works/pi-coding-agent': '0.75.3'
  '@aliou/pi-synthetic>@earendil-works/pi-tui': '0.75.3'
```

### Add root npm overrides for Pi's default package installer

Patch the root `package.json` that Pi's default npm installer uses for the extension install.
 This is usually
`/var/home/user/.pi/agent/npm/package.json` for user-scoped packages or `.pi/npm/package.json` for project-scoped
packages,
 not the Monochromatic pnpm workspace root.

```json
{
  "overrides": {
    "@earendil-works/pi-ai": "0.75.3",
    "@earendil-works/pi-coding-agent": "0.75.3",
    "@earendil-works/pi-tui": "0.75.3"
  }
}
```

Then rerun `npm install`.
 This keeps strict peer resolution enabled and makes npm resolve all Pi host peers to the
same 0.75.3 family.
 Tradeoff:
 the root project now overrides peer compatibility metadata across the tree,
 so remove
these overrides after upstream publishes corrected peer ranges.

### Use `--legacy-peer-deps` only as a temporary install unblocker

`npm install --legacy-peer-deps` resolves the graph in the reproduction harness.
 Tradeoff:
 it disables npm's peer
resolution checks for the install,
 so it can hide unrelated peer incompatibilities.

### Patch upstream package metadata

For `@aliou/pi-linkup`,
 change peer ranges from exact `0.74.0` to a range that includes the Pi 0.75.3 host packages.
For `@aliou/pi-synthetic`,
 make the same change for `pi-coding-agent` and `pi-tui`.
 The prototype below used
`0.74.0 || ^0.75.3`,
 which preserves the published 0.74.0 support claim and adds the observed Pi 0.75.3 host family.
Tradeoff:
 this still proves npm resolver compatibility only.
 Full runtime compatibility needs a Pi session that loads
and exercises the changed extensions.

## What does not work

- Downgrading only `@earendil-works/pi-coding-agent` to `0.74.0` makes `@aliou/pi-processes@0.9.1` fail because
  `pi-processes` requires `^0.75.3`.
- Fixing only `@aliou/pi-linkup` is incomplete when `@aliou/pi-synthetic@0.17.2` is also installed.
   npm then fails on
  `@aliou/pi-synthetic@0.17.2` for the same exact Pi 0.74.0 peer pin.
- Depending on `@aliou/pi-utils-ui@0.4.1` is not enough to relax the tree.
   Its own peer range already accepts Pi
  0.75.3,
   but npm still enforces the stricter peer ranges declared by direct consumers.

## Draft upstream issue

### Five-constraint upstream-filing check

1. **Is it really upstream's fault?
   ** Yes.
    The failing constraints are exact peer ranges in the published
   `@aliou/pi-linkup@0.11.0` and `@aliou/pi-synthetic@0.17.2` manifests.
2. **Can upstream fix it?
   ** Yes.
    The minimal fix is a package metadata change in each package's `peerDependencies`.
3. **Are they supporting this use case?
   ** Yes.
    These packages are Pi extensions,
    declare Pi host packages as peers,
   and ship `pi.extensions` entries in `package.json`.
4. **Will they likely fix it?
   ** Yes.
    Recent merged release PRs show active maintenance:
   `aliou/pi-linkup#30` released `0.11.0` on 2026-05-08,
    and `aliou/pi-synthetic#56` released `0.17.2` on
   2026-05-20.
    The sibling `@aliou/pi-processes@0.9.1` has already moved to `^0.75.3` peers.
5. **Have we prototyped a minimal fix compatible with their architecture?
   ** Yes.
    The prototype changed only
   `package.json` peer ranges in fresh clones and verified npm resolution with `--ignore-scripts`.

Prototype clone details:

```text
prototype_root=/tmp/pi-peer-prototype-9qgzHC8H8p
pi-linkup origin=https://github.com/aliou/pi-linkup.git
pi-linkup head=c41348b0ba833630a3480aa864f58a730da978af
pi-synthetic origin=https://github.com/aliou/pi-synthetic.git
pi-synthetic head=7ebe180f23f441bf2e5338623a6192157ddc47e0
```

Prototype verification command:

```sh
cd /tmp/pi-peer-prototype-9qgzHC8H8p/repro
npm install --package-lock-only --ignore-scripts
```

Prototype verification output:

```text
up to date, audited 141 packages in 3s

19 packages are looking for funding
  run `npm fund` for details

found 0 vulnerabilities
```

Prototype diff:

```diff
diff --git a/package.json b/package.json
index a88c508..a5c0ccf 100644
--- a/package.json
+++ b/package.json
@@ -37,9 +37,9 @@
     "@aliou/pi-utils-ui": "^0.4.0"
   },
   "peerDependencies": {
-    "@earendil-works/pi-ai": "0.74.0",
-    "@earendil-works/pi-coding-agent": "0.74.0",
-    "@earendil-works/pi-tui": "0.74.0"
+    "@earendil-works/pi-ai": "0.74.0 || ^0.75.3",
+    "@earendil-works/pi-coding-agent": "0.74.0 || ^0.75.3",
+    "@earendil-works/pi-tui": "0.74.0 || ^0.75.3"
   },
   "scripts": {
     "typecheck": "tsc --noEmit",
diff --git a/package.json b/package.json
index cfae4fa..8c23e2f 100644
--- a/package.json
+++ b/package.json
@@ -32,8 +32,8 @@
     "README.md"
   ],
   "peerDependencies": {
-    "@earendil-works/pi-coding-agent": "0.74.0",
-    "@earendil-works/pi-tui": "0.74.0"
+    "@earendil-works/pi-coding-agent": "0.74.0 || ^0.75.3",
+    "@earendil-works/pi-tui": "0.74.0 || ^0.75.3"
   },
   "scripts": {
     "typecheck": "tsc --noEmit",
```

### Draft for `aliou/pi-linkup`, do not file as-is without checking for a newer release

````md
Title: @aliou/pi-linkup 0.11.0 exact Pi 0.74.0 peer blocks npm installs with Pi 0.75.3

Labels: bug

Description:

`@aliou/pi-linkup@0.11.0` declares exact Pi 0.74.0 host peers:

```json
"peerDependencies": {
  "@earendil-works/pi-ai": "0.74.0",
  "@earendil-works/pi-coding-agent": "0.74.0",
  "@earendil-works/pi-tui": "0.74.0"
}
```

When installed beside `@earendil-works/pi-coding-agent@0.75.3` and `@aliou/pi-processes@0.9.1`, npm 11 fails with:

```text
While resolving: @aliou/pi-linkup@0.11.0
Could not resolve dependency:
peerOptional @earendil-works/pi-coding-agent@"0.74.0" from @aliou/pi-linkup@0.11.0
```

Minimal reproduction:

```sh
mkdir -p /tmp/pi-linkup-eresolve-repro
cd /tmp/pi-linkup-eresolve-repro
npm init -y >/dev/null
npm pkg set type=module \
  dependencies.'@earendil-works/pi-coding-agent'=0.75.3 \
  dependencies.'@aliou/pi-processes'=0.9.1 \
  dependencies.'@diegopetrucci/pi-openai-fast'=0.1.1 \
  dependencies.'@aliou/pi-linkup'=0.11.0 >/dev/null
npm install --package-lock-only --ignore-scripts
```

Suggested fix:

Relax the Pi host peer ranges to include the supported Pi 0.75.3 host family, for example:

```json
"peerDependencies": {
  "@earendil-works/pi-ai": "0.74.0 || ^0.75.3",
  "@earendil-works/pi-coding-agent": "0.74.0 || ^0.75.3",
  "@earendil-works/pi-tui": "0.74.0 || ^0.75.3"
}
```

A local prototype with that metadata change lets npm resolve the Pi 0.75.3 graph successfully with strict peer
resolution still enabled. It does not prove full runtime compatibility.
````

### Draft for `aliou/pi-synthetic`, do not file as-is without checking for a newer release

````md
Title: @aliou/pi-synthetic 0.17.2 exact Pi 0.74.0 peer blocks npm installs with Pi 0.75.3

Labels: bug

Description:

`@aliou/pi-synthetic@0.17.2` declares exact Pi 0.74.0 host peers:

```json
"peerDependencies": {
  "@earendil-works/pi-coding-agent": "0.74.0",
  "@earendil-works/pi-tui": "0.74.0"
}
```

After `@aliou/pi-linkup`'s peer range is relaxed, the same npm 11 graph fails at `@aliou/pi-synthetic@0.17.2`:

```text
While resolving: @aliou/pi-synthetic@0.17.2
Could not resolve dependency:
peerOptional @earendil-works/pi-coding-agent@"0.74.0" from @aliou/pi-synthetic@0.17.2
```

Minimal reproduction:

```sh
mkdir -p /tmp/pi-synthetic-eresolve-repro
cd /tmp/pi-synthetic-eresolve-repro
npm init -y >/dev/null
npm pkg set type=module \
  dependencies.'@earendil-works/pi-coding-agent'=0.75.3 \
  dependencies.'@aliou/pi-processes'=0.9.1 \
  dependencies.'@diegopetrucci/pi-openai-fast'=0.1.1 \
  dependencies.'@aliou/pi-synthetic'=0.17.2 >/dev/null
npm install --package-lock-only --ignore-scripts
```

Suggested fix:

Relax the Pi host peer ranges to include the supported Pi 0.75.3 host family, for example:

```json
"peerDependencies": {
  "@earendil-works/pi-coding-agent": "0.74.0 || ^0.75.3",
  "@earendil-works/pi-tui": "0.74.0 || ^0.75.3"
}
```

A local prototype with that metadata change, plus the matching `@aliou/pi-linkup` change, lets npm resolve the Pi
0.75.3 graph successfully with strict peer resolution still enabled. It does not prove full runtime compatibility.
````
