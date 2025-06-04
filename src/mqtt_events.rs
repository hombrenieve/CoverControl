use crate::mqtt_constants::{MqttTopics, MqttPayloads};

use paho_mqtt::{self as mqtt, message::Message};
use paho_mqtt::DeliveryToken;

pub trait Client: Send {
    fn publish(&self, msg: mqtt::Message) -> mqtt::Result<mqtt::DeliveryToken>;
    fn subscribe_many(&self, topics: &[&str], qos: &[u8]) -> mqtt::Result<mqtt::ServerResponse>;
}

impl Client for mqtt::Client{
    fn publish(&self, msg: Message) -> mqtt::Result<DeliveryToken> {
        let msg_clone = msg.clone();
        mqtt::Client::publish(self, msg_clone)?;
        Ok(DeliveryToken::new(msg))
    }
    fn subscribe_many(&self, topics: &[&str], qos: &[u8]) -> mqtt::Result<mqtt::ServerResponse>{
        //we need to convert u8 to i32
        let qos_i32: Vec<i32> = qos.iter().map(|&x| x as i32).collect();
        return mqtt::Client::subscribe_many(self, topics, qos_i32.as_slice());
    }
}
pub struct MqttEventHandler {
    pub client: Box<dyn Client>,
    state: StateEnum,
}

pub enum StateEnum {
   Open(OpenState),
    Close(CloseState),
    Opening(OpeningState),
    Closing(ClosingState)
}
pub trait State {
    fn process_message(&self, topic: &str, payload: &[u8], client: &dyn Client) -> StateEnum;
}



pub struct OpenState;

impl State for OpenState{
    fn process_message(&self, topic: &str, payload: &[u8], client: &dyn Client) -> StateEnum {
        if topic == MqttTopics::COVER_COMMAND && payload == MqttPayloads::COMMAND_CLOSE.as_bytes(){
            let _ = client.publish(
                mqtt::Message::new(MqttTopics::SWITCH_CLOSE_COMMAND, MqttPayloads::SWITCH_ON, 1)
            );
        }
        if topic == MqttTopics::SWITCH_CLOSE_STATE && payload == MqttPayloads::SWITCH_ON.as_bytes() {
            return StateEnum::Closing(ClosingState);
        }
        StateEnum::Open(OpenState)
    }
}

pub struct CloseState;

impl State for CloseState {
    fn process_message(&self, topic: &str, payload: &[u8], client: &dyn Client) -> StateEnum{
        if topic == MqttTopics::COVER_COMMAND && payload == MqttPayloads::COMMAND_OPEN.as_bytes(){
            let _ = client.publish(
                mqtt::Message::new(MqttTopics::SWITCH_OPEN_COMMAND, MqttPayloads::SWITCH_ON, 1)
            );
        }
        if topic == MqttTopics::SWITCH_OPEN_STATE && payload == MqttPayloads::SWITCH_ON.as_bytes() {
            return StateEnum::Opening(OpeningState);
        }
        StateEnum::Close(CloseState)
    }
}

pub struct OpeningState;

impl State for OpeningState {
    fn process_message(&self, topic: &str, payload: &[u8], client: &dyn Client) -> StateEnum{
        if topic == MqttTopics::COVER_COMMAND && payload == MqttPayloads::COMMAND_OPEN.as_bytes(){
            let _ = client.publish(
                mqtt::Message::new(MqttTopics::COVER_STATE, MqttPayloads::STATE_OPEN, 1)
            );
            return StateEnum::Open(OpenState);
        }
        StateEnum::Opening(OpeningState)
    }
}

pub struct ClosingState;

impl State for ClosingState {
    fn process_message(&self, topic: &str, payload: &[u8], client: &dyn Client) -> StateEnum{
        if topic == MqttTopics::COVER_COMMAND && payload == MqttPayloads::COMMAND_CLOSE.as_bytes(){
            let _ = client.publish(
                mqtt::Message::new(MqttTopics::COVER_STATE, MqttPayloads::STATE_CLOSE, 1)
            );
            return StateEnum::Close(CloseState);
        }
        StateEnum::Closing(ClosingState)
    }
}



