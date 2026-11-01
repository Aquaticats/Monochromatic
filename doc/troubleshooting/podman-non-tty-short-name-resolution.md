# Podman 5.8.4 non-TTY short-name pulls fail under enforced resolution

## Symptom

On Fedora 44 with `podman-5.8.4-1.fc44.x86_64`,
 pulling the unqualified
`capsulecode/singlefile` image from a process without a TTY fails before Podman contacts a registry:

```console
$ podman pull --quiet capsulecode/singlefile
Error: short-name resolution enforced but cannot prompt without a TTY
```

The `docker` command has the same failure because Fedora's `podman-docker` package provides a wrapper around Podman:

```console
$ docker pull --quiet capsulecode/singlefile
Emulate Docker CLI using podman. Create /etc/containers/nodocker to quiet msg.
Error: short-name resolution enforced but cannot prompt without a TTY
```

Interactive terminals may instead show a registry-selection prompt.
Automation,
 redirected input or output,
 and agent command runners expose the non-TTY failure.

## Root cause

The installed `/usr/bin/docker:2-4` checks only whether to print its compatibility message,
then replaces itself with `/usr/bin/podman`:

```sh
[ -e /etc/containers/nodocker ] || [ -e "${XDG_CONFIG_HOME-$HOME/.config}/containers/nodocker" ] || \
echo "Emulate Docker CLI using podman. Create /etc/containers/nodocker to quiet msg." >&2
exec /usr/bin/podman "$@"
```

The `docker` spelling therefore does not select Docker Engine.
The marker named in the message only suppresses that message.

Podman's pull command passes each image argument to the image engine at
[`cmd/podman/images/pull.go:220-228`][pull-command]:

```go
// Let's do all the remaining Yoga in the API to prevent us from
// scattering logic across (too) many parts of the code.
var errs utils.OutputErrors
for _, arg := range args {
    pullReport, err := registry.ImageEngine().Pull(registry.Context(), arg, pullOptions.ImagePullOptions)
    if err != nil {
        errs = append(errs, err)
        continue
    }
```

The ABI image engine forwards the raw image name to libimage at
[`pkg/domain/infra/abi/images.go:307-314`][abi-pull]:

```go
if !options.Quiet && pullOptions.Writer == nil {
    pullOptions.Writer = os.Stderr
}

pulledImages, err := ir.Libpod.LibimageRuntime().Pull(ctx, rawImage, options.PullPolicy, pullOptions)
if err != nil {
    return nil, err
}
```

Libimage invokes the vendored short-name resolver before pulling at
[`vendor/go.podman.io/common/libimage/pull.go:558-565`][libimage-resolve]:

```go
sys := r.systemContextCopy()
resolved, err := shortnames.Resolve(sys, imageName)
if err != nil {
    if localImage != nil && pullPolicy == config.PullPolicyNewer {
        return localImage, nil
    }
    return nil, err
}
```

The resolver reads `short-name-mode` and bypasses short-name expansion for a fully qualified reference at
[`vendor/go.podman.io/image/v5/pkg/shortnames/shortnames.go:243-264`][resolver-entry]:

```go
// Detect which mode we're running in.
mode, err := sysregistriesv2.GetShortNameMode(ctx)
if err != nil {
    return nil, err
}

isShort, shortRef, err := parseUnnormalizedShortName(name)
if err != nil {
    return nil, err
}
if !isShort { // no short name
    resolved.addCandidate(shortRef)
    return resolved, nil
}
```

For a short name,
 the resolver first looks for a configured alias and returns it without consulting search registries at
[`vendor/go.podman.io/image/v5/pkg/shortnames/shortnames.go:279-307`][resolver-alias]:

```go
// If there's already an alias, use it.
namedAlias, aliasOriginDescription, err := sysregistriesv2.ResolveShortNameAlias(ctx, shortNameRepo.String())
if err != nil {
    return nil, err
}

// Always use an alias if present.
if namedAlias != nil {
    // Tag and digest restoration omitted here.
    resolved.addCandidate(namedAlias)
    resolved.rationale = rationaleAlias
    resolved.originDescription = aliasOriginDescription
    return resolved, nil
}
```

Alias configuration itself cannot include a tag or digest.
The alias parser rejects both at
[`vendor/go.podman.io/image/v5/pkg/sysregistriesv2/shortnames.go:213-230`][alias-value]:

