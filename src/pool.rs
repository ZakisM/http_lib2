use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Condvar, Mutex, MutexGuard,
    },
    thread::{self, JoinHandle},
};

use crate::Result;

type Task = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    condvar: Arc<(Mutex<VecDeque<Task>>, Condvar)>,
    _num_cpus: usize,
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

        let num_of_tasks = Arc::new(AtomicUsize::new(0));

        let tasks = VecDeque::new();

        let condvar = Arc::new((Mutex::new(tasks), Condvar::new()));

        let mut workers = Vec::with_capacity(num_cpus as usize);

        for _ in 0..num_cpus {
            let condvar_c = condvar.clone();
            let num_of_tasks_c = num_of_tasks.clone();

            workers.push(thread::spawn(move || {
                let (tasks_lock, cvar) = &*condvar_c;

                loop {
                    let current_number_of_tasks = num_of_tasks_c.load(Ordering::Acquire);

                    let task = {
                        let mut tasks_lock = tasks_lock.lock().unwrap();

                        if current_number_of_tasks >= num_cpus {
                            cvar.notify_all();

                            tasks_lock.pop_front()
                        } else {
                            let mut cvar_guard: MutexGuard<VecDeque<Task>> =
                                cvar.wait(tasks_lock).unwrap();

                            cvar_guard.pop_front()
                        }
                    };

                    if let Some(task) = task {
                        task();

                        num_of_tasks_c.fetch_sub(1, Ordering::Release);
                    }
                }
            }));
        }

        Ok(Self {
            condvar,
            _num_cpus: num_cpus,
            _num_of_tasks: num_of_tasks,
            _workers: workers,
        })
    }

    pub fn spawn<F>(&mut self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let (tasks_lock, cvar) = &*self.condvar;

        {
            let mut tasks = tasks_lock.lock().unwrap();

            tasks.push_back(Box::new(task));
        }

        self._num_of_tasks.fetch_add(1, Ordering::Release);

        cvar.notify_all();
    }

    #[cfg(target_os = "windows")]
    fn num_cpus() -> Result<usize> {
        use std::env;

        let cpus_s = env::var("NUMBER_OF_PROCESSORS")?;

        let cpus = cpus_s.parse::<usize>()?;

        Ok(cpus)
    }

    #[cfg(target_os = "macos")]
    fn num_cpus() -> Result<usize> {
        use std::process::{Command, Stdio};

        use crate::error::HttpInternalError;

        let sysctl_cmd = Command::new("sysctl")
            .arg("-n")
            .arg("hw.logicalcpu")
            .stdout(Stdio::piped())
            .output()?;

        let cpus = String::from_utf8(sysctl_cmd.stdout)
            .map_err(HttpInternalError::from)
            .and_then(|s| {
                s.lines()
                    .next()
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or_else(|| HttpInternalError::new("Failed to read sysctl output"))
            })?;

        Ok(cpus)
    }

    #[cfg(target_os = "linux")]
    fn num_cpus() -> Result<usize> {
        let cpu_info = std::fs::read_to_string("/proc/cpuinfo")?;

        let cpus = cpu_info.lines().filter(|l| l.contains("processor")).count();

        Ok(cpus)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_num_cpus() {
        use super::*;

        let cpus = ThreadPool::num_cpus().unwrap();

        #[cfg(target_os = "windows")]
        assert_eq!(cpus, 8);

        #[cfg(target_os = "macos")]
        assert_eq!(cpus, 16);

        #[cfg(target_os = "linux")]
        assert_eq!(cpus, 8);
    }
}
