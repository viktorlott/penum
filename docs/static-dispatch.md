## Static dispatch

Support for enum dispatching.
1. `core` traits.
2. `std` traits. 
3. `custom` traits.

Terminology:
- Candidate    - is a contender that have the potential of being selected for dispatch.
- Dispatchelor - is the candidate that has been selected for dispach.
- Arbitor      - is the one who has the final decision in the selecting process.

First of all, there are some questions about what can be considered dispatchable.

When we have `group` of arity one, we don't have any problems knowing what field is dispatchable, because
there is only one possible candidate. But any other arity, even a nullary, can give us some problem because there 
can be disputes about which field should be considered for dispatch.

First lets consider naming some of the problems.
- Twin problem - They are literally the same.
- Duck problem - When they differ structurally/nominally, but behaves similarly/identically.

### Building a candidate list of potential `dispatchelors`.
Firsly, we should do the naive approach and just add all whom fit the behavior (has implemented the trait).
After that, we need to start deciding which of them should take precedence, because there will be disputes 
about which field is more suitable to dispatch. 

To be able to dispatch a variant containing multiple fields--implementing the same trait (i.e. different but same),
we need some kind of `arbitor` that does this for us.
1. User bias               - like if they have explicitly written, "I want THIS to be dispatched!".
2. Candidate occurance     - how often does a candidate occur in other variants.
3. Variant arity           - if we have a duck problem, we select based on previous unary variant `dispatchelors`.
4. First come First served - we pick the first one in the list.


```rust
trait Trait { fn run(&self) { println!("hello") } }
struct Adam;
struct Eva;
impl Trait for Adam {}
impl Trait for Eva {}

// NOTE: When Penum tries to match a variant with a pattern fragment, it does so by partial order.
#[penum( 
    // # Any pattern fragment that is of arity one (unary variants) won't have any problems 
    //   getting added as a `dispatchelor`. They are the only candidate we have..
    (T) | { name: T } |

    // # Here we have a tuple containing one concrete type and one generic param.
    //   Firstly, the only thing Penum knows right now is that `T` 
    //   implements `Trait`. It does not know that `Eva` also 
    //   implements `Trait`--because it hasn't been specified.
    //
    // # The only way Penum can know about `Eva` implementations is:
    //   1. If we have have an explicit bound telling us about it.
    //   2. If one of the variant matching `T` gets substituted by `Eva`. 
    //
    // # Selecting a `dispatchelor` here should be easy as long as `T` != `Eva`,
    //   for all variants, we know that we should dispatch `T` over `Eva`.
    //   This is because we don't know anything about Eva as long as `T` 
    //   isn't being substituted by it. So we can rule out `Eva` as a `dispatchelor`
    (Eva, T) | 

    // # Here we have a tuple containing a `placeholder` and a `generic` param.
    //   This one is also a little tricky because we can end up getting
    //   (Eva, Eva), which would cause a dispute because both are valid 
    //   candidates. To be able to choose a `dispatchelor`, one would have to 
    //   ask the arbitor.
    (_, T) | 

    // # Here we have a tuple containing two generic params.
    //   This one is tricky, because I don't know if this pattern fragment even can be matched given 
    //   the previous pattern fragment. This is because Penum has a partial match order, and because
    //   `_` is a wildcard, it will catch all variants that also will match this one.
    (T, U) |

    // # Here we have a struct containing two generic params.
    //   This pattern fragment should not have the same problem as the pattern fragment above. Here we actually 
    //   know that these two fields will be in a dispute, before any variants are present.
    //   To be able to choose a `dispatchelor`, one would have to ask the arbitor.
    { a: T, b: U }

    where 
        T: Trait, 
        U: Trait
)]
enum dispatched {
   V1(Eva, Adam),
   V2(Adam, Adam),
}
```

### Introduce new syntax 

Introducing a symbol marker that is used to mark traits as dispachable.
I'm thinking about `^` being that symbol. But I don't know if it should be inside a pattern fragment
or where clause. It would make more sense to have it in the where clause for a trait than 
for a generic param in a pattern fragment. This is because a generic param could have multiple traits 
and then it would be difficult to know which one should be dispatched..


1. This feels weird because we would have to declare it for every pattern fragment, making it tedius because we would have to then also mark `(T)` as dispatchable. 
   ```rust
   #[penum( (T) | (^T, U) where  T:  Trait + Tiart, U: Trait )]
   ```

2. This makes more sense because we are saying that all `T`s are dispatchable. But it's still a little hard to understand which trait is being dispatched. This could actually be a short hand for: All `T`s traits should be dispatchable.
   ```rust
   #[penum( (T) | (T,  U) where ^T:  Trait + Tiart, U: Trait )]
   ```

