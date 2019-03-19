use std::{thread, time::Duration};

use rumqtt::{client::Notification, MqttClient, MqttOptions, QoS, Receiver, ReconnectOptions};

pub fn setup_client(broker: &str, port: u16, id: &str) -> (MqttClient, Receiver<Notification>) {
    let reconnection_options = ReconnectOptions::Always(10);
    let mqtt_options = MqttOptions::new(id, broker, port)
        .set_keep_alive(10)
        .set_reconnect_opts(reconnection_options)
        .set_clean_session(false);

    match MqttClient::start(mqtt_options) {
        Ok((mqtt_client, notifications)) => (mqtt_client, notifications),
        Err(e) => {
            eprintln!("start mqtt client error: {:?}", e);
            thread::sleep(Duration::from_secs(10));
            setup_client(broker, port, id)
        }
    }
}

pub fn send_msg(client: &mut MqttClient, topic: &str, data: &[u8]) {
    match client.publish(topic, QoS::AtLeastOnce, false, data) {
        Ok(_) => (),
        Err(e) => eprintln!("send message error: {:?}", e),
    }
}
