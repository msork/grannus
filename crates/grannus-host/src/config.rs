//! Typed, validated host configuration with conservative pre-alpha defaults.

use serde::Deserialize;
use std::fmt;
use std::fs;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;

const MAX_HOST_NAME_LEN: usize = 64;
const MAX_CONFIG_BYTES: usize = 64 * 1024;
const MAX_DIMENSION: u16 = 8_192;
const MAX_REFRESH_HZ: u16 = 240;
const MIN_BITRATE_KBPS: u32 = 500;
const MAX_BITRATE_KBPS: u32 = 100_000;
const MIN_STALE_AFTER_MS: u32 = 10;
const MAX_NEUTRALIZE_AFTER_MS: u32 = 30_000;
const MAX_PLAYER_SLOTS: u8 = 8;

/// Fully resolved host configuration.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// General host behavior.
    pub host: HostConfig,
    /// Video defaults offered to future backends.
    pub video: VideoConfig,
    /// Audio defaults offered to future backends.
    pub audio: AudioConfig,
    /// Input freshness limits.
    pub input: InputConfig,
    /// Discovery and network-exposure controls.
    pub discovery: DiscoveryConfig,
    /// Pairing safety controls.
    pub security: SecurityConfig,
}

impl Config {
    /// Parses and validates a TOML configuration document.
    ///
    /// # Errors
    /// Returns [`ConfigError`] when TOML is malformed or a value violates a
    /// supported pre-alpha safety boundary.
    pub fn from_toml(text: &str) -> Result<Self, ConfigError> {
        let config: Self = toml::from_str(text).map_err(ConfigError::Parse)?;
        config.validate()?;
        Ok(config)
    }

    /// Loads and validates TOML configuration from a path.
    ///
    /// # Errors
    /// Returns [`ConfigError`] when the file cannot be read, TOML is malformed,
    /// or a value violates a supported pre-alpha safety boundary.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let mut reader = fs::File::open(path)
            .map_err(ConfigError::Read)?
            .take((MAX_CONFIG_BYTES + 1) as u64);
        let mut text = String::new();
        reader
            .read_to_string(&mut text)
            .map_err(ConfigError::Read)?;
        if text.len() > MAX_CONFIG_BYTES {
            return Err(ConfigError::invalid(
                "configuration",
                "file must not exceed 65536 bytes",
            ));
        }
        Self::from_toml(&text)
    }

    /// Validates the resolved configuration before it reaches host lifecycle code.
    ///
    /// # Errors
    /// Returns [`ConfigError::InvalidValue`] when a value falls outside an
    /// explicit supported or safety-preserving range.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.host.name.is_empty() || self.host.name.len() > MAX_HOST_NAME_LEN {
            return Err(ConfigError::invalid(
                "host.name",
                "must contain 1 to 64 bytes",
            ));
        }
        if self.host.internet_exposure {
            return Err(ConfigError::invalid(
                "host.internet_exposure",
                "must remain false until authenticated Internet transport exists",
            ));
        }
        if !self.host.listen.ip().is_loopback() {
            return Err(ConfigError::invalid(
                "host.listen",
                "must use a loopback address until authenticated listeners exist",
            ));
        }
        if self.host.listen.port() == 0 {
            return Err(ConfigError::invalid(
                "host.listen",
                "must use an explicit nonzero port",
            ));
        }
        if self.video.width == 0 || self.video.width > MAX_DIMENSION {
            return Err(ConfigError::invalid(
                "video.width",
                "must be between 1 and 8192",
            ));
        }
        if self.video.height == 0 || self.video.height > MAX_DIMENSION {
            return Err(ConfigError::invalid(
                "video.height",
                "must be between 1 and 8192",
            ));
        }
        if self.video.refresh_hz == 0 || self.video.refresh_hz > MAX_REFRESH_HZ {
            return Err(ConfigError::invalid(
                "video.refresh_hz",
                "must be between 1 and 240",
            ));
        }
        if !(MIN_BITRATE_KBPS..=MAX_BITRATE_KBPS).contains(&self.video.bitrate_kbps) {
            return Err(ConfigError::invalid(
                "video.bitrate_kbps",
                "must be between 500 and 100000",
            ));
        }
        if !self.video.low_latency || self.video.b_frames != 0 || self.video.lookahead_frames != 0 {
            return Err(ConfigError::invalid(
                "video",
                "requires low_latency=true, b_frames=0, and lookahead_frames=0",
            ));
        }
        if self.audio.sample_rate_hz != 48_000 || self.audio.channels != 2 {
            return Err(ConfigError::invalid(
                "audio",
                "currently supports only 48 kHz stereo",
            ));
        }
        if self.input.stale_after_ms < MIN_STALE_AFTER_MS
            || self.input.stale_after_ms >= self.input.neutralize_after_ms
            || self.input.neutralize_after_ms > MAX_NEUTRALIZE_AFTER_MS
        {
            return Err(ConfigError::invalid(
                "input",
                "requires 10 <= stale_after_ms < neutralize_after_ms <= 30000",
            ));
        }
        if self.input.max_player_slots == 0 || self.input.max_player_slots > MAX_PLAYER_SLOTS {
            return Err(ConfigError::invalid(
                "input.max_player_slots",
                "must be between 1 and 8",
            ));
        }
        if self.discovery.lan || self.discovery.upnp || self.discovery.pcp || self.discovery.nat_pmp
        {
            return Err(ConfigError::invalid(
                "discovery",
                "all discovery and NAT traversal features are unavailable in M0",
            ));
        }
        if self.security.pairing_enabled || self.security.allow_unpaired {
            return Err(ConfigError::invalid(
                "security",
                "pairing and unpaired access are unavailable in M0",
            ));
        }
        Ok(())
    }
}

