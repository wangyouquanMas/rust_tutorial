目标：
1. 理解 idl-build的用法


内容:
1. 用法
- **Problem**: The on-chain program received an instruction whose discriminator didn’t match any handler, so Anchor fell back to the (unsupported) fallback route and returned `InstructionFallbackNotFound`.
- **Reason**: The generated IDL was stale and still lacked the new `create_pool` entry. The client’s request builder (driven by the IDL) therefore emitted a fallback-formatted instruction, which the program rejected.
- **Solution**: Forward the `idl-build` feature to `anchor-spl` in `programs/fun-uniswap-v3/Cargo.toml`, rebuild the program/IDL, and redeploy. This keeps the IDL current so the client encodes `create_pool` with the correct discriminator.
- **Concepts involved**: Anchor IDL generation, instruction discriminators, feature flag propagation (`idl-build`), and Anchor’s fallback-instruction behavior.