`arrows &&& rust`
================

Functor, Applicative, Monad, [Arrow](https://en.m.wikibooks.org/wiki/Haskell/Understanding_arrows), and ArrowChoice in rust.

This leverages GATs and the type families approach explored by nikomatsakis (see [here](https://smallcultfollowing.com/babysteps/blog/2016/11/03/associated-type-constructors-part-2-family-traits/)) and RustyYato ([here](https://github.com/RustyYato/type-families)).

#### Arrows

I expected arrows in rust to be particularly ugly, but I was pleasantly surprised by the syntax and the quality of type inference provided by GATs.

Here is the default definition of the `***` operator from haskell's Control.Arrow:

```haskell
-- | Split the input between the two argument arrows and combine
--   their output.  Note that this is in general not a functor.
(***) :: a b c -> a b' c' -> a (b,b') (c,c')
f *** g = first f >>> arr swap >>> first g >>> arr swap
  where swap ~(x,y) = (y,x)
```

In rust:

```rust
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
    self.fst()
        .then_pure(flip)
        .then(right_arrow.fst())
        .then_pure(flip)
}
```

A snippet from the haskell arrow tutorial using proc notation:

```haskell
getWord :: StdGen -> Circuit () String
getWord rng = proc () -> do
    -- If this is the first game loop, run pickWord. mPicked becomes Just <word>.
    -- On subsequent loops, mPicked is Nothing.
    firstTime <- oneShot -< ()
    mPicked <- if firstTime
        then do
            picked <- pickWord rng -< ()
            returnA -< Just picked
        else returnA -< Nothing
    -- An accumulator that retains the last 'Just' value.
    mWord <- accum' Nothing mplus -< mPicked
    returnA -< fromJust mWord
```

The equivalent in rust:

```rust
let get_word = oneshot()
    .then_pure(Either::from)
    .then(arrow(|_| None).owise(pick_word.then_pure(Some)))
    .then(Circuit::accum_dup(None, Option::or));
```

For further comparison, there is a partial implementation of the haskell arrow [tutorial](https://en.m.wikibooks.org/wiki/Haskell/Arrow_tutorial) in [impls/circuit.rs](src/impls/circuit.rs).

Please don't hesitate to open an issue if you see somewhere the traits or implementations are inconsistent with either the theory or the way things actually work in other languages.
