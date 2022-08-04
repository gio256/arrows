use crate::monad::*;

/// The Either monad behaves as you would expect Result to behave, except
/// `Left(e)` represents errors by convention.
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub struct EitherFamily<L>(std::marker::PhantomData<L>);

impl<L> FunctorFamily for EitherFamily<L> {
    type M<T> = Either<L, T>;
}

impl<L, R> Functor<R> for Either<L, R> {
    type FFamily = EitherFamily<L>;

    fn fmap<F, B>(self, f: F) -> Either<L, B>
    where
        F: Fn(R) -> B,
    {
        match self {
            Self::Right(r) => Either::Right(f(r)),
            Self::Left(l) => Either::Left(l),
        }
    }
}

impl<L> ApplicativeFamily for EitherFamily<L> {
    type M<T> = Either<L, T>;
}

impl<L, R> Applicative<R> for Either<L, R> {
    type AFamily = EitherFamily<L>;

    fn pure(value: R) -> Self {
        Self::Right(value)
    }

    fn lift_a2<F, B, C>(self, other: Either<L, B>, f: F) -> Either<L, C>
    where
        F: Fn(R, B) -> C,
    {
        match (self, other) {
            (Self::Right(r_l), Either::Right(r_r)) => Either::Right(f(r_l, r_r)),
            (Self::Left(left), _) | (_, Either::Left(left)) => Either::Left(left),
        }
    }
}

impl<L> MonadFamily for EitherFamily<L> {
    type M<T> = Either<L, T>;
}

impl<L, R> Monad<R> for Either<L, R> {
    type MFamily = EitherFamily<L>;

    fn bind<F, B>(self, f: F) -> Either<L, B>
    where
        F: Fn(R) -> Either<L, B>,
    {
        match self {
            Self::Right(right) => f(right),
            Self::Left(left) => Either::Left(left),
        }
    }
}

impl<L, R> Either<L, R> {
    pub fn flip(self) -> Either<R, L> {
        match self {
            Either::Left(left) => Either::Right(left),
            Either::Right(right) => Either::Left(right),
        }
    }
    pub fn ok(self) -> Option<R> {
        match self {
            Self::Right(right) => Some(right),
            _ => None,
        }
    }
}

impl<A> From<(bool, A)> for Either<A, A> {
    fn from((b, value): (bool, A)) -> Self {
        if b {
            Self::Right(value)
        } else {
            Self::Left(value)
        }
    }
}

impl From<bool> for Either<(), ()> {
    fn from(b: bool) -> Self {
        if b {
            Self::Right(())
        } else {
            Self::Left(())
        }
    }
}
