use anyhow::{Error, Result};
use clap::{Parser, Subcommand};
use dht::NodeClient;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::net::SocketAddr;
use tarpc::{client, context, tokio_serde::formats::Json};

#[derive(Debug, Parser)]
struct Flags {
    /// Server Address
    #[clap(long)]
    server_addr: SocketAddr,
}

/// Dht Client Interface
#[derive(Debug, Parser)]
struct Repl {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Get(CmdGet),
    Set(CmdSet),
    Del(CmdDel),
}

/// Get the value in the store associated with the key
#[derive(Debug, Parser)]
struct CmdGet {
    key: String,
}

/// Insert key-value pair into the store
#[derive(Debug, Parser)]
struct CmdSet {
    key: String,
    value: String,
}

/// Delete the key-value pair in the store
#[derive(Debug, Parser)]
struct CmdDel {
    key: String,
}

/// Handles get command
async fn cmd_get(cmd: CmdGet, client: &mut NodeClient) {
    let resp = async move {
        tokio::select! {
            resp = client.get(context::current(), cmd.key) => { resp }
        }
    }
    .await;

    match resp {
        Ok(msg) => println!("{}", msg.unwrap_or("nil".to_string())),
        Err(e) => eprintln!("Error: {}", Error::from(e)),
    }
}

/// Handles set command
async fn cmd_set(cmd: CmdSet, client: &mut NodeClient) {
    let resp = async move {
        tokio::select! {
            resp = client.insert(context::current(), cmd.key, cmd.value) => { resp }
        }
    }
    .await;

    match resp {
        Ok(_) => println!("OK"),
        Err(e) => eprintln!("Error: {}", Error::from(e)),
    }
}

/// Handles del command
async fn cmd_del(cmd: CmdDel, client: &mut NodeClient) {
    let resp = async move {
        tokio::select! {
            resp = client.remove(context::current(), cmd.key) => { resp }
        }
    }
    .await;

    match resp {
        Ok(msg) => {
            println!("{}", if msg.is_none() { 0 } else { 1 })
        }
        Err(e) => eprintln!("Error: {}", Error::from(e)),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let flags = Flags::parse();
    let mut transport = tarpc::serde_transport::tcp::connect(flags.server_addr, Json::default);
    transport.config_mut().max_frame_length(usize::MAX);

    let mut client = NodeClient::new(client::Config::default(), transport.await?).spawn();

    let mut rl = DefaultEditor::new()?;
    #[cfg(feature = "with-file-history")]
    if rl.load_history("dht_cli_history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("dht> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let repl = Repl::try_parse_from(format!("dht {}", line).split(' '));
                if let Err(e) = repl {
                    println!("{}", e);
                    continue;
                }

                let repl = repl.unwrap();
                match repl.command {
                    Commands::Get(cmd) => {
                        cmd_get(cmd, &mut client).await;
                    }
                    Commands::Set(cmd) => {
                        cmd_set(cmd, &mut client).await;
                    }
                    Commands::Del(cmd) => {
                        cmd_del(cmd, &mut client).await;
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    #[cfg(feature = "with-file-history")]
    rl.save_history("dht_cli_history.txt");

    Ok(())
}
