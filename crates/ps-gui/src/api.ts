import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  CurrentPromptUpdate,
  DaemonStatus,
  DraftRecord,
  PermissionsView,
  SessionContext,
  SystemInfoView,
} from "./types";

export const api = {
  getCurrentPrompt: () => invoke<CurrentPromptUpdate>("get_current_prompt"),
  getDaemonStatus: () => invoke<DaemonStatus>("get_daemon_status"),
  listDrafts: (limit?: number) => invoke<DraftRecord[]>("list_drafts", { limit }),
  searchDrafts: (query: string, limit?: number) =>
    invoke<DraftRecord[]>("search_drafts", { query, limit }),
  getDraft: (id: number) => invoke<DraftRecord | null>("get_draft", { id }),
  deleteDraft: (id: number) => invoke<boolean>("delete_draft", { id }),
  deleteAllDrafts: () => invoke<number>("delete_all_drafts"),
  pinDraft: (id: number, pinned: boolean) => invoke<boolean>("pin_draft", { id, pinned }),
  recoverDraft: (id?: number) => invoke<DraftRecord>("recover_draft", { id }),
  copyText: (text: string) => invoke<void>("copy_text", { text }),
  getConfig: () => invoke<AppConfig>("get_config"),
  saveConfig: (config: AppConfig) => invoke<void>("save_config", { config }),
  setCapturePaused: (paused: boolean) => invoke<void>("set_capture_paused", { paused }),
  getPermissions: () => invoke<PermissionsView>("get_permissions"),
  refreshPermissions: () => invoke<PermissionsView>("refresh_permissions"),
  promptForPermissions: () => invoke<PermissionsView>("prompt_for_permissions"),
  openAccessibilitySettings: () => invoke<void>("open_accessibility_settings"),
  openInputMonitoringSettings: () => invoke<void>("open_input_monitoring_settings"),
  openDataDir: () => invoke<void>("open_data_dir"),
  getSystemInfo: () => invoke<SystemInfoView>("get_system_info"),
  markSlotSubmitted: (slotKey: string) => invoke<void>("mark_slot_submitted", { slotKey }),
  exportDraftText: (id: number) => invoke<string>("export_draft_text", { id }),
  getExecutablePath: () => invoke<string>("get_executable_path"),
  getFrontmostSession: () => invoke<SessionContext | null>("get_frontmost_session"),
  injectDraftForTest: (text: string, app: string, bundle: string) =>
    invoke<void>("inject_draft_for_test", { text, app, bundle }),
  getRecentLogs: (lines?: number) => invoke<string>("get_recent_logs", { lines }),
  getLaunchAtLoginStatus: () => invoke<boolean>("get_launch_at_login_status"),
  setLaunchAtLogin: (enabled: boolean) => invoke<boolean>("set_launch_at_login", { enabled }),
};
