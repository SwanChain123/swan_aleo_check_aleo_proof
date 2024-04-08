use std::str::FromStr;

use snarkvm::{
    console::{network::Testnet3 as CurrentNetwork, types::Address},
    ledger::coinbase::{CoinbasePuzzle, EpochChallenge, PartialSolution, ProverSolution, PuzzleCommitment, PuzzleProof},
    utilities::FromBytes,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "swan-aleo-check-aleo-proof", about = "Check Aleo proof", setting = structopt::clap::AppSettings::ColoredHelp)]
struct Opt {
    #[structopt(short = "p", long = "proof")]
    proof: Option<String>,

    #[structopt(short = "a", long = "address")]
    address: Option<String>,

    #[structopt(short = "n", long = "nonce_ex")]
    nonce_ex: Option<u64>,

    #[structopt(short = "l", long = "nonce_len")]
    nonce_len: Option<u64>,
}

fn main() {
    let opt = Opt::from_args();

    let address = opt.address.unwrap();

    let nonce_ex = if opt.nonce_ex.is_some() && opt.nonce_len.is_some() {
        let displacement = (8u64 - opt.nonce_len.unwrap()) << 3;
        opt.nonce_ex.unwrap() << displacement
    } else {
        0u64
    };

    if let Some(proof) = opt.proof {
        if let Some((task_id, nonce, challenge,solution, proof, target)) = parse_parameters(&proof) {
            let coinbase_puzzle = CoinbasePuzzle::<CurrentNetwork>::load().unwrap();
            println!(
                "\tnonce_ex:{:#x} \n\taddress:{} \n\ttask_id: {} \n\tnonce: {:#x} \n\tsolution: {} \n\tproof: {}, \n\ttarget: {}",
                nonce_ex, address, task_id, nonce,  solution, proof, target
            );

            let commitment = PuzzleCommitment::<CurrentNetwork>::from_str(&solution).unwrap();
            let proof = PuzzleProof::<CurrentNetwork>::from_bytes_le(
                hex::decode(&proof).unwrap().as_slice(),
            )
            .unwrap();
            let address = Address::<CurrentNetwork>::from_str(&address).unwrap();
            let prover_solution = ProverSolution::<CurrentNetwork>::new(
                PartialSolution::<CurrentNetwork>::new(address, nonce, commitment),
                proof,
            );
            let challenge = EpochChallenge::<CurrentNetwork>::from_bytes_le(
                hex::decode(&challenge).unwrap().as_slice(),
            )
            .unwrap();

            if (nonce_ex & nonce) != nonce_ex {
                println!("Verify nonce failed");
                return;
            }

            if let Ok(verify) = prover_solution.verify(
                coinbase_puzzle.coinbase_verifying_key(),
                &challenge,
                target,
            ) {
                println!("Verify solution {}", verify);
            } else {
                println!("Verify solution failed");
            }
        }
    }
}

fn parse_parameters(input: &str) -> Option<(u64, u64, String, String, String, u64)> {
    let mut task_id: Option<u64> = None;
    let mut nonce: Option<u64> = None;
    let mut challenge: Option<String> = None;
    let mut solution: Option<String> = None;
    let mut proof: Option<String> = None;
    let mut target: Option<u64> = None;

    for pair in input.split(',') {
        let parts: Vec<&str> = pair.split(':').collect();
        if parts.len() != 2 {
            continue;
        }

        match parts[0].trim() {
            "task_id" => {
                if let Ok(value) = parts[1].trim().parse() {
                    task_id = Some(value);
                }
            }
            "nonce" => {
                if let Ok(value) = parts[1].trim().parse() {
                    nonce = Some(value);
                }
            }
            "challenge" => {
                challenge = Some(parts[1].trim().to_string());
            }
            "solution" => {
                solution = Some(parts[1].trim().to_string());
            }
            "proof" => {
                proof = Some(parts[1].trim().to_string());
            }
            "target" => {
                if let Ok(value) = parts[1].trim().parse() {
                    target = Some(value);
                }
            }
            _ => {}
        }
    }

    if let (Some(task_id), Some(nonce), Some(challenge), Some(solution), Some(proof), Some(target)) =
        (task_id, nonce, challenge, solution, proof, target)
    {
        Some((task_id, nonce, challenge, solution, proof, target))
    } else {
        None
    }
}
