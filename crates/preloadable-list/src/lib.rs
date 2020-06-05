use std::{future::Future, rc::Rc};
use tokio::task::{spawn_local, JoinHandle};

type Loading<D, E> = JoinHandle<Result<(Vec<D>, Option<usize>), E>>;

pub struct PreloadableList<D, E, N, F>
where
    N: Fn(usize) -> F,
    D: 'static,
    E: 'static,
    F: 'static,
    F: Future<Output = Result<(Vec<D>, Option<usize>), E>>,
{
    loaded: Vec<Vec<Rc<D>>>,
    loading: Option<Loading<D, E>>,
    fetch_next: N,

    current: usize,
    total: Option<usize>,
}

impl<D, E, N, F> PreloadableList<D, E, N, F>
where
    N: Fn(usize) -> F,
    D: 'static,
    E: 'static,
    F: 'static,
    F: Future<Output = Result<(Vec<D>, Option<usize>), E>>,
{
    pub fn new(next: N) -> PreloadableList<D, E, N, F> {
        PreloadableList {
            loaded: Vec::new(),
            loading: None,
            fetch_next: next,
            current: 0,
            total: None,
        }
    }

    pub async fn next(&mut self) -> Result<&[Rc<D>], E> {
        if !self.loaded.is_empty() {
            self.current += 1;
        }
        if self.loaded.len() <= self.current {
            let temp;
            if let Some(handle) = self.loading.take() {
                temp = handle.await.unwrap()?;
            } else {
                temp = (self.fetch_next)(self.current).await?;
            }
            let (data, new_total) = temp;
            self.loaded.push(data.into_iter().map(Rc::new).collect());
            self.total = new_total.or(self.total);
        }
        Ok(&self.loaded[self.current])
    }

    pub fn prev(&mut self) -> &Vec<Rc<D>> {
        self.current -= 1;
        &self.loaded[self.current]
    }

    fn preload_next(&mut self) {
        if self.loaded.len() <= self.current + 1 && self.loading.is_none() {
            self.loading = Some(spawn_local((self.fetch_next)(self.current + 1)));
        }
    }

    fn current(&self) -> &Vec<Rc<D>> {
        &self.loaded[self.current]
    }

    fn current_len(&self) -> usize {
        self.current().len()
    }

    fn expected_total(&self) -> Option<usize> {
        self.total
    }
}
