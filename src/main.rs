use clap::{App, AppSettings, Arg};
use crossbeam_channel::bounded;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use spb_data_service::{receive_data, send_msg, setup_client_loop};
use spb_serial_data_parser::{parse, Battery, DcOut, SpbState, SwIn, SwOut, Ups};

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
    let (mut mqtt_client, _notifications) = setup_client_loop(broker, port, id);
    // mqtt_client.subscribe(topic, QoS::AtLeastOnce).unwrap();

    // setup data store
    let data_in = Arc::new(Mutex::new(SwIn::new(false, false, false, false)));
    let data_no = Arc::new(Mutex::new(SwOut::new(
        false, false, false, false, false, false,
    )));
    let data_ups = Arc::new(Mutex::new(Ups::new(0.0, 0.0, 0.0, false)));
    let data_bt = Arc::new(Mutex::new(Battery::new(0.0, 0.0, 0)));
    let data_dc = Arc::new(Mutex::new(DcOut::new(0.0, 0.0, 0.0)));
    let in1 = data_in.clone();
    let no1 = data_no.clone();
    let ups1 = data_ups.clone();
    let bt1 = data_bt.clone();
    let dc1 = data_dc.clone();

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
                    Ok(s) => {
                        // if the data received right now is diffrent from previous data, then store and send the data
                        match s {
                            SpbState::SwIn(swin) => match in1.lock() {
                                Ok(mut s) => {
                                    if *s != swin {
                                        match serde_json::to_string(&swin) {
                                            Ok(m) => {
                                                send_msg(&mut mqtt_client, topic, &m.into_bytes())
                                            }
                                            Err(e) => eprintln!("{:?}", e),
                                        }
                                        *s = swin
                                    }
                                }
                                Err(e) => eprintln!("{}", e),
                            },
                            SpbState::SwOut(swout) => match no1.lock() {
                                Ok(mut s) => {
                                    if *s != swout {
                                        match serde_json::to_string(&swout) {
                                            Ok(m) => {
                                                send_msg(&mut mqtt_client, topic, &m.into_bytes())
                                            }
                                            Err(e) => eprintln!("{:?}", e),
                                        }
                                        *s = swout
                                    }
                                }
                                Err(e) => eprintln!("{}", e),
                            },
                            SpbState::Ups(ups) => match ups1.lock() {
                                Ok(mut s) => {
                                    if *s != ups {
                                        match serde_json::to_string(&ups) {
                                            Ok(m) => {
                                                send_msg(&mut mqtt_client, topic, &m.into_bytes())
                                            }
                                            Err(e) => eprintln!("{:?}", e),
                                        }
                                        *s = ups
                                    }
                                }
                                Err(e) => eprintln!("{}", e),
                            },
                            SpbState::Bt(bt) => match bt1.lock() {
                                Ok(mut s) => {
                                    if *s != bt {
                                        match serde_json::to_string(&bt) {
                                            Ok(m) => {
                                                send_msg(&mut mqtt_client, topic, &m.into_bytes())
                                            }
                                            Err(e) => eprintln!("{:?}", e),
                                        }
                                        *s = bt
                                    }
                                }
                                Err(e) => eprintln!("{}", e),
                            },
                            SpbState::Dc(dcout) => match dc1.lock() {
                                Ok(mut s) => {
                                    if *s != dcout {
                                        match serde_json::to_string(&dcout) {
                                            Ok(m) => {
                                                send_msg(&mut mqtt_client, topic, &m.into_bytes())
                                            }
                                            Err(e) => eprintln!("{:?}", e),
                                        }
                                        *s = dcout
                                    }
                                }
                                Err(e) => eprintln!("{}", e),
                            },
                        }
                    }
                    Err(e) => eprintln!("{:?}", e),
                };
            }
            Err(e) => eprintln!("{:?}", e),
        }
    });

    // receive server messages by subscribe message topic using mqtt client
    // let h3 = thread::spawn(move || {
    //     for notification in notifications {
    //         //println!("{:?}", notification);
    //         match notification {
    //             rumqtt::client::Notification::Publish(publish) => {
    //                 io::stdout().write_all(&publish.payload).unwrap();
    //                 print!("\n")
    //             }
    //             _ => (),
    //         }
    //     }
    // });

    // send spb state per 60 seconds
    let h4 = thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(60));
        print!("\n**************\n{:?}\n", std::time::SystemTime::now());
        println!("{:?}", *data_in);
        println!("{:?}", *data_no);
        println!("{:?}", *data_ups);
        println!("{:?}", *data_bt);
        println!("{:?}", *data_dc);
        print!("\n**************\n");
    });

    let handles = vec![h1, h2, h4];
    for h in handles {
        h.join().unwrap();
    }
}
