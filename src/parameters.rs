use super::*;
use super::error::Error;

#[derive(Default, Debug)]
pub struct ButtonParameters {
    pub assign_type: ButtonAssignType,
    pub behavior: ButtonBehavior,
    pub note_number: u8,
    pub off_value: u8,
    pub on_value: u8,
}

#[derive(Default, Debug)]
pub struct SliderParameters {
    pub assign_type: SliderAssignType,
    pub note_number: u8,
    pub min_value: u8,
    pub max_value: u8,
}

#[derive(Default, Debug)]
pub struct ControllerGroupParameters {
    pub channel: MidiChannel,
    pub slider: SliderParameters,
    pub knob: SliderParameters,
    pub solo_button: ButtonParameters,
    pub mute_button: ButtonParameters,
    pub record_button: ButtonParameters,
}

#[derive(Default, Debug)]
pub struct Parameters {
    pub global_channel: u8,
    pub control_mode: ControlMode,
    pub led_mode: LedMode,
    pub groups: [ControllerGroupParameters; 8],
    pub transport_button_channel: MidiChannel,
    pub track_rewind:       ButtonParameters,
    pub track_fastforward:  ButtonParameters,
    pub cycle:              ButtonParameters,
    pub set:                ButtonParameters,
    pub marker_rewind:      ButtonParameters,
    pub marker_fastforward: ButtonParameters,
    pub rewind:             ButtonParameters,
    pub fastforward:        ButtonParameters,
    pub stop:               ButtonParameters,
    pub play:               ButtonParameters,
    pub record:             ButtonParameters,
    pub custom_daw_assign: [u8; 5],
}

impl Parameters {
    pub fn parse_scene_dump<'a>(&mut self, dump: &[u8]) -> Result<()> {
        let raw_scene_data: &[u8] = &dump[13..401];

        let global_channel_val = Self::get_raw_scene_data_value(&raw_scene_data, 0);
        self.global_channel = match global_channel_val {
            n if n < 16 => n,
            n => return Err(Error::InvalidGlobalChannel(n)),
        };

        let control_mode_val = Self::get_raw_scene_data_value(&raw_scene_data, 1);
        self.control_mode = parse_control_mode(control_mode_val)?;

        let led_mode_val = Self::get_raw_scene_data_value(&raw_scene_data, 2);
        self.led_mode = parse_led_mode(led_mode_val)?;

        for i in 0..8 {
            let index: usize = 3 + (i * 31);
            self.groups[i] = Self::parse_group_data(&raw_scene_data, index);
        }

        let transport_button_channel_val = Self::get_raw_scene_data_value(&raw_scene_data, 251);
        self.transport_button_channel =
            parse_midi_channel_incl_global(transport_button_channel_val)?;

        self.track_rewind       = Self::parse_button_data(&raw_scene_data, 252);
        self.track_fastforward  = Self::parse_button_data(&raw_scene_data, 258);
        self.cycle              = Self::parse_button_data(&raw_scene_data, 264);
        self.set                = Self::parse_button_data(&raw_scene_data, 270);
        self.marker_rewind      = Self::parse_button_data(&raw_scene_data, 276);
        self.marker_fastforward = Self::parse_button_data(&raw_scene_data, 282);
        self.rewind             = Self::parse_button_data(&raw_scene_data, 288);
        self.fastforward        = Self::parse_button_data(&raw_scene_data, 294);
        self.stop               = Self::parse_button_data(&raw_scene_data, 300);
        self.play               = Self::parse_button_data(&raw_scene_data, 306);
        self.record             = Self::parse_button_data(&raw_scene_data, 312);

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

    pub fn get_transport_button_parameters(&self, button_type: TransportButton)
    -> &ButtonParameters {
        match button_type {
            TransportButton::TrackRewind       => &self.track_rewind,
            TransportButton::TrackFastforward  => &self.track_fastforward,
            TransportButton::Cycle             => &self.cycle,
            TransportButton::Set               => &self.set,
            TransportButton::MarkerRewind      => &self.marker_rewind,
            TransportButton::MarkerFastforward => &self.marker_fastforward,
            TransportButton::Rewind            => &self.rewind,
            TransportButton::Fastforward       => &self.fastforward,
            TransportButton::Stop              => &self.stop,
            TransportButton::Play              => &self.play,
            TransportButton::Record            => &self.record,
        }
    }
}

fn parse_control_mode(val: u8) -> Result<ControlMode> {
    match val {
        0 => Ok(ControlMode::CcMode),
        1 => Ok(ControlMode::Cubase),
        2 => Ok(ControlMode::Dp),
        3 => Ok(ControlMode::Live),
        4 => Ok(ControlMode::ProTools),
        5 => Ok(ControlMode::Sonar),
        n => Err(Error::InvalidControlMode(n)),
    }
}

fn parse_led_mode(val: u8) -> Result<LedMode> {
    match val {
        0 => Ok(LedMode::Internal),
        1 => Ok(LedMode::External),
        n => Err(Error::InvalidLedMode(n)),
    }
}

fn parse_midi_channel_incl_global(val: u8) -> Result<MidiChannel> {
    match val {
        16 => Ok(MidiChannel::Global),
        n if n < 16 => Ok(MidiChannel::Custom(n)),
        n => Err(Error::InvalidMidiChannel(n)),
    }
}