3. This seems like the more natural choice because we are very selective about what trait should be dispatchable; and even if another generic also has the same trait bound, it won't be considered dispatchable because it's not marked with `^`. 
   ```rust
   #[penum( (T) | (T,  U) where  T: ^Trait + Tiart, U: Trait )]
   ```
   Another thing is that for something to be dispachable, all variants must include the generic with the marked trait?

   It should also be possible to use `impl ^Trait` to become a dispatchable candidate.
   Knowing that this syntax makes it a little confusing towards unmatched variants. e.g.



```rust
struct Random;

#[penum( 
   // # Given this impl trait type and the variants:
   //   - Sadly, because of the "dum" polymorphic builder, V2's first argument `Random` will be expected to implement AsRef.
   //     But we already know that it does not, if we had core/std trait knowledge.
   //     - i.e it would be possible to infer that V2s second argument could be a valid dispatcher because of V1s first argument.
   //       But this might lead to some sort of unsoundness given that the order and position should matter.
   //       How it should be implemented then is by a where clause.
   // 
   //       e.g. (..) where String: ^AsRef<str>
   //       
   //       This is so much cleaner. 
   (impl ^AsRef<str>, ..) | (..)
)]
enum dispatched {
   // First argument matches our impl trait bound, and because it's prefixed with `^`, we give mark variant with a dispatch arm.
   V1(String, Random),

   // Now, because 
   V2(Random, String),

   V3(Random, String, String),
}
```


### Dispatch semantics

One of the problems with implementing some sort of derived dispatch solution for enums is that most of the time the variants aren't even dispatchable.
When a variant isn't dispatchable, we often have to look at the trait method signature to be able to figure out what we can do.

Without knowing any specifics about the overall program (e.g knowing that we have implemented something for a type in another module), so that the only option for us is to deduce the meaning through semantically understanding the penum expression and the subject (that being the enum itself). 
- The type and its position in the pattern.
  - So for each variant that matches a certain pattern where one or more of the trait bound members are marked for dispatch, we build a candidate list
  where each candidate contains information about what variant it matched, what position it matched on, and what type it matched on.
  Given this list, we can better understand how we would solve problems like: 
  a. Variants without a dispatch match (e.g the might have matched for another pattern that didn't contain a dispatch member).
  b. Variants that match but where the type differs from previous matches.
- The trait signature.
  - This gives us information about how the actual method we are dispaching should handle variants that can't be dispatched.
    - Method signature without a return type can be handled by just matching the variant with a Unit type.
    - Method signature with a return type is handled differently depending on the return type implementations.
      - Return type implements Default and the value is owned:
        - Then we just match with `Default::default()`
      - Return type doesn't implements Default and the value is owned:
        - Then we might do a Option<T> wrap and return None

      - Return type implements Default and but it's a reference:
        - Can the type be const evaluated in a static declaration, then we of course do that.
        - If it cannot be const evaluated in a static declaration, then we might need to use `LazyCell`.

      - Return type doesn't implement Default -> Then we handle it by the use of semantic analyzing
        - If it's a core/std lib type, could we somehow know how to handle it?
        - If we don't know anything about the type, should we 
          a. Option<T> wrap it for the user?
          b. Add panic handlers?
          c. Try to 


   It should also be possible to use `impl ^Trait` to become a dispatchable candidate.
   Knowing that this syntax makes it a little confusing towards unmatched variants. e.g.



```rust
// Question is, should we always assume that a dispatch candidate always implements Default, or should we be required to specify it?.
// Feels like it should be specified.
// If we don't specify it, should we then just Option wrap it?
#[penum( (T, U) where T: ^AsRef<str> + Default )]
enum Disp {
    V1(String, i32),
    V2
}
impl AsRef<str> for Disp where String: Default {
    fn as_ref(&self) -> &str {
        match Self {
            Self::V1(arg1, ..) => AsRef::<str>::as_ref(self),
            Self::V2 => Default::default()
        }
    }
}
// -----------------------------------------------
// This won't work..
#[penum( (T, ..) where T: ^AsRef<str> )]
enum Disp {
    V1(String, i32),
    V2,
}
impl<'a> AsRef<Option<&'a str>> for Disp
where
    Self: 'a,
    String: Default,
{
    fn as_ref(&self) -> &Option<&str> {
        match self {
            Self::V1(arg1, ..) => &Some(AsRef::as_ref(arg1)),
            Self::V2 => &None,
        }
    }
}
```

