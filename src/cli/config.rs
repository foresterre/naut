use crate::cli::app::arg_names::{
    ARG_GLOB_NO_SKIP_UNSUPPORTED_EXTENSIONS, ARG_IMAGE_CRATE_FALLBACK, ARG_INPUT, ARG_INPUT_GLOB,
    ARG_OUTPUT, ARG_OUTPUT_GLOB,
};
use crate::cli::common_dir::CommonDir;
use crate::cli::glob_base_dir::glob_builder_base;
use anyhow::{bail, Context};
use clap::ArgMatches;
use globwalk::{FileType, GlobWalker};
use naut_image_engine::engine::Instr;
use naut_io::load::FrameIndex;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum PathVariant {
    StdStream,
    Path(PathBuf),
}

impl PathVariant {
    pub fn is_std_stream(&self) -> bool {
        match self {
            PathVariant::StdStream => true,
            PathVariant::Path(_) => false,
        }
    }
}

pub enum InputOutputMode {
    Single {
        input: PathVariant,
        output: PathVariant,
    },
    Batch {
        inputs: CommonDir,
        output_root_folder: PathBuf,
    },
}

impl InputOutputMode {
    pub fn try_from_matches(matches: &ArgMatches) -> anyhow::Result<Self> {
        let mode = InputOutputModeType::from_arg_matches(matches)?;

        match mode {
            InputOutputModeType::Simple => Ok(InputOutputMode::Single {
                input: match matches.value_of(ARG_INPUT) {
                    Some(p) => PathVariant::Path(p.into()),
                    None => PathVariant::StdStream,
                },
                output: match matches.value_of(ARG_OUTPUT) {
                    Some(p) => PathVariant::Path(p.into()),
                    None => PathVariant::StdStream,
                },
            }),
            InputOutputModeType::Batch => {
                let inputs = matches
                    .value_of(ARG_INPUT_GLOB)
                    .with_context(|| "Glob mode requires an input pattern")?;
                let output = matches
                    .value_of(ARG_OUTPUT_GLOB)
                    .with_context(|| "Glob mode requires an output folder")?;

                Ok(InputOutputMode::Batch {
                    inputs: {
                        let inputs = Self::create_glob_walker(inputs)?;

                        let paths = Self::lookup_paths(
                            inputs,
                            !matches.is_present(ARG_GLOB_NO_SKIP_UNSUPPORTED_EXTENSIONS),
                            matches.is_present(ARG_IMAGE_CRATE_FALLBACK),
                        )?;

                        CommonDir::try_new(paths)?
                    },
                    output_root_folder: { output.into() },
                })
            }
        }
    }

    fn create_glob_walker<PAT: AsRef<str>>(pattern: PAT) -> anyhow::Result<GlobWalker> {
        glob_builder_base(pattern.as_ref(), &[])?
            .follow_links(true)
            .file_type(FileType::FILE)
            .build()
            .with_context(|| "Unable to parse the given glob pattern")
    }

    fn lookup_paths(
        inputs: impl Iterator<Item = Result<globwalk::DirEntry, globwalk::WalkError>>,
        filter_unsupported: bool,
        image_crate_fallback_enabled: bool,
    ) -> anyhow::Result<Vec<PathBuf>> {
        let paths: Vec<PathBuf> = inputs
            .map(|entry| {
                entry
                    .map_err(|err| {
                        anyhow::anyhow!(
                            "Error while trying to find glob matches on the fs ({})",
                            err
                        )
                    })
                    .map(|f| f.into_path())
            })
            .collect::<anyhow::Result<Vec<PathBuf>>>()?;

        Ok(if filter_unsupported {
            filter_unsupported_paths(paths, image_crate_fallback_enabled)
        } else {
            paths
        })
    }
}

