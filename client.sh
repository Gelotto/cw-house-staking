#!/bin/bash

CMD=$1
NETWORK=$2
NODE=
CHAIN_ID=
FLAGS=

TAG=$3
if [ -z "$TAG" ]; then
  TAG=$(cat ./builds/latest)
fi

CONTRACT_ADDR=$(cat ./builds/build-$TAG/latest-contract)

shift 3

case $NETWORK in
  testnet)
    NODE="https://rpc.uni.juno.deuslabs.fi:443"
    CHAIN_ID=uni-3
    DENOM=ujunox
    ;;
  mainnet)
    NODE="https://rpc-juno.itastakers.com:443"
    CHAIN_ID=juno-1
    DENOM=ujuno
    ;;
  devnet)
    NODE="http://localhost:26657"
    CHAIN_ID=testing
    DENOM=ujunox
    ;;
esac


delegate() {
  sender=$1
  growth_amount=$2
  profit_amount=$3
  msg='{"delegate":{"growth":"'$growth_amount'","profit":"'$profit_amount'"}}'
  flags="\
  --node $NODE \
  --gas-prices 0.025$DENOM \
  --chain-id $CHAIN_ID \
  --from $sender \
  --gas auto \
  --gas-adjustment 1.5 \
  --broadcast-mode block \
  --output json \
  -y \
  "
  echo junod tx wasm execute $CONTRACT_ADDR "$msg" "$flags"
  response=$(junod tx wasm execute "$CONTRACT_ADDR" "$msg" $flags)
  echo $response | ./bin/utils/base64-decode-attributes | jq
}


query-select() {
  query='{"select":{"fields":null}}'
  flags="--chain-id $CHAIN_ID --output json --node $NODE"
  echo junod query wasm contract-state smart $CONTRACT_ADDR "$query" $flags
  response=$(junod query wasm contract-state smart $CONTRACT_ADDR "$query" $flags)
  echo $response | ./bin/utils/base64-decode-attributes | jq
}

set -e
echo "executing $CMD for $CONTRACT_ADDR"

case $CMD in
  delegate)
    delegate $1 1000000000 500000000
    ;;
  query-select) 
    query-select
    ;;
  *)
    echo "unrecognized option: $CMD" >&2
    exit -1
esac