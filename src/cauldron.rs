use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::Deserialize;
use typst::ecow::{EcoString, EcoVec};
use typst_as_lib::TypstEngine;
use walkdir::WalkDir;

use crate::{
    post::{Post, PostMeta},
    util::{self, handle_source_diagnostic},
};

pub struct CauldronInstance {
    pub config: Config,
    pub engine: TypstEngine,
    link_maps: LinkMaps,
    pub posts: HashMap<PathBuf, Post>,
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub site_base: PathBuf,
    pub static_sub: String,
    pub template_sub: String,
    pub content_sub: String,
    pub output_base: PathBuf,
    pub serve: bool,
    pub host: String,
    pub port: String,
}

struct LinkMaps {
    permalinks: HashMap<EcoString, EcoString>, // link -> permalink
    shortlinks: HashMap<EcoString, EcoString>, // short -> permalink
    redirects: HashMap<EcoString, EcoString>,
}

impl CauldronInstance {
    pub fn new(config: Config, engine: TypstEngine) -> Self {
        CauldronInstance {
            config,
            engine,
            link_maps: LinkMaps {
                permalinks: HashMap::new(),
                shortlinks: HashMap::new(),
                redirects: HashMap::new(),
            },
            posts: HashMap::new(),
        }
    }

    pub fn verify_structure(&self) -> Result<(), String> {
        let config = &self.config;
        for sub_dir in [
            &config.content_sub,
            &config.template_sub,
            &config.static_sub,
        ] {
            match fs::exists(config.site_base.join(sub_dir)) {
                Ok(ok) => {
                    if !ok {
                        Err(format!(
                            "{}/ doesn't exist under {}",
                            sub_dir,
                            &config.site_base.display()
                        ))?
                    }
                }
                Err(err) => Err(format!(
                    "Couldn't validate existence of {} {err}",
                    &config.site_base.join(sub_dir).display(),
                ))?,
            }
        }
        Ok(())
    }

    pub fn render_all(&mut self) -> Result<(), String> {
        for entry in WalkDir::new(&self.config.site_base.join(&self.config.content_sub))
            .contents_first(true)
            .into_iter()
            .filter_entry(|e| {
                e.file_type().is_dir()
                    || e.file_name()
                        .display()
                        .to_string()
                        .split_once(".")
                        .unwrap_or_default()
                        .1
                        .eq_ignore_ascii_case("typ")
            })
        {
            let entry = entry
                .map_err(|err| format!("Couldn't yield directory entry during FS-Walk: {err}"))?;
            if !entry.file_type().is_file() {
                continue;
            }
            self.render(entry.path())?;
        }
        Ok(())
    }

    pub fn render(&mut self, filepath: &Path) -> Result<(), String> {
        let meta = self.parse_meta(filepath)?;
        self.posts.insert(filepath.to_path_buf(), meta);
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
        fs::create_dir_all(output_parent)
            .map_err(|err| format!("Failed to create {}: {}", output_parent.display(), err))?;
        fs::write(output_path.with_extension("html"), html)
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
