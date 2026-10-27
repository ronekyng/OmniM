use std::env;
use std::sync::Arc;

use omnim_model::{Library, Project, LIBRARY};
use omnim_server::{ServerConfig, SharedProject};
use tokio::sync::RwLock;

const DEFAULT_PROJECT_FILE: &str = "project.omnim";

fn init_library() {
    let lib_path = dirs::home_dir()
        .map(|p| p.join(".omnim").join("library.omnimlib"));

    let library = match &lib_path {
        Some(p) if p.exists() => {
            Library::load_from_file(p.to_str().unwrap())
                .unwrap_or_else(|e| {
                    eprintln!("Warning: failed to load library from {}: {}", p.display(), e);
                    eprintln!("Falling back to built-in seeded library.");
                    Library::seeded()
                })
        }
        _ => Library::seeded(),
    };
    *LIBRARY.write().unwrap() = Some(library);
}

fn load_or_new(path: &str) -> Project {
    match Project::load(path) {
        Ok(proj) => proj,
        Err(_) => Project::new("OmniM Project"),
    }
}

fn main() {
    init_library();

    let project_file = env::var("OMNIM_PROJECT").unwrap_or_else(|_| DEFAULT_PROJECT_FILE.to_string());
    let proj = load_or_new(&project_file);

    let host = env::var("OMNIM_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("OMNIM_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let save_path = if project_file == DEFAULT_PROJECT_FILE {
        None
    } else {
        Some(project_file.clone())
    };

    let rt = tokio::runtime::Runtime::new().expect("create tokio runtime");
    rt.block_on(async {
        let project: SharedProject = Arc::new(RwLock::new(proj));
        let cfg = ServerConfig {
            host,
            port,
            project,
            save_path,
        };
        omnim_server::start(cfg).await;
    });
}