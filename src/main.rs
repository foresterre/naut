#![deny(clippy::all)]

use naut::cli::app::arg_names::{ARG_DEP_LICENSES, ARG_LICENSE};
use naut::cli::app::build_app_config;
use naut::cli::config::InputOutputMode;
use naut::cli::license::LicenseTexts;
use naut::cli::pipeline::{run_display_licenses, run_with_devices};

const LICENSE_SELF: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/LICENSE-MIT"));

const LICENSE_DEPS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/compressed_dep_licenses"));

const ABOUT: &str = include_str!("../resources/help-pages/about.txt");
const HELP_OPERATIONS_AVAILABLE: &str =
    include_str!("../resources/help-pages/image_operations.txt");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> anyhow::Result<()> {
    let app = naut::cli::app::create_app(VERSION, ABOUT, HELP_OPERATIONS_AVAILABLE);
    let matches = app.get_matches();

    let license_display = matches.is_present(ARG_LICENSE) || matches.is_present(ARG_DEP_LICENSES);

    let configuration = build_app_config(&matches)?;

    if license_display {
        run_display_licenses(
            &configuration,
            &LicenseTexts::new(LICENSE_SELF, LICENSE_DEPS),
        )
    } else {
        let io_device = InputOutputMode::try_from_matches(&matches)?;
        run_with_devices(io_device, &configuration)
    }
}
