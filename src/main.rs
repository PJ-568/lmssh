use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "lmssh")]
struct Args {
    /// 配置文件路径（默认为 ~/.config/lmssh/config.toml；优先 XDG_CONFIG_HOME）
    #[arg(short = 'c', long = "config")]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

async fn run() -> lmssh::error::Result<()> {
    let args = Args::parse();

    let config_path = args
        .config
        .unwrap_or_else(lmssh::config::default_config_path);
    let cfg = lmssh::config::Config::load_from_path(config_path)?;

    // Task 2：先打通 main 到 server 的调用链；server 先为空实现。
    lmssh::ssh::server::run_server(&cfg).await?;
    Ok(())
}
