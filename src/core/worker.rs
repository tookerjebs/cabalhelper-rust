use std::sync::{Arc, Mutex};
use std::thread;

pub struct Worker {
    running: Arc<Mutex<bool>>,
    status: Arc<Mutex<String>>,
}

impl Default for Worker {
    fn default() -> Self {
        Self {
            running: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new("Ready".to_string())),
        }
    }
}

impl Worker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start<F>(&self, task: F)
    where
        F: FnOnce(Arc<Mutex<bool>>, Arc<Mutex<String>>) + Send + 'static,
    {
        *self.running.lock().unwrap() = true;
        
        // Clone for the thread
        let running_clone = Arc::clone(&self.running);
        let status_clone = Arc::clone(&self.status);
        
        thread::spawn(move || {
            task(running_clone, status_clone);
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

    pub fn set_status(&self, text: &str) {
        *self.status.lock().unwrap() = text.to_string();
    }
}
