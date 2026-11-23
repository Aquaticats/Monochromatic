# Incus 7.6 launch waits before its API call when an automation harness leaves stdin open

Status: incomplete and unapproved. The cause and a tested invocation fix were recorded on 2026-11-23
during a broader, unfinished AppArmor rollout. This is not an upstream-filing artifact.

## Symptom

`incus launch` can remain silent before making an Incus API request or opening a network connection
to the image server. In the observed run, `incus operation list` was empty, the image directory had
not received an image, and the client had only its inherited standard-I/O sockets and runtime file
descriptors.

The command used here had valid image and instance arguments. Left with an open, nonterminal stdin
socket, it did not produce an AppArmor denial. It also did not make progress merely by waiting.

This is distinct from the later image alias error and the later missing root idmap error. Closed
stdin allowed those separate failures to surface immediately instead of hiding them behind an
unfinished input read.

## Root cause

Incus v7.6.0, commit `8f5e6c2235f6424ec59d3dc06f12e344a4a416e2`, intentionally treats nonterminal
stdin as optional instance configuration. `cmd/incus/create.go:104-116` loads YAML from `os.Stdin`
([selected
source](https://github.com/lxc/incus/blob/8f5e6c2235f6424ec59d3dc06f12e344a4a416e2/cmd/incus/create.go#L104-L116)):

```go
// If stdin isn't a terminal, read text from it
var stdinData api.InstancePut
if !termios.IsTerminal(getStdinFd()) {
        loader, err := yaml.NewLoader(os.Stdin)
        if err != nil {
                return nil, err
        }

        err = loader.Load(&stdinData)
        if err != nil && !errors.Is(err, io.EOF) {
                return nil, err
        }
}
```

An agent harness may keep command stdin open without supplying YAML or EOF. The YAML loader is then
waiting for completion of its input document, not for the Incus daemon or the image server. Until
EOF, the client never reaches the instance-creation API request.

`launch --explain` also distinguishes a syntax error from this wait. It showed `images:alpine` and
`apparmor-rollout-check` parsed as the intended image and instance name; the initial hang therefore
was not an unrecognized `launch` argument.

## Verification

The host package was `incus 7.6.0-1.1`. The matching v7.6.0 source tag resolved to the commit cited
here. An open, harness-provided stdin socket produced the stalled client. No local Incus operation
existed at that point.

The same arguments with closed stdin reached image lookup and returned an explicit error instead of
hanging:

```sh
timeout --signal=TERM --kill-after=5s 60 \
  sudo incus launch images:alpine/3.24 apparmor-rollout-check \
  --config limits.cpu=1 --config limits.memory=256MiB < /dev/null
```

The path that failed first was a different problem: `/etc/subuid` and `/etc/subgid` had no root
mapping. After that separate configuration was corrected, this exact invocation produced
`Launching apparmor-rollout-check`, and `incus info apparmor-rollout-check` reported
`Status: RUNNING`.

The successful workload was checked inside the container, then its disposable instance was deleted:

```sh
sudo incus exec apparmor-rollout-check -- /bin/sh -c \
  'printf "Incus workload started\n"; /bin/cat /proc/self/attr/current; \
  printf "Container filesystem access succeeded\n" > /root/apparmor-workload-proof.txt; \
  /bin/cat /root/apparmor-workload-proof.txt' < /dev/null
sudo incus delete --force apparmor-rollout-check < /dev/null
```

The inner label was `unconfined`. The host instead showed the stacked Incus profile ending in
`:unconfined (enforce)`. These two views must not be conflated: the outer host policy was enforcing,
while the container namespace did not add a second confined profile.

## Verified workaround

When no instance configuration is intended, close stdin for automated `launch`, `create`, and
comparable commands:

```sh
sudo incus launch images:alpine/3.24 apparmor-rollout-check \
  --config limits.cpu=1 --config limits.memory=256MiB < /dev/null
```

If stdin is a pipe and no YAML is supplied, an empty input must reach EOF. `/dev/null` supplies that
condition directly. The commit-pinned excerpt was verified through `agent-browser`. The corrected
invocation was verified on the running host.

This does not disable YAML input. When configuration is intentionally supplied, feed an explicit
trusted YAML file and close its stream after reading. Do not let Incus consume an agent protocol
stream or unrelated application input as root-level instance configuration.

The one-CPU and 256 MiB limits belong to this disposable smoke-test workload, not a minimum Incus
requirement. The observed instance reported about 4 MiB resident memory while running the checked
shell workload; no broader capacity claim is made.

## What does not work

- Keeping a harness stdin socket open when no YAML document will follow. The loader is still waiting
  for input; an empty `incus operation list` does not distinguish this from an omitted or failed
  server request.
- Using a nonexistent image alias as a readiness test. Once stdin was closed, `images:alpine`
  returned `Failed getting image: The requested image couldn't be found`. The downloaded image
  catalog advertised `alpine/3.24`, which was the working alias for this run.
- Treating the stalled process as evidence of an AppArmor denial. The source-level input read
  explains the observed boundary before any policy-relevant container operation.

## Upstream filing decision

Nothing should be filed for this behavior. No issue or comment is proposed, so there is no fileable
draft to publish.

1.  Upstream fault is not established. The observed behavior follows the documented
    stdin-configuration feature, and the host invocation supplied a nonterminal stream that remained
    open.
2.  No upstream repair is needed for the correct boundary invocation. Closing an unused input stream
    works without changing Incus.
3.  The use case is supported. The installed `launch --help` includes
    `incus launch images:debian/12 c2 < config.yaml` for intentional YAML input.
4.  Direct filing does not meet Incus's contribution rules. `CONTRIBUTING.md:21-26` states
    `We do NOT allow direct filing of issues by LLMs` and requires a human to enter the issue form
    ([selected
    policy](https://github.com/lxc/incus/blob/8f5e6c2235f6424ec59d3dc06f12e344a4a416e2/CONTRIBUTING.md#L21-L31)).
    No human-backed filing was requested.
5.  There is no demonstrated behavior for a maintainer to fix. The valid image alias and the lack of
    a root ID map were separate incidents, not effects to fold into this input diagnosis.
6.  The applicable fix is at the caller boundary: provide intentional YAML or close stdin. It was
    exercised through the complete create, inspect, workload, and delete path. No upstream-patch
    prototype is proposed.

The `.out-of-scope/` filenames were inspected and searched for `incus`, `subuid`, `subgid`, and
`stdin`. None of these records matched this incident. That absence is not a reason to turn ordinary
documented CLI behavior into an upstream report.
