use crate::monad::*;

pub struct VecFamily;

impl FunctorFamily for VecFamily {
    type M<T> = Vec<T>;
}
impl<A> Functor<A> for Vec<A> {
    type FFamily = VecFamily;

    // fmap (<$>) :: (a -> b) -> f a -> f b
    fn fmap<F, B>(self, f: F) -> Vec<B>
    where
        F: Fn(A) -> B,
    {
        self.into_iter().map(f).collect()
    }
}
impl ApplicativeFamily for VecFamily {
    type M<T> = Vec<T>;
}
impl<A> Applicative<A> for Vec<A> {
    type AFamily = VecFamily;

    // a -> f a
    fn pure(a: A) -> Self {
        vec![a]
    }

    // (a -> b -> c) -> f a -> f b -> f c
    fn lift_a2<F, B, C>(self, vec_b: Vec<B>, f: F) -> Vec<C>
    where
        F: Fn(A, B) -> C,
    {
        self.into_iter()
            .zip(vec_b.into_iter())
            .map(|(a, b)| f(a, b))
            .collect()
    }

    // (<*>) :: f (a -> b) -> f a -> f b
    fn apply<F, B>(self, fns: Vec<F>) -> Vec<B>
    where
        F: Fn(A) -> B,
    {
        fns.into_iter()
            .zip(self.into_iter())
            .map(|(f, a)| f(a))
            .collect()
    }
}

impl MonadFamily for VecFamily {
    type M<T> = Vec<T>;
}
impl<A> Monad<A> for Vec<A> {
    type MFamily = VecFamily;

    // bind (>>=) :: m a -> (a -> m b) -> m b
    fn bind<F, B>(self, f: F) -> Vec<B>
    where
        F: Fn(A) -> Vec<B>,
    {
        self.into_iter().flat_map(f).collect()
    }

    // compose (>=>) :: (a -> m b) -> (b -> m c) -> a -> m c
    fn compose<F, G, B, C>(f: F, g: G, a: A) -> Vec<C>
    where
        F: FnOnce(A) -> Vec<B>,
        G: Fn(B) -> Vec<C>,
    {
        f(a).into_iter().flat_map(g).collect()
    }

    // m (m a) -> m a
    fn join(vec_vec: Vec<Vec<A>>) -> Self {
        vec_vec.into_iter().flatten().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec() {
        // functor
        let vec = vec![1, 2, 3, 4];
        let res = vec.clone().fmap(|x| x * 2);
        assert_eq!(res, vec![2, 4, 6, 8]);

        let res = vec.clone().map_replace(7);
        assert_eq!(res, vec![7, 7, 7, 7]);

        // applicative
        assert_eq!(Vec::pure(5), vec![5]);

        let f = |x| x + 3;
        let fs = vec![f, f, f, f];
        let res = vec.clone().apply(fs);
        assert_eq!(res, vec![4, 5, 6, 7]);

        // monad
        let f = |n| vec![0; n];
        let res = vec.clone().bind(f);
        let len = 1 + 2 + 3 + 4;
        assert_eq!(res, vec![0; len]);

        let double_vec = vec![vec.clone()];
        assert_eq!(Vec::join(double_vec), vec);
    }
}
