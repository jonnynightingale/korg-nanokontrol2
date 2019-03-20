extern crate midir;

use std::error::Error;
use std::default::Default;
use std::fmt;
use std::iter::Iterator;

use midir::{MidiInput, MidiOutput, MidiInputConnection, MidiOutputConnection};

#[derive(Debug)]
pub struct KorgError(String);

impl fmt::Display for KorgError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Korg nanoKONTROL2 Error: {}", self.0)
    }
}

impl Error for KorgError {}

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

#[derive(Copy, Clone)]
pub enum KorgCommand {
    NativeModeIoRequest = 0x00,
    DataDumpRequest     = 0x1F,
    DataDump            = 0x7F,
}

#[derive(Copy, Clone)]
pub enum KorgFunction {
    CurrentSceneDataDumpRequest = 0x10,
    CurrentSceneDataDump        = 0x40,
    SceneWriteRequest           = 0x11,
    ModeRequest                 = 0x12,
}

pub struct Connection {
    midi_input_connection: Option<MidiInputConnection<()>>,
    midi_output_connection: Option<MidiOutputConnection>,
}

impl Connection {
    pub fn new() -> Self {
        Connection {
            midi_input_connection: None,
            midi_output_connection: None,
        }
    }

    pub fn open<F>(
        &mut self,
        callback: F
    ) -> Result<(), Box<Error>> 
    where
        F: Fn(u64, &[u8]) + Send + 'static {

        let midi_input = MidiInput::new("input")?;
        let midi_output = MidiOutput::new("output")?;

        if midi_input.port_count() == 0 {
            return Result::Err(Box::new(KorgError("No MIDI input ports found.".into())));
        }

        if midi_input.port_count() > 1 {
            return Result::Err(Box::new(KorgError("Multiple MIDI input ports found.".into())));
        }

        if midi_output.port_count() == 0 {
            return Result::Err(Box::new(KorgError("No MIDI ouput ports found.".into())));
        }

        if midi_output.port_count() > 1 {
            return Result::Err(Box::new(KorgError("Multiple MIDI output ports found.".into())));
        }

        self.midi_input_connection = midi_input.connect(0, "input_port", move |stamp, message, _| {
            callback(stamp, message);
        }, ()).ok();

        self.midi_output_connection = midi_output.connect(0, "output_port").ok();

        Ok(())
    }

    pub fn close(&mut self) {
        let midi_input_connection = self.midi_input_connection.take();
        match midi_input_connection {
            Some(connection) => {
                connection.close();
            },
            None => (),
        };

        let midi_output_connection = self.midi_output_connection.take();
        match midi_output_connection {
            Some(connection) => {
                connection.close();
            },
            None => (),
        };
    }

    pub fn get_slider_value_raw(
        &mut self,
        slider_index: u8
    ) -> Result<(), Box<Error>> {
        let message = Self::generate_two_byte_data_message(
            0x00,
            KorgCommand::DataDumpRequest,
            KorgFunction::CurrentSceneDataDumpRequest,
            slider_index
        );
        match &mut self.midi_output_connection {
            Some(connection) => {
                println!("             {:02X?}", message);
                connection.send(&message)?;
            },
            None => (),
        }
        Ok(())
    }

    fn generate_two_byte_data_message(
        channel: u8,
        command: KorgCommand,
        function: KorgFunction,
        data: u8,
    ) -> [u8; 11] {
        [
            0xF0,            // 1st Byte = F0 : Exclusive Status
            0x42,            // 2nd Byte = 42 : KORG
            0x40 | channel,  // 3rd Byte = 4g : g : Global MIDI Channel
            0x00,            // 4th Byte = 00 : Software Project (nanoKONTROL2: 000113H)
            0x01,            // 5th Byte = 01 : 
            0x13,            // 6th Byte = 13 : 
            0x00,            // 7th Byte = 00 : Sub ID
            command as u8,   // 8th Byte = cd : 0dvmmmmm  d     (0: Host->Controller)
                             //                           v     (0: 2 Bytes Data Format, 1: Variable)
                             //                           mmmmm (Command Number)
            function as u8,  // 9th Byte = nn : 2 Bytes Format: Function ID, Variable: Num of Data
            data,            // 10th Byte = dd : Data
            0xF7,            // LastByte = F7 : End of Exclusive
        ]
    }
}