// remove paths with extensions we don't recognise
fn filter_unsupported_paths(paths: Vec<PathBuf>, fallback_enabled: bool) -> Vec<PathBuf> {
    use crate::cli::pipeline::fallback::guess_output_by_path;
    use crate::combinators::FallbackIf;
    use naut_io::format::DetermineEncodingFormat;
    use naut_io::format::EncodingFormatByExtension;

    let checker = DetermineEncodingFormat::default();

    paths
        .into_iter()
        .filter(|path| {
            checker
                .by_extension(path)
                .fallback_if(fallback_enabled, guess_output_by_path, path)
                .is_ok()
        })
        .collect()
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InputOutputModeType {
    Simple,
    Batch,
}

impl InputOutputModeType {
    pub fn from_arg_matches(matches: &ArgMatches) -> anyhow::Result<InputOutputModeType> {
        Ok(
            match (
                matches.is_present(ARG_INPUT),
                matches.is_present(ARG_INPUT_GLOB),
            ) {
                (true, false) => InputOutputModeType::Simple,
                (false, true) => InputOutputModeType::Batch,
                _ => {
                    bail!("Unable select input/output mode: mode should either be simple xor glob")
                }
            },
        )
    }
}

#[derive(Debug, Clone)]
pub struct Config<'a> {
    pub tool_name: &'static str,

    pub mode: InputOutputModeType,

    // organisational
    /// Display license of this software or its dependencies.
    pub show_license_text_of: Option<SelectedLicenses>,

    pub selected_frame: FrameIndex,

    /// Disable color type adjustments on save.
    pub disable_automatic_color_type_adjustment: bool,

    /// Format to which an image will be converted (enforced).
    pub forced_output_format: Option<&'a str>,

    /// Encoding settings for specific output formats.
    pub encoding_settings: FormatEncodingSettings,

    /// If a user wants to perform image operations on input image, they will need to provide
    /// the image operation commands.
    /// THe value set here should be presented as a [naut_image_engine::engine::Program].
    /// If no program is present, an empty vec should be provided.
    pub image_operations_program: Vec<Instr>,
}

impl Default for Config<'_> {
    fn default() -> Self {
        Config {
            /// If using default, requires the `CARGO_PKG_NAME` to be set.
            tool_name: env!("CARGO_PKG_NAME"),

            mode: InputOutputModeType::Simple,

            /// Defaults to no displayed license text.
            show_license_text_of: None,

            /// By default the first frame of a gif is used.
            selected_frame: FrameIndex::First,

            /// Defaults to using automatic color type adjustment where appropriate.
            disable_automatic_color_type_adjustment: false,

            /// Defaults to not forcing a specific image output format.
            forced_output_format: None,

            /// Default format encoding settings.
            encoding_settings: FormatEncodingSettings {
                /// Default JPEG quality is set to 80.
                jpeg_quality: 80,

                /// Default encoding type of PNM files (excluding PAM) is set to binary.
                pnm_use_ascii_format: false,

                /// Do not fallback to image crate output recognition by default
                image_output_format_fallback: false,
            },

            /// Defaults to no provided image operations script.
            image_operations_program: Vec::new(),
        }
    }
}

/// Builder for [crate::config::Config]. Should be used with the Default implementation
/// of [crate::config::Config].
/// If the default trait is not used with this builder, some settings may be inaccessible.
/// For example, `output_path` can be set to some value, but not unset.
///
/// Builder is consuming.
#[derive(Debug, Default)]
pub struct ConfigBuilder<'a> {
    settings: Config<'a>,
}

impl<'a> ConfigBuilder<'a> {
    pub fn new() -> Self {
        ConfigBuilder::default()
    }

    pub fn mode(mut self, mode: InputOutputModeType) -> ConfigBuilder<'a> {
        self.settings.mode = mode;
        self
    }

    // organisational
    pub fn show_license_text_of(mut self, selection: SelectedLicenses) -> ConfigBuilder<'a> {
        self.settings.show_license_text_of = Some(selection);
        self
    }

    // config(in)
    pub fn select_frame(mut self, frame: FrameIndex) -> ConfigBuilder<'a> {
        self.settings.selected_frame = frame;
        self
    }

    // config(out)
    pub fn forced_output_format(mut self, format: &'a str) -> ConfigBuilder<'a> {
        self.settings.forced_output_format = Some(format);
        self
    }

    // config(out)
    pub fn disable_automatic_color_type_adjustment(mut self, toggle: bool) -> ConfigBuilder<'a> {
        self.settings.disable_automatic_color_type_adjustment = toggle;
        self
    }

    // config(out)
    pub fn jpeg_quality(mut self, quality: u8) -> ConfigBuilder<'a> {
        self.settings.encoding_settings.jpeg_quality = quality;
        self
    }

    // config(out)
    pub fn pnm_format_type(mut self, use_ascii: bool) -> ConfigBuilder<'a> {
        self.settings.encoding_settings.pnm_use_ascii_format = use_ascii;
        self
    }

    pub fn image_output_format_decider_fallback(
        mut self,
        enable_fallback: bool,
    ) -> ConfigBuilder<'a> {
        self.settings.encoding_settings.image_output_format_fallback = enable_fallback;
        self
    }

    // image-operations
    pub fn image_operations_program(mut self, program: Vec<Instr>) -> ConfigBuilder<'a> {
        self.settings.image_operations_program = program;
        self
    }

    pub fn build(self) -> Config<'a> {
        self.settings
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SelectedLicenses {
    ThisSoftware,
    Dependencies,
}

