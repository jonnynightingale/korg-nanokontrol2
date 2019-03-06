extern crate midir;

use std::error::Error;
use std::fmt;

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
}
