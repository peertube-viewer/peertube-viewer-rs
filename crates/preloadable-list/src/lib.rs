use std::{future::Future, pin::Pin, rc::Rc};
use tokio::task::{spawn_local, JoinHandle};

type Loading<D, E> = JoinHandle<Result<(Vec<D>, Option<usize>), E>>;

pub struct PreloadableList<L: AsyncLoader> {
    loaded: Vec<Vec<Rc<L::Data>>>,
    loading: Option<Loading<L::Data, L::Error>>,

    loader: L,
    current: usize,
    total: Option<usize>,
}

impl<L, D, E> PreloadableList<L>
where
    L: AsyncLoader<Data = D, Error = E>,
    D: 'static,
    E: 'static,
{
    pub fn new(loader: L) -> PreloadableList<L> {
        PreloadableList {
            loaded: Vec::new(),
            loading: None,
            loader,
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
                temp = self.loader.data(self.current).await?;
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

    pub fn preload_next(&mut self) {
        if self.loaded.len() <= self.current + 1 && self.loading.is_none() {
            self.loading = Some(spawn_local(self.loader.data(self.current + 1)));
        }
    }

    pub fn preload_id(&self, id: usize) {
        let data_cloned = self.loaded[self.current][id].clone();
        self.loader.item(data_cloned);
    }

    pub fn current(&self) -> &[Rc<D>] {
        &self.loaded[self.current]
    }

    pub fn current_len(&self) -> usize {
        self.current().len()
    }

    pub fn expected_total(&self) -> Option<usize> {
        self.total
    }
}

pub trait AsyncLoader {
    type Data: 'static;
    type Error: 'static;

    // Workaround async_trait not allowing requirement for the returned futures to be async
    fn data(
        &mut self,
        current: usize,
    ) -> Pin<
        Box<dyn 'static + Future<Output = Result<(Vec<Self::Data>, Option<usize>), Self::Error>>>,
    >;
    fn item(&self, _: Rc<Self::Data>) -> Pin<Box<dyn 'static + Future<Output = ()>>> {
        Box::pin(async {})
    }
}
