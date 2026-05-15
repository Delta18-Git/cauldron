use std::{fs, path::PathBuf};

use typst::diag::SourceDiagnostic;
use typst_as_lib::TypstEngine;

fn main() {
    let src = PathBuf::from("./templates/main.typ");
    typst_to_html(src, "./templates/");
}

fn typst_to_html(filepath: PathBuf, template_dir: &str) {
    let template = TypstEngine::builder()
        .with_file_system_resolver(template_dir)
        .build();
    let filename = filepath
        .file_name()
        .expect("file path is root or a directory")
        .to_str()
        .expect("filename not in unicode");
    let doc = template.compile::<_, typst_html::HtmlDocument>(filename);
    doc.warnings
        .iter()
        .for_each(|srcdiag| handle_source_diagnostic(srcdiag));

    match doc.output {
        Ok(document) => {
            let html = typst_html::html(&document)
                .unwrap_or_else(|_| unreachable!("typst_html::html never returns an error"));
            match fs::write(filepath.with_extension("html"), html) {
                Ok(_) => (),
                Err(html_err) => eprintln!("Failed to write html because of {html_err}"),
            }
        }
        Err(err) => println!("Failed because of TypstAsLibErr: {err}"),
    }
}

fn handle_source_diagnostic(diagnostic: &SourceDiagnostic) {
    let level = match &diagnostic.severity {
        typst::diag::Severity::Error => "ERROR",
        typst::diag::Severity::Warning => "WARN",
    };
    let message = &diagnostic.message;
    let hints = diagnostic.hints.join("\n");
    eprintln!("[{level}] {message}\n{hints}")
}
