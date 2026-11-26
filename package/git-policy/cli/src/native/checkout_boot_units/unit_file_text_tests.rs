//! What:
//!  The one shape rule's controls:
//!  one final newline,
//!  no blank tail,
//!  and a comment that names a file's own path.
//! Why:
//!  three texts are built through this helper,
//! so a change to it is a change to
//!  every file a setup writes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(fileText(['a'])).toBe('a\n');
//! ```

use crate::checkout_boot_units::unit_file_text::{
    file_text,
    path_comment,
};
use std::path::Path;

#[test]
fn one_newline_ends_the_file_and_no_second_one_follows() {
    assert_eq!(
        file_text(vec![String::from("a"), String::from("b")]),
        "a\nb\n"
    );
    assert_eq!(
        file_text(Vec::new()),
        "\n",
        "an empty file is still one line"
    );
}

#[test]
fn the_comment_names_the_path_the_file_belongs_at() {
    assert_eq!(
        path_comment(Path::new("/etc/systemd/system/x.service")),
        "# /etc/systemd/system/x.service"
    );
}
