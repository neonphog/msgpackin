/// Msgpackin config for encoders / decoders
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Config {
    /// maximum container depth when encoding / decoding
    pub max_depth: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { max_depth: 1024 }
    }
}
