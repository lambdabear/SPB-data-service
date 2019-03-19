extern crate clap;

use clap::{App, AppSettings, Arg};
use rumqtt::{MqttClient, MqttOptions, QoS, ReconnectOptions};
use std::{io, io::Write, thread};

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

    let reconnection_options = ReconnectOptions::Always(10);
    let mqtt_options = MqttOptions::new("test-pubsub2", broker, port)
        .set_keep_alive(10)
        .set_reconnect_opts(reconnection_options)
        .set_clean_session(false);

    let (mut mqtt_client, notifications) = MqttClient::start(mqtt_options).unwrap();
    mqtt_client
        .subscribe("hello/world", QoS::AtLeastOnce)
        .unwrap();

    let op = move |data: &[u8]| -> () {
        mqtt_client
            .publish("hello/world", QoS::AtLeastOnce, false, data)
            .unwrap();
    };

    thread::spawn(move || {
        spb_serial_data_receiver::receive_data(&port_name, &baud_rate, op);
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
