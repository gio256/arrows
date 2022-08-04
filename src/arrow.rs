use crate::impls::either::Either;

pub trait CatFamily {
    type M<T, U>: Category<T, U, CFamily = Self>;

    fn id<A>() -> Self::M<A, A>;
}

pub trait Category<A, B> {
    type CFamily: CatFamily<M<A, B> = Self>;

    // right-to-left composition
    // (>>>) :: Category cat => cat a b -> cat b c -> cat a c
    fn then<C>(
        self,
        consumer: <Self::CFamily as CatFamily>::M<B, C>,
    ) -> <Self::CFamily as CatFamily>::M<A, C>
    where
        A: 'static,
        B: 'static,
        C: 'static;

    // left-to-right composition
    // (.) or (<<<) :: Category cat => cat b c -> cat a b -> cat a c
    fn after<A0>(
        self,
        producer: <Self::CFamily as CatFamily>::M<A0, A>,
    ) -> <Self::CFamily as CatFamily>::M<A0, B>
    where
        Self: Sized,
        A: 'static,
        B: 'static,
        A0: 'static,
    {
        Category::then(producer, self)
    }
}

pub trait ArrowFamily: CatFamily {
    type M<T, U>: Arrow<T, U, AFamily = Self> + Category<T, U, CFamily = Self>;
}

pub trait Arrow<A, B>: Category<A, B> {
    type AFamily: ArrowFamily<M<A, B> = Self> + CatFamily<M<A, B> = Self>;

    // arr :: (a -> b) -> m a b
    fn arrow<F>(f: F) -> Self
    where
        F: Fn(A) -> B + Clone + 'static;

    // first :: m a b -> m (a, c) (b, c)
    fn fst<C>(self) -> <Self::AFamily as ArrowFamily>::M<(A, C), (B, C)>
    where
        A: 'static,
        B: 'static;

    // second :: m a b -> m (c,a) (c,b)
    fn snd<C>(self) -> <Self::AFamily as ArrowFamily>::M<(C, A), (C, B)>
    where
        Self: Sized,
        A: 'static,
        B: 'static,
        C: 'static;

    // (***) :: m b c -> m b' c' -> m (b,b') (c,c')
    fn both<A1, B1>(
        self,
        g: <Self::AFamily as ArrowFamily>::M<A1, B1>,
    ) -> <Self::AFamily as ArrowFamily>::M<(A, A1), (B, B1)>
    where
        Self: Sized,
        A: 'static,
        B: 'static,
        A1: 'static,
        B1: 'static;
    // The associated type bound says ~ that forall T, U | M<T, U>: Arrow<T, U>
    // But what we need here is a proof that forall T, U, T', U' | M<T, U>: Arrow<T', U'>.
    // Removing the default impl allows this bound to be removed.
    // <Self::AFamily as ArrowFamily>::M<(A, A1), (B, A1)>: Arrow<(B, A1), (A1, B)>,
    // {
    // let swap0 = Arrow::arrow(|(b, a1): (B, A1)| (a1, b));
    // let swap1 = Arrow::arrow(|(b1, b): (B1, B)| (b, b1));
    // self.fst::<A1>().then(swap0).then(g.fst()).then(swap1)
    // }

    // Also called fanout, because of it's relationship to |||/fanin/owise
    // (&&&) :: m a b -> m a b' -> m a (b,b')
    fn dup<B1>(
        self,
        g: <Self::AFamily as ArrowFamily>::M<A, B1>,
    ) -> <Self::AFamily as ArrowFamily>::M<A, (B, B1)>
    where
        Self: Sized,
        A: Clone + 'static,
        B: 'static,
        B1: 'static;

    // precomposition with a pure function
    // (^>>) :: Arrow m => (b -> c) -> m c d -> m b d
    fn after_pure<F, A0>(self, f: F) -> <Self::AFamily as ArrowFamily>::M<A0, B>
    where
        F: Fn(A0) -> A + Clone + 'static,
        A: 'static,
        B: 'static,
        A0: 'static;

    // postcomposition with a pure function
    // (>>^) :: Arrow m => m b c -> (c -> d) -> m b d
    fn then_pure<F, C>(self, f: F) -> <Self::AFamily as ArrowFamily>::M<A, C>
    where
        F: Fn(B) -> C + Clone + 'static,
        A: 'static,
        B: 'static,
        C: 'static;
}

pub trait ChoiceFamily {
    type M<T, U>: ArrowChoice<T, U, AFamily = Self> + Category<T, U, CFamily = Self>;
}

pub trait ArrowChoice<A, B>: Arrow<A, B> {
    type AcFamily: ChoiceFamily<M<A, B> = Self>;

    // left :: m b c -> m (Either b d) (Either c d)
    fn left<D>(self) -> <Self::AcFamily as ChoiceFamily>::M<Either<A, D>, Either<B, D>>
    where
        A: 'static,
        B: 'static;

    // right :: a b c -> a (Either d b) (Either d c)
    fn right<D>(self) -> <Self::AcFamily as ChoiceFamily>::M<Either<D, A>, Either<D, B>>
    where
        A: 'static,
        B: 'static,
        D: 'static;

    // (+++) :: m b c -> m b' c' -> m (Either b b') (Either c c')
    fn split<A1, B1>(
        self,
        g: <Self::AcFamily as ChoiceFamily>::M<A1, B1>,
    ) -> <Self::AcFamily as ChoiceFamily>::M<Either<A, A1>, Either<B, B1>>
    where
        A: 'static,
        B: 'static,
        A1: 'static,
        B1: 'static;

    // Also called fanin, because of it's relationship with &&&/fanout/dup
    // (|||) :: m a b -> m c b -> m (Either a c) b
    fn owise<C>(
        self,
        g: <Self::AcFamily as ChoiceFamily>::M<C, B>,
    ) -> <Self::AcFamily as ChoiceFamily>::M<Either<A, C>, B>
    where
        A: 'static,
        B: 'static,
        C: 'static;
}
