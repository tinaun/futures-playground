# Compat

While async/await syntax is a really powerful feature, the existing higher level libraries 
like `tokio` and `hyper` built upon futures 0.1 cannot work with async syntax without being
ported over to the futures 0.3 core type defintions first. until then, the futures crate 
offers the `Compat` struct, which wraps futures and streams to make them compatable with both 
major versions of the future crate.


## Comparasion
make a cool table here

futures 0.1 Item = T Error = E <=> Futures 0.3 Output = Result<T, E>

## Wrapping Futures 0.1 in `async fn`

`Future01CompatExt`
`Stream01CompatExt`

## Running Futures 0.3 on an Futures 0.1 Executor

* dealing with unpin (`.boxed()` combinator)
* compat combinator defined on `TryFutureExt` requires a handle to an executor
* `Executor01CompatExt`

## Putting it all together

* runable example showing hyper running through an async fn