use std::path::PathBuf;

use clap::Parser;
use rmcp::ServiceExt;
use rmcp::transport::stdio;

mod data;
mod fits;
mod game;
mod server;
mod stats;

/// MCP server, over stdio, that calculates EVE Online ship fits.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// A local sde.dat.
    #[clap(long)]
    sde: Option<PathBuf>,

    /// A local names.dat.
    #[clap(long, requires = "sde")]
    names: Option<PathBuf>,

    /// Where a downloaded SDE is kept. Default: the user's cache directory.
    #[clap(long)]
    cache_dir: Option<PathBuf>,
}

fn read(path: &PathBuf) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn load(args: &Args) -> Result<game::Game, String> {
    if let Some(sde) = &args.sde {
        return game::Game::new(read(sde)?, args.names.as_ref().map(read).transpose()?);
    }

    let cache_dir = args
        .cache_dir
        .clone()
        .or_else(|| dirs::cache_dir().map(|dir| dir.join("esf-mcp")))
        .ok_or("no cache directory; pass --cache-dir or --sde")?;
    let dir = data::cached(&cache_dir)?;
    let loaded = read(&dir.join("sde.dat"))
        .and_then(|sde| game::Game::new(sde, Some(read(&dir.join("names.dat"))?)));
    match &loaded {
        Ok(_) => data::prune(&cache_dir, &dir),
        Err(_) => {
            let _ = std::fs::remove_dir_all(&dir);
        }
    }
    loaded
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let game = match load(&args) {
        Ok(game) => game,
        Err(error) => {
            eprintln!("esf-mcp: {error}");
            std::process::exit(1);
        }
    };

    let service = match server::Server::new(game).serve(stdio()).await {
        Ok(service) => service,
        Err(error) => {
            eprintln!("esf-mcp: {error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = service.waiting().await {
        eprintln!("esf-mcp: {error}");
        std::process::exit(1);
    }
}
