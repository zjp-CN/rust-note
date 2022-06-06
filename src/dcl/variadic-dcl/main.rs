#![allow(unused)]

pub struct Ui {}
impl Ui {
    pub fn update<T>(&mut self, a: String, b: Vec<T>, f: impl FnOnce(&mut Ui))
        where T: PartialEq + Clone + 'static {
    }
}

pub trait Component<Params, Content> {
    fn call(&self, ui: &mut Ui, params: Params, content: Content);
}

macro_rules! impl_component {
    ($n:expr, $p:ident $($P:ident)*) => {
        impl<F, $p, $($P,)* Content> $crate::Component<( $p, $($P,)* ), Content> for F
            where F: Fn(&mut Ui, $p, $($P,)* Content),
                  $p: ::std::cmp::PartialEq + ::std::clone::Clone + 'static,
                  $( $P: ::std::cmp::PartialEq + ::std::clone::Clone + 'static, )*
                  Content: ::std::ops::FnOnce(&mut Ui)
        {

            fn call(&self, ui: &mut Ui, params: ( $p, $($P,)* ), content: Content) {
                #[allow(non_snake_case)]
                let ($p, $($P,)*) = params;
                self(ui, $p, $($P,)* content)
            }
        }
        impl_component!{$n-1, $($P)*}
    };
    ($n:expr,) => { };
}

impl_component!(12, P12 P11 P10 P9 P8 P7 P6 P5 P4 P3 P2 P1);

pub fn memoize<Params, Content, Comp>(ui: &mut Ui, component: Comp, params: Params,
                                      content: Content)
    where Params: PartialEq + Clone + 'static,
          Content: FnOnce(&mut Ui),
          Comp: Component<Params, Content>
{
    component.call(ui, params, content);
}

fn comp1(ui: &mut Ui, a: u8, f: impl FnOnce(&mut Ui)) { f(ui); }
fn comp_(ui: &mut Ui, a: &str, f: impl FnOnce(&mut Ui)) { f(ui); }
fn comp2(ui: &mut Ui, a: u8, b: u32, f: impl FnOnce(&mut Ui)) { f(ui); }
fn comp3(ui: &mut Ui, a: u8, b: u32, c: u64, f: impl FnOnce(&mut Ui)) { f(ui); }
fn comp4(ui: &mut Ui, a: u8, b: u32, c: u64, d: usize, f: impl FnOnce(&mut Ui)) { f(ui); }
#[allow(clippy::too_many_arguments)]
fn comp12(ui: &mut Ui, p1: u8, p2: u8, p3: u8, p4: u8, p5: u8, p6: u8, p7: u8, p8: u8, p9: u8,
          p10: u8, p11: u8, p12: u8, f: impl FnOnce(&mut Ui)) {
    f(ui);
}

fn main() {
    let mut ui = Ui {};
    memoize(&mut ui, comp1, (1,), |_| {});
    memoize(&mut ui, comp_, ("",), |_| {});
    memoize(&mut ui, comp2, (2, 3), |_| {});
    memoize(&mut ui, comp3, (1, 2, 3), |_| {});
    memoize(&mut ui, comp4, (0, 1, 2, 3), |_| {});
    memoize(&mut ui, comp12, (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0), |_| {});

    let args = (String::new(), vec![(1usize, 1.0f64)]);
    memoize(&mut ui, Ui::update, args, |_| {});
}
