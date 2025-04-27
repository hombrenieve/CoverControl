mod mqtt_events {
    use paho_mqtt as mqtt;

    pub struct MqttEventHandler {        
        pub client: mqtt::Client,
    }

    use std::time::Duration;

    const SWITCH_OPEN_STATUS_TOPIC: &str = "switch_open_status";
    const SWITCH_CLOSE_STATUS_TOPIC: &str = "switch_close_status";
    const COVER_COMMAND_TOPIC: &str = "cover_command";
    const COVER_STATUS_TOPIC: &str = "cover_status";
    const SWITCH_OPEN_COMMAND_TOPIC: &str = "switch_open_command";
    const SWITCH_CLOSE_COMMAND_TOPIC: &str = "switch_close_command";

    impl MqttEventHandler {
        pub fn new() -> Self {
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
            MqttEventHandler { client }
        }

        pub fn subscribe_to_topics(&self) {
            let topics = [
                SWITCH_OPEN_STATUS_TOPIC,
                SWITCH_CLOSE_STATUS_TOPIC,
                COVER_COMMAND_TOPIC,
            ];
            let qos = [1; 3];
            self.client.subscribe_many(&topics, &qos).unwrap();
        }

        pub fn process_message(&self, topic: &str, payload: &[u8]) {
            println!("Received message on topic: {} with payload: {:?}", topic, payload);
            // Send a message to the status and command topics.
            self.client.publish(mqtt::Message::new(COVER_STATUS_TOPIC, "cover_status", 1)).unwrap();
            self.client.publish(mqtt::Message::new(SWITCH_OPEN_COMMAND_TOPIC, "switch_open_command", 1)).unwrap();
            self.client.publish(mqtt::Message::new(SWITCH_CLOSE_COMMAND_TOPIC, "switch_close_command", 1)).unwrap();
        }
    }
}

use mqtt_events::MqttEventHandler;

fn main() {
    let event_handler = MqttEventHandler::new();
    event_handler.subscribe_to_topics();

    let rx = event_handler.client.start_consuming();

    println!("Waiting for messages...");
    while let Ok(msg) = rx.recv() {
        if let Some(msg) = msg {
            event_handler.process_message(msg.topic(), msg.payload());
        }
    }


}
