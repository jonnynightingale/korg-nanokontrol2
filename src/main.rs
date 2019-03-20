mod lib;

use std::thread::sleep;
use std::time::Duration;
use std::io::stdin;
use std::error::Error;
use lib::{Connection, KorgNanokontrol2State};

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {:?}", err)
    }
}

fn run() -> Result<(), Box<Error>> {
    let mut connection = Connection::new();
    connection.open(|stamp, message| {
        if message.len() > 50 {
            let mut state = KorgNanokontrol2State::default();
            state.parse_scene_dump(&message);
            println!("{:#?}", state);
        } else {
            println!("{}: {:02X?} (len = {})", stamp, message, message.len());
        }
    })?;

    let mut input = String::new();
    loop {
        input.clear();
        stdin().read_line(&mut input)?;
        if input.trim() == "q" {
            break;
        } else if input.trim() == "w" {
            connection.get_slider_value_raw(0)?;
        } else {
            sleep(Duration::from_millis(200));
        }
    }

    connection.close();
    Ok(())
}