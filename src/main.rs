use std::{fs, path::Path};

use typst::{
    diag::SourceDiagnostic,
    syntax::{FileId, VirtualPath},
};
use typst_as_lib::TypstEngine;

fn main() {
    let typst_engine = TypstEngine::builder()
        .with_file_system_resolver("./templates/")
        .build();
    let src = Path::new("./templates/main.typ");
    typst_to_html(&typst_engine, src);
}

fn typst_to_html(engine: &TypstEngine, filepath: &Path) {
    let filename = filepath
        .file_name()
        .expect("file path is root or a directory")
        .to_str()
        .expect("filename not in unicode");
    let doc = engine.compile::<_, typst_html::HtmlDocument>(filename);
    doc.warnings
        .iter()
        .for_each(|srcdiag| handle_source_diagnostic(srcdiag));

    match doc.output {
        Ok(document) => {
            let html = match typst_html::html(&document) {
                Ok(html_out) => html_out,
                Err(_) => unreachable!("typst_html::html doesn't return error"),
            };
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
    let file = &diagnostic
        .span
        .id()
        .unwrap_or(FileId::new_fake(VirtualPath::new(".")))
        .vpath()
        .as_rootless_path()
        .display();
    let hints = diagnostic.hints.join("\n");
    eprintln!("[{level}] {file} {message}\n{hints}")
}
