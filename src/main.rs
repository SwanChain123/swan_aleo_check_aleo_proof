mod server;

use snarkvm::{
    console::network::Testnet3 as CurrentNetwork,
    ledger::coinbase::CoinbasePuzzle,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "swan-aleo-check-aleo-proof", about = "Check Aleo proof", setting = structopt::clap::AppSettings::ColoredHelp)]
struct Opt {
    #[structopt(short = "p", long = "port")]
    port: Option<u16>,

    #[structopt(short = "d", long = "data")]
    data: Option<String>,
}


fn main() {
    let opt = Opt::from_args();

    let coinbase_puzzle = CoinbasePuzzle::<CurrentNetwork>::load().unwrap();

    if let Some(data) = opt.data {
        server::check_aleo_proof(&data, coinbase_puzzle);    
    } else {
        let port = opt.port.unwrap_or(8080);
        server::start_rpc_server(port, coinbase_puzzle);
    }
}