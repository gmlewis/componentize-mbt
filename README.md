# componentize-mbt

在 MoonBit 正式支持
[component model](https://github.com/WebAssembly/component-model)
之前，临时用于构建 component 的命令行工具。

## `bind-gen`

读取 [WIT](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md)
文件，生成 MoonBit 的 binding 代码。

## `build`

构建 MoonBit 项目，并合成出符合
[component model 规范](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Binary.md)的
`.wasm` 文件。
