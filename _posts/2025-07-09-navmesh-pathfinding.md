---
layout: post
title: "2D导航网格寻路"
date: 2025-07-09
tags: [Rust, 寻路算法]
---

# <center>2D 导航网格</center>

原文：[2D Navmesh Pathfinding](https://gabdube.github.io/articles/navmesh_pathfinding/navmesh_pathfinding.html)

寻路是一项复杂的工作。当你使用现成的游戏引擎（如 Unity）时，这不是需要操心的问题。然而，对于自研游戏引擎来说，开发一套寻路系统则是必不可少的，并且（大部分情况下）需要从头开始编写代码。

好消息是，网上具有大量的高质量文章，理解寻路的基本原理只需几次简单的 Google 搜索即可掌握。

坏消息是，理论不同于实践，你越是脱离基础的场景（例如基于网格的寻路），信息就越趋于碎片化，要把所有内容整合起来也就越困难。

这是一篇[可以互动的文章](https://gabdube.github.io/articles/navmesh_pathfinding/navmesh_pathfinding.html)，详细讲解了我构建自己 2D 寻路工具的所有步骤，包括导航网格的建立、使用 A* 算法计算最优路径，以及最终路径的生成。当然，整个过程使用的是 Rust 编程语言。

如果你的屏幕宽度小于 1300 像素，互动演示将默认隐藏。建议在桌面端打开以获得最佳体验。

演示相关代码可以在 github 上[访问](https://github.com/gabdube/gabdube.github.io/tree/master/articles/navmesh_pathfinding)，并且所有的寻路逻辑都在[这个](https://github.com/gabdube/gabdube.github.io/blob/master/articles/navmesh_pathfinding/wasm_src/src/data/navigation.rs)文件内。

特别鸣谢：

- [Red Blob Games](https://www.redblobgames.com/pathfinding/a-star/implementation.html): 可在线访问的最好的 A* 算法指导
- [PixelFrog](https://pixelfrog-assets.itch.io/): 本文使用的令人惊喜的资产（asset）
- [Delaunator](https://github.com/mapbox/delaunator): 处理 Delaunay 三角剖分的重要库
- [Delaunator-rs](https://github.com/mourner/delaunator-rs) delaunator 的 Rust 移植版
- [Simple stupid funnel algorithm](https://digestingduck.blogspot.com/2010/03/simple-stupid-funnel-algorithm.html): 路径平滑算法

---

## <center>使用 Delaunay 三角剖分生成基础导航网格</center>

导航网格（`navmesh`）的基础实际上只是一个由相互连接的多边形组成的网格，在本文场景下就是三角形。导航网格的生成通常是通过一个接收 2D 或 3D 对象作为输入的“烘焙”步骤完成的。

生成导航网格最佳方法称为 **Delaunay 三角剖分([Delaunay triangulation](https://zh.wikipedia.org/wiki/Delaunay_triangulation))**。 Delaunay 三角剖分的计算成本很低，并即使是在低性能的硬件上，也能够仅在数毫秒内轻松处理上千个顶点。此外，你不需要从头实现它，在每种编程语言中都有许多现成的库可供使用。它们大多数都是精小、独立且高度优化过的。

Rust 中用于 Delaunay 三角剖分的 crate 有：`delaunator`、`cdt`和`spade`。`spade`提供了一份文档，对这三者之间的功能进行了比较。在这三个库中，`spade`的功能最为丰富。

在这个演示中，我会使用`delaunator`，因为它是最简单的实现。其源代码仅包含一个约 600 行代码的文件。然而，这种简单性也带来了局限性，详见“局限性与改进方向”一节。

使用`delaunator`时，生成过程是通过构建一个点（也被称为顶点）列表，然后调用三角剖分函数来完成的：

```rust
pub fn triangulate(points: &[Point]) -> Triangulation
```

在处理精灵图（`sprite`）时，首先添加世界的边界，然后为每个具有碰撞属性的精灵图添加其包围盒（`bounding box`）的坐标。我们也会保存这些精灵图的包围盒，因为在后续步骤中我们需要用它们来移除不可到达的节点。

在`generation`演示中，您可以使用“显示导航网格”复选框切换导航网格。

代码在[这里](https://github.com/gabdube/gabdube.github.io/blob/master/articles/navmesh_pathfinding/wasm_src/src/data/navigation.rs#L120)

![导航网格三角形](/assets/images/navmesh/1.png)

---

## <center>理解三角剖分输出</center>

三角剖分的输出是一个双向连接边的列表（也称为[`DCEL`](https://en.wikipedia.org/wiki/Doubly_connected_edge_list)）。其被定义为：

```rust
pub struct Triangulation {
    /// 点索引向量，其中每个三元组代表一个Delaunay三角形。
    /// 所有三角形都是逆时针方向。
    pub triangles: Vec<usize>,

    /// 允许遍历三角图的相邻半边索引向量。
    ///
    /// 数组中的第`i`条半边，对应于该半边的起始点`triangles[i]`。
    /// `halfedges[i]`是相邻三角形中双半边的索引（或`EMPTY`表示凸包上的外半边）。
    pub halfedges: Vec<usize>,

    /// 三角剖分凸包上的相关点的索引向量，
    /// 逆时针方向。
    pub hull: Vec<usize>,
}
```

这个结构刚开始可能看起来很混乱，但一旦你掌握了三个关键概念，它就会变得非常容易理解。

**#1. 边**。边是`DCEL`的组成元素。它们存储在`triangles`Vec 中。向量中的每个值都是一条边的起点。如此：

- `triangulation.triangles.len()` 返回三角剖分中边的数量
- `triangulation.triangles[0]` 返回`#0`号边在原始顶点向量中的点索引
- `points[triangulation.triangles[0]]` 返回`#0`号边起始点的坐标（`points`是传递给三角剖分函数的原始向量）

**#2. 三角形**。三角形也存储在`triangles`中，每三个边组成了一个三角形。如此：

- `triangulation.triangles.len() / 3` 返回在三角剖分中三角形的数量
- `triangulation.triangles[0..3]` 定义了第一个三角形
- `index [0, 1]` 定义了第一条边的起始点和结束点
- `index [1, 2]` 定义了第二条边的起始点和结束点
- `index [2, 0]` 定义了第三条边的起始点和结束点

**#3. 从一个三角形移动到它的相邻三角形**。一个三角形最多可以有三个相邻的三角形，因此通过线性方式遍历 `triangles` 向量并不能有效实现这一操作。这就是需要借助`halfedges`向量的地方：对于边`X`，通过索引访问`halfedges`可以得到其对应的对边（即相邻三角形的边，与当前边方向相反，如果该边对应三角剖分之外，则返回 `usize::MAX`）。如此：

- `triangulation.halfedges[0]` 返回相邻三角形的边索引
- `triangulation.triangles[triangulation.halfedges[0]]` 返回相邻边的点索引
- `points[triangulation.triangles[triangulation.halfedges[0]]]` 返回相邻边的起始点坐标

有关更多详细信息，`delaunator`对数据结构的工作原理有[更深入的指导](https://mapbox.github.io/delaunator/)。

让我们将这些知识付诸实践，尝试在导航网格上执行一个最简单的操作……

---

## <center>将一个点映射到导航网格</center>

在导航网格中需要查询的最基本信息之一，就是某个点位于哪一个三角形内。虽然可以通过遍历所有三角形并逐个检查点是否在内部来实现（即暴力搜索），但得益于导航网格的构建方式，我们可以用更少的计算指令完成信息查询。

我们使用的导航网格具有一个重要的属性：所有三角形的顶点都是以**逆时针顺序**生成的。这使得我们不仅可以判断一个点是否在某个三角形内部，如果不在，还能知道应该朝哪个方向继续搜索。我们要做的只是计算当前被检测的点位于某条边的哪一侧。[`robust`](https://docs.rs/robust/latest/robust/fn.orient2d.html)库为我们提供了 `orient2d` 函数来完成这个任务。

这里有一个，呃，“视觉辅助”来解释它是如何工作的

![点与三角形位置关系](/assets/images/navmesh/triangulation.png)

- Y 不在 P0P1 的逆时针方向，因此 Y 最有可能在`triangulation.halfedges[P0]`方向（指相邻的三角形，以下同理）
- X 不在 P1P2 的逆时针方向，因此 X 最有可能在`triangulation.halfedges[P1]`方向
- W 不在 P2P0 的逆时针方向，因此 W 最有可能在`triangulation.halfedges[P2]`方向
- Z 在所有边的逆时针方向，因此 Z 是在这个三角形内的

所以，从导航网格中的任何一个三角形开始（`#0`号三角形和其他任何一个三角形都一样，而且能让你的导航网格不至于太大），执行上述逻辑，并且在邻居间移动，直到第四个条件返回`true`。

要调试这个功能，请在`navigation`栏，勾选“Debug triangle lookup”。`triangle_at`函数代码也能在[这里](https://github.com/gabdube/gabdube.github.io/blob/master/articles/navmesh_pathfinding/wasm_src/src/data/navigation.rs#L286)找到。

![三角形搜索调试](/assets/images/navmesh/2.png)

---

## <center>生成图</center>

A* 算法是基于图工作的，然而我们的导航网格还不是一个图。我们需要定义什么是单元(`cell`)，以及在它们之间移动的成本函数（`cost function`）是什么。在这个演示中，我使用了我能找到的最简单的方法：将导航网格中的三角形作为单元格，而在单元格之间移动的成本是每个三角形内切圆心之间的距离。`Red Blob games`也有一篇很好的文章《[`Map Representation`](https://theory.stanford.edu/~amitp/GameProgramming/MapRepresentations.html)》，对这一主题进行了更深入的探讨。

要调试该图，请转到`pathfinding`栏并选中“Show pathfinding graph”选项。

由于直接使用具有多级间接性的导航网格对代码的可读性和数据的局部性并不友好，因此我们可以存储的信息不仅仅是节点之间的距离。每个字段的意义将在后面进行详细解释，但目前，先来看看一个节点的数据类型定义：

```rust
pub struct Triangle(u32);

struct NavNodeNeighbor {
    pub center: PositionF32,
    pub segment: [PositionF32; 2],
    pub triangle: Triangle,
    pub distance: f32,
}

struct NavNode {
    pub triangle: Triangle,
    pub center: PositionF32,
    pub n0: NavNodeNeighbor,
    pub n1: NavNodeNeighbor,
    pub n2: NavNodeNeighbor,
    pub disconnected: u32
}
```

图形生成是通过遍历导航网格中的每个三角形，查询邻居并将结果存储在`NavNode`结构中来完成的。节点存储在`Vec<NavNode>`中，它映射于导航网格的三角形。

这里是[代码](https://github.com/gabdube/gabdube.github.io/blob/master/articles/navmesh_pathfinding/wasm_src/src/data/navigation.rs#L163)

![内切圆心图](/assets/images/navmesh/3.png)

---

## <center>从图中移除阻塞节点</center>

接下来，我们需要从图中删除被阻塞的节点。该算法通过遍历图中的每个节点，并逐一检查其是否位于`#1`小节步骤中保存的包围盒内来判断是否被阻塞。这种方式在将来数据规模扩大时会是一个需要优化的问题。

不要从`Vec`中删除节点，否则我们会丢失三角形->节点的映射关系，而且我们可能还需要从被阻塞的节点内部进行路径计算。相反：

- 添加一个`disconnected`标志。如果节点为阻塞的，设置该标志为`true`。
- 查找邻接节点。对于每一个邻接节点，将其和阻塞节点之间的距离设置为无穷大。
- 阻塞节点到其邻接节点的距离保持不变，这样我们仍然可以知道如何从内部“离开”该节点。

可以通过选中“Show blocked cells”来切换`navigation`栏中的阻塞单元格调试。

这里是[代码](https://github.com/gabdube/gabdube.github.io/blob/master/articles/navmesh_pathfinding/wasm_src/src/data/navigation.rs#L243)

![阻塞单元格](/assets/images/navmesh/4.png)

---

## <center>使用 A* 算法计算两个单元之间的最优路径</center>

Rust 在`pathfinding`包中有一个 A* 算法实现。然而，因为该算法很容易编码，所以我们做了自己的实现。可以通过选中“Debug pathfinding”在`pathfinding`栏中切换寻路调试开关，并选择一个小人来查看到鼠标光标位置的“粗略”计算路径。

此外，虽然编码这个算法很简单，但解释代码背后的逻辑应该是另一篇文章的主题。网上已经有几篇这样的文章。我找到的最好的一个是[Implementation of A* by Red Blob Games](https://www.redblobgames.com/pathfinding/a-star/implementation.html)。

基于我们的图，运行 A* 算法所需的所有信息都已经在节点中。单元中的每个邻居（n0，n1，n2）都有`distance`用来快速添加移动到该节点的`new_cost`。并且邻居节点的`center`可用于计算启发函数(`neighbor.center.distance(end_point)`)。

该算法输出的是移动单位必须穿越的一系列(`Vec`)路径段（`segments`），也可称为“闸门”（`gates`），这会在后续步骤中用于优化路径。

在[这里](https://github.com/gabdube/gabdube.github.io/blob/master/articles/navmesh_pathfinding/wasm_src/src/data/navigation.rs#L372)查看该项目的实现。

![路径段](/assets/images/navmesh/6.png)

---

## <center>使用简单笨漏斗算法优化路径</center>

接下来，我们需要优化最终路径。这是使用简单笨漏斗算法([Simple Stupid Funnel Algorithm](https://digestingduck.blogspot.com/2010/03/simple-stupid-funnel-algorithm.html))完成的。据我所知，在 Rust 中还没有实现了该算法的库，因此该项目[这里](https://github.com/gabdube/gabdube.github.io/blob/master/articles/navmesh_pathfinding/wasm_src/src/data/navigation.rs#L477)有自己的实现。

在`pathfinding`栏中，通过勾选"debug pathfinding funnel"来切换漏斗调试。

![路径漏斗调试](/assets/images/navmesh/7.png)

该算法接收一个起始点和在上一步中生成的“闸门”列表。在运行前，终点也被添加为一个“闸门”。它的输出是一个角色必须经过的路径点`Vec`列表。

我们首先将起点设为漏斗的顶点，然后沿着入口边的两侧点（在调试视图中，绿色表示右侧，蓝色表示左侧），只要能够进一步收窄漏斗的角度，就逐步推进漏斗的边界。这一判断可以通过调用`robust`库中的 `orient2d`函数来完成：

- `robust::orient2d(apex, right, new_right) <= 0.0`
- `robust::orient2d(apex, left, new_left) >= 0.0`

一旦你得到了最终的路径点数组，剩下的事情就是将它发送给你的游戏任务系统。

在`pathfinding`栏中，通过勾选“Debug pathfinding (smoothed)”来切换优化后的寻路显示。

![优化路径](/assets/images/navmesh/8.png)

---

## <center>局限性与改进方向</center>

本演示仅实现了非常基础的寻路功能。现实世界的场景通常还需要一些功能增强。这里列举其中一些。

### 路径次优问题

当某些三角形明显大于其他三角形时，使用三角形中心来计算节点之间的距离时会返回次优的结果。这个问题可以通过在三角剖分阶段检测较大的三角形并添加额外顶点进行细分来解决。然而，细分后的三角形会增加算法运行的开销，因此这是一个在准确性与速度之间的权衡。

![次优路径问题](/assets/images/navmesh/err2.png)

### 边缘断裂问题

当顶点之间非常接近时，`Delaunator` 会将它们合并，这会导致重叠精灵图生成的三角剖分结果无法准确反映实际的“阻塞区域”。如果必须保留边界，则应使用约束 Delaunay 三角剖分（[constrained Delaunay triangulation](https://gwlucastrig.github.io/TinfourDocs/DelaunayIntroCDT/index.html)）。[`cdt`](https://crates.io/crates/cdt) 和 [`spade`](https://crates.io/crates/spade) 这两个库都支持该功能。

![边缘断裂问题](/assets/images/navmesh/err1.png)

### 多单位移动

在当前演示中，我们一次只能移动一个单位。当需要同时移动多个单位时，它们的最终位置需要围绕目标点分散排列，并且可能有时根本不会落在你点击的那个单元格中。

### 单位包围盒

在这个演示中，单位的大小为零。而在真实场景中，单位应该具有一个包围盒，较大的单位甚至可能会横跨多个单元格。

### 地形影响因子

大多数游戏中常见的功能之一是不同类型的地形会影响角色的移动速度。可以通过为节点之间的距离函数添加一个“地形系数”来实现这一功能。这在生成图的阶段很容易完成。

### 门和其他“有趣”的功能

然而，有些功能并不容易实现，比如“门”（`doors`）机制。在这种情况下，寻路将不再只是一个对 `compute_path` 函数的简单调用，而是可能需要与游戏引擎中的其他系统进行集成。

### 多线程支持

JavaScript / WASM 是一个单线程环境（虽然有 Worker，但将其与 WebAssembly 一起使用是一个极其蛋疼的事情）。假设我们运行在一个真正的多线程环境中，并拥有一个基于任务的调度系统，那么我们可以轻松地将寻路任务分配到线程池中，因为所有代码依赖项都是不变的（`const`）。

```rust
pub fn compute_path(&self, start: PositionF32, end: PositionF32, output: &mut Vec<PositionF32>) -> bool {  }
```

## <center>结语</center>

构建一个自定义的寻路系统无疑是一项具有挑战性的任务，但通过将每个步骤拆解分析，它就会变得更容易掌控。我也希望这个互动演示对你有帮助，因为尽管我大部分是复用了另一个项目的系统，但制作这篇文章所花费的时间中，有 99% 都用在了这个演示的实现上（不过过程很有趣，未留遗憾）。
