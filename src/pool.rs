use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Condvar, Mutex, MutexGuard,
    },
    thread,
    time::Duration,
};

type Task = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    condvar: Arc<(Mutex<VecDeque<Task>>, Condvar)>,
    _num_of_tasks: Arc<AtomicUsize>,
}

impl ThreadPool {
    pub fn new(cores: u8) -> Self {
        let _num_of_tasks = Arc::new(AtomicUsize::new(0));
        let _num_of_tasks_c = _num_of_tasks.clone();

        let tasks = VecDeque::new();

        let condvar = Arc::new((Mutex::new(tasks), Condvar::new()));
        let condvar_c = condvar.clone();

        thread::spawn(move || loop {
            let (_, cvar) = &*condvar_c;
            let num_tasks = _num_of_tasks_c.load(Ordering::Acquire);

            match num_tasks {
                1 => cvar.notify_one(),
                n if n > 1 => cvar.notify_all(),
                _ => continue,
            };

            thread::sleep(Duration::from_millis(10));
        });

        for _ in 0..cores - 1 {
            let condvar_c = condvar.clone();

            thread::spawn(move || {
                let (tasks_lock, cvar) = &*condvar_c;

                loop {
                    let tasks_lock = tasks_lock.lock().unwrap();

                    let mut cvar_guard: MutexGuard<VecDeque<Task>> = cvar.wait(tasks_lock).unwrap();

                    if let Some(task) = cvar_guard.pop_front() {
                        task()
                    }
                }
            });
        }

        Self {
            condvar,
            _num_of_tasks,
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
