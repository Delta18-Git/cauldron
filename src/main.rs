mod cauldron;
mod post;
mod util;
use std::path::PathBuf;

use typst_as_lib::TypstEngine;

use crate::cauldron::{CauldronInstance, Config};

// TODO: Change to FS-walk
fn main() {
    let config = Config {
        site_base: PathBuf::from("./site"),
        static_sub: String::from("static"),
        template_sub: String::from("templates"),
        content_sub: String::from("content"),
        output_base: PathBuf::from("./rendered"),
        serve: false,
        host: String::new(),
        port: String::new(),
    };
    let mut cauldron = CauldronInstance::new(
        config.clone(),
        TypstEngine::builder()
            .with_file_system_resolver(&config.site_base)
            .build(),
    );
    if let Err(err) = cauldron.verify_structure() {
        eprintln!("{err}");
        return;
    }
    if let Err(err) = cauldron.render_all() {
        eprintln!("Error: {err}");
    };
    if let Err(err) = cauldron.build_collection() {
        eprintln!("Error: {err}");
    };
    if let Err(err) = cauldron.copy_static() {
        eprintln!("Error: {err}");
    }
}
