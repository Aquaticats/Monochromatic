# Symphonia 0.6.0 rejects MP4 HE-AAC/SBR during true-peak measurement with `aac too complex`

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Symptom

The music-player true-peak benchmark measured the library with the desktop app decoder path and reported one
failed file:

```text
# /tmp/agent/music-player-truepeak-bench/full-out/summary.json
/home/user/Seafile/Plain/Music/Fate/Super ☆ Affection.mp4: decode error: unsupported feature: aac: aac too complex
```

A narrower reproduction over only the `Fate` folder produced the same single failure:

```text
# /tmp/agent/music-player-truepeak-bench/fate-out/summary.json
{
  "files_found": 58,
  "files_measured": 57,
  "decode_errors": 1,
  "error_examples": [
    "/home/user/Seafile/Plain/Music/Fate/Super ☆ Affection.mp4: decode error: unsupported feature: aac: aac too complex"
  ]
}
```

`ffprobe` identifies the file's audio stream as HE-AAC:

```json
// ffprobe -hide_banner -v error -select_streams a:0 \
//   -show_entries stream=index,codec_name,profile,mime_codec_string,sample_rate,channels,extradata_size \
//   -print_format json -- "/home/user/Seafile/Plain/Music/Fate/Super ☆ Affection.mp4"
{
  "streams": [
    {
      "index": 1,
      "codec_name": "aac",
      "profile": "HE-AAC",
      "mime_codec_string": "mp4a.40.5",
      "sample_rate": "44100",
      "channels": 2,
      "extradata_size": 4
    }
  ]
}
```

## Root cause

The app selects the first known audio track and sends non-Opus audio to Symphonia.
That part is not the failure.
`package/music-player/desktop-app/src/decode.rs:388-400` selects the audio track:

```rust
// package/music-player/desktop-app/src/decode.rs:388-400
let track = format
    .first_track_known_codec(TrackType::Audio)
    .ok_or_else(|| PlayerError::Unsupported("no audio track".to_string()))?;
```

`package/music-player/desktop-app/src/decode.rs:415-435` reads the audio codec parameters and records whether
this is Opus:

```rust
// package/music-player/desktop-app/src/decode.rs:415-435
let audio_params = track
    .codec_params
    .as_ref()
    .and_then(|cp| cp.audio())
    .ok_or_else(|| {
        PlayerError::Unsupported("track has no audio codec parameters".to_string())
    })?;

(track.id, audio_params.codec == CODEC_ID_OPUS, track.clone())
```

`package/music-player/desktop-app/src/decode.rs:446-463` routes non-Opus audio through `SymphoniaSource`:

```rust
// package/music-player/desktop-app/src/decode.rs:446-463
if is_opus {
    let source = OpusSource::new(format, track, track_id)?;
    Ok(Box::new(source))
} else {
    let source = SymphoniaSource::new(format, track, track_id)?;
    Ok(Box::new(source))
}
```

The MP4 demuxer preserves the AAC `esds` extra data that identifies HE-AAC.
`symphonia-format-isomp4 0.6.0 src/atoms/esds.rs:77-84` parses the MPEG-4 audio config and stores the same
extra data on the sample entry:

```rust
// symphonia-format-isomp4-0.6.0/src/atoms/esds.rs:77-84
if let Some(ds_config) = self.es_desc.dec_config.dec_specific_info {
    // Try to read the audio specific configuration and populate the audio sample entry.
    if let Ok(asc) = AudioSpecificConfig::read(&ds_config.extra_data) {
        entry.profile = get_audio_codec_profile(&asc);
        entry.channels = asc.channels;
    }

    entry.extra_data = Some(ds_config.extra_data);
}
```

`symphonia-format-isomp4 0.6.0 src/atoms/stsd.rs:157-168` copies that extra data into
`AudioCodecParameters`:

```rust
// symphonia-format-isomp4-0.6.0/src/atoms/stsd.rs:157-168
pub(crate) fn make_codec_params(&self) -> AudioCodecParameters {
    AudioCodecParameters {
        codec: self.codec_id,
        profile: self.profile,
        sample_rate: Some(self.sample_rate as u32),
        bits_per_sample: self.bits_per_sample,
        bits_per_coded_sample: self.bits_per_coded_sample,
        channels: self.channels.clone(),
        max_frames_per_packet: self.frames_per_packet,
        verification_check: self.verification_check,
        extra_data: self.extra_data.clone(),
        ..Default::default()
    }
}
```

