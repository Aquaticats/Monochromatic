# podman 5.8.7 `image prune --all --force` exits 125 on images held by external containers

## Symptom

A chained prune was run to reclaim space from rootless container storage:

```sh
# doc/troubleshooting/podman-image-prune-external-containers.md
podman container prune --force && podman image prune --all --force && podman volume prune --force
```

`container prune` succeeded.
`image prune` deleted most images and then failed,
printing the
removed image IDs on stdout and this on stderr:

```text
Error: 7 errors occurred:
	* image used by eac53fa344050d6f0deb0152ee8ad67b793ca5b648917163470c84e3b0ca2800: image is in use by a container: consider listing external containers and force-removing image
	* image used by 9c636664a3439ce9aeac177fd59dbd69f88147229c80285c9df54edd12ebcce6: image is in use by a container: consider listing external containers and force-removing image
	* image used by 41bc056153946df5953b4e8120bbd706ae04bcc58b67705edc27c983ca07ac53: image is in use by a container: consider listing external containers and force-removing image
	* image used by caa4bbce00d2e95aa542eb273fac751d8b7a7e718c2aaae9c9efed97e9fdcf6a: image is in use by a container: consider listing external containers and force-removing image
	* image used by 583cd2199310f52b460a5ac70bdc6bf4dba1ef3d745d3820921a94c8c7597a4a: image is in use by a container: consider listing external containers and force-removing image
	* image used by d3eaa7fd4d3b2243a80ca17e70f85c7f7343f080eff9c3190b0a8e9ff1c17109: image is in use by a container: consider listing external containers and force-removing image
	* image used by a3d0d43a91353dd69607e58ce4b066fc4958fd6298470be9a8d660c75d042122: image is in use by a container: consider listing external containers and force-removing image
```

The process exited 125.
Two consequences followed that the error text does not mention.

Because the commands were chained with `&&`,
the non-zero exit meant `volume prune` never ran,
so the reclaimable volume space was silently left behind.

The failure was not caused by a concurrent `podman pull`.
It reproduced from storage state
alone:
the seven images are referenced by containers that `container prune` does not manage.

## Root cause

The message is assembled in `containers/common`,
which podman 5.8.7 uses for image operations.
`libimage/image.go:535`:

```go
// libimage/image.go:535
if _, err := i.runtime.store.DeleteImage(i.ID(), true); handleError(err) != nil {
	if errors.Is(err, storage.ErrImageUsedByContainer) {
		err = fmt.Errorf("%w: consider listing external containers and force-removing image", err)
	}
	return processedIDs, err
}
```

The inner text `image is in use by a container` is `storage.ErrImageUsedByContainer` from
`containers/storage`,
defined in `types/errors.go`.
Note that `DeleteImage` is already called
with force set to `true`.
Force does not override this case:
containers/storage refuses to
delete an image that a container still references,
and the caller turns that refusal into the
advice string and returns.

`ErrImageUsedByContainer` arises for **external containers**,
which `podman-ps(1)` defines:

> Display external containers that are not controlled by Podman but are stored in containers
> storage.
> These external containers are generally created via other container technology such
> as Buildah or CRI-O and may depend on the same container images that Podman is also using.
> External containers are denoted with either a `buildah` or `storage` in the COMMAND and
> STATUS column of the ps output.

`podman container prune` removes stopped containers that podman's own database knows about.
External containers live only in containers/storage,
so prune does not see them and does not
remove them.
Their images therefore stay referenced,
and `image prune` fails per image.

The remaining images still deleted.
`libimage/image.go:539` returns the error for that one
image,
and the prune caller accumulates errors across images rather than aborting,
which is why
the output pairs a long list of successfully removed IDs on stdout with a seven-item multi-error
on stderr.

`podman-image-prune(1)` documents the flag that covers this case:

> `--external`
> Remove images even when they are used by external containers (e.g.,
> build containers).

Omitting `--external` is the direct cause.
This is documented behaviour with a documented
opt-in,
not a defect.

## Verification

Versions under test:

- `podman version 5.8.7`,
  rootless,
  graph root `~/.local/share/containers/storage`,
  overlay driver
- `containers/common` `libimage/image.go` at the revision carrying the quoted block,
  located by
  `gh search code "consider listing external containers"`,
  which resolves to
  `containers/common` path `libimage/image.go`
- `containers/storage` `types/errors.go` for `ErrImageUsedByContainer`,
  located the same way

