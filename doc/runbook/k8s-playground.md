# Single-node Kubernetes playground (kind on rootless podman, mise-managed)

How to stand up a throwaway single-node Kubernetes cluster to experiment with,
 driven entirely by
mise,
 living in a self-contained directory outside this repo (`~/k8s-playground/`).
 The repo carries
no Kubernetes config;
 this runbook is the reproducible record of the procedure,
 so the playground can
be rebuilt from nothing on a fresh machine.

Honesty note:
 this procedure is fully automated,
 not a last-resort manual handoff.
 Every step is a
mise command,
 and the cluster below was created and verified by running those commands,
 not by hand.
The only "manual" actions are typing the mise commands and quitting the k9s TUI.
 There are no failed
bridges to report;
 the runbook exists because it was asked for as a durable,
 repeatable record.

## What this proves

Verified end to end on 2026-06-15,
 on Bazzite `44.20260608.0` (Fedora Kinoite 44,
 ostree-immutable),
kernel `7.0.9-ogc3.2.fc44.x86_64`,
 with rootless podman `5.8.2` and mise `2026.6.9`:

- `mise run up` creates a single control-plane kind cluster named `play` running vanilla upstream
  Kubernetes `v1.36.1` (node image `kindest/node:v1.36.1`) inside rootless podman,
   with no
  `Delegate=yes` preflight failure.
- `kubectl`/`k9s` reach it through the `kind-play` context merged into `~/.kube/config`.
- A real nginx workload serves HTTP through `kubectl port-forward`.

Tool versions installed by mise on this instance (yours may float,
 since they are pinned to
`latest`):
 kind `v0.32.0`,
 kubectl `v1.36.2`,
 k9s `v0.51.0`,
 helm `v4.2.1`,
 node `26.3.0`.

## Setup

Status:
 DONE

Prerequisites a fresh machine needs:

- Linux with rootless podman and cgroup v2 controller delegation (`cpu`,
   `memory`,
   `pids` at least).
  The two Linux-only workarounds below are gated on `process.platform`,
   so the same config also runs
  on macOS or Windows against a started `podman machine`,
   calling `kind` directly.
- mise installed and shell-activated.
   Confirm the shell sources it:
  `grep -- 'mise activate' ~/.bashrc` prints a line.
   Without activation,
   the `cd` enter hook and the
  directory tool environment do not fire.
- No real Docker daemon is required;
   on this host `docker` is podman's CLI shim.

Create the project directory and its `mise.toml`:

```toml
# ~/k8s-playground/mise.toml
[settings]
# mise's directory enter/leave/cd hooks are gated behind this. Already true in the global
# config; set here too so the playground is self-contained.
experimental = true

[tools]
# Single-node Kubernetes playground. kind runs vanilla upstream Kubernetes inside rootless
# podman; kubectl/k9s/helm are the clients. node backs the `node -e` task bodies (added here so
# the directory is self-contained rather than relying on a parent mise config). Unpinned
# ("latest") because this is a personal scratch project and kind's node image (the k8s version)
# tracks the kind binary it ships.
kind = "latest"
kubectl = "latest"
k9s = "latest"
helm = "latest"
node = "latest"

[env]
# kind has no podman provider flag; it reads this env var. Required because this host has no
# real Docker daemon (the `docker` binary is podman's CLI shim).
KIND_EXPERIMENTAL_PROVIDER = "podman"

[hooks]
# On `cd` into this dir, ensure the tools are installed so kind/kubectl/k9s/helm are directly
# runnable. Mirrors the monorepo's enter-hook pattern (mise.toml): a hook needs a string
# body and must be cross-OS depending only on mise itself (no tool is installed yet on first
# entry), so it delegates to a task whose array `run` mise sequences cross-platform, rather than
# an inline `mise install && echo ...` (the `&&`/`echo` are sh-only and break on Windows cmd).
enter = { task = "bootstrap" }

[tasks.bootstrap]
description = "Install the playground's mise tools (runs on directory entry)."
run = ["mise install"]

[tasks.setup]
description = "One-time, Linux only: point rootless podman at the k8s-file log driver so kind can tail container logs without hanging."
shell = "node -e"
run = '''
if (process.platform !== 'linux') {
  console.error('setup: nothing to do here (the k8s-file log-driver tweak is for rootless Linux podman).');
} else {
  const { writeFileSync, mkdirSync } = require('node:fs');
  const { join } = require('node:path');
  const base = process.env.XDG_CONFIG_HOME || join(process.env.HOME, '.config');
  const dir = join(base, 'containers', 'containers.conf.d');
  mkdirSync(dir, { recursive: true });
  const file = join(dir, '10-kind-log-driver.conf');
  writeFileSync(file, '[containers]\nlog_driver = "k8s-file"\n');
  console.error(`Wrote ${file}`);
}
'''

[tasks.up]
description = "Create (or no-op if present) the single-node kind cluster 'play', then show nodes."
depends = ["setup"]
shell = "node -e"
run = '''
const { spawnSync } = require('node:child_process');
const NAME = 'play';
const existing = spawnSync('kind', ['get', 'clusters'], { encoding: 'utf8' });
if ((existing.stdout || '').split('\n').includes(NAME)) {
  console.error(`Cluster ${NAME} already exists; skipping create.`);
} else {
  const kindArgs = ['create', 'cluster', '--name', NAME, '--wait', '90s'];
  // Linux only: run kind create inside a transient user scope so a KDE/Plasma session does not
  // strip the cpu cgroup controller from leaf processes (podman#22676). systemd-run exists only
  // on Linux; on macOS/Windows (podman machine) call kind directly.
  const useScope = process.platform === 'linux'
    && spawnSync('systemd-run', ['--version'], { stdio: 'ignore' }).status === 0;
  const [cmd, args] = useScope
    ? ['systemd-run', ['--user', '--scope', '--collect', '--quiet', 'kind', ...kindArgs]]
    : ['kind', kindArgs];
  const created = spawnSync(cmd, args, { stdio: 'inherit' });
  if (created.status !== 0) { throw new Error(`kind create cluster failed (exit ${created.status})`); }
}
spawnSync('kubectl', ['--context', `kind-${NAME}`, 'get', 'nodes'], { stdio: 'inherit' });
'''

[tasks.down]
description = "Delete the kind cluster 'play' and its kubeconfig context."
run = "kind delete cluster --name play"

[tasks.status]
description = "Show cluster info, the node, and all pods."
shell = "node -e"
run = '''
const { spawnSync } = require('node:child_process');
const ctx = 'kind-play';
spawnSync('kubectl', ['--context', ctx, 'cluster-info'], { stdio: 'inherit' });
spawnSync('kubectl', ['--context', ctx, 'get', 'nodes', '--output', 'wide'], { stdio: 'inherit' });
spawnSync('kubectl', ['--context', ctx, 'get', 'pods', '--all-namespaces'], { stdio: 'inherit' });
'''

[tasks.k9s]
description = "Open the k9s TUI against the playground cluster."
run = "k9s --context kind-play"
```

