use crate::connection::manager::ConnectionManager;
use crate::protocol::BinaryMessage;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};

/// Handle a WebSocket connection from a client
///
/// This function accepts a TCP stream, upgrades it to WebSocket,
/// and manages the bidirectional communication with the client.
pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    manager: Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("New WebSocket connection from: {}", addr);

    // Upgrade TCP stream to WebSocket
    let ws_stream = accept_async(stream).await?;
    tracing::debug!("WebSocket handshake completed for: {}", addr);

    // Split the WebSocket into sender and receiver
    let (mut write, mut read) = ws_stream.split();

    // Create unbounded channel for outgoing messages
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Register connection with manager
    manager.connect(addr, tx.clone()).await;

    // Spawn task to handle outbound messages
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            // Send message via WebSocket
            if let Err(e) = write.send(msg).await {
                tracing::error!("Failed to send message: {}", e);
                break;
            }
        }
        tracing::debug!("Outbound message task completed");
    });

    // Process inbound messages
    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Binary(data)) => {
                // Decode binary message
                match BinaryMessage::decode(&data) {
                    Ok(decoded_msg) => {
                        // Route to ConnectionManager
                        manager.handle_message(addr, decoded_msg).await;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to decode message from {}: {}", addr, e);
                        // Continue processing other messages
                    }
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!("Client {} initiated close", addr);
                break;
            }
            Ok(Message::Ping(data)) => {
                // Respond with Pong for keep-alive
                if let Err(e) = tx.send(Message::Pong(data)) {
                    tracing::error!("Failed to queue pong response: {}", e);
                    break;
                }
            }
            Ok(Message::Pong(_)) => {
                // Pong received, ignore (response to our ping)
                tracing::trace!("Pong received from {}", addr);
            }
            Ok(Message::Text(text)) => {
                // We only support binary protocol, log and ignore text messages
                tracing::warn!("Received unexpected text message from {}: {}", addr, text);
            }
            Ok(Message::Frame(_)) => {
                // Raw frame, shouldn't normally receive this
                tracing::trace!("Received raw frame from {}", addr);
            }
            Err(e) => {
                tracing::error!("WebSocket error for {}: {}", addr, e);
                break;
            }
        }
    }

    // Cleanup on disconnect
    tracing::info!("WebSocket disconnecting: {}", addr);

    // Cancel the send task
    send_task.abort();

    // Notify manager of disconnect
    manager.disconnect(addr).await;

    tracing::info!("WebSocket disconnected: {}", addr);

    Ok(())
}
