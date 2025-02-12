CONTRACT_ADDR=$(<./scripts/contract-address-junox)

ADMIN="${SENDER:-juno1zk4c4aamef42cgjexlmksypac8j5xw7n3s4wrd}"
LAST_BLOCK=2123734

MEADOW='{"type": "meadow", "created_at": 2123734}'
SUNFLOWER_1='{"type": "field", "created_at": 2123734, "plant": {"type": "sunflower", "growth_period": 10, "stages": 5, "created_at": 2123734, "watered_at": [1,1,1,1,1]} }'
SUNFLOWER_2='{"type": "field", "created_at": 2123734, "plant": {"type": "sunflower", "growth_period": 10, "stages": 5, "created_at": 2123734, "watered_at": [1,1,1,1,1]} }'
SUNFLOWER_3='{"type": "field", "created_at": 2123734, "plant": {"type": "sunflower", "growth_period": 10, "stages": 5, "created_at": 2123734, "watered_at": [1,1,1,1,1]} }'
SUNFLOWER_4='{"type": "field", "created_at": 2123734, "plant": {"type": "sunflower", "growth_period": 10, "stages": 5, "created_at": 2123734, "watered_at": [1,1,1,1,1]} }'
SUNFLOWER_5='{"type": "field", "created_at": 2123734, "plant": {"type": "sunflower", "growth_period": 10, "stages": 5, "created_at": 2123734, "watered_at": [1,1,1,1,1]} }'

WHEAT_1='{"type": "field", "created_at": 2123734, "plant": {"type": "wheat", "growth_period": 10, "stages": 4, "created_at": 2123734, "watered_at": [1,1,1,1]} }'
WHEAT_2='{"type": "field", "created_at": 2123734, "plant": {"type": "wheat", "growth_period": 10, "stages": 4, "created_at": 2123734, "watered_at": [1,1,1,1]} }'
WHEAT_3='{"type": "field", "created_at": 2123734, "plant": {"type": "wheat", "growth_period": 10, "stages": 4, "created_at": 2123734, "watered_at": [1,1,1,1]} }'
WHEAT_4='{"type": "field", "created_at": 2123734, "plant": {"type": "wheat", "growth_period": 10, "stages": 4, "created_at": 2123734, "watered_at": [1,1,1,1]} }'
WHEAT_5='{"type": "field", "created_at": 2123734, "plant": {"type": "wheat", "growth_period": 10, "stages": 4, "created_at": 2123734, "watered_at": [1,1,1,1]} }'

SETUP_FARM_MSG=$(cat <<-END
    {
      "setup_farm": {
        "addr": "%s",
        "farm": {
          "plots": [
            [${MEADOW},      ${MEADOW},      ${MEADOW},      ${MEADOW},      ${MEADOW},      ${MEADOW},      ${MEADOW}     , ${MEADOW},      ${MEADOW}     ],
            [${MEADOW},      ${SUNFLOWER_1}, ${SUNFLOWER_1}, ${SUNFLOWER_2}, ${SUNFLOWER_2}, ${SUNFLOWER_3}, ${SUNFLOWER_3}, ${MEADOW},      ${MEADOW}     ],
            [${MEADOW},      ${SUNFLOWER_1}, ${SUNFLOWER_1}, ${SUNFLOWER_2}, ${SUNFLOWER_2}, ${SUNFLOWER_3}, ${SUNFLOWER_3}, ${MEADOW},      ${MEADOW}     ],
            [${MEADOW},      ${MEADOW},      ${MEADOW},      ${MEADOW},      ${MEADOW},      ${MEADOW},      ${MEADOW}     , ${MEADOW},      ${MEADOW}     ],
            [${MEADOW},      ${SUNFLOWER_4}, ${SUNFLOWER_4}, ${SUNFLOWER_5}, ${SUNFLOWER_5}, ${MEADOW},      ${WHEAT_5},     ${WHEAT_5},     ${MEADOW}     ],
            [${MEADOW},      ${SUNFLOWER_4}, ${SUNFLOWER_4}, ${SUNFLOWER_5}, ${SUNFLOWER_5}, ${MEADOW},      ${WHEAT_5},     ${WHEAT_5},     ${MEADOW}     ],
            [${MEADOW},      ${MEADOW},      ${MEADOW},      ${MEADOW},      ${MEADOW},      ${MEADOW},      ${MEADOW}     , ${MEADOW},      ${MEADOW}     ],
            [${MEADOW},      ${WHEAT_4},     ${WHEAT_4},     ${WHEAT_4},     ${WHEAT_4},     ${WHEAT_4},     ${WHEAT_4}    , ${WHEAT_4},     ${MEADOW}     ],
            [${MEADOW},      ${MEADOW},      ${MEADOW},      ${MEADOW},      ${MEADOW},      ${MEADOW},      ${MEADOW}     , ${MEADOW},      ${MEADOW}     ]
          ]
        }
      }
    }
END
)

MSG=$(printf "$SETUP_FARM_MSG" "$ADMIN")

echo $MSG

TX_HASH=$(junod --chain-id uni-6 --node https://juno-testnet-rpc.polkachu.com:443 tx wasm execute "$CONTRACT_ADDR" "$MSG" --from "${ADMIN}" --gas-prices 0.075ujunox --gas auto --gas-adjustment 1.3 -o json -y | jq '.txhash' -r)
echo $TX_HASH

