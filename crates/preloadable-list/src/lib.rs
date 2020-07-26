use std::sync::Arc;
use std::thread::{spawn, JoinHandle};

type Loading<D, E> = JoinHandle<Result<(Vec<D>, usize), E>>;

pub struct PreloadableList<L: AsyncLoader> {
    loaded: Vec<Vec<Arc<L::Data>>>,
    loading: Option<Loading<L::Data, L::Error>>,

    loader: Arc<L>,
    current: usize,
    offset: usize,
    step: usize,
    total: usize,
}

impl<L, D, E> PreloadableList<L>
where
    L: AsyncLoader<Data = D, Error = E> + Send + Sync + 'static,
    D: 'static + Send,
    E: 'static + Send,
{
    pub fn new(loader: L, step: usize) -> PreloadableList<L> {
        PreloadableList {
            loaded: Vec::new(),
            loading: None,
            loader: Arc::new(loader),
            current: 0,
            offset: 0,
            step,
            total: 0,
        }
    }

    pub fn ensure_init(&mut self) -> Result<(), E> {
        if self.loaded.is_empty() {
            self.next().map(|_| ())
        } else {
            Ok(())
        }
    }

    pub fn next(&mut self) -> Result<&[Arc<D>], E> {
        if !self.loaded.is_empty() {
            self.offset += self.loaded[self.current].len();
            self.current += 1;
        }
        if self.loaded.len() <= self.current {
            let temp;
            if let Some(handle) = self.loading.take() {
                temp = handle.join().unwrap()?;
            } else {
                temp = self.loader.data(self.step, self.offset)?;
            }
            let (data, new_total) = temp;
            self.loaded.push(data.into_iter().map(Arc::new).collect());
            self.total = new_total;
        }
        Ok(&self.loaded[self.current])
    }

    pub fn prev(&mut self) -> &Vec<Arc<D>> {
        if self.current >= 1 {
            self.current -= 1;
            self.offset -= self.loaded[self.current].len();
        }
        &self.loaded[self.current]
    }

    pub fn preload_next(&mut self) {
        if self.loaded.len() <= self.current + 1 && self.loading.is_none() {
            let step = self.step;
            let offset = self.offset + self.loaded[self.current].len();
            let loader = self.loader.clone();
            self.loading = Some(spawn(move || loader.data(step, offset)));
        }
    }

    pub fn loader(&self) -> &L {
        &self.loader
    }

    pub fn preload_id(&self, id: usize) {
        let data_cloned = self.loaded[self.current][id].clone();
        self.loader.item(data_cloned);
    }

    pub fn current(&self) -> &[Arc<D>] {
        &self.loaded[self.current]
    }

    pub fn current_len(&self) -> usize {
        self.current().len()
    }

    pub fn expected_total(&self) -> usize {
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
    type Data: 'static + Send;
    type Error: 'static + Send;

    fn data(&self, step: usize, offset: usize) -> Result<(Vec<Self::Data>, usize), Self::Error>;
    fn item(&self, _: Arc<Self::Data>) {}
}
