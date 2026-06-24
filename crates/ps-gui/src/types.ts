export type CaptureSource = "accessibility" | "keystroke" | "clipboard" | "merged";

export interface SessionContext {
  bundle_id: string;
  app_name: string;
  pid: number;
  window_title: string;
}

export interface CaptureSnapshot {
  slot: {
    key: string;
    context: SessionContext;
  };
  content: string;
  source: CaptureSource;
  captured_at: string;
}

export interface CurrentPromptUpdate {
  snapshot: CaptureSnapshot | null;
  char_count: number;
  line_count: number;
  source: string | null;
  confidence: string;
  persisted: boolean;
  last_saved_at: string | null;
  paused: boolean;
}

export interface DraftRecord {
  id: number;
  slot_key: string;
  content: string;
  source: CaptureSource;
  app_name: string;
  bundle_id: string;
  window_title: string;
  char_count: number;
  updated_at: string;
  pinned: boolean;
}

export interface AppConfig {
  ax_poll_ms: number;
  debounce_ms: number;
  retention_days: number;
  max_drafts: number;
  excluded_bundle_ids: string[];
  capture_paused: boolean;
  launch_at_login: boolean;
  global_hotkey: string;
  crash_toast_enabled: boolean;
  recover_action: string;
  ui_language: string;
}

export interface PermissionsView {
  platform: string;
  gui_capture: boolean;
  input_monitoring: boolean;
  executable_path: string;
  data_dir: string;
}

export interface DaemonStatus {
  running: boolean;
  paused: boolean;
  started_at: string | null;
  db_path: string;
  data_dir: string;
  config_path: string;
  draft_count: number;
  db_size_bytes: number;
  platform: string;
}

export interface SystemInfoView {
  status: DaemonStatus;
  permissions: PermissionsView;
}
