---
layout: post
title: "Rust中“真正卫生”的Let语句"
date: 2025-10-24
tags: [Rust, macro, Hygienic]
---

> "Truly Hygienic" Let Statements in Rust

Rust 中“真正卫生”的 Let 语句

> 2024-09-22

> Remon is a responsible library developer. She cares about stability, flexibility and correctness, using whichever tools are presently accessible to achieve those goals. Her authored libraries feature automated testing and extensive documentation; she allots design decisions rationale; she knows her language features and traditions and how to apply them to best effect.

Remon 是一位负责任的库开发者。她关心稳定性、灵活性和正确性，并使用当前任何可用的工具来实现这些目标。她编写的库具有自动化测试和广泛的文档；她为设计决策提供合理依据；她深谙语言特性和传统，以及如何运用它们来达到最好的效果。

> And, somewhere to be discovered bound in the tangle of `.rs` files, there is Remon herself, tranquil and yet focused, meticulously crafting, polishing, studying and crafting again, a component she forsees to ease the life of her users, provides ergonomics inaccessible by traditional methods, brings to life the great gift of syntax without glue added to the cogs of the build process – a declarative macro.

在错综复杂的`.rs`文件中某处，有 Remon 本人，平静而专注，精心制作、打磨、研究并再次制作一个组件，她预见到这个组件能减轻用户的负担，提供传统方法无法获得的人体工程学，为构建过程的齿轮带来无胶水语法的伟大礼物——一个声明式宏。

> Refined and learned code-witch she is, Remon is keenly aware of Rust Cultures and Traditions, and so in keeping, would do nothing but summon a monstrous (documented, without doubt, but monstrous nonetheless) tornado of dollar signs and brackets, one whose gales would surely lift up and send flying a meek blog post as this one. Have sympathy! I cannot handle that – I must admit I have not even implemented `Send`, so the results could verge on disastrous. But a trained magician knows better than to create a beast they cannot tame, and so for this chronicle it is simplified to a wisp of its wild self – one where you must excuse the apparent folly of its existence – as follows:

作为一名精炼且博学的代码女巫，Remon 深刻了解 Rust 文化和传统，因此遵循传统，除了召唤一场可怕的（毫无疑问有文档注释，但仍然是可怕的）由美元符号和括号组成的龙卷风之外什么也没做，它的狂风肯定会把像这样的谦逊博客文章吹上天。请同情我！我处理不了那种东西——我必须承认我甚至没有实现过`Send`，所以结果可能接近灾难性的。但是训练有素的魔术师知道不应该创造自己无法控制的怪兽，因此在这篇记述中，它被简化为其狂野本身的一缕窥探——一个你必须原谅其明显愚蠢存在的例子——如下所示：

```rust
macro_rules! oh_my {
	() => {
		let Ok(x) = read_input() else { return Err(Error) };
		$crate::process(x);
	};
}
```

> Remon is a responsible library developer, and understands that all humans will make mistakes – and so she has solicited the services of a good friend, Wolfie, to comment on this slice of code.

Remon 是一位负责任的库开发者，她明白所有人都会犯错误——因此她请求一位好朋友 Wolfie 对这段代码发表评论。

> Well, Wolfie says, this macro is very impressive feat, and shall surely ease the lives of our users, provide ergonomics inaccessible by traditional methods, and bring to life the great gift of syntax without glue added to the cogs of the build process. But I do have one concern – the `let` in this macro is not hygienic.

嗯，Wolfie 说，这个宏是一项非常令人印象深刻的功能，肯定能为我们用户带来便利，提供传统方法无法获得的人体工程学，并且为构建流程的齿轮带来了无需胶水的语法的伟大礼物。但我确实有一个担忧——这个宏中的`let`不卫生。

> Now, Remon has read her literature, and knows that Rust macros are hygienic with regards to locals – they are guaranteed not to interfere with variables of the caller’s scope unless the variable’s name is explicitly passed in.

现在，Remon 已经阅读过相关文献，她知道 Rust 宏在局部变量方面是卫生的——除非显式传入变量名，否则他们保证不会干扰到调用者作用域中的变量。

> Is that so?, asks Remon. You and I both know that Rust macros use mixed-site hygiene. But I trust your experience as a developer and respect you as a person, so I will approach this incongruence with curiosity rather than dismissal. Thus I must ask you: Whatever do you mean?

是这样吗？Remon 问道。你和我都知道 Rust 宏使用混合站点卫生性。但我信任你作为开发者的经验，尊重你这个人，所以我会以好奇心而不是排斥的态度来对待这种不一致。因此我必须问你：你到底是什么意思？

> Wolfie thinks for a second, and concludes this point best communicated through the medium of code. So he quickly types out a demo of a certain way of use causing bugs:

Wolfie 思考了一会儿，认为通过代码演示能最好地传达这一点。所以他快速编写了一个导致 bug 的特定使用方式示例：

```rust
const x: &str = "26ad109e6f4d14e5cc2c2ccb1f5fb497abcaa223";
oh_my!();
```

> And upon entering input that is not the latest commit hash of the greatest Rust library of all time, Remon is dismayed and ashamed to discover that the code, incorrectly, results in an error. But it’s at least not hard to discover _why_: in the line containing `let Ok(x) =`, `x` is a identifier pattern, which means it can either refer to a constant if the constant is in scope, or create a new variable otherwise. Of course, the macro expects the latter to happen, but since constants are items, and thus unlike variables are unhygienic, if there is a constant x at the call site, it will be used instead. So our pattern becomes equivalent to `Ok("26ad109…")`, which will of course reject any value that is not the latest commit hash of the greatest Rust library of all time, resulting in silent bugs.

