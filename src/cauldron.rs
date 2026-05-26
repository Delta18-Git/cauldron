use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use typst::{
    ecow::{EcoString, EcoVec, string::ToEcoString},
    foundations::IntoValue,
};
use typst_as_lib::TypstEngine;
use walkdir::WalkDir;

use crate::{
    post::{Frontmatter, Post, posts_to_dict},
    util::{self, handle_source_diagnostic},
};

pub struct CauldronInstance {
    pub config: Config,
    pub engine: TypstEngine,
    link_maps: LinkMaps,
    pub posts: HashMap<PathBuf, Post>,
    pub tag_map: HashMap<EcoString, EcoVec<PathBuf>>,
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub site_base: PathBuf,
    pub static_sub: String,
    pub template_sub: String,
    pub content_sub: String,
    pub output_base: PathBuf,
    pub tags_template: Option<String>,
    pub posts_template: Option<String>,
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
            tag_map: HashMap::new(),
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

    pub fn render_recurse(&mut self) -> Result<(), String> {
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
        let post = self.parse_meta(filepath)?;
        self.index_tags(&post);
        self.posts.insert(filepath.to_path_buf(), post);
        let html = self.typst_to_html(filepath)?;
        let relative_path = filepath
            .strip_prefix(self.config.site_base.join(&self.config.content_sub))
            .map_err(|_| "Filepath must be under content base".to_string())?;
        let output_path = Path::new(&self.config.output_base).join(relative_path);
        self.write_html(&output_path, &html)
    }

    pub fn render_tag(&mut self, tag: EcoString) -> Result<(), String> {
        let html = match &self.config.tags_template {
            Some(template) => {
                let posts_for_tag: EcoVec<_> = self
                    .tag_map
                    .get(&tag)
                    .ok_or(format!("No posts for tag {tag}"))?
                    .iter()
                    .map(|path| self.posts.get(path).unwrap().clone()) // unwrap shouldn't fail as both maps have same set of paths
                    .collect();
                let mut dict = posts_to_dict(posts_for_tag);
                dict.insert("tag".into(), tag.clone().into_value());
                let doc = self
                    .engine
                    .compile_with_input::<_, _, typst_html::HtmlDocument>(
                        Path::new(&self.config.template_sub)
                            .join(template)
                            .display()
                            .to_string()
                            .as_str(), // what is this cursed crap
                        dict,
                    );
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
            None => Err(format!("Tag template doesn't exist")), // should log here
        }?;
        let output_path = Path::new(&self.config.output_base)
            .join("tags/")
            .join(format!("{tag}.html"));
        self.write_html(&output_path, &html)
    }

    pub fn render_post_index(&mut self) -> Result<(), String> {
        let html = match &self.config.posts_template {
            Some(template) => {
                let posts: EcoVec<_> = self.posts.values().cloned().collect();
                let doc = self
                    .engine
                    .compile_with_input::<_, _, typst_html::HtmlDocument>(
                        Path::new(&self.config.template_sub)
                            .join(template)
                            .display()
                            .to_string()
                            .as_str(), // what is this cursed crap
                        posts_to_dict(posts),
                    );
                doc.warnings.iter().for_each(handle_source_diagnostic);

                match doc.output {
                    Ok(document) => {
                        let html = typst_html::html(&document)
                            .map_err(|_| "typst_html::html failed unexpectedly".to_string())?;
                        Ok(html)
                    }
                    Err(err) => Err(format!("Failed because of TypstAsLibErr: {err}").to_string()),
                }
            }
            None => Err(format!("Posts template doesn't exist").to_string()), // should log here
        }?;
        let output_path = Path::new(&self.config.output_base)
            .join("posts/")
            .join(format!("index.html"));
        self.write_html(&output_path, &html)
    }

    pub fn render_tags(&mut self) -> Result<(), String> {
        let tags: Vec<_> = self.tag_map.keys().cloned().collect();
        for tag in tags {
            self.render_tag(tag)?;
        }
        Ok(())
    }

    pub fn index_all_tags(&mut self) {
        self.tag_map.clear();
        for post in self.posts.values() {
            for tag in post.frontmatter.tags.clone().unwrap_or_default() {
                self.tag_map
                    .entry(tag.clone())
                    .and_modify(|vec| vec.push(post.file_path.clone()))
                    .or_insert({
                        let mut vec = EcoVec::new();
                        vec.push(post.file_path.clone());
                        vec
                    });
            }
        }
    }

    pub fn index_tags(&mut self, post: &Post) {
        for tag in post.frontmatter.tags.clone().unwrap_or_default() {
            self.tag_map
                .entry(tag.clone())
                .and_modify(|vec| vec.push(post.file_path.to_path_buf()))
                .or_insert({
                    let mut vec = EcoVec::new();
                    vec.push(post.file_path.to_path_buf());
                    vec
                });
        }
    }

    pub fn parse_meta(&self, typst_filepath: &Path) -> Result<Post, String> {
        let meta_file = typst_filepath.with_file_name("meta.toml");
        let meta = match fs::read_to_string(&meta_file) {
            Ok(meta) => meta,
            Err(err) => Err(format!(
                "Failed to read metadata at path {}: {err}",
                &meta_file.display()
            ))?,
        };
        let post_meta: Frontmatter = toml::from_str(&meta).map_err(|err| {
            format!(
                "Failed to deserialise metadata at {}: {err}",
                meta_file.display()
            )
        })?;
        let link = typst_filepath
            .strip_prefix(self.config.site_base.join(&self.config.content_sub))
            .map_err(|_| format!("Couldn't generate link for {}", typst_filepath.display()))?
            .with_extension("html")
            .display()
            .to_eco_string();
        Ok(Post {
            file_path: PathBuf::from(typst_filepath),
            frontmatter: post_meta,
            link: link.clone(),
            shortlink: link, // TODO: implement shortlink generation later
        })
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
