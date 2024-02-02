## Generic Plonk Adapter

### Overview  
The general idea is this: 
- Almost all proving backends are some variation of AIR with plonkish addons (think lookups / copy constraints)
- Alot of work is being done with regards to this generalisation ( plaf, pil, airscript )  

Noir currently only has one supported backend (aztec's barretenberg) that uses plonk. Given that a plonkish backend already exists
it should be possible to support others without requiring a massive lift.

We can support more than one new proving system using a generic approach. 

ACIR -> Circuit Builder -> Proving System 

The Circuit Builder step should be able to be shared between proving backends. Then we can leverage a project that has already created generic proving adapters ( powdr ) to implement the proving system part for us. 


## Notice
This repository is in the ideation phase, and is not ready for contributions of any kind.
