use crate::mqtt_constants::{MqttTopics, MqttPayloads};

use paho_mqtt::{self as mqtt, message::Message};
use paho_mqtt::DeliveryToken;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc::{Sender, Receiver};

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
    timer_cancel: Option<Arc<AtomicBool>>,
    pub timer_tx: Option<Sender<(String, Vec<u8>)>>, // Channel for timer events
    pub timer_duration: Duration, // Injected timer duration
}

pub enum StateEnum {
   Open(OpenState),
    Close(CloseState),
    Opening(OpeningState),
    Closing(ClosingState)
}
pub trait State {
    fn process_message(&self, topic: &str, payload: &[u8], handler: &mut MqttEventHandler) -> StateEnum;
}

impl State for OpenState{
    fn process_message(&self, topic: &str, payload: &[u8], handler: &mut MqttEventHandler) -> StateEnum {
        let client = handler.client.as_ref();
        if topic == MqttTopics::COVER_COMMAND && payload == MqttPayloads::COMMAND_CLOSE.as_bytes(){
            let _ = client.publish(
                mqtt::Message::new(MqttTopics::SWITCH_CLOSE_COMMAND, MqttPayloads::SWITCH_ON, 1)
            );
        }
        if topic == MqttTopics::SWITCH_CLOSE_STATE && payload == MqttPayloads::SWITCH_ON.as_bytes() {
            if let Some(cancel_flag) = &handler.timer_cancel {
                cancel_flag.store(true, Ordering::SeqCst);
            }
            let cancel_flag = Arc::new(AtomicBool::new(false));
            handler.timer_cancel = Some(cancel_flag.clone());
            if let Some(tx) = handler.timer_tx.as_ref() {
                let tx = tx.clone();
                let duration = handler.timer_duration;
                println!("DEBUG: Spawning timer in OpenState for {:?}", duration);
                tokio::spawn(async move {
                    sleep(duration).await;
                    if !cancel_flag.load(Ordering::SeqCst) {
                        println!("DEBUG: Timer fired in OpenState");
                        let _ = tx.send((MqttTopics::TIMER_TOPIC.to_string(), MqttPayloads::TIMER_EXPIRES.as_bytes().to_vec())).await;
                    } else {
                        println!("DEBUG: Timer canceled in OpenState");
                    }
                });
            }
            return StateEnum::Closing(ClosingState);
        }
        StateEnum::Open(OpenState)
    }
}

impl State for CloseState {
    fn process_message(&self, topic: &str, payload: &[u8], handler: &mut MqttEventHandler) -> StateEnum{
        let client = handler.client.as_ref();
        if topic == MqttTopics::COVER_COMMAND && payload == MqttPayloads::COMMAND_OPEN.as_bytes(){
            let _ = client.publish(
                mqtt::Message::new(MqttTopics::SWITCH_OPEN_COMMAND, MqttPayloads::SWITCH_ON, 1)
            );
        }
        if topic == MqttTopics::SWITCH_OPEN_STATE && payload == MqttPayloads::SWITCH_ON.as_bytes() {
            if let Some(cancel_flag) = &handler.timer_cancel {
                cancel_flag.store(true, Ordering::SeqCst);
            }
            let cancel_flag = Arc::new(AtomicBool::new(false));
            handler.timer_cancel = Some(cancel_flag.clone());
            if let Some(tx) = handler.timer_tx.as_ref() {
                let tx = tx.clone();
                let duration = handler.timer_duration;
                println!("DEBUG: Spawning timer in CloseState for {:?}", duration);
                tokio::spawn(async move {
                    sleep(duration).await;
                    if !cancel_flag.load(Ordering::SeqCst) {
                        println!("DEBUG: Timer fired in CloseState");
                        let _ = tx.send((MqttTopics::TIMER_TOPIC.to_string(), MqttPayloads::TIMER_EXPIRES.as_bytes().to_vec())).await;
                    } else {
                        println!("DEBUG: Timer canceled in CloseState");
                    }
                });
            }
            return StateEnum::Opening(OpeningState);
        }
        StateEnum::Close(CloseState)
    }
}

