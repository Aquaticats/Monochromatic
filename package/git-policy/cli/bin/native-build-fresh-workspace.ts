/**
 What the build step reads from `cargo metadata`: the workspace root, the two directories Cargo builds in and puts
 programs in, and the members of that workspace.

 Cargo's own answer is the only source for these, because the build step must not decide a directory itself: a person's
 `~/.cargo/config.toml` keys the build directory by workspace path, cargo-shadow keys it by toolchain as well
 (`package/cargo-shadow/cli/src/lean_keyed_folder.rs`), and `CARGO_TARGET_DIR` moves the target directory. What a step
 that guessed would name is a program nobody built.

 In TS you'd write (pseudocode):
 ```ts
 // ask Cargo, read four fields, find this crate's member by its manifest
 ```
 */

/**
 One workspace member as `cargo metadata` describes it.
 */
export type CargoPackage = {
  /**
   The package's name, which is what `cargo clean --package` takes.
   */
  readonly name: string;
  /**
   The package's manifest, which is how this crate is told from its siblings.
   */
  readonly manifestPath: string;
};

/**
 The four facts this step uses out of `cargo metadata`.
 */
export type CargoWorkspaceMetadata = {
  /**
   The workspace root, named in the one line the build prints on success.
   */
  readonly workspaceRoot: string;
  /**
   Where final artifacts go.
   */
  readonly targetDirectory: string;
  /**
   Where intermediate build state goes, when this Cargo names it at all.
   */
  readonly buildDirectory?: string;
  /**
   The workspace's members.
   */
  readonly packages: readonly CargoPackage[];
};

/**
 Reads a field this step expects to be text.

 @param value - the value read from Cargo's JSON

 @param field - the field's name, for the failure that names it

 @returns the text

 @throws Error when the field is absent or is not text

 @example
 ```ts
 const root = textField({ value: seen.workspace_root, field: 'workspace_root' });
 ```
 */
function textField({
  value,
  field,
}: {
  readonly value: unknown;
  readonly field: string;
}): string {
  if ((typeof value) !== 'string')
    throw new Error(`cargo metadata named no ${field}, so this build cannot say where its program belongs`);
  return value;
}

/**
 Reads one member of Cargo's `packages` array, as a list of one member or none.

 Why a list: this repository models absence as an empty list rather than a union with `undefined`.

 @param value - the entry read from Cargo's JSON

 @returns the member, or an empty list when the entry is not one

 @example
 ```ts
 const members = listed.flatMap(memberOrNothing);
 ```
 */
export function memberOrNothing(value: unknown): readonly CargoPackage[] {
  if ((typeof value) !== 'object') return [];
  if (value === null) return [];
  /**
   The entry's own name, which a JSON object holds by key.
   */
  const name: unknown = Reflect.get(
    value,
    'name',
  );
  /**
   The entry's manifest, which is the other field this step reads.
   */
  const manifest: unknown = Reflect.get(
    value,
    'manifest_path',
  );
  if ((typeof name) !== 'string') return [];
  if ((typeof manifest) !== 'string') return [];
  return [{
    name,
    manifestPath: manifest,
  }];
}

/**
 Reads `cargo metadata`'s answer to the facts this step uses.

 @param printed - what the metadata command wrote on standard output

 @returns the facts

 @throws Error when the answer is not the JSON this step reads

 @example
 ```ts
 const seen = workspaceOf(described.stdout);
 ```
 */
export function workspaceOf(printed: string): CargoWorkspaceMetadata {
  /**
   The parsed answer, which is Cargo's own JSON.
   */
  const parsed = parsedOrThrow(printed,);
  /**
   The members Cargo named, laid out one entry per element.
   */
  const listed: unknown = Reflect.get(
    parsed,
    'packages',
  );
  if (!Array.isArray(listed))
    throw new Error('cargo metadata named no packages, so this build cannot tell this crate from its siblings');
  /**
   The build directory as a field Cargo may leave out, which `exactOptionalPropertyTypes` reads as absent or text.
   */
  const buildDirectory: unknown = Reflect.get(
    parsed,
    'build_directory',
  );
  return {
    workspaceRoot: textField({
      value: Reflect.get(
        parsed,
        'workspace_root',
      ),
      field: 'workspace_root',
    },),
    targetDirectory: textField({
      value: Reflect.get(
        parsed,
        'target_directory',
      ),
      field: 'target_directory',
    },),
    ...((typeof buildDirectory) === 'string' ? { buildDirectory } : {}),
    packages: listed.flatMap(memberOrNothing),
  };
}

/**
 Parses Cargo's text as one JSON object.

 @param printed - what the command wrote

 @returns the object the text holds

 @throws Error when the text is not JSON, is not an object, or is JSON's `null`

 @example
 ```ts
 const parsed = parsedOrThrow('{"workspace_root":"/repo"}');
 ```
 */
function parsedOrThrow(printed: string): object {
  /**
   The value the text holds, when it holds one.
   */
  const parsed: unknown = JSON.parse(printed,);
  if ((typeof parsed) !== 'object')
    throw new Error('cargo metadata wrote an answer that is not an object, so this build cannot say where its program belongs');
  if (parsed === null)
    throw new Error('cargo metadata wrote JSON\'s null, so this build cannot say where its program belongs');
  return parsed;
}

/**
 Finds this crate among the workspace's members, by the manifest beside the directory the step builds in.

 @param seen - the workspace Cargo described

 @param packageDirectory - the directory the step builds in

 @returns the member's package name

 @throws Error when no member holds this directory's own manifest

 @example
 ```ts
 const member = memberOf({ seen, packageDirectory: '/repo/package/git-policy/cli' });
 ```
 */
export function memberOf({
  seen,
  packageDirectory,
}: {
  readonly seen: CargoWorkspaceMetadata;
  readonly packageDirectory: string;
}): string {
  /**
   The manifest this step expects, as a path Cargo would name it.
   */
  const wanted = `${packageDirectory}/Cargo.toml`;
  /**
   The member whose manifest is that one.
   */
  const found = seen.packages
    .find(function holdsTheManifest(entry: CargoPackage): boolean {
      return entry.manifestPath === wanted;
    },);
  if (found === undefined) {
    throw new Error(
      `no member of the workspace at ${seen.workspaceRoot} holds the manifest ${wanted}, so this build cannot say which crate to clean and build`,
    );
  }
  return found.name;
}
