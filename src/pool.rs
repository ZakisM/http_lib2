use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Condvar, Mutex, MutexGuard,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::Result;

type Task = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    condvar: Arc<(Mutex<VecDeque<Task>>, Condvar)>,
    _num_of_tasks: Arc<AtomicUsize>,
    _workers: Vec<JoinHandle<()>>,
}

impl std::fmt::Debug for ThreadPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreadPool")
            .field("_num_of_tasks", &self._num_of_tasks)
            .field("_workers", &self._workers)
            .finish()
    }
}

impl ThreadPool {
    pub fn new() -> Result<Self> {
        let num_cpus = Self::num_cpus()?;

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
                        thread::sleep(Duration::from_nanos(10));
                    }
                };
            }
        });

        let available_workers = num_cpus - 1;

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

        Ok(Self {
            condvar,
            _num_of_tasks,
            _workers,
        })
    }

    pub fn spawn<F>(&mut self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut tasks = self.condvar.0.lock().unwrap();

        tasks.push_back(Box::new(task));

        self._num_of_tasks.fetch_add(1, Ordering::Release);
    }

    #[cfg(windows)]
    fn num_cpus() -> Result<usize> {
        use std::env;

        let cpus_s = env::var("NUMBER_OF_PROCESSORS")?;

        let cpus = cpus_s.parse::<usize>()?;

        Ok(cpus)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_get_num_cpus() {
        let cpus = ThreadPool::num_cpus().unwrap();

        assert_eq!(cpus, 8);
    }
}
