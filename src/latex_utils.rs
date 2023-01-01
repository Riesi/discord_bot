use std::sync::Once;
use tectonic;

use magick_rust::{magick_wand_genesis, MagickError, MagickWand};

pub fn pdf_latex(input_string: &str) -> Result<Vec<u8>, tectonic::Error> {
    let template_start = r#"\documentclass[preview]{standalone}
        \usepackage[utf8]{inputenc}
        \usepackage{amsfonts}
        \usepackage{amssymb}
        \usepackage{amsmath}
        \usepackage{color}
        \usepackage{xcolor}
        \usepackage{dsfont}
        \begin{document}{\color{white}
        "#;

    let template_end = r#"
        }\end{document}
        "#;

    let mut status = tectonic::status::plain::PlainStatusBackend::default();

    let auto_create_config_file = false;
    let config = tectonic::config::PersistentConfig::open(auto_create_config_file).expect("Failed to open the default configuration file!");

    let only_cached = false;
    let bundle = config.default_bundle(only_cached, &mut status).expect("Failed to load the default resource bundle!");

    let format_cache_path = config.format_cache_path().expect("Failed to set up the format cache!");

    let mut files = {
        // Looking forward to non-lexical lifetimes!
        let mut sb = tectonic::driver::ProcessingSessionBuilder::default();
        sb.bundle(bundle)
            .primary_input_buffer((template_start.to_owned() + input_string + template_end).as_bytes())
            .tex_input_name("texput.tex")
            .format_name("latex")
            .format_cache_path(format_cache_path)
            .keep_logs(false)
            .keep_intermediates(false)
            .print_stdout(false)
            .output_format(tectonic::driver::OutputFormat::Pdf)
            .do_not_write_output_files();

        let mut sess =
            sb.create(&mut status).expect("Failed to initialize the LaTeX processing session!");

        if let Err(w) = sess.run(&mut status) {
            eprintln!("The LaTeX engine failed!");
            return Err(w);
        }
        sess.into_file_data()
    };
    let pdf_bytes = match files.remove("texput.pdf") {
        Some(file) => file.data,
        None => vec![],
    };

    println!("Output PDF size is {} bytes", pdf_bytes.len());
    Ok(pdf_bytes)
}

static START: Once = Once::new();

pub fn convert_pdf_png(pdf_doc: &[u8]) -> Result<Vec<u8>, MagickError> {
    START.call_once(|| {
        magick_wand_genesis();
    });
    let wand = MagickWand::new();
    wand.set_resolution(500f64, 500f64)
        .expect("Setting resolution failed!");
    wand.read_image_blob(pdf_doc).expect("Reading PDF failed!");
    wand.write_image_blob("png")
}

pub fn latex_tex_png(input_string: &str) -> Result<Vec<u8>, tectonic::Error> {
    let pdf_doc = pdf_latex(input_string)?;
    Ok(convert_pdf_png(&pdf_doc).expect("Image conversion failed!"))
}