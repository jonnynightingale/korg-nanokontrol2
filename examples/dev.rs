extern crate korgnanokontrol2;

use std::thread::sleep;
use std::time::Duration;
use std::io::stdin;
use std::error::Error;
use korgnanokontrol2::connection::Connection;
use korgnanokontrol2::parameters::Parameters;

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {:?}", err)
    }
}

fn run() -> Result<(), Box<Error>> {
    let mut connection = Connection::new();
    connection.open(
        |timestamp, midi_channel, control_change, value| {
            println!("{}: {:02X?} {:02X?} {:02X?}", timestamp, midi_channel, control_change, value);
        },
        |timestamp, midi_channel, command_value, data| {
            match command_value {
                0x7F => {
                    let params = Parameters::parse_scene_dump(&data).unwrap();
                    println!("{:#?}", params);
                },
                _ => println!("{}: {:02X?} {:02X?} {:02X?}", timestamp, midi_channel, command_value, data),
            }
        }
    )?;

    let mut input = String::new();
    loop {
        input.clear();
        stdin().read_line(&mut input)?;
        if input.trim() == "q" {
            break;
        } else if input.trim() == "w" {
            connection.current_scene_data_dump_request(0)?;
        } else {
            sleep(Duration::from_millis(200));
        }
    }

    connection.close();
    Ok(())
}
