extern crate korgnanokontrol2;

use std::thread::sleep;
use std::time::Duration;
use std::io::stdin;
use std::error::Error;
use korgnanokontrol2::connection::Connection;

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {:?}", err)
    }
}

fn run() -> Result<(), Box<Error>> {
    let mut connection = Connection::new();
    connection.open(
        |timestamp, midi_channel, control_change, value|
        {
            println!("{}: {:02X?} {:02X?} {:02X?}", timestamp, midi_channel, control_change, value);
        },
        |_, _, _| ()
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
