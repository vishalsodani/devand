use super::FetchCallback;
use devand_core::User;
use gloo::timers::callback::Timeout;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::BeforeUnloadEvent;
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};

const DELAY_MS: u32 = 2_000;
const API_URL: &'static str = "/api/settings";

pub struct UserService {
    // put_handler is wrapped in Arc<Mutex> so it can be passed to Timeout
    put_handler: Arc<Mutex<PutHandler>>,
    get_handler: GetHandler,
    on_unload: Closure<dyn FnMut(BeforeUnloadEvent) -> ()>,
}

struct PutHandler {
    service: FetchService,
    callback: FetchCallback,
    task: Option<FetchTask>,
    debouncer: Option<Timeout>,
    pending: Arc<Mutex<bool>>,
}

struct GetHandler {
    service: FetchService,
    callback: FetchCallback,
    task: Option<FetchTask>,
    pending: Arc<Mutex<bool>>,
}

fn request<R>(
    service: &mut FetchService,
    callback: Callback<Result<User, anyhow::Error>>,
    pending: Arc<Mutex<bool>>,
    r: http::request::Request<R>,
) -> Result<FetchTask, anyhow::Error>
where
    R: std::convert::Into<std::result::Result<std::string::String, anyhow::Error>>,
{
    let handler = move |response: Response<Json<Result<User, anyhow::Error>>>| {
        let (meta, Json(data)) = response.into_parts();
        if meta.status.is_success() {
            callback.emit(data)
        } else {
            callback.emit(Err(anyhow::anyhow!("Error {} restoring user", meta.status)))
        }

        if let Ok(mut pending) = pending.lock() {
            log::debug!("Finished");
            *pending.deref_mut() = false;
        }
    };

    service.fetch(r, handler.into())
}

impl GetHandler {
    fn get(&mut self) {
        let req = Request::get(API_URL).body(Nothing).unwrap();
        self.task = request(
            &mut self.service,
            self.callback.clone(),
            self.pending.clone(),
            req,
        )
        .ok();
    }
}

impl PutHandler {
    fn put(&mut self, user: User) {
        let json = serde_json::to_string(&user).map_err(|_| anyhow::anyhow!("bo!"));
        let req = Request::put(API_URL).body(json).unwrap();
        self.task = request(
            &mut self.service,
            self.callback.clone(),
            self.pending.clone(),
            req,
        )
        .ok();
    }
}

impl UserService {
    pub fn new(callback: FetchCallback) -> Self {
        let put_handler = PutHandler {
            service: FetchService::new(),
            callback: callback.clone(),
            task: None,
            debouncer: None,
            pending: Arc::new(Mutex::new(false)),
        };

        let put_handler = Arc::new(Mutex::new(put_handler));

        let on_unload = make_on_unload_callback(put_handler.clone());

        let get_handler = GetHandler {
            service: FetchService::new(),
            callback: callback.clone(),
            task: None,
            pending: Arc::new(Mutex::new(false)),
        };

        Self {
            put_handler,
            get_handler,
            on_unload,
        }
    }

    pub fn restore(&mut self) {
        self.get_handler.get();
    }

    pub fn store(&mut self, user: &User) {
        let user: User = user.clone();

        let delayed_put_handler = self.put_handler.clone();

        if let Ok(mut put_handler) = self.put_handler.lock() {
            if let Ok(mut pending) = put_handler.pending.lock() {
                log::debug!("Start timer...");
                *pending.deref_mut() = true;
            }

            // Only if not already locked, clear previous timeout and set
            // a new delayed action. Note: overwriting debouncer clear the
            // previous timeout.
            put_handler.deref_mut().debouncer = Some(Timeout::new(DELAY_MS, move || {
                log::debug!("Start sending...");
                let mut put_handler = delayed_put_handler.lock().unwrap();
                put_handler.put(user);
            }));
        }
    }
}

fn make_on_unload_callback(
    put_handler: Arc<Mutex<PutHandler>>,
) -> Closure<dyn FnMut(BeforeUnloadEvent) -> ()> {
    let window = yew::utils::window();

    let on_unload = Box::new(move |e: BeforeUnloadEvent| {
        if let Ok(put_handler) = put_handler.lock() {
            if *put_handler.pending.lock().unwrap() {
                e.set_return_value("Changes you made may not be saved.");
            }
        }
    }) as Box<dyn FnMut(BeforeUnloadEvent)>;

    let on_unload = Closure::wrap(on_unload);

    window.set_onbeforeunload(Some(&on_unload.as_ref().unchecked_ref()));

    on_unload
}