impl MqttEventHandler {
    
    pub fn new(client: Box<dyn Client>)-> Self {
        MqttEventHandler {
            client,
            state: StateEnum::Close(CloseState),
        }
    }

    pub fn initialize(&self) -> Result<(), mqtt::Error> {
        let qos = [1; 3];
        let topics = [
            MqttTopics::SWITCH_OPEN_STATE,
            MqttTopics::SWITCH_CLOSE_STATE,
            MqttTopics::COVER_STATE,
        ];
        self.client.subscribe_many(&topics, &qos)?;
        self.client.publish(
            mqtt::Message::new(
                MqttTopics::COVER_AVAILABILITY,
                MqttPayloads::AVAILABILITY_ONLINE,
                1,
            )
        )?;
        Ok(())
    }

    pub fn process_message(&mut self, topic: &str, payload: &[u8]) {
        let new_state = self.state.process_message(topic, payload, self.client.as_ref());
        self.change_state(new_state);
        self.publish_state();
    }    
  
    fn publish_state(&self) {
        let payload = match &self.state {
            StateEnum::Open(_) => MqttPayloads::STATE_OPEN,
            StateEnum::Close(_) => MqttPayloads::STATE_CLOSE,
            StateEnum::Opening(_) => MqttPayloads::STATE_OPENING,
            StateEnum::Closing(_) => MqttPayloads::STATE_CLOSING,
        };
        self.client.publish(mqtt::Message::new(
            MqttTopics::COVER_STATE,
            payload,
            1,
        )).unwrap();
    }

    pub fn change_state(&mut self, new_state: StateEnum) {
        self.state = new_state;
    }

    pub fn finalize(&self) -> Result<(), mqtt::Error> {
        self.client.publish(
            mqtt::Message::new(
                MqttTopics::COVER_AVAILABILITY,
                MqttPayloads::AVAILABILITY_OFFLINE,
                1,
            )
        )?;
        Ok(())
    }
}

impl StateEnum {
    fn process_message(&self, topic: &str, payload: &[u8], client: &dyn Client) -> StateEnum {
        match self {
            StateEnum::Open(state) => state.process_message(topic, payload, client),
            StateEnum::Close(state) => state.process_message(topic, payload, client),
            StateEnum::Opening(state) => state.process_message(topic, payload, client),
            StateEnum::Closing(state) => state.process_message(topic, payload, client),
        }
    }
}



