#!/usr/bin/env bash

set -euo pipefail
shopt -s inherit_errexit

ADMIN="${1:-juno1zk4c4aamef42cgjexlmksypac8j5xw7n3s4wrd}"
KOMPLE_MINT_ADDR="${KOMPLE_MINT_ADDR:-juno17rth4jstxs7cmrusvyluwlnt34l80cxaz7nufpjfntts00pk79asjxelgs}"

JUNOFARMS_PATH='../junofarms'

function update_contract_address {
  local new_address="${1}"

  sed -i "s/\(VITE_CONTRACT_ADDRESS=\).*/\1${new_address}/g" "${JUNOFARMS_PATH}/package/ui/.env"
}

function compile {
  docker run --rm -v "$(pwd)":/code:Z \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    cosmwasm/rust-optimizer:0.12.11
}

function upload_code {
  local response
  response="$(junod --chain-id uni-6 --node https://juno-testnet-rpc.polkachu.com:443 tx wasm store artifacts/juno_farm_hackathon_template.wasm --from "${ADMIN}" --gas-prices 0.075ujunox --gas auto --gas-adjustment 1.1 -o json -y)"

  local code
  code="$(echo "${response}" | jq -r '.code')"
  if [[ "${code}" -ne 0 ]]; then
      echo "[ERROR] Uploading code failed:" >&2
      echo "${response}" >&2
      return 1
  fi
  
  echo "[DEBUG] Code uploaded: ${response}" >&2

  local tx_hash
  tx_hash="$(echo "${response}" | jq -r '.txhash')"

  sleep 10
  junod --chain-id uni-6 --node https://juno-testnet-rpc.polkachu.com:443 query tx "${tx_hash}" -o json | jq '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value' -r
}

function upload_noise_code {
  local wasm="${1}"

  local response
  response="$(junod tx wasm store \
    "${wasm}" \
    --from "${ADMIN}" \
    --chain-id uni-6 \
    --gas=auto \
    --gas-adjustment 1.4  \
    --gas-prices 0.025ujunox \
    --broadcast-mode=async \
    --node=https://juno-testnet-rpc.polkachu.com:443 -o json -y)"

  local code
  code="$(echo "${response}" | jq -r '.code')"
  if [[ "${code}" -ne 0 ]]; then
      echo "[ERROR] Uploading noise code failed:" >&2
      echo "${response}" >&2
      return 1
  fi
  
  echo "[DEBUG] Noise code uploaded: ${response}" >&2

  local tx_hash
  tx_hash="$(echo "${response}" | jq -r '.txhash')"

  sleep 10
  junod --chain-id uni-6 --node https://juno-testnet-rpc.polkachu.com:443 query tx "${tx_hash}" -o json | jq '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value' -r
}

function instantiate {
  local code_id="${1}"

  local instantiate_msg
  instantiate_msg=$(cat <<-END
      {
        "admin": "%s",
        "komple_mint_addr": "%s"
      }
END
  )

  local msg
  msg="$(printf "${instantiate_msg}" "$ADMIN" "${KOMPLE_MINT_ADDR}")"

  local tx_hash
  tx_hash=$(junod --chain-id uni-6 --node https://juno-testnet-rpc.polkachu.com:443 tx wasm instantiate "${code_id}" "${msg}" --from "${ADMIN}" --admin "${ADMIN}" --gas-prices 0.075ujunox --gas auto --gas-adjustment 1.1 --label " " -o json -y | jq '.txhash' -r)
  sleep 10

  junod --chain-id uni-6 --node https://juno-testnet-rpc.polkachu.com:443 query tx "${tx_hash}" -o json | jq 'last(.logs[0].events[] | .attributes[] | select(.key=="_contract_address") | .value)' -r
}

function migrate {
    local code_id="${1}"
    local contract_addr="${2}"

    tx_hash=$(junod --chain-id uni-6 --node https://juno-testnet-rpc.polkachu.com:443 --from "${ADMIN}" tx wasm migrate "${contract_addr}" "${code_id}" '{}' --gas-prices 0.075ujunox --gas auto --gas-adjustment 1.2 -o json -y | jq '.txhash' -r )
    echo "Migration TX hash: ${tx_hash}"
}

function instantiate_nois {
  local code_id="${1}"

  local msg
  msg=$(cat <<EOF
{
  "manager":"${ADMIN}",
  "prices": [
    {"denom":"ujunox","amount":"1000000"},
    {"denom":"ujunox","amount":"50000000"}
  ],
  "callback_gas_limit":500000,
  "test_mode":false,
  "mode": {
    "ibc_pay":{
      "unois_denom":{
        "ics20_channel":"channel-xx",
        "denom":"ujunox"
      }
    }
  }
}
EOF
)

  local response
  response="$(junod tx wasm instantiate "${code_id}" "${msg}" \
    --label=nois-proxy \
    --from "${ADMIN}" \
    --admin "${ADMIN}" \
    --chain-id uni-6 \
    --gas=auto \
    --gas-adjustment 1.4 \
    --gas-prices 0.025ujunox \
    --broadcast-mode=sync \
    --node=https://juno-testnet-rpc.polkachu.com:443 -o json -y)"

  local code
  code="$(echo "${response}" | jq -r '.code')"
  if [[ "${code}" -ne 0 ]]; then
      echo "[ERROR] Instantiating noise failed:" >&2
      echo "${response}" >&2
      return 1
  fi
  
  echo "[DEBUG] Noise code instantiated: ${response}" >&2

  local tx_hash
  tx_hash="$(echo "${response}" | jq -r '.txhash')"

  sleep 10
  junod --chain-id uni-6 --node https://juno-testnet-rpc.polkachu.com:443 query tx "${tx_hash}" -o json | jq | jq '.logs[0].events[]|select(.type=="wasm").attributes[]|select(.key=="_contract_address").value' -r
}

function deploy_new {
  compile
  local code_id
  code_id=$(upload_code)
  echo "CODE_ID: ${code_id}"
  local contract_addr
  contract_addr="$(instantiate "${code_id}")"
  echo "CONTRACT_ADDR: ${contract_addr}"
  echo "${contract_addr}" > ./scripts/contract-address-junox
  update_contract_address "${contract_addr}"
}

function deploy_update {
  compile
  local code_id
  code_id=$(upload_code)
  local contract_addr
  contract_addr="$(cat scripts/contract-address-junox)"
  echo "CODE_ID: ${code_id}"
  echo "CONTRACT_ADDR: ${contract_addr}"
  migrate "${code_id}" "${contract_addr}"
}

function deploy_nois {
  local wasm="${1}"
  local code_id
  code_id=$(upload_noise_code "${wasm}")
  local contract_addr
  contract_addr="$(instantiate_nois "${code_id}")"
  echo "NOIS_CODE_ID: ${code_id}"
  echo "NOIS_CONTRACT_ADDR: ${contract_addr}"
}

# deploy_new
# deploy_update
# deploy_nois ~/Downloads/nois_proxy.wasm
