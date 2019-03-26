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

impl From<u8> for ButtonAssignType {
    fn from(n: u8) -> Self {
        match n {
            1 => ButtonAssignType::ControlChange,
            2 => ButtonAssignType::Note,
            _ => ButtonAssignType::NoAssign,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ButtonBehavior {
    Momentary = 0,
    Toggle    = 1,
}

impl Default for ButtonBehavior {
    fn default() -> Self { ButtonBehavior::Momentary }
}

impl From<u8> for ButtonBehavior {
    fn from(n: u8) -> Self {
        match n {
            1 => ButtonBehavior::Toggle,
            _ => ButtonBehavior::Momentary,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MidiChannel {
    Custom(u8),
    Global,
}

impl Default for MidiChannel {
    fn default() -> Self { MidiChannel::Global }
}

impl From<u8> for MidiChannel {
    fn from(n: u8) -> Self {
        match n {
            k if k < 16 => MidiChannel::Custom(n),
            _ => MidiChannel::Global,
        }
    }
}

impl Into<u8> for MidiChannel {
    fn into(self) -> u8 {
        match self {
            MidiChannel::Custom(n) => n,
            MidiChannel::Global => 16,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SliderAssignType {
    Disable = 0,
    Enable  = 1,
}

impl Default for SliderAssignType {
    fn default() -> Self { SliderAssignType::Enable }
}

impl std::convert::From<u8> for SliderAssignType {
    fn from(n: u8) -> Self {
        match n {
            1 => SliderAssignType::Enable,
            _ => SliderAssignType::Disable,
        }
    }
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

impl From<u8> for ControlMode {
    fn from(n: u8) -> Self {
        match n {
            1 => ControlMode::Cubase,
            2 => ControlMode::Dp,
            3 => ControlMode::Live,
            4 => ControlMode::ProTools,
            5 => ControlMode::Sonar,
            _ => ControlMode::CcMode,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum LedMode {
    Internal = 0,
    External = 1,
}

impl Default for LedMode {
    fn default() -> Self { LedMode::Internal }
}

impl From<u8> for LedMode {
    fn from(n: u8) -> Self {
        match n {
            1 => LedMode::External,
            _ => LedMode::Internal,
        }
    }
}
