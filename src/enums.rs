use std::default::Default;

#[derive(Debug, Copy, Clone)]
pub enum TransportButton {
    TrackRewind,
    TrackFastforward,
    Cycle,
    Set,
    MarkerRewind,
    MarkerFastforward,
    Rewind,
    Fastforward,
    Stop,
    Play,
    Record,
}

#[derive(Debug, Copy, Clone)]
pub enum ButtonAssignType {
    NoAssign      = 0,
    ControlChange = 1,
    Note          = 2,
}

impl Default for ButtonAssignType {
    fn default() -> Self { ButtonAssignType::NoAssign }
}

#[derive(Debug, Copy, Clone)]
pub enum ButtonBehavior {
    Momentary = 0,
    Toggle    = 1,
}

impl Default for ButtonBehavior {
    fn default() -> Self { ButtonBehavior::Momentary }
}

#[derive(Debug, Copy, Clone)]
pub enum MidiChannel {
    Custom(u8),
    Global,
}

impl Default for MidiChannel {
    fn default() -> Self { MidiChannel::Global }
}

#[derive(Debug, Copy, Clone)]
pub enum SliderAssignType {
    Disable = 0,
    Enable  = 1,
}

impl Default for SliderAssignType {
    fn default() -> Self { SliderAssignType::Enable }
}

#[derive(Debug, Copy, Clone)]
pub enum ControlMode {
    CcMode   = 0,
    Cubase   = 1,
    Dp       = 2,
    Live     = 3,
    ProTools = 4,
    Sonar    = 5,
}

impl Default for ControlMode {
    fn default() -> Self { ControlMode::CcMode }
}

#[derive(Debug, Copy, Clone)]
pub enum LedMode {
    Internal = 0,
    External = 1,
}

impl Default for LedMode {
    fn default() -> Self { LedMode::Internal }
}
