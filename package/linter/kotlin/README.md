# kotlin-linter

Monorepo-wide [detekt][] rule set enforcing KDoc on every Kotlin declaration,
the Kotlin counterpart of the repo's `require-tsdoc` oxlint rule for TypeScript.
It is the Kotlin sibling of `package/linter/monochromatic-lint`:
a standalone linter run over all of `package/` by the root `lint:detekt` task.

## Why this exists

detekt ships `UndocumentedPublicClass`,
`UndocumentedPublicFunction`,
and `UndocumentedPublicProperty`,
but they only cover the public API surface and cannot be configured down to private or local declarations.
The repo's documentation standard is "everywhere",
so a custom rule is required.

## The rule contract

`RequireKDoc` reports any declaration of a covered kind that has no preceding KDoc
(`KtDeclaration.docComment == null`).

Covered kinds:

- classes,
  interfaces,
  annotation classes (`KtClass`)
- objects and companion objects (`KtObjectDeclaration`)
- named functions,
  including member and local ones (`KtNamedFunction`)
- properties,
  including member and local `val`/`var` (`KtProperty`)
- secondary constructors (`KtSecondaryConstructor`)
- type aliases (`KtTypeAlias`)
- enum entries (`KtEnumEntry`)

Deliberately skipped,
mirroring `require-tsdoc`:

- parameters (documented via the owner's `@param`)
- primary constructors (documented by the class KDoc)
- property accessors (documented by the property)
- anonymous object literals,
  destructuring entries,
  and `init` blocks

The implementation overrides a single choke point,
`visitDeclaration`,
because every declaration kind delegates to it in the PSI visitor chain.
The `super` call preserves the tree walk into bodies,
which is what reaches local and nested declarations.

## How it runs monorepo-wide

The root `lint:detekt` task (in `mise.toml`,
part of the `lint` aggregate) invokes the detekt CLI over `package/`
with this module's jar on `--plugins`.
Detekt's default config stays active,
and this package adds `require-kdoc` on top:

```sh
mise run lint:detekt    # scan all of package/
```

Global excludes drop build output,
`.gradle` caches,
build scripts (`.kts`),
and test sources (`**/src/test/**`,
`**/src/androidTest/**`) before parsing.

## Configuration

`detekt.yml` activates `require-kdoc` and leaves findings at detekt 2's default `error` severity,
so missing KDoc fails the run.
See the [default-config troubleshooting note](../../../doc/troubleshooting/detekt-default-config.md)
for the source trace.
The `allowOverride` rule option (default false) can let `override` members inherit documentation.

## Package tasks

```sh
mise run //package/linter/kotlin:test           # rule unit tests
mise run //package/linter/kotlin:lint           # local compile check
mise run //package/linter/kotlin:lint:detekt    # dogfood own src
mise run //package/linter/kotlin:build              # build plugin jar
mise run //package/linter/kotlin:publish:bundle     # signed Central bundle, no upload
mise run //package/linter/kotlin:verify:publication # bundle plus clean-consumer resolution
mise run //package/linter/kotlin:verify:central     # current release through detekt from Central
```

## Maven Central publication

The published coordinates are
`cat.aquati.monochromatic:detekt-rules:<version>`.
The release version is the `version` entry in `gradle.properties`.

A push to `main` that changes that value runs
`.github/workflows/kotlin-linter-publish.yml`.
The workflow:

- builds the compiled,
  sources,
  and documentation JARs;
- generates the required POM metadata;
- signs every published artifact with OpenPGP;
- verifies MD5,
  SHA-1,
  SHA-256,
  and SHA-512 checksums;
- uploads one Maven-layout ZIP through Sonatype's Portal Publisher API with `AUTOMATIC` publishing;
- waits until Sonatype reports `PUBLISHED` or `FAILED`.

Maven Central versions are immutable.
Change `gradle.properties` to a version that has never been published.
Changing another line in that file without changing `version` does not publish.
If GitHub cannot read the pre-push version,
the workflow skips automatic publication and emits a warning;
use a manual non-dry run after confirming the intended version.

Use the `kotlin-linter-publish` workflow's **Run workflow** control for manual operation.
Its `dry-run` input defaults to `true`,
which builds and validates without contacting Sonatype.
Set `dry-run` to `false` to retry a release upload manually.

The repository's GitHub Actions secrets are:

- `MAVEN_CENTRAL_USERNAME`:
  Sonatype Portal user-token username;
- `MAVEN_CENTRAL_PASSWORD`:
  Sonatype Portal user-token password;
- `MAVEN_SIGNING_KEY`:
  ASCII-armored private OpenPGP key;
- `MAVEN_SIGNING_PASSWORD`:
  private-key passphrase.

The matching public key has fingerprint
`5BB5 727B E92B 6283 BBD7 DA85 5E8E 6D8D 791B 1B45`
and is published on `keyserver.ubuntu.com`.
It expires on `2028-08-15`.
Extend or replace it before that date,
republish the public key,
and update both `MAVEN_SIGNING_*` secrets together.
A local recovery copy is held outside the repository.
Its passphrase is stored separately in the desktop secret service.

Sonatype's current requirements and API contract are documented in:

- <https://central.sonatype.org/publish/requirements/>;
- <https://central.sonatype.org/publish/publish-portal-api/>.

## Adding a rule

Add the rule class beside `RequireKDoc.kt`,
then register its factory in the map inside `KdocRuleSetProvider.kt`.
The `dev.detekt.api.RuleSetProvider` service file under `src/main/resources/META-INF/services/`
only needs the provider,
not individual rules.

[detekt]: https://detekt.dev
