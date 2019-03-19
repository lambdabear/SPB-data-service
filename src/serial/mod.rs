use std::io;
use std::time::Duration;

use serialport::prelude::*;

mod parser;
use parser::get_msg;

// receive data from serial port, use op closure to process data
pub fn receive_data<F: Fn(Vec<u8>) -> ()>(port_name: &str, baud_rate: &str, op: F) -> () {
    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(10);
    if let Ok(rate) = baud_rate.parse::<u32>() {
        settings.baud_rate = rate;
    } else {
        eprintln!("Error: Invalid baud rate '{}' specified", baud_rate);
        std::process::exit(1);
    }

    match serialport::open_with_settings(&port_name, &settings) {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            let mut msg_cache: Vec<u8> = vec![];
            println!("Receiving data on {} at {} baud:", &port_name, &baud_rate);
            loop {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(t) => {
                        match get_msg(&serial_buf[..t], &mut msg_cache) {
                            Some(msg) => op(msg.to_owned()),
                            None => (),
                        };
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            std::process::exit(1);
        }
    }
}