#[derive(Debug, Clone)]
pub struct FormatEncodingSettings {
    pub jpeg_quality: u8,
    pub pnm_use_ascii_format: bool,

    // Whether to fallback on the image crate to determine the output format if naut doesn't support it yet
    pub image_output_format_fallback: bool,
}

/// Strictly speaking not necessary here since the responsible owners will validate the quality as well.
/// However, by doing anyways it we can exit earlier.
pub fn validate_jpeg_quality(quality: u8) -> anyhow::Result<u8> {
    fn within_range(v: u8) -> anyhow::Result<u8> {
        // Upper bound is exclusive with .. syntax.
        // When the `range_contains` feature will be stabilised Range.contains(&v)
        // should be used instead.
        const ALLOWED_RANGE: std::ops::Range<u8> = 1..101;
        if ALLOWED_RANGE.contains(&v) {
            Ok(v)
        } else {
            bail!("JPEG Encoding Settings error: JPEG quality requires a number between 1 and 100 (inclusive).")
        }
    }

    within_range(quality)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use naut_image_engine::engine::Instr;
    use naut_image_engine::ImgOp;

    use super::*;

    #[test]
    fn jpeg_in_quality_range_lower_bound_inside() {
        let value: &str = "1";
        assert!(validate_jpeg_quality(u8::from_str(value).unwrap()).is_ok())
    }

    #[test]
    fn jpeg_in_quality_range_lower_bound_outside() {
        let value: &str = "0";
        assert!(validate_jpeg_quality(u8::from_str(value).unwrap()).is_err())
    }

    #[test]
    fn jpeg_in_quality_range_upper_bound_inside() {
        let value: &str = "100";
        assert!(validate_jpeg_quality(u8::from_str(value).unwrap()).is_ok())
    }

    #[test]
    fn jpeg_in_quality_range_upper_bound_outside() {
        let value: &str = "101";
        assert!(validate_jpeg_quality(u8::from_str(value).unwrap()).is_err())
    }

    #[test]
    fn config_builder_override_defaults() {
        let mut builder = ConfigBuilder::new();
        builder = builder.image_operations_program(vec![Instr::Operation(ImgOp::Blur(1.0))]);
        let config = builder.build();

        assert!(!config.image_operations_program.is_empty());
    }

    #[test]
    fn skip_unsupported_paths() {
        fn to_path_bufs<'s>(paths: impl IntoIterator<Item = &'s &'s str>) -> Vec<PathBuf> {
            paths
                .into_iter()
                .map(|s| FromStr::from_str(s).expect("test should have valid input"))
                .collect::<Vec<_>>()
        }

        let paths = &[
            "/scope/0.png",
            "/scope/1.jpg",
            "/scope/2.jpeg",
            "/scope/2.unsupported",
            "/scope/2",
        ];

        let path_bufs = to_path_bufs(paths);
        let expected_path_bufs = to_path_bufs(&[paths[0], paths[1], paths[2]]);
        let filtered = filter_unsupported_paths(path_bufs, false);

        assert_eq!(filtered, expected_path_bufs);
    }

    mod glob_skip_unsupported {
        use super::*;
        use parameterized::parameterized;

        parameterized::ide!();

        #[parameterized(paths_in = {
            &["/test/0.png", "/test/1.jpg", "/test/2.jpeg", "/test/2.unsupported", "/test/2"],
            &[],
            &["a.farbfeld", "a.ff"],
            &["a.farbfeld", "a.ff"],
        }, paths_expected = {
            &["/test/0.png", "/test/1.jpg", "/test/2.jpeg"],
            &[],
            &["a.farbfeld", "a.ff"],
            &["a.farbfeld"],
        }, fallback_on_imagecrate = {
            false,
            false,
            true,
            false,
        })]
        fn are_unsupported_paths_getting_filtered(
            paths_in: &[&str],
            paths_expected: &[&str],
            fallback_on_imagecrate: bool,
        ) {
            fn to_path_bufs<'s>(paths: impl IntoIterator<Item = &'s &'s str>) -> Vec<PathBuf> {
                paths
                    .into_iter()
                    .map(|s| FromStr::from_str(s).expect("test should have valid input"))
                    .collect::<Vec<_>>()
            }

            let path_bufs = to_path_bufs(paths_in);
            let expected_path_bufs = to_path_bufs(paths_expected);
            let filtered = filter_unsupported_paths(path_bufs, fallback_on_imagecrate);

            assert_eq!(filtered, expected_path_bufs);
        }
    }
}
