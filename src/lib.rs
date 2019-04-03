use std::time::Duration;
use std::{io, thread};

use rumqtt::{
    client::Notification, error::ConnectError, MqttClient, MqttOptions, QoS, Receiver,
    ReconnectOptions,
};
use serialport::prelude::*;
use spb_serial_data_parser::extract_msg;

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
                        match extract_msg(&serial_buf[..t], &mut msg_cache) {
                            Some(msg) => op(msg.to_owned()),
                            None => (),
                        };
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => {
                        eprintln!("{:?}", e);
                        thread::sleep(Duration::from_millis(1000))
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            std::process::exit(1);
        }
    }
}

pub fn setup_client(
    broker: String,
    port: u16,
    id: String,
) -> Result<(MqttClient, Receiver<Notification>), ConnectError> {
    let reconnection_options = ReconnectOptions::Always(10);
    let mqtt_options = MqttOptions::new(id, broker, port)
        .set_keep_alive(10)
        .set_reconnect_opts(reconnection_options)
        .set_clean_session(false);

    MqttClient::start(mqtt_options.clone())
}

pub fn setup_client_loop<S: Into<String>, T: Into<String>>(
    broker: S,
    port: u16,
    id: T,
) -> (MqttClient, Receiver<Notification>) {
    let reconnection_options = ReconnectOptions::Always(10);
    let mqtt_options = MqttOptions::new(id, broker, port)
        .set_keep_alive(10)
        .set_reconnect_opts(reconnection_options)
        .set_clean_session(false);

    // if mqtt start failed, it will restart after 10 seconds
    loop {
        match MqttClient::start(mqtt_options.clone()) {
            Ok((mqtt_client, notifications)) => return (mqtt_client, notifications),
            Err(e) => {
                eprintln!("start mqtt client error: {:?}", e);
                thread::sleep(Duration::from_secs(10));
            }
        }
    }
}

pub fn send_msg(client: &mut MqttClient, topic: &str, data: &[u8]) {
    match client.publish(topic, QoS::AtLeastOnce, false, data) {
        Ok(_) => (),
        Err(e) => eprintln!("send message error: {:?}", e),
    }
}
