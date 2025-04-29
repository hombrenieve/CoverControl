mod mqtt_topics;
mod mqtt_events;

use mqtt_events::{MqttEventHandler, Client};
use paho_mqtt as mqtt;
use std::time::Duration;

fn main() {
    // Create a client for the mqtt protocol.
    let create_opts = mqtt::CreateOptionsBuilder::new()        

        .server_uri("tcp://localhost:1883")
        .client_id("my_rust_client")
        .finalize();
    let client = mqtt::Client::new(create_opts).unwrap();
    let lwt = mqtt::Message::new("test", "Good bye", 1);
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .will_message(lwt).finalize();
    client.connect(conn_opts).unwrap();
    // We need to Box the real client to pass it to MqttEventHandler
    let boxed_client: Box<dyn Client> = Box::new(client.clone());
    // Create the event_handler and pass it the boxed client.
    let event_handler = MqttEventHandler::new(boxed_client);
    let _ = event_handler.initialize();
    let rx = client.start_consuming();    

    println!("Waiting for messages...\n");
    while let Ok(msg) = rx.recv() {
        if let Some(msg) = msg {
            event_handler.process_message(msg.topic(), msg.payload());
        }
    }

}
