#!/usr/bin/env bash
# deploy-upgrade.sh — Loki-app contract deployment and upgrade helper
#
# Usage:
#   ./scripts/deploy-upgrade.sh [OPTIONS]
#
# Options:
#   --manifest <path>      Path to deployment manifest JSON (default: scripts/deploy-manifest.json)
#   --network <network>    Override network from manifest (testnet|mainnet)
#   --upgrade-only         Skip deploy; upgrade existing contracts from latest deployment log
#   --dry-run              Print commands without executing them
#   --help                 Show this help message

set -euo pipefail

# ─── Colours ────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

log_info()    { echo -e "${CYAN}[INFO]${RESET}  $*"; }
log_ok()      { echo -e "${GREEN}[OK]${RESET}    $*"; }
log_warn()    { echo -e "${YELLOW}[WARN]${RESET}  $*"; }
log_error()   { echo -e "${RED}[ERROR]${RESET} $*" >&2; }
log_section() { echo -e "\n${BOLD}=== $* ===${RESET}"; }

# ─── Defaults ───────────────────────────────────────────────────────────────
MANIFEST="scripts/deploy-manifest.json"
NETWORK_OVERRIDE=""
UPGRADE_ONLY=false
DRY_RUN=false
DEPLOYMENTS_DIR="deployments"

# ─── Argument parsing ────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --manifest)   MANIFEST="$2"; shift 2 ;;
    --network)    NETWORK_OVERRIDE="$2"; shift 2 ;;
    --upgrade-only) UPGRADE_ONLY=true; shift ;;
    --dry-run)    DRY_RUN=true; shift ;;
    --help)
      sed -n '/^# Usage:/,/^$/p' "$0"
      exit 0
      ;;
    *) log_error "Unknown argument: $1"; exit 1 ;;
  esac
done

# ─── Dependency checks ───────────────────────────────────────────────────────
for dep in jq stellar; do
  if ! command -v "$dep" &>/dev/null; then
    log_error "Required tool not found: $dep"
    [[ "$dep" == "jq" ]]     && log_error "  Install jq: https://stedolan.github.io/jq/download/"
    [[ "$dep" == "stellar" ]] && log_error "  Install Stellar CLI: https://developers.stellar.org/docs/tools/developer-tools/stellar-cli"
    exit 1
  fi
done

# ─── Load manifest ───────────────────────────────────────────────────────────
if [[ ! -f "$MANIFEST" ]]; then
  log_error "Manifest not found: $MANIFEST"
  exit 1
fi

log_info "Loading manifest: $MANIFEST"
NETWORK=$(jq -r '.network' "$MANIFEST")
ADMIN_IDENTITY=$(jq -r '.admin_identity' "$MANIFEST")

# Override network if provided
[[ -n "$NETWORK_OVERRIDE" ]] && NETWORK="$NETWORK_OVERRIDE"

# ─── Network config ──────────────────────────────────────────────────────────
case "$NETWORK" in
  testnet)
    RPC_URL="https://soroban-testnet.stellar.org"
    NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
    ADMIN_PUBLIC="${TESTNET_ADMIN_PUBLIC_KEY:-}"
    ADMIN_SECRET="${TESTNET_ADMIN_SECRET_KEY:-}"
    ;;
  mainnet)
    RPC_URL="https://soroban.stellar.org"
    NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
    ADMIN_PUBLIC="${MAINNET_ADMIN_PUBLIC_KEY:-}"
    ADMIN_SECRET="${MAINNET_ADMIN_SECRET_KEY:-}"
    ;;
  *)
    log_error "Unsupported network: $NETWORK (must be testnet or mainnet)"
    exit 1
    ;;
esac

# Validate required env vars
if [[ -z "$ADMIN_PUBLIC" ]]; then
  log_error "Admin public key not set."
  log_error "  Export: $(echo "$NETWORK" | tr a-z A-Z)_ADMIN_PUBLIC_KEY"
  exit 1
fi
if [[ -z "$ADMIN_SECRET" ]]; then
  log_error "Admin secret key not set."
  log_error "  Export: $(echo "$NETWORK" | tr a-z A-Z)_ADMIN_SECRET_KEY"
  exit 1
fi

log_section "Loki-app Deployment — $NETWORK"
log_info "Admin public key : $ADMIN_PUBLIC"
log_info "Admin identity   : $ADMIN_IDENTITY"
log_info "RPC URL          : $RPC_URL"
[[ "$DRY_RUN" == true ]]     && log_warn "DRY-RUN mode — no commands will be executed"
[[ "$UPGRADE_ONLY" == true ]] && log_warn "UPGRADE-ONLY mode — skipping fresh deployment"

