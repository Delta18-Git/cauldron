use std::{fs, io, path::Path};

use typst::{
    diag::SourceDiagnostic,
    syntax::{FileId, VirtualPath},
};
use typst_as_lib::TypstEngine;

// TODO: Change constants to FS-walk + config
const site_base: &str = "./site";
const static_files: &str = "./site/static";
const template_base: &str = "./site/templates";
const content_base: &str = "./site/content";
const article_path: &str = "./site/content/article1/main.typ";
const output_base: &str = "./rendered";

fn main() {
    let typst_engine = TypstEngine::builder()
        .with_file_system_resolver(site_base)
        .build();
    let src = Path::new(article_path);
    typst_to_html(&typst_engine, src);
    let output_static = Path::new(output_base).join("static");
    if let Err(err) = copy_dir(static_files, &output_static) {
        eprintln!("couldn't copy output/static folder because of {err}");
    }
}

fn typst_to_html(engine: &TypstEngine, filepath: &Path) {
    let filename = filepath
        .strip_prefix(site_base)
        .expect("file path is not under base dir")
        .to_str()
        .expect("filename not in unicode");
    let doc = engine.compile::<_, typst_html::HtmlDocument>(filename);
    doc.warnings.iter().for_each(handle_source_diagnostic);

    match doc.output {
        Ok(document) => {
            let html = match typst_html::html(&document) {
                Ok(html_out) => html_out,
                Err(_) => unreachable!("typst_html::html doesn't return error"),
            };
            let output_path = Path::new(output_base).join(
                filepath
                    .strip_prefix(content_base)
                    .expect("filepath must be under base dir"),
            );
            let output_parent = output_path.parent().expect("content base must exist");
            match fs::create_dir_all(output_parent) {
                Ok(_) => match fs::write(output_path.with_extension("html"), html) {
                    Ok(_) => (),
                    Err(html_err) => eprintln!("Failed to write html because of {html_err}"),
                },
                Err(err) => {
                    let failed_path = output_parent.display();
                    eprintln!("Failed to create {failed_path} because of {err}")
                }
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

fn copy_dir(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