`mp4a.40.5` means MPEG-4 audio object type index 5.
`symphonia-common 0.6.0 src/mpeg/audio/mod.rs:67-73` maps index 5 to SBR:

```rust
// symphonia-common-0.6.0/src/mpeg/audio/mod.rs:67-73
const AUDIO_OBJECT_TYPES: &[AudioObjectType] = &[
    AudioObjectType::None,
    AudioObjectType::Main,
    AudioObjectType::Lc,
    AudioObjectType::Ssr,
    AudioObjectType::Ltp,
    AudioObjectType::Sbr,
```

`symphonia-common 0.6.0 src/mpeg/audio/mod.rs:219-256` has a first-class `sbr_present` flag and sets it when
that object type is SBR or PS:

```rust
// symphonia-common-0.6.0/src/mpeg/audio/mod.rs:219-256
pub struct AudioSpecificConfig {
    pub object_type: AudioObjectType,
    pub sample_rate: u32,
    pub channels: Option<Channels>,
    pub samples: usize,
    pub sbr_ps_info: Option<(u32, Option<Channels>)>,
    pub sbr_present: bool,
    pub ps_present: bool,
}

impl AudioSpecificConfig {
    /// Read the audio specific configuration from the provided buffer.
    /// ISO14496-3-2009
    pub fn read(buf: &[u8]) -> Result<AudioSpecificConfig> {
        let mut bs = BitReaderLtr::new(buf);

        let mut asc = AudioSpecificConfig {
            object_type: Self::read_audio_object_type(&mut bs)?,
            sample_rate: Self::read_sampling_frequency(&mut bs)?,
            ..Default::default()
        };

        if asc.sample_rate == 0 {
            return decode_error("common (mp4a): a sample rate of 0 is invalid");
        }

        asc.channels = Self::read_channel_config(&mut bs)?;

        if (asc.object_type == AudioObjectType::Sbr) || (asc.object_type == AudioObjectType::Ps) {
            asc.sbr_present = true;
            if asc.object_type == AudioObjectType::Ps {
                asc.ps_present = true;
            }
            let ext_srate = Self::read_sampling_frequency(&mut bs)?;
            asc.object_type = Self::read_audio_object_type(&mut bs)?;

            let ext_chans = if asc.object_type == AudioObjectType::ErBsac {
                Self::read_channel_config(&mut bs)?
```

`symphonia-common 0.6.0 src/mpeg/audio/mod.rs:472-481` maps AAC LC plus `sbr_present` to the HE-AAC codec
profile:

```rust
// symphonia-common-0.6.0/src/mpeg/audio/mod.rs:472-481
AudioObjectType::Lc => {
    if asc.ps_present {
        Some(CODEC_PROFILE_AAC_HE_V2)
    }
    else if asc.sbr_present {
        Some(CODEC_PROFILE_AAC_HE)
    }
    else {
        Some(CODEC_PROFILE_AAC_LC)
    }
}
```

The AAC decoder then rejects that configuration before decoding any packet.
`symphonia-codec-aac 0.6.0 src/aac/mod.rs:59-96` parses `AudioSpecificConfig` and returns
`aac: aac too complex` when `sbr_present` is true:

```rust
// symphonia-codec-aac-0.6.0/src/aac/mod.rs:59-96
// If extra data present, parse the audio specific config
let asc = if let Some(extra_data_buf) = &params.extra_data {
    validate!(extra_data_buf.len() >= 2);
    AudioSpecificConfig::read(extra_data_buf)?
}
else {
    // Otherwise, assume there is no ASC and use the codec parameters for ADTS.
    let mut asc = AudioSpecificConfig::default();

    asc.object_type = AudioObjectType::Lc;
    asc.samples = 1024;

    asc.sample_rate = match params.sample_rate {
        Some(rate) => rate,
        None => return unsupported_error("aac: sample rate is required"),
    };

    asc.channels = params.channels.clone();

    asc
};

// The channel configuration must be known.
//
// TODO: Support getting this from program configuration element (PCE). However, this would
// require deferring the rest of the initialization until the PCE has been read.
let channels = match &asc.channels {
    Some(channels) => channels.clone(),
    _ => return unsupported_error("aac: channels or channel layout is required"),
};

// Check complexity.
if asc.object_type != AudioObjectType::Lc
    || asc.sbr_present
    || channels.count() > 2
    || asc.samples != 1024
{
    return unsupported_error("aac: aac too complex");
}
```

