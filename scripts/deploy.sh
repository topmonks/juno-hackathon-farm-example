docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.11

ADMIN=${1:-juno1zk4c4aamef42cgjexlmksypac8j5xw7n3s4wrd}
TX_HASH=$(junod --chain-id uni-6 --node https://juno-testnet-rpc.polkachu.com:443 tx wasm store artifacts/juno_farm_hackathon_template.wasm --from $ADMIN --gas-prices 0.075ujunox --gas auto --gas-adjustment 1.1 -o json -y | jq '.txhash' -r)
sleep 10

CODE_ID=$(junod --chain-id uni-6 --node https://juno-testnet-rpc.polkachu.com:443 query tx "$TX_HASH" -o json | jq '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value' -r)
echo $CODE_ID

INSTANTIATE_MSG=$(cat <<-END
    {
      "admin": "%s"
    }
END
)

MSG=$(printf "$INSTANTIATE_MSG" "$ADMIN")

TX_HASH=$(junod --chain-id uni-6 --node https://juno-testnet-rpc.polkachu.com:443 tx wasm instantiate $CODE_ID "$MSG" --from $ADMIN --admin $ADMIN --gas-prices 0.075ujunox --gas auto --gas-adjustment 1.1 --label " " -o json -y | jq '.txhash' -r)
sleep 10
echo $TX_HASH

CONTRACT_ADDR=$(junod --chain-id uni-6 --node https://juno-testnet-rpc.polkachu.com:443 query tx "$TX_HASH" -o json | jq 'last(.logs[0].events[] | .attributes[] | select(.key=="_contract_address") | .value)' -r)
CONTRACT_ADDR=${CONTRACT_ADDR}
echo $CONTRACT_ADDR | tee ./scripts/contract-address-junox
