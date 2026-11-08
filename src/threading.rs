// this is just a small file for shared threading functions
use crossbeam::channel as cc;
use parking_lot::{Mutex, Once, RwLock};
use std::sync::mpsc::{RecvError, RecvTimeoutError, TryRecvError};
use std::sync::mpsc::{SendError, TrySendError};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
