pub trait FunctorFamily {
    type M<T>: Functor<T, FFamily = Self>;
}

pub trait Functor<A> {
    type FFamily: FunctorFamily<M<A> = Self>;

    // fmap (<$>) :: (a -> b) -> f a -> f b
    fn fmap<F, B>(self, f: F) -> <Self::FFamily as FunctorFamily>::M<B>
    where
        F: Fn(A) -> B;

    // (<$) :: a -> f b -> f a
    fn map_replace<B>(self, b: B) -> <Self::FFamily as FunctorFamily>::M<B>
    where
        B: Clone,
        Self: Sized,
    {
        self.fmap(|_| b.clone())
    }
}

pub trait ApplicativeFamily {
    type M<T>: Applicative<T, AFamily = Self>;
}

pub trait Applicative<A>: Functor<A> {
    type AFamily: ApplicativeFamily<M<A> = Self>;

    // a -> f a
    fn pure(a: A) -> Self;

    // (a -> b -> c) -> f a -> f b -> f c
    fn lift_a2<F, B, C>(
        self,
        fb: <Self::AFamily as ApplicativeFamily>::M<B>,
        f: F,
    ) -> <Self::AFamily as ApplicativeFamily>::M<C>
    where
        F: Fn(A, B) -> C;

    // (<*>) :: f (a -> b) -> f a -> f b
    fn apply<F, B>(
        self,
        func: <Self::AFamily as ApplicativeFamily>::M<F>,
    ) -> <Self::AFamily as ApplicativeFamily>::M<B>
    where
        F: Fn(A) -> B,
        Self: Sized,
    {
        self.lift_a2(func, |a, f: F| f(a))
    }
}

pub trait MonadFamily {
    type M<T>: Monad<T, MFamily = Self>;
}

pub trait Monad<A>: Applicative<A> {
    type MFamily: MonadFamily<M<A> = Self>;

    // bind (>>=) :: m a -> (a -> m b) -> m b
    fn bind<F, B>(self, f: F) -> <Self::MFamily as MonadFamily>::M<B>
    where
        F: Fn(A) -> <Self::MFamily as MonadFamily>::M<B>;

    // compose (>=>) :: (a -> m b) -> (b -> m c) -> a -> m c
    fn compose<F, G, B, C>(f: F, g: G, a: A) -> <Self::MFamily as MonadFamily>::M<C>
    where
        F: FnOnce(A) -> <Self::MFamily as MonadFamily>::M<B>,
        G: Fn(B) -> <Self::MFamily as MonadFamily>::M<C>,
    {
        f(a).bind::<_, C>(g)
    }

    // m (m a) -> m a
    fn join(mma: <Self::MFamily as MonadFamily>::M<<Self::MFamily as MonadFamily>::M<A>>) -> Self
    where
        Self: Sized,
    {
        mma.bind::<_, A>(std::convert::identity)
    }
}

// what trait bounds would look like
#[allow(unused)]
fn fmap_add<F, A>(f: F, n: usize) -> <F::FFamily as FunctorFamily>::M<A::Output>
where
    F: Functor<A>,
    A: std::ops::Add<usize>,
{
    f.fmap(|a| a + 1)
}
