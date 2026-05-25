use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::Deserialize;
use typst_as_lib::TypstEngine;

use crate::util::{self, handle_source_diagnostic};

pub struct CauldronInstance {
    pub config: Config,
    pub engine: TypstEngine,
    link_maps: LinkMaps,
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub site_base: PathBuf, // contains ./{static,templates,content}
    pub static_sub: String,
    pub template_sub: String,
    pub content_sub: String,
    pub output_base: PathBuf,
    pub serve: bool,
    pub host: String,
    pub port: String,
}

struct LinkMaps {
    permalinks: Arc<HashMap<String, String>>, // link -> permalink
    shortlinks: Arc<HashMap<String, String>>, // short -> permalink
    redirects: Arc<HashMap<String, String>>,
}

impl CauldronInstance {
    pub fn new(config: Config, engine: TypstEngine) -> Self {
        CauldronInstance {
            config,
            engine,
            link_maps: LinkMaps {
                permalinks: Arc::new(HashMap::new()),
                shortlinks: Arc::new(HashMap::new()),
                redirects: Arc::new(HashMap::new()),
            },
        }
    }

    pub fn render(&self, filepath: &Path) -> Result<(), String> {
        let html = self.typst_to_html(filepath)?;
        let relative_path = filepath
            .strip_prefix(self.config.site_base.join(&self.config.content_sub))
            .map_err(|_| "Filepath must be under content base".to_string())?;
        let output_path = Path::new(&self.config.output_base).join(relative_path);
        self.write_html(&output_path, &html)
    }

    pub fn typst_to_html(&self, filepath: &Path) -> Result<String, String> {
        let filename = filepath
            .strip_prefix(&self.config.site_base)
            .map_err(|_| "File path is not under base dir".to_string())?
            .to_str()
            .ok_or_else(|| "Filename not in unicode".to_string())?;
        let doc = self.engine.compile::<_, typst_html::HtmlDocument>(filename);
        doc.warnings.iter().for_each(handle_source_diagnostic);

        match doc.output {
            Ok(document) => {
                let html = typst_html::html(&document)
                    .map_err(|_| "typst_html::html failed unexpectedly".to_string())?;
                Ok(html)
            }
            Err(err) => Err(format!("Failed because of TypstAsLibErr: {err}")),
        }
    }

    pub fn write_html(&self, output_path: &Path, html: &str) -> Result<(), String> {
        let output_parent = output_path.parent().ok_or_else(|| {
            format!(
                "Output path {} has no parent, cannot create directories",
                output_path.display()
            )
        })?;
        std::fs::create_dir_all(output_parent)
            .map_err(|err| format!("Failed to create {}: {}", output_parent.display(), err))?;
        std::fs::write(output_path.with_extension("html"), html)
            .map_err(|err| format!("Failed to write html: {err}"))?;
        Ok(())
    }

    pub fn copy_static(&self) -> Result<(), io::Error> {
        util::copy_dir(
            &self.config.site_base.join(&self.config.static_sub),
            &self.config.output_base.join(&self.config.static_sub),
        )
    }
}
