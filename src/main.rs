//!
//! main.rs
//!
//! copyright Greg Parsons, 2025
//!
use futures_util::FutureExt;
use rust_socketio::{asynchronous::{ClientBuilder}, Payload, TransportType};
use serde_json::json;
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag="event", content = "data")]
enum TextMessage {

    #[serde(rename = "candlestick")]
    #[serde(alias = "candlestick")]
    Candlestick(String),

    #[serde(rename = "depth-update")]
    #[serde(alias = "depth-update")]
    DepthUpdate(String),

    #[serde(rename = "depth-snapshot")]
    #[serde(alias = "depth-snapshot")]
    DepthSnapshot(String),
}

#[tokio::main]
async fn main() {

    println!("connecting...");

    // get a socket that is connected to the admin namespace
    if let Ok(socket_io_client) = ClientBuilder::new("wss://stream.coindcx.com")
        .on_any(|_event, payload, _client| {
            async move {
                if let Payload::Text(json_vec) = payload {
                    for json_0 in json_vec {
                        let mesg = &json_0.to_string();
                        let result = serde_json::from_str::<TextMessage>(&mesg);
                        match result {
                            Ok(m) => {
                                println!("[TEXT]\n{}", serde_json::to_string_pretty(&m).unwrap())

                                // do something with the JSON here

                            },
                            Err(e) => println!("[text error] e: {e:?}"),
                        }
                    }
                } else {
                    println!("[payload] {:?}", &payload);
                }
            }.boxed()
        })
        .on("error", |err, _| { async move { eprintln!("Error: {:#?}", err) }.boxed() })
        .transport_type(TransportType::Websocket)
        .connect()
        .await {

        println!("connected");

        // pause a moment for socketio...otherwise panic; increase if main crashes
        std::thread::sleep(Duration::from_millis(600));

        // join 1m candle channel
        let json_payload = json!({"channelName": "B-BTC_USDT_1m"});
        socket_io_client.emit("join", json_payload).await.expect("[join] server unreachable");

        // join depth channel
        let json_payload = json!({"channelName": "B-BTC_USDT@orderbook@20"});
        socket_io_client.emit("join", json_payload).await.expect("[join] server unreachable");

        loop{
            let _ = tokio::time::sleep(Duration::from_millis(100));
            let json_payload = json!({"channelName": "B-BTC_USDT_1m"});
            // emit with an ack
            socket_io_client
                .emit("candlestick", json_payload)
                .await
                .expect("server unreachable");
        }
    }
}
