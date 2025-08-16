# 我与 Rust 语言的 5 年

Github：[@zjp-CN](https://github.com/zjp-CN) 
| English Nickname: [@Vague](https://users.rust-lang.org/u/vague)
| Chinese Nickname: 苦瓜小仔

# 学习 Rust 的过程

我初次听到 Rust 语言，大概是 2019 年左右在 Julia 语言的中文社区 QQ 群里。给我很深印象的是，一位
Julia 群友的昵称和头像都是 Rust。直到 2021 年初，我才开始真正学习它。

我入门 Rust 主要资料是官方的《[The Rust Programming Language][trpl]》(the book) 一书。
我从头到尾阅读过它每个句子、摘抄了电子笔记，以至于回答初学者问题的时候，我可以清楚地给出相应的章节和句子。

<details>

<summary>非计算机背景</summary>

但其实在第一次阅读它的时候，我没有那么顺利。因为在此之前，我的大多数经验是 Python、R 这类动态语言，对编译型语言不习惯。
看了几个 Rust 入门视频才知道如何安装和使用 Rust 编程。我印象最深的 [oeasy](http://www.oeasy.org/) 这位中国传媒大学老师，他带着你阅读
the book，并逐一演示那些示例如何工作或者不工作。oeasy 是我的启蒙老师，十几年前我还在高中的时候，就观看了他的一系列教程视频，比如办公软件
(word/excel/ppt)、音视频和图像编辑 (au/pr/ps)、前端 (html/css/js)。他打开了一扇扇窗，带我领略这些新的世界。感谢这位无私奉献的刘老师。

我没有什么机会去真正专业地学习计算机知识，只是按兴趣编程。2015 年我作为文科生考入南昌大学经济管理学院的第一届经济统计学专业。
这是一个实验性的专业，经济学和（数理）统计学交叉，但据说在我这一届毕业之后该专业不再招生。
经管学院能参加的全国性比赛不多，数学建模这类比赛是其中一个。通常参加数学建模比赛的是 3 人小组，并且至少有一个计算机专业的成员来负责编程。
我阴差阳错地担任了编程的角色，并且作为了队长。在这个 3 个经管学院小队里，我负责编写程序和帮助建立数学模型来解决问题。Matlab 
是参赛者的首选，而我选择了 Python 和 R 语言。倒不是因为我对这些语言很熟 —— 你不能期望一个大二的非计算机专业的学生能熟练编程；
而是因为它们用起来有趣，我能够用它们愉快地处理数据，并方便地调用计量经济、概率统计或机器学习之类的算法库。
在我大四的时候，竞争学校级别的保研名额失败。本来我想成为一个数据分析师，但找了一个仓储物流的运营工作。
我的工作从起初的仓库基层轮岗，到运营数据监控，到产品设计 MES（生产管理系统）与技术人员对接。业余时间我还参加了 2019
世界人工智能大会、第 12 届中国 R 会议。

所以我认为自己不是一个专业的程序员，而是一个编程爱好者。

</details>


我没有具体的动机就学习了 Rust 语言。当我逐渐发现这门语言越来越有趣，我开始搜集一些它的 [资料][rust-materials] ——
这是当时我所知最齐全的中文 Rust 资料清单 —— 其中的 Rust 官方组织相关的资料非常重要，有 [Reference]、[Nomicon]、[UCG] 等，它们是基础知识的来源。

我翻译过一些书籍、博客和文档，见 [此处][translations]，也会参与一些社区的讨论，比如中文社区飞书群、[rustcc] 论坛、[URLO] 论坛。

[trpl]: https://doc.rust-lang.org/book/
[rust-materials]: https://www.yuque.com/zhoujiping/programming/rust-materials
[Reference]: https://doc.rust-lang.org/reference/index.html
[Nomicon]: https://doc.rust-lang.org/stable/nomicon/index.html
[UCG]: https://github.com/rust-lang/unsafe-code-guidelines
[translations]: https://zjp-cn.github.io/translations/
[rustcc]: https://rustcc.cn/
[URLO]: https://users.rust-lang.org/
[RFCs]: https://github.com/rust-lang/rfcs

# Rust 用户论坛

Rust 官方的 [URLO] 用户论坛是一个学习 Rust 的很好的起点。
* 你可以在那搜索常见的 Rust 问题的各种回答；
* 当你找不到答案的时候，发一个帖子基本上能得到很好的解答；
* 当你有一些回答能力的时候，可以提供有价值的回复和帮助来提升自己；
* 经常逛一逛，也会在同一个问题的不同的视角。

在 2022-2023 年期间，我是 URLO 的活跃用户，常常在回答他人和阅读其他回复的过程中提高对 Rust 的认识。
当我在解释一个概念的时候，首先会寻找这个术语在官方资料里的出处。当指明和解释这个要点，就能够解答大多数入门问题。
但回答中高级问题通常需要丰富的 Rust 经验，很多时候经验的积累不仅仅通过自己编写代码，也可以通过集思广益来累积共识。

![](https://github.com/user-attachments/assets/3456f570-9a5d-4cee-8c3a-714116c6a811)

如果没有这个正向的社区，我认为我不会持续地学习 Rust 语言。感谢论坛的 [steffahn]、[quinedot]、[H2CO3]、[alice]、[scottmcm]、
[kpreid]、[kornel]、[yandros] 以及很多活跃用户对 URLO 论坛的建设，他们的独特视角和经验分享是社区的宝贵财富。因为很多时候，
Google 搜索 Rust 问题的结果就会指向 URLO 中的一篇帖子；而好的 AI 引擎肯定也从这些帖子中提取了精华给出高质量回答。

[steffahn]: https://users.rust-lang.org/u/steffahn
[quinedot]: https://users.rust-lang.org/u/quinedot
[H2CO3]: https://users.rust-lang.org/u/h2co3
[alice]: https://users.rust-lang.org/u/alice
[scottmcm]: https://users.rust-lang.org/u/scottmcm
[kpreid]: https://users.rust-lang.org/u/kpreid
[kornel]: https://users.rust-lang.org/u/kornel
[yandros]: https://users.rust-lang.org/u/yandros

# Rust Lang 项目

Rust 项目的主要仓库是 [rust-lang/rust]，其各个团队都在 [zulipchat] 上进行内部即时交流，但各自有设立专门的 Github 仓库以供追踪问题。

Rust 语言是一个庞大的项目，其组织结构记录在 [team] 仓库中。组织结构可能随着开发和管理需要而调整。但主体有几个部分：
* compiler team：包含 types、compiler-ops、rust-analyzer、miri 等子团队
* lang team：包含 lang-docs、opsem、spec 等子团队
* devtools team：包含 cargo、clippy、rustdoc、rustfmt、rustup、crates-io、docs-rs 等子团队
* libs team：包含 libs-api 等子团队
* infra team：包含 bootstrap、release、triagebot 等子团队
* ...

对于这些团队的具体介绍，见 compiler、bootstrap、infra 团队成员 [许杰友](https://github.com/jieyouxu) 撰写的
《[关于 Rust 项目、国际社区和发展动向](https://rustcc.cn/article?id=74964848-3def-4024-9e4b-b612303fffb0)》。

[rust-lang/rust]: https://github.com/rust-lang/rust
[zulipchat]: https://rust-lang.zulipchat.com/
[team]: https://github.com/rust-lang/team

我很少参与 Rust 语言项目，大部分时候只是作为长期的观察者。根据我的观察，Rust 项目的运作需要两方面的人员投入：全职人员和社区人员。

来自企业的全职从事 Rust 编译器开发的人员（以 AWS、FutureWei 为典型，聚集了一部分相当重要的 Rust 核心人物）。
他们具有最丰富的 Rust 开发和设计经验。但这几年来这些人员逐渐淡出了 Rust 语言项目，甚至最近一些顶级开发人员，比如开发异步新功能的
compiler-errors、专攻编译器优化的 nnethercote，正在寻找下一份全职 Rust 编译器工作，折射出些许危机。当这些中流砥柱力量减弱，则需要
漫长的时间才能弥补回来。

预防将来青黄不接的最好良剂只有培养更多的 Rust 编译器开发者，而第一步只能从积极贡献开始。北美和欧洲开发者对 Rust 语言的参与程度可能占据
80%。（不要引用这个数据，这只是我的一种感受，我没有进行统计）。中国开发者在其中占据一些力量，但比较微弱。语言交流是一个很大的阻碍，
但主要的因素是由兴趣驱动。

# Issues、PRs 和 RFCs

当你想知道一个功能或代码的实现过程或背景，那么 [rust-lang/rust] 组织的 issues 和 PRs 通常是最好的源头。

而 [RFCs] 记录了重要特性的设计思路，汇聚了许多人的想法和讨论结果，因此是非常重要的共识来源。

如果想了解新功能的设计，那么多去 [zulipchat] 上的各个频道逛逛。[语言][lang-issue]、[不安全语义][ucg-issue] 和
[编译器][compiler-issue] 的设计也会有相应的 issues。

Rust 语言的社区治理意识很强，具有相当高的透明度，因此完全可以参与到任何你感兴趣的讨论中贡献点子。

值得一提的是，[项目目标][rust-project-goals] 的 issues 承担了定期每月报告目标实施进展的功能。你可以在那看到
项目目标是如何持续推进或者遇到的阻碍是什么。如果你不知道项目目标的工作方式，可以查看我的 [Slides][slides-goals]。

[lang-issue]: https://github.com/rust-lang/lang-team/issues
[ucg-issue]: https://github.com/rust-lang/unsafe-code-guidelines/issues
[compiler-issue]: https://github.com/rust-lang/compiler-team/issues
[rust-project-goals]: https://github.com/rust-lang/rust-project-goals/issues
[slides-goals]: https://docs.qq.com/slide/DTHdPcE1XakJzZHBO

# 国内的 Rust 训练营

大概在 2021 年的时候，我就注意到了 [Rust 开源操作系统训练营][os-camp] 一年两次的训练营宣传，但从未想参加，直到 2024 年。

我没有抱着期待和目标参加 2024 年春季训练营，认为如果不再想了解，就随时停下。

操作系统这类底层知识离我很遥远，真正学习它的原理只有训练营第二阶段不到两个月的时间。

虽然我非常熟悉 Rust，但专业知识的匮乏不足以支撑我在第三阶段异步操作系统项目的短时间里做太多事情。

我写了一些 embassy 相关的 [笔记][os-notes]，然后试图在 rCore 当中添加 embassy 异步机制（失败告终）。我坚持到了最后，获得了通过。

在训练营最后一周，我非常不舍。于是联系了向老师，希望继续做一些事情。几周后，向老师找我做编写一个检查工具来提高操作系统组件库的质量。
我答应了，从而有了 [os-checker]，一个集成来自社区和学术检查工具的自动化 Rust 代码分析平台与质量监测系统。

os-checker 的主要开发时间有 4 个月，从后端检查逻辑到 [前端][os-checker-ui] 结果展示。后来逐渐集成了一些新的工具，并且根据检查诊断结果给 
OS 组件库提交一些修复。在这个过程中，我认识了秦博士和徐老师。

由此一年多的时间都在 Rust 检查代码这件事情上忙碌：除了 os-checker，我还为 verify-rust-std 编写了 [distributed-verification]，为
tag-std 编写了 [safety-tool] 并向 Rust 社区提交 [safety tags] RFC。这些都在我的 [周报] 中记录了。

[os-camp]: https://opencamp.cn/os2edu
[os-notes]: https://zjp-cn.github.io/os-notes/embassy-integrated-timers.html
[os-checker]: https://github.com/os-checker/os-checker
[os-checker-ui]: https://os-checker.github.io
[verify-rust-std]: https://github.com/model-checking/verify-rust-std
[distributed-verification]: https://github.com/os-checker/distributed-verification
[safety-tool]: https://github.com/Artisan-Lab/tag-std/blob/main/safety-tool
[safety tags]: https://github.com/rust-lang/rfcs/pull/3842
[周报]: https://os-checker.github.io/book/devlogs.html

---

无论是在校学生，或是工程师，亦或是 Rust 爱好者，都可以在一系列的 [Rust 训练营](https://opencamp.cn/camps) 里交到志同道合的朋友。

这些训练营主要聚焦于操作系统、嵌入式、AI、编译器等 Rust 领域，提供了以下机会：
* 参与者相互学习和交流
* 组织者寻找合作者（如实习、内推等）
* 组织者和参与者双方积累经验、传播知识
* 赛事型训练营提供较为丰厚的奖品

在我参加的那一届训练营里，就有两位文科专业的学生，一个是金融学，另一个是教育学。而理工科跨专业到计算机也早已不是特别稀奇的事情。

所以我特别鼓励那些非计算机专业且希望学习计算机知识的人，**保持热爱和好奇，勇敢地去探索你想了解的一切，就能欣赏到美丽的风景。** 

# 结语

在 Rust 语言 1.0 十周年之际，祝愿每个 Rustacean 在各自的人生轨迹上继续发光发热~
