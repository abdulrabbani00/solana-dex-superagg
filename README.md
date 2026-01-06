# solana-dex-superagg

A simple Rust library for accessing **Titan** and **Jupiter** DEX aggregators on Solana. Easily plug and play into your code to get the best swap prices across multiple aggregators.

## Features

- **Multi-aggregator support**: Access both Titan and Jupiter from a single interface
- **Smart routing**: Automatically compare aggregators and use the one with the best price
- **Flexible strategies**: Choose from multiple routing strategies including best price, preferred aggregator, or lowest slippage climber
- **Slippage tracking**: Know exactly what slippage was used for your swaps (especially useful for staircase strategy)

## Environment Variables

Set the following environment variables before using the library:

### Required

```bash
# Solana RPC endpoint
export DEX_SUPERAGG_SHARED__RPC_URL="https://api.mainnet-beta.solana.com"

# Wallet keypair (base58 string, JSON array, or comma-separated bytes)
export DEX_SUPERAGG_SHARED__WALLET_KEYPAIR="your_keypair_here"
```

### Jupiter Configuration

```bash
# Jupiter API URL (defaults to https://api.jup.ag/swap/v1 if not set)
export DEX_SUPERAGG_JUPITER__JUP_SWAP_API_URL="https://api.jup.ag/swap/v1"

# Jupiter API key (optional but recommended for production)
# Get your API key from https://portal.jup.ag/
export DEX_SUPERAGG_JUPITER__API_KEY="your_jupiter_api_key"
```

### Titan Configuration (Optional)

```bash
# Titan WebSocket endpoint
export DEX_SUPERAGG_TITAN__TITAN_WS_ENDPOINT="us1.api.demo.titan.exchange"

# Titan API key (required)
export DEX_SUPERAGG_TITAN__TITAN_API_KEY="your_titan_api_key"
```

### Optional Configuration

```bash
# Slippage tolerance in basis points (default: 50 = 0.5%)
export DEX_SUPERAGG_SHARED__SLIPPAGE_BPS="25"

# Compute unit price in micro lamports (default: 0)
export DEX_SUPERAGG_SHARED__COMPUTE_UNIT_PRICE_MICRO_LAMPORTS="5000"

# Default routing strategy (default: best_price)
# Options: best_price, preferred_aggregator, lowest_slippage_climber
export DEX_SUPERAGG_SHARED__ROUTING_STRATEGY__TYPE="best_price"
```

## Quick Start

### 1. Create the Client

```rust
use solana_dex_superagg::{client::DexSuperAggClient, config::ClientConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from environment variables
    let config = ClientConfig::from_env()?;
    
    // Validate configuration (optional but recommended)
    if let Err(errors) = config.validate().await {
        eprintln!("Configuration errors:");
        for error in errors {
            eprintln!("  - {}", error);
        }
        return Err(anyhow::anyhow!("Invalid configuration"));
    }
    
    // Create the client
    let client = DexSuperAggClient::new(config)?;
    
    // Use the client for swaps...
    
    // Clean up when done (closes Titan WebSocket connection if used)
    client.close().await?;
    
    Ok(())
}
```

### 2. Router Options

The library supports three routing strategies:

#### Best Price (Default)

Automatically compares all available aggregators and uses the one that gives you the most tokens:

```rust
use solana_dex_superagg::config::{RouteConfig, RoutingStrategy};

let route_config = RouteConfig {
    routing_strategy: Some(RoutingStrategy::BestPrice),
    ..Default::default()
};

let result = client
    .swap_with_route_config(
        "So11111111111111111111111111111111111111112", // SOL
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
        1_000_000_000, // 1 SOL in lamports
        route_config,
    )
    .await?;

println!("Swap successful! Signature: {}", result.signature);
println!("Received: {} lamports", result.out_amount);
```

#### Preferred Aggregator

Use a specific aggregator (Jupiter or Titan):

```rust
use solana_dex_superagg::config::{Aggregator, RouteConfig, RoutingStrategy};

// Use Jupiter
let route_config = RouteConfig {
    routing_strategy: Some(RoutingStrategy::PreferredAggregator {
        aggregator: Aggregator::Jupiter,
        simulate: false, // Set to true to simulate before executing
    }),
    ..Default::default()
};

let result = client
    .swap_with_route_config(input_token, output_token, amount, route_config)
    .await?;

// Use Titan
let route_config = RouteConfig {
    routing_strategy: Some(RoutingStrategy::PreferredAggregator {
        aggregator: Aggregator::Titan,
        simulate: false,
    }),
    ..Default::default()
};

let result = client
    .swap_with_route_config(input_token, output_token, amount, route_config)
    .await?;
```

#### Lowest Slippage Climber (Staircase Strategy)

Tests multiple slippage levels starting from a floor value and incrementing until a swap succeeds. This is useful for finding the minimum slippage needed for a swap:

