Functor, Applicative, Monad, [Arrow](https://en.m.wikibooks.org/wiki/Haskell/Understanding_arrows), and ArrowChoice in rust.

This uses GATs and the type families approach explored by nikomatsakis (see [here](https://smallcultfollowing.com/babysteps/blog/2016/11/03/associated-type-constructors-part-2-family-traits/)) and RustyYato ([here](https://github.com/RustyYato/type-families)).

I expected arrows to be particularly ugly, but I was pleasantly surprised by the syntax and the quality of type inference.

Here is the default definition of the `***` operator from haskell's Control.Arrow:

```haskell
-- | Split the input between the two argument arrows and combine
--   their output.  Note that this is in general not a functor.
--
--   The default definition may be overridden with a more efficient
--   version if desired.
(***) :: a b c -> a b' c' -> a (b,b') (c,c')
f *** g = first f >>> arr swap >>> first g >>> arr swap
  where swap ~(x,y) = (y,x)
```

In rust:

```rust
self.fst()
    .then_pure(flip)
    .then(right_arrow.fst())
    .then_pure(flip)
```

For comparison, there is a partial implementation of the haskell arrow [tutorial](https://en.m.wikibooks.org/wiki/Haskell/Arrow_tutorial) in [impls/circuit.rs](src/impls/circuit.rs).

Please don't hesitate to open an issue if you see somewhere the traits or implementations are inconsistent with either the theory or the way things actually work in other languages.
