use std::ffi::{c_int, c_uint, c_void};
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, SendError, Sender, TryRecvError};

use crate::ffi::{self, G_PRIORITY_DEFAULT};
use crate::util::{abort_on_panic, bool_value};

pub struct UiSender<T>(Sender<T>);

impl<T> Clone for UiSender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> UiSender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.0.send(value)
    }
}

pub struct UiSource {
    id: c_uint,
    not_send: PhantomData<Rc<()>>,
}

impl Drop for UiSource {
    fn drop(&mut self) {
        unsafe {
            ffi::g_source_remove(self.id);
        }
    }
}

struct ChannelData<T> {
    receiver: Receiver<T>,
    handler: Box<dyn FnMut(T)>,
}

pub fn ui_channel<T, F>(handler: F) -> (UiSender<T>, UiSource)
where
    T: Send + 'static,
    F: FnMut(T) + 'static,
{
    let (sender, receiver) = mpsc::channel();
    let data = Box::new(ChannelData {
        receiver,
        handler: Box::new(handler),
    });
    let id = unsafe {
        ffi::g_timeout_add_full(
            G_PRIORITY_DEFAULT,
            50,
            Some(channel_trampoline::<T>),
            Box::into_raw(data).cast(),
            Some(drop_channel_data::<T>),
        )
    };
    assert_ne!(id, 0, "GLib failed to create a UI channel");
    (
        UiSender(sender),
        UiSource {
            id,
            not_send: PhantomData,
        },
    )
}

unsafe extern "C" fn channel_trampoline<T: Send + 'static>(data: *mut c_void) -> c_int {
    let data = unsafe { &mut *data.cast::<ChannelData<T>>() };
    let mut connected = true;
    abort_on_panic(|| loop {
        match data.receiver.try_recv() {
            Ok(message) => (data.handler)(message),
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                connected = false;
                break;
            }
        }
    });
    bool_value(connected)
}

unsafe extern "C" fn drop_channel_data<T>(data: *mut c_void) {
    drop(unsafe { Box::from_raw(data.cast::<ChannelData<T>>()) });
}
