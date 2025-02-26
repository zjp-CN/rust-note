# Sans-IO

## 视频

python: <https://sans-io.readthedocs.io/>

Cory Benfield - Building Protocol Libraries The Right Way - PyCon 2016: <https://www.youtube.com/watch?v=7cC3_jGwl_U&ab_channel=PyCon2016>

* 某些代码只有一种方式编写（或者只有一种实现规范） —— 良好定义的问题和唯一正确的结果：算术、压缩算法以及各种格式解析（文件格式、网络协议）
* 现实情况：python 各种网络库不共用网络协议的解析代码，每个库因为支持的 IO 不同而重复编写解析逻辑（选择什么库取决于选择什么 IO）。这导致
  * 浪费时间：重复劳动但没有带来可重用的代码；开发资源投入在编写解析器上，导致在实验新 IO 和 API 等其他有趣的地方不足够（机会成本）
  * 重复错误：所有 python 库都会在解析 HTTP 上犯完全相同的错误，因为它们不知道或者无法建立在生态正确决策的其他基础上
  * 限制优化：优化和解析是不同的技能，擅长编写解析器的人可能不善长优化 socket IO（比如处理网络中的丢包、或者有的人关心低延迟）
* 解决方式：不要在解析器及其状态机中进行 IO。解析器应该可以插入到不同的 IO 范式而不出现问题。解析逻辑和 IO 逻辑的代码应该界限分明。
  一旦 IO 代码泄露到解析代码，在不同的环境中重用代码就会非常困难。
  * 比如对于 HTTP2 的流量控制（限制对方发送的数据量），HTTP 解析器应提供所需的 handles，让使用者做出自己的决定
* 高级别的网络库应共用解析器代码作为基础，把解析器与 IO 封装起来。
  当不同的人关心其中一个部分（解析器、IO、高级 API），不必担心其他部分被破坏，也不必知道其他部分的细节。
* 这种方式带来以下好处：
  * 非常易于测试：你可以彻底而广泛地测试解析器和任何 IO，覆盖各种边缘情况（相反：如果 IO 隐藏在代码深处，那么无法对其他 IO 测试，或者测试时必须执行 IO）
  * 结合特定于应用的 IO

--- 

Amos 视频《The case for sans-io》
* 视频：<https://www.youtube.com/watch?v=RYHYiXMJdZI>
* 仓库：<https://github.com/bearcove/rc-zip> （含与 IO 无关的 rc-zip，以及结合 IO 的 rc-zip-sync 和 rc-zip-tokio 等多个 crates）

该视频主要三部分组成：
1. 简要介绍 ZIP 文件、格式怪癖和解析要点
2. 介绍他重写的 zip 库（基于 sans-io 风格）： rc-zip （背景：相同的功能库在同步和异步 IO 上完全分裂，zip vs async_zip 由不同的人重新编写）
3. 把 rc-zip 集成到 monoio：与 IO 无关的 rc-zip 可以像插件一样集成不同异步运行时（虽然没有 rc-zip-monoio，但这部分介绍与 io_uring 风格的运行时结合的要点）

## 文章

A good blog post about Sans-IO: <https://www.firezone.dev/blog/sans-io>

Another good blog post reasoning about the problem it solves: <https://without.boats/blog/let-futures-be-futures/>


Examples in Rust:

* quinn-proto (underpinnings to quinn, which is a QUIC implementation) <https://crates.io/crates/quinn-proto>
* str0m (WebRTC library, disclaimer: my own) <https://github.com/algesten/str0m>
