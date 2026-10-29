//! License and notice selection,
//!  headings,
//!  crate entries,
//!  and damage,
//!  on small tables sorted as the
//! build writes them.

use super::{collect, is_notice, write};
use crate::{
    content_digest::fnv1a,
    runtime::embedded::{EmbeddedFile, EmbeddedRuntime},
};

/// A crate license list with one entry,
///  as the `notices` task writes it.
const CRATES: &[u8] = br#"{"licenses": [
{"id": "MIT", "name": "MIT License", "source": "/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anyhow-1.0.104/LICENSE-MIT", "crates": [{"name": "anyhow", "version": "1.0.104"}], "text": "Copyright (c) The anyhow authors\n"}
]}"#;

/// One file of each kind the executable embeds,
///  sorted by path.
static FILES: [EmbeddedFile; 10] = [
    EmbeddedFile {
        path: "LICENSES/LGPL-3.0-or-later.txt",
        bytes: b"GNU LESSER GENERAL PUBLIC LICENSE\n",
        digest: fnv1a(b"GNU LESSER GENERAL PUBLIC LICENSE\n"),
    },
    EmbeddedFile {
        path: "LICENSES/crates.json",
        bytes: CRATES,
        digest: fnv1a(CRATES),
    },
    EmbeddedFile {
        path: "LICENSES/font/Inter-LICENSE.txt",
        bytes: b"SIL OPEN FONT LICENSE",
        digest: fnv1a(b"SIL OPEN FONT LICENSE"),
    },
    EmbeddedFile {
        path: "runtime/Helix-LICENSE",
        bytes: b"Mozilla Public License Version 2.0\n",
        digest: fnv1a(b"Mozilla Public License Version 2.0\n"),
    },
    EmbeddedFile {
        path: "runtime/grammars/sql.so",
        bytes: b"parser",
        digest: fnv1a(b"parser"),
    },
    EmbeddedFile {
        path: "runtime/licenses/slint/REUSE-headers.txt",
        bytes: b"Copyright Slint\n",
        digest: fnv1a(b"Copyright Slint\n"),
    },
    EmbeddedFile {
        path: "runtime/licenses/sql/LICENSE",
        bytes: b"MIT License\n",
        digest: fnv1a(b"MIT License\n"),
    },
    EmbeddedFile {
        path: "runtime/manifest.json",
        bytes: b"{}",
        digest: fnv1a(b"{}"),
    },
    EmbeddedFile {
        path: "runtime/queries/ripple/readme.md",
        bytes: b"Taken from a repository\n",
        digest: fnv1a(b"Taken from a repository\n"),
    },
    EmbeddedFile {
        path: "runtime/queries/snakemake/LICENSE",
        bytes: b"Copyright (c) 2023\n",
        digest: fnv1a(b"Copyright (c) 2023\n"),
    },
];

/// The table of [`FILES`].
static RUNTIME: EmbeddedRuntime = EmbeddedRuntime {
    key: "0123456789abcdef",
    files: &FILES,
};

/// A notice whose bytes no longer match the recorded digest.
static DAMAGED: [EmbeddedFile; 1] = [EmbeddedFile {
    path: "runtime/licenses/sql/LICENSE",
    bytes: b"MIT Licensf\n",
    digest: fnv1a(b"MIT License\n"),
}];

/// The table of [`DAMAGED`].
static DAMAGED_RUNTIME: EmbeddedRuntime = EmbeddedRuntime {
    key: "0123456789abcdef",
    files: &DAMAGED,
};

/// A table holding license files but no crate license list.
static WITHOUT_CRATES: [EmbeddedFile; 1] = [EmbeddedFile {
    path: "runtime/licenses/sql/LICENSE",
    bytes: b"MIT License\n",
    digest: fnv1a(b"MIT License\n"),
}];

/// The table of [`WITHOUT_CRATES`].
static WITHOUT_CRATES_RUNTIME: EmbeddedRuntime = EmbeddedRuntime {
    key: "0123456789abcdef",
    files: &WITHOUT_CRATES,
};

