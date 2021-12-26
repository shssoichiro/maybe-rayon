#![warn(clippy::all)]

cfg_if::cfg_if! {
  if #[cfg(any(not(feature = "threads"), all(target_arch="wasm32", not(target_feature = "atomics"))))] {
    pub struct ThreadPoolBuilder ();
    impl ThreadPoolBuilder {
      pub fn new() -> ThreadPoolBuilder {
        ThreadPoolBuilder()
      }
      pub fn build(self) -> Result<ThreadPool, ()> {
        Ok(ThreadPool())
      }
      pub fn num_threads(self, _num_threads: usize) -> ThreadPoolBuilder {
        ThreadPoolBuilder()
      }
    }
    #[derive(Debug)]
    pub struct ThreadPool ();
    impl ThreadPool {
      pub fn install<OP, R>(&self, op: OP) -> R where
            OP: FnOnce() -> R + Send,
                R: Send, {
        op()
      }
    }

    pub mod iter {
      pub trait IntoParallelIterator {
          type Iter: Iterator<Item = Self::Item>;
          type Item: Send;

          fn into_par_iter(self) -> Self::Iter;
      }

      impl<I: IntoIterator> IntoParallelIterator for I where
        I::Item : Send {
        type Item = I::Item;
        type Iter = I::IntoIter;

        fn into_par_iter(self) -> I::IntoIter {
          self.into_iter()
        }
      }

      pub trait IntoParallelRefMutIterator<'data> {
          type Iter: IntoParallelIterator<Item = Self::Item>;
          type Item: Send + 'data;

          fn par_iter_mut(&'data mut self) -> Self::Iter;
      }

      impl<'data, I: 'data + ?Sized> IntoParallelRefMutIterator<'data> for I
      where
          &'data mut I: IntoParallelIterator,
      {
          type Iter = <&'data mut I as IntoParallelIterator>::Iter;
          type Item = <&'data mut I as IntoParallelIterator>::Item;

          fn par_iter_mut(&'data mut self) -> Self::Iter {
              self.into_par_iter()
          }
      }

      pub trait ParallelIterator: Iterator {
        fn flat_map_iter<U, F>(self, f: F) -> std::iter::FlatMap<Self, U, F>
        where
          Self: Sized,
          U: IntoIterator,
          F: FnMut(<Self as Iterator>::Item) -> U,
        {
          self.flat_map(f)
        }
      }

      impl<I: Iterator> ParallelIterator for I {}
    }
    pub mod slice {
      pub trait ParallelSlice<T: Sync> {
        fn par_chunks_exact(
          &self, chunk_size: usize,
        ) -> std::slice::ChunksExact<'_, T>;
      }

      impl<T: Sync> ParallelSlice<T> for [T] {
        #[inline]
        fn par_chunks_exact(
          &self, chunk_size: usize,
        ) -> std::slice::ChunksExact<'_, T> {
          self.chunks_exact(chunk_size)
        }
      }
    }

    pub mod prelude {
      pub use super::iter::*;
      pub use super::slice::*;
    }

    pub fn join<A, B, RA, RB>(oper_a: A, oper_b: B) -> (RA, RB)
    where
      A: FnOnce() -> RA + Send,
      B: FnOnce() -> RB + Send,
      RA: Send,
      RB: Send {
      (oper_a(), oper_b())
    }

    use std::marker::PhantomData;

    pub struct Scope<'scope>{
      marker: PhantomData<Box<dyn FnOnce(&Scope<'scope>) + Send + Sync + 'scope>>,
    }

    impl<'scope> Scope<'scope> {
      pub fn spawn<BODY>(&self, body: BODY)
        where BODY: FnOnce(&Scope<'scope>) + Send + 'scope
      {
        body(self)
      }
    }

    pub fn scope<'scope, OP, R>(op: OP) -> R
      where OP: for<'s> FnOnce(&'s Scope<'scope>) -> R + 'scope + Send, R: Send,
    {
      op(&Scope { marker: PhantomData})
    }
  } else {
    pub use rayon::*;
  }
}
