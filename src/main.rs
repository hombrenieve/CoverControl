mod mqtt_topics;
mod mqtt_events;

use mqtt_events::MqttEventHandler;
use mqtt_topics::MqttTopics;

fn main() {
    let event_handler = MqttEventHandler::new();
    event_handler.subscribe_to_topics(&[
        MqttTopics::SWITCH_OPEN_STATUS,
        MqttTopics::SWITCH_CLOSE_STATUS
    ]);

    let rx = event_handler.client.start_consuming();

    println!("Waiting for messages...\n");
    while let Ok(msg) = rx.recv() {
        if let Some(msg) = msg {
            event_handler.process_message(msg.topic(), msg.payload());
        }
    }

}
