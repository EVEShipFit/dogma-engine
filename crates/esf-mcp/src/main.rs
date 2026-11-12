use std::path::PathBuf;

use clap::Parser;
use esf_mcp::{Game, Server, data};
use rmcp::ServiceExt;
use rmcp::transport::stdio;

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

fn load(args: &Args) -> Result<Game, String> {
    if let Some(sde) = &args.sde {
        let names = args.names.as_deref().map(data::read).transpose()?;
        return Game::new(data::read(sde)?, names);
    }

    let cache_dir = args
        .cache_dir
        .clone()
        .or_else(|| dirs::cache_dir().map(|dir| dir.join("esf-mcp")))
        .ok_or("no cache directory; pass --cache-dir or --sde")?;
    data::load(&cache_dir)
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

    let service = match Server::new(game).serve(stdio()).await {
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
