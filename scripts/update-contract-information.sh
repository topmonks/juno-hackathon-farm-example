CONTRACT_ADDR=$(<./scripts/contract-address-junox)

ADMIN="${SENDER:-juno1zk4c4aamef42cgjexlmksypac8j5xw7n3s4wrd}"
KOMPLE_MINT_ADDR="${KOMPLE_MINT_ADDR:-juno17rth4jstxs7cmrusvyluwlnt34l80cxaz7nufpjfntts00pk79asjxelgs}"

SETUP_FARM_MSG=$(cat <<-END
    {
      "update_contract_information": {
        "contract_information": {
          "admin": "%s",
          "komple_mint_addr": "%s"
        }
      }
    }
END
)

MSG=$(printf "$SETUP_FARM_MSG" "$ADMIN" "$KOMPLE_MINT_ADDR")

echo $MSG

TX_HASH=$(junod --chain-id uni-6 --node https://juno-testnet-rpc.polkachu.com:443 tx wasm execute "$CONTRACT_ADDR" "$MSG" --from "${ADMIN}" --gas-prices 0.075ujunox --gas auto --gas-adjustment 1.3 -o json -y | jq '.txhash' -r)
echo $TX_HASH

