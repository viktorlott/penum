
## Penums expressions

#### General thoughts
There are different ways of specifying a Penum expression. This might be confusing, but as for now I
kind of want it to be as flexible as possible to get a feel on which expression kind will be
prefered. But note that the different Penum expressions also has different use-cases. 

For example, expressions that requires you to use `^` when wanting to dispatch a trait, is there so
that if one would want to restrict a type with another trait wont cause Penum to also dispatch that
trait. In the future, It might even be possible to use the "extra" non-dispatched trait bounds to
decide how the dispatchable trait should be resolved. An expression like this could be possible in
the future: `_ where String: ^Trait + Default("random {self}")`

But it's kind of hard to know how one would interprete that expression. So that might be a bad
example. 

The point is that we might be able to use these extra trait bound to decide how a dispatched trait
should behave.
