use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};

use core::task;
use std::{
    future::Future,
    sync::{Arc, Mutex, mpsc::{sync_channel, Receiver, SyncSender}},
    task::{Context},
    time::Duration,
};

use timer_future::TimerFuture;

#[derive(Clone)]
struct Spawner {
    task_sender: SyncSender<Arc<Task>>
}

struct Exect {
    ready_queue: Receiver<Arc<Task>>
}

struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,
    task_sender: SyncSender<Arc<Task>>
}

fn new_exec() -> (Exect, Spawner) {
    const MAX_QUEUED_TASK: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASK);
    (Exect { ready_queue }, Spawner{ task_sender })
}

impl Spawner {
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let task = Arc::new(Task {
            future: Mutex::new(Some(Box::pin(future))),
            task_sender: self.task_sender.clone()
        });
        self.task_sender.send(task).expect("Too many tasks queued")
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let cloned = arc_self.clone();
        arc_self.task_sender.send(cloned).expect("Too many tasks queued")
    }
}

impl Exect {
    fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take()  {
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&waker);
                if future.as_mut().poll(context).is_pending() {
                    *future_slot = Some(future)
                }
            }
        }
    }
}

fn main() {
    let (executor, spawner) = new_exec();
    spawner.spawn(async {
        println!("And I will cough and I will puff and I will blow your house down")
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("done");
    });
    drop(spawner);
    executor.run();
}

/* 
struct Socket {
    id: usize,
    local_executor: Exect
}

struct IoBlocker;

impl IoBlocker {
    fn new() -> Self {
        Self
    }

    fn add_io_event_interest(&self, io_object: &IoObject, event: Event) {}
}



struct Signals;

struct Event {
    id: usize,
    signals: Signals,
}

impl Socket {
    fn set_readable_callback(&self, waker: Waker) {
        let local_executor = self.local_executor;
        let id = self.id;
    }
}

*/