use anyhow::{anyhow, Result};
use solana_dex_superagg::{Aggregator, ClientConfig, DexSuperAggClient};

/// Default test pair (same as examples/tests)
const SOL_TOKEN: &str = "So11111111111111111111111111111111111111112";
const USDC_TOKEN: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env if present (useful for local runs)
    let _ = dotenvy::dotenv();

    // Logging: default to info unless RUST_LOG is set
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    // Load config (same env vars as the library + examples)
    let config = ClientConfig::from_env().map_err(|e| anyhow!("Failed to load config: {e}"))?;

    // NOTE: we intentionally do NOT call config.validate() here.
    // Validation can hard-fail if *one* aggregator endpoint is flaky, even though others work.
    // The client will still gracefully skip simulators/executors that fail.

    let client = DexSuperAggClient::new(config)?;

    // Use a tiny amount for a smoke-test swap (100k lamports ~= 0.0001 SOL)
    let amount: u64 = std::env::var("BENCH_AMOUNT_LAMPORTS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(100_000);

    let input = std::env::var("BENCH_INPUT_MINT").unwrap_or_else(|_| SOL_TOKEN.to_string());
    let output = std::env::var("BENCH_OUTPUT_MINT").unwrap_or_else(|_| USDC_TOKEN.to_string());

    tracing::info!(
        input = input,
        output = output,
        amount = amount,
        "Starting bench swap"
    );

    let summary = client.swap(&input, &output, amount).await?;

    tracing::info!(
        signature = %summary.swap_result.signature,
        out_amount = summary.swap_result.out_amount,
        aggregator_used = ?summary.swap_result.aggregator_used,
        execution_time_ms = summary
            .swap_result
            .execution_time
            .map(|d| d.as_secs_f64() * 1000.0),
        "Swap complete"
    );

    // Show all simulation results that were available
    for (agg, sim) in &summary.sim_results {
        let agg_name = match agg {
            Aggregator::Jupiter => "Jupiter",
            Aggregator::Titan => "Titan",
            Aggregator::Dflow => "DFlow",
        };
        tracing::info!(
            aggregator = agg_name,
            out_amount = sim.out_amount,
            sim_time_ms = sim.sim_time.map(|d| d.as_secs_f64() * 1000.0),
            "Quote result"
        );
    }

    // Clean up (closes Titan WS if it was initialized)
    client.close().await?;

    Ok(())
}
