# 声明宏案例

这些案例的代码不算典型，**仅供搜集**。

## DSL：伪重载运算符

```rust
#![feature(min_specialization)]
# // 原帖：https://rustcc.cn/article?id=d68047dd-4003-4ac7-bef1-10950ab57404
# // 这里的实现并不重要，只是一个例子
#pub trait CustomAssign<Rhs = Self> {
#    // bind to a~b
#    fn custom_assign(&mut self, rhs: Rhs); // a~b means a.custom_assign(b)
#}
#pub trait CustomInitialize<Rhs = Self> {
#    // bind to a:=b
#    fn custom_initialize(rhs: Rhs) -> Self; // a:=b means a=<type of a>::custom_initialize(rhs);
#}
#impl<T> CustomAssign<T> for T {
#    #[inline(always)]
#    default fn custom_assign(&mut self, rhs: T) { *self = rhs }
#}
#impl<T: Copy> CustomAssign<&T> for T {
#    #[inline(always)]
#    default fn custom_assign(&mut self, rhs: &T) { *self = *rhs }
#}
#impl<'a, T: CustomAssign<&'a T>> CustomAssign<&'a mut T> for T {
#    #[inline(always)]
#    default fn custom_assign(&mut self, rhs: &'a mut T) { self.custom_assign(rhs as &T) }
#}
#impl<T> CustomInitialize<T> for T {
#    #[inline(always)]
#    default fn custom_initialize(rhs: T) -> Self { rhs }
#}
#impl<T: Copy> CustomInitialize<&T> for T {
#    #[inline(always)]
#    default fn custom_initialize(rhs: &T) -> Self { *rhs }
#}
#impl<'a, T: CustomInitialize<&'a T>> CustomInitialize<&'a mut T> for T {
#    #[inline(always)]
#    default fn custom_initialize(rhs: &'a mut T) -> Self { Self::custom_initialize(rhs as &T) }
#}
#impl CustomAssign<i64> for i32 {
#    fn custom_assign(&mut self, rhs: i64) { *self = rhs as i32; }
#}
#impl CustomInitialize<i64> for i32 {
#    fn custom_initialize(rhs: i64) -> Self { rhs as i32 }
#}

macro_rules! cai_exec {
    ($id:ident ~ $ex:expr; $($tail:tt)*) => {
        $id.custom_assign($ex);
        cai_exec!($($tail)*);
    };
    ($($id:ident)+ $(: $type:ty)? : = $ex:expr; $($tail:tt)*) => {
        $($id)+ $(: $type)? = CustomInitialize::custom_initialize($ex);
        cai_exec!($($tail)*);
    };
    ($st:stmt; $($tail:tt)*) => {
        $st
        cai_exec!($($tail)*);
    };
    ($ex:expr) => {
        $ex
    };
    () => {};
}

fn main() {
    cai_exec! {let _a:i32 :=1;};
    cai_exec! {{let _a:i32 =1;}};
    cai_exec! {let mut a:i32 :=1; a~1;};
    cai_exec! {
        let _a=1;           // stmt
        let mut a:i32 :=1;  // :=初始化语句，需要手工翻译
        a~1;                // 赋值语句，需要手工翻译
        a+=1;               // stmt
        a.custom_assign(1)  // expr，有可能不以分号结尾
    };
}
```

## 自动生成 `new` 方法

```rust
#// src: https://rustcc.cn/article?id=5dbddd4b-4a25-48bd-a78d-8e8d0a952346
#![allow(unused)]

macro_rules! struct_new {
    ($(#[$attr:meta])* $vis:vis struct $s_name:ident ; $($tail:tt)*) => {
        $vis struct $s_name ;
        struct_new! { $($tail)* }
    };
    (
        $(#[$attr:meta])* $vis:vis struct $s_name:ident $(<$($generic:tt),*>)?
        $(
            where $($where_id:tt : $where_tr:tt),*
        )?
        $(
            ($($p_vis:vis $p_name:ident: $p_type:ty),* $(,)?)
        )?
        { $($field_vis:vis $field:ident: $type:ty = $val:expr),* $(,)? }
        $($tail:tt)*
    ) => {
        $(#[$attr])*
        $vis struct $s_name $(<$($generic),*>)?
        $(where $($where_id : $where_tr),* )?
        {
            $($($p_vis $p_name: $p_type,)*)?
            $($field_vis $field: $type,)*
        }
        impl $(<$($generic),*>)? $s_name $(<$($generic),*>)?
            $(where $($where_id : $where_tr),* )? {
            fn new($($($p_name: $p_type),*)?) -> Self {
                $s_name {
                    $($($p_name,)*)?
                    $($field: $val,)*
                }
            }
        }
        struct_new! { $($tail)* }
    };
    () => {};
}

struct_new! {
    #[derive(Debug)]
    struct A<'a>(foo: u8, pub bar: &'a str,) {
        abc: u8 = 255,
        xyz: &'a str = "xyz",
    }
    struct B {}
    #[derive(Debug)]
    struct C;
    #[derive(Debug)]
    struct D<T> where T: Copy (foo: u8, pub bar: T,) {
        abc: u8 = 255,
    }
}

fn main() {
    let a = A::new(1, "1");
    dbg!(a);
    let d = D::new(1, "1");
    dbg!(d);
}
```

相关项目：
1. 使用过程宏创建结构体和枚举体的构造函数（不局限于 `new`）： [derive-new](https://crates.io/crates/derive-new)

## 从同质 variants 取同类型数据

https://rustcc.cn/article?id=f4dbc7c9-3eef-467f-9a47-df28d53936cb
