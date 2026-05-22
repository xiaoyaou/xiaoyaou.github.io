---
layout: post
title: "Bevy核心：依赖注入"
date: 2026-05-20
tags: [Rust, dependency injection, bevy]
original: https://taintedcoders.com/bevy/building-bevy
---

# 构建 Bevy

<details markdown="1">
<summary>原标题</summary>

# Building Bevy
</details>

> **Bevy 版本：**0.18  
> **最后更新：**2026 年 1 月 18 日

<details markdown="1">
<summary>原文</summary>

> **Bevy version:** 0.18  
> **Last updated:** January 18, 2026
</details>

这篇文章是在一篇[原始教程](https://promethia-27.github.io/dependency_injection_like_bevy_from_scratch/introductions.html)
基础上的个人改写，并加入了不少实质性的补充。

<details markdown="1">
<summary>原文</summary>

This article is a personal rewriting with some substantial additions of an [original tutorial](https://promethia-27.github.io/dependency_injection_like_bevy_from_scratch/introductions.html).

</details>

Bevy 提供了一种非常符合人体工学的方式来添加我们的[系统](https://taintedcoders.com/bevy/systems)。它们看起来像普通的 Rust 函数，但会带有一些特殊参数。

<details markdown="1">
<summary>原文</summary>

Bevy has a highly ergonomic way for us to add our [systems](https://taintedcoders.com/bevy/systems). They look like normal rust functions, but they have some special parameters.
</details>

```rust
// Here we declared a `query` which is injected in by our App during a game tick
fn movement_system(query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in &query {
        // ...
    }
}
```

随后我们会把这些系统调度到 `App` 中，并在主循环的特定阶段运行它们。

<details markdown="1">
<summary>原文</summary>

We then schedule these systems into our `App` which will run them at a specific part of our main loop.
</details>

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, movement_system)
        .run();
}
```

有趣的是，我们从未显式地把 `query` 传给 `movement_system`。但我们的 `App` 不知怎地知道在运行时把包含这些特定组件的查询传给我们的系统。

<details markdown="1">
<summary>原文</summary>

Interestingly, we never provided the `query` explicitly to our `movement_system`. Somehow our `App` knows to pass that query with those specific components to our system at runtime.
</details>

这篇文章要探讨的，就是 Bevy 究竟如何借助一种叫作**依赖注入**的技术做到这一点。

<details markdown="1">
<summary>原文</summary>

This post is an exploration of how exactly Bevy accomplishes this using a technique called **dependency injection**.
</details>

---

## 依赖注入

<details markdown="1">
<summary>原标题</summary>

## Dependency injection
</details>

之前我们说“系统就是普通的 Rust 函数”，但这并不完全准确。每个系统**只**接收实现了 `SystemParam` trait 的参数。

<details markdown="1">
<summary>原文</summary>

Before we said that "our systems are normal rust functions", but that's not quite true. Each system **only** takes parameters that implement the `SystemParam` trait.
</details>

每个 `SystemParam` 都有两个关联类型：

<details markdown="1">
<summary>原文</summary>

Every `SystemParam` has two associated types:
</details>

1. `State`：用于存储持久化数据。

2. `Item`：在构造系统参数时返回的类型。

<details markdown="1">
<summary>原文</summary>

1. `State` which is used to store persistent data

2. `Item` which is what gets returned when constructing the system param
</details>

在游戏循环时，Bevy 会使用 `Item` 来构造你系统参数的一个实例，并给它全新的生命周期，这个生命周期会持续到当前系统执行结束。

<details markdown="1">
<summary>原文</summary>

When your game loops, Bevy will use the `Item` to construct another instance of your system parameter with brand new lifetimes which extend to the duration of the currently executing system.
</details>

因此，不是我们在函数内部自己初始化这些状态，而是 Bevy 根据类型推断如何取出我们需要的数据并把它们注入进来。

<details markdown="1">
<summary>原文</summary>

So instead of us initializing this state within our functions, Bevy uses our types to figure out how to retrieve the data we need and injects them in.
</details>

要在 Rust 里真正实现依赖注入，可以借助 trait。为了用一个简单例子说明，我们先定义一个日志 trait：

<details markdown="1">
<summary>原文</summary>

To actually implement dependency injection in Rust we can use traits. To illustrate this with a simple example, lets create a logger trait:
</details>

```rust
trait Logger {
    fn log(&self, message: &str);
}
```

然后我们可以创建一个实现了该 trait 的简单控制台日志器：

<details markdown="1">
<summary>原文</summary>

Then we can make a simple console logger that implements this trait:
</details>

```rust
struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn log(&self, message: &str) {
        println!("{}", message);
    }
}
```

接着我们可以设想一个需要日志器的东西，任何类型的日志器都行：

<details markdown="1">
<summary>原文</summary>

Later, we can imagine something that needs a logger, any kind of logger:
</details>

```rust
// The logger is anything that implements the `Logger` trait
struct DoSomething<T: Logger> {
    logger: T,
}

// This is the generic implementation for anything implementing logger
impl<T: Logger> DoSomething<T> {
    fn new(logger: T) -> Self {
        Self { logger }
    }

    fn perform_action(&self) {
        self.logger.log("Action performed.");
    }
}
```

关键点在于：`DoSomething` 的实现不依赖某个具体的 `Logger`，只要求接收到的对象实现了 `Logger` trait。

<details markdown="1">
<summary>原文</summary>

The important part is that the `DoSomething` implementation does not depend on any specific `Logger`, just that whatever it receives implements the `Logger` trait.
</details>

这就是依赖注入：我们把 `Logger` 这个依赖提供给结构体，它就能在不关心具体实现的情况下使用它。而我们已经把依赖注入到了这个结构体中。

<details markdown="1">
<summary>原文</summary>

This is dependency injection. We provide the `Logger` dependency to the struct and it can use it without caring about the specific implementation. We have injected the dependency into the struct.
</details>

---

## 从零开始

<details markdown="1">
<summary>原标题</summary>

## Starting from scratch
</details>

这篇教程的目标是重建 Bevy 依赖注入机制的核心。所以我们会尽量贴近 Bevy 的实现。

<details markdown="1">
<summary>原文</summary>

The goal of this tutorial will be to recreate the core of Bevy's dependency injection. So we will stick as close to Bevy's implementation as possible.
</details>

先勾勒一下它大概会是什么样子：

<details markdown="1">
<summary>原文</summary>

Lets outline what this would look like:
</details>

1. 我们的系统应当是带有特殊参数的简单 Rust 函数。

2. 我们需要一个地方保存系统和资源，以便在循环期间访问。

3. 系统需要能够**自动地**接收任意数量的这些参数并被调用。

<details markdown="1">
<summary>原文</summary>

1. Our systems should be simple rust functions with special parameters

2. We need a place to store our systems and resources so they can be accessed during the loop

3. Systems need to be callable with any number of these parameters **automagically**
</details>

首先，我们找一个地方来存放系统和资源。把它叫作 `Scheduler`：

<details markdown="1">
<summary>原文</summary>

To start off, lets find a place to store our systems and resources. Let's call it a `Scheduler`:
</details>

```rust
use std::any::{Any, TypeId};
use std::collections::HashMap;

struct Scheduler {
    systems: Vec<StoredSystem>,
    resources: HashMap<TypeId, Box<dyn Any>>,
}

struct StoredSystem;
```

为了简化初始范围，我们暂时先不处理任何借用类型。

<details markdown="1">
<summary>原文</summary>

To simplify our initial scope we won't do any borrowing types just yet.
</details>

相反，我们先让系统参数全部使用 `'static` 生命周期。系统可以是任意接收 `'static` 生命周期参数并返回 `()` 的函数。

<details markdown="1">
<summary>原文</summary>

Instead we will start with just the `'static` lifetime for all parameters of our systems. Systems will be any function that takes parameters with a `'static` lifetime and returns `()`.
</details>

这里的静态生命周期只是表示它们会持续整个程序运行期间，不会离开作用域并被清理。

<details markdown="1">
<summary>原文</summary>

A static lifetime here just means that they will last the entire lifetime of our program, they never go out of scope and get cleaned up.
</details>

---

## 创建可调用系统

<details markdown="1">
<summary>原标题</summary>

## Creating callable systems
</details>

为了让系统能够被传来传去并且可调用，我们需要实现 `FnMut`。

<details markdown="1">
<summary>原文</summary>

To allow us to pass our systems around and have them be callable we will need to implement `FnMut`.
</details>

一个实现 `FnMut` 的简单示例如下：

<details markdown="1">
<summary>原文</summary>

A simple example of implementing `FnMut` would look like:
</details>

```rust
// Enable unstable features required for implementing FnMut
#![feature(unboxed_closures, fn_traits)]

// Define a struct that implements `FnMut`
struct Counter {
    count: u32,
}

impl FnMut<()> for Counter {
    // Define the `call_mut` method required by `FnMut`
    extern "rust-call" fn call_mut(&mut self, _args: ()) -> u32 {
        self.count += 1;
        self.count
    }
}

// Also implement FnOnce, which is required by FnMut
impl FnOnce<()> for Counter {
    type Output = u32;

    extern "rust-call" fn call_once(mut self, _args: ()) -> Self::Output {
        self.count += 1;
        self.count
    }
}

fn main() {
    let mut counter = Counter { count: 0 };

    // Call the `Counter` instance as if it were a function
    let result = counter();

    println!("Result: {}", result); // Output: Result: 1

    // Call it again to show it's mutable
    println!("Second call: {}", counter());  // Output: Second call: 2
    println!("Third call: {}", counter());   // Output: Third call: 3
}
```

实现 `FnMut` 会把 `Counter` 结构体变成可调用对象。这正是我们确切希望 `System` 所具备的行为。

<details markdown="1">
<summary>原文</summary>

Implementing `FnMut` turns the `Counter` struct into something callable. Exactly the kind of behavior we would want from our `System`.
</details>

这看起来足够简单，但我们很快会遇到 Rust 的核心限制之一：**没有可变参数泛型**。所以我们无法定义参数数量可变的泛型类型。

<details markdown="1">
<summary>原文</summary>

This seems simple enough, but we quickly run into one of the core limitations of rust: **no variadic generics**. So we can't have generic types with a variable number of arguments.
</details>

---

## 处理可变参数泛型问题

<details markdown="1">
<summary>原标题</summary>

## Handling variadic generics
</details>

很遗憾，这意味着我们需要针对系统可能接收的每一种泛型参数数量分别写实现。

<details markdown="1">
<summary>原文</summary>

So unfortunately, we would need to create implementations for each number of generic arguments we want our systems to take.
</details>

```rust
trait System<Input> {}

// An implementation of systems that take no parameters:
impl<F: FnMut()> System<()> for F {}

// Here we implement our system with a single parameter:
impl<F: FnMut(T1), T1: 'static> System<(T1,)> for F {}

// etc...
```

为了避免手写每一个实现，我们可以写一个宏：

<details markdown="1">
<summary>原文</summary>

To not have to write out each implementation manually, we can write a macro:
</details>

```rust
macro_rules! impl_system {
  (
      $($params:ident),*
  ) => {
    impl<
      F: FnMut($($params),*) $( ,$params: 'static),* >
    System<($($params),*)> for F {}
  }
}

impl_system!();
impl_system!(T1);
impl_system!(T1, T2);
impl_system!(T1, T2, T3);
impl_system!(T1, T2, T3, T4);
// ... etc
```

这个宏会在编译期展开成和之前手写实现完全一致的代码。这样后续修改时，我们只需要改宏，而不是逐个实现都改一遍。

<details markdown="1">
<summary>原文</summary>

This macro will be expanded at compile time to become exactly what we had before. When we make any changes we only have to change the macro, and not each of the implementations individually.
</details>

稍后我们可以很方便地把函数实现也放进这些宏里。

<details markdown="1">
<summary>原文</summary>

This will come in handy later when we put implementations of functions inside these macros.
</details>

---

## 泛型方式调用系统

<details markdown="1">
<summary>原标题</summary>

## Calling systems generically
</details>

要让 `Scheduler` 能以泛型方式调用系统，我们需要一个统一的调用接口。

<details markdown="1">
<summary>原文</summary>

For our `Scheduler` to be able to call our systems generically we would need a common interface to invoke them.
</details>

> _**“我们怎么才能用一个函数签名调用所有这些系统？”**_

<details markdown="1">
<summary>原文</summary>

> _**"How can we have one function signature that can call any of these systems?"**_
</details>

我们需要通过提供一个满足全部实现的单一参数，让我们的输入扁平化。这样调用方就不必关心每个系统参数的具体类型。

<details markdown="1">
<summary>原文</summary>

We need to expose some way to flatten our input by providing a single parameter that satisfies all implementations. That way our caller doesn't have to care about the specific types of each system.
</details>

所以我们定义一个接收统一 `resources` 参数的 `.run` 方法：

<details markdown="1">
<summary>原文</summary>

So lets define a `.run` method that takes a unified `resources` parameter:
</details>

```rust
trait System<Input> {
    fn run(&mut self, resources: &mut HashMap<TypeId, Box<dyn Any>>);
}
```

然后把它加进宏实现中：

<details markdown="1">
<summary>原文</summary>

Which we can then add to our macro implementation:
</details>

```rust
macro_rules! impl_system {
  (
      $($params:ident),*
  ) => {
    #[allow(non_snake_case, unused)]
    impl<F: FnMut($($params),*) $( ,$params: 'static)* >
    System<( $($params),* )> for F {
      fn run(&mut self, resources: &mut HashMap<TypeId, Box<dyn Any>>) {
        $(
          let $params = resources.remove(&TypeId::of::<$params>()).unwrap().downcast::<$params>().unwrap();
        )*

        (self)( $($params),* );
      }
    }
  }
}
```

现在，不需要知道具体参数类型也可以调用我们的系统了。

<details markdown="1">
<summary>原文</summary>

Now our systems can be called without knowing their specific parameter types.
</details>

我们还希望能使用 `Box<dyn System>`，而不是 `Box<dyn Any>`。

<details markdown="1">
<summary>原文</summary>

We would also like to be able to use `Box<dyn System>` instead of `Box<dyn Any>`.
</details>

`dyn Any` 的含义只是：我们想存某种实现了 `Any` 的对象，但具体类型要到运行时才知道。

<details markdown="1">
<summary>原文</summary>

The `dyn Any` just means we want some kind of object that implements the `Any` trait, but we won't know the exact type until runtime.
</details>

`std::Any` 要求 `'static`，这使得迁移到并行系统变得不可能。

<details markdown="1">
<summary>原文</summary>

`std::Any` type has a `'static` requirement, which makes moving to parallel systems impossible.
</details>

另外，装进 `Box` 里做动态分发的 trait 本身不能是泛型的。所以我们不能写 `Box<dyn System<Input>>`。

<details markdown="1">
<summary>原文</summary>

However, any traits we dynamically dispatch to in a `Box` cannot themselves be generic. So we could not do `Box<dyn System<Input>>`.
</details>

每当在 Rust 里使用像 `System<(T1,)>` 这样的泛型参数时，编译器都会针对你传入的具体类型参数生成专门代码，例如
`System<(i32,)>`。

<details markdown="1">
<summary>原文</summary>

Every time we use a generic argument in rust like `System<(T1,)>` the compiler generates code specialized to the concrete type parameters you called e.g: `System<(i32,)>`.
</details>

每次实例化都会生成一套对应函数，就像你手写了不同版本一样。而这在编译时必须是有限的，代码生成器才能结束工作。因此我们不能有 `Box<dyn System<Input>>`，因为 `System<Input>` 里包含泛型参数 `Input`。

<details markdown="1">
<summary>原文</summary>

Each instantiation will have their functions generated as though you physically typed out the different versions. This must be finite at compile time so the generator can finish. Therefore we cannot have a `Box<dyn System<Input>>` because `System<Input>` contains a generic parameter `Input`.
</details>

---

## 类型擦除

<details markdown="1">
<summary>原标题</summary>

## Erasing types
</details>

相反，我们可以用一种叫作[type erasure](https://vgatherps.github.io/2020-04-14-erasure/)（类型擦除）的技术，以不透明方式对待存储类型，从而减少这种单态化。

<details markdown="1">
<summary>原文</summary>

We could instead reduce this monomorphization by treating the stored type opaquely using a technique called [type erasure](https://vgatherps.github.io/2020-04-14-erasure/):
</details>

```rust
// https://gist.github.com/335g/42f61a8ca0fbb845e134db675d13cc7b
trait AnimalExt {
    fn eat(&self, food: String);
}

struct Dog;

impl AnimalExt for Dog {
    fn eat(&self, food: String) {
        println!("dog: {:?}", food);
    }
}

// Instead of a generic argument we are using a smart pointer to a function.
// This way we have erased the pointer to a concrete type and are instead
// storing type-erased smart pointers (on the heap instead of the stack)
struct AnyAnimal {
    eat: Box<dyn Fn(String)>,
}

impl AnyAnimal {
    fn new<A>(animal: A) -> Self
    where
        A: AnimalExt + 'static,
    {
        AnyAnimal {
            eat: Box::new(move |s| animal.eat(s)),
        }
    }
}

// Here we implement the trait which invokes the type-erased pointer
impl AnimalExt for AnyAnimal {
    fn eat(&self, food: String) {
        (self.eat)(food); // ok
        // self.eat(food) <- fatal runtime error: stack overflow
    }
}

fn main() {
    let a = AnyAnimal::new(Dog);
    a.eat("aaa".to_string());
}
```

使用示例中类似 `AnyAnimal` 的包装结构体，还可以阻止在 `AnyAnimal` 的传入泛型参数上对 `AnimalExt` 的实现进行内联。它还会为其提供一个类型安全的包装层。

<details markdown="1">
<summary>原文</summary>

The idea of using a wrapper struct like `AnyAnimal` from the example above will also prevent inlining of the implementations of `AnimalExt` on any generic arguments to `AnyAnimal`. Its also going to provide a type-safe wrapper around it.
</details>

于是现在你可以持有 `Box<AnyAnimal>`，而不是 `Box<dyn Any>`。

<details markdown="1">
<summary>原文</summary>

So now you could hold `Box<AnyAnimal>` instead of using `Box<dyn Any>`.
</details>

我们可以把同样思路用在自己的系统上：

<details markdown="1">
<summary>原文</summary>

We could try the same thing with our own systems:
</details>

```rust
trait System<Input> {
    fn run(&mut self, resources: &mut HashMap<TypeId, Box<dyn Any>>);
}

trait ErasedSystem {
    fn run(&mut self, resources: &mut HashMap<TypeId, Box<dyn Any>>);
}

impl<S: System<I>, I> ErasedSystem for S {
    fn run(&mut self, resources: &mut HashMap<TypeId, Box<dyn Any>>) {
        <Self as System<I>>::run(self);
    }
}
```

不过我们会得到一个类型错误：

<details markdown="1">
<summary>原文</summary>

However we would get a type error:
</details>

```
error[E0207]: the type parameter I is not constrained by the impl trait,
self type, or predicates
```

在 Rust 中定义泛型类型或函数时，必须给泛型参数指定约束。这些约束会限制我们可以传入的泛型参数类型。

<details markdown="1">
<summary>原文</summary>

In Rust, when defining a generic type or function, you need to specify the constraints on the generic type parameters. These constraints limit the types we can pass as a generic argument.
</details>

在上面的代码里，我们试图给任意类型 `S` 实现 `ErasedSystem`，条件是 `S` 实现了 `System<I>`，其中 `I` 是泛型参数。

<details markdown="1">
<summary>原文</summary>

In our code above, we are trying to implement the `ErasedSystem` trait for any type `S` that implements the `System<I>` trait where `I` is a generic type parameter.
</details>

但由于 `I` 没有任何约束，编译器无法保证该实现对所有可能的 `I` 都有效。

<details markdown="1">
<summary>原文</summary>

However because we don't have any constraints on `I` the compiler cannot guarantee the implementation is valid for all possible types of `I`.
</details>

由于系统可以实现多个 trait（例如 `FnMut(T)` 或 `FnMut(T, U)`），我们必须明确 `I` 到底可以是哪一种。

<details markdown="1">
<summary>原文</summary>

Because our systems can implement multiple traits such as `FnMut(T)` or `FnMut(T, U)` we would need to be specific about which one `I` could be.
</details>

回看我们最初的系统定义：

<details markdown="1">
<summary>原文</summary>

Looking back at our original system definition:
</details>

```rust
trait System<Input> {}

// An implementation of systems that take no parameters:
impl<F: FnMut()> System<()> for F {}

// Here we implement our system with a single parameter:
impl<F: FnMut(T1), T1: 'static> System<(T1,)> for F {}
```

记住，`I` 是系统的 `Input`，它是一个由一个或多个类型组成的元组，代表系统声明并希望被传入的参数。

<details markdown="1">
<summary>原文</summary>

Remember that `I` is the `Input` of our system, which is a tuple of one or more types, representing the arguments our system will declare and want to be fed in.
</details>

尽管 `F` 可以实现多个 `FnMut` trait，但如果把 `F` 包在一个结构体里，这个结构体就能“选中”某个特定实现。

<details markdown="1">
<summary>原文</summary>

While `F` can implement multiple `FnMut` traits, if we wrap `F` in a struct then that struct can "select" a specific implementation.
</details>

这样一来，编译器就不是按 `F` 生成多个实现变体，而是改为动态调用参数 `F`，这会把原本 `System` 上的类型要求擦除并转移到新结构体中。

<details markdown="1">
<summary>原文</summary>

In this way, the compiler is not generating variations of the implementation depending on `F` but is instead dynamically invoking the argument `F` which would erase the type requirement from our `System` and move it into the new struct.
</details>

该结构体最终采用的实现，就是与其泛型参数匹配的那一个，且只能有一个实现满足匹配。

<details markdown="1">
<summary>原文</summary>

The implementation the struct chooses is whichever matches the struct's generic parameters, which only one implementation can do.
</details>

```rust
struct FunctionSystem<Input, F> {
    f: F,
    // we need a marker because otherwise we're not using `Input`.
    // fn() -> Input is chosen because just using Input would not be `Send` + `Sync`,
    // but the fnptr is always `Send` + `Sync`.
    marker: PhantomData<fn() -> Input>,
}
```

现在我们把 `System` 从函数自身迁移到 `FunctionSystem` 上：

<details markdown="1">
<summary>原文</summary>

Now let's move `System` from being on the function itself to `FunctionSystem`:
</details>

```rust
macro_rules! impl_system {
  (
    $($params:ident),*
  ) => {
    #[allow(non_snake_case, unused)]
    impl<F: FnMut($($params),*) $(,$params: 'static)*> System
    for FunctionSystem<($($params),*), F> {
      fn run(&mut self, resources: &mut HashMap<TypeId, Box<dyn Any>>) {
        $(
          let $params = *resources.remove(&TypeId::of::<$params>()).unwrap().downcast::<$params>().unwrap();
        )*

        (self.f)( $($params),* );
      }
    }
  }
}
```

现在 `System` 不再带关联类型或泛型参数（类型已被擦除），装箱就很容易了：

<details markdown="1">
<summary>原文</summary>

Now that System takes no associated types or generic parameters, we can box it easily:
</details>

```rust
trait System {}
type StoredSystem = Box<dyn System>;
```

---

## 把函数转换为系统

<details markdown="1">
<summary>原标题</summary>

## Converting functions to systems
</details>

我们还希望能方便地把 `FnMut(...)` 转成系统，而不是每次都手动初始化 `FunctionSystem` 结构体：

<details markdown="1">
<summary>原文</summary>

We'll also want to be able to convert `FnMut(...)` to a system easily instead of manually initializing our `FunctionSystem` struct each time:
</details>

```rust
trait IntoSystem<Input> {
    type System: System;

    // Wrap ourself into the type of System above
    fn into_system(self) -> Self::System;
}

// Example output:
// impl<F: FnMut(T1), T1> IntoSystem<(T1,)> for F {
//     type System = FunctionSystem<(T1,), Self>;
//
//     fn into_system(self) -> Self::System {
//         FunctionSystem {
//             f: self,
//             marker: Default::default(),
//         }
//     }
// }

macro_rules! impl_into_system {
  (
    $($params:ident),*
  ) => {
    impl<F: FnMut($($params),*) $(, $params:'static)* > IntoSystem<($($params,)*)> for F {
      type System = FunctionSystem<( $($params,)* ), Self>;

      fn into_system(self) -> Self::System {
        FunctionSystem {
          f: self,
          marker: Default::default(),
        }
      }
    }
  }
}

impl_into_system!();
impl_into_system!(T1);
impl_into_system!(T1, T2);
impl_into_system!(T1, T2, T3);
impl_into_system!(T1, T2, T3, T4);
```

现在我们可以开始定义 `Scheduler` 的公开 API，用于把系统加入游戏循环：

<details markdown="1">
<summary>原文</summary>

Now we can start to define the public API of our `Scheduler` to add these systems to our game loop:
</details>

```rust
struct Scheduler {
    systems: Vec<StoredSystem>,
    resources: HashMap<TypeId, Box<dyn Any>>,
}

trait IntoSystem<Input> {
    type System: System;

    fn into_system(self) -> Self::System;
}

impl Scheduler {
    pub fn run(&mut self) {
        for system in self.systems.iter_mut() {
            system.run(&mut self.resources);
        }
    }

    pub fn add_system<I, S: System + 'static>(&mut self, system: impl IntoSystem<I, System=S>) {
        self.systems.push(Box::new(system.into_system()));
    }

    pub fn add_resource<R: 'static>(&mut self, res: R) {
        self.resources.insert(TypeId::of::<R>(), Box::new(res));
    }
}
```

这会让我们定义出第一个真正可运行的系统：

<details markdown="1">
<summary>原文</summary>

This would let us define our first real working system:
</details>

```rust
fn main() {
    let mut scheduler = Scheduler {
        systems: vec![],
        resources: HashMap::new(),
    };

    scheduler.add_system(foo);
    scheduler.add_resource(12i32);

    scheduler.run();
}

fn foo(int: i32) {
    println!("int! {int}");
}
```

但我们仍然无法使用借用类型。按目前这样，资源在每个游戏周期（tick）都会被消耗掉。 那如果我们放开系统参数数量上限的话……

<details markdown="1">
<summary>原文</summary>

However we still cannot use borrowed types. As it stands resources are consumed every game tick. If we lifted our maximum limit on system parameters...
</details>

---

## 处理可变借用

<details markdown="1">
<summary>原标题</summary>

## Handling mutable borrowing
</details>

假设我们选择让 `.run` 返回资源引用，那么当前实现会失败：

<details markdown="1">
<summary>原文</summary>

Assuming we chose to have our `.run` return resource references, our current implementation would fail:
</details>

```
error[E0277]: the trait bound fn(i32) {foo}: IntoSystem<_> is not satisfied
```

当系统只使用只读引用时不会报这个错，只有可变借用才会触发。

<details markdown="1">
<summary>原文</summary>

We don't get this error when we write a system that uses only read-only references, only mutable borrowing will trigger the error.
</details>

如果手动实现所有可能组合，对每个要支持的参数个数 n，都需要 3<sup>n</sup> 种实现。即使用宏，这也会变得不合理（会增大编译时间和程序体积）。

<details markdown="1">
<summary>原文</summary>

Manually implementing the possible combinations would be 3^n for every n parameters we would want to support. Even with macros this would become unreasonable (by increasing the compile time and size of the program).
</details>

我们可以换个方向，抽象所有可能的系统参数：

<details markdown="1">
<summary>原文</summary>

Instead we could try to abstract over all possible system parameters:
</details>

```rust
trait SystemParam {
    fn retrieve(resources: &mut HashMap<TypeId, Box<dyn Any>>) -> Self;
}

struct Res<'a, T: 'static> {
    value: &'a T,
}

impl<'a, T: 'static> SystemParam for Res<'a, T> {
    fn retrieve(resources: &mut HashMap<TypeId, Box<dyn Any>>) -> Self {
        let value = resources.get(&TypeId::of::<T>()).unwrap().downcast_ref::<T>().unwrap();
        Res { value }
    }
}

struct ResMut<'a, T: 'static> {
    value: &'a mut T,
}

impl<'a, T: 'static> SystemParam for ResMut<'a, T> {
    fn retrieve(resources: &mut HashMap<TypeId, Box<dyn Any>>) -> Self {
        let value = resources.get_mut(&TypeId::of::<T>()).unwrap().downcast_mut::<T>().unwrap();
        ResMut { value }
    }
}

struct ResOwned<T: 'static> {
    value: T
}

impl<T: 'static> SystemParam for ResOwned<T> {
    fn retrieve(resources: &mut HashMap<TypeId, Box<dyn Any>>) -> Self {
        let value = *resources.remove(&TypeId::of::<T>()).unwrap().downcast::<T>().unwrap();
        ResOwned { value }
    }
}
```

这里我们用不同类型去实现非泛型 trait `SystemParam`。

<details markdown="1">
<summary>原文</summary>

Here we are using different types to implement our non-generic trait `SystemParam`.
</details>

但编译后如果尝试使用借用引用，会得到：

<details markdown="1">
<summary>原文</summary>

However compiling and trying to use a borrowed reference will give us:
</details>

```
error: lifetime may not live long enough
```

我们可以在 `SystemParam` 实现中引入生命周期，但签名将变成：

<details markdown="1">
<summary>原文</summary>

We could introduce lifetimes to our implementations of `SystemParam` but that would change the signature to:
</details>

```rust
trait SystemParam<'a> {
    fn retrieve(resources: &'a mut HashMap<TypeId, Box<dyn Any>>) -> Self;
}
```

这又会要求在 `System`、`IntoSystem`、`StoredSystem` 和 `.add_system` 上都引入生命周期。

<details markdown="1">
<summary>原文</summary>

Which would then require lifetimes introduced for `System`, `IntoSystem`, `StoredSystem` and `.add_system`.
</details>

同时，因为对于参数数量 > 1 的情况，我们会多次可变借用资源，因此还会触发借用错误：

<details markdown="1">
<summary>原文</summary>

But because we are mutably borrowing resources multiple times for variants with > 1 parameters, we would get a borrowing error:
</details>

```
error[E0499]: cannot borrow *resources as mutable more than once at a time
```

而如果实现类似下面这样的代码：

<details markdown="1">
<summary>原文</summary>

And implementing something like:
</details>

```rust
impl<'a, T: 'static> SystemParam<'a> for Res<'a, T> {
    fn retrieve(resources: &'a HashMap<TypeId, Box<dyn Any>>) -> Self {
        let value = resources.get(&TypeId::of::<T>()).unwrap().downcast_ref::<T>().unwrap();
        Res { value }
    }
}

pub fn add_system<I, S: for<'a> System<'a> + 'static>(&mut self, system: impl for<'a> IntoSystem<'a, I, System=S>) {
    self.systems.push(Box::new(system.into_system()));
}
```

会得到如下错误：

<details markdown="1">
<summary>原文</summary>

Would lead to an error of:
</details>

```
error: implementation of System is not general enough
```

因为 `.add_system` 把系统参数定义成 `impl for<'a> IntoSystem<'a, I, System = S>`，其中 `for<'a>` 意味着
`IntoSystem<'a, I, System = S>` 必须比所有生命周期都活得更久，包括 `'static`。

<details markdown="1">
<summary>原文</summary>

Because our `.add_system` defines the system parameter as `impl for<'a> IntoSystem<'a, I, System = S>` the `for<'a>` implies that `IntoSystem<'a, I, System = S>` must outlive all lifetimes, including `'static'`.
</details>

截至 2022 年 10 月，Rust
社区仍在推进这类生命周期问题的解决：[https://blog.rust-lang.org/2022/10/28/gats-stabilization.html](https://blog.rust-lang.org/2022/10/28/gats-stabilization.html)。

<details markdown="1">
<summary>原文</summary>

As of October, 2022 the Rust community is working on these kind of lifetime problems: [https://blog.rust-lang.org/2022/10/28/gats-stabilization.html](https://blog.rust-lang.org/2022/10/28/gats-stabilization.html)
</details>

我们来看看 Bevy 如何绕开这类问题：

<details markdown="1">
<summary>原文</summary>

Lets have a look at what Bevy does to get around these kinds of problems:
</details>

```rust
pub unsafe trait SystemParam: Sized {
    // Used to store data which persists across invocations of a system.
    type State: Send + Sync + 'static;

    // The item type returned when constructing this system param.
    // The value of this associated type should be `Self`, instantiated with new lifetimes.
    //
    // You could think of `SystemParam::Item<'w, 's>` as being an *operation* that changes the lifetimes bound to `Self`.
    type Item<'world, 'state>: SystemParam<State=Self::State>;

    // ...
}
```

所以 `SystemParam` 是使用了一个叫 `Item` 的 GAT，它本身也是 `SystemParam`，但拥有不同生命周期。这意味着我们把函数的生命周期，替换成传入资源的新生命周期。

<details markdown="1">
<summary>原文</summary>

So `SystemParam` is using a GAT called `Item` which is itself a `SystemParam`, but it has a different lifetime. That means we are taking the functions lifetime and giving it the new lifetime of the passed resource.
</details>

泛型关联类型（GAT）允许你在关联类型上使用泛型（类型、生命周期或常量）。

<details markdown="1">
<summary>原文</summary>

Generic associated types (GAT) allow you to have generics (type, lifetime, or const) on associated types.
</details>

因此我们可以在简化版里使用同样技巧：

<details markdown="1">
<summary>原文</summary>

So we can use the same kind of trick for our simplified version:
</details>

```rust
trait SystemParam {
    type Item<'new>;

    fn retrieve<'r>(resources: &'r HashMap<TypeId, Box<dyn Any>>) -> Self::Item<'r>;
}

impl<'res, T: 'static> SystemParam for Res<'res, T> {
    type Item<'new> = Res<'new, T>;

    fn retrieve<'r>(resources: &'r HashMap<TypeId, Box<dyn Any>>) -> Self::Item<'r> {
        Res { value: resources.get(&TypeId::of::<T>()).unwrap().downcast_ref().unwrap() }
    }
}

macro_rules! impl_system {
  (
      $($params:ident),*
  ) => {
    #[allow(non_snake_case)]
    #[allow(unused)]
    impl<F, $($params: SystemParam),*> System for FunctionSystem<($($params,)*), F>
      where
        for<'a, 'b> &'a mut F:
          FnMut( $($params),* ) +
          FnMut( $(<$params as SystemParam>::Item<'b>),* )
    {
      fn run(&mut self, resources: &mut HashMap<TypeId, Box<dyn Any>>) {
        // type notation
        // This call_inner is necessary to tell rust which function impl to call
        fn call_inner<$($params),*>(
          mut f: impl FnMut($($params),*),
          $($params: $params),*
        ) {
          f($($params),*)
        }

        $(
          let $params = $params::retrieve(resources);
        )*

        call_inner(&mut self.f, $($params),*)
      }
    }
  }
}

macro_rules! impl_into_system {
  (
    $($params:ident),*
  ) => {
    impl<F, $($params: SystemParam),*> IntoSystem<($($params,)*)> for F
      where
        for<'a, 'b> &'a mut F:
          FnMut( $($params),* ) +
          FnMut( $(<$params as SystemParam>::Item<'b>),* )
    {
      type System = FunctionSystem<($($params,),*), Self>;

      fn into_system(self) -> Self::System {
        FunctionSystem {
          f: self,
          marker: Default::default(),
        }
      }
    }
  }
}
```

现在有了 `SystemParam`，要扩展到支持任意多参数就很容易了。

<details markdown="1">
<summary>原文</summary>

Now that we have `SystemParam` in place, it'll be easy to expand this to work with as many parameters as we want.
</details>

---

## 解锁无限参数

<details markdown="1">
<summary>原标题</summary>

## Unlocking unlimited parameters
</details>

我们只需要一个关键想法：如果由 `SystemParam` 组成的元组本身也是 `SystemParam` 会怎样？让我们来实现它：

<details markdown="1">
<summary>原文</summary>

We just need one crucial idea: what if a tuple of `SystemParam` is, itself, a `SystemParam`? Let's implement:
</details>

```rust
impl<T1: SystemParam, T2: SystemParam> SystemParam for (T1, T2) {
    type Item<'new> = (T1::Item<'new>, T2::Item<'new>);

    fn retrieve<'r>(resources: &'r HashMap<TypeId, Box<dyn Any>>) -> Self::Item<'r> {
        (
            T1::retrieve(resources),
            T2::retrieve(resources),
        )
    }
}

fn foo(int: (Res<i32>, Res<u32>)) {
    println!("int! {} uint! {}", int.0.value, int.1.value);
}
```

虽然看起来我们只为“两个 `SystemParam` 的元组”定义了实现，但递归实际上允许无限扩展：

<details markdown="1">
<summary>原文</summary>

Even though it looks like we have defined an implementation for a tuple of two `SystemParam`, the recursion actually allows us to have unlimited:
</details>

```rust
fn foo(int: (Res<One>, (Res<Two>, (Res<Three>, (Res<Four>))))) {
    // ...
}
```

不过，如果只允许嵌套元组且最多两个位置参数，使用起来会比较麻烦，所以我们应该更新宏，并明确支持多少个参数（Bevy 选择了 15 个）：

<details markdown="1">
<summary>原文</summary>

It would however be a bit cumbersome to only allow nested tuples and a max of two positional arguments, so we should update our macro and choose exactly how many arguments to support (Bevy chose 15):
</details>

```rust
macro_rules! impl_system_param {
  (
    $($params:ident),*
  ) => {
    #[allow(unused)]
    impl<$($params: SystemParam),*> SystemParam for ($($params,)*) {
      type Item<'new> = ($($params::Item<'new>,)*);

      fn retrieve<'r>(resources: &'r HashMap<TypeId, Box<dyn Any>>) -> Self::Item<'r> {
        (
          $($params::retrieve(resources),)*
        )
      }
    }
  }
}

impl_system_param!();
impl_system_param!(T1);
impl_system_param!(T1, T2);
impl_system_param!(T1, T2, T3);
// and so on
```

## 更多内容

* [https://promethia-27.github.io/dependency_injection_like_bevy_from_scratch/introductions.html](https://promethia-27.github.io/dependency_injection_like_bevy_from_scratch/introductions.html)

## 附录：完整代码

<details markdown="1">
<summary>代码</summary>
```rust
use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

fn main() {
    let mut scheduler = Scheduler::new();
    scheduler.add_resource(13i32);
    scheduler.add_resource("Yeah");
    scheduler.add_resource(24usize);

    scheduler.add_system(|x: Res<i32>, y: Res<&str>, z: Res<usize>| {
        println!("{} {} {}", *x, *y, *z);
    });

    scheduler.add_system(|mut x: ResMut<i32>| {
        *x += 1;
    });

    scheduler.add_system(|x: Res<i32>| {
        println!("{} again", *x);
    });

    scheduler.run();
}

/// 系统：作为调度器的调度单位，需要`dyn`安全性（擦除泛型和关联类型），是依赖注入的入口
trait System {
    /// 系统调用：驱动依赖注入和任务执行
    fn run(&mut self, resources: &HashMap<TypeId, RefCell<Box<dyn Any>>>);
}

/// 函数系统：作为函数对象的包装器，解决原生函数泛型覆盖实现 [`System`] 时泛型未约束的问题
struct FunctionSystem<I, F> {
    f: F,
    /// 消耗泛型 `I`，使用函数指针`fn() -> I`是为了避免因`I`引入`!Sync`或`!Send`
    marker: PhantomData<fn() -> I>,
}

/// 转换系统：便于统一系统收集
trait IntoSystem<I> {
    type System: System;
    fn into_system(self) -> Self::System;
}

/// 调度器：维护系统集和依赖资源，并调度系统
struct Scheduler {
    /// 系统集合，`dyn System` 统一管理调度
    systems: Vec<Box<dyn System>>,
    /// 依赖资源集合，按类型`TypeId`维护，使用内部可变性延后别名检查，便于拆分资源引用
    resources: HashMap<TypeId, RefCell<Box<dyn Any>>>,
}

impl Scheduler {
    fn new() -> Self {
        Self {
            systems: Vec::new(),
            resources: HashMap::new(),
        }
    }

    /// （按需）调度系统
    fn run(&mut self) {
        for system in self.systems.iter_mut() {
            system.run(&self.resources);
        }
    }

    /// 添加系统
    fn add_system<I, S>(&mut self, system: S)
    where
        S: IntoSystem<I>,
        S::System: 'static,
    {
        self.systems.push(Box::new(system.into_system()));
    }

    /// 添加系统依赖资源
    fn add_resource<R>(&mut self, resource: R)
    where
        R: 'static,
    {
        self.resources
            .insert(TypeId::of::<R>(), RefCell::new(Box::new(resource)));
    }
}

/// 系统参数：为参数依赖建立统一的使用方式和约束规范
trait SystemParam {
    /// 带生命周期的GAT用法，解决依赖参数的生命周期问题，因为`Self`生命周期无法约束
    type Item<'new>;
    /// 提取参数依赖，并赋予指定的生命周期范围（生命周期受限的`Self::Item`，而非任意生命周期的`Self`）
    fn retrieve<'r>(resources: &'r HashMap<TypeId, RefCell<Box<dyn Any>>>) -> Self::Item<'r>;
}

/// 不可变依赖引用
struct Res<'a, T: 'static> {
    /// 不可变引用，由于`borrow`返回的`Ref`是一个引用“值”，需要持有才能继续使用其依赖引用
    value: Ref<'a, Box<dyn Any>>,
    /// 消耗泛型`T`
    _marker: PhantomData<&'a T>,
}

impl<'a, T: 'static> SystemParam for Res<'a, T> {
    /// 指定生命周期的资源引用
    type Item<'new> = Res<'new, T>;

    fn retrieve<'r>(resources: &'r HashMap<TypeId, RefCell<Box<dyn Any>>>) -> Self::Item<'r> {
        let value = resources.get(&TypeId::of::<T>()).unwrap().borrow();
        Res {
            value,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for Res<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value.downcast_ref().unwrap()
    }
}

/// 可变依赖引用
struct ResMut<'a, T: 'static> {
    /// 可变引用，由于`borrow_mut`返回的`RefMut`是一个引用“值”，需要持有才能继续使用其依赖引用
    value: RefMut<'a, Box<dyn Any>>,
    /// 消耗泛型`T`
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T: 'static> SystemParam for ResMut<'a, T> {
    /// 指定生命周期的资源引用
    type Item<'new> = ResMut<'new, T>;

    fn retrieve<'r>(resources: &'r HashMap<TypeId, RefCell<Box<dyn Any>>>) -> Self::Item<'r> {
        let value = resources.get(&TypeId::of::<T>()).unwrap().borrow_mut();
        ResMut {
            value,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for ResMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value.downcast_ref().unwrap()
    }
}
impl<T> DerefMut for ResMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.downcast_mut().unwrap()
    }
}

/// 为多参数函数系统实现[`System`]系统特征
macro_rules! impl_system {
    ($($param:ident),*) => {
        #[allow(unused_variables)]
        #[allow(non_snake_case)]
        impl<F, $($param: SystemParam),*> System for FunctionSystem<($($param,)*), F>
        where
            for <'a, 'b> &'a mut F: FnMut($($param),*) + FnMut($($param::Item<'b>),*),
        {
            fn run(&mut self, resources: &HashMap<TypeId, RefCell<Box<dyn Any>>>) {
                #[inline(always)]
                fn call_inner<$($param),*>(
                    mut f: impl FnMut($($param),*),
                    $($param: $param,)*
                ) {
                    f($($param),*);
                }

                $(let $param = $param::retrieve(resources);)*
                call_inner(&mut self.f, $($param),*)
            }
        }
    };
}
/// 为多参数函数对象实现[`IntoSystem`]系统转换特征
macro_rules! impl_into_system {
    ($($param:ident),*) => {
        impl<F, $($param: SystemParam),*> IntoSystem<($($param,)*)> for F
        where
            for<'a, 'b> &'a mut F: FnMut($($param),*) + FnMut($($param::Item<'b>),*),
        {
            type System = FunctionSystem<($($param,)*), Self>;

            fn into_system(self) -> Self::System {
                FunctionSystem {
                    f: self,
                    marker: Default::default(),
                }
            }
        }
    };
}
/// 为多元素组合实现系统参数特征
macro_rules! impl_tuple_param {
    ($($param:ident),*) => {
        #[allow(unused_variables)]
        impl<$($param: SystemParam),*> SystemParam for ($($param,)*) {
            type Item<'new> = ($($param::Item<'new>,)*);

            fn retrieve<'r>(resources: &'r HashMap<TypeId, RefCell<Box<dyn Any>>>) -> Self::Item<'r> {
                ($($param::retrieve(resources),)*)
            }
        }
    };
}
/// 自动为多泛型参数宏操作数量迭代的泛型实现，简化模板操作
macro_rules! expand_times {
    ($impl_macro:ident @ ($one:ident $(,$generic:ident)* $(,)?)) => {
        $impl_macro!($one $(,$generic)*);
        expand_times!($impl_macro @ ($($generic),*));
    };
    ($impl_macro:ident @ ()) => {
        $impl_macro!();
    };
}
// 实现多参数系统特征
expand_times!(impl_system @ (
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16
));
// 实现多参数系统转换特征
expand_times!(impl_into_system @ (
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16
));
// 实现多元素系统参数元组
expand_times!(impl_tuple_param @ (
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16
));

```
</details>

