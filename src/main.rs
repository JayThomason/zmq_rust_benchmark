extern crate clap;
use clap::{Arg, App, AppSettings, SubCommand};

fn get_addr(matches: &clap::ArgMatches) -> String {
    let default_protocol = "tcp";
    let default_address = "127.0.0.1";
    let default_port = "4321";
    
    let protocol = matches.value_of("protocol").unwrap_or(default_protocol);
    let address = matches.value_of("address").unwrap_or(default_address);
    let port: i32 = matches.value_of("port").unwrap_or(default_port).parse().unwrap();

    if protocol == "tcp" {
        return format!("{}://{}:{}", protocol, address, port);
    } else if protocol == "ipc" {
        return format!("{}://{}", protocol, address);
    } else {
        panic!(format!("Unsupported protocol {}, please use tcp or ipc", protocol));
    }
}

fn bind(addr: String) {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::REP).unwrap();
    println!("Binding to {}", addr);
    socket.bind(&addr).unwrap();
    loop {
        let mut recv_msg = zmq::Message::new();
        let mut result = socket.recv(&mut recv_msg, 0);
        assert!(result.is_ok());
        result = socket.send(recv_msg, 0);
        assert!(result.is_ok());
    }
}
        

fn connect(addr: String, num_msgs: i32, msg_size: usize) {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::REQ).unwrap();
    println!("Connecting to {}", addr);
    socket.connect(&addr).unwrap();
    let start = std::time::Instant::now();
    for _ in 1..num_msgs {
        let send_msg = zmq::Message::with_size(msg_size);
        let mut recv_msg = zmq::Message::with_size(msg_size);
        socket.send(send_msg, 0).unwrap();
        let result = socket.recv(&mut recv_msg, 0);
        assert!(result.is_ok());
    }
    let elapsed: f64 = start.elapsed().as_secs_f64();
    println!("Time elapsed for {} {} byte ping-pongs: {}", num_msgs, msg_size, elapsed);
    println!("Average time per ping-pong: {}", elapsed / num_msgs as f64);
}

fn main() {
    let matches = App::new("zmq_bench")
        .version("0.1")
        .author("Jay Thomason <jay@covariant.ai>")
        .about("Benchmark zeromq in Rust.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(Arg::with_name("protocol")
             .help("The protocol to use (e.g. tcp or ipc).")
             .short("p")
             .long("protocol")
             .takes_value(true))
        .arg(Arg::with_name("address")
             .help("The address to use (e.g. 127.0.0.1 for tcp or /tmp/zmq_bench/0 for ipc")
             .short("a")
             .long("address")
             .takes_value(true))
        .arg(Arg::with_name("port")
             .help("The port to use (defaults to 4321)")
             .long("port")
             .takes_value(true))
        .subcommand(SubCommand::with_name("bind")
                    .about("Bind to a port"))
        .subcommand(SubCommand::with_name("connect")
                    .about("Connect to a port")
                    .arg(Arg::with_name("num_msgs")
                         .short("nb")
                         .long("num_msgs")
                         .takes_value(true))
                    .arg(Arg::with_name("num_bytes")
                         .short("b")
                         .long("num_bytes")
                         .takes_value(true)))
        .get_matches();

    let addr = get_addr(&matches);

    match matches.subcommand() {
        ("bind", _) => {
            bind(addr);
        },
        ("connect", Some(connect_matches)) => {
            let num_msgs: i32 = connect_matches.value_of("num_msgs").unwrap_or("100").parse().unwrap();
            let msg_size: usize = connect_matches.value_of("num_bytes").unwrap_or("1000000").parse().unwrap();
            connect(addr, num_msgs, msg_size); 
        }
        (command, _) => {
            panic!("Unexpected subcommand {}", command);
        }
    }
}
