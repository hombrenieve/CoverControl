pub struct MqttTopics;
pub struct MqttPayloads;

impl MqttTopics {
    // Topics for the virtual cover
    pub const COVER_AVAILABILITY: &'static str = "cover/availability";
    pub const COVER_STATE: &'static str = "cover/state";
    pub const COVER_COMMAND: &'static str = "cover/command";

    // Topics for the open switch
    pub const SWITCH_OPEN_STATE: &'static str = "switch/open/state";
    pub const SWITCH_OPEN_COMMAND: &'static str = "switch/open/command";
    // Topics for the close switch
    pub const SWITCH_CLOSE_STATE: &'static str = "switch/close/state";
    pub const SWITCH_CLOSE_COMMAND: &'static str = "switch/close/command";

    // Fake topic for the timer
    pub const TIMER_TOPIC: &'static str = "timer/cover";
}

impl MqttPayloads {
    // Commands sent to the virtual device
    pub const COMMAND_OPEN: &'static str = "open";
    pub const COMMAND_STOP: &'static str = "stop";
    pub const COMMAND_CLOSE: &'static str = "close";

    // State payloads received from the state topic

    pub const STATE_CLOSE: &'static str = "close";
    pub const STATE_OPENING: &'static str = "opening";
    pub const STATE_OPEN: &'static str = "open";
    pub const STATE_CLOSING: &'static str = "closing";

    // Commands for switches
    pub const SWITCH_ON: &'static str = "on";
    pub const SWITCH_OFF: &'static str = "off";

    // Availability payloads
    pub const AVAILABILITY_ONLINE: &'static str = "online";
    pub const AVAILABILITY_OFFLINE: &'static str = "offline";

    // Timer payloads
    pub const TIMER_EXPIRES: &'static str = "expires";
}