# Juno Farm Hackathon Template

A very rough contract template that implements 2 example executes and 2 queries to get you started.

There are MANY areas of improvement throughout the application which can be made.

The base template includes:

- [Proof of Concept](./_ProofOfConcept/)
- [Contract Template Source](./src/)
- [Bash E2E testing](./e2e/)

If you have `make` installed, you can run `make compile` to get your contract to compile (x86_64 processors).

The farm.rs has the base farm game logic. The code here will need cleanup. Only the start & till functions will 100% work as expected.

It is up to you to be creative, redesign, and implement better features off what has been provided.

```bash
wasm_cmd $FARM_CONTRACT '{"start":{}}' "" show_log

profile=$(query_contract $FARM_CONTRACT '{"get_farm_profile":{"address":"juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl"}}' | jq -r '.data') && echo $profile
# { "plots": [ [ "Grass", "Grass", "Grass" ], [ "Grass", "Grass", "Grass" ], [ "Grass", "Grass", "Grass" ] ], "cooldowns": {} }

wasm_cmd $FARM_CONTRACT '{"till_ground":{"x":0,"y":0}}' "" show_log

profile=$(query_contract $FARM_CONTRACT '{"get_farm_profile":{"address":"juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl"}}' | jq -r '.data') && echo $profile
# { "plots": [ [ "Dirt", "Grass", "Grass" ], [ "Grass", "Grass", "Grass" ], [ "Grass", "Grass", "Grass" ] ], "cooldowns": {} }
```

Generate your Typescript from [CosmWasm/ts-codegen](https://github.com/CosmWasm/ts-codegen) to convert your contract's executes and queries into a Typescript interface.

Use [https://juno.reece.sh/](https://juno.reece.sh/) to register your contract with FeeShare (mainnet)!
