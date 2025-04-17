use anyhow::{Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UnixStream};

pub async fn send_to_unix_socket(
    socket_path: &str,
    command: &str,
    content: &str,
) -> Result<String> {
    let mut stream = UnixStream::connect(socket_path).await.context(format!(
        "Failed to connect to Unix socket at {}",
        socket_path
    ))?;

    // Send the command
    let mut message = command.to_string();
    message.push('\n');

    if command == "apply" {
        // Add the configuration content
        message.push_str(content);
        message.push('\n');
    }

    stream.write_all(message.as_bytes()).await?;
    stream.flush().await?;

    // Read the response
    let mut response = String::new();
    stream.read_to_string(&mut response).await?;

    Ok(response)
}

pub async fn send_to_tcp(host: &str, port: u16, command: &str, content: &str) -> Result<String> {
    let mut stream = TcpStream::connect(format!("{}:{}", host, port))
        .await
        .context(format!("Failed to connect to {}:{}", host, port))?;

    // Send the command
    let mut message = command.to_string();
    message.push('\n');

    if command == "apply" {
        // Add the configuration content
        message.push_str(content);
        message.push('\n');
    }

    stream.write_all(message.as_bytes()).await?;
    stream.flush().await?;

    // Read the response
    let mut response = String::new();
    stream.read_to_string(&mut response).await?;

    Ok(response)
}
