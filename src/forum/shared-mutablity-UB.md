# 共享可变的错误/UB 做法

## 多线程中缺乏同步机制

<https://users.rust-lang.org/t/why-cant-self-lambda-be-fnmut-when-self-is-borrowed-immutably/113989/4>


```rust
#![feature(sync_unsafe_cell)]

pub struct Analyzer<F> {
    postprocess: ::core::cell::SyncUnsafeCell<F>,
}

impl<F: FnMut(i32) -> i32> Analyzer<F> {
    fn process(&self, n: i32) -> i32 { n + 1 }
    pub fn pipeline(&self, n: i32) -> i32 {
        let n = self.process(n);
        // 1. let's assume we were allowed to get `&mut postprocess`
        // e.g., here, using (unsound) `unsafe` to make Rust look
        // other way.
        let postprocess: &mut F = unsafe {
            &mut *self.postprocess.get()
        };
        postprocess(n)
    }
}

fn main() {
    let mut v = Vec::new();
    let mut total = 0;
    let postprocess = |n| {
        v.push(1);
        total += n;
        n + 1
    };
    let analyzer = Analyzer {
        postprocess: ::core::cell::SyncUnsafeCell::new(postprocess),
    };
    // 2. then, the following code compiles fine:
    ::std::thread::scope(|s| {
        _ = s.spawn(|| analyzer.pipeline(1));
        _ = s.spawn(|| analyzer.pipeline(2));
    });
    let res = analyzer.pipeline(1);
    println!("{res}");
}
```

```rust
error: Undefined Behavior: Data race detected between (1) retag write on thread `unnamed-1` and (2) retag write of type `{closure@src/main.rs:24:23: 24:26}` on thread `unnamed-2` at alloc1096. (2) just happened here
  --> src/main.rs:15:13
   |
15 |             &mut *self.postprocess.get()
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Data race detected between (1) retag write on thread `unnamed-1` and (2) retag write of type `{closure@src/main.rs:24:23: 24:26}` on thread `unnamed-2` at alloc1096. (2) just happened here
   |
help: and (1) occurred earlier here
  --> src/main.rs:24:23
   |
24 |       let postprocess = |n| {
   |  _______________________^
25 | |         v.push(1);
26 | |         total += n;
27 | |         n + 1
28 | |     };
   | |_____^
   = help: retags occur on all (re)borrows and as well as when references are copied or moved
   = help: retags permit optimizations that insert speculative reads or writes
   = help: therefore from the perspective of data races, a retag has the same implications as a read or write
   = help: this indicates a bug in the program: it performed an invalid operation, and caused Undefined Behavior
   = help: see https://doc.rust-lang.org/nightly/reference/behavior-considered-undefined.html for further information
   = note: BACKTRACE (of the first span) on thread `unnamed-2`:
   = note: inside `Analyzer::<{closure@src/main.rs:24:23: 24:26}>::pipeline` at src/main.rs:15:13: 15:41
note: inside closure
  --> src/main.rs:35:24
   |
35 |         _ = s.spawn(|| analyzer.pipeline(2));
   |                        ^^^^^^^^^^^^^^^^^^^^
```

## 单线程中打破别名规则

<https://users.rust-lang.org/t/why-cant-self-lambda-be-fnmut-when-self-is-borrowed-immutably/113989/5>

```rust
use std::cell::UnsafeCell;
use std::rc::Rc;

pub struct Analyzer<F> {
    postprocess: UnsafeCell<F>,
}

impl<F: FnMut(i32) -> i32> Analyzer<F> {
    fn process(&self, n: i32) -> i32 { n + 1 }
    pub fn pipeline(&self, n: i32) -> i32 {
        let n = self.process(n);
        // 1. let's assume we were allowed to get `&mut postprocess`
        // e.g., here, using (unsound) `unsafe` to make Rust look
        // other way.
        let postprocess: &mut F = unsafe {
            &mut *self.postprocess.get()
        };
        postprocess(n)
    }
}

fn main() {
    let analyzer = Rc::<Analyzer<Box<dyn FnMut(i32) -> i32>>>::new_cyclic(|analyzer| {
        let analyzer = analyzer.clone();
        let mut s = String::new();
        let mut first = true;
        let postprocess = Box::new(move |n| {
            s = String::from("foo");
            let s_ref = &*s;
            
            if first {
                first = false;
                // 2. here a re-entrant call is made, leading to a second call to
                // postprocess while the first one is still running and
                // ultimately creating two aliasing mutable references.
                // This is then exploited to access deallocated data
                // when printing s_ref in the first call, as at that point
                // the second call will have replaced s.
                analyzer.upgrade().unwrap().pipeline(0);
            }
            
            println!("{s_ref}");
            n
        });
        
        Analyzer { postprocess: UnsafeCell::new(postprocess) }
    });

    analyzer.pipeline(1);
}
```

```rust
error: Undefined Behavior: not granting access to tag <3301> because that would remove [Unique for <2766>] which is strongly protected because it is an argument of call 646
  --> src/main.rs:16:13
   |
16 |             &mut *self.postprocess.get()
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ not granting access to tag <3301> because that would remove [Unique for <2766>] which is strongly protected because it is an argument of call 646
   |
   = help: this indicates a potential bug in the program: it performed an invalid operation, but the Stacked Borrows rules it violated are still experimental
   = help: see https://github.com/rust-lang/unsafe-code-guidelines/blob/master/wip/stacked-borrows.md for further information
help: <3301> was created by a SharedReadWrite retag at offsets [0x10..0x20]
  --> src/main.rs:16:19
   |
16 |             &mut *self.postprocess.get()
   |                   ^^^^^^^^^^^^^^^^^^^^^^
help: <2766> is this argument
  --> src/main.rs:18:9
   |
18 |         postprocess(n)
   |         ^^^^^^^^^^^^^^
   = note: BACKTRACE (of the first span):
   = note: inside `Analyzer::<std::boxed::Box<dyn std::ops::FnMut(i32) -> i32>>::pipeline` at src/main.rs:16:13: 16:41
note: inside closure
  --> src/main.rs:39:17
   |
39 |                 analyzer.upgrade().unwrap().pipeline(0);
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: inside `<std::boxed::Box<dyn std::ops::FnMut(i32) -> i32> as std::ops::FnMut<(i32,)>>::call_mut` at /playground/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:2071:9: 2071:49
note: inside `Analyzer::<std::boxed::Box<dyn std::ops::FnMut(i32) -> i32>>::pipeline`
  --> src/main.rs:18:9
   |
18 |         postprocess(n)
   |         ^^^^^^^^^^^^^^
note: inside `main`
  --> src/main.rs:49:5
   |
49 |     analyzer.pipeline(1);
   |     ^^^^^^^^^^^^^^^^^^^^
```