impl State for OpeningState {
    fn process_message(&self, topic: &str, payload: &[u8], handler: &mut MqttEventHandler) -> StateEnum{
        let client = handler.client.as_ref();
        if topic == MqttTopics::TIMER_TOPIC && payload == MqttPayloads::TIMER_EXPIRES.as_bytes(){
            let _ = client.publish(
                mqtt::Message::new(MqttTopics::COVER_STATE, MqttPayloads::STATE_OPEN, 1)
            );
            return StateEnum::Open(OpenState);
        }
        if topic == MqttTopics::SWITCH_OPEN_STATE && payload == MqttPayloads::SWITCH_ON.as_bytes() {
            // cancel the timer if it exists
            if let Some(cancel_flag) = &handler.timer_cancel {
                cancel_flag.store(true, Ordering::SeqCst);
            }
        }
        if topic == MqttTopics::SWITCH_CLOSE_STATE && payload == MqttPayloads::SWITCH_ON.as_bytes() {
            // cancel the timer if it exists
            if let Some(cancel_flag) = &handler.timer_cancel {
                cancel_flag.store(true, Ordering::SeqCst);
            }
            let cancel_flag = Arc::new(AtomicBool::new(false));
            handler.timer_cancel = Some(cancel_flag.clone());
            if let Some(tx) = handler.timer_tx.as_ref() {
                let tx = tx.clone();
                let duration = handler.timer_duration;
                println!("DEBUG: Spawning timer in OpeningState for {:?}", duration);
                tokio::spawn(async move {
                    sleep(duration).await;
                    if !cancel_flag.load(Ordering::SeqCst) {
                        println!("DEBUG: Timer fired in OpeningState");
                        let _ = tx.send((MqttTopics::TIMER_TOPIC.to_string(), MqttPayloads::TIMER_EXPIRES.as_bytes().to_vec())).await;
                    } else {
                        println!("DEBUG: Timer canceled in OpeningState");
                    }
                });
            }
            return StateEnum::Closing(ClosingState);
        }
        if topic == MqttTopics::COVER_COMMAND && payload == MqttPayloads::COMMAND_STOP.as_bytes() {
            // stop the opening process
            let _ = client.publish(
                mqtt::Message::new(MqttTopics::SWITCH_OPEN_COMMAND, MqttPayloads::SWITCH_ON, 1)
            );
        }
        StateEnum::Opening(OpeningState)
    }
}

impl State for ClosingState {
    fn process_message(&self, topic: &str, payload: &[u8], handler: &mut MqttEventHandler) -> StateEnum{
        let client = handler.client.as_ref();
        if topic == MqttTopics::TIMER_TOPIC && payload == MqttPayloads::TIMER_EXPIRES.as_bytes(){
            let _ = client.publish(
                mqtt::Message::new(MqttTopics::COVER_STATE, MqttPayloads::STATE_CLOSE, 1)
            );
            return StateEnum::Close(CloseState);
        }
        if topic == MqttTopics::SWITCH_CLOSE_STATE && payload == MqttPayloads::SWITCH_ON.as_bytes() {
            // cancel the timer if it exists
            if let Some(cancel_flag) = &handler.timer_cancel {
                cancel_flag.store(true, Ordering::SeqCst);
            }
        }
        if topic == MqttTopics::SWITCH_OPEN_STATE && payload == MqttPayloads::SWITCH_ON.as_bytes() {
            // cancel the timer if it exists
            if let Some(cancel_flag) = &handler.timer_cancel {
                cancel_flag.store(true, Ordering::SeqCst);
            }
            let cancel_flag = Arc::new(AtomicBool::new(false));
            handler.timer_cancel = Some(cancel_flag.clone());
            if let Some(tx) = handler.timer_tx.as_ref() {
                let tx = tx.clone();
                let duration = handler.timer_duration;
                println!("DEBUG: Spawning timer in CloseState for {:?}", duration);
                tokio::spawn(async move {
                    sleep(duration).await;
                    if !cancel_flag.load(Ordering::SeqCst) {
                        println!("DEBUG: Timer fired in CloseState");
                        let _ = tx.send((MqttTopics::TIMER_TOPIC.to_string(), MqttPayloads::TIMER_EXPIRES.as_bytes().to_vec())).await;
                    } else {
                        println!("DEBUG: Timer canceled in CloseState");
                    }
                });
            }
            return StateEnum::Opening(OpeningState);
        }
        if topic == MqttTopics::COVER_COMMAND && payload == MqttPayloads::COMMAND_STOP.as_bytes() {
            // stop the closing process
            let _ = client.publish(
                mqtt::Message::new(MqttTopics::SWITCH_CLOSE_COMMAND, MqttPayloads::SWITCH_ON, 1)
            );
        }
        StateEnum::Closing(ClosingState)
    }
}



