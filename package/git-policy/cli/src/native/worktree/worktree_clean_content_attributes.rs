//! What:
//!  The attribute files a content decision reads:
//! which of them apply to one path,
//! and what the decision does with the answer.
//! Why:
//!  `worktree_clean_content.rs` decides whether a file equals its committed bytes,
//! and that decision may only trust the
//!  bytes when no attribute Git applies converts them;
//! the decision itself lives here so it reads as the question it answers,
//! and the reading of the files lives in the siblings this file re-exports.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const judge = ContentJudge.new(...); const applies = judge.convertingAttribute(path, parents);
//! ```

use crate::fs_beneath_walk_chain::open_directory_beneath;
use crate::worktree_attributes::may_apply;
use crate::worktree_clean_content::ContentJudge;
use crate::worktree_clean_content_attribute_sources::{
    AttributeFile,
    FixedSources,
    first_converting_token,
    has_quoted_line,
};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

impl<'a> ContentJudge<'a> {
    /// The `.gitattributes` of the directory below the root named by `parents`,
    /// read once.
    pub(super) fn directory_file(
        &mut self,
        parents: &[OsString],
    ) -> Result<Option<AttributeFile>, String> {
        let mut names: Vec<Vec<u8>> = Vec::new();
        for name in parents {
            names.push(name.as_bytes().to_vec());
        }
        let key: Vec<u8> = names.join(&b'/');
        if !self.directories.contains_key(&key) {
            let directory = match open_directory_beneath(self.root, parents) {
                Ok(opened) => opened,
                Err(error) => {
                    return Err(format!(
                        "{} cannot be opened below the worktree ({error})",
                        String::from_utf8_lossy(key.as_slice())
                    ));
                }
            };
            let label: String =
                format!("{}/.gitattributes", String::from_utf8_lossy(key.as_slice()));
            let file: Option<AttributeFile> = self.read_in(&directory, label, key.clone())?;
            self.directories.insert(key.clone(), file);
        }
        return Ok(self.directories.get(&key).cloned().flatten());
    }

    /// The first converting attribute that may apply to the file at `path` and where it is set,
    /// or nothing.
    pub(super) fn converting_attribute(
        &mut self,
        path: &[u8],
        parents: &[OsString],
    ) -> Result<Option<(String, String)>, String> {
        if self.fixed.is_none() {
            self.fixed = Some(self.load_fixed());
        }
        let loaded: Result<FixedSources, String> = match self.fixed.clone() {
            Some(found) => found,
            None => return Err(String::from("the attributes were not loaded")),
        };
        let fixed: FixedSources = loaded?;
        let mut files: Vec<AttributeFile> = fixed.files;
        for depth in 1..=parents.len() {
            if let Some(file) = self.directory_file(&parents[..depth])? {
                files.push(file);
            }
        }
        for file in &files {
            let below_base: bool = match path.strip_prefix(file.base.as_slice()) {
                Some(rest) => rest.starts_with(b"/"),
                None => false,
            };
            let below: bool = file.base.is_empty() || below_base;
            if below && has_quoted_line(file.lines.as_slice()) {
                return Ok(Some((String::from("a quoted line"), file.label.clone())));
            }
            for line in &file.lines {
                if line.is_macro {
                    continue;
                }
                let Some(token) = first_converting_token(line.tokens.as_slice(), &fixed.macros)
                else {
                    continue;
                };
                if may_apply(line.subject.as_slice(), file.base.as_slice(), path) {
                    return Ok(Some((
                        String::from_utf8_lossy(token.as_slice()).into_owned(),
                        file.label.clone(),
                    )));
                }
            }
        }
        return Ok(None);
    }
}

/// Controls for the attribute reading stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clean_content_attributes_tests.rs"]
mod tests;
