use crate::{arrow::*, impls::either::Either};

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

    // (>>>) :: Category cat => cat a b -> cat b c -> cat a c
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

fn flip<A, B>((a, b): (A, B)) -> (B, A) {
    (b, a)
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
    fn both<A1, B1>(self, right_arrow: Circuit<A1, B1>) -> Circuit<(A, A1), (B, B1)>
    where
        A: 'static,
        B: 'static,
        A1: 'static,
        B1: 'static,
    {
        self.fst()
            .then_pure(flip)
            .then(right_arrow.fst())
            .then_pure(flip)
    }

    // (&&&) :: m a b -> m a b' -> m a (b,b')
    // f &&& g = arr (\a -> (a,a)) >>> f *** g
    fn dup<B1>(self, right_arrow: Circuit<A, B1>) -> Circuit<A, (B, B1)>
    where
        Self: Sized,
        A: Clone + 'static,
        B: 'static,
        B1: 'static,
    {
        self.both(right_arrow).after_pure(|a: A| (a.clone(), a))
    }

    // precomposition with a pure function
    // (^>>) :: Arrow m => (b -> c) -> m c d -> m b d
    fn after_pure<F, A0>(self, f: F) -> Circuit<A0, B>
    where
        F: Fn(A0) -> A + Clone + 'static,
        A: 'static,
        B: 'static,
        A0: 'static,
    {
        arrow(f).then(self)
    }

    // postcomposition with a pure function
    // (>>^) :: Arrow m => m b c -> (c -> d) -> m b d
    fn then_pure<F, C>(self, f: F) -> Circuit<A, C>
    where
        F: Fn(B) -> C + Clone + 'static,
        A: 'static,
        B: 'static,
        C: 'static,
    {
        self.then(arrow(f))
    }
}

impl ChoiceFamily for CircuitFamily {
    type M<A, B> = Circuit<A, B>;
}

impl<A, B> ArrowChoice<A, B> for Circuit<A, B> {
    type AcFamily = CircuitFamily;

    // left :: a b c -> a (Either b d) (Either c d)
    fn left<D>(self) -> Circuit<Either<A, D>, Either<B, D>>
    where
        A: 'static,
        B: 'static,
    {
        let f = move |either_ad| match either_ad {
            Either::Left(a) => {
                let (circ_new, res) = self.call(a);
                (circ_new.left(), Either::Left(res))
            }
            Either::Right(d) => (self.left(), Either::Right(d)),
        };
        Circuit::new(f)
    }

    // right :: a b c -> a (Either d b) (Either d c)
    fn right<D>(self) -> Circuit<Either<D, A>, Either<D, B>>
    where
        A: 'static,
        B: 'static,
        D: 'static,
    {
        arrow(std::convert::identity).split(self)
    }

    // (+++) :: a b c -> a b' c' -> a (Either b b') (Either c c')
    fn split<A1, B1>(self, g: Circuit<A1, B1>) -> Circuit<Either<A, A1>, Either<B, B1>>
    where
        A: 'static,
        B: 'static,
        A1: 'static,
        B1: 'static,
    {
        self.left()
            .then_pure(Either::flip)
            .then(g.left())
            .then_pure(Either::flip)
    }

    // (|||) :: m a b -> m c b -> m (Either a c) b
    fn owise<C>(self, g: Circuit<C, B>) -> Circuit<Either<A, C>, B>
    where
        A: 'static,
        B: 'static,
        C: 'static,
    {
        self.split(g).then_pure(|either| match either {
            Either::Left(x) => x,
            Either::Right(y) => y,
        })
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

impl<A> Circuit<A, A> {
    #[cfg(test)]
    fn id() -> Self {
        CircuitFamily::id()
    }
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

    #[cfg(test)]
    fn output(self, a: A) -> B {
        self.call(a).1
    }

    #[cfg(test)]
    fn run(self, xs: impl IntoIterator<Item = A>) -> impl Iterator<Item = B> {
        xs.into_iter().scan(Some(self), |circ, x| {
            let (circ_new, item) = circ.take()?.call(x);
            *circ = Some(circ_new);
            Some(item)
        })
    }

    #[cfg(test)]
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

    #[cfg(test)]
    fn accum_dup<F>(acc: B, mut f: F) -> Self
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
    use super::*;
    use rand::{thread_rng, Rng};
    use std::ops;

    #[test]
    fn test_category() {
        let cat = Circuit::id();
        let res: Vec<_> = cat.run(vec![1, 2, 3, 4]).collect();
        assert_eq!(res, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_arrow() {
        let eq2 = arrow(|n| n == 2).dup(Circuit::id()).output(2);
        assert_eq!(eq2, (true, 2));

        // if n is even { Some(n + 1) } else { None }
        let inc_even_else_none = |n| {
            arrow(|x| x & 1 == 0)
                .dup(Circuit::id())
                .then_pure(Either::from)
                .then(arrow(|_| None).owise(arrow(|n| Some(n + 1))))
                .output(n)
        };
        assert_eq!(inc_even_else_none(1), None);
        assert_eq!(inc_even_else_none(2), Some(3));
        assert_eq!(inc_even_else_none(3), None);
        assert_eq!(inc_even_else_none(4), Some(5));
    }

    fn total() -> Circuit<usize, usize> {
        Circuit::accum_dup(0, ops::Add::add)
    }

    fn oneshot() -> Circuit<(), bool> {
        Circuit::accum(true, |(), acc| (acc, false))
    }

    #[test]
    fn test_hangman() {
        let res: Vec<_> = total().run(vec![1, 0, 1, 0, 0, 2]).collect();
        assert_eq!(res, vec![1, 1, 2, 2, 2, 4]);

        let const1 = arrow(|_| 1);
        let uncurry_div = arrow(|(a, b)| a / b);
        let get_mean = total().dup(const1.then(total())).then(uncurry_div);

        let running_avg: Vec<_> = get_mean.run(vec![1, 5, 8, 12, 100]).collect();
        assert_eq!(running_avg, vec![1, 3, 4, 6, 25]);

        // return true once, then false forever
        // let oneshot = Circuit::accum(true, |(), acc| (acc, false));
        let res: Vec<_> = oneshot().run([(), (), (), ()]).collect();
        assert_eq!(res, vec![true, false, false, false]);

        // returns the last value it was given
        let delayed_echo = |acc| Circuit::accum(acc, |a, b| (b, a));
        let res: Vec<_> = delayed_echo(false)
            .run([true, false, true, false])
            .collect();
        assert_eq!(res, vec![false, true, false, true]);

        let dict = ["foo", "bar", "baz"];
        let rng = thread_rng();
        let range = 0..(dict.len() - 1);

        // choose a random word from the wordlist
        let generator = Circuit::accum(rng, move |(), mut rng| (rng.gen_range(range.clone()), rng));
        let pick_word = generator.then_pure(move |i| dict[i]);

        // pick the word and then repeat it
        let get_word = oneshot()
            .then_pure(Either::from)
            .then(arrow(|_| None).owise(pick_word.then_pure(Some)))
            .then(Circuit::accum_dup(None, Option::or));

        let mut repeat_word: Vec<_> = get_word.run([(), (), (), ()]).collect();
        assert_eq!(repeat_word.len(), 4);
        repeat_word.dedup();
        assert_eq!(repeat_word.len(), 1);
        assert!(dict.contains(&repeat_word[0].unwrap()));
    }
}
