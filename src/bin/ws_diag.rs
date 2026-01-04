use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
// use url::Url;

#[tokio::main]
async fn main() {
    let addr = "ws://127.0.0.1:3489";
    println!("Connecting to {}...", addr);

    match connect_async(addr).await {
        Ok((ws_stream, _)) => {
            println!("Connected to {}!", addr);
            let (mut write, mut read) = ws_stream.split();

            // Try different messages
            let messages = vec![
                r#"{"action": "realtime"}"#,
                r#"/v1/realtime"#,
                r#"realtime"#,
                r#"{"request": "realtime"}"#,
                r#"{"msg": "realtime"}"#,
            ];

            for msg in messages {
                println!("Sending: {}", msg);
                if let Err(e) = write.send(Message::Text(msg.to_string().into())).await {
                    println!("Failed to send: {}", e);
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }

            println!("Waiting for messages...");

            // Optional: Try sending something if silent for 5 seconds
            // tokio::spawn(async move {
            //     tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            //     println!("Sending probe...");
            //     write.send(Message::Text("ping".to_string())).await.unwrap();
            // });

            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => println!("Received Text: {}", text),
                    Ok(Message::Binary(bin)) => println!("Received Binary: {} bytes", bin.len()),
                    Ok(Message::Ping(_)) => println!("Received Ping"),
                    Ok(Message::Pong(_)) => println!("Received Pong"),
                    Ok(Message::Close(_)) => {
                        println!("Connection closed");
                        break;
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        break;
                    }
                    _ => println!("Received other message"),
                }
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}
