目标：
1. 理解ini库用法

内容:
1. 用法
`configparser::ini::Ini` gives the client a lightweight way to read values from the INI config file. In `load_cfg` it loads the user’s `client_config.ini`, pulls out entries such as `http_url`, `ws_url`, wallet keypair paths, Raydium program ID, pool mint IDs, and AMM parameters, and then uses them to build the `ClientConfig` struct and derive the various PDAs needed at runtime.