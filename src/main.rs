extern crate clap;

use clap::{App, AppSettings, Arg};

fn main() {
    let matches = App::new("Smart power box serial data receiver")
        .about("Receive SBP data to print to console")
        .setting(AppSettings::DisableVersion)
        .arg(
            Arg::with_name("port")
                .help("The device path to a serial port")
                .use_delimiter(false)
                .required(true),
        )
        .arg(
            Arg::with_name("baud")
                .help("The baud rate to connect at")
                .use_delimiter(false)
                .required(true),
        )
        .get_matches();

    let port_name = matches.value_of("port").unwrap();
    let baud_rate = matches.value_of("baud").unwrap();

    spb_serial_data_receiver::receive_data(port_name, baud_rate);
}
