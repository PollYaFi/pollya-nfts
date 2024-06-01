./build/pollyad  tx wasm store ../cw-nfts/artifacts/cw721_non_transferable-aarch64.wasm  --chain-id chain-3Q1pP7 --home ./.testnets/node0/pollyad --from node0 --gas 7000000

./build/pollyad tx wasm instantiate 3 '{"name": "PollyaNFT", "symbol": "PN", "minter": "poll1a4mp7kedzuquntnqtfd2yjulk9t53c66mqgyx6", "admin": "poll1a4mp7kedzuquntnqtfd2yjulk9t53c66mqgyx6"}' --from node0  --home ./.testnets/node0/pollyad  --gas "4000000"  --label "test" --no-admin --chain-id chain-3Q1pP7

export CONTRACT_ADDRESS=poll1qwlgtx52gsdu7dtp0cekka5zehdl0uj3fhp9acg325fvgs8jdzkszh66s7

./build/pollyad tx wasm execute $CONTRACT_ADDRESS '{"mint": {"token_id": "tkn-1", "owner": "poll1a4mp7kedzuquntnqtfd2yjulk9t53c66mqgyx6", "extension": {"did": "test-did", "wallet": "wall"}}}' --from node0  --home ./.testnets/node0/pollyad  --gas "4000000" --chain-id chain-3Q1pP7 