#[derive(Debug)]
enum ButtonAssignType {
    NoAssign      = 0,
    ControlChange = 1,
    Note          = 2,
}

impl Default for ButtonAssignType {
    fn default() -> Self { ButtonAssignType::NoAssign }
}

#[derive(Debug)]
enum ButtonBehavior {
    Momentary = 0,
    Toggle    = 1,
}

impl Default for ButtonBehavior {
    fn default() -> Self { ButtonBehavior::Momentary }
}

#[derive(Debug)]
enum MidiChannel {
    Custom(u8),
    Global,
}

impl Default for MidiChannel {
    fn default() -> Self { MidiChannel::Global }
}

#[derive(Debug)]
enum SliderAssignType {
    Disable = 0,
    Enable  = 1,
}

impl Default for SliderAssignType {
    fn default() -> Self { SliderAssignType::Disable }
}

#[derive(Debug)]
enum ControlMode {
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

#[derive(Debug)]
enum LedMode {
    Internal = 0,
    External = 1,
}

impl Default for LedMode {
    fn default() -> Self { LedMode::Internal }
}

#[derive(Default, Debug)]
struct ButtonParameters {
    assign_type: ButtonAssignType,
    behavior: ButtonBehavior,
    note_number: u8,
    off_value: u8,
    on_value: u8,
}

#[derive(Default, Debug)]
struct SliderParameters {
    assign_type: SliderAssignType,
    note_number: u8,
    min_value: u8,
    max_value: u8,
}

#[derive(Default, Debug)]
struct ControllerGroupParameters {
    channel: MidiChannel,
    slider: SliderParameters,
    knob: SliderParameters,
    solo_button: ButtonParameters,
    mute_button: ButtonParameters,
    record_button: ButtonParameters,
}

#[derive(Default, Debug)]
pub struct KorgNanokontrol2State {
    global_channel: u8,
    control_mode: ControlMode,
    led_mode: LedMode,
    group: [ControllerGroupParameters; 8],
    transport_button_channel: MidiChannel,
    prev_track:   ButtonParameters,
    next_track:   ButtonParameters,
    cycle_button: ButtonParameters,
    marker_set:   ButtonParameters,
    prev_marker:  ButtonParameters,
    next_marker:  ButtonParameters,
    rew:          ButtonParameters,
    ff:           ButtonParameters,
    stop:         ButtonParameters,
    play:         ButtonParameters,
    rec:          ButtonParameters,
    custom_daw_assign: [u8; 5],
}

impl KorgNanokontrol2State {
    pub fn parse_scene_dump<'a>(&mut self, dump: &[u8]) -> Result<(), Box<Error>> {
        let raw_scene_data: &[u8] = &dump[13..401];

        let global_channel_val = Self::get_raw_scene_data_value(&raw_scene_data, 0);
        self.global_channel = match global_channel_val {
            n if n < 16 => n,
            n => return Err(Box::new(KorgError(format!("Invalid global channel: {}", n)))),
        };

        let control_mode_val = Self::get_raw_scene_data_value(&raw_scene_data, 1);
        self.control_mode = parse_control_mode(control_mode_val)?;

        let led_mode_val = Self::get_raw_scene_data_value(&raw_scene_data, 2);
        self.led_mode = parse_led_mode(led_mode_val)?;

        for i in 0..8 {
            let index: usize = 3 + (i * 31);
            self.group[i] = Self::parse_group_data(&raw_scene_data, index);
        }

        let transport_button_channel_val = Self::get_raw_scene_data_value(&raw_scene_data, 251);
        self.transport_button_channel =
            parse_midi_channel_incl_global(transport_button_channel_val)?;

