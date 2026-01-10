use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Worker {
    running: Arc<Mutex<bool>>,
    status: Arc<Mutex<String>>,
    log: Arc<Mutex<VecDeque<String>>>,
}

impl Default for Worker {
    fn default() -> Self {
        let mut log = VecDeque::new();
        log.push_back("Ready".to_string());
        Self {
            running: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new("Ready".to_string())),
            log: Arc::new(Mutex::new(log)),
        }
    }
}

impl Worker {
    const MAX_LOG_LINES: usize = 200;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn start<F>(&self, task: F)
    where
        F: FnOnce(Arc<Mutex<bool>>, Arc<Mutex<String>>, Arc<Mutex<VecDeque<String>>>) + Send + 'static,
    {
        *self.running.lock().unwrap() = true;

        // Clone for the thread
        let running_clone = Arc::clone(&self.running);
        let status_clone = Arc::clone(&self.status);
        let log_clone = Arc::clone(&self.log);

        thread::spawn(move || {
            task(running_clone, status_clone, log_clone);
        });
    }

    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
        self.set_status("Stopped");
    }

    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    pub fn get_status(&self) -> String {
        self.status.lock().unwrap().clone()
    }

    pub fn get_log(&self) -> Vec<String> {
        self.log.lock().unwrap().iter().cloned().collect()
    }

    pub fn push_log(log: &Arc<Mutex<VecDeque<String>>>, text: &str) {
        let mut log = log.lock().unwrap();
        log.push_back(text.to_string());
        while log.len() > Self::MAX_LOG_LINES {
            log.pop_front();
        }
    }

    pub fn set_status(&self, text: &str) {
        let mut status = self.status.lock().unwrap();
        if status.as_str() == text {
            return;
        }
        *status = text.to_string();

        Self::push_log(&self.log, text);
    }
}
