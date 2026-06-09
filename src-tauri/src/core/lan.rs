use rdev::EventType;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap},
    net::UdpSocket,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager, Runtime, State, command};

const PROTOCOL_VERSION: u8 = 1;
const DEFAULT_PORT: u16 = 47832;
const BROADCAST_ADDRESS: &str = "255.255.255.255";
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(2);
const SOCKET_POLL_INTERVAL: Duration = Duration::from_millis(500);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(6);
const MAX_PACKET_SIZE: usize = 8 * 1024;
const MAX_CLIENTS: usize = 16;
const INPUT_THROTTLE_INTERVAL: Duration = Duration::from_millis(33);
const LAN_STATE_CHANGED_EVENT: &str = "lan-state-changed";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanSettings {
    pub nickname: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LanCursorPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LanInputSnapshot {
    pub keyboard_keys: Vec<String>,
    pub mouse_buttons: Vec<String>,
    pub cursor: Option<LanCursorPosition>,
    pub gamepad_buttons: BTreeMap<String, f32>,
    pub gamepad_axes: BTreeMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LanInputEvent {
    Keyboard { key: String, pressed: bool },
    MouseButton { button: String, pressed: bool },
    MouseMove { x: f64, y: f64 },
    GamepadButton { name: String, value: f32 },
    GamepadAxis { name: String, value: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotPayload {
    snapshot: LanInputSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InputPayload {
    input: LanInputEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LanEnvelope {
    version: u8,
    client_id: String,
    nickname: String,
    message_type: String,
    sequence: u64,
    timestamp: u64,
    payload: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanLocalClient {
    pub client_id: String,
    pub nickname: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanRemoteClient {
    pub client_id: String,
    pub nickname: String,
    pub last_seen_at: u64,
    pub input_state: LanInputSnapshot,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanStatePayload {
    pub enabled: bool,
    pub status: String,
    pub error: Option<String>,
    pub local_client: LanLocalClient,
    pub remote_clients: Vec<LanRemoteClient>,
}

#[derive(Debug)]
struct RemoteClientState {
    nickname: String,
    last_sequence: u64,
    last_seen_at: u64,
    last_seen_instant: Instant,
    input_state: LanInputSnapshot,
}

pub struct LanSyncState {
    client_id: String,
    runtime: Mutex<Option<Arc<LanRuntime>>>,
    settings: Mutex<LanLocalClient>,
}

struct LanRuntime {
    client_id: String,
    nickname: String,
    port: u16,
    socket: Arc<UdpSocket>,
    shutdown: Arc<AtomicBool>,
    sequence: AtomicU64,
    local_snapshot: Mutex<LanInputSnapshot>,
    remote_clients: Mutex<HashMap<String, RemoteClientState>>,
    last_mouse_sent_at: Mutex<Option<Instant>>,
    last_axis_sent_at: Mutex<HashMap<String, Instant>>,
}

impl Default for LanSyncState {
    fn default() -> Self {
        let client_id = format!(
            "{:x}-{}",
            current_timestamp_ms(),
            std::process::id()
        );
        let nickname = resolve_nickname("", &client_id);

        Self {
            settings: Mutex::new(LanLocalClient {
                client_id: client_id.clone(),
                nickname,
                port: DEFAULT_PORT,
            }),
            client_id,
            runtime: Mutex::new(None),
        }
    }
}

impl LanRuntime {
    fn local_client(&self) -> LanLocalClient {
        LanLocalClient {
            client_id: self.client_id.clone(),
            nickname: self.nickname.clone(),
            port: self.port,
        }
    }

    fn remote_clients_snapshot(&self) -> Vec<LanRemoteClient> {
        let mut clients = self
            .remote_clients
            .lock()
            .expect("lan remote clients lock poisoned")
            .iter()
            .map(|(client_id, client)| LanRemoteClient {
                client_id: client_id.clone(),
                nickname: client.nickname.clone(),
                last_seen_at: client.last_seen_at,
                input_state: client.input_state.clone(),
            })
            .collect::<Vec<_>>();

        clients.sort_by(|left, right| right.last_seen_at.cmp(&left.last_seen_at));

        clients
    }

    fn emit_state<R: Runtime>(
        &self,
        app_handle: &AppHandle<R>,
        status: &str,
        error: Option<String>,
    ) {
        let _ = app_handle.emit(
            LAN_STATE_CHANGED_EVENT,
            LanStatePayload {
                enabled: true,
                status: status.to_string(),
                error,
                local_client: self.local_client(),
                remote_clients: self.remote_clients_snapshot(),
            },
        );
    }

    fn current_snapshot(&self) -> LanInputSnapshot {
        self.local_snapshot
            .lock()
            .expect("lan local snapshot lock poisoned")
            .clone()
    }

    fn should_send_mouse_move(&self) -> bool {
        let mut last_sent_at = self
            .last_mouse_sent_at
            .lock()
            .expect("lan mouse throttle lock poisoned");
        let now = Instant::now();

        if last_sent_at
            .as_ref()
            .is_some_and(|value| now.duration_since(*value) < INPUT_THROTTLE_INTERVAL)
        {
            return false;
        }

        *last_sent_at = Some(now);

        true
    }

    fn should_send_axis(&self, axis_name: &str) -> bool {
        let mut last_sent_at = self
            .last_axis_sent_at
            .lock()
            .expect("lan axis throttle lock poisoned");
        let now = Instant::now();

        if last_sent_at
            .get(axis_name)
            .is_some_and(|value| now.duration_since(*value) < INPUT_THROTTLE_INTERVAL)
        {
            return false;
        }

        last_sent_at.insert(axis_name.to_string(), now);

        true
    }

    fn update_local_snapshot(&self, input: &LanInputEvent) {
        let mut snapshot = self
            .local_snapshot
            .lock()
            .expect("lan local snapshot lock poisoned");

        apply_input_event(&mut snapshot, input.clone());
    }

    fn broadcast_input_event(&self, input: LanInputEvent) {
        self.update_local_snapshot(&input);

        let should_send = match &input {
            LanInputEvent::MouseMove { .. } => self.should_send_mouse_move(),
            LanInputEvent::GamepadAxis { name, .. } => self.should_send_axis(name),
            _ => true,
        };

        if !should_send {
            return;
        }

        let payload = InputPayload { input };

        let _ = self.broadcast_message("input", &payload);
    }

    fn broadcast_message<T: Serialize>(&self, message_type: &str, payload: &T) -> Result<(), String> {
        let payload = serde_json::to_value(payload).map_err(|error| error.to_string())?;
        let envelope = LanEnvelope {
            version: PROTOCOL_VERSION,
            client_id: self.client_id.clone(),
            nickname: self.nickname.clone(),
            message_type: message_type.to_string(),
            sequence: self.sequence.fetch_add(1, Ordering::SeqCst),
            timestamp: current_timestamp_ms(),
            payload,
        };
        let bytes = serde_json::to_vec(&envelope).map_err(|error| error.to_string())?;

        if bytes.len() > MAX_PACKET_SIZE {
            return Err("LAN packet exceeded maximum size".to_string());
        }

        self.socket
            .send_to(&bytes, (BROADCAST_ADDRESS, self.port))
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn remove_timed_out_clients(&self) -> bool {
        let mut remote_clients = self
            .remote_clients
            .lock()
            .expect("lan remote clients lock poisoned");
        let now = Instant::now();
        let before_len = remote_clients.len();

        remote_clients.retain(|_, client| now.duration_since(client.last_seen_instant) <= CLIENT_TIMEOUT);

        before_len != remote_clients.len()
    }

    fn process_message(&self, message: LanEnvelope) -> bool {
        if message.version != PROTOCOL_VERSION || message.client_id == self.client_id {
            return false;
        }

        let mut remote_clients = self
            .remote_clients
            .lock()
            .expect("lan remote clients lock poisoned");
        let now = Instant::now();

        if message.message_type == "leave" {
            let should_remove = remote_clients
                .get(&message.client_id)
                .is_some_and(|client| message.sequence > client.last_sequence);

            if should_remove {
                remote_clients.remove(&message.client_id);
            }

            return should_remove;
        }

        if !remote_clients.contains_key(&message.client_id) && remote_clients.len() >= MAX_CLIENTS {
            return false;
        }

        let client = remote_clients.entry(message.client_id.clone()).or_insert_with(|| RemoteClientState {
            nickname: message.nickname.clone(),
            last_sequence: 0,
            last_seen_at: 0,
            last_seen_instant: now,
            input_state: LanInputSnapshot::default(),
        });

        if message.sequence <= client.last_sequence {
            return false;
        }

        client.nickname = message.nickname.clone();
        client.last_sequence = message.sequence;
        client.last_seen_at = message.timestamp;
        client.last_seen_instant = now;

        match message.message_type.as_str() {
            "hello" | "heartbeat" => {
                let Ok(payload) = serde_json::from_value::<SnapshotPayload>(message.payload) else {
                    return false;
                };

                client.input_state = payload.snapshot;

                true
            }
            "input" => {
                let Ok(payload) = serde_json::from_value::<InputPayload>(message.payload) else {
                    return false;
                };

                apply_input_event(&mut client.input_state, payload.input);

                true
            }
            _ => false,
        }
    }
}

#[command]
pub fn get_lan_sync_state(state: State<'_, LanSyncState>) -> LanStatePayload {
    if let Some(runtime) = state
        .runtime
        .lock()
        .expect("lan runtime lock poisoned")
        .as_ref()
        .cloned()
    {
        return LanStatePayload {
            enabled: true,
            status: "listening".to_string(),
            error: None,
            local_client: runtime.local_client(),
            remote_clients: runtime.remote_clients_snapshot(),
        };
    }

    let local_client = state
        .settings
        .lock()
        .expect("lan settings lock poisoned")
        .clone();

    LanStatePayload {
        enabled: false,
        status: "disabled".to_string(),
        error: None,
        local_client,
        remote_clients: Vec::new(),
    }
}

#[command]
pub fn start_lan_sync<R: Runtime>(
    app_handle: AppHandle<R>,
    state: State<'_, LanSyncState>,
    settings: LanSettings,
) -> Result<(), String> {
    if settings.port == 0 {
        emit_stopped_state(&app_handle, &state, Some("LAN port must be greater than 0".to_string()));

        return Err("LAN port must be greater than 0".to_string());
    }

    stop_runtime(&state);

    let resolved_nickname = resolve_nickname(&settings.nickname, &state.client_id);
    let socket = UdpSocket::bind(("0.0.0.0", settings.port)).map_err(|error| {
        let message = format!("Failed to bind LAN UDP port {}: {}", settings.port, error);
        emit_stopped_state(&app_handle, &state, Some(message.clone()));
        message
    })?;

    socket
        .set_broadcast(true)
        .map_err(|error| {
            let message = format!("Failed to enable UDP broadcast: {}", error);
            emit_stopped_state(&app_handle, &state, Some(message.clone()));
            message
        })?;
    socket
        .set_read_timeout(Some(SOCKET_POLL_INTERVAL))
        .map_err(|error| {
            let message = format!("Failed to set LAN socket timeout: {}", error);
            emit_stopped_state(&app_handle, &state, Some(message.clone()));
            message
        })?;

    {
        let mut local_settings = state
            .settings
            .lock()
            .expect("lan settings lock poisoned");

        local_settings.nickname = resolved_nickname.clone();
        local_settings.port = settings.port;
    }

    let runtime = Arc::new(LanRuntime {
        client_id: state.client_id.clone(),
        nickname: resolved_nickname,
        port: settings.port,
        socket: Arc::new(socket),
        shutdown: Arc::new(AtomicBool::new(false)),
        sequence: AtomicU64::new(1),
        local_snapshot: Mutex::new(LanInputSnapshot::default()),
        remote_clients: Mutex::new(HashMap::new()),
        last_mouse_sent_at: Mutex::new(None),
        last_axis_sent_at: Mutex::new(HashMap::new()),
    });

    {
        let mut shared_runtime = state.runtime.lock().expect("lan runtime lock poisoned");
        *shared_runtime = Some(runtime.clone());
    }

    spawn_receiver_thread(app_handle.clone(), runtime.clone());
    spawn_heartbeat_thread(app_handle.clone(), runtime.clone());

    if let Err(error) = runtime.broadcast_message(
        "hello",
        &SnapshotPayload {
            snapshot: runtime.current_snapshot(),
        },
    ) {
        stop_runtime(&state);
        emit_stopped_state(&app_handle, &state, Some(error.clone()));

        return Err(error);
    }

    runtime.emit_state(&app_handle, "listening", None);

    Ok(())
}

#[command]
pub fn stop_lan_sync<R: Runtime>(
    app_handle: AppHandle<R>,
    state: State<'_, LanSyncState>,
) -> Result<(), String> {
    stop_runtime(&state);
    emit_stopped_state(&app_handle, &state, None);

    Ok(())
}

pub fn forward_local_device_event<R: Runtime>(app_handle: &AppHandle<R>, event: &EventType) {
    let Some(runtime) = active_runtime(app_handle) else {
        return;
    };

    let input = match event {
        EventType::ButtonPress(button) => LanInputEvent::MouseButton {
            button: format!("{:?}", button),
            pressed: true,
        },
        EventType::ButtonRelease(button) => LanInputEvent::MouseButton {
            button: format!("{:?}", button),
            pressed: false,
        },
        EventType::MouseMove { x, y } => LanInputEvent::MouseMove { x: *x, y: *y },
        EventType::KeyPress(key) => LanInputEvent::Keyboard {
            key: format!("{:?}", key),
            pressed: true,
        },
        EventType::KeyRelease(key) => LanInputEvent::Keyboard {
            key: format!("{:?}", key),
            pressed: false,
        },
        _ => return,
    };

    runtime.broadcast_input_event(input);
}

pub fn forward_local_gamepad_event<R: Runtime>(
    app_handle: &AppHandle<R>,
    name: &str,
    value: f32,
    is_axis: bool,
) {
    let Some(runtime) = active_runtime(app_handle) else {
        return;
    };

    let input = if is_axis {
        LanInputEvent::GamepadAxis {
            name: name.to_string(),
            value,
        }
    } else {
        LanInputEvent::GamepadButton {
            name: name.to_string(),
            value,
        }
    };

    runtime.broadcast_input_event(input);
}

fn active_runtime<R: Runtime>(app_handle: &AppHandle<R>) -> Option<Arc<LanRuntime>> {
    app_handle
        .state::<LanSyncState>()
        .runtime
        .lock()
        .expect("lan runtime lock poisoned")
        .as_ref()
        .cloned()
}

fn stop_runtime(state: &LanSyncState) {
    let runtime = state
        .runtime
        .lock()
        .expect("lan runtime lock poisoned")
        .take();

    if let Some(runtime) = runtime {
        let _ = runtime.broadcast_message("leave", &Value::Null);
        runtime.shutdown.store(true, Ordering::SeqCst);
    }
}

fn emit_stopped_state<R: Runtime>(
    app_handle: &AppHandle<R>,
    state: &LanSyncState,
    error: Option<String>,
) {
    let local_client = state
        .settings
        .lock()
        .expect("lan settings lock poisoned")
        .clone();

    let _ = app_handle.emit(
        LAN_STATE_CHANGED_EVENT,
        LanStatePayload {
            enabled: false,
            status: if error.is_some() {
                "error".to_string()
            } else {
                "disabled".to_string()
            },
            error,
            local_client,
            remote_clients: Vec::new(),
        },
    );
}

fn spawn_receiver_thread<R: Runtime>(app_handle: AppHandle<R>, runtime: Arc<LanRuntime>) {
    thread::spawn(move || {
        let mut buffer = vec![0_u8; MAX_PACKET_SIZE];

        while !runtime.shutdown.load(Ordering::SeqCst) {
            match runtime.socket.recv_from(&mut buffer) {
                Ok((size, _)) => {
                    let Ok(message) = serde_json::from_slice::<LanEnvelope>(&buffer[..size]) else {
                        continue;
                    };

                    let changed = runtime.process_message(message);
                    let removed_timed_out_clients = runtime.remove_timed_out_clients();

                    if changed || removed_timed_out_clients {
                        runtime.emit_state(&app_handle, "listening", None);
                    }
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    if runtime.remove_timed_out_clients() {
                        runtime.emit_state(&app_handle, "listening", None);
                    }
                }
                Err(error) => {
                    runtime.shutdown.store(true, Ordering::SeqCst);
                    runtime.emit_state(&app_handle, "error", Some(error.to_string()));
                    break;
                }
            }
        }
    });
}

fn spawn_heartbeat_thread<R: Runtime>(app_handle: AppHandle<R>, runtime: Arc<LanRuntime>) {
    thread::spawn(move || {
        while !runtime.shutdown.load(Ordering::SeqCst) {
            thread::sleep(HEARTBEAT_INTERVAL);

            if runtime.shutdown.load(Ordering::SeqCst) {
                break;
            }

            let removed_timed_out_clients = runtime.remove_timed_out_clients();
            let result = runtime.broadcast_message(
                "heartbeat",
                &SnapshotPayload {
                    snapshot: runtime.current_snapshot(),
                },
            );

            if removed_timed_out_clients {
                runtime.emit_state(&app_handle, "listening", None);
            }

            if let Err(error) = result {
                runtime.emit_state(&app_handle, "error", Some(error));
            }
        }
    });
}

fn apply_input_event(snapshot: &mut LanInputSnapshot, input: LanInputEvent) {
    match input {
        LanInputEvent::Keyboard { key, pressed } => update_unique_items(&mut snapshot.keyboard_keys, key, pressed),
        LanInputEvent::MouseButton { button, pressed } => {
            update_unique_items(&mut snapshot.mouse_buttons, button, pressed)
        }
        LanInputEvent::MouseMove { x, y } => {
            snapshot.cursor = Some(LanCursorPosition { x, y });
        }
        LanInputEvent::GamepadButton { name, value } => {
            update_numeric_map(&mut snapshot.gamepad_buttons, name, value);
        }
        LanInputEvent::GamepadAxis { name, value } => {
            update_numeric_map(&mut snapshot.gamepad_axes, name, value);
        }
    }
}

fn update_unique_items(items: &mut Vec<String>, value: String, active: bool) {
    if active {
        if !items.iter().any(|item| item == &value) {
            items.push(value);
        }

        return;
    }

    items.retain(|item| item != &value);
}

fn update_numeric_map(values: &mut BTreeMap<String, f32>, key: String, value: f32) {
    if value.abs() < f32::EPSILON {
        values.remove(&key);

        return;
    }

    values.insert(key, value);
}

fn resolve_nickname(raw_nickname: &str, client_id: &str) -> String {
    let trimmed = raw_nickname.trim();

    if !trimmed.is_empty() {
        return trimmed.chars().take(32).collect();
    }

    for key in ["COMPUTERNAME", "HOSTNAME", "USER", "USERNAME"] {
        let Ok(value) = std::env::var(key) else {
            continue;
        };
        let trimmed = value.trim();

        if !trimmed.is_empty() {
            return trimmed.chars().take(32).collect();
        }
    }

    let suffix = client_id.chars().take(6).collect::<String>();

    format!("BongoCat-{}", suffix)
}

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis() as u64)
        .unwrap_or_default()
}
