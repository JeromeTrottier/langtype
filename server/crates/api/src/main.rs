use anyhow::Context;
use clap::{Parser, Subcommand};
use mnemo_api::{ApiDoc, AppState, config::Config, router, security};
use mnemo_store::Store;
use std::{net::SocketAddr, sync::Arc};
use utoipa::OpenApi;

#[derive(Parser)]
#[command(name = "mnemo", version, about = "Mnemo: structured memory for Claude agents")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run the server on 127.0.0.1 (default).
    Serve,
    /// Print the OpenAPI document (used to generate the client's types).
    Openapi,
    /// Bridge MCP over stdio to the local server, for Claude Desktop (M2).
    McpStdio {
        /// Knowledge base id.
        #[arg(long)]
        kb: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mnemo=debug".into()),
        )
        .with_writer(std::io::stderr)
        .init();

    match Cli::parse().command.unwrap_or(Command::Serve) {
        Command::Serve => serve().await,
        Command::Openapi => {
            println!("{}", ApiDoc::openapi().to_pretty_json()?);
            Ok(())
        }
        Command::McpStdio { kb } => {
            anyhow::bail!("mcp-stdio for {kb}: not implemented yet (milestone M2)")
        }
    }
}

async fn serve() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    std::fs::create_dir_all(&config.data_dir).context("creating data dir")?;
    let token = security::load_or_create_token(&config.data_dir).context("loading local token")?;
    let store = Store::open(&config.database_url).await.context("opening database")?;
    let identity = store.ensure_local_identity().await?;
    let state = AppState {
        store,
        identity,
        security: Arc::new(security::LocalSecurity::new(token, config.port, &config.extra_origins)),
    };

    // Local mode: never listen on other interfaces.
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, token_file = %config.data_dir.join("local_token").display(), "mnemo listening");
    axum::serve(listener, router(state)).await?;
    Ok(())
}
