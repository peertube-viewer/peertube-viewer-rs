use std::{future::Future, rc::Rc};
use tokio::task::{spawn_local, JoinHandle};

type Loading<D, E> = JoinHandle<Result<(Vec<D>, Option<usize>), E>>;

pub struct PreloadableList<Data, Error, Next, Id> {
    loaded: Vec<Vec<Rc<Data>>>,
    loading: Option<Loading<Data, Error>>,
    fetch_next: Next,
    fetch_id: Id,

    current: usize,
    total: Option<usize>,
}

impl<Data, Error, Next, Id, NF, IF> PreloadableList<Data, Error, Next, Id>
where
    Data: 'static,
    Error: 'static,

    Next: Fn(usize) -> NF,
    NF: Future<Output = Result<(Vec<Data>, Option<usize>), Error>> + 'static,

    Id: Fn(Rc<Data>) -> IF,
    IF: Future<Output = ()> + 'static,
{
    pub fn new(fetch_next: Next, fetch_id: Id) -> PreloadableList<Data, Error, Next, Id> {
        PreloadableList {
            loaded: Vec::new(),
            loading: None,
            fetch_next,
            fetch_id,
            current: 0,
            total: None,
        }
    }

    pub async fn next(&mut self) -> Result<&[Rc<Data>], Error> {
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

    pub fn prev(&mut self) -> &Vec<Rc<Data>> {
        self.current -= 1;
        &self.loaded[self.current]
    }

    fn preload_next(&mut self) {
        if self.loaded.len() <= self.current + 1 && self.loading.is_none() {
            self.loading = Some(spawn_local((self.fetch_next)(self.current + 1)));
        }
    }

    fn preload_id(&self, id: usize) {
        let data_cloned = self.loaded[self.current][id].clone();
        spawn_local((self.fetch_id)(data_cloned));
    }

    fn current(&self) -> &[Rc<Data>] {
        &self.loaded[self.current]
    }

    fn current_len(&self) -> usize {
        self.current().len()
    }

    fn expected_total(&self) -> Option<usize> {
        self.total
    }
}
