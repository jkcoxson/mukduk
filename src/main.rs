// Jackson Coxson

use std::{
    process::Stdio,
    sync::atomic::{AtomicU32, Ordering},
};

use log::{error, info, warn};
use tokio::{io::copy_bidirectional, net::TcpStream, process::Command};

mod config;
mod loading;

static THREAD_COUNT: AtomicU32 = AtomicU32::new(0);

#[tokio::main]
async fn main() {
    println!("Initializing logger");
    env_logger::init();
    info!("Logger initialized!");

    // Get the first argument
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        error!("No config file supplied! Generate a default config with `mukduk write`");
        return;
    }
    let config_path = &args[1];

    // Write config if needed
    if config_path == "write" {
        if let Err(e) = config::Config::write() {
            error!("Unable to write config file! {:?}", e);
        }
        return;
    }

    // Read config file
    let config = match config::Config::load(config_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Change the directory as needed
    if let Err(e) = std::env::set_current_dir(&config.executable.path) {
        error!("Unable to cd to {}: {e}", config.executable.path);
    }

    // Create the command
    let mut command = Command::new(config.executable.command);
    for arg in config.executable.args {
        command.arg(arg);
    }
    if config.process.pipe {
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
        command.stdin(Stdio::inherit());
    }
    let mut child = None;
    let internal_port = loading::serve(config.process.load).await;

    info!("Starting proxy");
    if let Ok(listener) = tokio::net::TcpListener::bind(&config.bind).await {
        info!("Proxy started on {:?}", config.bind);
        loop {
            tokio::select! {
                Ok((mut stream, _)) = listener.accept() => {
                    info!("Accepted connection");
                    let target = match child {
                        Some(_) => {
                            format!("127.0.0.1:{}", config.process.port)
                        }
                        None => {
                            // Start the child and return loading page
                            info!("Starting child process");
                            child = Some(command.spawn().unwrap());
                            format!("127.0.0.1:{}", internal_port)
                        }
                    };
                    let mut target_stream = match TcpStream::connect(target).await {
                        Ok(stream) => stream,
                        Err(e) => {
                            if let Some(mut c) = child {
                                error!("Could not connect to target stream: {e}");
                                if let Err(e) = c.kill().await {
                                    error!("Unable to kill child process: {:?}", e);
                                }
                                child = None
                            }
                            continue;
                        }
                    };
                    // Proxy requests
                    tokio::spawn(async move {
                        info!("Proxying the request");
                        THREAD_COUNT.fetch_add(1, Ordering::SeqCst);
                        if let Err(e) = copy_bidirectional(&mut stream, &mut target_stream)
                            .await {
                                warn!("Bidrectional copy failed: {e}");
                            }
                        info!("Client disconnected");
                        THREAD_COUNT.fetch_sub(1, Ordering::SeqCst);
                    });
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(config.process.inactivity.into())) => {
                    if THREAD_COUNT.load(Ordering::SeqCst) == 0 {
                        if let Some(mut c) = child {
                            if let Err(e) = c.kill().await {
                                error!("Unable to kill child process: {:?}", e);
                            }
                            child = None;
                            info!("Child process killed due to inactivity")
                        }
                    }
                }
                else => {
                    warn!("Failed to accept connection");
                }
            }
        }
    } else {
        error!("Failed to bind to {}!", config.bind);
    }
}
