use snafu::Snafu;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Config parse failed: {}", source))]
    ConfigParseFailed { source: config::ConfigError },

    #[snafu(display("Failed to write config: {}", source))]
    WriteConfigFailed { source: std::io::Error },
}
