use std::error::Error;
use std::fmt;
use midir::{MidiInput, MidiOutput, MidiInputConnection, MidiOutputConnection};

#[derive(Debug)]
pub struct ConnectionError(String);

impl fmt::Display for ConnectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Korg nanoKONTROL2 Error: {}", self.0)
    }
}

impl Error for ConnectionError {}

#[derive(Debug, Copy, Clone)]
pub enum Command {
    NativeModeIoRequest = 0x00,
    DataDumpRequest     = 0x1F,
    DataDump            = 0x7F,
}

#[derive(Debug, Copy, Clone)]
pub enum Function {
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
            return Result::Err(Box::new(ConnectionError("No MIDI input ports found.".into())));
        }

        if midi_input.port_count() > 1 {
            return Result::Err(Box::new(ConnectionError("Multiple MIDI input ports found.".into())));
        }

        if midi_output.port_count() == 0 {
            return Result::Err(Box::new(ConnectionError("No MIDI ouput ports found.".into())));
        }

        if midi_output.port_count() > 1 {
            return Result::Err(Box::new(ConnectionError("Multiple MIDI output ports found.".into())));
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
            Command::DataDumpRequest,
            Function::CurrentSceneDataDumpRequest,
            slider_index
        );
        match &mut self.midi_output_connection {
            Some(connection) => {
                connection.send(&message)?;
            },
            None => (),
        }
        Ok(())
    }

    fn generate_two_byte_data_message(
        channel: u8,
        command: Command,
        function: Function,
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