Trust the new config so mise evaluates its env and runs the hook:

```sh
mise trust ~/k8s-playground
```

Expected:
 `mise trusted /var/home/user/k8s-playground` (the path reflects your home).

## Steps

Status:
 DONE

1.  Run **`cd ~/k8s-playground`**.
    Expected:
     the `enter` hook fires and prints `[bootstrap] $ mise install` followed by
    `mise all tools are installed` (the first run instead downloads kind/kubectl/k9s/helm/node).
    After it,
     `command -v kind` resolves to a path under `~/.local/share/mise/installs/kind/`.

2.  Run **`mise run up`**.
    Expected:
     lines `Ensuring node image (kindest/node:v1.36.1)`,
     then
    `Starting control-plane`,
     then `Set kubectl context to "kind-play"`,
     and finally a node table row
    `play-control-plane   Ready    control-plane   <age>   v1.36.1`.
     First run takes about a minute
    while the node image downloads;
     later runs print `Cluster play already exists; skipping create.`

3.  Run **`mise run status`**.
    Expected:
     `Kubernetes control plane is running at https://127.0.0.1:<port>` and every pod in the
    `kube-system` namespace showing `Running` (coredns,
     etcd,
     kindnet,
     kube-apiserver,
    kube-controller-manager,
     kube-proxy,
     kube-scheduler) plus `local-path-provisioner`.

4.  Optional,
     prove a real workload serves traffic.
     Run,
     one command per line:

    ```sh
    kubectl --context kind-play create deployment hello --image=nginx --port=80
    kubectl --context kind-play rollout status deployment/hello --timeout=120s
    kubectl --context kind-play port-forward deployment/hello 8080:80 &
    curl --silent --retry 15 --retry-connrefused --retry-delay 1 http://localhost:8080
    kill %1
    kubectl --context kind-play delete deployment hello
    ```

    Expected:
     `deployment "hello" successfully rolled out`,
     then the nginx HTML containing
    `<title>Welcome to nginx!</title>`,
     then `deployment.apps "hello" deleted`.

5.  Optional,
     browse interactively.
     Run **`mise run k9s`**,
     then quit with **`:q`**.
    Expected:
     the k9s TUI opens listing the cluster's pods;
     **`:q`** returns to the shell.

## What to check

Status:
 DONE

- Cluster exists:
   `mise exec -- kind get clusters` prints `play`.
- Node is ready:
   `mise exec -- kubectl --context kind-play get nodes` prints a row whose status
  column is exactly `Ready` for `play-control-plane`.
- Backing container is up:
   `podman ps --format '{{.Names}} {{.Status}}'` includes a line starting
  `play-control-plane` with status beginning `Up`.
- Context is wired:
   `mise exec -- kubectl config get-contexts kind-play` prints a row with `*` in the
  current column and `kind-play` in the name,
   cluster,
   and authinfo columns.
- Linux log-driver drop-in was written:
   the file
  `~/.config/containers/containers.conf.d/10-kind-log-driver.conf` exists and contains
  `log_driver = "k8s-file"`.

## Restore

Status:
 TODO

- Delete the cluster:
   run **`mise run down`**.
  Expected:
   `Deleting cluster "play" ...` and `Deleted nodes: ["play-control-plane"]`.
   Confirm with
  `mise exec -- kind get clusters`,
   which should no longer list `play` (it prints
  `No kind clusters found.` when none remain).
- Remove the project entirely (optional):
   `rm -rf ~/k8s-playground`.
- Remove the podman log-driver drop-in (optional,
   Linux):
  `rm -f ~/.config/containers/containers.conf.d/10-kind-log-driver.conf`.
- Confirm no leaked containers:
   `podman ps --all` lists no `play-control-plane` entry.
