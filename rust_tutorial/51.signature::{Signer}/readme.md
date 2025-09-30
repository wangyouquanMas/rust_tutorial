目标：
1. 理解Singer库的用法

内容:
1. 用法
`Signer` brings the `Signer` trait from `solana_sdk`, which defines the common signing interface for keypairs and other signing types. By importing it, you gain the extension methods that trait provides—most notably `pubkey()` and `try_pubkey()`—on `Keypair` and any other types implementing `Signer`.