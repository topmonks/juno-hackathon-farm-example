# Test script for Juno Smart Contracts (By @Reecepbcups)
#
# sh ./e2e/test_e2e.sh
#
# NOTES: anytime you use jq, use `jq -rc` for ASSERT_* functions (-c removes format, -r is raw to remove \" quotes)

# get functions from helpers file 
# -> query_contract, wasm_cmd, mint_cw721, send_nft_to_listing, send_cw20_to_listing
source ./e2e/helpers.sh

# NOTE: Its probably better to e2e test with JS/TS, but this gives you some info for runing in CLI 
# (I'm on linux 86_64. If you are on a M1/M2 mac, compile_and_copy & start_docker will not work)
# We do have a arm based juno on v13 here:
# docker pull ghcr.io/cosmoscontracts/juno:v13.0.1@sha256:7fd1f38098342355b28ba01d31ae2e32924ea18e739bcd0e550cdf13bc8a5683
# (https://github.com/CosmosContracts/juno/pkgs/container/juno)

CONTAINER_NAME="juno_farm_hackathon"
BINARY="docker exec -i $CONTAINER_NAME junod"
DENOM='ujunox'
JUNOD_CHAIN_ID='testing'
JUNOD_NODE='http://localhost:26657/'
TX_FLAGS="--gas-prices 0.1$DENOM --gas-prices="0ujunox" --gas 5000000 -y -b block --chain-id $JUNOD_CHAIN_ID --node $JUNOD_NODE --output json"
export JUNOD_COMMAND_ARGS="$TX_FLAGS --from test-user"
export KEY_ADDR="juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl"


# ===================
# === Docker Init ===
# ===================
function stop_docker {
    docker kill $CONTAINER_NAME
    docker rm $CONTAINER_NAME
    docker volume rm --force junod_data
}

function start_docker {
    IMAGE_TAG=${2:-"v13.0.1"}
    BLOCK_GAS_LIMIT=${GAS_LIMIT:-10000000} # mirrors mainnet

    echo "Building $IMAGE_TAG"
    echo "Configured Block Gas Limit: $BLOCK_GAS_LIMIT"

    stop_docker    

    # run junod docker
    docker run --rm -d --name $CONTAINER_NAME \
        -e STAKE_TOKEN=$DENOM \
        -e GAS_LIMIT="$GAS_LIMIT" \
        -e UNSAFE_CORS=true \
        -e TIMEOUT_COMMIT="500ms" \
        -p 1317:1317 -p 26656:26656 -p 26657:26657 \
        --mount type=volume,source=junod_data,target=/root \
        ghcr.io/cosmoscontracts/juno:$IMAGE_TAG /opt/setup_and_run.sh $KEY_ADDR    
}

function compile_and_copy {    
    # compile vaults contract here
    docker run --rm -v "$(pwd)":/code \
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
      cosmwasm/rust-optimizer:0.12.11

    # copy wasm to docker container
    docker cp ./artifacts/juno_farm_hackathon_template.wasm $CONTAINER_NAME:/juno_farm_hackathon_template.wasm
}

function health_status {
    # validator addr
    VALIDATOR_ADDR=$($BINARY keys show validator --address) && echo "Validator address: $VALIDATOR_ADDR"

    BALANCE_1=$($BINARY q bank balances $VALIDATOR_ADDR) && echo "Pre-store balance: $BALANCE_1"

    echo "Address to deploy contracts: $KEY_ADDR"
    echo "JUNOD_COMMAND_ARGS: $JUNOD_COMMAND_ARGS"
}

