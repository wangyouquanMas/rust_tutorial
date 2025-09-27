背景：在本地 `fun-uniswap-v3` Anchor 项目中执行 `anchor test`，期望完成 Rust 单元测试与 TypeScript 集成测试。

错误1：
背景：【运行 `anchor test` 调用 Anchor.toml 中的测试脚本】
错误：【命令输出 `yarn: error: no such option: -p`，随后提示 `Parsing scenario file run`，发现系统中的 `yarn` 实际指向 Debian 的 `cmdtest`】
方案：【通过 `npm install --global yarn` 安装官方 Yarn CLI（或替换为 corepack 管理的版本），确保 `yarn run ts-mocha` 使用真实 Yarn；同时在 `Anchor.toml` 中使用 `yarn run ts-mocha -- -p ./tsconfig.json -t 1000000 tests/**/*.ts` 将参数转发给 `ts-mocha`】

错误2：
背景：【重新运行 `anchor test` 后进入 TypeScript 测试阶段】
错误：【Yarn 报错 `Command "ts-mocha" not found.`，因为 `node_modules/.bin/ts-mocha` 尚未生成】
方案：【在项目根目录执行 `yarn install`，安装 package.json 中定义的依赖，使 `ts-mocha` 等可执行文件进入 PATH】

错误3：
背景：【安装依赖时运行 `yarn install`】
错误：【安装过程中提示 `@solana/codecs-numbers@2.3.0` 需要 Node `>=20.18.0`，当前环境为 `18.19.1`】
方案：【使用 `npm install --global n && n 20.18.0`（或其他 Node 版本管理工具）升级 Node 至满足要求的版本，随后重新执行 `yarn install` 并再次运行 `anchor test`】

