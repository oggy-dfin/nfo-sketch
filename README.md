# nfo

Welcome to your new nfo project and to the internet computer development community. By default, creating a new project adds this README and some template files to your project directory. You can edit these template files to customize your project and to include your own code to speed up the development cycle.

To get started, you might want to explore the project directory structure and the default configuration file. Working with this project in your development environment will not affect any production deployment or identity tokens.

To learn more before you start working with nfo, see the following documentation available online:

- [Quick Start](https://smartcontracts.org/docs/quickstart/quickstart-intro.html)
- [SDK Developer Tools](https://smartcontracts.org/docs/developers-guide/sdk-guide.html)
- [Rust Canister Devlopment Guide](https://smartcontracts.org/docs/rust-guide/rust-intro.html)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://smartcontracts.org/docs/candid-guide/candid-intro.html)
- [JavaScript API Reference](https://erxue-5aaaa-aaaab-qaagq-cai.raw.ic0.app)

If you want to start working on your project right away, you might want to try the following commands:

```bash
cd nfo/
dfx help
dfx config --help
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:8000?canisterId={asset_canister_id}`.

# Notes

Use case:
  1. Ognjen owns two sword NFOs
  2. Satya owns 3 potion NFOs
  3. Ognjen wants to trade the two swords for 3 potions
  4. The trade should happen only if Satya agrees

Level 1 policy:
 Sword: can be transferred by owner
 Potion: can be transferred by owner


======================================================
Level 2 policy: dynamic, can be added/changed by users
======================================================


Basic objects:
  (object_id: #1, type: #sword, metadata: { owner: Ognjen }, #operations: {name: owner.write, authorizer: owner})
  (object_id: #2, type: #sword, metadata: { owner: Ognjen }, #operations: {name: owner.write, authorizer: owner})
  (object_id: #3, type: #potion, metadata: { owner: Satya }, #operations: {name: owner.write, authorizer: owner})
  (object_id: #4, type: #potion, metadata: { owner: Satya }, #operations: {name: owner.write, authorizer: owner})
  (object_id: #5, type: #potion, metadata: { owner: Satya }, #operations: {name: owner.write, authorizer: owner})

Ognjen calls create_proposal on the ledger:
   object(#1).owner.write(Satya); (needs to be approved by: Ognjen)
   object(#2).owner.write(Satya); (needs to be approved by: Ognjen)
   object(#3).owner.write(Ognjen); (needs to be approved by: Satya)
   object(#4).owner.write(Ognjen); (needs to be approved by: Satya)
   object(#5).owner.write(Ognjen); (needs to be approved by: Satya)

Result: proposal #15

Satya calls accept_proposal(#15)
   - The ledger checks that all basic operations are approved by either creator or the acceptor of the proposal
   - The ledger transfers the NFOs

------------------------------

Use case #2:
1. NFT creator can mint new objects as they want
2. Ognjen owns an egg

Basic object:
  (object_id: #1, type: egg, metadata: { owner: Ognjen }, operations: {name: burn, authorizer: owner } )

NFT creator calls create_proposal on the ledger:
   object(#1).burn(); (needs to be approved by Ognjen)
   mint({ type: baby_dino, metadata: { owner: Ognjen }, operations: {name: burn, authorizer: owner } } );
                (needs to be approved by NFT creator)

Result: proposal #42

Ognjen calls accept_proposal(#42)
   - The ledger checks that all basic operations are approved by either the creator or the acceptor of the proposal
   - The ledger executes the rule; as a result, egg is burnt, baby dino is minted
   - The proposal #42 is deleted

------------------------------

=======================================
Level 2.5 policy: transitive delegation
=======================================


Use case #3:
  Ognjen wants to sell all 3 of his swords, but only if he can get both 2 bottles of potion from Satya and 1 magic hat from Jan

  (object_id: #1, type: #sword, metadata: { owner: Ognjen }, #operations: {name: owner.write, authorizer: owner})
  (object_id: #2, type: #sword, metadata: { owner: Ognjen }, #operations: {name: owner.write, authorizer: owner})
  (object_id: #3, type: #sword, metadata: { owner: Ognjen }, #operations: {name: owner.write, authorizer: owner})
  (object_id: #4, type: #potion, metadata: { owner: Satya }, #operations: {name: owner.write, authorizer: owner})
  (object_id: #5, type: #potion, metadata: { owner: Satya }, #operations: {name: owner.write, authorizer: owner})
  (object_id: #6, type: #magic_hat, metadata: { owner: Jan }, #operations: {name: owner.write, authorizer: owner})

Satya calls create_proposal for Ognjen with
   object(#1).owner.write(Satya); (needs to be approved by: Ognjen)
   object(#2).owner.write(Satya); (needs to be approved by: Ognjen)
   object(#4).owner.write(Ognjen); (needs to be approved by: Satya)
   object(#5).owner.write(Ognjen); (needs to be approved by: Satya)

Result: proposal #1

Ognjen calls create_proposal for Jan with:
   object(#3).owner.write(Jan); (needs to be approved by: Ognjen)
   object(#6).owner.write(Ognjen); (needs to be approved by: Jan)
   accept_proposal(#1)

Result: proposal #2
   
Jan calls accept_proposal(#2); this results in the full trade.
With proposal #2, Ognjen (conditionally) delegated his right to accept proposal #1 to Jan.

==============================
Level 3: programmatic policies
==============================

Use case #4: like #2, but we want a "standing order" by the NFT creator that is applicable to all eggs

Level 3 policy (with parameters, assertions and so on)

NFT creator create_proposal on the ledger:
  - the proposal takes a parameter, object_id

    require(object(object_id).type = "egg");
    object(object_id).burn();
    mint({ type: baby_dino, metadata: { owner: object(object_id).owner }, operations: {name: burn, authorizer: owner } } 


------------------------------

