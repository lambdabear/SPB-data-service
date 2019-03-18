extern crate serialport;

use std::io;
use std::time::Duration;

use serialport::prelude::*;

pub fn receive_data(port_name: &str, baud_rate: &str, op: fn(&[u8]) -> ()) -> () {
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
      println!("Receiving data on {} at {} baud:", &port_name, &baud_rate);
      loop {
        match port.read(serial_buf.as_mut_slice()) {
          //Ok(t) => io::stdout().write_all(&serial_buf[..t]).unwrap(),
          Ok(t) => op(&serial_buf[..t]),
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
