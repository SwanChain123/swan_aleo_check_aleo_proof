use std::str::FromStr;

use snarkvm::{
    console::{network::Testnet3 as CurrentNetwork, types::Address},
    ledger::coinbase::{
        CoinbasePuzzle, EpochChallenge, PartialSolution, ProverSolution, PuzzleCommitment,
        PuzzleProof,
    },
    utilities::FromBytes,
};

use jsonrpc_http_server::jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::ServerBuilder;

pub fn start_rpc_server(port: u16, coinbase_puzzle: CoinbasePuzzle<CurrentNetwork>) {
    // let coinbasePuzzle_clone = coinbasePuzzle.clone();
    let mut io = IoHandler::default();
    // 注册了一个say_hello方法，忽略参数，直接返回hello
    io.add_method("check_aleo_proof", move |params: Params| {
        let mut address: Option<String> = None;
        let mut nonce_ex: Option<u64> = None;
        let mut nonce_len: Option<u64> = None;
        let mut target: Option<u64> = None;
        let mut proof: Option<String> = None;

        match params {
            Params::Map(p_map) => {
                for (key, value) in &p_map {
                    match key.as_str() {
                        "address" => {
                            address = String::from_str(value.as_str().unwrap()).ok();
                        }
                        "nonce_ex" => {
                            nonce_ex = value.as_u64();
                        }
                        "nonce_len" => {
                            nonce_len = value.as_u64();
                        }
                        "mix_target" => {
                            target = value.as_u64();
                        }
                        "proof" => {
                            proof = String::from_str(value.as_str().unwrap()).ok();
                        }
                        _ => {
                            
                        }
                    }
                }
            }
            _ => {}
        };

        let coinbase_puzzle = coinbase_puzzle.clone(); 
        async move {
            if address.is_some() && nonce_ex.is_some() && nonce_len.is_some() && proof.is_some() && target.is_some() {
                let address = &address.unwrap();
                let displacement = (8u64 - nonce_len.unwrap()) << 3;
                let nonce_ex = nonce_ex.unwrap() << displacement;
                let mix_target = target.unwrap();
        
                if let Some((task_id, nonce, challenge,solution, proof, target)) = parse_parameters(&proof.unwrap()) {
                    println!(
                        "Request check_aleo_proof: \n\tnonce_ex:{:#x} \n\taddress:{} \n\tmix_target:{} \n\ttask_id: {} \n\tnonce: {:#x} \n\tsolution: {} \n\tproof: {}, \n\ttarget: {}",
                        nonce_ex, address, mix_target, task_id, nonce,  solution, proof, target
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
                        Ok(Value::Bool(false))
                    } else {
                        if mix_target > target {
                            println!("Response check_aleo_proof:false  \n\ttarget < mix_target");
                            return Ok(Value::Bool(false))
                        }
                        if let Ok(verify) = prover_solution.verify(
                            coinbase_puzzle.coinbase_verifying_key(),
                            &challenge,
                            target,
                        ) {
                            println!("Response check_aleo_proof:{verify}  \n\tVerify proof {}", verify);
                            Ok(Value::Bool(verify))
                        } else {
                            println!("Response check_aleo_proof:false  \n\tVerify proof solution failed");
                            Ok(Value::Bool(false))
                        }
                    }
    
                } else {
                    println!("Response check_aleo_proof:false  \n\tVerify proof parse_parameters failed");
                    Ok(Value::Bool(false))
                }
            } else {
                println!("Response check_aleo_proof:false  \n\tVerify proof missing parameters failed");
                Ok(Value::Bool(false))
            }
        }
        
    });

    let url = format!("127.0.0.1:{}", port);
    
    let server = ServerBuilder::new(io)
        .threads(3)
        .start_http(&url.parse().unwrap())
        .unwrap();

    server.wait();
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