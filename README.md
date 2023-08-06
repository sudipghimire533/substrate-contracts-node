# substrate-contracts-node

## Changes made
- Fix value index of all pallets in [construct_runtime](runtime/src/lib.rs#L390)
- Added test [chain_extension_is_enabled](runtime/src/lib.rs#L641)
- Change extension to AssetExtension in [pallet-contract::Config](runtime/src/lib.rs#L370)
- Implement AssetExtension in [asset_extension.rs](runtime/src/asset_extension.rs)
- Contract to showcase how to use in ink! [asset_contract](contract/)

## Completeness
- [ X ] Pass `cargo +nightly test`
- [ X ] Can build and run node `cargo +nightly run -- --dev`
- [ X ] Can verify chain is working
- [ X ] Can verify default `flipper` contract is working with new configuration of pallet_contract
- [ X ] Can verify the contract ( in another repo ) which make use of chain extension is working
