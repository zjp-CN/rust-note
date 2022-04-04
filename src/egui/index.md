# egui

> 项目地址：[https://github.com/emilk/egui](https://github.com/emilk/egui)

## 关联项目

### egui_demo_lib/app

[egui_demo_lib](https://github.com/emilk/egui/tree/master/egui_demo_lib) 
是 [egui_demo_app](https://github.com/emilk/egui/tree/master/egui_demo_app) 的实现库，它们都是样例。

它们的结果可通过 WASM 从这个网页体验：[https://www.egui.rs/#demo](https://www.egui.rs/#demo)。

当然，你也可以把项目下载到本地，把它编译成本地应用来获得更好的体验：

```console
git clone https://github.com/emilk/egui.github
cargo run -p --release egui_demo_app
```

在 linux 下，如果你想在本地编译成 WASM 你需要安装以下软件，然后运行 `build_demo_web.sh` 脚本：

```console
# 给 Rust 的 WASM 文件生成 JS bindings
cargo install wasm-bindgen
# wabt 的 wasm-strip 工具用来减小 WASM 体积
# binaryen 的 wasm-opt 工具用来优化 WASM 
apt install binaryen jq wabt

./sh/build_demo_web.sh
```

如果在其他平台想编译成 WASM 项目，也可以参照这个 sh 脚本（里面的工具命令）。

作为 `egui` 的样例和集成测试，它的代码非常多。幸好它的许多部分都附上了代码地址，你对哪部分感兴趣可以直接从代码中学习。

### eframe

> doc: [https://docs.rs/eframe](https://docs.rs/eframe)

[eframe_template]: https://github.com/emilk/eframe_template/

`egui` 和 `epi` 的框架库，实质上做了以下一些事情：
1. 对 `egui` 和 `epi` 进行重导出：它们与 `eframe` 的版本号是一致，从而你无需把这两个 crates 添加到 `Cargo.toml` 依赖
    - `eframe` 对它们的 features 也做了重导出，所以你不能利用 `eframe/egui/xx` 多级目录形式控制其依赖的 features
2. 利用条件编译，整合了 native 与 WASM 下的样板代码，让你更方便地从统一入口编写跨 native/WASM 
    平台的应用：这也是 `eframe` 的主要意义

它有一个项目样例 [eframe_template]，你可以基于它的代码把代码编译 native/WASM 平台的应用。

温馨提示：你可阅读它提供的 sh 脚本，然后基于它进行修改应用；如果在其他平台上，则参考里面的工作流程和工具命令。
<!-- 2. [egui/docs](https://github.com/emilk/egui/tree/master/docs) 可看出一个网页部署样例，你可以把 [eframe_template] -->
<!--     生成的 WASM 代码和 JS 文件替换过去（再改一下 index.html）的一些代码，就是一个完整 -->


此外，它有一些[小样例文件](https://github.com/emilk/egui/tree/master/eframe/examples)，你可以从中更快速地学习。

通常，你会在考虑把基于 `egui` 的代码编译成 native 与 WASM 时考虑使用 
`eframe`，所以随之而来的是，你需要使用能够跨 WASM 
编译的库（比如[这里](https://github.com/emilk/egui/tree/master/eframe#companion-crates)）。
