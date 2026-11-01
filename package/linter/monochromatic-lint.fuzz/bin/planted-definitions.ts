/**
 The inputs and the planted defects of the fuzz invariant controls.
 */

/**
 One text replacement of a planted defect.
 */
export type PlantedEdit = {
  readonly from: string;
  readonly to: string;
};

/**
 One planted defect: the file it edits and the edits.
 */
export type Plant = {
  readonly name: string;
  readonly file: string;
  readonly edits: readonly PlantedEdit[];
};

/**
 Repository-relative inputs the helper library needs to build.
 */
export const inputs: readonly string[] = [
  'package/linter/monochromatic-lint/Cargo.toml',
  'package/linter/monochromatic-lint/Cargo.lock',
  'package/linter/monochromatic-lint/README.md',
  'package/linter/monochromatic-lint/src',
  'package/linter/monochromatic-lint/fixtures',
  'package/linter/monochromatic-lint.fuzz/Cargo.toml',
  'package/linter/monochromatic-lint.fuzz/Cargo.lock',
  'package/linter/monochromatic-lint.fuzz/src',
  'package/linter/monochromatic-lint.fuzz/fuzz_targets',
  'package/jsonc-edit/core-rust/Cargo.toml',
  'package/jsonc-edit/core-rust/Cargo.lock',
  'package/jsonc-edit/core-rust/src',
  'package/jsonc-edit/core-rust/fixtures',
  'package/jsonc-edit/core-rust.fuzz/Cargo.toml',
  'package/jsonc-edit/core-rust.fuzz/Cargo.lock',
  'package/jsonc-edit/core-rust.fuzz/src',
  'package/jsonc-edit/core-rust.fuzz/fuzz_targets',
];

/**
 Defects with observable behavior, one per fuzz target.
 Each `from` must occur exactly once so a control cannot silently stop applying.
 */
export const plants: readonly Plant[] = [
  {
    name: 'configuration: the warn severity is rejected',
    file: 'package/linter/monochromatic-lint/src/configuration_rules.rs',
    edits: [
      {
        from: 'if severity != "off" && severity != "warn" && severity != "error" {',
        to: 'if severity != "off" && severity != "error" {',
      },
    ],
  },
  {
    name: 'merge_values: a later array replaces an earlier one instead of concatenating',
    file: 'package/linter/monochromatic-lint/src/config_merge.rs',
    edits: [
      {
        from: 'merged.extend_from_slice(elements);',
        to: 'merged = elements.to_vec();',
      },
    ],
  },
  {
    name: 'rust_style: a file named like a sidecar is still reported',
    file: 'package/linter/monochromatic-lint/src/rust_tests_in_sidecars.rs',
    edits: [
      {
        from: 'stem == "tests" || stem.ends_with("_tests")',
        to: 'stem == "tests"',
      },
    ],
  },
  {
    name: 'rust_explicit_types: an initialized binding needs no annotation',
    file: 'package/linter/monochromatic-lint/src/rust_explicit_declarations.rs',
    edits: [
      {
        from: 'if statement.ty().is_some() {',
        to: 'if statement.ty().is_some() || statement.initializer().is_some() {',
      },
    ],
  },
  {
    name: 'markdown: a trailing colon is not heading punctuation',
    file: 'package/linter/monochromatic-lint/src/markdown_punctuation.rs',
    edits: [
      {
        from: "if character != '.' && character != ':' {",
        to: "if character != '.' {",
      },
    ],
  },
  {
    name: 'orchestration: an unlabeled rustdoc fence is not a doc test',
    file: 'package/linter/monochromatic-lint/src/processors_fences.rs',
    edits: [
      {
        from: '            && !(rustdoc && language.is_empty())\n',
        to: '',
      },
    ],
  },
];
