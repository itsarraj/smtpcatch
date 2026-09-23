use anyhow::Result;
use clap::Parser;
use colored::*;
use mailparse::*;
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 1025)]
    port: u16,

    /// Bind address
    #[arg(short, long, default_value = "127.0.0.1")]
    bind: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let addr: SocketAddr = format!("{}:{}", args.bind, args.port).parse()?;

    let listener = TcpListener::bind(addr).await?;
    println!("{} {}", "smtpcatch listening on".green(), addr);

    loop {
        let (socket, peer) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, peer).await {
                eprintln!("Error handling client {}: {}", peer, e);
            }
        });
    }
}

async fn handle_client(mut socket: TcpStream, peer: SocketAddr) -> Result<()> {
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    writer.write_all(b"220 smtpcatch ready\r\n").await?;

    let mut data_mode = false;
    let mut email_data = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }

        if data_mode {
            if line == ".\r\n" {
                data_mode = false;
                writer.write_all(b"250 OK\r\n").await?;

                println!("\n{}", "=".repeat(50).blue());
                println!("{} {}", "Received email from:".cyan(), peer);
                if let Ok(parsed) = parse_mail(email_data.as_bytes()) {
                    for header in parsed.get_headers() {
                        println!("{}: {}", header.get_key().yellow(), header.get_value());
                    }
                    if let Ok(body) = parsed.get_body() {
                        println!("\n{}", body);
                    }
                } else {
                    println!("{}", email_data);
                }
                println!("{}", "=".repeat(50).blue());

                email_data.clear();
            } else {
                if line.starts_with("..") {
                    email_data.push_str(&line[1..]);
                } else {
                    email_data.push_str(&line);
                }
            }
            continue;
        }

        let cmd = line.trim_end().to_uppercase();
        if cmd.starts_with("HELO") || cmd.starts_with("EHLO") {
            writer.write_all(b"250 Hello\r\n").await?;
        } else if cmd.starts_with("MAIL FROM:") || cmd.starts_with("RCPT TO:") {
            writer.write_all(b"250 OK\r\n").await?;
        } else if cmd == "DATA" {
            writer
                .write_all(b"354 End data with <CR><LF>.<CR><LF>\r\n")
                .await?;
            data_mode = true;
        } else if cmd == "QUIT" {
            writer.write_all(b"221 Bye\r\n").await?;
            break;
        } else {
            writer.write_all(b"500 Command not recognized\r\n").await?;
        }
    }

    Ok(())
}