impl MqttEventHandler {
    
    pub fn new(client: Box<dyn Client>)-> Self {
        MqttEventHandler {
            client,
            state: StateEnum::Close(CloseState),
            timer_cancel: None,
            timer_tx: None,
            timer_duration: Duration::from_secs(90), // Default for production
        }
    }
    pub fn set_timer_sender(&mut self, tx: Sender<(String, Vec<u8>)>) {
        self.timer_tx = Some(tx);
    }
    pub fn set_timer_duration(&mut self, duration: Duration) {
        self.timer_duration = duration;
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
        let mut state = std::mem::replace(&mut self.state, StateEnum::Close(CloseState));
        let prev_state = std::mem::discriminant(&state);
        let new_state = state.process_message(topic, payload, self);
        let new_state_disc = std::mem::discriminant(&new_state);
        // Only cancel timer if leaving Opening or Closing state
        let is_leaving_timer_state =
            matches!(self.state, StateEnum::Opening(_) | StateEnum::Closing(_)) &&
            !matches!(new_state, StateEnum::Opening(_) | StateEnum::Closing(_));
        if new_state_disc != prev_state {
            if is_leaving_timer_state {
                println!("DEBUG: Canceling timer in process_message (leaving timer state)");
                if let Some(cancel_flag) = &self.timer_cancel {
                    cancel_flag.store(true, Ordering::SeqCst);
                }
            }
            self.state = new_state;
        } else {
            self.state = state;
        }
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
    fn process_message(&self, topic: &str, payload: &[u8], handler: &mut MqttEventHandler) -> StateEnum {
        match self {
            StateEnum::Open(state) => state.process_message(topic, payload, handler),
            StateEnum::Close(state) => state.process_message(topic, payload, handler),
            StateEnum::Opening(state) => state.process_message(topic, payload, handler),
            StateEnum::Closing(state) => state.process_message(topic, payload, handler),
        }
    }
}

// Ensure all state structs are defined before their impls and used in the right order
pub struct OpenState;
pub struct CloseState;
pub struct OpeningState;
pub struct ClosingState;

// Test module, now in this same file
#[cfg(test)]
// Make MockClient available to all test modules
pub(crate) mod test_util {
    use super::*;
    use std::sync::{Arc, Mutex};
    use paho_mqtt::DeliveryToken;