# ─── Helpers ─────────────────────────────────────────────────────────────────
run() {
  # Wrap every stellar CLI call so dry-run works transparently
  if [[ "$DRY_RUN" == true ]]; then
    echo -e "${YELLOW}[DRY-RUN]${RESET} $*"
    echo "DRY_RUN_PLACEHOLDER"
  else
    "$@"
  fi
}

# Ensure identity is configured
configure_identity() {
  log_info "Configuring identity: $ADMIN_IDENTITY"
  if [[ "$DRY_RUN" == false ]]; then
    stellar keys generate "$ADMIN_IDENTITY" \
      --rpc-url "$RPC_URL" \
      --network-passphrase "$NETWORK_PASSPHRASE" \
      --secret-key "$ADMIN_SECRET" 2>/dev/null \
    || stellar keys address "$ADMIN_IDENTITY" &>/dev/null \
    || { log_error "Failed to configure identity $ADMIN_IDENTITY"; exit 1; }
  else
    log_warn "[DRY-RUN] Would configure identity $ADMIN_IDENTITY"
  fi
}

# Substitute template variables in init_args
substitute_vars() {
  local args="$1"
  args="${args//\{ADMIN_PUBLIC\}/$ADMIN_PUBLIC}"
  args="${args//\{CIRCLE_WASM_HASH\}/${CIRCLE_WASM_HASH:-}}"
  args="${args//\{TREASURY_ID\}/${CONTRACT_IDS[treasury]:-}}"
  args="${args//\{REP_ID\}/${CONTRACT_IDS[reputation_registry]:-}}"
  echo "$args"
}

# ─── Fresh deployment ────────────────────────────────────────────────────────
deploy_contracts() {
  log_section "Deploying Contracts"

  mkdir -p "$DEPLOYMENTS_DIR"
  TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  LOG_FILE="$DEPLOYMENTS_DIR/${NETWORK}-${TIMESTAMP//[: ]/-}.json"

  # Associative map: contract name → deployed ID or wasm hash
  declare -A CONTRACT_IDS
  CIRCLE_WASM_HASH=""

  local num_contracts
  num_contracts=$(jq '.contracts | length' "$MANIFEST")

  for (( i=0; i<num_contracts; i++ )); do
    local name wasm install_only init_args
    name=$(jq -r ".contracts[$i].name" "$MANIFEST")
    wasm=$(jq -r ".contracts[$i].wasm" "$MANIFEST")
    install_only=$(jq -r ".contracts[$i].install_only" "$MANIFEST")
    init_args=$(jq -r ".contracts[$i].init_args" "$MANIFEST")

    log_info "[$((i+1))/$num_contracts] Processing: $name"

    # Validate WASM file exists (skip in dry-run)
    if [[ "$DRY_RUN" == false && ! -f "$wasm" ]]; then
      log_error "WASM file not found: $wasm"
      log_error "  Build contracts first: cargo build --target wasm32v1-none --release && make optimize"
      exit 1
    fi

    if [[ "$install_only" == "true" ]]; then
      # Install WASM and capture hash
      log_info "  Installing WASM: $wasm"
      local hash
      hash=$(run stellar contract install \
        --wasm "$wasm" \
        --source "$ADMIN_IDENTITY" \
        --network "$NETWORK")
      CONTRACT_IDS["${name}_wasm_hash"]="$hash"
      # Export for use in dependent contracts
      if [[ "$name" == "circle" ]]; then
        CIRCLE_WASM_HASH="$hash"
      fi
      log_ok "  Installed $name WASM hash: $hash"
    else
      # Deploy contract and capture ID
      log_info "  Deploying: $wasm"
      local contract_id
      contract_id=$(run stellar contract deploy \
        --wasm "$wasm" \
        --source "$ADMIN_IDENTITY" \
        --network "$NETWORK")
      CONTRACT_IDS["$name"]="$contract_id"
      log_ok "  Deployed $name: $contract_id"

      # Initialise contract if init_args provided
      if [[ -n "$init_args" ]]; then
        local resolved_args
        resolved_args=$(substitute_vars "$init_args")
        log_info "  Initialising $name with args: $resolved_args"
        # shellcheck disable=SC2086
        run stellar contract invoke \
          --id "$contract_id" \
          --source "$ADMIN_IDENTITY" \
          --network "$NETWORK" \
          -- init $resolved_args \
        && log_ok "  Initialised $name" \
        || log_warn "  $name init skipped (may already be initialised)"
      fi
    fi
  done

  # ─── Write deployment log ─────────────────────────────────────────────────
  log_section "Writing Deployment Log"
  {
    echo "{"
    echo "  \"network\": \"$NETWORK\","
    echo "  \"timestamp\": \"$TIMESTAMP\","
    echo "  \"admin\": \"$ADMIN_PUBLIC\","
    echo "  \"rpc_url\": \"$RPC_URL\","
    echo "  \"contracts\": {"
    local first=true
    for key in "${!CONTRACT_IDS[@]}"; do
      [[ "$first" == false ]] && echo ","
      echo -n "    \"$key\": \"${CONTRACT_IDS[$key]}\""
      first=false
    done
    echo ""
    echo "  }"
    echo "}"
  } > "$LOG_FILE"

  log_ok "Deployment log written: $LOG_FILE"

  # ─── Summary ──────────────────────────────────────────────────────────────
  log_section "Deployment Summary"
  echo -e "${BOLD}Network:${RESET}  $NETWORK"
  echo -e "${BOLD}Admin:${RESET}    $ADMIN_PUBLIC"
  echo ""
  echo -e "${BOLD}Contract IDs:${RESET}"
  for key in "${!CONTRACT_IDS[@]}"; do
    printf "  %-30s %s\n" "$key" "${CONTRACT_IDS[$key]}"
  done
  echo ""
  echo -e "${GREEN}Save these IDs to config/config.yaml in the backend.${RESET}"
}

