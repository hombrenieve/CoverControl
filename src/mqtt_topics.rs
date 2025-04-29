pub struct MqttTopics;

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
}