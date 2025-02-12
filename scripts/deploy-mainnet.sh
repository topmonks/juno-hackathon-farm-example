#!/usr/bin/env bash

set -euo pipefail
shopt -s inherit_errexit

ADMIN=${1:-juno1sp3wrffwvtsaz255x63w7c5m40glej8p3vwcwt}

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
  local tx_hash
  tx_hash="$(junod tx wasm store artifacts/juno_farm_hackathon_template.wasm --from "${ADMIN}" --gas-prices 0.075ujuno --gas auto --gas-adjustment 1.1 -o json -y | jq '.txhash' -r)"
  sleep 10

  junod query tx "${tx_hash}" -o json | jq '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value' -r
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

  local komple_mint_addr
  komple_mint_addr=juno17rth4jstxs7cmrusvyluwlnt34l80cxaz7nufpjfntts00pk79asjxelgs

  local msg
  msg="$(printf "${instantiate_msg}" "$ADMIN" "$komple_mint_addr")"

  local tx_hash
  tx_hash=$(junod tx wasm instantiate "${code_id}" "${msg}" --from "${ADMIN}" --admin "${ADMIN}" --gas-prices 0.075ujuno --gas auto --gas-adjustment 1.1 --label " " -o json -y | jq '.txhash' -r)
  sleep 10

  junod query tx "${tx_hash}" -o json | jq 'last(.logs[0].events[] | .attributes[] | select(.key=="_contract_address") | .value)' -r
}

function migrate {
    local code_id="${1}"
    local contract_addr="${2}"

    tx_hash=$(junod --from "${ADMIN}" tx wasm migrate "${contract_addr}" "${code_id}" '{}' --gas-prices 0.075ujuno --gas auto --gas-adjustment 1.2 -o json -y | jq '.txhash' -r )
    echo "Migration TX hash: ${tx_hash}"
}

function deploy_new {
  compile
  local code_id
  code_id=$(upload_code)
  echo "CODE_ID: ${code_id}"
  local contract_addr
  contract_addr="$(instantiate "${code_id}")"
  echo "CONTRACT_ADDR: ${contract_addr}"
  echo "${contract_addr}" > ./scripts/contract-address
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

deploy_new
# deploy_update
