
## Penums expressions

### What does Penum do?
It tries to automate the tedious task of having to write implementations
for each trait that is to be statically dispatched, and impose rules 
that describes the shape of which the enum must conform.


### Background
Automatically creating implementations as if they were written by hand
requires us of having some knowledge about the enum and trait. For
instance, to even be able to create an impl, we need to know which enum
variant fields and methods are eligable for dispatch. So we cannot
simple just know idents, we also need to know their definitions. But
even with that, we'll still have some interpretation problems. Take for
example the issue of unsupported variants, in other words, variants that
does not contain fields that are eligable for dispatch. How should they
be handled? The simplest solution would be to just resolving them with a
panic, which sounds totally reasonable given that they are not
supported. But what if the method we're dispatching returns a `Result`
type, wouldn't it make sense to instead return `Err(_)` instead of a
panic?

#### General thoughts
There are different ways of specifying a Penum expression. This might be
confusing, but as for now I kind of want it to be as flexible as
possible to get a feel on which expression kind will be prefered. But
note that the different Penum expressions also has different use-cases. 

For example, expressions that requires you to use `^` when wanting to
dispatch a trait, is there so that if one would want to restrict a type
with another trait wont cause Penum to also dispatch that trait. In the
future, It might even be possible to use the "extra" non-dispatched
trait bounds to decide how the dispatchable trait should be resolved. An
expression like this could be possible in the future: `_ where String:
^Trait + Default("random {self}")`

But it's kind of hard to know how one would interprete that expression.
So that might be a bad example. 

The point is that we might be able to use these extra trait bound to
decide how a dispatched trait should behave.

