mod mqtt_constants;
mod mqtt_events;

use mqtt_events::{MqttEventHandler, Client};
use paho_mqtt as mqtt;
use std::time::Duration;
use std::sync::{Arc, Mutex};

fn main() {
    // Create a client for the mqtt protocol.
    let create_opts = mqtt::CreateOptionsBuilder::new()        
        .server_uri("tcp://localhost:1883")
        .client_id("CoverControl")
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
    
    if let Err(err) = event_handler.initialize(){
        eprintln!("Error initializing MqttEventHandler: {}", err);
        return;
    }

    // Create an Arc<Mutex<_>> to share event_handler with the signal handler
    let event_handler = Arc::new(Mutex::new(event_handler));
    let event_handler_clone = event_handler.clone();

    // Set up the SIGTERM handler
    ctrlc::set_handler(move || {
        println!("Received SIGTERM signal, shutting down...");
        if let Ok(handler) = event_handler_clone.lock() {
            if let Err(e) = handler.finalize() {
                eprintln!("Error during finalization: {}", e);
            }
        }
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    let rx = client.start_consuming();    
    println!("Waiting for messages...\n");
    
    while let Ok(msg) = rx.recv() {
        if let Some(msg) = msg {
            if let Ok(mut handler) = event_handler.lock() {
                handler.process_message(msg.topic(), msg.payload());
            }
        }
    }
}