        self.prev_track   = Self::parse_button_data(&raw_scene_data, 252);
        self.next_track   = Self::parse_button_data(&raw_scene_data, 258);
        self.cycle_button = Self::parse_button_data(&raw_scene_data, 264);
        self.marker_set   = Self::parse_button_data(&raw_scene_data, 270);
        self.prev_marker  = Self::parse_button_data(&raw_scene_data, 276);
        self.next_marker  = Self::parse_button_data(&raw_scene_data, 282);
        self.rew          = Self::parse_button_data(&raw_scene_data, 288);
        self.ff           = Self::parse_button_data(&raw_scene_data, 294);
        self.stop         = Self::parse_button_data(&raw_scene_data, 300);
        self.play         = Self::parse_button_data(&raw_scene_data, 306);
        self.rec          = Self::parse_button_data(&raw_scene_data, 312);

        for i in 0..5 {
            self.custom_daw_assign[i] = Self::get_raw_scene_data_value(&raw_scene_data, i + 318);
        }

        Ok(())
    }

    fn parse_group_data(raw_scene_data: &[u8], index: usize) -> ControllerGroupParameters {
        ControllerGroupParameters {
            channel: match Self::get_raw_scene_data_value(&raw_scene_data, index) {
                16 => MidiChannel::Global,
                n => MidiChannel::Custom(n),
            },

            slider: Self::parse_slider_data(&raw_scene_data, index + 1),
            knob:   Self::parse_slider_data(&raw_scene_data, index + 7),

            solo_button:   Self::parse_button_data(&raw_scene_data, index + 13),
            mute_button:   Self::parse_button_data(&raw_scene_data, index + 19),
            record_button: Self::parse_button_data(&raw_scene_data, index + 25),
        }
    }

    fn parse_slider_data(raw_scene_data: &[u8], index: usize) -> SliderParameters {
        SliderParameters {
            assign_type: match Self::get_raw_scene_data_value(&raw_scene_data, index) {
                1 => SliderAssignType::Enable,
                _ => SliderAssignType::Disable,
            },
            note_number: Self::get_raw_scene_data_value(&raw_scene_data, index + 2),
            min_value:   Self::get_raw_scene_data_value(&raw_scene_data, index + 3),
            max_value:   Self::get_raw_scene_data_value(&raw_scene_data, index + 4),
        }
    }

    fn parse_button_data(raw_scene_data: &[u8], index: usize) -> ButtonParameters {
        ButtonParameters {
            assign_type: match Self::get_raw_scene_data_value(&raw_scene_data, index) {
                2 => ButtonAssignType::Note,
                1 => ButtonAssignType::ControlChange,
                _ => ButtonAssignType::NoAssign,
            },
            behavior: match Self::get_raw_scene_data_value(&raw_scene_data, index + 1) {
                1 => ButtonBehavior::Toggle,
                _ => ButtonBehavior::Momentary,
            },
            note_number: Self::get_raw_scene_data_value(&raw_scene_data, index + 2),
            off_value:   Self::get_raw_scene_data_value(&raw_scene_data, index + 3),
            on_value:    Self::get_raw_scene_data_value(&raw_scene_data, index + 4),
        }
    }

    fn get_raw_scene_data_value(raw_scene_data: &[u8], index: usize) -> u8 {
        let i = ((index / 7) * 8) + ((index % 7) + 1);
        raw_scene_data[i]
    }
}

fn parse_control_mode(val: u8) -> Result<ControlMode, String> {
    match val {
        0 => Ok(ControlMode::CcMode),
        1 => Ok(ControlMode::Cubase),
        2 => Ok(ControlMode::Dp),
        3 => Ok(ControlMode::Live),
        4 => Ok(ControlMode::ProTools),
        5 => Ok(ControlMode::Sonar),
        n => Err(format!("Invalid control mode: {}.", n)),
    }
}

fn parse_led_mode(val: u8) -> Result<LedMode, String> {
    match val {
        0 => Ok(LedMode::Internal),
        1 => Ok(LedMode::External),
        n => Err(format!("Invalid LED mode: {}.", n)),
    }
}

fn parse_midi_channel_incl_global(val: u8) -> Result<MidiChannel, String> {
    match val {
        16 => Ok(MidiChannel::Global),
        n if n < 16 => Ok(MidiChannel::Custom(n)),
        n => Err(format!("Invalid MIDI channel: {}.", n)),
    }
}