# ========================
# === Contract Uploads ===
# ========================
function upload_farm_contract {
    echo "Storing contract..."
    UPLOAD=$($BINARY tx wasm store /juno_farm_hackathon_template.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
    BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"

    # == INSTANTIATE ==
    ADMIN="$KEY_ADDR"

    # JSON_MSG=$(printf '{"addresses":["%s","%s","%s"],"data":[{"id":"JUNO","exponent":6}],"max_submit_rate":10}' "$ADMIN" "juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk" "juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y")
    TX_HASH=$($BINARY tx wasm instantiate "$BASE_CODE_ID" "{}" --label "farm" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $VAULT_TX


    export FARM_CONTRACT=$($BINARY query tx $TX_HASH --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "FARM_CONTRACT: $FARM_CONTRACT"
}

# ===============
# === ASSERTS ===
# ===============
FINAL_STATUS_CODE=0

function ASSERT_EQUAL {
    # For logs, put in quotes. 
    # If $1 is from JQ, ensure its -r and don't put in quotes
    if [ "$1" != "$2" ]; then        
        echo "ERROR: $1 != $2" 1>&2
        FINAL_STATUS_CODE=1 
    else
        echo "SUCCESS"
    fi
}

function ASSERT_CONTAINS {
    if [[ "$1" != *"$2"* ]]; then
        echo "ERROR: $1 does not contain $2" 1>&2        
        FINAL_STATUS_CODE=1 
    else
        echo "SUCCESS"
    fi
}

function add_accounts {
    # provision juno default user i.e. juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl
    echo "decorate bright ozone fork gallery riot bus exhaust worth way bone indoor calm squirrel merry zero scheme cotton until shop any excess stage laundry" | $BINARY keys add test-user --recover --keyring-backend test
    # juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk
    echo "wealth flavor believe regret funny network recall kiss grape useless pepper cram hint member few certain unveil rather brick bargain curious require crowd raise" | $BINARY keys add other-user --recover --keyring-backend test
    # juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
    echo "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose" | $BINARY keys add user3 --recover --keyring-backend test

    # send some 10 junox funds to the users
    $BINARY tx bank send test-user juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk 10000000ujunox $JUNOD_COMMAND_ARGS
    $BINARY tx bank send test-user juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y 100000ujunox $JUNOD_COMMAND_ARGS

    # check funds where sent
    # other_balance=$($BINARY q bank balances juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk --output json)
    # ASSERT_EQUAL "$other_balance" '{"balances":[{"denom":"ujunox","amount":"10000000"}],"pagination":{"next_key":null,"total":"0"}}'
}

# === COPY ALL ABOVE TO SET ENVIROMENT UP LOCALLY ====



# =============
# === LOGIC ===
# =============

start_docker
compile_and_copy # the compile takes time for the docker container to start up

sleep 5
# add query here until state check is good, then continue

# Don't allow errors after this point
# set -e

health_status

add_accounts

upload_farm_contract
# FARM_CONTRACT=juno14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9skjuwg8

# == INITIAL TEST ==
info=$(query_contract $FARM_CONTRACT '{"contract_info":{}}' | jq -r '.data') && echo $info



# start
wasm_cmd $FARM_CONTRACT '{"start":{}}' "" show_log
profile=$(query_contract $FARM_CONTRACT '{"get_farm_profile":{"address":"juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl"}}' | jq -r '.data') && echo $profile
# { "plots": [ [ "Grass", "Grass", "Grass" ], [ "Grass", "Grass", "Grass" ], [ "Grass", "Grass", "Grass" ] ], "cooldowns": {} }

wasm_cmd $FARM_CONTRACT '{"till_ground":{"x":0,"y":0}}' "" show_log
profile=$(query_contract $FARM_CONTRACT '{"get_farm_profile":{"address":"juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl"}}' | jq -r '.data') && echo $profile

# ...

# OLD
# submit price (so $1 is 1_000_000. Then when we query, we just / 1_000_000 = 1)
# only the addresses in 'addresses' can submit prices. 
# wasm_cmd $FARM_CONTRACT '{"submit":{"data":[{"id":"JUNO","value":1000000}]}}' "" show_log
# wasm_cmd $FARM_CONTRACT '{"submit":{"data":[{"id":"JUNO","value":1001000}]}}' "" show_log "$TX_FLAGS --keyring-backend test --from other-user"
# wasm_cmd $FARM_CONTRACT '{"submit":{"data":[{"id":"JUNO","value":1004000}]}}' "" show_log "$TX_FLAGS --keyring-backend test --from user3"