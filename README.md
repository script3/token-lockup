# soroban-token-lockup

Soroban token lockup is a library implementing a smart contract designed for token via token lockups. It has two implementations:

Standard Lockup Contracts which implement basic lockup functionality by specifying a series of unlocks and the percent of total tokens that can be claimed at each lockup. These can be used as vesting contracts by retaining the admin role, or into lockup contracts by revoking it.

and 

Blend Lockup Contracts which enable interactions with Blend Protocols backstop contract. These are used for Blend's ecosystem distribution, but could be adapted for usage with other protocols if teams wish to issue a lockup but still allow the tokens to be utilized in their protocol.
