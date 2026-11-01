//! What:
//!  Fixed-vector controls for the handwritten SHA-256.
//! Why:
//!  Expected digests were measured independently with Node's `createHash('sha256')`,
//! and the first three are also the published FIPS 180-4 examples.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(sha256Hex(bytes)).toBe(createHash('sha256').update(bytes).digest('hex'));
//! ```

/// Import the digest and both round functions under test.
use super::{choose, majority, sha256_hex};

/// Both round functions equal their FIPS 180-4 definitions on every combination of three input bits:
/// each of the eight combinations appears at one bit position of these three words.
#[test]
fn choose_and_majority_match_their_fips_definitions() {
    let x: u32 = 0xf0f0_f0f0;
    let y: u32 = 0xcccc_cccc;
    let z: u32 = 0xaaaa_aaaa;
    assert_eq!(choose(x, y, z), (x & y) ^ (!x & z));
    assert_eq!(choose(x, y, z), 0xcaca_caca);
    assert_eq!(majority(x, y, z), (x & y) ^ (x & z) ^ (y & z));
    assert_eq!(majority(x, y, z), 0xe8e8_e8e8);
}

/// Published examples:
///  the empty message,
///  `abc`,
///  and the 448-bit two-block message.
#[test]
fn published_examples_match() {
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

/// Every padding branch:
///  remainder below,
///  at and above the 56-byte limit,
///  and exact block multiples.
#[test]
fn padding_boundaries_match_measured_digests() {
    let cases: [(usize, &str); 10] = [
        (
            1,
            "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb",
        ),
        (
            55,
            "9f4390f8d30c2dd92ec9f095b65e2b9ae9b0a925a5258e241c9f1e910f734318",
        ),
        (
            56,
            "b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a",
        ),
        (
            57,
            "f13b2d724659eb3bf47f2dd6af1accc87b81f09f59f2b75e5c0bed6589dfe8c6",
        ),
        (
            63,
            "7d3e74a05d7db15bce4ad9ec0658ea98e3f06eeecf16b4c6fff2da457ddc2f34",
        ),
        (
            64,
            "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb",
        ),
        (
            65,
            "635361c48bb9eab14198e76ea8ab7f1a41685d6ad62aa9146d301d4f17eb0ae0",
        ),
        (
            119,
            "31eba51c313a5c08226adf18d4a359cfdfd8d2e816b13f4af952f7ea6584dcfb",
        ),
        (
            120,
            "2f3d335432c70b580af0e8e1b3674a7c020d683aa5f73aaaedfdc55af904c21c",
        ),
        (
            128,
            "6836cf13bac400e9105071cd6af47084dfacad4e5e302c94bfed24e013afb73e",
        ),
    ];
    for (length, expected) in cases {
        let bytes: Vec<u8> = vec![b'a'; length];
        assert_eq!(sha256_hex(bytes.as_slice()), expected, "length {length}");
    }
}

/// Every byte value participates,
///  and the million-byte published vector crosses many blocks.
#[test]
fn every_byte_value_and_a_long_message_match() {
    let mut all: Vec<u8> = Vec::<u8>::new();
    for value in 0..=255_u8 {
        all.push(value);
    }
    assert_eq!(
        sha256_hex(all.as_slice()),
        "40aff2e9d2d8922e47afd4648e6967497158785fbd1da870e7110266bf944880"
    );
    let million: Vec<u8> = vec![b'a'; 1_000_000];
    assert_eq!(
        sha256_hex(million.as_slice()),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
    assert_eq!(
        sha256_hex(b"image bytes"),
        "de7030234493a8bea844dbe1d8676e68a2c1a4b014c721f0425a22b6df66faec"
    );
}
