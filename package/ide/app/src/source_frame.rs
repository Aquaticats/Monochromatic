//! Distinguish glyph-image changes from caret-only presentation changes.

/// Visible hint labels and diagnostic marks are paint inputs too.
use crate::annotation::Visible;
/// Annotation inks follow the system scheme.
use crate::annotation_layout::AnnotationColors;
/// Palette colors affect pixels but not caret geometry.
use crate::code_colors::CodeColors;
/// Canonical source revision and selection define source paint inputs.
use crate::document::Document;
/// In-file find ranges are painted from the same shaped rows as selection.
use crate::find::FindRanges;
/// Physical tile geometry determines materialized rows and font scale.
use crate::shaped_text::{Clip, Viewport};
/// Syntax spans belong to the source paint key even when text is unchanged.
use crate::source_style::SourceStyles;
/// Pointer equality avoids walking unchanged classifications on caret-only updates.
use std::sync::Arc;

/// What:
///  Retain every input affecting source pixels,
///  independent of caret position.
/// Why:
///  Reusing an image requires exact paint identity,
///  not a lossy hash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FrameStamp = { revision: number; selection: [number, number]; /* paint inputs */ };
/// ```
pub struct FrameStamp {
    /// Content change sequence within the displayed document.
    revision: u64,
    /// Normalized range;
    ///  all collapsed carets share an empty paint selection.
    selection: (usize, usize),
    /// Materialized rows,
    ///  physical scale,
    ///  and tile width.
    viewport: Viewport,
    /// Foreground and selection colors include current system appearance.
    colors: CodeColors,
    /// Shared immutable classifications prevent copying the whole span list on each caret step.
    styles: SourceStyles,
    /// Shared immutable in-file find ranges;
    ///  empty while the find bar shows nothing.
    matches: FindRanges,
    /// What:
    ///  `Arc<Visible>` shares the hint labels and diagnostic marks of the materialized rows only.
    /// Why:
    ///  A snapshot change outside the materialized rows changes no pixel,
    ///  so it must not repaint.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// annotations: Readonly<Visible>;
    /// ```
    annotations: Arc<Visible>,
    /// What:
    ///  `Option<AnnotationColors>` is the annotation inks,
    ///  or nothing for a frame without annotations.
    /// Why:
    ///  A scheme change re-tints hints and underlines.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// annotationColors?: AnnotationColors;
    /// ```
    annotation_colors: Option<AnnotationColors>,
}

/// Compare exact paint inputs,
///  using shared snapshot identity before comparing new span contents.
impl PartialEq for FrameStamp {
    /// Unchanged source classifications require no per-span equality scan.
    fn eq(&self, other: &Self) -> bool {
        return self.revision == other.revision
            && self.selection == other.selection
            && self.viewport == other.viewport
            && self.colors == other.colors
            && (Arc::ptr_eq(&self.styles, &other.styles) || self.styles == other.styles)
            && (Arc::ptr_eq(&self.matches, &other.matches) || self.matches == other.matches)
            && (Arc::ptr_eq(&self.annotations, &other.annotations)
                || self.annotations == other.annotations)
            && self.annotation_colors == other.annotation_colors;
    }
}

/// Whether two shared values are the same allocation or equal,
/// which skips the comparison for the same one.
fn shared_or_equal<T: PartialEq + ?Sized>(left: &Arc<T>, right: &Arc<T>) -> bool {
    return Arc::ptr_eq(left, right) || left == right;
}

/// Capture immutable paint inputs without treating a collapsed caret as image content.
impl FrameStamp {
    /// What:
    ///  Whether every paint input but the visible annotations is the same in both stamps.
    /// Why:
    ///  Hints and messages follow the wrap width at once,
    /// but a displayed tile keeps the ones it was painted with
    ///  until the tile for the new width arrives,
    /// so only the annotations are expected to differ meanwhile.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// sameExceptAnnotations(other: FrameStamp): boolean;
    /// ```
    pub fn same_except_annotations(&self, other: &Self) -> bool {
        return other.revision == self.revision
            && other.selection == self.selection
            && other.viewport == self.viewport
            && other.colors == self.colors
            && shared_or_equal(&self.styles, &other.styles)
            && shared_or_equal(&self.matches, &other.matches)
            && other.annotation_colors == self.annotation_colors;
    }

    /// Capture image inputs without including a collapsed caret's position.
    /// Reset the cached stamp when replacing the document with a different file.
    pub fn new(
        document: &Document,
        viewport: Viewport,
        colors: CodeColors,
        styles: SourceStyles,
    ) -> Self {
        let position = document.position();
        let mut selection = (0, 0);
        if position.anchor != position.head {
            selection = (
                position.anchor.min(position.head),
                position.anchor.max(position.head),
            );
        }
        // Transfer the immutable shared handle, not a copy of all source classifications.
        return Self {
            revision: document.revision(),
            selection,
            viewport,
            colors,
            styles,
            matches: Arc::from([]),
            annotations: Arc::new(Visible::default()),
            annotation_colors: None,
        };
    }

    /// What:
    ///  Replace the clip of this stamp's viewport.
    /// Why:
    ///  The shaper cuts a frame to the raster limit when the lines are taller than the window's map held,
    ///  and the frame's identity must name the clip it was really painted with.
    ///  The stamp is consumed and returned so existing callers stay unchanged.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// withClip(clip?: Clip): FrameStamp;
    /// ```
    pub fn with_clip(mut self, clip: Option<Clip>) -> Self {
        self.viewport.clip = clip;
        return self;
    }

    /// Replace the in-file find ranges of this stamp;
    ///  a different match list is a different frame.
    /// The stamp is consumed and returned so existing callers without matches stay unchanged.
    pub fn with_matches(mut self, matches: FindRanges) -> Self {
        self.matches = matches;
        return self;
    }

    /// Replace the visible hint labels and diagnostic marks and their inks;
    ///  any difference is a different frame.
    /// The stamp is consumed and returned so existing callers without annotations stay unchanged.
    pub fn with_annotations(mut self, annotations: Arc<Visible>, colors: AnnotationColors) -> Self {
        self.annotations = annotations;
        // `Some` records that this frame paints annotations with these inks.
        self.annotation_colors = Some(colors);
        return self;
    }
}