```go
// parseShortNameValue parses the specified alias into a reference.Named.  The alias is
// expected to not be tagged or carry a digest and *must* include a
// domain/registry.
func parseShortNameValue(alias string) (reference.Named, error) {
    ref, err := reference.Parse(alias)
    if err != nil {
        return nil, fmt.Errorf("parsing alias %q: %w", alias, err)
    }

    if _, ok := ref.(reference.Digested); ok {
        return nil, fmt.Errorf("invalid alias %q: must not contain digest", alias)
    }

    if _, ok := ref.(reference.Tagged); ok {
        return nil, fmt.Errorf("invalid alias %q: must not contain tag", alias)
    }
```

This machine's `/etc/containers/registries.conf.d/000-shortnames.conf` has no
`capsulecode/singlefile` alias.
Its `/etc/containers/registries.conf:22,79` instead defines three search registries and enforcing mode:

```toml
unqualified-search-registries = ["registry.fedoraproject.org", "registry.access.redhat.com", "docker.io"]
short-name-mode = "enforcing"
```

The resolver turns every configured registry into a candidate at
[`vendor/go.podman.io/image/v5/pkg/shortnames/shortnames.go:312-332`][resolver-candidates]:

```go
unqualifiedSearchRegistries, usrConfig, err := sysregistriesv2.UnqualifiedSearchRegistriesWithOrigin(ctx)
if err != nil {
    return nil, err
}

for _, reg := range unqualifiedSearchRegistries {
    named, err := reference.ParseNormalizedNamed(fmt.Sprintf("%s/%s", reg, name))
    if err != nil {
        return nil, fmt.Errorf("creating reference with unqualified-search registry %q: %w", reg, err)
    }
    resolved.addCandidate(named)
}
```

One candidate needs no choice.
Multiple candidates in enforcing mode require both standard input and standard output to be TTYs.
The exact diagnostic comes from
[`vendor/go.podman.io/image/v5/pkg/shortnames/shortnames.go:340-353`][resolver-tty]:

```go
// If we have only one candidate, there's no ambiguity.
if len(resolved.PullCandidates) == 1 {
    return resolved, nil
}

// If we don't have a TTY, act according to the mode.
if !term.IsTerminal(int(os.Stdout.Fd())) || !term.IsTerminal(int(os.Stdin.Fd())) {
    switch mode {
    case types.ShortNameModePermissive:
        // Permissive falls back to using all candidates.
        return resolved, nil
    case types.ShortNameModeEnforcing:
        // Enforcing errors out without a prompt.
        return nil, errors.New("short-name resolution enforced but cannot prompt without a TTY")
```

This is an ambiguity guard,
 not a registry outage and not a SingleFile failure.
The image exists at `docker.io/capsulecode/singlefile:latest`,
 as the fully qualified manifest probe confirms.

## Verification

The reproduced environment was:

```text
Podman version: 5.8.4
Git commit: 5431df23c742e5edea35bef34eed696f4db0106b
RPM: podman-5.8.4-1.fc44.x86_64
Configuration: three unqualified registries, short-name-mode="enforcing"
Invocation: non-TTY command runner
```

### Source checkout and boundary

The source trace used the read-only third-party checkout at
`~/temp/agent/podman-v5.8.4-20260903`.
Its origin was `https://github.com/podman-container-tools/podman.git`,
its exact tag was `v5.8.4`,
and `git rev-parse HEAD` returned the reported commit.
`git status --short` was empty after investigation.
No source workaround or prototype was applied to the repository that we do not own.

### Patterns that fail

Both direct Podman and Fedora's Docker compatibility wrapper reproduce the same resolver diagnostic:

```console
$ podman pull --quiet capsulecode/singlefile
Error: short-name resolution enforced but cannot prompt without a TTY

$ docker pull --quiet capsulecode/singlefile
Emulate Docker CLI using podman. Create /etc/containers/nodocker to quiet msg.
Error: short-name resolution enforced but cannot prompt without a TTY
```

`podman pull --help` exposes no per-command short-name-mode option.
`--quiet` controls pull output and does not affect resolution.

### Patterns that work cleanly

The probes used a private throwaway directory:

```console
mkdir --parents "$HOME/temp/agent"
chmod 700 "$HOME/temp/agent"
fixture="$(mktemp --directory "$HOME/temp/agent/podman-short-name.XXXXXXXX")"
chmod 700 "$fixture"
```

A fully qualified target bypasses short-name expansion.
The exact target returned a valid OCI image index:

```console
$ podman manifest inspect docker.io/capsulecode/singlefile:latest
{
    "schemaVersion": 2,
    "mediaType": "application/vnd.oci.image.index.v1+json",
    "manifests": [
```

A generic fully qualified pull also succeeded in an isolated temporary image store.
This checks the pull path without downloading the larger SingleFile image:

