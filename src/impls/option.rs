use crate::monad::*;

pub struct OptionFamily;

impl FunctorFamily for OptionFamily {
    type M<T> = Option<T>;
}
impl<A> Functor<A> for Option<A> {
    type FFamily = OptionFamily;

    // fmap (<$>) :: (a -> b) -> f a -> f b
    fn fmap<F, B>(self, f: F) -> Option<B>
    where
        F: Fn(A) -> B,
    {
        self.map(f)
    }
}
impl ApplicativeFamily for OptionFamily {
    type M<T> = Option<T>;
}
impl<A> Applicative<A> for Option<A> {
    type AFamily = OptionFamily;

    // a -> f a
    fn pure(a: A) -> Self {
        Some(a)
    }

    // (a -> b -> c) -> f a -> f b -> f c
    fn lift_a2<F, B, C>(self, opt_b: Option<B>, f: F) -> Option<C>
    where
        F: Fn(A, B) -> C,
    {
        Some(f(self?, opt_b?))
    }

    // (<*>) :: f (a -> b) -> f a -> f b
    fn apply<F, B>(self, maybe_fn: Option<F>) -> Option<B>
    where
        F: Fn(A) -> B,
    {
        Some((maybe_fn?)(self?))
    }
}

impl MonadFamily for OptionFamily {
    type M<T> = Option<T>;
}
impl<A> Monad<A> for Option<A> {
    type MFamily = OptionFamily;

    // bind (>>=) :: m a -> (a -> m b) -> m b
    fn bind<F, B>(self, f: F) -> Option<B>
    where
        F: Fn(A) -> Option<B>,
    {
        self.and_then(f)
    }

    // compose (>=>) :: (a -> m b) -> (b -> m c) -> a -> m c
    fn compose<F, G, B, C>(f: F, g: G, a: A) -> Option<C>
    where
        F: FnOnce(A) -> Option<B>,
        G: Fn(B) -> Option<C>,
    {
        g(f(a)?)
    }

    // m (m a) -> m a
    fn join(opt_opt: Option<Option<A>>) -> Self
    where
        Self: Sized,
    {
        opt_opt.and_then(std::convert::identity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opt() {
        // functor
        let none: Option<usize> = None;
        let opt = Some(1);
        assert_eq!(opt.fmap(|x| x * 2), Some(2));
        assert_eq!(none.fmap(|x| x * 2), None);

        assert_eq!(opt.map_replace(7), Some(7));
        assert_eq!(none.map_replace(7), None);

        // applicative
        assert_eq!(Option::pure(5), Some(5));

        let f = Some(|x| x + 3);
        let f_none: Option<fn(usize) -> usize> = None;
        // let res = vec.clone().apply(fs);
        assert_eq!(opt.apply(f), Some(4));
        assert_eq!(none.apply(f), None);
        assert_eq!(opt.apply(f_none), None);

        // monad
        assert_eq!(opt.bind(Some), opt);

        let double_opt = Some(opt);
        assert_eq!(Option::join(double_opt), opt);
    }
}
