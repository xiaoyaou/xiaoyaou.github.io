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

    scheduler.add_system(|mut x: ResMut<i32>, y: Res<&str>| {
        *x += 1;
        println!("{} again with {}", *y, *x);
    });

    scheduler.add_system(|_: ResMut<i32>, _: Res<i32>| {
        panic!("this should panic earlier")
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
            for <'a> F: FnMut($($param),*) + FnMut($($param::Item<'a>),*),
        {
            fn run(&mut self, resources: &HashMap<TypeId, RefCell<Box<dyn Any>>>) {
                $(let $param = $param::retrieve(resources);)*
                (self.f)($($param),*)
            }
        }
    };
}
/// 为多参数函数对象实现[`IntoSystem`]系统转换特征
macro_rules! impl_into_system {
    ($($param:ident),*) => {
        impl<F, $($param: SystemParam),*> IntoSystem<($($param,)*)> for F
        where
            for<'a> F: FnMut($($param),*) + FnMut($($param::Item<'a>),*),
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
expand_times!(impl_system      @ (T1, T2, T3, T4, T5, T6, T7, T8));
// 实现多参数系统转换特征
expand_times!(impl_into_system @ (T1, T2, T3, T4, T5, T6, T7, T8));
// 实现多元素系统参数元组
expand_times!(impl_tuple_param @ (T1, T2, T3, T4, T5, T6, T7, T8));

