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

        let global_channel_val = raw_scene_data[index_to_data_dump_index(0)];
        self.global_channel = match global_channel_val {
            n if n < 16 => n,
            n => return Err(Error::InvalidGlobalChannel(n)),
        };

        let control_mode_val = raw_scene_data[index_to_data_dump_index(1)];
        self.control_mode = ControlMode::from(control_mode_val);

        let led_mode_val = raw_scene_data[index_to_data_dump_index(2)];
        self.led_mode = LedMode::from(led_mode_val);

        for i in 0..8 {
            let index: usize = 3 + (i * 31);
            self.groups[i] = parse_group_data(&raw_scene_data, index);
        }

        let transport_button_channel_val = raw_scene_data[index_to_data_dump_index(251)];
        self.transport_button_channel = MidiChannel::from(transport_button_channel_val);

        self.track_rewind       = parse_button_data(&raw_scene_data, 252);
        self.track_fastforward  = parse_button_data(&raw_scene_data, 258);
        self.cycle              = parse_button_data(&raw_scene_data, 264);
        self.set                = parse_button_data(&raw_scene_data, 270);
        self.marker_rewind      = parse_button_data(&raw_scene_data, 276);
        self.marker_fastforward = parse_button_data(&raw_scene_data, 282);
        self.rewind             = parse_button_data(&raw_scene_data, 288);
        self.fastforward        = parse_button_data(&raw_scene_data, 294);
        self.stop               = parse_button_data(&raw_scene_data, 300);
        self.play               = parse_button_data(&raw_scene_data, 306);
        self.record             = parse_button_data(&raw_scene_data, 312);

        for i in 0..5 {
            let data_dump_index: usize = index_to_data_dump_index(318 + i);
            self.custom_daw_assign[i] = raw_scene_data[data_dump_index];
        }

        Ok(())
    }

    pub fn create_scene_dump(&self) -> [u8; 389] {
        let mut scene_dump: [u8; 389] = [0; 389];

        scene_dump[index_to_data_dump_index(0)] = self.global_channel as u8;
        scene_dump[index_to_data_dump_index(1)] = self.control_mode as u8;
        scene_dump[index_to_data_dump_index(2)] = self.led_mode as u8;

        for i in 0..8 {
            let index: usize = 3 + (i * 31);
            add_group_data_to_dump(&mut scene_dump, &self.groups[i], index);
        }

        add_button_data_to_dump(&mut scene_dump, &self.track_rewind,       252);
        add_button_data_to_dump(&mut scene_dump, &self.track_fastforward,  258);
        add_button_data_to_dump(&mut scene_dump, &self.cycle,              264);
        add_button_data_to_dump(&mut scene_dump, &self.set,                270);
        add_button_data_to_dump(&mut scene_dump, &self.marker_rewind,      276);
        add_button_data_to_dump(&mut scene_dump, &self.marker_fastforward, 282);
        add_button_data_to_dump(&mut scene_dump, &self.rewind,             288);
        add_button_data_to_dump(&mut scene_dump, &self.fastforward,        294);
        add_button_data_to_dump(&mut scene_dump, &self.stop,               300);
        add_button_data_to_dump(&mut scene_dump, &self.play,               306);
        add_button_data_to_dump(&mut scene_dump, &self.record,             312);

        for i in 0..5 {
            let data_dump_index: usize = index_to_data_dump_index(318 + i);
            scene_dump[data_dump_index] = self.custom_daw_assign[i];
        }

        scene_dump
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

const fn index_to_data_dump_index(index: usize) -> usize {
    ((index / 7) * 8) + ((index % 7) + 1)
}

fn parse_group_data(raw_scene_data: &[u8], index: usize) -> ControllerGroupParameters {
    ControllerGroupParameters {
        channel: MidiChannel::from(raw_scene_data[index_to_data_dump_index(index)]),
        slider: parse_slider_data(&raw_scene_data, index + 1),
        knob: parse_slider_data(&raw_scene_data, index + 7),
        solo_button: parse_button_data(&raw_scene_data, index + 13),
        mute_button: parse_button_data(&raw_scene_data, index + 19),
        record_button: parse_button_data(&raw_scene_data, index + 25),
    }
}

fn parse_slider_data(raw_scene_data: &[u8], index: usize) -> SliderParameters {
    SliderParameters {
        assign_type: SliderAssignType::from(raw_scene_data[index_to_data_dump_index(index)]),
        note_number: raw_scene_data[index_to_data_dump_index(index + 2)],
        min_value: raw_scene_data[index_to_data_dump_index(index + 3)],
        max_value: raw_scene_data[index_to_data_dump_index(index + 4)],
    }
}

fn parse_button_data(raw_scene_data: &[u8], index: usize) -> ButtonParameters {
    ButtonParameters {
        assign_type: ButtonAssignType::from(raw_scene_data[index_to_data_dump_index(index)]),
        behavior: ButtonBehavior::from(raw_scene_data[index_to_data_dump_index(index + 1)]),
        note_number: raw_scene_data[index_to_data_dump_index(index + 2)],
        off_value: raw_scene_data[index_to_data_dump_index(index + 3)],
        on_value: raw_scene_data[index_to_data_dump_index(index + 4)],
    }
}

fn add_group_data_to_dump(mut dump: &mut [u8], group_params: &ControllerGroupParameters, index: usize) {
    dump[index_to_data_dump_index(index)] = group_params.channel.into();
    add_slider_data_to_dump(&mut dump, &group_params.slider, index_to_data_dump_index(index + 1));
    add_slider_data_to_dump(&mut dump, &group_params.knob, index_to_data_dump_index(index + 7));
    add_button_data_to_dump(&mut dump, &group_params.solo_button, index_to_data_dump_index(index + 13));
    add_button_data_to_dump(&mut dump, &group_params.mute_button, index_to_data_dump_index(index + 19));
    add_button_data_to_dump(&mut dump, &group_params.record_button, index_to_data_dump_index(index + 25));
}

fn add_slider_data_to_dump(dump: &mut [u8], slider_params: &SliderParameters, index: usize) {
    dump[index_to_data_dump_index(index)] = slider_params.assign_type as u8;
    dump[index_to_data_dump_index(index + 2)] = slider_params.note_number as u8;
    dump[index_to_data_dump_index(index + 3)] = slider_params.min_value as u8;
    dump[index_to_data_dump_index(index + 4)] = slider_params.max_value as u8;
}

fn add_button_data_to_dump(dump: &mut [u8], button_params: &ButtonParameters, index: usize) {
    dump[index_to_data_dump_index(index)] = button_params.assign_type as u8;
    dump[index_to_data_dump_index(index + 1)] = button_params.behavior as u8;
    dump[index_to_data_dump_index(index + 2)] = button_params.note_number as u8;
    dump[index_to_data_dump_index(index + 3)] = button_params.off_value as u8;
    dump[index_to_data_dump_index(index + 4)] = button_params.on_value as u8;
}
