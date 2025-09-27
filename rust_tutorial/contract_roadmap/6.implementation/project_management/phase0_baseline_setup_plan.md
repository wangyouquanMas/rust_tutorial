# Phase 0 Baseline Setup 执行计划

1：任务目标：【验证 Anchor 工程脚手架可编译、可运行，确保基础环境稳定，为后续合约开发提供可靠基础】
2：执行步骤：
step1: 在项目根目录运行 `anchor --version` 与 `solana --version`，确认工具链版本符合预期并记录。
step2: 执行 `anchor build`，观察输出，若出现编译错误，记录错误信息并在 `error_logs` 目录建立条目。
step3: 执行 `anchor test`，确保集成测试脚手架可通过；若测试失败，定位具体测试并修复。
step4: 将本阶段的命令与结果简要记录到 `contract_roadmap/6.implementation/1.anchor_tutorial/readme.md` 以备查。
3：任务完成标准：【`anchor build` 与 `anchor test` 均成功通过且无未解决错误记录；日志文档更新完成】

---

1：任务目标：【搭建合约项目的模块化骨架，明确 state、instructions、errors、events 等目录，为分阶段开发打好结构基础】
2：执行步骤：
step1: 在 `programs/fun-uniswap-v3/src` 下创建 `state`, `instructions`, `errors`, `events`, `utils` 等子模块文件（如 `mod.rs` + 具体实现文件）。
step2: 在 `lib.rs` 中引入新模块，保持入口文件结构清晰；编写必要的 pub use 导出。
step3: 为每个模块添加占位注释或基础结构体/枚举定义，确保后续扩展时有明确位置。
step4: 更新 `1.anchor_tutorial/readme.md`，记录模块规划与用途说明。
3：任务完成标准：【`lib.rs` 成功引用各模块并通过 `anchor build`；目录结构清晰，文档中记录模块职责】

---

1：任务目标：【建立 Phase 0 的文档与版本管理规范，确保后续阶段的变更可追踪、可复用】
2：执行步骤：
step1: 在 `contract_roadmap/6.implementation/1.anchor_tutorial/readme.md` 中新增 Phase 0 小节，描述目标、关键命令与常见问题。
step2: 若在执行过程中发现问题，及时在 `error_logs/Anchor测试环境问题排查.md` 中追加条目，包含现象、原因及解决方案。
step3: 完成所有步骤后执行 `git status` 确认修改，按需拆分提交；遵循 `git add`、`git commit -m "错误总结"`、`git push` 的流程。
3：任务完成标准：【文档新增 Phase 0 内容并涵盖操作说明；错误日志更新完善；版本库已将本阶段改动推送至远程】


