use clap::Parser;
use std::net::IpAddr;
//use std::vec;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
#[derive(Debug, Parser)]

struct Args {
    #[arg()]
    addr: IpAddr, // this takes the input after flag "--" as an IpAddr enum
    #[arg(long, default_value_t = 1)] // means it is flagged using --port_start <port>
    port_start: u16,
    #[arg(long, default_value_t = 1024)] // means it is flagged using --port_end <port>
    port_end: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> /*return error without crash*/ {
    let args = Args::parse(); /*this uses the struct Args and
    assignes the parse'd string as an addr attribute */
    assert!(args.port_start > 0);
    assert!(args.port_end >= args.port_start);

    let rt = Runtime::new()?; // ? is basically unwrap but returns err to func
    let (tx, mut rx) = mpsc::channel(10); // creates a channel of maximum 10 messages
    // tx means transmit, rx means recieve... rx is mutable because it will change per while
    let mut tasks: Vec<_> = vec![];
    rt.block_on(async {
        //blocks the main() function
        for port in args.port_start..=args.port_end {
            println!("? = {}:{}", args.addr, port); // simply printing all ports
            let tx = tx.clone(); // shadows the previous tx each time so that message is new
            let task = tokio::spawn(async move {
                let scan_attempt = scan(args.addr, port, tx).await;
                if let Err(err) = scan_attempt {
                    eprintln!("error: {err}");
                }
            });
            tasks.push(task); // just added this line
            let _ = task.await; // waits for everything to finish
        }
    });

    while let Ok((addr, port)) = rx.try_recv() {
        //loops rx (hence mut) for each message in channel
        println!("= {}:{}", addr, port); // prints every addr and port from Ok(_open) ports
    }

    Ok(()) // end the function if no errors occur
}

async fn scan(
    addr: IpAddr,
    port: u16,
    results_tx: mpsc::Sender<(IpAddr, u16)>,
) -> Result<(), mpsc::error::SendError<(IpAddr, u16)>> {
    // move: all incoming variables are captured and locked within the scope
    let connection_attempt = TcpStream::connect((addr, port)).await;
    if let Ok(_open) = connection_attempt {
        // only do following if tcp connects (port is open)
        results_tx.send((addr, port)).await?;
        //&mut open_ports.push((args.addr, port));
        println!("= {}:{}", addr, port);
    };
    //println!("= {}:{}", args.addr, port);
    Ok(())
}