# ─── Upgrade-only mode ────────────────────────────────────────────────────────
upgrade_contracts() {
  log_section "Upgrading Contracts"

  # Find the latest deployment log
  local latest_log
  latest_log=$(find "$DEPLOYMENTS_DIR" -name "${NETWORK}-*.json" 2>/dev/null | sort | tail -1)
  if [[ -z "$latest_log" ]]; then
    log_error "No previous deployment log found in $DEPLOYMENTS_DIR/ for network: $NETWORK"
    log_error "  Run a fresh deployment first (without --upgrade-only)"
    exit 1
  fi

  log_info "Using deployment log: $latest_log"

  local num_contracts
  num_contracts=$(jq '.contracts | length' "$MANIFEST")

  for (( i=0; i<num_contracts; i++ )); do
    local name wasm install_only
    name=$(jq -r ".contracts[$i].name" "$MANIFEST")
    wasm=$(jq -r ".contracts[$i].wasm" "$MANIFEST")
    install_only=$(jq -r ".contracts[$i].install_only" "$MANIFEST")

    log_info "Upgrading: $name"

    # For install_only contracts (e.g. circle), install new WASM and update factory
    if [[ "$install_only" == "true" ]]; then
      log_info "  Installing new WASM for $name"
      local new_hash
      new_hash=$(run stellar contract install \
        --wasm "$wasm" \
        --source "$ADMIN_IDENTITY" \
        --network "$NETWORK")
      log_ok "  New $name WASM hash: $new_hash"
      log_warn "  Remember to update circle_factory with new wasm hash: $new_hash"
    else
      # Look up existing contract ID from log
      local contract_id
      contract_id=$(jq -r ".contracts.\"$name\" // empty" "$latest_log")
      if [[ -z "$contract_id" ]]; then
        log_warn "  $name not found in deployment log, skipping"
        continue
      fi

      # Install new WASM
      log_info "  Installing new WASM: $wasm"
      local new_hash
      new_hash=$(run stellar contract install \
        --wasm "$wasm" \
        --source "$ADMIN_IDENTITY" \
        --network "$NETWORK")
      log_ok "  New WASM hash: $new_hash"

      # Invoke upgrade
      log_info "  Invoking upgrade on $contract_id"
      run stellar contract invoke \
        --id "$contract_id" \
        --source "$ADMIN_IDENTITY" \
        --network "$NETWORK" \
        -- upgrade --new_wasm_hash "$new_hash" \
      && log_ok "  Upgraded $name ($contract_id)" \
      || log_warn "  Upgrade invocation failed for $name — check contract supports upgrade"
    fi
  done

  log_section "Upgrade Complete"
  log_ok "All applicable contracts upgraded on $NETWORK"
}

# ─── Main ────────────────────────────────────────────────────────────────────
configure_identity

if [[ "$UPGRADE_ONLY" == true ]]; then
  upgrade_contracts
else
  deploy_contracts
fi