```console
$ podman --root "$fixture/root" --runroot "$fixture/runroot" \
    pull --quiet docker.io/library/hello-world:latest
e2ac70e7319a02c5a477f5825259bd118b94e8b02c279c67afa63adab6d8685b
$ podman --root "$fixture/root" --runroot "$fixture/runroot" \
    image inspect docker.io/library/hello-world:latest --format '{{.RepoTags}}'
[docker.io/library/hello-world:latest]
```

A target-specific per-user alias also returned the OCI image index:

```console
$ alias_home="$fixture/alias-home"
$ mkdir --parents "$alias_home/.config/containers/registries.conf.d"
$ printf '%s\n' '[aliases]' \
    '"capsulecode/singlefile" = "docker.io/capsulecode/singlefile"' \
    > "$alias_home/.config/containers/registries.conf.d/999-singlefile.conf"
$ HOME="$alias_home" podman manifest inspect capsulecode/singlefile
{
    "schemaVersion": 2,
    "mediaType": "application/vnd.oci.image.index.v1+json",
```

A configuration with one search registry is unambiguous even in enforcing mode:

```console
$ printf '%s\n' 'unqualified-search-registries = ["docker.io"]' \
    'short-name-mode = "enforcing"' > "$fixture/single-registry.conf"
$ CONTAINERS_REGISTRIES_CONF="$fixture/single-registry.conf" \
    podman manifest inspect capsulecode/singlefile
{
    "schemaVersion": 2,
    "mediaType": "application/vnd.oci.image.index.v1+json",
$ podman --root "$fixture/root" --runroot "$fixture/runroot" \
    image rm --force docker.io/library/hello-world:latest
$ podman unshare find "$fixture" -depth -delete
```

## Verified workarounds

### Use a fully qualified image reference

This is the preferred workaround for commands,
 automation,
 and container files:

```console
podman pull docker.io/capsulecode/singlefile:latest
```

It preserves enforcing mode and states which registry is trusted.
Its tradeoff is that the registry is now part of every image reference.
The mutable `latest` tag can still move,
 so reproducible automation should use an approved digest.

### Add a narrow per-user alias

For existing callers that cannot change the short name,
 create
`$HOME/.config/containers/registries.conf.d/999-singlefile.conf`:

```toml
[aliases]
"capsulecode/singlefile" = "docker.io/capsulecode/singlefile"
```

Then the original command resolves without prompting:

```console
podman pull capsulecode/singlefile:latest
```

The tested per-user drop-in returned the target's OCI image index under the system's enforcing mode.
Its tradeoff is hidden machine-local policy:
 another machine without the alias still fails or may resolve differently.
Because the alias parser rejects a tag or digest in an alias value,
 callers must retain those in the image argument.

### Configure one trusted search registry

An environment-specific `registries.conf` can remove ambiguity:

```toml
unqualified-search-registries = ["docker.io"]
short-name-mode = "enforcing"
```

The tested configuration resolved the target without a TTY.
Its tradeoff is broad scope:
 every otherwise unaliased short name resolves through Docker Hub.
Use this only where Docker Hub is the intended source for that whole environment.

### Bypass Podman for SingleFile

For this capture task,
 the native package was installed and exercised directly:

```console
npm install --global single-file-cli@2.7.2
single-file --help
```

This avoids container-name resolution entirely.
Its tradeoff is a host-level Node and browser installation instead of container isolation.

## What does not work

### Creating a `nodocker` marker

A temporary user marker suppressed only the compatibility message:

```console
$ touch "$HOME/.config/containers/nodocker"
$ docker pull --quiet capsulecode/singlefile
Error: short-name resolution enforced but cannot prompt without a TTY
```

`/usr/bin/docker:2-4` shows why:
 the marker gates `echo`,
 while `exec /usr/bin/podman "$@"` is unconditional.

### Changing the command spelling from `docker` to `podman`

The Fedora wrapper already invokes `/usr/bin/podman` with the same arguments.
Both spellings therefore reach the same resolver and produce the same diagnostic.

### Adding `--quiet`

`--quiet` only changes output setup in
[`pkg/domain/infra/abi/images.go:307-311`][abi-pull].
Short-name resolution still runs later in libimage,
 so the TTY requirement remains.

### Globally weakening short-name enforcement

`short-name-mode="permissive"` would avoid this particular hard failure,
but the resolver then returns every configured candidate at
[`vendor/go.podman.io/image/v5/pkg/shortnames/shortnames.go:345-350`][resolver-tty].
That does not preserve the registry choice that enforcing mode protects.
A fully qualified name or narrow alias fixes the ambiguity without weakening policy for unrelated images.

## Upstream filing decision

