#!/bin/bash
set -e

NETWORK="${1:-testnet}"
echo "=== Loki-app Contract Deployment — $NETWORK ==="

# Load config
if [ "$NETWORK" = "testnet" ]; then
    RPC_URL="https://soroban-testnet.stellar.org"
    NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
    ADMIN_PUBLIC="${TESTNET_ADMIN_PUBLIC_KEY}"
    ADMIN_SECRET="${TESTNET_ADMIN_SECRET_KEY}"
    if [ -z "$ADMIN_PUBLIC" ] || [ -z "$ADMIN_SECRET" ]; then
        echo ""
        echo "ERROR: Testnet admin keys not configured."
        echo ""
        echo "  Generate a testnet keypair:"
        echo "    stellar keys generate loki-app-testnet --network testnet"
        echo "    stellar keys show loki-app-testnet"
        echo ""
        echo "  Then export the keys:"
        echo "    export TESTNET_ADMIN_PUBLIC_KEY=\"<public-key>\""
        echo "    export TESTNET_ADMIN_SECRET_KEY=\"<secret-key>\""
        echo ""
        echo "  Or use a .env file:"
        echo '    echo "TESTNET_ADMIN_PUBLIC_KEY=<public-key>" > .env'
        echo '    echo "TESTNET_ADMIN_SECRET_KEY=<secret-key>" >> .env'
        echo "    source .env"
        echo ""
        exit 1
    fi
else
    RPC_URL="https://soroban.stellar.org"
    NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
    ADMIN_PUBLIC="${MAINNET_ADMIN_PUBLIC_KEY}"
    ADMIN_SECRET="${MAINNET_ADMIN_SECRET_KEY}"
    if [ -z "$ADMIN_PUBLIC" ] || [ -z "$ADMIN_SECRET" ]; then
        echo "ERROR: Set MAINNET_ADMIN_PUBLIC_KEY and MAINNET_ADMIN_SECRET_KEY environment variables"
        exit 1
    fi
fi

IDENTITY="loki-app-deployer"
echo "Configuring identity: $IDENTITY"
soroban config identity generate "$IDENTITY" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    --secret-key "$ADMIN_SECRET" 2>/dev/null || \
soroban config identity address "$IDENTITY" 2>/dev/null

WASM_DIR="target/wasm32v1-none/release"

# Deploy contracts in dependency order
echo "1/4 Deploying Circle Factory..."
FACTORY_ID=$(soroban contract deploy \
    --wasm "$WASM_DIR/circle_factory.optimized.wasm" \
    --source "$IDENTITY" \
    --network "$NETWORK")
echo "   Factory: $FACTORY_ID"

echo "2/4 Deploying Circle (base template)..."
CIRCLE_WASM_HASH=$(soroban contract install \
    --wasm "$WASM_DIR/circle.optimized.wasm" \
    --source "$IDENTITY" \
    --network "$NETWORK")
echo "   Circle WASM Hash: $CIRCLE_WASM_HASH"

echo "3/4 Deploying Reputation Registry..."
REP_ID=$(soroban contract deploy \
    --wasm "$WASM_DIR/reputation_registry.optimized.wasm" \
    --source "$IDENTITY" \
    --network "$NETWORK")
echo "   Reputation: $REP_ID"

echo "4/4 Deploying Treasury..."
TREASURY_ID=$(soroban contract deploy \
    --wasm "$WASM_DIR/treasury.optimized.wasm" \
    --source "$IDENTITY" \
    --network "$NETWORK")
echo "   Treasury: $TREASURY_ID"

echo ""
echo "Initializing deployed contracts..."
soroban contract invoke --id "$TREASURY_ID" --source "$IDENTITY" --network "$NETWORK" -- init --admin "$ADMIN_PUBLIC" 2>/dev/null || echo "   Treasury already initialized or skipped"
soroban contract invoke --id "$REP_ID" --source "$IDENTITY" --network "$NETWORK" -- init --admin "$ADMIN_PUBLIC" 2>/dev/null || echo "   Reputation Registry already initialized or skipped"
soroban contract invoke --id "$FACTORY_ID" --source "$IDENTITY" --network "$NETWORK" -- init_with_config --admin "$ADMIN_PUBLIC" --fee_bps 50 --treasury "$TREASURY_ID" --reputation_registry "$REP_ID" --circle_wasm_hash "$CIRCLE_WASM_HASH" 2>/dev/null || echo "   Circle Factory already initialized or skipped"

echo ""
echo "=== Deployment Complete ==="
echo "Network: $NETWORK"
echo "RPC: $RPC_URL"
echo ""
echo "Contract IDs:"
echo "  Circle Factory:     $FACTORY_ID"
echo "  Circle WASM Hash:   $CIRCLE_WASM_HASH"
echo "  Reputation Registry:$REP_ID"
echo "  Treasury:           $TREASURY_ID"
echo ""
echo "Admin Public Key: $ADMIN_PUBLIC"
echo ""
echo "Save these IDs to config/config.yaml in the backend."
