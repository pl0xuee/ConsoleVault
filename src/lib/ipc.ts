// Typed wrappers over the Rust `#[tauri::command]` backend.
import { invoke } from "@tauri-apps/api/core";

export type InstallMethod = "pacman" | "flatpak" | "source";

export interface AppConfig {
  rom_folders: string[];
  emulator_overrides: Record<string, string>;
  setup_complete: boolean;
}

export type Invocation =
  | { kind: "native"; program: string }
  | { kind: "flatpak"; app_id: string };

export interface EmulatorStatus {
  system: string;
  system_label: string;
  emulator: string;
  installed: boolean;
  invocation: Invocation | null;
  bios_required: boolean;
  has_pacman: boolean;
  has_flatpak: boolean;
}

export interface InstallPlan {
  system: string;
  emulator: string;
  method: InstallMethod;
  commands: string[];
  available_methods: InstallMethod[];
}

export interface UninstallPlan {
  system: string;
  emulator: string;
  commands: string[];
}

export interface Game {
  system: string;
  title: string;
  path: string;
  id: string;
}

export const getConfig = () => invoke<AppConfig>("get_config");
export const saveConfig = (config: AppConfig) => invoke<void>("save_config", { config });
export const detectEmulators = () => invoke<EmulatorStatus[]>("detect_emulators");
export const installPlan = (system: string, method?: InstallMethod) =>
  invoke<InstallPlan>("install_plan", { system, method: method ?? null });
export const installEmulator = (system: string, method: InstallMethod) =>
  invoke<void>("install_emulator", { system, method });
export const uninstallPlan = (system: string) =>
  invoke<UninstallPlan>("uninstall_plan", { system });
export const uninstallEmulator = (system: string) =>
  invoke<void>("uninstall_emulator", { system });
export const scanLibrary = () => invoke<Game[]>("scan_library");
export const getBoxart = (system: string, title: string) =>
  invoke<string | null>("get_boxart", { system, title });
export const launchGame = (system: string, path: string) =>
  invoke<void>("launch_game", { system, path });

export const SYSTEM_LABELS: Record<string, string> = {
  snes: "SNES",
  n64: "N64",
  ps1: "PS1",
  ps2: "PS2",
  ps3: "PS3",
};
