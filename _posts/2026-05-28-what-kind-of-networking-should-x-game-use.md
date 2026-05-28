---
layout: post
title: "某个游戏应该使用哪种网络方案？"
date: 2026-05-28
tags: [netcode, replication]
original: https://github.com/bevyengine/bevy/discussions/8675
---

> [erlend-sh](https://github.com/bevyengine/bevy/discussions/8675#discussion-5232891) 于 2023 年 5 月 25 日

<details markdown="1">
<summary>原文</summary>

> [erlend-sh](https://github.com/bevyengine/bevy/discussions/8675#discussion-5232891) on May 25, 2023

</details>

我又一次发现自己在给做多人游戏、来问最佳实践的人贴上这篇非常棒的网络解释。
它被放在 Discord 某个无法被索引的线索里，实在埋没了它的价值，所以我打算在这里重发，希望它很快能进入正式文档页面。

<details markdown="1">
<summary>原文</summary>

Once again I've found myself linking to this excellent networking explainer when someone making a multiplayer game showed up with questions about best practice. Having it stored on an unindexable Discord server within a thread does it a disservice, so I'm gonna repost it here in hopes that it can soon make its way into a documentation page.

</details>

来自 Joy @maniwani：

<details markdown="1">
<summary>原文</summary>

From Joy @maniwani:

</details>

# 什么类型的网络方案适合 X 游戏？

<details markdown="1">
<summary>原文</summary>

# What kind of networking should X game use?

</details>

我认为，回答下面这三个问题，基本就能帮你做出选择。

<details markdown="1">
<summary>原文</summary>

IMO answering these three questions can decide for you.

</details>

## 1. 确定性，还是权威性？

<details markdown="1">
<summary>原文</summary>

## 1. Deterministic or authoritative?

</details>

> 想一想：
>
> - 模拟有多复杂？需要同步复制多少个实体？
> - 是否存在隐藏信息？在准许之前没有人能够访问到它，这件事重要吗？
> - 你所有想使用的库中会引入非确定性吗？

<details markdown="1">
<summary>原文</summary>

> Think about:
>
> - How complex is the simulation? How many entities need to be replicated?
> - Is there hidden information? Is it important that nobody can access it before they're supposed to?
> - Do any of the libraries you want to use introduce non-determinism?

</details>

在**确定性复制**中，客户端（直接或间接地）交换输入，并各自独立地模拟世界。

<details markdown="1">
<summary>原文</summary>

With **deterministic replication**, clients exchange inputs (directly or indirectly) and independently simulate the world.

</details>

在**权威性复制**中，游戏状态中的每个元素都有一个属于更新源的拥有者。
通常服务器拥有一切的（“服务器权威”），客户端只是向它发送输入；但所有权也可以分散到客户端（“客户端权威”），这种情况下客户端也会发送状态。

<details markdown="1">
<summary>原文</summary>

With **authoritative replication**, each element of the game state has an owner who is its source of updates. Usually, the server owns everything ("server auth") and clients only send inputs to it, but ownership can be distributed among clients ("client auth"), and then they can send state as well.

</details>

我们把它们简称为 DR 和 AR。

<details markdown="1">
<summary>原文</summary>

Let's call 'em DR and AR.

</details>

如果你在做 RTS 游戏，优先考虑 DR。
玩家可能一次控制几百个单位。AR 需要大量数据包来交换所有单位的信息，但是带宽往往是有限的。DR 只需要交换输入，无论实体数量多少，所以 DR 通常更合适具有大量网络实体的游戏。（当然也[*确实有*][5]使用 AR 的 RTS 游戏。）

<details markdown="1">
<summary>原文</summary>

If you're making an RTS game, consider DR. Players might control hundreds of units at once. AR would need lots of packets to exchange information about all of them, but bandwidth is often limited. DR only has to exchange inputs, no matter the number of entities, so DR is generally better for games with lots of networked entities. (There [*are*][5] RTS games that use AR though.)

</details>

这种经验法则也有一些例外：具有超多数量玩家（在同一局里）的游戏，或者具有超大规模开放环境的游戏。
在 DR 下，每台设备通常要模拟整个世界，我不认为谁能把整个提瓦特大陆（注：原神游戏世界）稳定跑到 60 FPS。
AR 则更易于筛选每个客户端需要发送和模拟的数据,使其成为这些场景中更好的选择。

<details markdown="1">
<summary>原文</summary>

Some exceptions to that heuristic would be games with a massive number of players (in the same session) and games with massive, open-world environments. With DR, each device generally has to simulate the entire world. I don't think anyone can run all of Teyvat at 60 FPS. AR more easily allows for cherry-picking what to send to and simulate on each client, making it the better choice in those scenarios.

</details>

同理，AR 通常也更适合需要隐藏信息的游戏。
如果你正在开发一款在线扑克游戏，那么如果你能够通过程序在其他玩家出牌之前就泄露他们的手牌，那就将是相当糟糕了。
此时 DR 大概是个错误的选择，因为每个玩家都会将整个游戏状态保存在内存中。
不过话虽如此，如果你谨慎地将隐藏信息仅作为输入，并且只在准备出牌时才发送，那么 DR 也是可行的。

<details markdown="1">
<summary>原文</summary>

For the same reasons, AR is generally better for games that require hidden information. If you're making an online poker game, it'd be really bad if you could use a program to leak another player's hand before they reveal it. DR is probably the wrong choice here, since every player will have the entire game state in memory. That said, DR *could* work if you are careful to send hidden information as input only, and only when it's ready to be revealed.

</details>

## 确定性很难保证

<details markdown="1">
<summary>原文</summary>

## Determinism is hard to guarantee

</details>

实践中，在比较 DR 和 AR 之前，你可能得先问自己：DR 到底是否可行。

<details markdown="1">
<summary>原文</summary>

In practice, before you compare DR and AR, you should probably ask yourself if DR is even a feasible option.

</details>

### 依赖（Dependencies）

要做到确定性，你在模拟中调用的函数就必须在相同参数下始终返回相同结果。
理想情况下，这些函数也是纯函数/无状态函数，即：他们所有参数全都是显式传入的。
有状态的也是可以的，**前提是**你能够访问、修改并克隆那些状态。

<details markdown="1">
<summary>原文</summary>

To be deterministic, the functions you use in your simulation should always return the same results given the same arguments. Ideally, those functions are also pure/stateless, i.e. all of their arguments are explict. Being stateful is OK **if** you can access, modify, and clone that state.

</details>

但很不幸，你往往做不到。

<details markdown="1">
<summary>原文</summary>

Unfortunately, oftentimes you can't.

</details>

例如，很多物理库（除了用户提供的碰撞体、刚体和关节之外）还拥有用于加速计算的内部数据结构，比如用于碰撞检测的 BVH，以及用于约束动力学的约束岛（islands）。
PhysX、Bullet 和 Box2D 都很流行，但它们（至少是集成它们的游戏引擎）似乎都不暴露这些内部状态。
这使得它们**在你还需要做预测时**变得与 DR 不兼容，因为你无法正确执行回滚。（这对 AR 来说也同样不方便。）

<details markdown="1">
<summary>原文</summary>

Many physics libraries, for example, have internal data structures to speed things up (alongside the colliders, bodies, and joints supplied by the user), like BVHs for the collision detection and islands for the constrained dynamics. PhysX, Bullet, and Box2D are some of the most popular libraries, yet none of them (or at least the game engines that integrate them) seem to expose their internal state, which makes them incompatible with DR *when you also want prediction* since you can't rollback properly. (This is inconvenient for AR as well.)

</details>

### 数学（Math）

浮点运算是臭名昭著的非确定性来源。

<details markdown="1">
<summary>原文</summary>

Floating-point math is a notorious source of non-determinism.

</details>

- 浮点运算会对结果做舍入（因为精度有限），所以浮点运算不满足结合律。`(a + b) + c` 与 `a + (b + c)` 并不等价。
- IEEE-754 精确定义了浮点数的二进制表示，以及加减乘除应如何计算，因此这些运算（应当）在各平台一致。
  *但是*，它并没有约束许多其他操作（如 sqrt、sin、cos、tan、exp、log 等），所以这些操作的硬件实现可以不同。
- 编译器在 `NaN` 值上基本[完全][3][不][1][遵守][2] IEEE-754。

<details markdown="1">
<summary>原文</summary>

- Floating-point math operations round their results (because precision is finite), so floating-point math is non-associative. `(a + b) + c` and `a + (b + c)` are not equivalent.
- IEEE-754 exactly defines the binary representation of floating-point numbers and how addition, subtraction, multiplication, and division should be calculated, so they (should) match on all platforms. *However*, it does not constrain a bunch of other operations (e.g. sqrt, sin, cos, tan, exp, log, etc.), so hardware implementations of those operations can differ.
- Compilers basically [don't][1] [comply][2] [at all][3] with IEEE-754 on `NaN` values.

</details>

不过也有好消息！这些问题（显然）是有办法处理的。
如果你能做到：

<details markdown="1">
<summary>原文</summary>

But good news! It is possible (apparently) to address those issues. If you can:

</details>

- 确保编译器[无法干扰][4]你的浮点表达式。
- 确保舍入模式一致。
- 尽量只使用加、减、乘、除。
- 像 sqrt、sin、log 这类超越函数，尽量从 `libm`（分段多项式近似）获取，而不是从 `std`（硬件函数指令）获取，以得到一致结果。
- 永远不要依赖 NaN 的精确比特位。

<details markdown="1">
<summary>原文</summary>

- Make sure the compiler [can't mess with][4] your floating-point expressions.
- Make sure the rounding mode is consistent.
- Mostly stick to add, subtract, multiply, and divide.
- Source any transcendental functions (e.g. sqrt, sin, log, etc.) from a library like `libm` (piecewise polynomial approximations) instead of `std` (hardware intrinsics) to get consistent results.
- Never rely on the exact bits in a NaN value.

</details>

之后你应该就能得到跨平台一致的结果。

<details markdown="1">
<summary>原文</summary>

Then you should get results that are consistent cross-platform.

</details>

## 2. 要不要做预测？做多少？

<details markdown="1">
<summary>原文</summary>

## 2. To predict or not? And how much?

</details>

> 想一想：
>
> - 这个游戏是实时运行的吗？
> - 错误预测造成的视觉异常可以接受吗？
> - 延迟补偿造成的视觉异常可以接受吗？
> - 客户端能负担得起模拟什么？

<details markdown="1">
<summary>原文</summary>

> Think about:
>
> - Does the game run in real-time?
> - Is visual weirdness from mispredictions OK?
> - Is visual weirdness from lag compensation OK?
> - What can clients afford to simulate?

</details>

没人喜欢延迟，所以如果你的游戏是实时的，游戏客户端就应该尽快尝试给玩家提供输入反馈。
预测（回滚）是掩盖网络往返时间最常见的方法，但并不是*唯一*方法。

<details markdown="1">
<summary>原文</summary>

Nobody likes lag, so if your game runs in real-time, your game clients should try to give players input feedback as quickly as possible. Prediction (rollback) is the most common way to hide the network round-trip time. Not the *only* way though.

</details>

一方面，格斗游戏玩家热爱回滚，并希望所有游戏都支持。 另一方面，错误预测可能会看起来像游戏故障。
RTS 游戏通常在同步锁定（不预测）下运行，但会加入人为输入延迟，让所有玩家都有足够时间交换输入而不至于停顿（也就是“基于延迟的网络代码”），再用动画去掩盖这段延迟。

<details markdown="1">
<summary>原文</summary>

On one hand, fighting game players love rollback and want it in every game. On the other hand, mispredictions can look glitchy. RTS games often run in lockstep (no prediction) but add artificial input delay to give all players just enough time to exchange inputs without stalling (hence "delay-based netcode"), then they try to hide that delay with animations.

</details>

DR 客户端通常是全预测或者完全不预测。
AR 客户端更常见的是预测一部分但不预测另一部分，再由服务器用“延迟补偿”来解决预测实体与非预测实体之间的交互。
这在 FPS 游戏里非常常见，但也会导致那些[“这都能判定命中？！”][6]的时刻。

<details markdown="1">
<summary>原文</summary>

DR clients mostly predict everything or predict nothing. AR clients more commonly predict some things and not others, with the server doing "lag compensation" to resolve interactions between predicted and non-predicted entities. Very common in FPS games but causes those ["How did that hit connect?!"][6] moments.

</details>

全量预测在格斗游戏和像《火箭联盟》这类重物理游戏中帮助很大，因为玩家可以对正在发生的事情做出反应，并且与物体交互(比如撞击飞行中的巨型足球)，同时又[不会显得奇怪][7]，但代价也更加昂贵了。

<details markdown="1">
<summary>原文</summary>

Predicting everything really helps in fighting games and physics-heavy games like Rocket League because players can react to what's happening and interact with objects (i.e. slam into a giant, flying soccer ball) at the same time [without any weirdness][7], but it's more expensive.

</details>

## 3. C/S 还是 P2P？

<details markdown="1">
<summary>原文</summary>

## 3. Client-server or peer-to-peer?

</details>

> 想一想：
>
> - 网络术语到底有多模糊。

<details markdown="1">
<summary>原文</summary>

> Think about:
>
> - Just how vague networking terminology is.

</details>

如果你问的是“星型网络还是网状网络”，那你选星型网络拓扑结构一定不会出错。
网状网络会：

<details markdown="1">
<summary>原文</summary>

If you're asking in the sense of "star network or mesh network?", you really can't go wrong with the star network topology. Mesh networks:

</details>

- 消耗更多带宽（连接更多，数据更多），某些消费级网络套餐可能不够。
- 遇到更多 NAT/防火墙问题（连接更多，问题也更多）。
- 容易引入各种边缘情况来让复制更加复杂化，从而需要[复杂一致性模型][8]来解决问题。

<details markdown="1">
<summary>原文</summary>

- Consume more bandwidth (more connections, more data). Some consumer-grade internet plans may not have enough.
- Encounter more NAT/firewall issues (more connections, more problems).
- Tend to make replication more complicated by introducing edge cases that inspire [complex consensus models][8] to resolve.

</details>

如果你问的是“独立服务器还是客户端主机”，那更多是成本问题。
服务器只是一种角色，这个角色是由玩家的机器执行，还是由数据中心机房执行，都不应影响任何代码。
为了支持客户端开主机模式（在 IPv6 成为主流之前），你会需要用到 NAT 穿透方案（如 ICE、STUN、TURN）；除此之外，整体上是同一件事。

<details markdown="1">
<summary>原文</summary>

If you're asking in the sense of "dedicated server or client host?", that's more of a money question. The server is just a role. Whether that role is performed by a player's machine or a datacenter rack shouldn't affect any code. You would need something to do NAT traversal (e.g. ICE, STUN, TURN) to support clients hosting (until IPv6 becomes the dominant protocol), but other than that, it's all the same.

</details>

[1]: https://github.com/rust-lang/unsafe-code-guidelines/issues/237
[2]: https://github.com/rust-lang/rust/issues/73328
[3]: https://github.com/WebAssembly/design/blob/main/Rationale.md#nan-bit-pattern-nondeterminism
[4]: https://simonbyrne.github.io/notes/fastmath/
[5]: https://www.forrestthewoods.com/blog/tech_of_planetary_annihilation_chrono_cam/
[6]: https://youtu.be/ueEmiDM94IE?t=1802
[7]: https://youtu.be/ueEmiDM94IE?t=2235
[8]: https://github.com/heidihoward/distributed-consensus-reading-list

另外可参考：[网络代码术语（Netcode Terminology）](https://gist.github.com/maniwani/f92cc5d827b00163f5846ea7dcb90d44)

---

###### [lokimckay](https://github.com/bevyengine/bevy/discussions/8675#discussioncomment-13231866) 于 2025 年 5 月 22 日：

> 表达得非常清晰，太棒的资料了，感谢发布。
> 
> 关于网状网络带宽那句：_"某些消费级网络套餐可能不够"_ 这条也适用于 DR 吗（只发送输入）？
> 
> 我猜这取决于输入频率和输入量，但我不确定从网状转向星型有没有什么好的经验法则可参考。

<details markdown="1">
<summary>原文</summary>

> Super clear, fantastic resource thanks for posting.
>
> RE mesh network bandwidth: _"Some consumer-grade internet plans may not have enough"_
Does this also apply to DR (only inputs sent)?
>
> I suppose it would depend on the frequency and volume of inputs, but I'm not sure what a good rule of thumb would be to consider transitioning from mesh -> star.
>

</details>

---

###### [Plecra](https://github.com/bevyengine/bevy/discussions/8675#discussioncomment-15812632) 于 2026 年 2 月 16 日：

> > DR 客户端通常是全预测或者完全不预测
> 
> 始终记住：你**依然**需要为延迟方案设计游戏机制。
> [8 Frames in 16ms: Rollback Networking in Mortal Kombat and Injustice 2](https://www.youtube.com/watch?v=7jb0FOcImdg) 直接讲的是确定性回滚系统；
> [I Shot You First: Networking the Gameplay of Halo: Reach](https://www.youtube.com/watch?v=h47zZrqjgLc) 也直接讨论了延迟补偿的机制设计。
> 再加上 [Overwatch Gameplay Architecture and Netcode](https://www.youtube.com/watch?v=W3aieHjyNvw)，我都很推荐。
>
> 简介：游戏里的机械交互经常给玩家带来截然不同的预期，这也导致了感觉“正确”的延迟方案多种多样。
> 最简单的例子：你会希望伤害能被立刻预测，这样才能马上知道自己是否打中；但你**绝不**希望敌人被预测为死亡后，又因为回滚被复活。
> 相反，你应该调试模拟方案来确保死亡只发生在已经确认的输入下。
> 《真人快打》的演讲里也讲到了特殊技的类似处理。
> 《光环》的演讲里有个开盾例子：他们给能力激活设计了一个游戏内固有延迟，角色会先做出启动护盾的动画，以此确保对护盾的攻击不会被错误预测。
> 你会想要将所有这些设计决策也用于 DR 的。

<details markdown="1">
<summary>原文</summary>

> > DR clients mostly predict everything or predict nothing
>
> It's good to keep in mind that you *do* still want to design game mechanics for latency resolution. [8 Frames in 16ms: Rollback Networking in Mortal Kombat and Injustice 2](https://www.youtube.com/watch?v=7jb0FOcImdg) is directly covering a deterministic rollback system, and [I Shot You First: Networking the Gameplay of Halo: Reach](https://www.youtube.com/watch?v=h47zZrqjgLc) also directly discusses mechanical design for latency compesnsation. Id recommend both, along with [Overwatch Gameplay Architecture and Netcode](https://www.youtube.com/watch?v=W3aieHjyNvw).
>
> TLDW, mechanical interactions in a game will often carry quite drastically different expectations from a player, which lead to different kinds of latency resolution feeling "right": The simplest example is that you want damage to be immediately predicted, to understand whether you're connecting right, but you *never* want an enemy to be predicted to die and then be resurrected by rollback. Instead you want to tune the simulation to make sure that deaths always happen only based on input that has been confirmed. The Mortal Kombat talk covers this in the case of special moves too. The halo talk has an example about activating shields, where they decided to have the ability activation have a builtin delay within the game, where the character is animated to initiate the shield, to make sure that attacks against a shield didnt have to be mispredicted. You want to be applying all of these design decisions with DR too.
>

</details>

---

###### [captkirk88](https://github.com/bevyengine/bevy/discussions/8675#discussioncomment-15861953) 于 2026 年 2 月 19 日：

> 很棒的话题！
>
> 很久以前我曾深入研究过，试图找出完美的网络策略。结论是：不存在完美网络策略。每种方案都有取舍。
> 可惜后来我丢了那份源码和那台电脑（那时候还没云服务，GitHub 也没流行，还记得 SVN 吗？这个就不展开了）。
> 我的方案是：服务器基于玩家环境维护所有实体状态（这会非常吃服务器内存，不过可以用数据库缓解）。
>
> 在发送完玩家环境状态后，服务器充当了一个简单的输入中转站。
> 通过一个“等待生成”的延迟，这个方案是可行的，但是会出现实体不同步。
> 解决也简单：通过发送玩家在等待期间发生的输入来“追赶”同步。
> 不要同步到最新状态，否则你会陷入永远追不上的挫败感。
> 
> 这种方案要求你围绕一个限制做设计：所有实体处理都必须由输入驱动。
> 玩家与 NPC 交互，必须要发送输入（鼠标点击）；玩家走过地图上一个触发点，然后刷怪。
> 
> 好处是客户端知道 NPC 会怎么反应、触发点会发生什么，服务器只做一个中转。
> 坏处是客户端控制权太大。
>
> 经过大量研究以及被折腾坏的人际关系之后，我发现没有所谓最好的方法。
> 唯一的方法是：选择与你要做的游戏类型、以及你拥有的资源相匹配的方案。

<details markdown="1">
<summary>原文</summary>

> Great topic!
>
> A long time ago I delved deeply into trying to come up with the perfect networking strategy.  There is no perfect networking strategy.  All have trade offs.  Unfortunately I lost that source code and PC (this was before the cloud and Github wasn't popular yet, remember SVN? I won't go there.)  My approach was that the Server kept entity state of everything based on player environment (this requires a lot of RAM on the server unfortunately, which could be mitigated with databases).
>
> Server acted as a simple relay of inputs after player environment state was sent.  This is possible using a "wait to spawn" delay but you run into the trouble of desynced entities.  Easily resolved by playing "catch-up" by sending the player inputs that happened while waiting until you catch up and sync.  Don't sync to the most up-to-date state because then you will hit the frustration of never catching up.  This approach requires you to design around the limitation of all entity handling is done by input.  A player interacts with a NPC, they had to send input (mouse click) to do that.  A player walks over a trigger on the map, enemies spawn.  Benefit is the client side knows what will happen with the NPC and what that trigger will do.  Server is only a relay.  Downside is client has way too much control.
>
> After a lot of research and destroyed relationships, I found there is no best approach.  The only approach is what works with the kind of game you are designing and the resources you have available.

</details>