/// General host settings.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct HostConfig {
    /// Human-readable host label.
    pub name: String,
    /// Reserved future listener address; must remain loopback in M0.
    pub listen: SocketAddr,
    /// Whether future Internet exposure is requested.
    pub internet_exposure: bool,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            name: "grannus".to_owned(),
            listen: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 47_990),
            internet_exposure: false,
        }
    }
}

/// Video defaults.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct VideoConfig {
    /// Requested frame width.
    pub width: u16,
    /// Requested frame height.
    pub height: u16,
    /// Requested refresh rate.
    pub refresh_hz: u16,
    /// Requested codec.
    pub codec: VideoCodec,
    /// Requested bitrate in kilobits per second.
    pub bitrate_kbps: u32,
    /// Low-latency encoder mode.
    pub low_latency: bool,
    /// Requested B-frame count.
    pub b_frames: u8,
    /// Requested lookahead frame count.
    pub lookahead_frames: u8,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            width: 1_920,
            height: 1_080,
            refresh_hz: 60,
            codec: VideoCodec::H264,
            bitrate_kbps: 20_000,
            low_latency: true,
            b_frames: 0,
            lookahead_frames: 0,
        }
    }
}

/// Video codecs available in the initial configuration schema.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VideoCodec {
    /// H.264 is the initial compatibility default.
    H264,
}

/// Audio defaults.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct AudioConfig {
    /// Sample rate in hertz.
    pub sample_rate_hz: u32,
    /// Interleaved channel count.
    pub channels: u8,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate_hz: 48_000,
            channels: 2,
        }
    }
}

/// Input freshness defaults.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct InputConfig {
    /// Telemetry and future freshness threshold.
    pub stale_after_ms: u32,
    /// Timeout after which the backend is neutralized.
    pub neutralize_after_ms: u32,
    /// Maximum controller slots exposed by future lifecycle code.
    pub max_player_slots: u8,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            stale_after_ms: 100,
            neutralize_after_ms: 250,
            max_player_slots: MAX_PLAYER_SLOTS,
        }
    }
}

/// Network discovery controls.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(default, deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)] // TOML exposes each independently opt-in feature by name.
pub struct DiscoveryConfig {
    /// LAN discovery request.
    pub lan: bool,
    /// `UPnP` request.
    pub upnp: bool,
    /// PCP request.
    pub pcp: bool,
    /// NAT-PMP request.
    pub nat_pmp: bool,
}

/// Pairing and unauthenticated-access controls.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct SecurityConfig {
    /// Pairing request.
    pub pairing_enabled: bool,
    /// Unauthenticated access request.
    pub allow_unpaired: bool,
}

/// Configuration loading or validation failure.
#[derive(Debug)]
pub enum ConfigError {
    /// The configuration file could not be read.
    Read(std::io::Error),
    /// The configuration document is not valid TOML or does not match the schema.
    Parse(toml::de::Error),
    /// A parsed setting violates a supported bound or safety requirement.
    InvalidValue {
        field: &'static str,
        message: &'static str,
    },
}

impl ConfigError {
    fn invalid(field: &'static str, message: &'static str) -> Self {
        Self::InvalidValue { field, message }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(error) => write!(formatter, "could not read configuration: {error}"),
            Self::Parse(error) => write!(formatter, "invalid configuration: {error}"),
            Self::InvalidValue { field, message } => {
                write!(formatter, "invalid configuration value {field}: {message}")
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read(error) => Some(error),
            Self::Parse(error) => Some(error),
            Self::InvalidValue { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEMP_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn empty_document_uses_safe_defaults() {
        assert_eq!(Config::from_toml("").unwrap(), Config::default());
    }

    #[test]
    fn example_configuration_loads() {
        assert_eq!(
            Config::from_toml(include_str!("../../../config/grannus.example.toml")).unwrap(),
            Config::default()
        );
    }

    #[test]
    fn rejects_unknown_fields() {
        let error = Config::from_toml("[host]\nunknown = true\n").unwrap_err();
        assert!(error.to_string().contains("unknown field"));
    }

    #[test]
    fn rejects_unsafe_or_inconsistent_values() {
        for document in [
            "[host]\ninternet_exposure = true\n",
            "[host]\nlisten = '0.0.0.0:47990'\n",
            "[host]\nlisten = '127.0.0.1:0'\n",
            "[video]\nb_frames = 1\n",
            "[input]\nstale_after_ms = 250\nneutralize_after_ms = 250\n",
            "[discovery]\nupnp = true\n",
            "[security]\nallow_unpaired = true\n",
        ] {
            assert!(Config::from_toml(document).is_err(), "{document}");
        }
    }

    #[test]
    fn loads_from_a_file() {
        let suffix = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "grannus-config-{}-{suffix}.toml",
            std::process::id()
        ));
        fs::write(&path, "[host]\nname = 'test-host'\n").unwrap();
        let loaded = Config::load(&path);
        fs::remove_file(&path).unwrap();
        assert_eq!(loaded.unwrap().host.name, "test-host");
    }

    #[test]
    fn rejects_an_oversized_file() {
        let suffix = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "grannus-config-{}-{suffix}.toml",
            std::process::id()
        ));
        fs::write(&path, "#".repeat(MAX_CONFIG_BYTES + 1)).unwrap();
        let error = Config::load(&path).unwrap_err();
        fs::remove_file(&path).unwrap();

        assert!(error.to_string().contains("must not exceed 65536 bytes"));
    }
}
