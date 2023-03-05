## Static dispatch

Support for enum dispatching.
1. `core` traits.
2. `std` traits.
3. `custom` traits.

First of all, there are some questions about what should be considered as `dispatchable`.

When we have `shapes` of arity one, we don't have any problems knowing what field should be dispatched.
But any other arity (even a nullary / unit) gives us some problem because there can be disputes about
which field should be considered for dispatchability.

Build a candidate list about this sound reasonable..

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
struct Berit;
impl Trait for Adam {}
impl Trait for Berit {}

// NOTE: When Penum tries to match a variant with a shape, it does so it a partial order,
//       left to right.
#[penum( 
    // Any shape that is of arity one (unary variants) won't have any problems 
    // getting added as a `dispachelor`. They are the only candidate we have..
    (T) | 
    { name: T}

    // # Here we have one concrete type and one generic type.
    //   Firstly, the only thing Penum knows right now is that `T` 
    //   implements `Trait`. It does not know that `Berit` also 
    //   implements `Trait`--because it hasn't been specified.
    //
    // # The only way Penum can know about `Berit` implementations is:
    //   1. If we have have an explicit bound telling us about it.
    //   2. If one of the variant matching `T` gets substituted by `Berit`. 
    //
    // # Selecting a `dispachelor` here should be easy as long as `T` != `Berit`,
    //   for all variants, we know that we should dispatch `T` over `Berit`.
    //   This is because we don't know anything about Berit as long as `T` 
    //   isn't being substituted by it. So we can rule out `Berit` as a `dispachelor`
    (Berit, T) | 

    // # This one is also a little tricky because we can end up getting
    //   (Berit, Berit), which would cause a dispute because both are valid 
    //   candidates. To be able to choose a `dispachelor`, one would have to 
    //   ask the arbitor.
    (_, T) | 

    // # This one is tricky, because I don't know how we should rule based on
    //   the position it has in the `shape` list and that variant would 
    //   (based on the partial order) match the above shape first.
    //   There is no way for us to know that
    //  
    (T, U) 
    
    where T: Trait, U: Trait
)]
enum Alpha {
    V1(Berit, Adam),
    V2(Adam, Adam),
}

let x = Alpha::V1(Adam);
x.run()

```