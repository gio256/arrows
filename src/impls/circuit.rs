use crate::arrow::*;

pub struct CircuitFamily;

impl CatFamily for CircuitFamily {
    type M<A, B> = Circuit<A, B>;

    fn id<A>() -> Self::M<A, A> {
        let id = |a| (Self::id(), a);
        Circuit::new(id)
    }
}

pub struct Circuit<A, B> {
    step: Box<dyn FnOnce(A) -> (Circuit<A, B>, B)>,
}

impl<A, B> Category<A, B> for Circuit<A, B> {
    type CFamily = CircuitFamily;

    fn then<C>(self, consumer: Circuit<B, C>) -> Circuit<A, C>
    where
        A: 'static,
        B: 'static,
        C: 'static,
    {
        let inner = |a| {
            let (producer, b) = self.call(a);
            let (consumer, c) = consumer.call(b);
            (producer.then(consumer), c)
        };
        Circuit::new(inner)
    }

    // (.) or (<<<) :: Category cat => cat b c -> cat a b -> cat a c
    fn after<A0>(self, producer: Circuit<A0, A>) -> Circuit<A0, B>
    where
        A: 'static,
        B: 'static,
        A0: 'static,
    {
        producer.then(self)
    }
}

impl ArrowFamily for CircuitFamily {
    type M<A, B> = Circuit<A, B>;
}

impl<A, B> Arrow<A, B> for Circuit<A, B> {
    type AFamily = CircuitFamily;

    fn arrow<F>(f: F) -> Self
    where
        F: Fn(A) -> B + Clone + 'static,
    {
        f.into()
    }

    // first :: m a b -> m (a, c) (b, c)
    fn fst<C>(self) -> Circuit<(A, C), (B, C)>
    where
        A: 'static,
        B: 'static,
    {
        let f = |(a, c)| {
            let (circ_next, b) = self.call(a);
            (circ_next.fst(), (b, c))
        };
        Circuit::new(f)
    }

    // second :: m a b -> m (c,a) (c,b)
    fn snd<C>(self) -> <Self::AFamily as ArrowFamily>::M<(C, A), (C, B)>
    where
        Self: Sized,
        A: 'static,
        B: 'static,
        C: 'static,
    {
        arrow(std::convert::identity).both(self)
    }

    // (***) :: a b c -> a b' c' -> a (b,b') (c,c')
    // f *** g = first f >>> arr swap >>> first g >>> arr swap   where swap ~(x,y) = (y,x)
    fn both<A1, B1>(
        self,
        right_arrow: <Self::AFamily as ArrowFamily>::M<A1, B1>,
    ) -> <Self::AFamily as ArrowFamily>::M<(A, A1), (B, B1)>
    where
        A: 'static,
        B: 'static,
        A1: 'static,
        B1: 'static,
    {
        let swap0 = arrow(|(b, a1)| (a1, b));
        let swap1 = arrow(|(b1, b)| (b, b1));
        self.fst().then(swap0).then(right_arrow.fst()).then(swap1)
    }

    // (&&&) :: m a b -> m a b' -> m a (b,b')
    // f &&& g = arr (\a -> (a,a)) >>> f *** g
    fn fan<B1>(
        self,
        right_arrow: <Self::AFamily as ArrowFamily>::M<A, B1>,
    ) -> <Self::AFamily as ArrowFamily>::M<A, (B, B1)>
    where
        Self: Sized,
        A: Clone + 'static,
        B: 'static,
        B1: 'static,
    {
        arrow(|a: A| (a.clone(), a)).then(self.both(right_arrow))
    }
}

impl<F, A, B> From<F> for Circuit<A, B>
where
    F: FnOnce(A) -> B + Clone + 'static,
{
    fn from(f: F) -> Self {
        let inner = move |a| (Self::from(f.clone()), f(a));
        Circuit::new(inner)
    }
}

pub fn arrow<F, A, B>(f: F) -> Circuit<A, B>
where
    F: FnOnce(A) -> B + Clone + 'static,
{
    f.into()
}

impl<A, B> Circuit<A, B> {
    fn new<F>(f: F) -> Self
    where
        F: FnOnce(A) -> (Self, B) + 'static,
    {
        Self { step: Box::new(f) }
    }

    fn call(self, a: A) -> (Self, B) {
        (self.step)(a)
    }

    fn run(self, xs: impl IntoIterator<Item = A>) -> impl Iterator<Item = B> {
        xs.into_iter().scan(Some(self), |cir, x| {
            let (cir_new, item) = cir.take()?.call(x);
            *cir = Some(cir_new);
            Some(item)
        })
    }

    fn accum<Acc, F>(acc: Acc, mut f: F) -> Self
    where
        Acc: 'static,
        F: FnMut(A, Acc) -> (B, Acc) + 'static,
    {
        let g = |a| {
            let (res, acc_new) = f(a, acc);
            (Self::accum(acc_new, f), res)
        };
        Self::new(g)
    }

    fn accum_show<F>(acc: B, mut f: F) -> Self
    where
        F: FnMut(A, B) -> B + 'static,
        B: Clone + 'static,
    {
        Self::accum(acc, move |a, b| {
            let b_new = f(a, b);
            (b_new.clone(), b_new)
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use std::ops;

    #[test]
    fn test_cat() {
        let cat = CircuitFamily::id::<usize>();
        let res: Vec<_> = cat.run(vec![1, 2]).collect();
        dbg!(res);
    }

    #[test]
    fn test_accum() {
        let total = Circuit::accum_show(0, ops::Add::add);
        let res: Vec<_> = total.run(vec![1, 0, 1, 0, 0, 2]).collect();
        assert_eq!(res, vec![1, 1, 2, 2, 2, 4])
    }
}
