---
layout: post
title: "介绍 Matchbox"
date: 2026-05-28
tags: [multiplayer, p2p, WebRTC]
original: https://johanhelsing.studio/posts/introducing-matchbox
---

# Matchbox 介绍

我非常高兴地宣布 [Matchbox](https://github.com/johanhelsing/matchbox) 项目。这是一个用于在 Rust WASM 中实现无痛点对点网络通信的解决方案。

<details markdown="1">
<summary>原文</summary>

I'm very happy to announce the [Matchbox](https://github.com/johanhelsing/matchbox) project. A solution for painless peer-to-peer networking in rust web assembly.
</details>

Matchbox 的诞生是因为我正在用 Rust 开发一款多人在线网页游戏，当时我遇到了以下问题：

> 如何将 N 个网页浏览器通过不可靠、无序的点对点连接进行互联？

<details markdown="1">
<summary>原文</summary>

Matchbox was born because I was making a multiplayer web game in rust, and I had the following problem:

> How can I connect N web browsers with unreliable, unordered peer-to-peer connections?
</details>

## 动机

那么，我为什么需要这个呢？

<details markdown="1">
<summary>原文</summary>

Now, why did I want this?
</details>

1. 我需要不可靠且无序的连接，因为这是实时多人游戏的理想连接类型——谁有时间等着重新发送那些早已过时、与当前游戏状态毫无关系的丢失数据包呢？
2. 我想要直接的点对点连接，也就是不需要中间服务器参与，因为这样可以实现更低的延迟。如果两个对等端距离很近，例如在同一个局域网内，它们就不需要等待数据被发送到某个可能很远的服务器再返回。此外，这样做成本更低，因为不必去运行服务器。而且点对点的方式兼容类似 [GGPO](https://www.ggpo.net/) 的回滚机制，比如出色的 [GGRS](https://github.com/gschup/ggrs)。
3. 最后，我非常喜欢制作 Game Jam 游戏。虽然这听起来可能有些疯狂，但我想尝试做一款联网多人游戏，而且我认为要让人们愿意去尝试它，它必须是一款网页游戏。

<details markdown="1">
<summary>原文</summary>

1. I wanted unreliable and unordered connections because that's the ideal connection type for real-time multiplayer - Who has time to wait around for re-sending lost packets sent so long ago they are no longer relevant to the current game state?
2. I wanted direct peer connections, i.e. not involving a server in the middle, because this allows even lower latency. If the peers are close together, for instance on the same network, they don't have to wait for traffic to be sent to some possibly far-away server and back. Also, it's cheaper because it means I don't have to run a server. And being p2p means it's compatible with [GGPO](https://www.ggpo.net/)-like rollback, like the amazing [GGRS](https://github.com/gschup/ggrs).
3. Finally, I really enjoy making game jam games. And though it probably sounds crazy, I want to have a go at making a networked multiplayer game, and I believe that in order to get people to try it, it really has to be a web game.
</details>

## 现有项目

目前（排除浏览器插件的情况下），用不可靠、无序连接来连接网页浏览器的唯一方式，是通过[建立过程是出了名的复杂](https://gafferongames.com/post/why_cant_i_send_udp_packets_from_a_browser/#what-about-webrtc)的 WebRTC。

<details markdown="1">
<summary>原文</summary>

Currently (and excluding browser plugins), the only way to connect web browsers with unreliable, unordered connections, is through WebRTC, [which is notoriously difficult to set up](https://gafferongames.com/post/why_cant_i_send_udp_packets_from_a_browser/#what-about-webrtc).
</details>

有一些项目旨在简化这一过程。

<details markdown="1">
<summary>原文</summary>

There are a couple of projects that aim to make it easier.
</details>

例如，[naia-socket](https://github.com/naia-rs/naia-socket) 是一个出色的项目，它几乎隐藏了通过 WebRTC 将客户端连接到原生服务器的所有复杂性。唯一的问题是它与 C/S（客户端-服务端）架构紧密耦合，[无法在两个浏览器之间直接建立连接](https://github.com/naia-rs/naia-socket/issues/45)。

<details markdown="1">
<summary>原文</summary>

For instance, there is [naia-socket](https://github.com/naia-rs/naia-socket), which is an amazing project that hides away almost all of the complexity of connecting clients with WebRTC to a native server. The only problem is that it's very closely tied to a client-server architecture, and offers [no way to establish a connection directly between two browsers](https://github.com/naia-rs/naia-socket/issues/45).
</details>

我发现了 Ernest Wong 的一个项目——[dango-tribute](https://github.com/ErnWong/dango-tribute)，其中包含了修改 naia-socket 的补丁，使 WebRTC 服务器可以托管在其中一个浏览器的 iframe 中。这几乎正是我想要的，但它本质上仍然是 C/S 连接。我想要的是每个玩家之间都能互相连接，这就是为什么我最终将它拆解并重新组装成了 Matchbox。

<details markdown="1">
<summary>原文</summary>

I stumbled upon a project by Ernest Wong, [dango-tribute](https://github.com/ErnWong/dango-tribute), which has patches that modify naia-socket so the WebRTC server is hosted in its own iframe in one of the browsers. This was almost what I wanted, but it's still a client-server connection. I wanted connections between every player, which is why I ended up picking it apart and putting it back together as Matchbox.
</details>

## Matchbox 服务器

上面提到的"服务器"这个词可能会让人觉得这不是真正的点对点通信。但事实并非如此！服务器只是在建立对等连接时才需要使用。

<details markdown="1">
<summary>原文</summary>

So the word "server" above might make it sound like this is not really peer-to-peer. That's not the case! The server is only needed to get peer connections up and running.
</details>

虽然 WebRTC 支持点对点连接，但建立这些连接的过程有些棘手。浏览器需要知道彼此的存在、各自的公网 IP 地址，以及如何穿越 NAT 和其他阻碍通信的麻烦事物。交换这类信息的过程称为信令（Signalling），尽管 WebRTC 规范描述了需要交换哪些信息，但规范有意没有规定具体的交换方式。

<details markdown="1">
<summary>原文</summary>

Though WebRTC allows peer-to-peer connections, establishing those connections is a bit tricky. The browsers need to know about each other, what their public IP addresses are, and how to traverse NATs and other pesky things in the way. Exchanging this kind of information is called signalling, and though the WebRTC spec describes what kind of information needs to be exchanged, it has been intentionally left out of the spec exactly how it is to be exchanged.
</details>

无论如何，这意味着我需要一个信令服务器，所以我编写了 matchbox_server。

<details markdown="1">
<summary>原文</summary>

In any case, this meant I needed a signalling server, so I made matchbox_server.
</details>

Matchbox 服务器是一个相对简单的 Rust Web 服务器应用。每当有人想加入一个点对点网络时，他们通过 WebSocket 连接到 Matchbox 服务器并提供一个房间 ID。然后，同一房间中的所有人都会收到新对端的通知，服务器会中转这些对端建立直接 WebRTC 连接所需的信令消息。

<details markdown="1">
<summary>原文</summary>

Matchbox server is relatively simple rust web server application. Whenever someone wants to join a p2p network, they connect to a Matchbox server over websockets and provide a room id. Everyone in the same room is then notified about the new peer, and the server will relay whatever signalling messages those peers need to establish direct WebRTC connections.
</details>

还有一种特殊的动态房间类型，即 next_n 房间，其中 n 是房间的最大玩家数。它会为接下来连接到信令服务器的 n 个玩家创建新房间。也就是说，这是一种非常简易的匹配机制。

<details markdown="1">
<summary>原文</summary>

There is also a special type of dynamic room, next_n rooms, where n is the maximum number of players in a room, which creates new rooms for the next n players to connect to the signalling server. i.e. it's a very crude form of matchmaking.
</details>

## matchbox_socket

`matchbox_socket` 是一个 Rust crate，它封装了如何连接到 Matchbox 信令服务器以及与房间中所有对端建立不可靠且无序的 WebRTC 数据连接的所有复杂细节。

<details markdown="1">
<summary>原文</summary>

`matchbox_socket` is a rust crate that hides away the intricacies of how to connect to a matchbox signalling server and establish unreliable, unordered WebRTC data connections to all peers in the room.
</details>

你只需要提供一个 Matchbox 服务器地址，然后 `.await` 或轮询一个消息处理循环，就会在新的对端连接建立时收到通知，并且可以使用非阻塞的普通函数调用，向这些对端发送和接收消息。

<details markdown="1">
<summary>原文</summary>

All you need to do, is provide a matchbox server address, and `.await` or poll a message processing loop, and you will be notified whenever new peer connections have been established, and you'll be able to send and receive messages to those peers using non-blocking regular function calls.
</details>

`matchbox_socket` 还有一个 `ggrs` 集成特性，它实现了 `ggrs` 的 `NonBlockingSocket` trait，这意味着它可以直接与 `ggrs` 配合使用。

<details markdown="1">
<summary>原文</summary>

`matchbox_socket` also has a `ggrs` integration feature, which adds an implementation of `ggrs` 's `NonBlockingSocket` trait which means it can be directly used with `ggrs`.
</details>

例如，以下是如何在 Bevy 系统中启动 matchbox socket 并处理消息的示例。

<details markdown="1">
<summary>原文</summary>

For instance, here's how to start a matchbox socket and handle messages in Bevy systems.
</details>

```rust
use matchbox_socket::WebRtcSocket;

fn start_matchbox_socket(mut commands: Commands, task_pool: Res<IoTaskPool>) {
    let url = "wss://matchbox.example.com/next_2";
    let (socket, message_loop) = WebRtcSocket::new(url);

    // 消息循环需要被 await，否则什么都不会发生。
    // 这里我们使用 Bevy 的任务系统来实现。
    task_pool.spawn(message_loop).detach();

    // 将 socket 接口放在其他系统可以访问的地方
    commands.insert_resource(socket);
}

fn handle_new_peers(mut socket: ResMut<WebRtcSocket>) {
    // 检查自上次以来是否有新连接
    // 返回对端 ID 的向量列表。
    let new_peers = socket.accept_new_connections();
}

fn send_stuff(mut socket: ResMut<WebRtcSocket>) {
    let payload: Box<[u8]> = // 你想发送的任何内容
    socket.send(payload, "peer-id");
}

fn receive_stuff(mut socket: ResMut<WebRtcSocket>) {
    for (peer_id, payload) in socket.receive() {
        // 处理来自对端的数据
    }
}
```

## 演示

要感受这一切在实际中是如何运作的，最简单的方式就是查看 [matchbox_demo](https://github.com/johanhelsing/matchbox/tree/main/matchbox_demo) 示例的代码。

<details markdown="1">
<summary>原文</summary>

The easiest way to get a feel for how all this works in practice, is to take a look at the code of the [matchbox_demo](https://github.com/johanhelsing/matchbox/tree/main/matchbox_demo) example.
</details>

它展示了如何制作一个可运行的浏览器多人"游戏"（如果在一个平面上移动方块也算游戏的话）。它将 `matchbox_socket` 与 [Bevy](https://bevyengine.org/) 游戏引擎和 [GGRS](https://gschup.github.io/ggrs/) 结合使用。

<details markdown="1">
<summary>原文</summary>

It shows how to make a working browser multiplayer "game" (if moving cubes around a plane can be called a game). It uses matchbox_socket with the [Bevy](https://bevyengine.org/) game engine and [GGRS](https://gschup.github.io/ggrs/) for rollback.
</details>

这里有一个在线演示版本（使用 WASD 移动方块）：

* 2 人模式：https://helsing.studio/box_game/
* 3 人模式：https://helsing.studio/box_game/?players=3
* N 人模式：修改上面的链接即可。

<details markdown="1">
<summary>原文</summary>

There is a live version here (move the cube with WASD):

* 2-Player: https://helsing.studio/box_game/
* 3-Player: https://helsing.studio/box_game/?players=3
* N-player: Edit the link above.
</details>

**注意：运行演示时，请确保每个窗口保持可见状态。Bevy 目前在标签页最小化或隐藏时会停止运行。**

<details markdown="1">
<summary>原文</summary>

**NOTE: Make sure you keep each window visible while running the demo. Bevy currently stops running if the tab is minimized or hidden.**
</details>

你也可以通过克隆 Matchbox 仓库在本地启动示例：https://github.com/johanhelsing/matchbox。

<details markdown="1">
<summary>原文</summary>

You can also start the example locally by cloning the Matchbox repo: https://github.com/johanhelsing/matchbox.
</details>

在项目目录中，构建并启动 matchbox 服务器：

```shell
cargo run --bin matchbox_server
```

<details markdown="1">
<summary>原文</summary>

From inside the project folder build and start the matchbox server:

```shell
cargo run --bin matchbox_server
```
</details>

然后进入 `matchbox_demo` 目录，构建、运行并托管示例：

```shell
cargo make serve -p release
```

<details markdown="1">
<summary>原文</summary>

Then go into the `matchbox_demo` folder and build, run and host the example:

```shell
cargo make serve -p release
```
</details>

如果你没有安装 `cargo-make`，可以通过 `cargo install cargo-make` 来安装。

<details markdown="1">
<summary>原文</summary>

If you don't have `cargo-make` you can install it with `cargo install cargo-make`.
</details>

现在你可以在两个浏览器中访问 http://localhost:4000 ，应该就能看到两个方块，并可以用 WASD 键来移动它们。

<details markdown="1">
<summary>原文</summary>

You can now access http://localhost:4000 in two browsers and you should be able to move two cubes around with WASD.
</details>

## 下一步计划

这个项目目前还处于早期阶段，有很多事情我想去完善。提升代码质量、添加完善的错误处理和断开连接机制是当前最高优先级的工作。

<details markdown="1">
<summary>原文</summary>

It's still quite early days for the project, and there are lots of things I want to fix. Improving code quality, adding proper error handling and disconnects are at the very top.
</details>

到目前为止，我只和少数几个人以及几种浏览器/设备进行过测试，所以可能还有大量我尚未发现的问题。

<details markdown="1">
<summary>原文</summary>

I've only yet tested this with a few people and a couple of browsers/devices, so there are probably loads of broken things I have yet to discover.
</details>

`matchbox_socket` 和 `matchbox_server` 现已发布到 crates.io :)

<details markdown="1">
<summary>原文</summary>

`matchbox_socket` and `matchbox_server` have now been released to creates.io :)
</details>

本次介绍就到这里。如果你感兴趣的话，请查看 [GitHub 仓库](https://github.com/johanhelsing/matchbox)。

<details markdown="1">
<summary>原文</summary>

That's all for now. Make sure to check out [GitHub repository](https://github.com/johanhelsing/matchbox) if you're interested.
</details>

我还写了一篇关于如何用 [Bevy、GGRS 和 Matchbox](https://johanhelsing.studio/posts/extreme-bevy) 制作一个完整游戏的教程。

<details markdown="1">
<summary>原文</summary>

I've also written a tutorial about how to make a complete game with [Bevy, GGRS and matchbox](https://johanhelsing.studio/posts/extreme_bevy).
</details>


