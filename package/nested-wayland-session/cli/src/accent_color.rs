//! The accent color the private appearance portal serves to hosted toolkits.
//!
//! A color is three 8-bit channels written `RRGGBB`.
//! The portal wants three doubles between 0 and 1,
//!  so the conversion lives here with the parser.

/// The channel count and width of the textual form,
///  `RRGGBB`.
const HEX_DIGITS: usize = 6;

/// The largest value of one 8-bit channel,
///  the divisor that maps a channel into 0 to 1.
const CHANNEL_MAX: f64 = 255.0;

/// Tells whether one byte is an ASCII hexadecimal digit.
fn is_hex_byte(byte: u8) -> bool {
    return byte.is_ascii_hexdigit();
}

/// The one refusal text both the startup option and the control verb show.
fn refusal(text: &str) -> String {
    return format!("must be six hexadecimal digits (RRGGBB), got: {text}");
}

/// Reads the two digits at `start` as one channel.
fn channel(text: &str, start: usize) -> Result<u8, String> {
    let Some(digits) = text.get(start..start + 2) else {
        return Err(refusal(text));
    };
    return match u8::from_str_radix(digits, 16) {
        Ok(value) => Ok(value),
        Err(_) => Err(refusal(text)),
    };
}

/// An opaque color with 8-bit channels.
///
/// What:
///      `pub struct AccentColor { red: u8, green: u8, blue: u8 }`.
/// Why:
///       `--accent-color` and the `accent-color` verb take the same text,
///  and the portal compares colors exactly to know whether a change happened.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccentColor {
    /// Red channel,
    ///  0 to 255.
    red: u8,
    /// Green channel,
    ///  0 to 255.
    green: u8,
    /// Blue channel,
    ///  0 to 255.
    blue: u8,
}

/// Parsing,
///  packing,
///  and the portal's double-precision form.
impl AccentColor {
    /// Parses exactly six hexadecimal digits,
    ///  upper or lower case,
    ///  with no prefix.
    ///
    /// What:
    ///      `pub fn parse(text: &str) -> Result<Self, String>`.
    ///  The error text names the offending value and the accepted form.
    /// Why:
    ///       A `#` prefix or a short form would be a second spelling of the same color,
    ///  and the control verb must refuse what the startup option refuses.
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.len() != HEX_DIGITS || !text.bytes().all(is_hex_byte) {
            return Err(refusal(text));
        }
        return Ok(Self {
            red: channel(text, 0)?,
            green: channel(text, 2)?,
            blue: channel(text, 4)?,
        });
    }

    /// Packs the color into one `0x00RRGGBB` integer for an atomic cell.
    #[must_use]
    pub fn pack(self) -> u32 {
        return (u32::from(self.red) << 16) | (u32::from(self.green) << 8) | u32::from(self.blue);
    }

    /// Restores a color from `pack`;
    ///  bits above the blue channel's byte pair are ignored.
    #[must_use]
    pub fn unpack(packed: u32) -> Self {
        return Self {
            red: ((packed >> 16) & 0xFF) as u8,
            green: ((packed >> 8) & 0xFF) as u8,
            blue: (packed & 0xFF) as u8,
        };
    }

    /// Returns the `(ddd)` triple the XDG Settings portal defines for `accent-color`.
    ///
    /// What:
    ///      `pub fn portal_value(self) -> (f64, f64, f64)`.
    ///  Each channel divided by 255,
    ///  so `FF0000` is `(1.0, 0.0, 0.0)`.
    /// Why:
    ///       Slint reads the setting as three doubles in `[0, 1]` and rebuilds an 8-bit color from them.
    #[must_use]
    pub fn portal_value(self) -> (f64, f64, f64) {
        return (
            f64::from(self.red) / CHANNEL_MAX,
            f64::from(self.green) / CHANNEL_MAX,
            f64::from(self.blue) / CHANNEL_MAX,
        );
    }
}

/// Writes the color the way `--accent-color` reads it.
impl std::fmt::Display for AccentColor {
    /// Formats as six uppercase hexadecimal digits.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(formatter, "{:02X}{:02X}{:02X}", self.red, self.green, self.blue);
    }
}

/// Unit tests for the color grammar and its portal form.
#[cfg(test)]
#[path = "accent_color_tests.rs"]
mod tests;
