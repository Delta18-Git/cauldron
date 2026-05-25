use std::{
    collections::HashMap,
    io::Result,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::cauldron::CauldronInstance;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PostMeta {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub date: Option<String>, // RFC 3339
    pub tags: Option<Vec<String>>,
    pub collections: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct Post {
    pub file_path: PathBuf,
    pub metadata: PostMeta,
    pub link: String,
    pub shortlink: String, // permashortlink
}

#[derive(Serialize)]
pub struct TagIndex {
    title: Option<String>,
    author: Option<String>,
    description: Option<String>,
    date: Option<String>,
    link: String,
}

impl CauldronInstance {
    fn build_collection(posts: &[Post], out_dir: &Path) -> Result<()> {
        let post_list: Vec<_> = posts
            .iter()
            .map(|p| match toml::to_string(&p.metadata) {
                Ok(string) => string,
                Err(err) => {
                    eprintln!("{err} when serialising metadata to toml");
                    "".to_string()
                }
            })
            .collect();
        std::fs::write(
            out_dir.join("_index.toml"),
            match toml::to_string(&post_list) {
                Ok(toml) => toml,
                Err(err) => unreachable!("{err} how"),
            },
        )?;

        let mut tags_map: HashMap<String, Vec<String>> = HashMap::new();
        for post in posts {
            let vec = Vec::default();
            let index_entry = TagIndex {
                title: post.metadata.title.clone(),
                author: post.metadata.author.clone(),
                description: post.metadata.description.clone(),
                date: post.metadata.date.clone(),
                link: post.link.clone(),
            };
            for tag in match &post.metadata.tags {
                Some(tags) => tags,
                None => &vec,
            } {
                tags_map.entry(tag.to_string()).or_default().push(
                    match toml::to_string(&index_entry) {
                        Ok(toml) => toml,
                        Err(err) => {
                            eprintln!("{err} when serialising metadata to toml");
                            "".to_string()
                        }
                    },
                );
            }
            for collection in match &post.metadata.collections {
                Some(tags) => tags,
                None => &vec,
            } {
                tags_map.entry(collection.to_string()).or_default().push(
                    match toml::to_string(&index_entry) {
                        Ok(toml) => toml,
                        Err(err) => {
                            eprintln!("{err} when serialising metadata to toml");
                            "".to_string()
                        }
                    },
                );
            }
        }

        std::fs::write(
            out_dir.join("_tags.toml"),
            match toml::to_string(&tags_map) {
                Ok(toml) => toml,
                Err(err) => unreachable!("{err} how"),
            },
        )?;

        Ok(())
    }
}
