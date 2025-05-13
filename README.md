# Virtual Cover Device for Home Assistant

## Project Purpose

This project implements a virtual cover device designed for integration with Home Assistant. The application leverages the MQTT protocol to interact with Home Assistant, enabling control and status reporting for a virtual cover entity.

The core functionality of the virtual cover device is to:

1.  **Receive Commands:** Listen for commands from Home Assistant to open or close the virtual cover as well as from the switches that are also reporting
in a different topic.
2.  **Publish Commands:** Send commands to virtual switches to simulate the open/close actions.
3.  **Report Status:** Update Home Assistant with the status of the virtual switches (open/closed).

## Components

The virtual cover device consists of the following components:

1.  **Virtual Cover Device:** The central application that:
    *   Receives commands (open/close) from Home Assistant via MQTT.
    *   Publishes commands to virtual switches via MQTT.
2.  **Virtual Switches (2):** Two virtual switches, one for opening and one for closing the cover.
    * The switches will change their status when the virtual cover sends a message, this app will process and send to the appropriate topic depending on the switch.
    *   Each switch will publish its status (open/close) via MQTT.
3. **Home Assistant:**
   * The home automation platform that sends commands to the virtual cover.
   * Receives status messages from the virtual switches.

## MQTT Interaction

The application interacts with MQTT, responding to specific topics and publishing commands via MQTT messages. The specific topics used are defined within the application.

## Mermaid Diagram

Here's a simple diagram illustrating the interaction between the components:
```mermaid
graph LR
    HA[Home Assistant] -- Command(open/close) --> VCD(Virtual Cover Device)
    VCD -- Command(open)--> OpenSwitch(Open Switch)
    VCD -- Command(close) --> CloseSwitch(Close Switch)
    OpenSwitch -- Status(open/closed) --> HA
    CloseSwitch -- Status(open/closed) --> HA
```
**Explanation of the Diagram**

* Home Assistant send a command to the virtual cover device.
* Virtual cover device sends a command to one of the switches.
* The switches will update their status and publish this to home assistant.

## State Machine

The cover device implements a state machine to maintain and track the current state of the virtual cover device. This is essential because:

1. **State Consistency**: The state machine ensures that the cover device can only transition between valid states in a controlled manner, preventing invalid state combinations.
2. **State Memory**: It maintains the current position/state of the cover (open, closed, or in transition) even when no commands are being received.
3. **Input Validation**: Only valid commands for the current state are processed, while invalid or irrelevant commands are ignored, making the system more robust.
4. **Predictable Behavior**: The state transitions are well-defined and documented, making the system's behavior predictable and easier to test.

The state machine has four states:

1. **Close**: The initial state, representing a fully closed cover
2. **Opening**: Transitional state when the cover is in the process of opening
3. **Open**: Represents a fully opened cover
4. **Closing**: Transitional state when the cover is in the process of closing

### State Transitions

The state transitions are triggered by specific MQTT messages:

- **Close → Opening**: Triggered by "opening" message
- **Opening → Open**: Triggered by "open" message
- **Open → Closing**: Triggered by "closing" message
- **Closing → Close**: Triggered by "close" message

Here's a diagram showing the state transitions:

```mermaid
stateDiagram-v2
    Close --> Opening: "opening"
    Opening --> Open: "open"
    Open --> Closing: "closing"
    Closing --> Close: "close"
```

Any other messages received in a state will be ignored and the state will remain unchanged.

## Getting Started

To run the application, follow these steps:

1.  Ensure you have Rust and Cargo installed.
2.  Clone the repository:
```
bash
    git clone [repository url]
    
```
3.  Navigate to the project directory:
```
bash
    cd [project directory]
    
```
4.  Run the project
```
bash
    cargo run
    
```
## Future Improvements

*   Error handling and logging improvements.
*   Add support for more complex cover functionalities, such as setting a specific position.
*   Implement an external configuration.