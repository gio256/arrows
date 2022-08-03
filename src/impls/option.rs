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
