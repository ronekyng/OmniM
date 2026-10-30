use std::env;
use std::sync::Arc;

use omnim_model::{Library, Project, LIBRARY};
use omnim_server::{default_projects_root, ServerConfig, SharedProject};
use tokio::sync::RwLock;

fn init_library() {
    let lib_path = dirs::home_dir().map(|p| p.join(".omnim").join("library.omnimlib"));

    let library = match &lib_path {
        Some(p) if p.exists() => Library::load_from_file(p.to_str().unwrap()).unwrap_or_else(|e| {
            eprintln!(
                "Warning: failed to load library from {}: {}",
                p.display(),
                e
            );
            eprintln!("Falling back to built-in seeded library.");
            Library::seeded()
        }),
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

/// OMNIM_EPHEMERAL=1|true|yes → run fully in memory (no file writes).
fn is_ephemeral() -> bool {
    matches!(
        env::var("OMNIM_EPHEMERAL")
            .unwrap_or_default()
            .to_lowercase()
            .as_str(),
        "1" | "true" | "yes"
    )
}

fn main() {
    init_library();

    // Contract: the workspace is not bound to the working directory. The
    // default root is `OMNIM_PROJECTS_DIR` or `~/.omnim/projects`; an explicit
    // `OMNIM_PROJECT` still wins for scripted runs.
    let project_file = env::var("OMNIM_PROJECT").unwrap_or_else(|_| {
        default_projects_root()
            .join("project.omnim")
            .to_string_lossy()
            .into_owned()
    });
    let proj = load_or_new(&project_file);

    // Loopback by default: the API is unauthenticated, so binding all
    // interfaces out of the box would expose the whole project to the LAN
    // (`UI-SEC-00`). Set OMNIM_HOST=0.0.0.0 (or any specific address) to
    // publish deliberately.
    let host = env::var("OMNIM_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = env::var("OMNIM_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    // Persistence: ON by default. The project file always persists; setting
    // OMNIM_EPHEMERAL=1/true/yes runs the server fully in memory (useful for
    // demos and for tests that must not touch the working directory).
    let save_path = if is_ephemeral() {
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
