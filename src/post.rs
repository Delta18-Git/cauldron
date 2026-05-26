use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use typst::{
    ecow::{EcoString, EcoVec, string::ToEcoString},
    foundations::{Dict, IntoValue},
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Frontmatter {
    pub title: Option<EcoString>,
    pub author: Option<EcoString>,
    pub description: Option<EcoString>,
    pub date: Option<EcoString>, // RFC 3339
    pub tags: Option<EcoVec<EcoString>>,
}

#[derive(Clone, Debug)]
pub struct Post {
    pub file_path: PathBuf,
    pub frontmatter: Frontmatter,
    pub link: EcoString,
    pub shortlink: EcoString, // permashortlink
}

pub fn posts_to_dict(posts: impl IntoIterator<Item = Post>) -> Dict {
    let mut dict = Dict::new();
    dict.insert(
        "posts".into(),
        posts
            .into_iter()
            .map(|p| Dict::from(p).into_value())
            .collect::<Vec<_>>()
            .into_value(),
    );
    dict
}

impl From<Post> for Dict {
    fn from(value: Post) -> Self {
        let mut dict = Dict::new();
        dict.insert(
            "file_path".into(),
            value.file_path.display().to_eco_string().into_value(),
        );
        dict.insert("link".into(), value.link.into_value());
        dict.insert("shortlink".into(), value.shortlink.into_value());
        let mut frontmatter = Dict::new();
        if let Some(title) = &value.frontmatter.title {
            frontmatter.insert("title".into(), title.as_str().into_value());
        }
        if let Some(author) = &value.frontmatter.author {
            frontmatter.insert("author".into(), author.as_str().into_value());
        }
        if let Some(description) = &value.frontmatter.description {
            frontmatter.insert("description".into(), description.as_str().into_value());
        }
        if let Some(date) = &value.frontmatter.date {
            frontmatter.insert("date".into(), date.as_str().into_value());
        }
        if let Some(tags) = &value.frontmatter.tags {
            let tags: Vec<_> = tags.iter().map(|t| t.as_str().into_value()).collect();
            frontmatter.insert("tags".into(), tags.into_value());
        }
        dict.insert("frontmatter".into(), frontmatter.into_value());
        dict
    }
}
