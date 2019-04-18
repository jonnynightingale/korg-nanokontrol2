pub mod connection;
pub mod data;
pub mod enums;
pub mod error;
pub mod parameters;

use connection::Connection;
use data::Data;
use enums::*;
pub use error::{Result, Error};
use parameters::*;

pub struct KorgNanokontrol2 {
    connection: Connection,
    parameters: Parameters,
    data: Data,
}

impl KorgNanokontrol2 {
    pub fn connect(&mut self) -> Result<()> {
        self.connection.open(
            |_, _, _, _| (),
            |timestamp, midi_channel, command_value, function_id, data| {
                match command_value {
                    0x40 => (),
                    0x5F => (),
                    0x7F => (),
                    _ => (),
                }
            })?;
        Ok(())
    }

    pub fn get_slider_value(&self, group_index: usize) -> f32 {
        let value = self.data.groups[group_index].slider_value;
        let slider_parameters = &self.parameters.groups[group_index].slider;
        get_continuous_value(value, slider_parameters)
    }

    pub fn get_slider_value_raw(&self, group_index: usize) -> u8 {
        self.data.groups[group_index].slider_value
    }

    pub fn get_knob_value(&self, group_index: usize) -> f32 {
        let value = self.data.groups[group_index].knob_value;
        let slider_parameters = &self.parameters.groups[group_index].knob;
        get_continuous_value(value, slider_parameters)
    }

    pub fn get_knob_value_raw(&self, group_index: usize) -> u8 {
        self.data.groups[group_index].knob_value
    }

    pub fn get_transport_button_state(&self, button_type: TransportButton) -> bool {
        let value = self.data.get_transport_button_value(button_type);
        let button_parameters = self.parameters.get_transport_button_parameters(button_type);
        get_button_state(value, button_parameters)
    }

    pub fn get_solo_button_state(&self, group_index: usize) -> bool {
        let value = self.data.groups[group_index].solo;
        let button_parameters = &self.parameters.groups[group_index].solo_button;
        get_button_state(value, button_parameters)
    }

    pub fn get_mute_button_state(&self, group_index: usize) -> bool {
        let value = self.data.groups[group_index].mute;
        let button_parameters = &self.parameters.groups[group_index].mute_button;
        get_button_state(value, button_parameters)
    }

    pub fn get_record_button_state(&self, group_index: usize) -> bool {
        let value = self.data.groups[group_index].record;
        let button_parameters = &self.parameters.groups[group_index].record_button;
        get_button_state(value, button_parameters)
    }
}

fn get_continuous_value(value: u8, slider_parameters: &SliderParameters) -> f32 {
    let min_value = slider_parameters.min_value;
    let max_value = slider_parameters.max_value;

    let range = max_value - min_value;

    match range {
        0 => 0.0,
        range => (value - min_value) as f32 / range as f32,
    }
}

fn get_button_state(value: u8, button_parameters: &ButtonParameters) -> bool {
    let off_value = button_parameters.off_value;
    let on_value = button_parameters.on_value;

    match value {
        n if n == off_value => false,
        n if n == on_value => true,
        n => {
            // use the Hamming distance to pick a value, favoring false when equal
            let off_distance = (n ^ off_value).count_zeros();
            let on_distance = (n ^ on_value).count_zeros();
            (off_distance < on_distance)
        }
    }
}
