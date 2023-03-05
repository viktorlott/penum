## Static dispatch

Support for enum dispatching.
1. `core` traits.
2. `std` traits.
3. `custom` traits.

First of all, there are some questions about what should be considered as `dispatchable`.

When we have `shapes` of arity one, we don't have any problems knowing what field should be dispatched.
But any other arity (even a nullary / unit) gives us some problem because there can be disputes about
which field should be considered for dispatchability.

First lets consider naming some of the problems.
- Twin problem - They are literally the same.
- Duck problem - When they differ structurally/nominally, but behaves similarly/identically.

### Building a candidate list of potential `dispachelors`.
Firsly, we should do the naive approach and just add all whom fit the behavior (has implemented the trait).
After that, we need to start deciding which of them should take precedence, because there will be disputes 
about which field is more suitable to dispatch. 

To be able to dispatch a variant containing multiple fields--implementing the same trait (i.e. different but same),
we need some kind of `arbitor` that does this for us.
1. User bias               - like if they have explicitly written, "I want THIS to be dispached!".
2. Candidate occurance     - how often does a candidate occur in other variants.
3. Variant arity           - if we have a duck problem, we select based on previous unary variant `dispachelors`.
4. First come First served - we pick the first one in the list.


```rust
trait Trait { fn run(&self) { println!("hello") } }
struct Adam;
struct Eva;
impl Trait for Adam {}
impl Trait for Eva {}

// NOTE: When Penum tries to match a variant with a shape, it does so by partial order, left->right.
#[penum( 
    // # Any shape that is of arity one (unary variants) won't have any problems 
    //   getting added as a `dispachelor`. They are the only candidate we have..
    (T) | { name: T } |

    // # Here we have one concrete type and one generic type.
    //   Firstly, the only thing Penum knows right now is that `T` 
    //   implements `Trait`. It does not know that `Eva` also 
    //   implements `Trait`--because it hasn't been specified.
    //
    // # The only way Penum can know about `Eva` implementations is:
    //   1. If we have have an explicit bound telling us about it.
    //   2. If one of the variant matching `T` gets substituted by `Eva`. 
    //
    // # Selecting a `dispachelor` here should be easy as long as `T` != `Eva`,
    //   for all variants, we know that we should dispatch `T` over `Eva`.
    //   This is because we don't know anything about Eva as long as `T` 
    //   isn't being substituted by it. So we can rule out `Eva` as a `dispachelor`
    (Eva, T) | 

    // # This one is also a little tricky because we can end up getting
    //   (Eva, Eva), which would cause a dispute because both are valid 
    //   candidates. To be able to choose a `dispachelor`, one would have to 
    //   ask the arbitor.
    (_, T) | 

    // # This one is tricky, because I don't know if this shape even can be match, given 
    //   the previous shapes. This is because Penum has a partial match order, and because
    //   `_` is a wildcard, it will catch all variants that also will match this one.
    (T, U) |

    // # This shape should not have the same problem as the shape above. Here we actually 
    //   know that these two fields will be in a dispute, before any variants are present.
    //   To be able to choose a `dispachelor`, one would have to ask the arbitor.
    { a: T, b: U }

    where 
        T: Trait, 
        U: Trait
)]
enum Dispached {
    V1(Eva, Adam),
    V2(Adam, Adam),
}
```


#### Introduce new syntax 

Might want to introduce a symbol marker that indicates if something should be dispachable.
I'm thinking about `^` being that symbol. But I don't know if it should be inside a shape
or where clause. It would make more sense to have it in the where clause for a trait than 
for a generic type. This is because a generic type could have multiple traits and then it
would be difficult to know which one should be dispached..


```rust
// # This feels weird because we only declare it for a shape, making it harder to understand
//   if `(T)` also it dispachable. 
#[penum( (T) | (^T, U) where  T:  Trait + Tiart, U: Trait )]

// # This makes more sense because we are saying that all `T`s are dispachable. But it's still 
//   a little hard to understand which trait is being dispatched. This could actually be a short
//   hand for: All `T`s traits should be dispachable.
#[penum( (T) | (T,  U) where ^T:  Trait + Tiart, U: Trait )]

// # This seems like the more natural choice because we are very selective about what trait should
//   be dispachable; and even if another generic also has the same trait bound, it won't be considered 
//   dispachable because it's not marked with `^`.
#[penum( (T) | (T,  U) where  T: ^Trait + Tiart, U: Trait )]
```


I don't know if this is even worth focusing on right now, but this might be 
interesting to think of in the future.
```rust
#[penum {
    bound = T: Trait, U: Trait;
    shape = (T) | (T, _) | (_, U) | { ident: T };
}]
```