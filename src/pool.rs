use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Condvar, Mutex, MutexGuard,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

type Task = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    condvar: Arc<(Mutex<VecDeque<Task>>, Condvar)>,
    _num_of_tasks: Arc<AtomicUsize>,
    _workers: Vec<JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(cores: u8) -> Self {
        let _num_of_tasks = Arc::new(AtomicUsize::new(0));
        let _num_of_tasks_c = _num_of_tasks.clone();

        let tasks = VecDeque::new();

        let condvar = Arc::new((Mutex::new(tasks), Condvar::new()));
        let condvar_c = condvar.clone();

        thread::spawn(move || {
            let (_, cvar) = &*condvar_c;

            loop {
                let num_tasks = _num_of_tasks_c.load(Ordering::Acquire);

                match num_tasks {
                    1 => cvar.notify_one(),
                    n if n > 1 => cvar.notify_all(),
                    _ => {
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                };

                thread::sleep(Duration::from_nanos(100));
            }
        });

        let available_workers = cores - 1;

        let mut _workers = Vec::with_capacity(available_workers as usize);

        for _ in 0..available_workers {
            let condvar_c = condvar.clone();
            let _num_of_tasks_c = _num_of_tasks.clone();

            _workers.push(thread::spawn(move || {
                let (tasks_lock, cvar) = &*condvar_c;

                loop {
                    let task = {
                        let tasks_lock = tasks_lock.lock().unwrap();
                        let mut cvar_guard: MutexGuard<VecDeque<Task>> =
                            cvar.wait(tasks_lock).unwrap();

                        cvar_guard.pop_front()
                    };

                    if let Some(task) = task {
                        _num_of_tasks_c.fetch_sub(1, Ordering::Release);

                        task();
                    }
                }
            }));
        }

        Self {
            condvar,
            _num_of_tasks,
            _workers,
        }
    }

    pub fn spawn<F>(&mut self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut tasks = self.condvar.0.lock().unwrap();

        tasks.push_back(Box::new(task));

        self._num_of_tasks.fetch_add(1, Ordering::Release);
    }
}
