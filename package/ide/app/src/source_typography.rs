//! Immutable source typography selects real variable font instances and real italic faces.

/// Invalid font requests fail visibly rather than creating non-finite layout coordinates.
use anyhow::{Result, bail};
/// Typed features preserve OpenType tags and values without an application-specific string syntax.
use parley::{FontFeature, setting::Tag};

/// Font choices retained for one shaper;
///  creating a new shaper invalidates native frame state.
#[derive(Clone)]
pub struct SourceTypography {
    /// Continuous JetBrains Mono weight within the bundled axis range,
    ///  not a static-face preset.
    pub weight: f32,
    /// Choose the real italic font file,
    ///  rather than skewing roman outlines.
    pub italic: bool,
    /// Additional OpenType choices;
    ///  defaults enable programming ligatures through calt.
    pub features: Vec<FontFeature>,
}

/// Normal source typography keeps the font's other stylistic choices at their defaults.
impl Default for SourceTypography {
    /// Use the regular weight and explicit programming-ligature policy.
    fn default() -> Self {
        return Self {
            weight: 400.0,
            italic: false,
            features: vec![FontFeature::new(Tag::new(b"calt"), 1)],
        };
    }
}

/// Validate settings before any layout or cache identity is created.
impl SourceTypography {
    /// The packaged JetBrains Mono files advertise wght 100 through 800.
    pub fn validate(&self) -> Result<()> {
        if !self.weight.is_finite() || !(100.0..=800.0).contains(&self.weight) {
            bail!(
                "Source font weight {} is outside the bundled JetBrains Mono range 100 to 800",
                self.weight
            );
        }
        return Ok(());
    }
}
