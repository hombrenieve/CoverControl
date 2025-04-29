pub struct MqttTopics;

impl MqttTopics {
    pub const SWITCH_OPEN_STATUS: &'static str = "switch_open_status";
    pub const SWITCH_CLOSE_STATUS: &'static str = "switch_close_status";
    pub const COVER_COMMAND: &'static str = "cover_command";
}