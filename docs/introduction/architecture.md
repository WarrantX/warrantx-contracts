# Architecture

The factory stores the approved treasury WASM hash and deploys isolated treasury instances. Each treasury owns one Stellar asset and stores its configuration, member records, spending policies, allowance counters, payment requests, and approvals. The treasury uses the Soroban token client for deposits and executions.

