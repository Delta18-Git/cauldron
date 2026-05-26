use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use typst::ecow::{EcoString, EcoVec, string::ToEcoString};

use crate::cauldron::CauldronInstance;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PostMeta {
    pub title: Option<EcoString>,
    pub author: Option<EcoString>,
    pub description: Option<EcoString>,
    pub date: Option<EcoString>, // RFC 3339
    pub tags: Option<EcoVec<EcoString>>,
    pub collections: Option<EcoVec<EcoString>>,
}

#[derive(Clone, Debug)]
pub struct Post {
    pub file_path: PathBuf,
    pub metadata: PostMeta,
    pub link: EcoString,
    pub shortlink: EcoString, // permashortlink
}

#[derive(Clone, Serialize)]
pub struct TagIndex {
    title: Option<EcoString>,
    author: Option<EcoString>,
    description: Option<EcoString>,
    date: Option<EcoString>,
    link: EcoString,
}

impl CauldronInstance {
    pub fn parse_meta(&self, typst_filepath: &Path) -> Result<Post, String> {
        let meta_file = typst_filepath.with_file_name("meta.toml");
        let meta = match fs::read_to_string(&meta_file) {
            Ok(meta) => meta,
            Err(err) => Err(format!(
                "Failed to read metadata at path {}: {err}",
                &meta_file.display()
            ))?,
        };
        let post_meta: PostMeta = toml::from_str(&meta).map_err(|err| {
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
            file_path: meta_file,
            metadata: post_meta,
            link: link.clone(),
            shortlink: link, // TODO: implement shortlink generation later
        })
    }

    pub fn build_collection(&self) -> Result<(), String> {
        // Build _index.toml — array of tables, each a full PostMeta
        let posts: Vec<&PostMeta> = self.posts.values().map(|p| &p.metadata).collect();
        let index_content = HashMap::from([("posts", &posts)]);
        let index_toml = toml::to_string(&index_content)
            .map_err(|err| format!("Failed to serialize index: {err}"))?;
        fs::write(&self.config.output_base.join("_index.toml"), index_toml)
            .map_err(|err| format!("Failed to write index: {err}"))?;

        // Build _tags.toml — tag-name → array of TagIndex entries
        let mut tags_map: HashMap<EcoString, Vec<TagIndex>> = HashMap::new();
        for post in self.posts.values() {
            let entry = TagIndex {
                title: post.metadata.title.clone(),
                author: post.metadata.author.clone(),
                description: post.metadata.description.clone(),
                date: post.metadata.date.clone(),
                link: post.link.clone(),
            };
            if let Some(tags) = &post.metadata.tags {
                for tag in tags {
                    tags_map.entry(tag.clone()).or_default().push(entry.clone());
                }
            }
            if let Some(collections) = &post.metadata.collections {
                for collection in collections {
                    tags_map
                        .entry(collection.clone())
                        .or_default()
                        .push(entry.clone());
                }
            }
        }
        let tags_toml = toml::to_string(&tags_map)
            .map_err(|err| format!("Failed to serialize tags index: {err}"))?;
        fs::write(&self.config.output_base.join("_tags.toml"), tags_toml)
            .map_err(|err| format!("Failed to write tags index: {err}"))?;

        Ok(())
    }
}