Removing only that guard would not add HE-AAC support.
`symphonia-codec-aac 0.6.0 src/aac/mod.rs:186-204` detects SBR payloads but ignores the extension payload:

```rust
// symphonia-codec-aac-0.6.0/src/aac/mod.rs:186-204
// Check if the ID_FIL element contains SBR data. Note that ID_FIL elements with
// SBR data may not contain other extension payloads.
if count > 0 {
    let ext_type = bs.read_bits_leq32(4)?;

    match ext_type {
        // EXT_SBR_DATA (0xd)
        // EXT_SBR_DATA_CRC (0xe)
        0xd | 0xe => self.asc.sbr_present = true,
        // EXT_FILL (0x0)
        // EXT_FILL_DATA (0x1)
        // EXT_DATA_ELEMENT (0x2)
        // EXT_DYNAMIC_RANGE (0xb)
        // EXT_SAC_DATA (0xc)
        _ => (),
    }

    // Ignore extension payload(s).
```

The earlier file-corruption hypothesis was wrong.
`ffmpeg` decodes the original file's audio stream without error,
 so this is a Symphonia decoder capability gap:

```bash
# from /var/home/user/Monochromatic
ffmpeg -hide_banner -v error -nostdin \
  -i "/home/user/Seafile/Plain/Music/Fate/Super ☆ Affection.mp4" \
  -map 0:a:0 \
  -f null -
```

That command exits successfully with no output.

## Verification

Version under test:

- `symphonia 0.6.0`,
   checksum `1758d6c853020a7244de03cc3e0185eaea3f58715122422dd3cc7452e6d4c16a`.
- `symphonia-codec-aac 0.6.0`,
   checksum `f1979c515a76371b186aad2feff5f23e21cbec775bf95de08bf1e3af92a2ad76`.
- `symphonia-common 0.6.0`,
   checksum `8257891ffa7f05e02b58f4761e2abf7e5278c8744fd59e981559e050f86eef55`.
- `symphonia-format-isomp4 0.6.0`,
   checksum `2d179a01305b3505940135a9f0180d6ef4b487912748fe97554756f120fbd05e`.

The source clone used for source reading was `/tmp/agent/symphonia-0-6-investigate`.
Its origin was `https://github.com/pdeljanov/Symphonia.git`,
 and its checked-out commit was
`9b791099ae99bed4f4fe7f7c1243ef4b8e7b3ccd`.
The checked-out source matches the same AAC guard and SBR parsing paths as the published 0.6.0 crates.

The reproduction harness was the throwaway benchmark in `/tmp/agent/music-player-truepeak-bench`.
It includes the desktop app's `decode.rs`,
 `opus.rs`,
 `error.rs`,
 and `truepeak.rs` directly.
Run it against the containing folder:

```bash
# from /tmp/agent/music-player-truepeak-bench
mise run bench -- \
  /home/user/Seafile/Plain/Music/Fate \
  /tmp/agent/music-player-truepeak-bench/fate-out
```

Expected result:

```json
// /tmp/agent/music-player-truepeak-bench/fate-out/summary.json
{
  "files_found": 58,
  "files_measured": 57,
  "decode_errors": 1,
  "error_examples": [
    "/home/user/Seafile/Plain/Music/Fate/Super ☆ Affection.mp4: decode error: unsupported feature: aac: aac too complex"
  ]
}
```

Failing catalog:

- Original MP4 video with HE-AAC audio,
   `mp4a.40.5`,
   fails with `unsupported feature: aac: aac too complex`.
- Remuxed audio-only M4A with `-c:a copy` remains HE-AAC,
   still `mp4a.40.5`,
   and fails with the same error.

Working catalog:

- Original MP4 decoded by `ffmpeg` works,
   proving the file is not corrupt.
- Transcoded AAC-LC M4A,
   `mp4a.40.2`,
   measures successfully through the production decoder path.

The failing remux and working transcode were created with:

```bash
# from /var/home/user/Monochromatic
scratch=/tmp/agent/music-player-aac-failure
file="/home/user/Seafile/Plain/Music/Fate/Super ☆ Affection.mp4"
rm --recursive --force "$scratch"
mkdir --parents "$scratch/copy" "$scratch/transcoded"

ffmpeg -hide_banner -v error -nostdin \
  -i "$file" \
  -map 0:a:0 \
  -c:a copy \
  "$scratch/copy/super-affection-copy.m4a"

ffmpeg -hide_banner -v error -nostdin \
  -i "$file" \
  -map 0:a:0 \
  -vn \
  -c:a aac \
  -profile:a aac_low \
  -b:a 160k \
  "$scratch/transcoded/super-affection-aac-lc.m4a"
```

Then verified with:

```bash
# from /tmp/agent/music-player-truepeak-bench
mise run bench -- \
  /tmp/agent/music-player-aac-failure/copy \
  /tmp/agent/music-player-aac-failure/out-copy

mise run bench -- \
  /tmp/agent/music-player-aac-failure/transcoded \
  /tmp/agent/music-player-aac-failure/out-transcoded
```

The remux result fails:

```json
// /tmp/agent/music-player-aac-failure/out-copy/summary.json
{
  "files_found": 1,
  "files_measured": 0,
  "decode_errors": 1,
  "error_examples": [
    "/tmp/agent/music-player-aac-failure/copy/super-affection-copy.m4a: decode error: unsupported feature: aac: aac too complex"
  ]
}
```

The AAC-LC transcode result passes:

```json
// /tmp/agent/music-player-aac-failure/out-transcoded/summary.json
{
  "files_found": 1,
  "files_measured": 1,
  "decode_errors": 0,
  "error_examples": []
}
```

## Verified workarounds

### Transcode HE-AAC/SBR files to AAC-LC

Command:

```bash
# from /var/home/user/Monochromatic
ffmpeg -hide_banner -v error -nostdin \
  -i "/home/user/Seafile/Plain/Music/Fate/Super ☆ Affection.mp4" \
  -map 0:a:0 \
  -vn \
  -c:a aac \
  -profile:a aac_low \
  -b:a 160k \
  /tmp/agent/music-player-aac-failure/transcoded/super-affection-aac-lc.m4a
```

Tradeoffs:

- This is lossy transcoding from one AAC stream to another.
- It discards the video stream in this command.
  Keep or remap video separately if a video file is needed.
- It changes codec profile from HE-AAC to AAC-LC,
   increasing bitrate for similar quality at low bitrates.
- It makes the file decodable by Symphonia's current AAC-LC decoder.

### Keep the file unsupported and skip its peak measurement

The true-peak benchmark already keeps this failure non-fatal.
That is acceptable for analysis because one unsupported HE-AAC file cannot invalidate the measured classifier over the
files Symphonia can decode.

Tradeoffs:

- Playback and true-peak measurement for this file remain unavailable through the current Symphonia decoder path.
- The app should surface or log this as an unsupported codec,
   not as a corrupt file.

## What does not work

### Remuxing without transcoding

`ffmpeg -c:a copy` moves the HE-AAC bitstream into M4A without changing the codec profile.
The copied file still reports `HE-AAC` and `mp4a.40.5`,
 and the production decoder still fails with
`unsupported feature: aac: aac too complex`.

### Changing MP4 stream selection

The app is already selecting the first known audio track and routing non-Opus audio to Symphonia.
The selected stream is the only audio stream in this file.
Changing stream selection would not change its HE-AAC/SBR profile.

### Removing only the `sbr_present` guard

The decoder detects SBR extension payloads but ignores them.
Removing the constructor guard would allow unsupported HE-AAC data into a decoder that has no SBR reconstruction path.
That would risk wrong audio rather than fixing decode support.

### Treating upstream issue 325 as the same bug

[`pdeljanov/Symphonia#325`][symphonia-325] mentions `aac: aac too complex`,
 but the maintainer diagnosed that report as
an ADTS stream auto-detection issue fixed in the 0.6 development branch.
This report is an MP4 HE-AAC/SBR decode limitation in 0.6.0.
The closed issue is not a duplicate.

## Upstream filing decision

No `.out-of-scope/` exemption matches Symphonia or AAC.
Checked files under `.out-of-scope/` included TypeScript,
 Claude Code,
 Bun,
 Codex,
 pi,
 Lightning CSS,
 JSR,
Cargo workspace,
 and related project-specific exemptions.

Duplicate search:

