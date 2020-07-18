use std::{future::Future, pin::Pin, rc::Rc};
use tokio::task::{spawn_local, JoinHandle};

type Loading<D, E> = JoinHandle<Result<(Vec<D>, Option<usize>), E>>;

pub struct PreloadableList<L: AsyncLoader> {
    loaded: Vec<Vec<Rc<L::Data>>>,
    loading: Option<Loading<L::Data, L::Error>>,

    loader: L,
    current: usize,
    offset: usize,
    step: usize,
    total: Option<usize>,
}

impl<L, D, E> PreloadableList<L>
where
    L: AsyncLoader<Data = D, Error = E>,
    D: 'static,
    E: 'static,
{
    pub fn new(loader: L, step: usize) -> PreloadableList<L> {
        PreloadableList {
            loaded: Vec::new(),
            loading: None,
            loader,
            current: 0,
            offset: 0,
            step,
            total: None,
        }
    }

    pub async fn ensure_init(&mut self) -> Result<(), E> {
        if self.loaded.is_empty() {
            self.next().await.map(|_| ())
        } else {
            Ok(())
        }
    }

    pub async fn next(&mut self) -> Result<&[Rc<D>], E> {
        if !self.loaded.is_empty() {
            self.offset += self.loaded[self.current].len();
            self.current += 1;
        }
        if self.loaded.len() <= self.current {
            let temp;
            if let Some(handle) = self.loading.take() {
                temp = handle.await.unwrap()?;
            } else {
                temp = self.loader.data(self.step, self.offset).await?;
            }
            let (data, new_total) = temp;
            self.loaded.push(data.into_iter().map(Rc::new).collect());
            self.total = new_total.or(self.total);
        }
        Ok(&self.loaded[self.current])
    }

    pub fn prev(&mut self) -> &Vec<Rc<D>> {
        if self.current >= 1 {
            self.current -= 1;
            self.offset -= self.loaded[self.current].len();
        }
        &self.loaded[self.current]
    }

    pub fn preload_next(&mut self) {
        if self.loaded.len() <= self.current + 1 && self.loading.is_none() {
            self.loading = Some(spawn_local(
                self.loader
                    .data(self.step, self.offset + self.loaded[self.current].len()),
            ));
        }
    }

    pub fn loader_mut(&mut self) -> &mut L {
        &mut self.loader
    }

    pub fn loader(&self) -> &L {
        &self.loader
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

    pub fn step(&self) -> usize {
        self.step
    }

    pub fn set_step(&mut self, step: usize) {
        self.step = step;
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

pub trait AsyncLoader {
    type Data: 'static;
    type Error: 'static;

    // This should be an async trait but they are not stable yet.
    // The async-trait crate provides some of the required features but can't require the produced
    // future to be 'static
    //
    // To understand what's happening here check the docs for async-trait https://docs.rs/async-trait/0.1.33/async_trait/
    #[allow(clippy::type_complexity)]
    fn data(
        &mut self,
        step: usize,
        offset: usize,
    ) -> Pin<
        Box<dyn 'static + Future<Output = Result<(Vec<Self::Data>, Option<usize>), Self::Error>>>,
    >;
    fn item(&self, _: Rc<Self::Data>) {}
}
