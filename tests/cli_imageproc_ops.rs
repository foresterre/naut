#![cfg(feature = "imageproc-ops")]

pub mod common;

#[macro_use]
extern crate parameterized;

#[cfg(test)]
mod tests {
    use crate::common::*;

    ide!();

    #[parameterized(
        ops = {
            r#"draw-text "example" coord(0,1) rgba(0,0,0,255) size(24) font("%font%");"#,
        },
        output_file = {
            "imageproc_ops_draw_text_apply_operations"
        },
    )]
    fn check_imageproc_ops_with_script(ops: &str, output_file: &str) {
        let font_file = &[
            env!("CARGO_MANIFEST_DIR"),
            "/resources/font/Lato-Regular.ttf",
        ]
        .concat();

        let ops = ops.replace("%font%", font_file);
        let mut process = command_unsplit_with_features(
            "unsplash_763569_cropped.jpg",
            format!("{}.png", output_file).as_str(),
            &["--apply-operations", &ops],
            &["imageproc-ops"],
        );
        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }

    #[parameterized(
        ops = {
            &["--draw-text", "example", "coord(0,1)", "rgba(0,0,0,255)", "size(24)", "font('▲')"],
            &["--draw-text", "example", "coord(0,1)", "rgba(0,0,0,255)", "size(24)", "font(\"▲\")"],
            &["--draw-text", "example", "coord(0,1)", "rgba(0,0,0,255)", "size(24)", "font(\"▲\')"],
        },
        output_file = {
            "imageproc_ops_draw_text_cli_arg_0_ok",
            "imageproc_ops_draw_text_cli_arg_1_ok",
            "imageproc_ops_draw_text_cli_arg_2_err",
        },
        ok = {
            true,
            true,
            false,
        }
    )]
    fn check_imageproc_ops_with_cli_args(ops: &[&str], output_file: &str, ok: bool) {
        let font_file = &[
            env!("CARGO_MANIFEST_DIR"),
            "/resources/font/Lato-Regular.ttf",
        ]
        .concat();

        let ops = ops
            .iter()
            .map(|v| v.replace('▲', font_file))
            .collect::<Vec<_>>();

        let ops = ops.iter().map(|v| v.as_str()).collect::<Vec<_>>();

        let mut process = command_unsplit_with_features(
            "unsplash_763569_cropped.jpg",
            format!("{}.png", output_file).as_str(),
            &ops,
            &["imageproc-ops"],
        );
        let result = process.wait();
        assert!(result.is_ok());
        let result = result.unwrap();

        if ok {
            assert!(result.success());
        } else {
            assert!(!result.success());
        }
    }
}