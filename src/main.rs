use clap::{App, AppSettings, Arg};
use crossbeam_channel::bounded;
use rumqtt::QoS;

use std::{io, io::Write, thread};

use spb_serial_data_parser::{parse, to_json};
use spb_serial_data_receiver::{receive_data, send_msg, setup_client};

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

    // set serial port arguments
    let port_name = matches.value_of("port").unwrap().to_owned();
    let baud_rate = matches.value_of("baud").unwrap().to_owned();

    // setup mqtt client
    let broker = "test.mosquitto.org";
    let port = 1883;
    let id = "spb001";
    let topic = "hello/world";
    let (mut mqtt_client, notifications) = setup_client(broker, port, id);
    mqtt_client.subscribe(topic, QoS::AtLeastOnce).unwrap();

    // setup channel for communication between threads
    let (s, r) = bounded(1);

    // get serial port data, send the data through channel
    let h1 = thread::spawn(move || {
        let send = move |data: Vec<u8>| -> () {
            match s.send(data) {
                Ok(_) => (),
                Err(e) => eprintln!("send msg through thread error: {:?}", e),
            }
        };
        receive_data(&port_name, &baud_rate, send);
    });

    // receive serial data from channel, publish the data using mqtt client
    let h2 = thread::spawn(move || loop {
        match r.recv() {
            Ok(msg) => {
                match parse(&msg) {
                    Ok(s) => match to_json(s) {
                        Ok(m) => send_msg(&mut mqtt_client, topic, &m.into_bytes()),
                        Err(e) => eprintln!("{:?}", e),
                    },
                    Err(e) => eprintln!("{:?}", e),
                };
            }
            Err(e) => eprintln!("{:?}", e),
        }
    });

    // receive server messages by subscribe message topic using mqtt client
    let h3 = thread::spawn(move || {
        for notification in notifications {
            //println!("{:?}", notification);
            match notification {
                rumqtt::client::Notification::Publish(publish) => {
                    io::stdout().write_all(&publish.payload).unwrap();
                    print!("\n")
                }
                _ => (),
            }
        }
    });

    let handles = vec![h1, h2, h3];
    for h in handles {
        h.join().unwrap();
    }
}