No `.out-of-scope/` entry covers Podman or container short-name resolution.

Searches across open and closed issues and pull requests used
`short name resolution`,
 `cannot prompt`,
 and the exact diagnostic.
They found the exact duplicate,
 [Podman issue 11530][issue-11530].
In the [closing maintainer comment][issue-11530-closing],
 the maintainer says enforcing mode “hard-fail[s]” when a script cannot be prompted and calls this intended behavior.
An earlier [issue comment][issue-11530-qualified] recommends that the caller use `docker.io/kindest/node`.
The same remedy applies to the SingleFile image's fully qualified name.

The six constraints are:

1. **Is it really upstream's fault?**
    No defect was established.
   Podman emits the documented,
    source-confirmed result of enforcing mode with ambiguous registries and no TTY.
2. **Can upstream fix it?**
    Mechanically yes,
    but silently choosing a registry would alter the security policy.
   Existing configuration and qualified-name paths already represent the intended solutions.
3. **Are they supporting this use case?**
    Yes for non-interactive pulls with unambiguous references.
   Podman's pull documentation recommends fully qualified references and explains aliases.
4. **Would the repo welcome our contribution?**
    Generally yes.
   The checked `README.md`,
   `CONTRIBUTING.md`,
   `ISSUE.md`,
   `SUPPORT.md`,
   `.github/ISSUE_TEMPLATE.md`,
   and `.github/PULL_REQUEST_TEMPLATE.md` describe external reports and changes.
   [`CONTRIBUTING.md`][contributing] asks for reproducible bug reports.
   A recent external-contributor change was merged as [pull request 29682][pr-29682].
   No prohibition on AI-assisted reports was found in those files.
5. **Will they likely fix it?**
    No for the behavior reported here.
   [Issue 11530][issue-11530] records the maintainer's intended-behavior decision.
6. **Have we prototyped a minimal fix compatible with their architecture?**
    Not applicable.
   Constraints 1 and 5 fail.
   The verified configuration paths solve the caller's ambiguity without an upstream patch.

There is nothing additive to post on issue 11530.
The existing thread already records the non-TTY failure,
 intended enforcing behavior,
security rationale,
 and fully qualified name workaround.
A version-only comment that the same policy remains in 5.8.4 would amount to a non-additive “still happens” report,
so no upstream comment or new issue should be filed.

[pull-command]:
https://github.com/podman-container-tools/podman/blob/5431df23c742e5edea35bef34eed696f4db0106b/cmd/podman/images/pull.go#L220-L228
[abi-pull]:
https://github.com/podman-container-tools/podman/blob/5431df23c742e5edea35bef34eed696f4db0106b/pkg/domain/infra/abi/images.go#L307-L314
[libimage-resolve]:
https://github.com/podman-container-tools/podman/blob/5431df23c742e5edea35bef34eed696f4db0106b/vendor/go.podman.io/common/libimage/pull.go#L558-L565
[resolver-entry]:
https://github.com/podman-container-tools/podman/blob/5431df23c742e5edea35bef34eed696f4db0106b/vendor/go.podman.io/image/v5/pkg/shortnames/shortnames.go#L243-L264
[resolver-alias]:
https://github.com/podman-container-tools/podman/blob/5431df23c742e5edea35bef34eed696f4db0106b/vendor/go.podman.io/image/v5/pkg/shortnames/shortnames.go#L279-L307
[alias-value]:
https://github.com/podman-container-tools/podman/blob/5431df23c742e5edea35bef34eed696f4db0106b/vendor/go.podman.io/image/v5/pkg/sysregistriesv2/shortnames.go#L213-L230
[resolver-candidates]:
https://github.com/podman-container-tools/podman/blob/5431df23c742e5edea35bef34eed696f4db0106b/vendor/go.podman.io/image/v5/pkg/shortnames/shortnames.go#L312-L332
[resolver-tty]:
https://github.com/podman-container-tools/podman/blob/5431df23c742e5edea35bef34eed696f4db0106b/vendor/go.podman.io/image/v5/pkg/shortnames/shortnames.go#L340-L353
[issue-11530]: https://github.com/podman-container-tools/podman/issues/11530
[issue-11530-closing]: https://github.com/podman-container-tools/podman/issues/11530#issuecomment-917564959
[issue-11530-qualified]: https://github.com/podman-container-tools/podman/issues/11530#issuecomment-917369275
[pr-29682]: https://github.com/podman-container-tools/podman/pull/29682
[contributing]:
https://github.com/podman-container-tools/podman/blob/5431df23c742e5edea35bef34eed696f4db0106b/CONTRIBUTING.md#reporting-issues
