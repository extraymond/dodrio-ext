use crate::prelude::*;
use async_executors::*;
use async_trait::async_trait;
use dodrio::{RootRender, VdomWeak};
use futures::channel::mpsc::unbounded;
use futures::lock::Mutex;
use std::rc::Rc;

pub type Message<T> = Box<dyn Messenger<Target = T>>;
pub type MessageSender<T> = Sender<(Message<T>, oneshot::Sender<()>)>;
pub type MessageReceiver<T> = Receiver<(Message<T>, oneshot::Sender<()>)>;

pub trait Messenger {
    type Target;

    fn update(
        self: Box<Self>,
        target: &mut Self::Target,
        sender: &MessageSender<Self::Target>,
        render_tx: &Sender<((), oneshot::Sender<()>)>,
    ) -> bool {
        false
    }

    fn dispatch(self, sender: &MessageSender<Self::Target>) -> JoinHandle<()>
    where
        Self: Sized + 'static,
    {
        let executor = Bindgen::new();
        let mut sender = sender.clone();
        let task = executor
            .spawn_handle_local(async move {
                let (tx, rx) = oneshot::channel::<()>();
                let _ = sender.send((Box::new(self), tx)).await;
                let _ = rx.await;
            })
            .unwrap();
        task
    }
}

pub fn consume<T, M>(
    convert: impl Fn(Event) -> M + 'static,
    sender: &MessageSender<T>,
) -> impl Fn(&mut dyn RootRender, VdomWeak, Event) + 'static
where
    M: Messenger<Target = T> + 'static,
    T: 'static,
{
    let sender = sender.clone();
    move |_, _, event| {
        let msg = convert(event);
        spawn_local(msg.dispatch(&sender));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct Data {
        button: bool,
    }

    pub struct Data2 {
        button: bool,
    }

    pub enum Msg {
        Flipit,
    }

    pub enum Msg2 {
        Secret,
    }

    impl Messenger for Msg {
        type Target = Data;

        fn update(
            self: Box<Self>,
            target: &mut Self::Target,
            sender: &MessageSender<Self::Target>,
            render_tx: &Sender<((), oneshot::Sender<()>)>,
        ) -> bool {
            target.button = !target.button;
            true
        }
    }

    impl Messenger for Msg2 {
        type Target = Data;

        fn update(
            self: Box<Self>,
            target: &mut Self::Target,
            sender: &Sender<(
                Box<dyn Messenger<Target = Self::Target>>,
                oneshot::Sender<()>,
            )>,
            render_tx: &Sender<((), oneshot::Sender<()>)>,
        ) -> bool {
            log::info!("not sure what to do, {}", target.button);
            false
        }
    }

    pub struct Container<T> {
        data: Rc<Mutex<T>>,
    }

    impl Container<Data> {
        fn start_handling(&self) {
            let (render_tx, _) = unbounded::<((), oneshot::Sender<()>)>();
            let (tx, mut rx) = unbounded::<(Message<Data>, oneshot::Sender<()>)>();
            let data = self.data.clone();
            let tx_handle = tx.clone();
            let fut = async move {
                while let Some((msg, ready)) = rx.next().await {
                    let mut content = data.lock().await;
                    msg.update(&mut content, &tx_handle, &render_tx);
                    let _ = ready.send(());
                    log::info!("content value: {}", content.button);
                }
            };

            spawn_local(fut);
        }
    }
}
