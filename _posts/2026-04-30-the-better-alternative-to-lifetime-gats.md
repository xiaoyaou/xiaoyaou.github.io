---
layout: post
title: "生命周期 GAT 的更优替代方案"
date: 2026-04-30
tags: [Rust, GAT, lifetime, HRTB]
toc: true
---

# 生命周期 GAT 的更优替代方案

<details>
<summary>原文</summary>

The Better Alternative to Lifetime GATs
</details>

> [2022-05-01（2022-05-30 更新）](https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats)

<details>
<summary>原文</summary>

> [2022-05-01 (updated 2022-05-30)](https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats)
</details>

更新（2022-05-30）：[danielhenrymantilla](https://github.com/danielhenrymantilla) 最近发布了一个 crate —— [nougat](https://docs.rs/nougat)。它提供了一个过程宏，让你可以用与常规 GAT 相同的语法来使用本文介绍的技巧。非常推荐你试试看！

<details>
<summary>原文</summary>

Update (2022-05-30): [danielhenrymantilla](https://github.com/danielhenrymantilla) recently released a crate, [nougat](https://docs.rs/nougat), which provides a proc macro that allows you to use the technique presented in this article with the same syntax as regular GATs. I encourage you to check it out!
</details>

## 真正的 GAT 缺点在哪里

<details>
<summary>原文</summary>

Where real GATs fall short
</details>

[GATs](https://github.com/rust-lang/rust/issues/44265) 是 Rust 的一个不稳定特性，预计会在接下来的几个版本中稳定。它允许你在 trait 的关联类型上添加泛型参数。这个特性的经典动机示例是 “借出式迭代器” trait：在任意时刻，它只允许其中一个 item 存在。使用生命周期 GAT 时，它的签名大概如下：

<details>
<summary>原文</summary>

[GATs](https://github.com/rust-lang/rust/issues/44265) are an unstable feature of Rust, likely to be stabilized in the next few versions, that allow you to add generic parameters on associated types in traits. The motivating example for this feature is the “lending iterator” trait, which allows you to define an iterator for which only one of its items can exist at any given time. With lifetime GATs, its signature would look something like this:
</details>

```rust
pub trait LendingIterator {
    type Item<'this>
    where
        Self: 'this;
    fn next(&mut self) -> Option<Self::Item<'_>>;
}
```

它还允许你实现一些原本做不到的迭代器，比如 `WindowsMut`（它返回的切片彼此重叠，普通迭代器无法表达这种场景）：

<details>
<summary>原文</summary>

and it would allow you to implement iterators you otherwise wouldn’t have been able to, like `WindowsMut` (since the slices it returns overlap, a regular iterator won’t work):
</details>

```rust
use ::core::mem;

pub fn windows_mut<T, const WINDOW_SIZE: usize>(
    slice: &mut [T],
) -> WindowsMut<'_, T, WINDOW_SIZE> {
    assert_ne!(WINDOW_SIZE, 0);
    WindowsMut { slice, first: true }
}

pub struct WindowsMut<'a, T, const WINDOW_SIZE: usize> {
    slice: &'a mut [T],
    first: bool,
}

impl<'a, T, const WINDOW_SIZE: usize> LendingIterator
for WindowsMut<'a, T, WINDOW_SIZE>
{
    type Item<'this> = &'this mut [T; WINDOW_SIZE] where 'a: 'this;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if !self.first {
            self.slice = &mut mem::take(&mut self.slice)[1..];
        }
        self.first = false;

        Some(self.slice.get_mut(..WINDOW_SIZE)?.try_into().unwrap())
    }
}
```

很好！`LendingIterator` trait 搞定了，也验证了它能工作。本文到此结束。

<details>
<summary>原文</summary>

Great! That’s our `LendingIterator` trait, done and dusted, and we’ve proven that it works. End of article.
</details>

不过先别急着结束，我们再做最后一件事：真正使用一下 `WindowsMut` 迭代器。其实我很确信它能跑，但为了学习，我们还是试一试，对吧？

<details>
<summary>原文</summary>

Well, before we go let’s just try one last thing: actually consuming the `WindowsMut` iterator. There’s no need to really because I’m sure it’ll work, but we’ll do it anyway for the learning experience, right?
</details>

先定义一个函数，把借出式迭代器的每个元素打印出来。这很简单：特征约束用 [HRTB](https://doc.rust-lang.org/nomicon/hrtb.html) 来写，消费逻辑用 `while let` 循环即可。

<details>
<summary>原文</summary>

So first we’ll define a function that prints each element of a lending iterator. This is pretty simple, we just have to use [HRTBs](https://doc.rust-lang.org/nomicon/hrtb.html) to write the trait bound and a `while let` loop for the actual consumption.
</details>

```rust
fn print_items<I>(mut iter: I)
where
    I: LendingIterator,
    for<'a> I::Item<'a>: Debug,
{
    while let Some(item) = iter.next() {
        println!("{item:?}");
    }
}
```

目前一切正常，这段代码可以顺利编译。接下来我们用一个迭代器真正调用它：

<details>
<summary>原文</summary>

All good so far, this compiles fine. Now we’ll actually call it with an iterator:
</details>

```rust
print_items::<WindowsMut<'_, _, 2>>(windows_mut(&mut [1, 2, 3]));
```

按理说这显然该能编译，因为 `&mut [i32; 2]` 肯定实现了 `Debug`。那我们直接 `cargo run` 看输——

<details>
<summary>原文</summary>

This should obviously compile since &mut [i32; 2] is definitely Debug. So we can just run `cargo run` and see the ou–
</details>

```
error[E0716]: temporary value dropped while borrowed
   --> src/main.rs:45:58
   |
45 |     print_items::<WindowsMut<'_, _, 2>>(windows_mut(&mut [1, 2, 3]));
   |     -----------------------------------------------------^^^^^^^^^--
   |     |                                                    |
   |     |                                                    creates a temporary which is freed while still in use
   |     argument requires that borrow lasts for `'static`
46 | }
   | - temporary value is freed at the end of this statement
```

呃。

<details>
<summary>原文</summary>

oh.
</details>

糟了。

<details>
<summary>原文</summary>

oh no.
</details>

### 哪里出了问题？

<details>
<summary>原文</summary>

What went wrong?
</details>

很明显，这里不太对劲。`rustc` 告诉我们：不知为何，对数组 `[1, 2, 3]` 的借用必须活到 `'static`。但我们根本没写任何 `'static` 约束，这看起来很不合理。我们得暂时站在编译器视角，看看究竟发生了什么。

<details>
<summary>原文</summary>

Clearly, something’s not right here. `rustc` is telling us that for some reason, our borrow of the array `[1, 2, 3]` is required to live for `'static` — but we haven’t written any `'static` bounds anywhere, so this doesn’t really make much sense. We’ll have to put ourselves in the mindset of the compiler for a bit so that we can try to figure out what’s happening.
</details>

首先，我们创建了一个 `WindowsMut<'0, i32, 2>` 迭代器，其中 `'0` 是某个局部生命周期（显然它一定短于 `'static`）。然后把这个迭代器传给 `print_items`，也就是把其泛型参数 `I` 具体化为 `WindowsMut<'0, i32, 2>`。

<details>
<summary>原文</summary>

First of all, we create an iterator of `WindowsMut<'0, i32, 2>`, where `'0` is the name of some local lifetime (notably, this lifetime is necessarily shorter than `'static`). Then we pass this iterator type into the function `print_items`, in doing so setting its I generic parameter to the aforementioned type `WindowsMut<'0, i32, 2>`.
</details>

接下来只需检查特征约束是否成立。把 `print_items` 的 `where` 子句里 `I` 替换成具体类型后，需要验证的是：

<details>
<summary>原文</summary>

So now we just need to make sure that the trait bounds hold. Substituting I for its actual type in the `where` clause of `print_items`, we get this bound that needs to be checked:
</details>

```rust
where
    for<'a> <WindowsMut<'0, i32, 2> as LendingIterator>::Item<'a>: Debug,
```

`for<'a>` 的含义是：右侧位置必须能替换成任意生命周期，并且特征约束依然成立。这里一个典型边界值是 `'static`；若它都不成立，整体 bound 必然失败。于是我们得到：

<details>
<summary>原文</summary>

The `for<'a>` syntax means that we must verify that any lifetime can be substituted in the right hand side and the trait bound must still pass. A good edge case to check here is `'static`, since we know that if that check fails the overall bound will definitely fail. So we end up with this:
</details>

```rust
where
    <WindowsMut<'0, i32, 2> as LendingIterator>::Item<'static>: Debug,
```

换句话说，`WindowsMut` 的关联项类型在传入 `'static` 生命周期时，必须实现 `Debug`。我们回到 `WindowsMut` 对 `LendingIterator` 的实现，看看是否真的成立。相关代码是：

<details>
<summary>原文</summary>

Or in other words, the associated item type of `WindowsMut` must implement `Debug` when fed the lifetime `'static`. Let’s hop back to the implementation of `LendingIterator` for `WindowsMut` to see if that actually holds. As a quick refresher, the relevant bit of code is here:
</details>

```rust
impl<'a, T, const WINDOW_SIZE: usize> LendingIterator
for WindowsMut<'a, T, WINDOW_SIZE>
{
    type Item<'this> = &'this mut [T; WINDOW_SIZE] where 'a: 'this;
    /* ... */
}
```

嗯……有点复杂。把泛型替换成具体类型后会更清楚：

<details>
<summary>原文</summary>

Uhh…that’s a bit complex. Let’s replace the generic types with our concrete ones to simplify it.
</details>

```rust
impl LendingIterator for WindowsMut<'0, i32, 2> {
    type Item<'static> = &'static mut [i32; 2]
    where
        '0: 'static;
}
```

现在问题就很清楚了。前面已经说明，`'0` 是 `[1, 2, 3]` 的局部生命周期，必然短于 `'static`。因此 `'0: 'static` 根本不可能成立，`<WindowsMut<'0, i32, 2> as LendingIterator>::Item<'static>` 这个类型本身就是非法的。既然类型都不存在，编译器当然无法验证它是否实现了 `Debug`。这才是之前报错真正想表达的意思，只是提示比较绕。

<details>
<summary>原文</summary>

And now we can finally see what’s going wrong. As we established earlier, `'0` is the local lifetime of `[1, 2, 3]` and is therefore definitely a shorter lifetime than `'static`. This means that there is absolutely no way that the bound `'0: 'static` will hold, making `<WindowsMut<'0, i32, 2> as LendingIterator>::Item<'static>` an invalid type altogether. So of course the compiler can’t verify that it implements `Debug` — it doesn’t even exist at all! This was what the compiler was really trying to tell us earlier, even if it was a bit obtuse about it.
</details>

最终结论是：HRTB 基本没法直接和生命周期 GAT 搭配使用。`for<'a>` 表达的并不是我们真正想要的约束——我们并不需要“任意生命周期”都满足，只需要“短于 `'0` 的生命周期”满足即可。理想情况下，我们希望在这里写 `where` 子句，让 `print_items` 的约束变成：

<details>
<summary>原文</summary>

The ultimate conclusion of all this is that HRTBs basically can’t be used with lifetime GATs at all. `for<'a>` just doesn’t express the right requirement — we don’t want to require the bound for _any_ lifetime, we only really want to require it for lifetimes _shorter than_ `'0`. Ideally, we would be able to write in a `where` clause there, so the bounds of `print_items` could become:
</details>

```rust
fn print_items<I>(mut iter: I)
where
    I: LendingIterator,
    for<'a where I: 'a> I::Item<'a>: Debug,
```

这样一来，`'static` 就不能被选作 HRTB 的生命周期，因为 `WindowsMut<'0, i32, 2>` 明确不是 `'static`。我们上面的反证逻辑也就不成立了，编译器应当会无问题地接受这段正确代码。

<details>
<summary>原文</summary>

This would mean that `'static` can’t be selected as the lifetime chosen for the HRTB since `WindowsMut<'0, i32, 2>` is _definitely_ not `'static`, so our above proof-by-contradiction would no longer work and the compiler would accept our correct code without problem.
</details>

但遗憾的是，这个特性短期内看不到落地希望。写作时我并未获知有任何相关 RFC 或正式提案（除了 [这个 rust-lang/rust issue](https://github.com/rust-lang/rust/issues/95268)），所以即便未来会支持，也可能要很久才能进稳定版。在那之前，只要你使用生命周期 GAT，就会受一个硬限制：除非 trait 的实现者是 `'static`，否则你无法给 GAT 写特征约束，也无法要求它是某个具体类型。

<details>
<summary>原文</summary>

But unfortunately it doesn’t look like we’ll be getting this feature any time soon. At the time of writing I do not know of any RFC or formal suggestion for this feature (other than [one rust-lang/rust issue](https://github.com/rust-lang/rust/issues/95268)) so it’ll be a long time before it actually arrives on stable should we get it at all. Until then, we’re stuck with a hard limitation every time you use lifetime GATs: you can’t place trait bounds on GATs or require them to be a specific type unless the trait implementor is `'static`.
</details>

这使得真正的GAT在多数场景下几乎不可用。我仍然很高兴它们正在稳定化，但在这个问题解决前，它们恐怕很难在 API 设计中被广泛采用。

<details>
<summary>原文</summary>

This makes real GATs practically unusable for most use cases. I’m still happy they’re being stabilized, but they likely won’t see wide adoption in APIs until this problem is solved.
</details>

那么，在此之前我们能做什么？

<details>
<summary>原文</summary>

So, what can we do in the meantime?
</details>

## 方案 1：把 `dyn Trait` 当作 HKT

<details>
<summary>原文</summary>

Workaround 1: dyn Trait as a HKT
</details>

正如 [@jix](https://github.com/jix) 在 [这篇大纲](https://gist.github.com/jix/42d0e4a36ace4c618a59f0ba03be5bf5) 中最先分享的，一个可行方案是把 `dyn Trait` 当作一种 HKT 使用，因为 `dyn Trait` 的类型里可以写 HRTB，并且关联类型可以随 HRTB 的生命周期变化。

<details>
<summary>原文</summary>

As first shared in [this gist](https://gist.github.com/jix/42d0e4a36ace4c618a59f0ba03be5bf5) by [@jix](https://github.com/jix), one workaround is to use `dyn Trait` as a form of HKT, because `dyn Trait` accepts an HRTB in its type, and supports changing associated types based on the HRTB’s lifetime.
</details>

要在代码中应用这个思路，先把 `LendingIterator` 改成这样：

<details>
<summary>原文</summary>

To implement the design in our code, first we modify the `LendingIterator` trait to look like this:
</details>

```rust
pub trait GivesItem<'a> {
    type Item;
}

pub trait LendingIterator {
    type Item: ?Sized + for<'this> GivesItem<'this>;
    fn next(&mut self) -> Option<<Self::Item as GivesItem<'_>>::Item>;
}
```

关键点在具体类型对 `LendingIterator` 的实现上。`WindowsMut` 的写法如下：

<details>
<summary>原文</summary>

The magic comes in the implementation of `LendingIterator` for specific types. For `WindowsMut` it looks like this:
</details>

```rust
impl<'a, T, const WINDOW_SIZE: usize> LendingIterator
for WindowsMut<'a, T, WINDOW_SIZE>
{
    type Item = dyn for<'this> GivesItem<
        'this,
        Item=&'this mut [T; WINDOW_SIZE],
    >;

    /* ... */
}
```

可以看到，这里的 `Item` 被设为带 HRTB 的 `dyn Trait`，其关联类型依赖于输入的 HRTB 生命周期。因此，尽管 `type Item` 语法上只是一个类型，它实际上像是一个从生命周期到类型的映射函数，行为上很接近“真正的 GAT”。

<details>
<summary>原文</summary>

As you can see, the Item type is set to a `dyn Trait` with an HRTB, where the `dyn Trait`’s associated type depends on the input HRTB lifetime. So even though ```type Item``` is only a single type, it actually acts like a function from a lifetime to a type, just like a real GAT.
</details>

然后把 `print_items` 的签名改成：

<details>
<summary>原文</summary>

We can then modify the signature of print_items like so:
</details>

```rust
fn print_items<I>(mut iter: I)
where
    I: LendingIterator,
    for<'a> <I::Item as GivesItem<'a>>::Item: Debug,
```

结果居然真的可行！

<details>
<summary>原文</summary>

And lo and behold, it works!
</details>

```
[1, 2]
[2, 3]
```

不过，这个方案很快就会遇到一些很麻烦的限制。比如我们在借出式迭代器上定义一个 `map` 操作：

<details>
<summary>原文</summary>

However, this approach runs into some nasty limitations rather quickly. Let’s say that we have now defined a mapping operation on lending iterators:
</details>

```rust
pub fn map<I, F>(iter: I, mapper: F) -> Map<I, F>
where
    I: LendingIterator,
    F: for<'a> Mapper<'a, <I::Item as GivesItem<'a>>::Item>,
{
    Map { iter, mapper }
}

pub struct Map<I, F> {
    iter: I,
    mapper: F,
}

impl<I, F> LendingIterator for Map<I, F>
where
    I: LendingIterator,
    F: for<'a> Mapper<'a, <I::Item as GivesItem<'a>>::Item>,
{
    type Item = dyn for<'this> GivesItem<
        'this,
        Item=<F as Mapper<'this, <I::Item as GivesItem<'this>>::Item>>::Output,
    >;

    fn next(&mut self) -> Option<<Self::Item as GivesItem<'_>>::Item> {
        self.iter.next().map(&mut self.mapper)
    }
}

// Trait helper to allow the lifetime of a mapping function's output to depend
// on its input. Without this, `map` on an iterator would always force lending
// iterators to become non-lending which we don't really want.
pub trait Mapper<'a, I>: FnMut(I) -> <Self as Mapper<'a, I>>::Output {
    type Output;
}

impl<'a, I, F, O> Mapper<'a, I> for F
where
    F: FnMut(I) -> O,
{
    type Output = O;
}
```

然后我们决定不用原始迭代器，改用映射后的迭代器：

<details>
<summary>原文</summary>

and then decide to use a mapped iterator instead of the normal one:
</details>

```rust
let mut array = [1, 2, 3];
let iter = windows_mut::<_, 2>(&mut array);

fn mapper(input: &mut [i32; 2]) -> &mut i32 {
    &mut input[0]
}
let mapped = map(iter, mapper);

print_items::<Map<_, _>>(mapped);
```

这段是可以工作的，会按预期打印 `1` 然后 `2`。

<details>
<summary>原文</summary>

This works fine, printing the desired result of `1` followed by `2`.
</details>

但如果我们临时决定把 `print_items` 的逻辑内联，就会遇到一个不太愉快的惊喜：

<details>
<summary>原文</summary>

But if we suddenly decide that the code in `print_items` should be inlined, we’re in for a not-so-fun little surprise:
</details>

```rust
let mut mapped = map(iter, mapper);

while let Some(item) = mapped.next() {
    println!("{item:?}");
}
```

```
error[E0308]: mismatched types
   --> src/main.rs:97:35
   |
97 |     while let Some(item) = mapped.next() {
   |                                   ^^^^ one type is more general than the other
   |
   = note: expected associated type `<(dyn for<'this> GivesItem<'this, for<'this> Item = &'this mut [i32; 2]> + 'static) as GivesItem<'_>>::Item`
              found associated type `<(dyn for<'this> GivesItem<'this, for<'this> Item = &'this mut [i32; 2]> + 'static) as GivesItem<'this>>::Item`
```

老实说，这条错误信息我完全看不懂在说什么——但我几乎可以确定它是编译器在胡说八道，因为泛型版本明明可以正常工作。

<details>
<summary>原文</summary>

To be honest, I have absolutely no idea what this error message is saying — but I’m pretty sure it’s just nonsense because the generic version works fine.
</details>

这倒不算最糟糕的问题——不方便是有的，但通常都能绕过去。不过，我们还是可以继续提升易用性。

<details>
<summary>原文</summary>

This isn’t the worst problem in the world — it’s inconvenient but it can probably always be worked around. That said, it is still possible to improve the ergonomics.
</details>

## 方案 2：HRTB 超类特征

<details>
<summary>原文</summary>

Workaround 2: HRTB supertrait
</details>

那我们换个思路。从“真正的 GAT”版本重新开始，不过这次把生命周期全都显式写出来（马上你就会知道原因）：

<details>
<summary>原文</summary>

Let’s try a different approach then. We’ll start again from the real GAT version, but this time with explicit lifetimes (you’ll see why in a minute):
</details>

```rust
pub trait LendingIterator {
    type Item<'this> where Self: 'this;
    fn next<'this>(&'this mut self) -> Option<Self::Item<'this>>;
}
```

你会发现这个 trait 里的所有项都用到了 `'this` 生命周期。于是我们可以把这个生命周期提升一层：不再作为每个项的泛型参数，而是作为整个 trait 的泛型参数，从而去掉 GAT。

<details>
<summary>原文</summary>

You’ll notice that all items of the trait use the `'this` lifetime. So we can eliminate the use of GATs by raising that lifetime up one level, to become a generic parameter of the whole trait instead of each item on the trait.
</details>

```rust
pub trait LendingIterator<'this>
// This where bound is raised from the GAT
where
    Self: 'this,
{
    type Item;
    fn next(&'this mut self) -> Option<Self::Item<'this>>;
}
```

这样一来，`for<'a> LendingIterator<'a>` 就和原先的 `LendingIterator` 等价：给定某个生命周期后，我们同时得到 `next` 函数和 `Item` 关联类型。

<details>
<summary>原文</summary>

This way, `for<'a> LendingIterator<'a>` becomes an identical trait to the old `LendingIterator` trait — given a specific lifetime, we get both a `next` function and `Item` associated type.
</details>

不过，这样声明 trait 会有几个问题：

<details>
<summary>原文</summary>

However, there are a few problems with a trait declared this way:
</details>

1. `fn next(&'this mut self)` 很啰嗦，生命周期也不能省略。
2. `for<'a> LendingIterator<'a>` 这个约束又长又不方便写。
3. 像 `for_each` 这类函数，其签名往往要求 `Self: for<'a> LendingIterator<'a>`；但在 `LendingIterator<'this>` 这个 trait 内部并不好表达，因为 HRTB 不在当前层级。

<details>
<summary>原文</summary>

1. `fn next(&'this mut self)` is verbose and doesn’t allow eliding the lifetimes.
2. The trait bound `for<'a> LendingIterator<'a>` is long and inconvenient to spell out.
3. Some functions like `for_each` need `Self` to implement `for<'a> LendingIterator<'a>` in order for their signature to work. But it’s hard to express that within a trait `LendingIterator<'this>` where the HRTB is not already present.
</details>

要解决这些问题，可以把 trait 拆成两个：可带泛型参数（函数）的部分放到外层的无生命周期子 trait；不能带泛型参数的部分（类型）放到内层带生命周期的超类 trait。

<details>
<summary>原文</summary>

To solve them we can split the trait into two, moving the parts that can have generic parameters (functions) into an outer lifetime-less subtrait and the parts that can’t have generic parameters (types) into an inner lifetimed supertrait:
</details>

```rust
pub trait LendingIteratorLifetime<'this>
where
    Self: 'this,
{
    type Item;
}

pub trait LendingIterator: for<'this> LendingIteratorLifetime<'this> {
    fn next(&mut self) -> Option<<Self as LendingIteratorLifetime<'_>>::Item>;
}
```

现在终于可以重新实现 `WindowsMut` 了：

<details>
<summary>原文</summary>

Now we can finally get to reimplementing `WindowsMut`:
</details>

```rust
impl<'this, 'a, T, const WINDOW_SIZE: usize> LendingIteratorLifetime<'this>
for WindowsMut<'a, T, WINDOW_SIZE>
where
Self: 'this,
{
    type Item = &'this mut [T; WINDOW_SIZE];
}

impl<'a, T, const WINDOW_SIZE: usize> LendingIterator
for WindowsMut<'a, T, WINDOW_SIZE>
{
    fn next(&mut self) -> Option<<Self as LendingIteratorLifetime<'_>>::Item> {
        if !self.first {
            self.slice = &mut mem::take(&mut self.slice)[1..];
        }
        self.first = false;

        Some(self.slice.get_mut(..WINDOW_SIZE)?.try_into().unwrap())
    }
}
```

来试试！直接 `cargo build`，然后……

<details>
<summary>原文</summary>

Let’s try it out then! Just run `cargo build` and…
</details>

```
error[E0477]: the type `WindowsMut<'a, T, WINDOW_SIZE>` does not fulfill the required lifetime
   --> src/main.rs:41:39
   |
41 | impl<'a, T, const WINDOW_SIZE: usize> LendingIterator
   |                                       ^^^^^^^^^^^^^^^
```

好吧——到这个阶段我早该知道第一次就成功不太现实。

<details>
<summary>原文</summary>

Right — I should know better than to expect things to work first try at this point.
</details>

这个错误提示非常不友好，但背后其实有合理原因。再次切换到编译器视角：检查 trait 实现时，其中一个任务是验证超类 trait 是否满足。这里需要满足的约束是：

<details>
<summary>原文</summary>

That error’s extremely unhelpful, but there is actually a legitimate explanation for what’s happening here. Once again putting on our compiler hats, one of our jobs when checking a trait implementation is to check whether the supertraits hold. In this case that means we have to satisfy this trait bound:
</details>

```rust
WindowsMut<'a, T, WINDOW_SIZE>: for<'this> LendingIteratorLifetime<'this>
```

和之前一样，检查 HRTB 时一个常见边界值是替换 `'static`。也就是说，上述 bound 成立的必要条件之一是：

<details>
<summary>原文</summary>

Like before, a good edge case to check for with HRTB bounds is whether substituting in `'static` holds. In other words, a necessary condition for the above bound to be satisfied is that this bound is also satisfied:
</details>

```rust
WindowsMut<'a, T, WINDOW_SIZE>: LendingIteratorLifetime<'static>
```

那就来检查。回到 `WindowsMut` 对 `LendingIteratorLifetime` 的实现：

<details>
<summary>原文</summary>

So let’s check that. Jumping to the implementation of `LendingIteratorLifetime` for `WindowsMut`, we see this:
</details>

```rust
impl<'this, 'a, T, const WINDOW_SIZE: usize> LendingIteratorLifetime<'this>
for WindowsMut<'a, T, WINDOW_SIZE>
where
    Self: 'this,
```

把 `'this` 替换成 `'static` 后：

<details>
<summary>原文</summary>

and substituting in `'this` for `'static`:
</details>

```rust
impl<'a, T, const WINDOW_SIZE: usize> LendingIteratorLifetime<'static>
for WindowsMut<'a, T, WINDOW_SIZE>
where
    Self: 'static,
```

……啊，`Self: 'static`。这看起来就是问题所在。

<details>
<summary>原文</summary>

…ah. Self: `'static`. That’s probably a problem.
</details>

确实，如果在 `LendingIterator` 的实现上加 `where Self: 'static`，代码会通过编译：

<details>
<summary>原文</summary>

Indeed, if we add a `where Self: 'static` to the `LendingIterator` implementation it does compile:
</details>

```rust
impl<'a, T, const WINDOW_SIZE: usize> LendingIterator
for WindowsMut<'a, T, WINDOW_SIZE>
where
    Self: 'static,
```

但这显然不是我们想要的——它意味着 `WindowsMut` 只能用于空切片、全局变量、或被泄漏（leak）出来的变量。

<details>
<summary>原文</summary>

But that’s definitely not something we want to do — it would mean that `WindowsMut` would only work on empty slices, global variables and leaked variables.
</details>

这和前面 GAT 版本遇到的问题非常相似：理想情况下，我们希望在 `for<'a>` 约束内部再写一个 `where`，只允许替换成短于 `Self` 的生命周期，从而为非`'static`的`Self`排除`'static` 这类的生命周期。签名大概会像这样：

<details>
<summary>原文</summary>

This is a very similar problem to the one we faced before with the GAT version: ideally, we’d be able to specify a `where` clause within the `for<'a>` bound so that only lifetimes shorter than `Self` could be substituted in, excluding lifetimes like `'static` for non-`'static` `Self`s. The signature could look something like this:
</details>


```rust
pub trait LendingIterator
where
    Self: for<'this where Self: 'this> LendingIteratorLifetime<'this>,
```

但和之前一样，HRTB 里目前并不支持这种 `where` 子句，所以这条路看起来又是死路一条。可惜。

<details>
<summary>原文</summary>

But just as before `where` clauses in HRTBs unfortunately don’t exist yet, so it looks like this is just another dead end. What a shame.
</details>

## HRTB 的隐式约束

<details>
<summary>原文</summary>

HRTB implicit bounds
</details>

> 由于你为 Rust 生态带来可靠稳定的生命周期 GAT 的使命彻底失败，你羞愧难当，干脆退出编程，发誓余生在乡下当个普通土豆农。带着不多的积蓄和一点梦想，你搬进了苏格兰一座破旧的石头农舍，从此过上平静不被打扰的生活。
> 
> <details>
> <summary>原文</summary>
>
> Having failed thoroughly in your mission to bring reliable and stable lifetime GATs to the Rust ecosystem, you quit programming altogether out of shame and vow to live out the rest of your days as a lowly potato farmer in the countryside. With nothing but a small amount savings and a dream, you move in to a run-down stone farmhouse in Scotland where you can live onwards peacefully and undisturbed.
> </details>
>
> 许多年过去了。你早已习惯自然：你亲眼见过植物发芽、枯萎、死去的次数，比 `smallvec` 出过 CVE 的次数还多。四季在你眼里已模糊成一片——白天黑夜、夏天冬天互相交叠，转瞬即逝。你每晚都睡得安稳踏实，因为你知道自己再也不用面对铺天盖地的链接器报错了。你对家周围的每条小路都熟得像本能，闭着眼都能走。这个地方的一切细节都深深刻进你的脑海：每株植物的位置、每个鸟巢的地点、每颗石子的大小形状。
> 
> <details>
> <summary>原文</summary>
>
> Many years pass. You have grown accustomed to nature: you have seen plants grow, wither and die before your eyes more times than smallvec has had CVEs, and the seasons are now no more than a blur — day, night, summer, winter all morphing into one another and passing faster than the blink of an eye. You sleep deeply and peacefully every night, safe and comfortable in the knowledge that you’ll never have to deal with wall of text linker errors ever again. You have become so familiar with the pathways and routes around your home that you can walk them in your sleep. Every single nook and cranny of the place down to the most minute detail is etched deep into your brain: the position of each plant, the location of every nest, the size and shape of each pebble.
> </details>
>
> 所以，在一个寒冷的三月清晨，你第一眼就注意到灌木下露出一截不寻常的白色薄片，也就不足为奇了。走近一看，那是一张纸，被晨露打得微微潮湿。你捡起它，盯着纸上那些神秘符号看了许久；慢慢地——很慢很慢地——一段模糊记忆开始回到脑海。没错，这是 “Rust”。而纸上的这段 “Rust”，似乎是一段很短的程序：
> 
> <details>
> <summary>原文</summary>
>
> So it is no surprise that on one chilly March morning, you immediately notice the abnormal presence of a thin white object sticking out from under a bush. Drawing closer, it appears to be a piece of paper, slightly damp from absorbing the cold morning dew. You pick it up, and as you stare at the mysterious sigils printed on the page, slowly — very slowly — a vague memory begins to come back to you. That’s right, it’s “Rust”. And this “Rust” on the page appears to form a very short program:
> </details>
> 
> ```rust
> fn example<T>(value: T)
> where
> for<'a> &'a T: Debug,
> {
>     eprintln!("{:?}", &value);
> }
> let array = [1, 2, 3];
> example(&array);
> ```
> 
> 你拿着这张神秘纸片往农舍走，一路都在琢磨它到底是什么意思。当然，这代码不可能编译，你对此很确定：`for<'a>` 可以选 `'static`，那就意味着 `&'static T` 必须实现 `Debug`；而对文中的 `&'array [i32; 3]` 来说这显然不成立（`&'static &'array [i32; 3]` 这种类型本身就不可能存在，更别说实现 `Debug`）。
> 
> <details>
> <summary>原文</summary>
>
> As you make your way back to the farmhouse, mysterious piece of paper in hand, you ponder about what it could mean. Of course, there’s no way it would compile, you know _that_ much: `for<'a>` would be able to select `'static` as its lifetime, meaning `&'static T` would need to implement `Debug`, which is obviously not true for the `&'array [i32; 3]` shown (as `&'static &'array [i32; 3]` can’t even exist, let alone be `Debug`).
> </details>
> 
> 那为什么会有人费劲打印一段不能工作的代码，还特意放到你的农场里？你一边想着这个问题，一边从仓库深处翻出旧笔记本。它五年没碰，落了点灰；但你按下电源键后，屏幕还是像多年前一样瞬间亮起，恢复了生机。
>
> <details>
> <summary>原文</summary>
>
> So why would someone go to the effort of printing out code that doesn’t even work — and what’s more, placing it all the way in your farm? It is this that you wonder about while you dig out your old laptop from deep inside storage. It hasn’t been touched for five years, so it’s gotten a little dusty — but you press the power button and screen bursts into colour and life, exactly as it used to do those so many years ago.
> </details>
> 
> 你试探着打开编辑器，把纸上的内容敲了进去。然后想：我当年是怎么构建来着？Shipment？Freight？Haul？不对，是另一个词……啊对，`cargo`。你在终端里敲下了久违多年的命令：
> 
> <details>
> <summary>原文</summary>
>
> Tentatively, you open a text editor, and begin copying out the contents of that paper inside it. Now, how do I build it again? Shipment? Freight? Haul? No, it was something different…ah, cargo, that was it. Into the shell you type out the words you haven’t seen for so, so long:
> </details>
> 
> ```shell
> cargo run
> ```
> 
> 你深吸一口气，按下回车。风扇转起，CPU 开始忙碌。短短一会儿却像过了很久，Cargo 显示着 “Building”；最终它完成了，屏幕滚出一行字：
> 
> <details>
> <summary>原文</summary>
>
> You take a deep breath, and then press the enter key. The fan whirrs as the CPU starts into life. For a short moment that feels like an eon, Cargo displays “Building” — but eventually it finishes, and as it does, one line of text rolls down the screen:
> </details>
> 
> ```
> [1, 2, 3]
> ```

等等，什么？再来一次。

<details>
<summary>原文</summary>

Wait, what? Do that again.
</details>

> 你又深吸一口气，按下回车。风扇嗡鸣，CPU 运转。Cargo 短暂地显示 “Building”，最后结束时，屏幕上还是那一行：
> 
> <details>
> <summary>原文</summary>
>
> You take a deep breath, and then press the enter key. The fan whirrs as the CPU starts into life. For a short moment that feels like an eon, Cargo displays “Building” — but eventually it finishes, and as it does, one line of text rolls down the screen:
> </details>
>
> ```
> [1, 2, 3]
> ```

看来不是偶然。但这完全说不通：按我们熟悉的规则，这段代码 _根本不可能_ 编译通过。那到底发生了什么？

<details>
<summary>原文</summary>

So it wasn’t just a fluke. But that makes no sense at all: by all the rules we knew, there is _no way_ that code should’ve compiled. So what’s happening here?
</details>

答案是：虽然 `for<'a>` 不支持显式 `where` 子句，但它在某些情况下会带有 _隐含的_ `where` 子句——这里就是 `for<'a where I: 'a>`。不过这只会在特定场景出现：当 HRTB 作用的类型或特征约束中存在 _隐式约束_ 时，这个隐式约束会被转发到 HRTB 的隐式 `where` 子句里。

<details>
<summary>原文</summary>

The answer is that while `for<'a>` does not support explicit `where` clauses, it actually can, sometimes, have an _implied_ `where` clause — in this case, it’s `for<'a where I: 'a>`. But it only occurs in specific scenarios: in particular, when there is an _implicit bound_ in the type or trait bound the HRTB is applied to, that implicit bound gets forwarded to the implicit `where` clause of the HRTB.
</details>

所谓隐式约束，就是确实存在、但没有在泛型或 `where` 子句里用冒号显式写出来的 trait bound。正如上例所示，`&'a T` 含有 `T: 'a` 的隐式约束——这是一条非常基础的规则，用来避免 `&'static &'short_lifetime i32` 这类无意义类型（外层引用活得比被借用内容更久）。正是这条规则让 `for<'a> &'a T` 的行为等价于 `for<'a where T: 'a> &'a T`，从而使那段代码能够运行并输出 `[1, 2, 3]`。

<details>
<summary>原文</summary>

An implicit bound is a trait bound that is present, but not stated explicitly by a colon in the generics or `where` clause. As you can infer from the example above, `&'a T` contains an implicit bound for `T: 'a` — this is a really simple rule to prevent nonsense types like `&'static &'short_lifetime i32` (a reference that outlives borrowed contents). It’s this rule that causes `for<'a> &'a T` to act like it’s actually `for<'a where T: 'a> &'a T`, enabling that code to run and successfully print `[1, 2, 3]`.
</details>

隐式约束也会出现在结构体上。例如这个结构体：

<details>
<summary>原文</summary>

Implicit bounds can appear on structs too. For example, take this struct:
</details>

```rust
#[derive(Debug)]
struct Reference<'a, T>(&'a T);
```

由于 `&'a T` 带有 `T: 'a` 的隐式约束，`Reference` 结构体 _同样_ 会带上 `T: 'a` 的隐式约束。可以通过下面这段可编译代码来验证：

<details>
<summary>原文</summary>

Because `&'a T` has an implicit bound of `T: 'a`, the struct `Reference` _also_ has an implicit bound of `T: 'a`. You can prove this because this code compiles:
</details>

```rust
fn example<T>(value: T)
where
    for<'a /* where T: 'a */> Reference<'a, T>: Debug,
{
    dbg!(Reference(&value));
}

let array = [1, 2, 3];
example(&array);
```

然而，一旦你尝试把这个隐式约束改成显式约束，就会发现代码不再能编译：

<details>
<summary>原文</summary>

However, as soon as you try to upgrade the implicit bound to an explicit one you will notice it no longer compiles:
</details>

```rust
#[derive(Debug)]
struct Reference<'a, T: 'a>(&'a T);

fn example<T>(value: T)
where
    for<'a> Reference<'a, T>: Debug,
{
    dbg!(Reference(&value));
}

let array = [1, 2, 3];
example(&array);
```

```
error[E0597]: `array` does not live long enough
   --> src/main.rs:15:13
   |
15 |     example(&array);
   |     --------^^^^^^-
   |     |       |
   |     |       borrowed value does not live long enough
   |     argument requires that `array` is borrowed for `'static`
16 | }
   | - `array` dropped here while still borrowed
```

HRTB 里的隐式约束……是 Rust 中一个非常玄学的特性。我至今不确定它是语言有意设计，还是当前实现的某种隐蔽副作用。但不管怎样，它对我们非常有用。如果能把它利用到 `LendingIterator` 的超 trait HRTB 上，也许我们就能在不加 `'static` 约束的前提下让方案真正跑通。谢谢你，神秘纸条。

<details>
<summary>原文</summary>

Implicit bounds in HRTBs are…a very weird feature of Rust. I’m still not sure whether they are intended to exist or are just an obscure side-effect of the current implementation. But either way, this is an incredibly useful feature for us. If we can somehow leverage this to apply it in our supertrait HRTB of `LendingIterator`, then we can maybe get it to actually work without the `'static` bound! Thanks, mysterious piece of paper.
</details>

## 方案 3：更好的 GAT

<details>
<summary>原文</summary>

Workaround 3: The better GATs
</details>

有了对隐含约束的新认知，我们要做的就是让它和 `for<'a> LendingIteratorLifetime<'a>` 这个超 trait 协同工作。一种做法是给 `LendingIteratorLifetime` 增加一个哑元类型参数，让 HRTB 借此触发自己的隐式约束：

<details>
<summary>原文</summary>

Armed with our new knowledge of implied bounds, all we have to do is get it to work in conjunction with that `for<'a> LendingIteratorLifetime<'a>` supertrait. One way to achieve this is to introduce a new dummy type parameter to `LendingIteratorLifetime`, so that HRTBs can make use of it to apply their own implicit bounds:
</details>

```rust
pub trait LendingIteratorLifetime<'this, ExtraParam> {
    type Item;
}

pub trait LendingIterator
where
    Self: for<'this /* where Self: 'this */>
    LendingIteratorLifetime<'this, &'this Self>,
{
    fn next(&mut self) -> Option<<Self as LendingIteratorLifetime<'_, &Self>>::Item>;
}
```

这 _能工作_，但每次都得手写 `&'this Self`，非常烦。可以用默认类型参数稍微改善易用性：

<details>
<summary>原文</summary>

This _works_, but it’s a pain to have to write out `&'this Self` every time you want to use the trait. Ergonomics can be improved slightly by using a default type parameter:
</details>

```rust
// Give every usage of this trait an implicit `where Self: 'this` bound
pub trait LendingIteratorLifetime<'this, ImplicitBounds = &'this Self> {
    type Item;
}

pub trait LendingIterator
where
    Self: for<'this /* where Self: 'this */> LendingIteratorLifetime<'this>,
{
    fn next(&mut self) -> Option<<Self as LendingIteratorLifetime<'_>>::Item>;
}
```

还有一个小优化：为了避免调用方误把 `ImplicitBounds` 设成 `&'this Self` 之外的类型，我们可以使用 sealed 类型和 trait。这样就得到我目前推荐的定义：

<details>
<summary>原文</summary>

There is still one slight improvement we can make to reduce the chance the API is accidentally misused by setting the `ImplicitBounds` parameter to something other than `&'this Self`, and that is using a sealed type and trait. This leads to my current recommended definition for this trait:
</details>

```rust
pub trait LendingIteratorLifetime<'this, ImplicitBounds: Sealed = Bounds<&'this Self>> {
    type Item;
}

mod sealed {
    pub trait Sealed: Sized {}
    pub struct Bounds<T>(T);
    impl<T> Sealed for Bounds<T> {}
}
use sealed::{Bounds, Sealed};

pub trait LendingIterator: for<'this> LendingIteratorLifetime<'this> {
    fn next(&mut self) -> Option<<Self as LendingIteratorLifetime<'_>>::Item>;
}
```

有了新 trait，我们可以把 `WindowsMut` 改写为：

<details>
<summary>原文</summary>

New trait in hand, we can rewrite our type `WindowsMut` to use it:
</details>

```rust
impl<'this, 'a, T, const WINDOW_SIZE: usize> LendingIteratorLifetime<'this>
for WindowsMut<'a, T, WINDOW_SIZE>
{
    type Item = &'this mut [T; WINDOW_SIZE];
}

impl<'a, T, const WINDOW_SIZE: usize> LendingIterator
for WindowsMut<'a, T, WINDOW_SIZE>
{
    fn next(&mut self) -> Option<<Self as LendingIteratorLifetime<'_>>::Item> {
        if !self.first {
            self.slice = &mut mem::take(&mut self.slice)[1..];
        }
        self.first = false;

        Some(self.slice.get_mut(..WINDOW_SIZE)?.try_into().unwrap())
    }
}
```

`Map` 也可以照样改（`Mapper` trait 仍然需要）：

<details>
<summary>原文</summary>

as well as `Map` (the `Mapper` trait is still needed):
</details>

```rust
impl<'this, I, F> LendingIteratorLifetime<'this> for Map<I, F>
where
    I: LendingIterator,
    F: for<'a> Mapper<'a, <I as LendingIteratorLifetime<'a>>::Item>,
{
    type Item = <F as Mapper<
        'this,
        <I as LendingIteratorLifetime<'this>>::Item,
    >>::Output;
}

impl<I, F> LendingIterator for Map<I, F>
where
    I: LendingIterator,
    F: for<'a> Mapper<'a, <I as LendingIteratorLifetime<'a>>::Item>,
{
    fn next(&mut self) -> Option<<Self as LendingIteratorLifetime<'_>>::Item> {
        self.iter.next().map(&mut self.mapper)
    }
}
```

而且和“真正的 GAT”以及[方案 1](#方案-1把-dyn-trait-当作-hkt)不同，这个方案无论是 _直接_ 消费具体类型，还是通过泛型 `print_items` 函数消费，都能工作。完美！

<details>
<summary>原文</summary>

and unlike both real GATs and [workaround 1](#方案-1把-dyn-trait-当作-hkt), this works with both consuming the concrete type _directly_ and through the generic `print_items` function. Perfect!
</details>

## Dyn 安全性

<details>
<summary>原文</summary>

Dyn safety
</details>

和方案 1 相比，方案 3 的主要缺点是它不是 `dyn`-safe（对象安全）的。如果你尝试把它当 trait object 用，`rustc` 会“贴心”地告诉你：

<details>
<summary>原文</summary>

The main disadvantage of workaround 3 in comparison to workaround 1 is that it is not `dyn`-safe. If you try to use it as a trait object, `rustc` helpfully tells you this:
</details>

```
note: for a trait to be "object safe" it needs to allow building a vtable to allow the call to be resolvable dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-safety>
   --> src/main.rs:14:28
   |
14 | pub trait LendingIterator: for<'this> LendingIteratorLifetime<'this> {
   |           ---------------  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ...because it uses `Self` as a type parameter
   |           |
   |           this trait cannot be made into an object...
```

这里说“because it uses `Self` as a type parameter”，其实指的是我们塞进去的隐藏默认参数 `Bounds<&'this Self>`。因此，让 `LendingIterator` 直接支持 `dyn` 基本不可能。

<details>
<summary>原文</summary>

When it says “because it uses `Self` as a type parameter” it’s actually referring to the hidden `Bounds<&'this Self>` default parameter we inserted. As a result, making `LendingIterator` directly work with `dyn` is simply not possible.
</details>

但这 _不_ 意味着动态分发就完全无解——我们只需要再定义一个辅助 trait！只要这个辅助 trait 采用方案 1，它就可以做到对象安全。代价是 trait object 的使用体验会稍差一些（受具体类型上的编译器 bug 影响），但这点目前基本无能为力。

<details>
<summary>原文</summary>

But that is _not_ to say that dynamic dispatch is altogether impossible — all we have to do is define a helper trait for it! And as long as that helper trait uses workaround 1, it will be perfectly object-safe. This does lead to slightly worse ergnomics when using trait objects (due to that compiler bug with concrete types) but there really isn’t much we can do about that.
</details>

先把旧版 `LendingIterator` 定义拿回来，不过这次改名为 `ErasedLendingIterator`：

<details>
<summary>原文</summary>

So let’s start by bringing back our old definition of `LendingIterator`, but this time under the name `ErasedLendingIterator`:
</details>

```rust
pub trait LendingIteratorGats<'a> {
    type Item;
}

pub trait ErasedLendingIterator {
    type Gats: ?Sized + for<'this> LendingIteratorGats<'this>;
    fn erased_next(&mut self) -> Option<<Self::Gats as LendingIteratorGats<'_>>::Item>;
}
```

接着，为所有 `LendingIterator` 提供这个 trait 的覆盖实现：

<details>
<summary>原文</summary>

Next, we add a blanket implementation of this trait for all `LendingIterators`:
</details>

```rust
impl<I: ?Sized + LendingIterator> ErasedLendingIterator for I {
    type Gats = dyn for<'this> LendingIteratorGats<
        'this,
        Item = <I as LendingIteratorLifetime<'this>>::Item,
    >;

	fn erased_next(&mut self) -> Option<<Self::Gats as LendingIteratorGats<'_>>::Item> {
		self.next()
	}
}
```

最后，在我们拥有的所有 trait object 上实现常规 `LendingIterator` trait：

<details>
<summary>原文</summary>

Finally, we implement the regular `LendingIterator` trait on all the trait objects we own:
</details>

```rust
impl<'this, Gats> LendingIteratorLifetime<'this>
for dyn '_ + ErasedLendingIterator<Gats = Gats>
where
    Gats: ?Sized + for<'a> LendingIteratorGats<'a>,
{
    type Item = <Gats as LendingIteratorGats<'this>>::Item;
}

impl<Gats> LendingIterator
for dyn '_ + ErasedLendingIterator<Gats = Gats>
where
    Gats: ?Sized + for<'a> LendingIteratorGats<'a>,
{
    fn next(&mut self) -> Option<<Self as LendingIteratorLifetime<'_>>::Item> {
        self.erased_next()
    }
}

// omitted implementations for all the permutations of auto traits. in a real
// implementation, you'd probably use a macro to generate all 32 versions
// (since there are 5 auto traits)
```

这是把非对象安全的特征包装成对象安全版本的相当标准的样板代码，所以这里不再展开细讲。

<details>
<summary>原文</summary>

This is fairly standard boilerplate for defining an object-safe version of a non-object-safe trait, so I won’t explain it in great detail here.
</details>

很好，来试试！这里我们可以创建一个迭代器，遍历大小为 2 或者 3 的窗口。

<details>
<summary>原文</summary>

Great, let’s try it out! Here, we can use it to create an iterator over either windows of size 2 or windows of size 3.
</details>

```rust
let mut array = [1, 2, 3, 4];

fn unsize<const N: usize>(array: &mut [i32; N]) -> &mut [i32] {
    array
}

type Gats = dyn for<'a> LendingIteratorGats<'a, Item = &'a mut [i32]>;
type Erased<'iter> = dyn 'iter + ErasedLendingIterator<Gats = Gats>;

let mut iter: Box<Erased<'_>> = if true {
    Box::new(map(windows_mut::<_, 2>(&mut array), unsize))
} else {
    Box::new(map(windows_mut::<_, 3>(&mut array), unsize))
};

while let Some(item) = iter.next() {
    println!("{item:?}");
}
```

然后执行 `cargo build`……

<details>
<summary>原文</summary>

and `cargo build` it…
</details>

```
error: implementation of `LendingIteratorLifetime` is not general enough
    --> src/main.rs:166:3
    |
166 |         Box::new(map(windows_mut::<_, 2>(&mut array), unsize))
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ implementation of `LendingIteratorLifetime` is not general enough
    |
    = note: `Map<WindowsMut<'_, i32, 2_usize>, for<'r> fn(&'r mut [i32; 2]) -> &'r mut [i32] {unsize::<2_usize>}>` must implement `LendingIteratorLifetime<'0>`, for any lifetime `'0`...
    = note: ...but it actually implements `LendingIteratorLifetime<'1>`, for some specific lifetime `'1`
```

……啊，又是一条谜语人报错。

<details>
<summary>原文</summary>

…ah. Another cryptic error.
</details>

我认为这里发生的事和[方案 1](#workaround-1-dyn-trait-as-a-hkt)遇到的易用性问题本质相同：编译器存在某个 bug，导致这种写法在具体类型上失效。

<details>
<summary>原文</summary>

I believe what’s happening here is the same ergonomics issue as faced with [workaround 1](#workaround-1-dyn-trait-as-a-hkt): There’s some compiler bug which makes this not work with concrete types.
</details>

因此修复方法也很直接：把它移进泛型函数！而且这个版本确实可以编译：

<details>
<summary>原文</summary>

So that means all we have to do to fix it is to move it into a generic function! And indeed this version does compile:
</details>

```rust
fn box_erase<'iter, I>(iter: I) -> Box<Erased<'iter>>
where
    I: 'iter + LendingIterator,
    I: for<'a> LendingIteratorLifetime<'a, Item = &'a mut [i32]>,
{
    Box::new(iter)
}

let mut iter: Box<Erased<'_>> = if true {
    box_erase(map(windows_mut::<_, 2>(&mut array), unsize))
} else {
    box_erase(map(windows_mut::<_, 3>(&mut array), unsize))
};
```

但还可以更进一步：泛型只是抹去具体类型的一种方式，也可以通过返回位置的 `impl Trait` 来做。

<details>
<summary>原文</summary>

But we can do better than that, because generics are only one way to erase a value’s concrete type: you can also do it via return-position `impl Trait`.
</details>

```rust
fn funnel_opaque<'iter, I>(iter: I) -> impl 'iter + ErasedLendingIterator<Gats = Gats>
where
    I: 'iter + LendingIterator,
    I: for<'a> LendingIteratorLifetime<'a, Item = &'a mut [i32]>,
{
    iter
}

let mut iter: Box<Erased<'_>> = if false {
    Box::new(funnel_opaque(map(windows_mut::<_, 2>(&mut array), unsize)))
} else {
    Box::new(funnel_opaque(map(windows_mut::<_, 3>(&mut array), unsize)))
};
```

这个也行。

<details>
<summary>原文</summary>

And this also works.
</details>

如果愿意，你还可以把 `funnel_opaque` 进一步泛化，让它支持任意 `&'a mut T`，而不只是 `&'a mut [i32]`：

<details>
<summary>原文</summary>

If you want to, you can generalize `funnel_opaque` further so that it works with any `&'a mut T` type instead of just `&'a mut [i32]`:
</details>

```rust
type Gats<T> = dyn for<'a> LendingIteratorGats<'a, Item = &'a mut T>;
type Erased<'iter, T> = dyn 'iter + ErasedLendingIterator<Gats = Gats<T>>;

fn funnel_opaque<'iter, I, T>(iter: I) -> impl 'iter + ErasedLendingIterator<Gats = Gats<T>>
where
    T: ?Sized,
    I: 'iter + LendingIterator,
    I: for<'a> LendingIteratorLifetime<'a, Item = &'a mut T>,
{
    iter
}

let mut iter: Box<Erased<'_, [i32]>> = if false {
    Box::new(funnel_opaque(map(windows_mut::<_, 2>(&mut array), unsize)))
} else {
    Box::new(funnel_opaque(map(windows_mut::<_, 3>(&mut array), unsize)))
};
```

但遗憾的是，你没法把它彻底泛化到任意 `LendingIterator`，因为又会撞上同一个编译器 bug。

<details>
<summary>原文</summary>

But unfortunately you can’t generalize it completely to any `LendingIterator`, because you just run into that compiler bug again.
</details>

## 总结

<details>
<summary>原文</summary>

Conclusion
</details>

以上就是全部内容。就我所知，这套技巧是目前在 Rust 里使用生命周期 GAT 的最佳方式。即便将来“真正的 GAT”稳定了，我预计它在很长一段时间内仍然有价值，所以你不妨提前熟悉一下。

<details>
<summary>原文</summary>

So there we have it - this technique is, to my knowledge, the best way to use lifetime GATs in Rust. Even once real GATs become stabilized, I predict it’ll likely still be useful for a long time to come, so you might want to familiarize yourself with it.
</details>