```bash
# from /var/home/user/Monochromatic
gh search issues --repo pdeljanov/Symphonia "HE-AAC SBR" --state open --limit 10
gh search issues --repo pdeljanov/Symphonia "HE-AAC SBR" --state closed --limit 10
gh search issues --repo pdeljanov/Symphonia "aac too complex" --state open --limit 10
gh search issues --repo pdeljanov/Symphonia "aac too complex" --state closed --limit 10
gh search prs --repo pdeljanov/Symphonia "HE-AAC SBR" --state open --limit 10
gh search prs --repo pdeljanov/Symphonia "HE-AAC SBR" --state closed --limit 10
gh search prs --repo pdeljanov/Symphonia "aac too complex" --state open --limit 10
gh search prs --repo pdeljanov/Symphonia "aac too complex" --state closed --limit 10
```

Only `pdeljanov/Symphonia#325` appeared,
 and it is not the same root cause.

Six-constraint check:

- Is it really upstream's fault?
  As a bug report,
   no.
  Symphonia documents AAC-LC support,
   and the AAC crate README says the decoder implements the LC profile.
  Symphonia also documents HE-AAC and HE-AACv2 with blank status in `README.md:102-106`.
- Can upstream fix it?
  Yes.
  HE-AAC/SBR support is implementable in principle,
   but it is real decoder work rather than a small guard change.
- Are they supporting this use case?
  No.
  The top-level README lists AAC-LC as supported and HE-AAC/HE-AACv2 as not currently supported.
  The AAC crate README says the decoder implements the LC profile.
- Would the repo welcome our contribution?
  Not an AI-authored external communication.
  `CONTRIBUTING.md` welcomes new decoders and features,
   but `CONTRIBUTING.md:54-65` forbids copied or heavily
  borrowed AI-generated implementation work,
   and `CONTRIBUTING.md:67-79` says responsible AI use must be disclosed.
  It also says not to use AI for communications unless translation.
  A human-authored issue or PR may be acceptable,
   but this AI-authored draft must not be filed as-is.
- Will they likely fix it?
  Unknown.
  The README explicitly marks HE-AAC support absent and points to `symphonia-adapter-fdk-aac` as a third-party
  option for AAC-LC,
   HE-AAC,
   and HE-AACv2.
  No matching open issue or PR was found.
- Have we prototyped a minimal fix compatible with their architecture?
  No.
  The earlier constraints fail,
   and the source trace shows the minimal real fix is SBR support,
   not a local guard
  change.
  The auto-prototype rule therefore does not fire.

Decision:
Do not file upstream from this session.
This is a documented unsupported feature,
 and Symphonia's contribution policy bars this AI-authored external
communication from being filed as-is.

Draft,
 do not file as-is:

~~~md
Title: HE-AAC/SBR MP4 audio returns `unsupported feature: aac: aac too complex` in Symphonia 0.6.0

This is a feature request, not a bug report against documented AAC-LC support.

Reproduction input:

- MP4 audio stream: AAC, HE-AAC, `mp4a.40.5`, 44.1 kHz, stereo.
- Symphonia crates: `symphonia-codec-aac 0.6.0`, `symphonia-common 0.6.0`, `symphonia-format-isomp4 0.6.0`.

Observed failure:

```text
unsupported feature: aac: aac too complex
```

Source trace:

- `symphonia-format-isomp4/src/atoms/esds.rs` reads `AudioSpecificConfig` and stores `extra_data` on the sample entry.
- `symphonia-format-isomp4/src/atoms/stsd.rs` copies that `extra_data` into `AudioCodecParameters`.
- `symphonia-common/src/mpeg/audio/mod.rs` maps audio object type index 5 to `AudioObjectType::Sbr` and sets `sbr_present`.
- `symphonia-codec-aac/src/aac/mod.rs` rejects `asc.sbr_present` in `AacDecoder::try_new` with `aac: aac too complex`.
- The same decoder later detects SBR extension payloads and ignores them, so removing only the constructor guard is not a correct fix.

Suggested fix direction:

Implement HE-AAC/SBR support in `symphonia-codec-aac` behind the existing AAC architecture, with tests using
real-world HE-AAC vectors.
The constructor guard in `symphonia-codec-aac/src/aac/mod.rs` should only be relaxed after SBR reconstruction is implemented.
~~~

[symphonia-325]: https://github.com/pdeljanov/Symphonia/issues/325
