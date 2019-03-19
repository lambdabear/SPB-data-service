use clap::{App, AppSettings, Arg};
use crossbeam_channel::bounded;
use rumqtt::QoS;
use std::{io, io::Write, thread};

mod serial;
use serial::receive_data;

mod mqtt;
use mqtt::{send_msg, setup_client};

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

    let port_name = matches.value_of("port").unwrap().to_owned();
    let baud_rate = matches.value_of("baud").unwrap().to_owned();

    let broker = "test.mosquitto.org";
    let port = 1883;
    let id = "spb001";
    let topic = "hello/world";

    let (mut mqtt_client, notifications) = setup_client(broker, port, id);

    mqtt_client.subscribe(topic, QoS::AtLeastOnce).unwrap();

    let (s, r) = bounded(1);

    let send = move |data: Vec<u8>| -> () {
        match s.send(data) {
            Ok(_) => (),
            Err(e) => eprintln!("send msg through thread error: {:?}", e),
        }
    };

    thread::spawn(move || {
        receive_data(&port_name, &baud_rate, send);
    });

    thread::spawn(move || loop {
        match r.recv() {
            Ok(msg) => send_msg(&mut mqtt_client, topic, &msg),
            Err(_) => (),
        }
    });

    for notification in notifications {
        //println!("{:?}", notification);
        match notification {
            rumqtt::client::Notification::Publish(publish) => {
                io::stdout().write_all(&publish.payload).unwrap()
            }
            _ => (),
        }
    }
}
