use midir::{MidiInput, MidiOutput, MidiInputConnection, MidiOutputConnection};

use super::error::Error;
use super::Result;

// #[derive(Debug, Copy, Clone)]
// pub enum Command {
//     NativeModeIoRequest = 0x00,
//     DataDumpRequest     = 0x1F,
//     DataDump            = 0x7F,
// }

#[derive(Debug, Copy, Clone)]
pub enum Command {
    NativeModeInOutRequest = 0x00,
    DataDumpRequest        = 0x1F,
    NativeModeInOut        = 0x40,
    PacketCommunication    = 0x5F,
    DataDump               = 0x7F,
}

impl Command {
    fn try_parse(n: u8) -> Option<Self> {
        match n {
            0x00 => Some(Command::NativeModeInOutRequest),
            0x1F => Some(Command::DataDumpRequest),
            0x40 => Some(Command::NativeModeInOut),
            0x5F => Some(Command::PacketCommunication),
            0x7F => Some(Command::DataDump),
            _ => None,
        }
    }
}

// #[derive(Debug, Copy, Clone)]
// pub enum Function {
//     CurrentSceneDataDumpRequest = 0x10,
//     CurrentSceneDataDump        = 0x40,
//     SceneWriteRequest           = 0x11,
//     ModeRequest                 = 0x12,
// }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Function {
    CurrentSceneDataDump = 0x10,
    DataLoadCompleted    = 0x23,
    DataLoadError        = 0x24,
    WriteCompleted       = 0x21,
    WriteError           = 0x22,
    ModeData             = 0x42,
}

