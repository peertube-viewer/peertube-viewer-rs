use rustyline::error::ReadlineError;

use std::sync::mpsc as sync_mpsc;
use tokio::sync::mpsc as async_mpsc;

use std::thread;

pub struct Editor {
    runner: thread::JoinHandle<()>,
    sender: sync_mpsc::Sender<ToCli>,
    receiver: async_mpsc::UnboundedReceiver<FromCli>,
}

impl Editor {
    pub fn new() -> Editor {
        let (sender, sync_rec) = sync_mpsc::channel();
        let (async_sender, receiver) = async_mpsc::unbounded_channel();
        let runner = thread::Builder::new()
            .name("RustyLine runner".to_string())
            .spawn(move || run(sync_rec, async_sender))
            .unwrap();
        Editor {
            runner,
            sender,
            receiver,
        }
    }

    pub async fn prompt(&mut self, prompt: String) -> rustyline::Result<String> {
        self.sender.send(ToCli::Prompt(prompt));
        match self.receiver.recv().await.unwrap() {
            FromCli::Terminated => panic!("rustyline thread was terminated early"),
            FromCli::Line(l) => l,
        }
    }
}

enum ToCli {
    Prompt(String),
    Terminate,
}

enum FromCli {
    Line(rustyline::Result<String>),
    Terminated,
}

fn run(mut receiver: sync_mpsc::Receiver<ToCli>, mut sender: async_mpsc::UnboundedSender<FromCli>) {
    let mut editor = rustyline::Editor::<()>::new();
    let mut line = Ok(String::new());
    loop {
        match receiver.recv() {
            Ok(ToCli::Prompt(p)) => line = editor.readline(&p),
            Err(_) | Ok(ToCli::Terminate) => break,
        };

        if sender.send(FromCli::Line(line)).is_err() {
            break;
        }
    }
    sender.send(FromCli::Terminated);
}
