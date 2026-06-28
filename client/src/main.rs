mod agent;
mod client;
mod protocol;
mod state;

use std::str::FromStr;

use agent::{NeuralAgent, load_config, load_genome};
use client::ZappyClient;

const DEFAULT_HOST: &str = "localhost";
const CONFIG_PATH: &str = "arch.json";
const GENOME_PATH: &str = "best_genome.npy";

fn print_usage() {
    println!("USAGE: ./zappy_ai -p port -n name -h machine");
    println!("  -p port     port number");
    println!("  -n name     name of the team");
    println!("  -h machine  hostname of the server (default: {DEFAULT_HOST})");
}

fn default_env_variable<T>(var: Option<T>, env_var: &str) -> Option<T>
where
    T: FromStr,
{
    if var.is_some() {
        return var;
    }

    if let Ok(s) = std::env::var(env_var)
        && let Some(parsed) = s.parse::<T>().ok().map(Some)
    {
        return parsed;
    }

    var
}

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--help") {
        print_usage();
        return Ok(());
    }

    let mut port: Option<u16> = None;
    let mut team: Option<String> = None;
    let mut host = String::from(DEFAULT_HOST);

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-p" => {
                i += 1;
                let raw = args
                    .get(i)
                    .ok_or_else(|| anyhow::anyhow!("-p requires a value"))?;
                port = Some(
                    raw.parse::<u16>()
                        .map_err(|_| anyhow::anyhow!("Invalid port: {raw}"))?,
                );
            }
            "-n" => {
                i += 1;
                team = Some(
                    args.get(i)
                        .ok_or_else(|| anyhow::anyhow!("-n requires a value"))?
                        .clone(),
                );
            }
            "-h" => {
                i += 1;
                host = args
                    .get(i)
                    .ok_or_else(|| anyhow::anyhow!("-h requires a value"))?
                    .clone();
            }
            other => anyhow::bail!("Unknown argument: {other}"),
        }
        i += 1;
    }

    let port = default_env_variable(port, "ZAPPY_PORT")
        .ok_or_else(|| anyhow::anyhow!("-p port is required"))?;
    let team = default_env_variable(team, "ZAPPY_TEAMS")
        .ok_or_else(|| anyhow::anyhow!("-n name is required"))?;

    let config = match load_config(CONFIG_PATH) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {e:?}");
            return Err(e);
        }
    };
    let genome = match load_genome(GENOME_PATH) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Error loading genome: {e:?}");
            return Err(e);
        }
    };
    let agent = NeuralAgent { config, genome };

    let mut client = ZappyClient::connect(&host, port, &team)?;
    client.run(&agent)?;

    Ok(())
}