impl Function {
    fn try_parse(n: u8) -> Option<Self> {
        match n {
            0x10 => Some(Function::CurrentSceneDataDump),
            0x23 => Some(Function::DataLoadCompleted),
            0x24 => Some(Function::DataLoadError),
            0x21 => Some(Function::WriteCompleted),
            0x22 => Some(Function::WriteError),
            0x42 => Some(Function::ModeData),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RequestType {
    CurrentSceneDataDumpRequest = 0x10,
    CurrentSceneDataDump        = 0x40,
    SceneWriteRequest           = 0x11,
    ModeRequest                 = 0x12,
}

#[derive(Debug, Copy, Clone)]
pub enum IoType {
    Out = 0x00,
    In  = 0x01,
}

#[derive(Debug, Copy, Clone)]
pub enum DataFormat {
    TwoBytes,
    Variable,
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

    pub fn open<F, G>(
        &mut self,
        mut control_change_callback: F,
        mut system_exclusive_callback: G,
    ) -> Result<()> where
        F: FnMut(u64, u8, u8, u8) + Send + 'static,
        G: FnMut(u64, u8, u8, &[u8]) + Send + 'static {

        let midi_input = MidiInput::new("input")?;
        let midi_output = MidiOutput::new("output")?;

        let mut input_port: Option<usize> = None;
        for i in 0..midi_input.port_count() {
            if midi_input.port_name(i) == Ok("nanoKONTROL2 1 SLIDER/KNOB".to_string()) {
                input_port = Some(i);
                break;
            }
        };

        let mut output_port: Option<usize> = None;
        for i in 0..midi_output.port_count() {
            if midi_output.port_name(i) == Ok("nanoKONTROL2 1 CTRL".to_string()) {
                output_port = Some(i);
                break;
            }
        };

        if input_port == None {
            return Err(Error::MidiInputPortNotFound);
        }

        if output_port == None {
            return Err(Error::MidiOutputPortNotFound);
        }

        self.midi_input_connection = midi_input.connect(input_port.unwrap(), "input_port",
            move |timestamp, message, _| {
            let mut iter = message.iter().enumerate();

            match iter.next() {
                Some((_, &n)) if n & 0b1111_0000 == 0xB0 => {
                    let midi_channel = n & 0b0000_1111;
                    let control_change = match iter.next() {
                        Some((_, &m)) => m,
                        None => return,
                    };
                    let value = match iter.next() {
                        Some((_, &m)) => m,
                        None => return,
                    };
                    control_change_callback(timestamp, midi_channel, control_change, value);
                },
                Some((_, &0xF0)) => {
                    match iter.next() {
                        Some((_, &0x42)) => (),
                        _ => return,
                    };

                    let global_channel = match iter.next() {
                        Some((_, &n)) if n & 0b1111_0000 == 0x40 => n & 0b0000_1111,
                        _ => return,
                    };

                    let (command_index, command_value) = match iter.nth(4) {
                        Some((i, &n)) => (i, n),
                        None => return, 
                    };

                    let data = match command_value & 0b0010_0000 == 0x00 {
                        true => &message[command_index + 1..command_index + 3],
                        false => {
                            let (data_start_index, num_data): (usize, usize) = match iter.next() {
                                Some((_, &0x7F)) => {
                                    match iter.next() {
                                        Some((_, &0x02)) => (),
                                        _ => return,
                                    };
                                    let msb: usize = match iter.next() {
                                        Some((_, &n)) => n as usize,
                                        None => return,
                                    };
                                    let (index, lsb): (_, usize) = match iter.next() {
                                        Some((i, &n)) => (i, n as usize),
                                        None => return,
                                    };
                                    let num = (msb << 7) | lsb;
                                    (index + 2, num)
                                },
                                Some((i, &n)) => (i + 2, n as usize),
                                None => return,
                            };
                            if message.len() < data_start_index + num_data {
                                return;
                            }
                            &message[data_start_index..data_start_index + num_data]
                        },
                    };

                    match iter.nth(data.len()) {
                        Some((_, &0xF7)) => (),
                        _ => return,
                    }

                    system_exclusive_callback(timestamp, global_channel, command_value, data);
                },
                _ => (),
            };
        }, ()).ok();

        self.midi_output_connection = midi_output.connect(output_port.unwrap(), "output_port").ok();

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

    pub fn current_scene_data_dump_request(&mut self, global_channel: u8) -> Result<()> {
        match &mut self.midi_output_connection {
            Some(connection) => {
                let message: [u8; 11] = [
                    0xF0, 0x42, 0x40 | global_channel,
                    0x00, 0x01, 0x13, 0x00,  
                    0x1F,
                    0x10,
                    0x00,
                    0xF7,
                ];
                connection.send(&message)?;
                Ok(())
            },
            None => Err(Error::ConnectionClosed),
        }
    }

    fn scene_write_request(&mut self, global_channel: u8) -> Result<()> {
        match &mut self.midi_output_connection {
            Some(connection) => {
                let message: [u8; 11] = [
                    0xF0, 0x42, 0x40 | global_channel,
                    0x00, 0x01, 0x13, 0x00,  
                    0x1F,
                    0x11,
                    0x00,
                    0xF7,
                ];
                connection.send(&message)?;
                Ok(())
            },
            None => Err(Error::ConnectionClosed),
        }
    }

    fn native_mode_io_request(&mut self, global_channel: u8, io_type: IoType) -> Result<()> {
        match &mut self.midi_output_connection {
            Some(connection) => {
                let message: [u8; 11] = [
                    0xF0, 0x42, 0x40 | global_channel,
                    0x00, 0x01, 0x13, 0x00,  
                    0x00,
                    0x00,
                    io_type as u8,
                    0xF7,
                ];
                connection.send(&message)?;
                Ok(())
            },
            None => Err(Error::ConnectionClosed),
        }
    }

    fn mode_request(&mut self, global_channel: u8, io_type: IoType) -> Result<()> {
        match &mut self.midi_output_connection {
            Some(connection) => {
                let message: [u8; 11] = [
                    0xF0, 0x42, 0x40 | global_channel,
                    0x00, 0x01, 0x13, 0x00,  
                    0x1F,
                    0x12,
                    0x00,
                    0xF7,
                ];
                connection.send(&message)?;
                Ok(())
            },
            None => Err(Error::ConnectionClosed),
        }
    }

    fn current_scene_data_dump(&mut self, global_channel: u8, scene_data: &[u8; 389]) -> Result<()> {
        match &mut self.midi_output_connection {
            Some(connection) => {
                let mut message: [u8; 403] = [0; 403];
                message[0] = 0xF0;
                message[1] = 0x42;
                message[2] = 0x40 | global_channel;
                message[3] = 0x00;
                message[4] = 0x01;
                message[5] = 0x13;
                message[6] = 0x00;
                message[7] = 0x7F;
                message[8] = 0x7F;
                message[9] = 0x02;
                message[10] = 0x03;
                message[11] = 0x05;
                message[12] = 0x40;
                message[13..402].clone_from_slice(scene_data);
                message[402] = 0xF7;
                connection.send(&message)?;
                Ok(())
            },
            None => Err(Error::ConnectionClosed),
        }
    }
}