// Test module, now in this same file
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // Struct to mock the mqtt::Client
    #[derive(Clone)]    
    struct MockClient {
        // Use mutex to share the published field
        published: Arc<Mutex<Vec<(String, String)>>>,
        subscribed: Arc<Mutex<Vec<Vec<String>>>>,

    }    
    impl Client for MockClient{
        fn publish(&self, msg: mqtt::Message) -> mqtt::Result<DeliveryToken> {
            let mut published = self.published.lock().unwrap();
            published.push((msg.topic().to_string(),msg.payload_str().to_string()));
            Ok(DeliveryToken::new(msg))
        }
        fn subscribe_many(&self, topics: &[&str], _qos: &[u8]) -> mqtt::Result<mqtt::ServerResponse>{
            let mut subscribed = self.subscribed.lock().unwrap();
            subscribed.push(topics.iter().map(|s| s.to_string()).collect());
            Ok(mqtt::ServerResponse::default())
        }
    }

    impl MockClient {
        fn new() -> Self {
            MockClient {
                published: Arc::new(Mutex::new(Vec::new())),
                subscribed: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[test]
    fn test_process_message_close() {
        // Create a MockClient
        let mock_client = Arc::new(MockClient::new());
        let mock_client_clone = mock_client.clone();
        
        // Create a Box<dyn Client> from MockClient
        let boxed_client: Box<dyn Client> = Box::new((*mock_client_clone).clone());
        let mut event_handler = MqttEventHandler::new(boxed_client);
        event_handler.process_message("some_topic", "some_payload".as_bytes());
        let published = mock_client.published.lock().unwrap();
        //the first message is not sent in process message.
        assert_eq!(published.len(), 1);
        assert_eq!(published[0], (MqttTopics::COVER_STATE.to_string(), MqttPayloads::STATE_CLOSE.to_string()));
    }   

    #[test]
    fn test_initialize() {
        // Create a MockClient
        let mock_client = Arc::new(MockClient::new());
        let mock_client_clone = mock_client.clone();

        // Create a Box<dyn Client> from MockClient
        let boxed_client: Box<dyn Client> = Box::new((*mock_client_clone).clone());
        let event_handler = MqttEventHandler::new(boxed_client);
        event_handler.initialize().unwrap();

        // Verify that the handler subscribed to the correct topics
        let subscribed = mock_client.subscribed.lock().unwrap();
        assert_eq!(subscribed.len(), 1);
        assert_eq!(
            subscribed[0],
            vec![
                MqttTopics::SWITCH_OPEN_STATE.to_string(),
                MqttTopics::SWITCH_CLOSE_STATE.to_string(),
                MqttTopics::COVER_STATE.to_string(),
            ]
        );

        // Verify that the handler published the "online" message
        let published = mock_client.published.lock().unwrap();
        assert_eq!(published.len(), 1);
        assert_eq!(
            published[0],
            (MqttTopics::COVER_AVAILABILITY.to_string(), MqttPayloads::AVAILABILITY_ONLINE.to_string())
        );
    }
    #[test]
    fn test_opening_to_open() {
        // Create a MockClient
        let mock_client = Arc::new(MockClient::new());
        let mock_client_clone = mock_client.clone();

        // Create a Box<dyn Client> from MockClient
        let boxed_client: Box<dyn Client> = Box::new((*mock_client_clone).clone());
        let mut event_handler = MqttEventHandler::new(boxed_client);
        event_handler.change_state(StateEnum::Opening(OpeningState));
        event_handler.publish_state();
        event_handler.process_message(MqttTopics::COVER_COMMAND, MqttPayloads::COMMAND_OPEN.as_bytes());
        let published = mock_client.published.lock().unwrap();
        assert_eq!(published.len(), 3); // state_opening, state_open, state_open
        assert_eq!(published[1], (MqttTopics::COVER_STATE.to_string(), MqttPayloads::STATE_OPEN.to_string()));
        assert_eq!(published[2], (MqttTopics::COVER_STATE.to_string(), MqttPayloads::STATE_OPEN.to_string()));
    }
    #[test]
    fn test_closing_to_close() {
         // Create a MockClient
         let mock_client = Arc::new(MockClient::new());
         let mock_client_clone = mock_client.clone();
 
         // Create a Box<dyn Client> from MockClient
         let boxed_client: Box<dyn Client> = Box::new((*mock_client_clone).clone());
         let mut event_handler = MqttEventHandler::new(boxed_client);
         event_handler.change_state(StateEnum::Closing(ClosingState));
         event_handler.publish_state();
         event_handler.process_message(MqttTopics::COVER_COMMAND, MqttPayloads::COMMAND_CLOSE.as_bytes());
         let published = mock_client.published.lock().unwrap();
         assert_eq!(published.len(), 3); // state_closing, state_close, state_close
         assert_eq!(published[1], (MqttTopics::COVER_STATE.to_string(), MqttPayloads::STATE_CLOSE.to_string()));
         assert_eq!(published[2], (MqttTopics::COVER_STATE.to_string(), MqttPayloads::STATE_CLOSE.to_string()));
    }
    #[test]
    fn test_finalize() {
        // Create a MockClient
        let mock_client = Arc::new(MockClient::new());
        let mock_client_clone = mock_client.clone();

        // Create a Box<dyn Client> from MockClient
        let boxed_client: Box<dyn Client> = Box::new((*mock_client_clone).clone());
        let event_handler = MqttEventHandler::new(boxed_client);
        
        // Call finalize
        event_handler.finalize().unwrap();

        // Verify that the offline message was published
        let published = mock_client.published.lock().unwrap();
        assert_eq!(published.len(), 1);
        assert_eq!(
            published[0],
            (MqttTopics::COVER_AVAILABILITY.to_string(), MqttPayloads::AVAILABILITY_OFFLINE.to_string())
        );
    }
    #[test]
    fn test_close_state_process_message_behavior() {
        // Create a MockClient
        let mock_client = Arc::new(MockClient::new());
        let mock_client_clone = mock_client.clone();

        // Create a Box<dyn Client> from MockClient
        let boxed_client: Box<dyn Client> = Box::new((*mock_client_clone).clone());
        let mut event_handler = MqttEventHandler::new(boxed_client);

        // 1. Send COVER_COMMAND/COMMAND_OPEN, should publish to SWITCH_OPEN_COMMAND/SWITCH_ON
        event_handler.process_message(MqttTopics::COVER_COMMAND, MqttPayloads::COMMAND_OPEN.as_bytes());
        let published = mock_client.published.lock().unwrap();
        assert_eq!(published[0], (MqttTopics::SWITCH_OPEN_COMMAND.to_string(), MqttPayloads::SWITCH_ON.to_string()));
        assert_eq!(published[1], (MqttTopics::COVER_STATE.to_string(), MqttPayloads::STATE_CLOSE.to_string())); // publish_state
        drop(published);

        // 2. Send SWITCH_OPEN_STATE/SWITCH_ON, should transition to OpeningState and publish state_opening
        event_handler.process_message(MqttTopics::SWITCH_OPEN_STATE, MqttPayloads::SWITCH_ON.as_bytes());
        let published = mock_client.published.lock().unwrap();
        // The last message should be COVER_STATE/STATE_OPENING
        assert_eq!(published[2], (MqttTopics::COVER_STATE.to_string(), MqttPayloads::STATE_OPENING.to_string()));
        // After this, the state should be OpeningState
        match &event_handler.state {
            StateEnum::Opening(_) => {} // OK
            _ => panic!("State should be OpeningState"),
        }
    }
    #[test]
    fn test_open_state_process_message_behavior() {
        // Create a MockClient
        let mock_client = Arc::new(MockClient::new());
        let mock_client_clone = mock_client.clone();

        // Create a Box<dyn Client> from MockClient
        let boxed_client: Box<dyn Client> = Box::new((*mock_client_clone).clone());
        let mut event_handler = MqttEventHandler::new(boxed_client);
        // Set state to OpenState
        event_handler.change_state(StateEnum::Open(OpenState));
        event_handler.publish_state(); // Publish initial state

        // 1. Send COVER_COMMAND/COMMAND_CLOSE, should publish to SWITCH_CLOSE_COMMAND/SWITCH_ON
        event_handler.process_message(MqttTopics::COVER_COMMAND, MqttPayloads::COMMAND_CLOSE.as_bytes());
        let published = mock_client.published.lock().unwrap();
        // The first message is the initial publish_state (STATE_OPEN)
        assert_eq!(published[0], (MqttTopics::COVER_STATE.to_string(), MqttPayloads::STATE_OPEN.to_string()));
        // The second message should be SWITCH_CLOSE_COMMAND/SWITCH_ON
        assert_eq!(published[1], (MqttTopics::SWITCH_CLOSE_COMMAND.to_string(), MqttPayloads::SWITCH_ON.to_string()));
        // The third message should be publish_state (still STATE_OPEN)
        assert_eq!(published[2], (MqttTopics::COVER_STATE.to_string(), MqttPayloads::STATE_OPEN.to_string()));
        drop(published);

        // 2. Send SWITCH_CLOSE_STATE/SWITCH_ON, should transition to ClosingState and publish state_closing
        event_handler.process_message(MqttTopics::SWITCH_CLOSE_STATE, MqttPayloads::SWITCH_ON.as_bytes());
        let published = mock_client.published.lock().unwrap();
        // The last message should be COVER_STATE/STATE_CLOSING
        assert_eq!(published[3], (MqttTopics::COVER_STATE.to_string(), MqttPayloads::STATE_CLOSING.to_string()));
        // After this, the state should be ClosingState
        match &event_handler.state {
            StateEnum::Closing(_) => {} // OK
            _ => panic!("State should be ClosingState"),
        }
    }
}