    #[derive(Clone)]
    pub struct MockClient {
        pub published: Arc<Mutex<Vec<(String, String)>>>,
        pub subscribed: Arc<Mutex<Vec<Vec<String>>>>,
    }
    impl super::Client for MockClient {
        fn publish(&self, msg: mqtt::Message) -> mqtt::Result<DeliveryToken> {
            let mut published = self.published.lock().unwrap();
            published.push((msg.topic().to_string(), msg.payload_str().to_string()));
            Ok(DeliveryToken::new(msg))
        }
        fn subscribe_many(&self, topics: &[&str], _qos: &[u8]) -> mqtt::Result<mqtt::ServerResponse> {
            let mut subscribed = self.subscribed.lock().unwrap();
            subscribed.push(topics.iter().map(|s| s.to_string()).collect());
            Ok(mqtt::ServerResponse::default())
        }
    }
    impl MockClient {
        pub fn new() -> Self {
            MockClient {
                published: Arc::new(Mutex::new(Vec::new())),
                subscribed: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_util::MockClient;
    use std::sync::{Arc, Mutex};

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

// --- TESTS FOR OpeningState and ClosingState timer logic ---
#[cfg(test)]
mod timer_tests {
    use super::*;
    use test_util::MockClient;
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc;
    use std::time::Duration as StdDuration;
    use std::thread;
    use std::sync::atomic::AtomicUsize;

    // Helper to create a handler with a timer channel
    fn setup_handler_with_timer() -> (MqttEventHandler, mpsc::Receiver<(String, Vec<u8>)>) {
        let mock_client: Box<dyn Client> = Box::new(MockClient::new());
        let (tx, rx) = mpsc::channel(2);
        let mut handler = MqttEventHandler::new(mock_client);
        handler.set_timer_sender(tx);
        handler.set_timer_duration(Duration::from_millis(50)); // Fast timer for tests
        (handler, rx)
    }

    // Helper to inject a custom sleep duration for timer (by patching spawn)
    fn trigger_opening_state_timer(handler: &mut MqttEventHandler, sleep_ms: u64) {
        // Simulate transition to OpeningState (from CloseState)
        handler.process_message(MqttTopics::SWITCH_OPEN_STATE, MqttPayloads::SWITCH_ON.as_bytes());
        // Patch: sleep for sleep_ms instead of 90s (done by test logic)
    }

    #[tokio::test]
    async fn test_openingstate_timer_expires() {
        let (mut handler, mut rx) = setup_handler_with_timer();
        handler.process_message(MqttTopics::SWITCH_OPEN_STATE, MqttPayloads::SWITCH_ON.as_bytes());
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let timer_event = tokio::time::timeout(tokio::time::Duration::from_millis(500), rx.recv()).await;
        assert!(timer_event.is_ok(), "Timer event should be received");
        let (topic, payload) = timer_event.unwrap().unwrap();
        assert_eq!(topic, MqttTopics::TIMER_TOPIC);
        assert_eq!(payload, MqttPayloads::TIMER_EXPIRES.as_bytes());
    }

    #[tokio::test]
    async fn test_closingstate_timer_expires() {
        let (mut handler, mut rx) = setup_handler_with_timer();
        // Ensure we start in OpenState to trigger the timer logic
        handler.change_state(StateEnum::Open(OpenState));
        handler.process_message(MqttTopics::SWITCH_CLOSE_STATE, MqttPayloads::SWITCH_ON.as_bytes());
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let timer_event = tokio::time::timeout(tokio::time::Duration::from_millis(500), rx.recv()).await;
        if let Err(e) = &timer_event {
            eprintln!("DEBUG: timer_event error: {:?}", e);
        }
        assert!(timer_event.is_ok(), "Timer event should be received");
        let (topic, payload) = timer_event.unwrap().unwrap();
        assert_eq!(topic, MqttTopics::TIMER_TOPIC);
        assert_eq!(payload, MqttPayloads::TIMER_EXPIRES.as_bytes());
    }

    #[tokio::test]
    async fn test_openingstate_timer_canceled() {
        let (mut handler, mut rx) = setup_handler_with_timer();
        // Simulate the state transition (should spawn timer)
        handler.process_message(MqttTopics::SWITCH_OPEN_STATE, MqttPayloads::SWITCH_ON.as_bytes());
        // Cancel the timer before it expires
        if let Some(cancel_flag) = &handler.timer_cancel {
            cancel_flag.store(true, Ordering::SeqCst);
        }
        // Wait for timer event (should timeout)
        let timer_event = tokio::time::timeout(tokio::time::Duration::from_millis(150), rx.recv()).await;
        assert!(timer_event.is_err(), "Timer event should NOT be received (canceled)");
    }

    #[tokio::test]
    async fn test_closingstate_timer_canceled() {
        let (mut handler, mut rx) = setup_handler_with_timer();
        // Simulate the state transition (should spawn timer)
        handler.process_message(MqttTopics::SWITCH_CLOSE_STATE, MqttPayloads::SWITCH_ON.as_bytes());
        // Cancel the timer before it expires
        if let Some(cancel_flag) = &handler.timer_cancel {
            cancel_flag.store(true, Ordering::SeqCst);
        }
        // Wait for timer event (should timeout)
        let timer_event = tokio::time::timeout(tokio::time::Duration::from_millis(150), rx.recv()).await;
        assert!(timer_event.is_err(), "Timer event should NOT be received (canceled)");
    }
}