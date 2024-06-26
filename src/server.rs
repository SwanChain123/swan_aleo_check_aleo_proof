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
use serde_json::Value as JsonValue;

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
                        "input_param" => {
                            let input_param = String::from_str(value.as_str().unwrap()).ok();
                            if let Some(input_param) = input_param {
                                let parsed: Value = serde_json::from_str(&input_param).unwrap();
                                nonce_ex = parsed.get("nonce_ex").and_then(JsonValue::as_u64);
                                nonce_len = parsed.get("nonce_len").and_then(JsonValue::as_u64);
                                target = parsed.get("min_proof_target").and_then(JsonValue::as_u64);
                                address = parsed.get("address").and_then(JsonValue::as_str).map(|s| s.to_string());
                            }
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
                    println!("\n***********************************************************************************************************************");
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
                        println!("Response check_aleo_proof:\t\tfalse  \n\tnonce_ex!= nonce");
                        println!("***********************************************************************************************************************\n");
                        Ok(Value::Bool(false))
                    } else {
                        if mix_target > target {
                            println!("Response check_aleo_proof:\t\tfalse  \n\ttarget < mix_target");
                            println!("***********************************************************************************************************************\n");
                            return Ok(Value::Bool(false))
                        }
                        if let Ok(verify) = prover_solution.verify(
                            coinbase_puzzle.coinbase_verifying_key(),
                            &challenge,
                            target,
                        ) {
                            println!("Response check_aleo_proof:\t\t{verify}  \n\tVerify proof {}", verify);
                            println!("***********************************************************************************************************************\n");
                            Ok(Value::Bool(verify))
                        } else {
                            println!("Response check_aleo_proof:\t\tfalse  \n\tVerify proof solution failed");
                            println!("***********************************************************************************************************************\n");
                            Ok(Value::Bool(false))
                        }
                    }
    
                } else {
                    println!("Response check_aleo_proof:\t\tfalse  \n\tVerify proof parse_parameters failed");
                    println!("***********************************************************************************************************************\n");
                    Ok(Value::Bool(false))
                }
            } else {
                println!("Response check_aleo_proof:\t\tfalse  \n\tVerify proof missing parameters failed");
                println!("\n***********************************************************************************************************************\n");
                Ok(Value::Bool(false))
            }
        }
        
    });

    let url = format!("0.0.0.0:{}", port);
    
    let server = ServerBuilder::new(io)
        .threads(3)
        .start_http(&url.parse().unwrap())
        .unwrap();

    println!("Server started on: {}", url);

    server.wait();
}

pub fn check_aleo_proof(data: &str, coinbase_puzzle: CoinbasePuzzle<CurrentNetwork>) -> Result<JsonValue, ()> {
    let parsed: Value = serde_json::from_str(data).unwrap();
    let input_param = parsed.get("input_param").and_then(JsonValue::as_str).map(|s| s.to_string());
    let input_proof = parsed.get("proof").and_then(JsonValue::as_str).map(|s| s.to_string());

    if let (Some(input_param), Some(input_proof)) = (input_param, input_proof) {
        let input_param: Value = serde_json::from_str(&input_param).unwrap();
        let nonce_ex = input_param.get("nonce_ex").and_then(JsonValue::as_u64); 
        let nonce_len = input_param.get("nonce_len").and_then(JsonValue::as_u64);
        let target = input_param.get("min_proof_target").and_then(JsonValue::as_u64);
        let address = input_param.get("address").and_then(JsonValue::as_str).map(|s| s.to_string());

        if let (Some(nonce_ex), Some(nonce_len), Some(target), Some(address)) = (nonce_ex, nonce_len, target, address) {
            let address = &address;
            let displacement = (8u64 - nonce_len) << 3;
            let nonce_ex = nonce_ex << displacement;
            let mix_target = target;

            if let Some((task_id, nonce, challenge,solution, proof, target)) = parse_parameters(&input_proof) {
                let commitment = PuzzleCommitment::<CurrentNetwork>::from_str(&solution).unwrap();
                let proof = PuzzleProof::<CurrentNetwork>::from_bytes_le(
                    hex::decode(&proof).unwrap().as_slice(),
                ).unwrap();

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
                    println!("false");
                    Ok(Value::Bool(false))
                } else {
                    if mix_target > target {
                        println!("false");
                        return Ok(Value::Bool(false))
                    }
                    if let Ok(verify) = prover_solution.verify(
                        coinbase_puzzle.coinbase_verifying_key(),
                        &challenge,
                        target,
                    ) {
                        println!("{verify}");
                        Ok(Value::Bool(verify))
                    } else {
                        println!("false");
                        Ok(Value::Bool(false))
                    }
                }
            } else {
                println!("false");
                Ok(Value::Bool(false))
            }
        } else {
            println!("false");
            Ok(Value::Bool(false))
        }
    } else {
        println!("false");
        Ok(Value::Bool(false))
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