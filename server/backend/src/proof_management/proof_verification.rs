use anyhow::{anyhow, Error, Ok, Result};
use reqwest::Client;
use std::{env, thread, time::Duration};
use zunnogame_script::ProofOutput;

pub async fn verify_proof(proof_result: ProofOutput) -> Result<String, anyhow::Error> {
    /// Setting up the zkVerify Relayer API
    let zkv_api_base_url: String = env::var("ZKV_API_BASE_URL").unwrap();
    let relayer_api_key: String = env::var("RELAYER_API_KEY").unwrap();

    /// proof submission payload for ZKV
    let zkv_proof_submission = serde_json::json!({
        "proofType": "sp1",
        "vkRegistered": "false",
        "proofData": {
            "proof": proof_result.proof,
            "publicSignals": proof_result.pub_inputs,
            "vk": proof_result.image_id
        }
    });

    let client = Client::new();

    tracing::info!("Initiating submission to ZKV.");

    let client_response = client
        .post(format!(
            "{}/submit-proof/{}",
            zkv_api_base_url, relayer_api_key
        ))
        .json(&zkv_proof_submission)
        .send()
        .await?;

    let submission_response: serde_json::Value = client_response.json().await?;

    if submission_response["optimisticVerify"] != "success" {
        return Err(anyhow!("Proof submission for Verification failed !"));
    }

    let job_id = submission_response["jobId"].as_str().unwrap();
    tracing::info!(
        joib_id = job_id.to_string(),
        "Fetched proof submission job id."
    );

    loop {
        let job_status = client
            .get(format!(
                "{}/job-status/{}/{}",
                zkv_api_base_url, relayer_api_key, job_id
            ))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let status = job_status["status"].as_str().unwrap_or("Unknown");

        if status == "Finalized" || status == "Aggregated" || status == "AggregationPending" {
            println!("Job Finalized successfully");
            println!("{:?}", job_status);
            return Ok(job_status["txHash"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string());
        } else {
            println!("Job status: {}", status);
            println!("Waiting for job to finalized...");
            thread::sleep(Duration::from_secs(5));
        }
    }
}
