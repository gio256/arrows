pub trait CatFamily {
    type M<T, U>: Category<T, U, CFamily = Self>;

    fn id<A>() -> Self::M<A, A>;
}

pub trait Category<A, B> {
    type CFamily: CatFamily<M<A, B> = Self>;

    // (>>>) :: Category cat => cat a b -> cat b c -> cat a c
    fn then<C>(
        self,
        consumer: <Self::CFamily as CatFamily>::M<B, C>,
    ) -> <Self::CFamily as CatFamily>::M<A, C>
    where
        A: 'static,
        B: 'static,
        C: 'static;

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

pub trait ArrowFamily {
    type M<T, U>: Arrow<T, U, AFamily = Self>;
}

pub trait Arrow<A, B>: Category<A, B> {
    type AFamily: ArrowFamily<M<A, B> = Self>;

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

    // (***) :: a b c -> a b' c' -> a (b,b') (c,c')
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

    fn fan<B1>(
        self,
        g: <Self::AFamily as ArrowFamily>::M<A, B1>,
    ) -> <Self::AFamily as ArrowFamily>::M<A, (B, B1)>
    where
        Self: Sized,
        A: Clone + 'static,
        B: 'static,
        B1: 'static;
}
