---
layout: post
title: "如果你想构建一个基于ECS的GUI框架"
date: 2025-08-13
tags: [Rust, ECS, bevy_ui]
---

# 如果你想构建一个基于 ECS 的 GUI 框架

> `bevy_ui`未来面临的挑战与机遇
>
> Alice I. Cecile, Rose Peck | 2023-11-27
>
> 原文：[So you want to build an ECS-backed GUI framework](https://www.leafwing-studios.com/blog/ecs-gui-framework/)

如果你想用 Rust 构建一个 UI，有什么比实体-组件-系统（ECS）框架更好的工具来实现这一点呢？它是一种类型安全、新流行的状态管理解决方案，且最重要的是：它会是极快的（显然不需要基准测试）。

而[Bevy](https://bevyengine.org)正是在做这件事！实际上，它已经做了好几年了。为什么它没有主导竞争，没有俘获数百万人的心，也没有让[areweguiyet.rs](https://areweguiyet.com)过时？

虽然基于 ECS 的 GUI 可能并不常见，但有先例表明这并非*不可能*。[flecs](https://www.flecs.dev/flecs) [在这篇文章中实现了许多关键设想](https://www.flecs.dev/flecs/md_docs_FlecsScriptTutorial.html)，现有的实验如[`belly`](https://github.com/jkb0o/belly)、[`bevy_lunex`](https://github.com/bytestring-net/bevy-lunex)、[`bevy_ui_dsl`](https://github.com/Anti-Alias/bevy_ui_dsl)、[`cuicui_layout`](https://github.com/nicopap/cuicui_layout)和[`kayak_ui`](https://github.com/StarArawn/kayak_ui)展示了使用 Bevy ECS 的巨大潜力。甚至还有一个用 Javascript 编写的独立的 ECS 优先的 GUI 库，叫做[Polyphony](https://github.com/traffaillac/traffaillac.github.io/issues/1)！

事实证明，困扰[`bevy_ui`](https://docs.rs/bevy_ui/latest/bevy_ui)的大多数问题*并非是*由使用 ECS 或使用 Rust 的决定导致。它们都是些无聊、繁琐且让人沮丧的事情：因为编写 GUI 框架是一项有很多活动部件的繁杂工作。程序 Bug、样板代码以及缺失的功能特性扼杀了用户和开发者逐步改进系统的意愿。

但在我们深入细节之前，有一个重要的免责声明。Alice 是 Bevy 的维护者，但不是项目负责人，甚至不是 UI 领域的专家。Rose 是 [Foresight Spatial Labs](https://www.foresightmining.com) 的员工，在日常工作中使用 Bevy 和传统 web 框架（React）来构建 GUI 密集型应用程序。**这些观点纯粹是我们自己的，不是最终或官方的说法！**

这篇文章旨在记录*如何*构建 GUI 框架，*为什么*我们使用 ECS，以及我们需要*什么*来使`bevy_ui`真正优秀。在[太多地方](https://github.com/bevyengine/bevy/discussions/5604)已经有了很多重复的讨论，但很少有实质性的进展（除了[ickshonpe](https://github.com/bevyengine/bevy/pulls/ickshonpe)，你很棒）。说起“`bevy_ui`应该像我最喜欢的 UI 框架一样工作”很容易，但实际上将其转化为可行的设计，达成共识并*构建出*它要困难得多。

通过编写一份关于需求、愿景和进展的最新、全面、少用流行语的文档，我们希望 Bevy 社区能够团结起来解决`bevy_ui`今天面临的问题，明确排除某些可能性，并为关键的缺失部分提出可靠的设计。

不过谁知道呢？也许那是十年后了，而*你正*在读这篇文章，梦想着编写自己的基于 ECS 的 GUI 框架。

根据我非常疲惫的经验，关于`bevy_ui`的讨论通常会以三种常见方式偏离主题：

1. Bevy 应该只使用现有的 GUI 框架。
2. 一个适用于游戏和应用程序的单一 GUI 框架是不可能的。
3. 你不能在 ECS 中构建 GUI 框架。

## 为什么不直接使用`egui`？（或者`dioxus`、`tauri`、`iced`、`yew`...？）

已经有[很多](https://blog.logrocket.com/state-of-rust-gui-libraries)关于 Rust UI 框架的讨论。其中一些甚至得到了积极维护、有文档记录且基本功能完备！

社区为其中一些框架制作了[出色的互操作包](https://github.com/mvlabat/bevy_egui)，像 Foresight 这样的公司甚至使用这些第三方 GUI 框架制作了[复杂的产品级应用程序](https://github.com/bevyengine/bevy/discussions/5522)。

试图自己编写的 Bevy 准是“重复造轮子”(Not Invented Here)综合症的典型案例。当我们本可以使用现有解决方案编写即将到来的 Bevy 编辑器时，为什么要将稀缺的精力（和决策）投入到这个方向？毕竟，[我们可以直接与 Dioxus 进行官方合作](https://github.com/bevyengine/bevy/discussions/9538#discussioncomment-6984809)并跳过很多年的工作。

以下是我们认为 Bevy 不应该这样做的技术和社交原因：

1. 与引擎其余部分保持一致本身就有价值。
   1. 它为新用户提供了更简单、更一致的学习体验。
   2. 它使系统更易于维护。
   3. 它将所有更改保留在同一存储库中，消除了对依赖树进行仔细协调发布的需要。
   4. 对引擎其他区域的改进有益于 UI，反之亦然。Cart 认为许多挑战对 UI 来说并不独特，我们同意！
2. Bevy 已经为 GUI 库需要执行的许多核心任务提供了良好的解决方案。
   1. 渲染、状态管理、资产、输入、窗口、异步...
   2. 为什么要引入重复还略有不兼容的方式来完成这些任务？
3. 向外部 UI 框架发送数据并从中接收数据本来就是容易出错、复杂、难以维护而且样板代码繁重。
   1. 由于集成层和不匹配的数据模型的需要，这是一个不可避免的底线。
   2. 这对 UI 来说并不独特：[`bevy_rapier`](https://github.com/dimforge/bevy_rapier)在物理方面遇到类似问题（尽管它仍然是一个优秀的库）。
4. 跳出标准的“屏幕上的方框” UI 设计变得更加困难。
   1. 世界空间 UI 是游戏的关键特性：单位覆盖、VR 菜单、游戏世界里的电脑屏幕等...
   2. 游戏 UI [通常希望与游戏世界状态紧密集成并具有不同寻常的艺术效果](https://forum.unity.com/threads/i-look-forward-to-a-better-ui-system.1156304/)。
   3. 使用第三方解决方案编写自定义着色器来覆盖某些节点的行为要困难得多。
5. 现有的 Rust GUI 项目都没有很好地解决借用检查器真的讨厌图和并且真的讨厌分割可变性这一事实。
   1. 随着[关系](https://github.com/bevyengine/bevy/issues/3742)的加入，Bevy 承诺了一种独特而强大的处理 Rust 中图的方法。
   2. Bevy 的系统是一种灵活、无恐慌、快速且可靠的共享对世界状态可变访问的解决方案。这背后有很多黑魔法支撑，天哪，我们不想去写两遍。
6. 其他项目不是由 Bevy 项目运营的。
   1. 我们的目标可能会分歧：例如，[`egui`](https://www.egui.rs)故意专注于简单、快速构建的 UI，并在性能和可定制性上做出权衡来实现这一点。
   2. 变更变得更难协调：需要迁移 PR，我们无法快速添加编辑器需要的功能。
   3. 上游包可能会被遗弃（[再次](https://github.com/vislyhq/stretch/issues/86)）。如果 Bevy 计划存在几十年，UI 解决方案也会存在吗？
   4. 我们无法确保关键依赖的质量。
   5. 这给较小的第三方依赖带来了很大的维护压力，因为有这么大的客户在向他们提出请求。
7. 许多常用的第三方 GUI 库通过依赖 C、C++或 JavaScript 依赖项显著复杂化了 Bevy 的构建和分发过程。
8. 不想太苛刻，但很多现有的 Rust GUI 解决方案...就是不太好。
   1. 有很多过得去的选择，但它们都有不小的缺点。没有人真正脱颖而出成为明显的赢家。
   2. [areweguiyet.rs](https://areweguiyet.com)说“根基不深但种子已种下”是有原因的。
   3. 深层来说，我们都知道我们可以做得更好，我们也*应该*。
9. 偏好第三方 GUI 解决方案的用户无论如何都会使用它们。

我们会从其他 GUI 框架中学习吗？当然。我们会全盘正式采用它们吗？绝对不。

## 一个 GUI 框架统治所有？

关于`bevy_ui`讨论中的另一个常见的善意问题是"我们真的能用单一 UI 框架满足所有用户的需求吗"？

我见过的一些潜在分歧：

- [应用程序 UI vs 简单游戏 UI vs 复杂游戏 UI](https://github.com/bevyengine/bevy/issues/254#issuecomment-886235989)
- [喜欢 CSS 和 web 的人 vs 讨厌它的人](https://github.com/bevyengine/bevy/issues/254#issuecomment-850216295)
- [程序员友好的过程式 GUI vs 艺术家友好的资产驱动式 GUI](https://github.com/bevyengine/bevy/discussions/9538#discussioncomment-7388170)
- 即时 UI vs 保留模式 UI

我相信你能想到更多：分立很容易也很有趣！理论上，我们可以[效仿 Unity](https://forum.unity.com/threads/why-is-unity-creating-yet-another-ui-system.1148585/)，在 Bevy 中创建多个相争的 UI 框架。这将是[非常糟糕的](https://www.reddit.com/r/Unity3D/comments/no6j19/a_unity_rant_from_a_small_studio/)，因为：

1. 这对用户来说非常困惑。
2. 它分散了开发者的注意力。
3. 权衡选择哪种解决方案，对于用户来说并不总是清晰的。
4. 在两个冲突的解决方案之间迁移非常痛苦。
5. 在同一项目中使用多种解决方案从根本上说是不可行的。
6. 需要两倍的时间（如果你幸运的话）。

幸运的是，“确定具有不同需求的多个用户群体的要求”并不是 UI 独有的问题。我们在架构层面有很好的工具来管理这个问题：

- 这个问题是假设性的，实际上已经在 web 上解决了。
  - 我们不会争论 web UI 是否是曾经创造的最伟大的 UI 解决方案（它有很多明显的和不明显的缺陷）。
  - 但人们已经成功地使用 HTML/CSS/JavaScript 构建了几乎你能想到的任何类型的 UI：网页、代码编辑器、游戏（兼具浏览器和独立的）、CAD 应用程序、终端等等。有一个常见笑话就是关于怎么"未来一切都是 chrome"的（感谢[Electron](https://www.electronjs.org)）
  - 以防需要说明，web UI 堆栈*并不是*为这些用例设计的。可以说，它不是为其中任何一个设计的！
- 模块化：确保用户可以采用或放弃他们不喜欢的解决方案部分。
  - 组件、系统、插件和特性标志对此很有帮助！
  - 当前存在的第三方 UI 库，还并将继续存在。
- 可扩展性：确保内部结构可访问并且可被构建。
  - 公共组件和资源在这里非常有帮助。
  - 想象一个丰富的`bevy_ui`可互操作扩展库生态系统，所有这些都建立在我们的核心渲染、交互和布局范式之上。
- 抽象设计中的[渐进式披露](https://www.uxpin.com/studio/blog/what-is-progressive-disclosure)。
  - 小部件由节点构建。
  - 节点就是实体。
  - 在整个过程中，没有什么能阻止你在更低的层次上挂钩。

如果用户可以将相同的 ECS 和渲染工具应用于从像素艺术平台到单元着色视觉小说再到 PBR 竞技场射击游戏的所有内容，我们就可以制作一个足够灵活和愉快的 UI 解决方案，适合于每个人。

## ECS 中的 GUI：`bevy_ui`实际上是如何工作的？

解决了这些常见异议后，我们希望能够谈论如何实际构建我们的 UI 框架。让我们思考一下我们的实际产品需求，这样我们就能看到`bevy_ui`的不足之处。

对我们来说不幸的是，GUI 框架是极其复杂的野兽。有几个部分是基础的，以至于移除它们就会使整个系统瘫痪：

1. 存储节点树
   1. 几乎每个不平凡的 UI 范式都有一个或多个嵌套的元素树
   2. “节点”是这些元素之一：最小的不可分割的 UI 原子
   3. 你需要将这些数据存储在某处！
   4. 在`bevy_ui`中，这存储在[`World`](https://docs.rs/bevy/0.12.0/bevy/ecs/world/struct.World.html)中：每个节点都是一个带有[`Node`](https://docs.rs/bevy/0.12.0/bevy/ui/struct.Node.html)组件的实体
   5. UI 实体使用[`Parent`](https://docs.rs/bevy/0.12.0/bevy/hierarchy/struct.Parent.html)和[`Children`](https://docs.rs/bevy/0.12.0/bevy/hierarchy/struct.Children.html)组件连接在一起
2. 布局
   1. 一旦你有了节点集合，你希望能够描述它们在屏幕上的位置。
   2. 简单地指定绝对大小和位置不是很健壮：当节点被添加/删除或屏幕大小改变时会出问题。
   3. 在`bevy_ui`中，这是通过[Style](https://docs.rs/bevy/0.12.0/bevy/ui/struct.Style.html)组件指定的（为这个名称责怪 CSS，抱歉）。
   4. `bevy_ui`使用[taffy](https://github.com/dioxuslabs/taffy)（Alice 帮助维护！）：它支持[`flexbox`](https://css-tricks.com/snippets/css/a-guide-to-flexbox/)和[`css-grid`](https://css-tricks.com/snippets/css/complete-guide-grid/)布局策略
   5. 如果你不依赖 Web 布局算法，[`morphorm`](https://github.com/vizia/morphorm)（在我们看来）是更好的选择
3. 输入
   1. 收集用户输入，包括键盘按键、鼠标点击、鼠标移动、触摸屏点击、游戏手柄输入等
   2. 通常与“拾取”配对：根据位置确定指针事件关联的元素
   3. 理想情况下为这一点构建一些不错的抽象，以涵盖悬停和按下、释放和长按按钮等
   4. `bevy_ui`依赖于`bevy_input`，后者又从[`winit`](https://github.com/rust-windowing/winit)和[`gilrs`](https://docs.rs/gilrs/latest/gilrs/)获取数据
4. 文本
   1. 将字符串转换为我们可以绘制在屏幕上的像素
   2. 在包含它的节点边界内布局文本
   3. 确切的像素对渲染很重要，但其大小对节点布局的输入也很重要
   4. `bevy_ui`目前使用[`glyph_brush`](https://crates.io/crates/glyph_brush)
   5. [`cosmic-text`](https://github.com/pop-os/cosmic-text)对非拉丁文字有更好的造形支持
5. 窗口管理
   1. 实际创建一个窗口（或更多）来绘制你的 UI
   2. bevy 使用`winit`，你也应该这样做！
6. 渲染
   1. 获取 UI 元素并将其绘制到用户的屏幕上
   2. Bevy 在这里使用[`bevy_render`](https://docs.rs/bevy_render/0.12.0/bevy_render/)，也因此用的是[`wgpu`](https://docs.rs/wgpu/0.12.0/wgpu/index.html)
   3. 如果你在构建自己的 Rust GUI 框架，请查看[`vello`](https://github.com/linebender/vello)！
7. 状态管理
   1. 跟踪 UI 持久功能的状态
   2. 填充文本、单选按钮、动画进度、菜单是否打开或关闭、深/浅色模式等
   3. 在`bevy_ui`中，状态存储为实体上的组件（或很少情况下，作为全局资源）。这非常有效！
8. 数据传输
   1. 在 UI 和其他数据存储之间相互传输数据
   2. 在 Bevy 的上下文中，“其他数据存储”是存储所有游戏/应用程序状态的 ECS `World`
   3. 数据绑定是用于自动化此过程的抽象：自动和细粒度地传输变更
   4. 目前，`bevy_ui`使用系统从`World`其它地方来回发送数据

在此基础上，你可能还想添加：

1. 导航
   1. 以原则性的离散方式浏览 GUI 菜单："tab"是常见的按键绑定
   2. 对键盘和游戏手柄都非常有用
   3. 传统应用程序重要的可访问特性
   4. `bevy_ui`对此没有第一手解决方案
2. 样式
   1. 小部件和节点有很多主要是装饰性的属性。
   2. 我们希望确保应用程序具有一致的外观和感觉，并能够快速切换。
   3. 对于应用程序（特别是移动应用程序），[*原生*外观和感觉](https://www.quora.com/What-does-native-UI-mean)非常理想
   4. 这可能采取以下形式：
      1. 层叠继承（像 CSS 中一样）
      2. 选择器（像 CSS 中一样，或者像你在`bevy_ui`中使用查询一样）
      3. 全局主题如浅色和深色模式
      4. 小部件特定样式
   5. 样式通常需要有可预测的组合规则：当多个样式同时影响一个元素时会发生什么？
   6. `bevy_ui`目前没有任何第一方抽象。
3. 可组合、可重用小部件的抽象
   1. 即使是简单的小部件类型（单选按钮、文本输入框等）也相当复杂！
   2. 用户应该能够编写一次，然后在他们的项目中重复使用，提高开发速度和 UI 一致性
   3. 小部件可能由一个或多个节点/元素组成
   4. 每个小部件的节点数可以动态变化：想想一个不断增长的待办事项列表
   5. 需要能够接受参数来更改其内容或行为。例如，创建一个可自定义文本的可重用按钮。
   6. `bevy_ui`目前使用[`Bundle`](https://docs.rs/bevy/0.12.0/bevy/ecs/bundle/trait.Bundle.html)类型来实现这一点，但由于它无法处理多个节点，所以效果很差
4. 操作抽象
   1. 撤销-重做
   2. 可重新绑定的快捷键
   3. 命令面板
   4. `bevy_ui`没有第一手的解决方案，甚至第三方解决方案也不成熟（抱歉！）
5. 可访问性
   1. 为你的 UI 创建并暴露一个对机器友好的 API：读取状态、改变渲染/显示、发送输入并检测当这些输入改变时发生的事情
   2. 通常与键盘导航挂钩
   3. API 被如屏幕阅读器这类工具使用，以提供满足残障用户需求的替代用户界面
   4. `bevy_a11y`与[`accesskit`](https://github.com/AccessKit/accesskit)挂钩，你的 GUI 框架也应该这样做
   5. 关于可访问性有很多潜在可以讨论的内容，但可惜我们没有足够的字数在这里讨论
6. 本地化
   1. 有不止一种语言：你需要一种方法来交换 UI 元素（特别是文本）以满足偏好不同语言的用户需求
   2. 一些语言从右到左阅读而不是从左到右，如果不考虑这一点，某些 UI 设计往往会出现颠倒问题
   3. 图标和表情符号在不同地方也有不同的文化含义
   4. 说真的，直接使用[`fluent`](https://crates.io/crates/fluent)
7. 资源管理
   1. UI 经常使用预渲染的图像或图标进行视觉展示，特别是在游戏中
   2. 你需要自定义装饰和图标，或者按其自身规格显示图像和视频
   3. `bevy_ui`使用[`bevy_asset`](https://crates.io/crates/bevy_asset)来实现这一点！
8. 动画
   1. 小动画，特别是当 UI 元素改变时，可以极大地提高 UI 的精致感和生动感
   2. 折叠/展开上下文菜单、滑动抽屉、旋转加载图标、淡入/淡出等
   3. `bevy_ui`理论上与[`bevy_animation`](https://crates.io/crates/bevy_animation)集成来实现这一点，但集成还不完善
9. 调试工具
   1. 在 UI 渲染后快速检查和修改 UI 树
   2. 这对捕捉错误和调整样式非常有用
   3. `bevy_ui`对此没有解决方案，但[`bevy_inspector_egui`](https://github.com/jakobhellermann/bevy-inspector-egui)很不错
10. UI 序列化（内存对象到文件）和反序列化（文件到内存对象）
    1. 如果我们可以基于存储在文件中的定义构建 UI，我们可以：
       1. 使外部工具（如游戏编辑器）更容易构建 UI
       2. 使 UI 更容易供末端用户自定义（想想 Greasemonkey 和游戏模组）
       3. 更容易构建调试工具
       4. 减少编译时间：只需热重载资源
       5. 允许完全控制用于定义对象的格式和语法
       6. 提供更好的模块化工具来创建[更高层抽象](https://github.com/bevyengine/bevy/issues/3877)以及无需修改源代码的自动迁移
    2. 在游戏中，这被称为“数据驱动”方法
    3. `bevy_ui`目前使用场景（来自[`bevy_scene`](https://docs.rs/bevy_scene/0.12.0/bevy_scene/)）来实现这一点
11. 异步任务
    1. 有时，UI 触发的工作需要相当长的时间才能完成
    2. 你不希望程序在此期间冻结！
    3. 在`bevy_ui`中，这使用[`bevy_tasks`](https://docs.rs/bevy_tasks/0.12.0/bevy_tasks/)来实现

## 为什么`bevy_ui`很糟糕？

通过连接到 Bevy 这个功能齐全（但尚未完成）的游戏引擎，`bevy_ui`实际上在大多数这些领域都有初步的解决方案！

那么为什么它被普遍认为更像是"Bavy"而不是"Bevy"？[使用过](https://github.com/bevyengine/bevy/discussions/2235)、开发过并听取过使用`bevy_ui`的用户反馈后，以下是截至 Bevy 0.12 的关键问题。这些问题大致按对用户体验的主观影响程度排序。

1. 生成具有大量自定义属性的实体需要大量样板代码。
   1. 无尽的嵌套和到处都是的`..Default::default()`。
   2. [在处理以树形结构排列的多个实体时](https://github.com/bevyengine/bevy/blob/v0.12.0/examples/ui/ui.rs)，这变得更糟。如前所述，你不能为此使用 bundles。
   3. 数据驱动的工作流程没有被广泛使用，因为 Bevy 的场景[冗长且文档不足](https://github.com/bevyengine/bevy/discussions/9538)。
2. Bevy 需要一个真正的小部件抽象。
   1. 并非所有小部件都能有意义地表示为单个实体。
   2. Bevy 提供的预构建小部件很少：我们只有按钮和图像。
   3. 由于缺乏标准化抽象，即使是[添加最简单、最有用的小部件也存在争议并陷入僵局](https://github.com/bevyengine/bevy/pull/7116)。（需要明确的是，这不是审阅者或作者的错。）
3. 在调度中使用系统并不那么适用于数据绑定。
   1. UI 行为几乎总是一次性的或非常稀疏的。
   2. 从 UI 启动的任务通常要么很小，要么是将工作扔进异步池。
   3. 我们真的希望能够引用单个特定实体及其父级和子级。
      1. 解决这个问题需要创建数十个标记组件：几乎每个按钮、文本框、图像、容器等都需要一个。
   4. 99%的时间里，这些系统不会做任何工作。这会浪费时间，因为调度必须不断轮询查看是否需要做任何事情。
4. 在`bevy_ecs`中管理和遍历层次结构（向上和向下）真的很糟糕。
   1. [关系](https://github.com/bevyengine/bevy/issues/3742)来得不够快。
5. Bevy 的 UI 输入处理非常原始。
   1. 处理指针输入的[`Interaction`](https://docs.rs/bevy/0.12.0/bevy/ui/enum.Interaction.html)组件[过于有限](https://github.com/bevyengine/bevy/issues/7371)。
   2. 移动端的[多点触控支持](https://github.com/bevyengine/bevy/issues/15)[相当有限](https://github.com/bevyengine/bevy/issues/2333)。
   3. 目前缺少[键盘和游戏手柄导航](https://github.com/bevyengine/rfcs/pull/41)。
   4. 没有[操作抽象](https://github.com/leafwing-studios/leafwing-input-manager)的第一手支持来实现可配置的键绑定。
   5. Bevy 的拾取支持非常简单，不容易扩展到非矩形元素或世界空间中的元素。（有请[`bevy_mod_picking`](https://crates.io/crates/bevy_mod_picking)...）
6. Flexbox（以及在较小程度上的 CSS Grid）[很难学习，有令人沮丧的边缘情况，和糟糕的 API](https://elk.zone/mastodon.gamedev.place/@alice_i_cecile/111349519044271857)。*你*能解释一下`flex-basis`是做什么的吗？
7. 由于[刚刚修复的错误](https://github.com/bevyengine/bevy/pull/10537)，`bevy_ui`中的字体渲染有时相当丑陋。
8. Bevy 缺少样式抽象。
   1. 如今可以完成实现：只需修改组件！
9. 向`bevy_ui`添加非凡的视觉效果太难了。
   1. 我们缺少[圆角](https://github.com/bevyengine/bevy/pull/8973)：对于外观良好的代码定义 UI 来说是必需的。（它们目前在 UI 中非常流行。我们可以等几年让它们过时，但几年后它们又会回来。）
   2. 我们也没有投影，但没人关心。
   3. 我们缺少[九宫格支持](https://github.com/bevyengine/bevy/pull/10588)：对于外观良好但灵活的资源定义 UI 来说是必需的。
   4. 在 Bevy 0.12 的 UI 材质之前，没有能让你在`bevy_ui`中添加自己的渲染抽象的安全出口。
10. 纯代码构建 UI 或手动编写场景文件可能很痛苦且容易出错：可视化编辑器会很棒。
11. [世界空间 UI](https://github.com/bevyengine/bevy/issues/5476)支持非常差，并且使用[完全不同的工具集](https://github.com/bevyengine/bevy/blob/v0.12.0/examples/2d/text2d.rs)。
    1. 这对游戏（血条、单位框架）来说是必需的，但对 GIS 或 CAD 应用程序中的标记和标签等也很有用。
12. `bevy_ui`没有一流的动画支持。
13. `bevy_ui`节点都有[`Transform`](https://docs.rs/bevy/0.12.0/bevy/transform/components/struct.Transform.html)和[`GlobalTransform`](https://docs.rs/bevy/0.12.0/bevy/transform/components/struct.GlobalTransform.html)组件，但你不许触碰它们。
14. 在 Bevy 中处理异步任务的体验令人沮丧：需要太多手动跟踪和轮询任务。

在这些问题中，只有 1（实体生成样板）、2（小部件抽象）、3（系统不适合回调）和 4（层次结构痛苦）是由我们选择使用 ECS 架构引起的。其余这些都是标准的 GUI 问题：无论你使用什么范式都需要解决。而且至关重要的是，*每一个与 ECS 相关的问题*都是 Bevy 应该为其他用例修复的：

1. 生成自定义实体（特别是实体组装）对普通游戏代码来说很糟糕，场景也不够好。例如，生成一个玩家及其所有武器。
2. Bevy 缺少能覆盖多实体层次结构的代码定义抽象级别：bundles 不够好。
3. 一次性系统对各种定制的复杂逻辑都很有用，我们需要开发有效的使用模式。
4. Bevy 处理层次结构的方法从根本上说是缓慢、脆弱且难以使用的。关系需要成为一流原语。

ECS 和 GUI 之间没有根本的阻抗不匹配或架构不兼容。`bevy_ui`不是一个根本上有缺陷的概念，它的 ECS 基础[只是还不够好](https://elk.zone/mastodon.gamedev.place/@alice_i_cecile/111349511360164259)。

## `bevy_ui`的前进之路

让`bevy_ui`变得真正优秀还有很长的路要走，但我们可以一步一步地走。仍有一些大的开放性问题，并且核心组件即将重写，但这并*不*意味着所有的`bevy_ui`都将被彻底推翻。GUI 框架涉及大量复杂且大多独立的子组件：一个领域的改进不会因其他领域的重写而失效！

我们可以将要完成的工作分为三类：**直白的**、**有争议的**和**研究性的**。

直白的任务只需要完成。它们可能容易也可能不容易，但在如何或是否应该完成方面不应该有太多争议。目前包括：

1. 审查并合并[对圆角的支持](https://github.com/bevyengine/bevy/pull/8973)。
2. 审查并合并[九宫格支持](https://github.com/bevyengine/bevy/pull/10588)。
3. 审查并合并[用于插值和混合的 Animatable 特性](https://github.com/bevyengine/bevy/pull/4482)。
4. 审查并合并 [winit 更新](https://github.com/bevyengine/bevy/pull/10702)，这可能会修复各种小错误和限制。
5. 完成、审查并合并 [cosmic-text 迁移](https://github.com/bevyengine/bevy/pull/10193)，这将解锁系统字体和复杂的字体塑形功能。
6. 添加对世界空间 UI 的支持，从审查和合并[相机驱动的 UI PR](https://github.com/bevyengine/bevy/pull/10559)开始。
7. 添加对变更 [UI 不透明度](https://github.com/bevyengine/bevy/issues/6956)的支持。
8. 为`bevy_scene`添加更多文档、示例和测试，使其更容易扩展和学习。
9. 为[在 Bevy 中处理多点触控输入](https://github.com/bevyengine/bevy/issues/15)添加更好的示例和功能。
10. 改善在 Bevy 中处理异步任务的体验。
11. 向`taffy`添加[Morphorm](https://github.com/DioxusLabs/taffy/issues/308)和/或[`cuicui_layout`](https://cuicui.nicopap.ch/layout/index.html)布局策略，并在 Bevy 中暴露出来。
12. 添加数十个小部件（受制于对良好小部件抽象的共识）。

有争议的任务是我们有明确认识和广泛共识的，但具有重要的架构影响和权衡：

1. 创建一个样式抽象，通过修改组件值来工作。
   1. Alice 为这个工作方式写了一个非常老的[RFC](https://github.com/bevyengine/rfcs/pull/1)，[`bevy_kot`](https://github.com/UkoeHB/bevy_kot)有一个样式层叠方法，viridia 的[`quill`](https://github.com/viridia/quill)实验也有一个很好的提案。
2. 以[`bevy_fluent`](https://github.com/kgv/bevy_fluent)上游，将其置于 Bevy 项目的保护下进行长期维护。
3. 添加对[键盘和游戏手柄导航](https://github.com/bevyengine/rfcs/pull/41)的支持，并将其集成到`bevy_a11y`中
4. 添加一个[处理指针事件和状态的适当抽象](https://github.com/bevyengine/bevy/issues/7371)。
5. 完善并实现[Cart 的`bsn`提案](https://github.com/bevyengine/bevy/discussions/9538)以改善场景的可用性。
   1. 这受到现有工作启发并与之密切相关，如[`cuicui`](https://lib.rs/crates/cuicui_layout)、[`belly`](https://github.com/jkb0o/belly)和[`polako`](https://github.com/polako-rs/polako)。
6. 添加一个[类似 bundles 的抽象](https://github.com/bevyengine/bevy/issues/2565)，但适用于多实体层次组装。
   1. 添加一个 bsn!宏，使实例化 Bevy 实体特别是实体层次结构时减少样板代码。
   2. 添加一种通过派生宏从结构体生成这些的方法。
   3. 先前的技术包括[`bevy_proto`](https://docs.rs/bevy_proto/0.12.0/bevy_proto/)和[`moonshine-spawn`](https://crates.io/crates/moonshine-spawn)。
7. 添加[插值颜色](https://github.com/bevyengine/bevy/issues/1402)的方法以促进 UI 动画。
8. 创建一个[特定于 UI 的变换类型](https://github.com/bevyengine/bevy/issues/7876)，以实现更快的布局和更清晰、更类型安全的 API。
9. 向`taffy`添加在单个树中混合布局策略的支持。
10. 按照[`bevy_easings`](https://github.com/vleue/bevy_easings)和[`bevy_tweening`](https://github.com/djeedai/bevy_tweening)的方式，添加对缓动/补间动画的支持。
11. 上游[`leafwing-input-manager`](https://github.com/leafwing-studios/leafwing-input-manager)以创建键绑定抽象。
12. 上游[`bevy_mod_picking`](https://github.com/aevyrie/bevy_mod_picking)以解锁高性能、灵活的元素选择。
13. 实现[关系](https://github.com/bevyengine/bevy/issues/3742)，并在`bevy_ui`中使用它们。

研究性任务将需要大量的设计专业知识，仔细考虑截然不同的提案，并且可能没有明确的要求：

1. 定义并实现一个[标准小部件抽象](https://github.com/bevyengine/bevy/discussions/5604)。这应该是：
   1. 可组合的：小部件可以与其他小部件组合创建新的小部件类型
   2. 灵活的：我们应该能够使用此抽象支持从按钮到列表到选项卡视图的所有内容
   3. 可配置的：用户可以在不创建自己类型的情况下更改小部件工作的重要属性
   4. 可以映射到一个 Bevy 实体或多个实体，以使用普通系统动态更新的方式
   5. 与 Bevy 场景之间相互可序列化
2. 弄清楚我们如何处理 UI 行为（和数据绑定）以避免仅使用系统涉及的问题
   1. 这是 Alice 创建[一次性系统](https://github.com/bevyengine/bevy/blob/v0.12.0/examples/ecs/one_shot_systems.rs)的最初动机
   2. [事件冒泡](https://github.com/aevyrie/bevy_eventlistener)和[各种杂项](https://github.com/viridia/quill)以及[各种响应式 UI](https://crates.io/crates/futures-signals) 实验似乎是有趣的潜在工具。
   3. [Raph Levien 关于 Xilem 的文章](https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html)很有趣，尽管并不总是直接适用
   4. 数据模型是这里的关键挑战：很容易在所有权问题上遇到麻烦
3. 弄清楚如何将数据绑定逻辑集成到 Bevy 场景中
   1. [`Callback`即`Asset` PR](https://github.com/bevyengine/bevy/pull/10711) 看起来很有前景
   2. [Vultix 提议](https://github.com/bevyengine/bevy/discussions/9538#discussioncomment-7667372)了一种使用`.bsn`文件定义的语法和策略。
4. 构建 [Bevy 编辑器](https://github.com/orgs/bevyengine/projects/12)，并添加使用它构建 GUI 场景的支持
   1. 这里存在某种循环依赖：`bevy_ui`越好，构建它就越容易

显然，还有很多工作要做！但关键的是，没有一项工作是*不可能的*。如果我们（Bevy 开发者社区）能够团结起来，稳步地逐一解决这些问题，我们（Alice 和 Rose）真诚地相信`bevy_ui`总有一天会达到我们对引擎其他部分所期望的质量、灵活性和人体工程学标准。
