use clap::App;
use clap::Arg;
use clap::SubCommand;
use log::error;
use log::info;
use std::env;
use std::fs;
use std::process::exit;
use threema::packets::Message;
use threema::packets::Packet;
use threema::Threema;
use threema::ThreemaID;

fn send(mut threema: Threema, recipient: &str, message: String) {
    let recipient = match ThreemaID::from_string(recipient) {
        Ok(id) => id,
        Err(e) => {
            error!("Invalid threema id: {:?}", e);
            exit(1);
        }
    };
    let mid = match threema.send_text_message(recipient, message) {
        Ok(mid) => mid,
        Err(e) => {
            error!("Couldn't send message: {:?}", e);
            exit(1);
        }
    };

    loop {
        let packet = match threema.receive_packet() {
            Ok((p, _)) => p,
            Err(e) => {
                error!("Error during receiving packets: {:?}", e);
                exit(1);
            }
        };
        if let Packet::ServerAck(_, ack_mid) = packet {
            if ack_mid == mid {
                info!("Message processed by server");
                return;
            }
        }
    }
}

fn receive(mut threema: Threema) {
    info!("Entering receive loop");
    loop {
        let msg = match threema.receive() {
            Ok(m) => m,
            Err(e) => {
                error!("Error during receiving packets: {:?}", e);
                exit(1);
            }
        };

        let sender = msg.sender;
        let mid = msg.msg_id;
        match msg.data {
            Message::Text(t) => {
                println!("{} [{}] `{}`", mid, sender, t.message);
            }
            Message::DeliveryReceipt(status, mid) => {
                println!("{} [{}] => {:?}", mid, sender, status);
            }
            other => {
                println!("{} [{}] :: {:?}", mid, sender, other);
            }
        }
    }
}

fn setup_logging() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
}

fn main() {
    setup_logging();
    let matches = App::new("threema-cli")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("identity")
                .short("i")
                .long("identity")
                .value_name("FILE")
                .default_value("identity")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("identity_password")
                .short("p")
                .long("password")
                .value_name("PWD")
                .default_value("testtest")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("send")
                .arg(
                    Arg::with_name("nick")
                        .short("n")
                        .long("nick")
                        .value_name("NICK")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("recipient")
                        .value_name("RECIPIENT")
                        .required(true),
                )
                .arg(
                    Arg::with_name("message")
                        .value_name("MESSAGE")
                        .required(true),
                ),
        )
        .subcommand(SubCommand::with_name("receive"))
        .get_matches();

    let ifile = matches.value_of("identity").unwrap();
    info!("Loading identity from {}", ifile);
    let data = match fs::read_to_string(ifile) {
        Ok(d) => d,
        Err(e) => {
            error!("Could't read identity file: {:?}", e);
            exit(1);
        }
    };

    let mut threema =
        match Threema::from_backup(&data, matches.value_of("identity_password").unwrap()) {
            Ok(t) => t,
            Err(e) => {
                error!("Couldn't initialize client: {:?}", e);
                exit(1);
            }
        };
    info!("Connecting to backend");
    if let Err(e) = threema.connect() {
        error!("Couldn't connect: {:?}", e);
        exit(1);
    }

    match matches.subcommand() {
        ("send", Some(matches)) => {
            if let Some(n) = matches.value_of("nick") {
                threema.nick = Some(n.to_string());
            }
            send(
                threema,
                matches.value_of("recipient").unwrap(),
                matches.value_of("message").unwrap().to_owned(),
            )
        }
        ("receive", _) => receive(threema),
        (other, _) => {
            error!("Unexpected command {}", other);
            exit(1)
        }
    }
}
