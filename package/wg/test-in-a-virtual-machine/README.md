# wg-test-in-a-virtual-machine

Runs the `package/wg` tests that need privileges inside a throwaway virtual
machine, so the host that starts them keeps its network, its firewall, its
cgroup tree and its service manager as they were.

```console
mise run //package/wg/test-in-a-virtual-machine:run -- bypass
```

The four tests it runs are the ones that cannot run on a host:

- `bypass`: makes veth links with addresses in the caller's own network
  namespace, starts a route watcher as root and drives the built CLI up and
  down.
- `route`: builds two namespaces, a transferring WireGuard tunnel, and drives
  the built CLI, which writes firewall and OpenSnitch state as root.
- `functional`: runs every ignored test of `wg-quicker-exempt` as root and every
  test that needs a user manager as the ordinary account.
- `functional-protocols`: loads all four protocol hooks on one throwaway cgroup
  as root.

## What one run does

1.  Reads `toolchain.lock.json` for this platform's Node and pnpm archives,
    downloads each once into
    `${XDG_CACHE_HOME:-~/.cache}/monochromatic-dev/wg-test-in-a-virtual-machine`
    and checks its SHA-256 against the lock.
2.  Packs the checkout: every workspace `package.json`, every `src` directory,
    the two `pnpmfile` files, the lockfile and `package/wg/cli-quicker`'s built
    bundles. Build output, dependency folders and Git history stay out.
3.  Serves those files on the loopback interface, which the guest reaches at
    `10.0.2.2` in QEMU's user-mode network. A push through the guest agent moves
    about 0.4 MiB per second; this moves a hundred times that.
4.  Creates a machine with `mvm`: the `ubuntu` cloud image, 4 vCPUs and 8 GiB,
    as `mvm` defaults describe.
5.  Provisions the guest: the packages the tests need (`wireguard-tools`,
    `nftables`, `iproute2`, `iputils-ping`, `sudo`, `curl`, `jq`, `python3`,
    `libatomic1`, `build-essential`), the locked Node and pnpm, then the
    checkout, then a passwordless `sudo` for the `ubuntu` account, its user
    manager through lingering, and the marker directory that tells a test it
    runs in a machine.
6.  Installs the checkout's Node dependencies from its lockfile with
    `pnpm install --frozen-lockfile`, or installs Rust through `rustup` and
    builds the crate, whichever the test needs.
7.  Runs each phase as the caller it was written for, prints what each printed,
    lists what was left behind in the guest, and destroys the machine.

Every machine is destroyed on the way out, including after a failed phase, so
`mvm list` answers with the same machines it did before the run.

## The caller each test was written for

`functional` runs in two phases, in this order:

1.  The six tests that need an unprivileged caller with a user manager of its
    own (`manager_probe_tests`, the real-manager sweep, the real-manager
    adoption, and the sentinel test that needs a running root unit it may not
    stop) run as `ubuntu`.
2.  Every other ignored test runs as root through `sudo`, with those six
    skipped.

The order is not cosmetic. The crate keeps its fixtures in a scratch directory
below its build output and the adoption gate is a file that only its owner may
open, so a root phase that ran first would leave both owned by root, and the
account's phase then fails with `Permission denied` in a fixture rather than in
what the test is about.

## Requirements

- Linux with KVM (`/dev/kvm`), and `mvm`'s prerequisites: `virsh` and
  `qemu-img`, on `PATH` or in the `org.virt_manager.virt-manager` Flatpak.
- About 4 GiB of free space for images and disks below `mvm`'s data directory,
  and about 1.2 GiB of disk inside each guest.
- Network access on the host for the first run's archives, and inside the guest
  for the package installation, `rustup` and the npm registry.
- `mvm`'s data directory can be moved with `MVM_DATA_DIR`, which a host whose
  `~/.local/share/mvm` cannot hold the images needs.

## What it refuses

The tests themselves refuse to run outside a prepared machine and name the mise
task that provides one, so a person who copies a phase command out of a log, or
runs `cargo test -- --ignored` on a host, is told what to do instead of changing
the host they are on.

## Limits

- A first run downloads about 620 MiB of cloud image and 90 MiB of toolchains,
  and bakes the image's template, which takes minutes. A later run clones the
  template and takes about a minute to a few minutes per test.
- The guest's account is the image's `ubuntu`, uid 1000, and the tests' manager
  paths name that uid. Provisioning checks it and stops otherwise.
- Only the Linux x86-64 archives are read from the lock, so the runner is
  x86-64 only.
- The inventory printed before destruction lists what the phases left behind;
  it is not a cleanup. Anything a failing phase leaves goes away with the
  machine.

## Not final and not approved

This package was written to run four tests that had never run, and the report of
that work names what the runs showed. It is not reviewed.
