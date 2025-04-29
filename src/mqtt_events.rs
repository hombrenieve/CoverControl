use crate::mqtt_topics::MqttTopics;

use paho_mqtt as mqtt;

pub struct MqttEventHandler {
    pub client: mqtt::Client,
    state: StateEnum,
}

pub trait State {
    fn process_message(&self, event_handler: &MqttEventHandler, topic: &str, payload: &[u8]);
}

pub enum StateEnum {
   Open(OpenState),
    Close(CloseState),
    Opening(OpeningState),
    Closing(ClosingState)
}

impl State for StateEnum {
    fn process_message(&self, event_handler: &MqttEventHandler, topic: &str, payload: &[u8]) {
       match self {
            StateEnum::Open(state) => state.process_message(event_handler, topic, payload),
            StateEnum::Close(state) => state.process_message(event_handler, topic, payload),
            StateEnum::Opening(state) => state.process_message(event_handler, topic, payload),
            StateEnum::Closing(state) => state.process_message(event_handler, topic, payload),
        }
    }
}

pub struct OpenState;

impl State for OpenState{
    fn process_message(&self, event_handler: &MqttEventHandler, _topic: &str, _payload: &[u8]) {
        event_handler.client.publish(
            mqtt::Message::new(
                MqttTopics::COVER_COMMAND, 
                "open",
                1
            )
        ).unwrap();
    }
}

pub struct CloseState;

impl State for CloseState {
    fn process_message(&self, event_handler: &MqttEventHandler, _topic: &str, _payload: &[u8]) {
        event_handler.client.publish(
            mqtt::Message::new(
                MqttTopics::COVER_COMMAND, 
                "close",
                1
            )
        ).unwrap();
    }
}

pub struct OpeningState;

impl State for OpeningState {
    fn process_message(&self, event_handler: &MqttEventHandler, _topic: &str, _payload: &[u8]) {
        event_handler.client.publish(
            mqtt::Message::new(
                MqttTopics::COVER_COMMAND, 
                "opening",
                1
            )
        ).unwrap();
    }
}

pub struct ClosingState;

impl State for ClosingState {
    fn process_message(&self, event_handler: &MqttEventHandler, _topic: &str, _payload: &[u8]) {
        event_handler.client.publish(
            mqtt::Message::new(
                MqttTopics::COVER_COMMAND, 
                "closing",
                1
            )
        ).unwrap();
    }
}

    
use std::time::Duration;

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
        MqttEventHandler {
            client,
            state: StateEnum::Close(CloseState),
        }
    }

    pub fn subscribe_to_topics(&self, topics: &[&str]) {
        let qos = [1; 3];
        self.client.subscribe_many(&topics, &qos).unwrap();
    }

    pub fn process_message(&self, topic: &str, payload: &[u8]) {
       self.state.process_message(self, topic, payload);
    }

    pub fn change_state(&mut self, new_state: StateEnum) {
        self.state = new_state;
    }
}