#[test]
fn license_folders_and_license_named_files_are_notices_and_nothing_else_is() {
    let mut selected: Vec<&str> = Vec::new();
    for file in FILES.iter() {
        if is_notice(file.path) {
            selected.push(file.path);
        }
    }
    assert_eq!(
        selected,
        vec![
            "LICENSES/LGPL-3.0-or-later.txt",
            "LICENSES/font/Inter-LICENSE.txt",
            "runtime/Helix-LICENSE",
            "runtime/licenses/slint/REUSE-headers.txt",
            "runtime/licenses/sql/LICENSE",
            "runtime/queries/snakemake/LICENSE",
        ]
    );
    assert!(is_notice("runtime/queries/demo/COPYING"));
    assert!(is_notice("runtime/queries/demo/Notice.txt"));
    assert!(is_notice("runtime/queries/demo/licence.md"));
    assert!(!is_notice(
        "runtime/queries/licenses-and-more/highlights.scm"
    ));
}

#[test]
fn every_text_is_printed_in_full_under_a_heading_naming_its_component_and_path() {
    let notices = collect(&RUNTIME).expect("intact table");
    let mut out = Vec::new();
    write(&notices, "9.9.9", &mut out).expect("write to memory");
    let text = String::from_utf8(out).expect("UTF-8 output");
    assert!(
        text.starts_with(
            "Monochromatic IDE 9.9.9 carries these 7 license and notice texts, each in full below.\n"
        ),
        "{text}"
    );
    for (heading, path, body) in [
        (
            "Monochromatic IDE: LGPL-3.0-or-later.txt",
            "LICENSES/LGPL-3.0-or-later.txt",
            "GNU LESSER GENERAL PUBLIC LICENSE\n",
        ),
        (
            "Font compiled into the application: Inter-LICENSE.txt",
            "LICENSES/font/Inter-LICENSE.txt",
            "SIL OPEN FONT LICENSE\n",
        ),
        (
            "Helix (highlighting queries and the Helix crates compiled in): Helix-LICENSE",
            "runtime/Helix-LICENSE",
            "Mozilla Public License Version 2.0\n",
        ),
        (
            "Language grammar slint: REUSE-headers.txt",
            "runtime/licenses/slint/REUSE-headers.txt",
            "Copyright Slint\n",
        ),
        (
            "Language grammar sql: LICENSE",
            "runtime/licenses/sql/LICENSE",
            "MIT License\n",
        ),
        (
            "Helix highlighting queries for snakemake: LICENSE",
            "runtime/queries/snakemake/LICENSE",
            "Copyright (c) 2023\n",
        ),
    ] {
        let rule = "=".repeat(78);
        let block = format!("\n{rule}\n{heading}\nEmbedded as {path}\n{rule}\n\n{body}");
        assert!(text.contains(&block), "missing block for {path}:\n{text}");
    }
    let rule = "=".repeat(78);
    let crate_block = format!(
        "\n{rule}\nRust crates under MIT License (MIT): 1 crate\nUsed by: anyhow 1.0.104\nText from the crate file anyhow-1.0.104/LICENSE-MIT\n{rule}\n\nCopyright (c) The anyhow authors\n"
    );
    assert!(
        text.contains(&crate_block),
        "missing the crate block:\n{text}"
    );
    assert!(
        !text.contains("Embedded as LICENSES/crates.json"),
        "the raw list was printed"
    );
    assert!(!text.contains("Taken from a repository"));
    assert!(!text.contains("parser"));
}

#[test]
fn a_damaged_text_prints_nothing_and_names_the_file_and_the_remedy() {
    let error = collect(&DAMAGED_RUNTIME).expect_err("damaged notice");
    let message = format!("{error:#}");
    assert!(
        message.contains("runtime/licenses/sql/LICENSE"),
        "{message}"
    );
    assert!(message.contains("is damaged"), "{message}");
    assert!(message.contains("fresh copy"), "{message}");
}

#[test]
fn an_executable_without_the_crate_list_is_reported_instead_of_printing_a_partial_list() {
    let error = collect(&WITHOUT_CRATES_RUNTIME).expect_err("no crate list");
    let message = format!("{error:#}");
    assert!(message.contains("LICENSES/crates.json"), "{message}");
    assert!(message.contains("fresh copy"), "{message}");
}
