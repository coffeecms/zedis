#![allow(unexpected_cfgs)]
#![allow(unused_imports)]
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod config;
mod server;
mod core;
mod io;
mod security;
mod hardware;
mod persistence;
mod scripting;
mod compatibility;
mod flow;


use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("Starting Zedis (God Tier) - High Performance In-Memory Database");
    
    // Hardware Discovery (Simplified for now)
    let core_count = num_cpus::get();
    info!("Detected {} CPU cores. Initializing Thread-per-Core architecture...", core_count);

    // Config Load
    let conf = config::Config::default();


    // Start Server
    server::run(conf).await?;

    Ok(())
}