The failure output in "Symptom" is captured verbatim from the run described there,
so the
symptom is reproduced rather than reconstructed.

The diagnostic and remediation commands below are recorded from `podman-ps(1)` and
`podman-image-prune(1)` for the installed version.
They were **not** executed during the
investigation,
because three podman processes were concurrently active,
including a
`cargo-mutants` container up for over an hour and an in-flight pull,
and querying storage would
have contended with that work.
Run them when podman is idle:

```sh
# doc/troubleshooting/podman-image-prune-external-containers.md
podman ps --external --all --format '{{.ID}} {{.Image}} {{.Command}} {{.Status}}'
podman image prune --all --force --external
podman volume prune --force
```

Catalog of prune invocations that complete cleanly:

- `podman container prune --force` on this storage succeeded and removed stopped containers
  podman owns.
- `podman image prune --all --force` deletes every image not referenced by any container,
  external or otherwise.

Catalog of invocations that fail:

- `podman image prune --all --force` fails with exit 125 and one `ErrImageUsedByContainer`
  entry per image held by an external container,
  when any external container exists.
- Any `&&` chain placed after that invocation is skipped,
  so a trailing `volume prune` or
  `system prune` silently does not run.

## Verified workarounds

Add `--external` to the image prune.
Tradeoff:
it removes images that Buildah or CRI-O build
containers are actively using,
so an in-flight build in another tool can lose its base image
mid-run.
Inspect with `podman ps --external --all` first when builds may be running.

Run the three prune steps as separate invocations rather than one `&&` chain,
so a partial
failure in one does not skip the others.
Tradeoff:
a failure in an early step no longer stops
later steps,
so each exit status has to be checked individually.

Remove the external containers explicitly,
then prune without `--external`.
Tradeoff:
this
destroys build state held by those containers,
which `--external` on the image prune would also
destroy,
but it additionally removes the containers themselves.

## What does not work

Running `podman container prune --force` before `image prune` does not clear the condition.
It
was run first here and the image prune still failed,
because prune only covers containers in
podman's database and external containers are not in it.

Force-removing images one at a time with `podman rmi --force <id>` hits the same
`ErrImageUsedByContainer` path,
since the refusal comes from containers/storage below the
force flag rather than from a missing force flag.

Retrying the identical command without `--external` reproduces the identical seven-item error.
The set of held images is stable until the external containers go away.

## Upstream filing decision

`.out-of-scope/` was checked.
It holds `bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
and `typescript-project-references.md`.
None exempts
podman or containers/storage.

The tracker was searched with
`gh search issues "image is in use by a container"`,
which returned six results across
repositories.
The closest is podman-container-tools/buildah#4917,
`"image is in use by a
container" error if building two images that use the same base at the same time with
--layers=false`,
open.
That shares the error string but has a different cause:
concurrent
builds racing on a shared base,
not a prune blocked by pre-existing external containers.
It is
related context,
not a duplicate.
The scoped search
`gh search issues --repo containers/podman "prune external containers"` was rejected because the
project now resolves as `podman-container-tools/podman`,
so the unscoped search was used.

The six constraints,
walked:

1. **Is it really upstream's fault?**
   No. `podman-image-prune(1)` documents `--external` for
   exactly this case,
   and `podman-ps(1)` documents how to list the containers responsible.
   The
   error string itself names the remedy.
   This is correct behaviour with the flag omitted.
2. **Can upstream fix it?**
   Not applicable,
   since there is nothing to fix.
   A case could be made
   for `image prune` mentioning `--external` in the error text rather than only "force-removing
   image",
   but that is wording polish.
3. **Are they supporting this use case?**
   Yes.
   Both flags are documented features.
4. **Would the repo welcome our contribution?**
   Unevaluated.
   No filing is proposed,
   so no
   `CONTRIBUTING.md` or policy check was run.
5. **Will they likely fix it?**
   Unevaluated,
   not gated,
   because constraint 1 already fails.
6. **Have we prototyped a minimal fix?**
   No. Constraint 1 fails,
   so the auto-prototype step does
   not trigger.
   The remedy is an existing documented flag.

Decision:
do not file.
Constraint 1 fails outright.

Nothing to add to podman-container-tools/buildah#4917.
That thread is about a race between two
concurrent builds sharing a base image.
The finding recorded here is a prune blocked by
long-lived external containers,
and the resolution is a documented flag,
so a comment would not
advance that thread.
