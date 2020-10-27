use thiserror::Error;

#[derive(Debug, Error)]
pub enum SicImageEngineError {
    #[error("unable to crop; required top-left anchor < bottom-right anchor; note that (x=0,y=0) is the smallest top-left coordinate; [top-left anchor: (x={0}, y={1}), bottom-right anchor: (x={2}, y={3})]")]
    CropInvalidSelection(u32, u32, u32, u32),

    #[error("unable to crop; anchor coordinates should be within image bounds [image size: (x={0}, y={1}), top-left anchor: (x={2}, y={3}), bottom-right anchor: (x={4}, y={5})]")]
    CropCoordinateOutOfBounds(u32, u32, u32, u32, u32, u32),

    #[error("unable to load image argument from given path")]
    LoadImageFromPath,

    #[error("filter type '{0}' not found")]
    UnknownFilterType(String),

    #[cfg(feature = "imageproc-ops")]
    #[error("unable to load font: invalid format")]
    FontError,

    #[cfg(feature = "imageproc-ops")]
    #[error("unable to open font file from path: '{0}'")]
    FontFileLoadError(std::io::Error),
}
