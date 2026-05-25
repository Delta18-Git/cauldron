use std::{io::Result, path::Path};

use typst::{
    diag::SourceDiagnostic,
    syntax::{FileId, VirtualPath},
};

pub fn copy_dir(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn handle_source_diagnostic(diagnostic: &SourceDiagnostic) {
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