```rust
use solana_dex_superagg::config::{RouteConfig, RoutingStrategy};

let route_config = RouteConfig {
    routing_strategy: Some(RoutingStrategy::LowestSlippageClimber {
        floor_slippage_bps: 10,  // Start at 0.1%
        max_slippage_bps: 100,   // Up to 1%
        step_bps: 10,            // Step by 0.1%
    }),
    ..Default::default()
};

let result = client
    .swap_with_route_config(input_token, output_token, amount, route_config)
    .await?;

println!("Swap successful!");
println!("Transaction: {}", result.signature);
println!("Output Amount: {} lamports", result.out_amount);
println!("Slippage Used: {} bps ({:.2}%)", 
    result.slippage_bps_used.unwrap_or(0),
    result.slippage_bps_used.unwrap_or(0) as f64 / 100.0
);
```

### 3. Triggering a Swap

#### Simple Swap (Uses Default Strategy)

```rust
// Uses the default routing strategy from config (usually BestPrice)
let result = client
    .swap(
        "So11111111111111111111111111111111111111112", // SOL
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
        1_000_000_000, // 1 SOL in lamports
    )
    .await?;

println!("Swap signature: {}", result.signature);
println!("Received: {} lamports", result.out_amount);
```

#### Swap with Custom Route Configuration

```rust
use solana_dex_superagg::config::{RouteConfig, RoutingStrategy, Aggregator};

let route_config = RouteConfig {
    routing_strategy: Some(RoutingStrategy::PreferredAggregator {
        aggregator: Aggregator::Jupiter,
        simulate: true, // Simulate before executing
    }),
    compute_unit_price_micro_lamports: 5000, // Custom compute unit price
    ..Default::default()
};

let result = client
    .swap_with_route_config(
        input_token,
        output_token,
        amount,
        route_config,
    )
    .await?;
```

## Complete Example

```rust
use solana_dex_superagg::{
    client::DexSuperAggClient,
    config::{ClientConfig, RouteConfig, RoutingStrategy, Aggregator},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config from environment
    let config = ClientConfig::from_env()?;
    
    // Validate config
    config.validate().await?;
    
    // Create client
    let client = DexSuperAggClient::new(config)?;
    
    // Token addresses
    let sol = "So11111111111111111111111111111111111111112";
    let usdc = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let amount = 1_000_000_000; // 1 SOL
    
    // Example 1: Best price (compares all aggregators)
    println!("=== Best Price Strategy ===");
    let route_config = RouteConfig {
        routing_strategy: Some(RoutingStrategy::BestPrice),
        ..Default::default()
    };
    let result = client
        .swap_with_route_config(sol, usdc, amount, route_config)
        .await?;
    println!("✓ Swap successful: {}", result.signature);
    println!("  Output: {} lamports", result.out_amount);
    
    // Example 2: Preferred aggregator (Jupiter)
    println!("\n=== Jupiter Swap ===");
    let route_config = RouteConfig {
        routing_strategy: Some(RoutingStrategy::PreferredAggregator {
            aggregator: Aggregator::Jupiter,
            simulate: false,
        }),
        ..Default::default()
    };
    let result = client
        .swap_with_route_config(sol, usdc, amount, route_config)
        .await?;
    println!("✓ Swap successful: {}", result.signature);
    println!("  Output: {} lamports", result.out_amount);
    
    // Example 3: Lowest slippage climber (staircase)
    println!("\n=== Lowest Slippage Climber ===");
    let route_config = RouteConfig {
        routing_strategy: Some(RoutingStrategy::LowestSlippageClimber {
            floor_slippage_bps: 10,  // Start at 0.1%
            max_slippage_bps: 100,   // Up to 1%
            step_bps: 10,            // Step by 0.1%
        }),
        ..Default::default()
    };
    let result = client
        .swap_with_route_config(sol, usdc, amount, route_config)
        .await?;
    println!("✓ Swap successful: {}", result.signature);
    println!("  Output: {} lamports", result.out_amount);
    if let Some(slippage) = result.slippage_bps_used {
        println!("  Slippage Used: {} bps ({:.2}%)", slippage, slippage as f64 / 100.0);
    }
    
    // Clean up
    client.close().await?;
    
    Ok(())
}
```

## SwapResult

The `SwapResult` struct contains:

```rust
pub struct SwapResult {
    /// Transaction signature of the executed swap
    pub signature: String,
    /// Output amount received (in lamports/base units)
    pub out_amount: u64,
    /// Slippage tolerance actually used in basis points
    /// This is particularly useful for LowestSlippageClimber strategy
    pub slippage_bps_used: Option<u16>,
}
```

The `slippage_bps_used` field shows what slippage was actually used for the swap. This is especially useful with the `LowestSlippageClimber` strategy to know which slippage level succeeded.

## Notes

- **Token amounts**: All amounts are in lamports/base units (not human-readable). For example, 1 SOL = 1,000,000,000 lamports, 1 USDC = 1,000,000 lamports.
- **Slippage**: Slippage is specified in basis points (bps). 1 bps = 0.01%, so 25 bps = 0.25%.
- **Titan WebSocket**: The Titan aggregator maintains a persistent WebSocket connection that is reused across swaps for efficiency. Make sure to call `client.close().await?` when you're done to clean up resources.
- **ATA Creation**: By default, the client will create Associated Token Accounts (ATAs) for output tokens if they don't exist. You can control this behavior via `RouteConfig::output_ata`.