当输入不是有史以来最伟大的 Rust 库的最新提交哈希时，Remon 沮丧而羞愧地发现，代码错误地，产生了错误。但至少不难发现*原因*：在包含`let Ok(x) =`的行中，`x`是一个标识符模式，这意味着如果常量在作用域内它可以引用该常量，否则创建一个新变量。当然，宏期望是后者发生，但由于常量是项，因此与变量不同，它们是非卫生的，如果调用点有一个常量 x，它将被使用。所以我们的模式等价于`Ok("26ad109…")`，这当然会拒绝任何不是有史以来最伟大的 Rust 库的最新提交哈希的值，从而产生无形的 bug。

> Okay, thinks Remon. I know of a way to fix this: the pattern `IDENT @ PATTERN` will unambiguously have `IDENT` bound as a variable, never to be treated as a constant. Since there are no other restrictions to be placed on the data, our `PATTERN` can simply be a wildcard – `_`. So that’s what she does:

好吧，Remon 想。我知道一种修复方法：模式`IDENT @ PATTERN`将明确地将`IDENT`绑定为变量，永远不会被视为常量。由于对数据没有其他限制，我们的`PATTERN`可以简单地是一个通配符——`_`。所以她就这样做了：

```rust
macro_rules! oh_my {
	() => {
		let Ok(x @ _) = read_input() else { return Err(Error) };
		$crate::process(x);
	};
}
```

> But Wolfie is still not pleased, and Remon is still surprised, because now there is a compilation error.

但是 Wolfie 仍然不满意，Remon 也依然感到惊讶，因为现在出现了一个编译错误。

```
error[E0530]: let bindings cannot shadow constants
 --> src/main.rs:3:10
  |
3 |         let Ok(x @ _) = read_input() else { return Err(Error) };
  |                ^ cannot be named the same as a constant
...
8 |     const x: &str = "TODO";
  |     ---------------------- the constant `x` is defined here
9 |     oh_my!();
  |     -------- in this macro invocation
  |
```

> This is of course not as bad as buggy behaviour, but Wolfie knows that Remon is a responsible library developer who cares about flexibility and correctness, and it is unpredicable that the macro would suddenly start failing just because of some constants that happen to be there at the call site.

当然，这不像 bug 行为那么糟糕，但 Wolfie 知道 Remon 是一位关心灵活性和正确性的负责任的库开发者，况且宏仅因为在调用点碰巧存在一些常量就突然开始失败是不可预测的。

> Remon has never seen this error before, but remains undeterred. After all, there is one more trick up her sleeve: although `let` bindings cannot shadow constants, those two do not account for every member of the value namespace. Functions are a member just as well. And functions, unlike `consts`, have the property that they _can_ be shadowed – and by virtue of being an item, they may shadow the latter as well (if introduced in a smaller scope).

Remon 以前从未见过这个错误，但并未气馁。毕竟，她还有一个技巧：虽然`let`绑定不能遮蔽常量，但这两种情况并不能涵盖值命名空间的所有成员。函数也是一个成员。而且函数与`consts`不同，具有*可以*被遮蔽的属性——而且由于它们是项，它们也可以遮蔽后者（如果在较小的作用域中引入）。

> So, she introduces that new scope into her macro, and inside it, defines a dummy function. As it happens, functions are never valid in patterns, and so the `x @ _` trick is no longer needed.

因此，她在宏中引入了这个新作用域，并在其中定义了一个伪函数。恰好函数在模式中从不有效，所以`x @ _`技巧不再需要。

```rust
macro_rules! oh_my {
	() => {{
        #[allow(dead_code)]
        fn x() {}
		let Ok(x) = read_input() else { return Err(Error) };
		$crate::process(x);
    }};
}
```

> And despite Wolfie’s attempts to break it, this iteration remains hygienic even in the presence of strange environments.

尽管 Wolfie 试图破坏它，但这次迭代仍然保持卫生性，即使是在奇怪的环境中。

> But Remon isn’t satisfied. Because now, being the responsible library developer she is, whenever she uses this trick, she must document it. And she has to introduce a shadowing helper function for every single identifier used in the macro – something that is very easy to forget, negating the benefit of using this trick in the first place. It increases her codebase’s size, in an already-complex macro, for a gain that seems marginal at best.

但 Remon 并不满意。因为现在，作为负责任的库开发者，每当她使用这个技巧时，她都必须记录下来。而且她必须为宏中使用的每个标识符引入一个遮蔽辅助函数——这是很容易忘记的事情，否定了首先使用这个技巧的好处。在一个已经复杂的宏中，它增加了代码库的大小，而收益看起来最多只是边际性的。

> And so, against her instincts to be fully correct, Remon turns to Wolfie and says, plainly, _No_. With the incantation of a `git reset`, she erases these changes from history, choosing instead to live in the ignorant bliss of very-slightly-unhygienic declarative macros.

因此，Remon 违背了她完全正确的本能，转向 Wolfie 并直接说："不"。通过`git reset`的咒语，她从历史中抹去了这些更改，选择生活在略微不卫生的声明式宏的无知幸福中。

> After all, who names constants in lowercase anyway?

毕竟，谁会用小写命名常量呢？

