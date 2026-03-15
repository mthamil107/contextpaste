// ContextPaste — Tauri IPC Wrappers
// All Tauri invoke/listen calls go through this file.
// Components should NEVER call invoke() directly.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AiStatus, AutoPasteEvent, AutoPasteResult, ClipItem, PasteRule, RankedItem, WorkflowChain } from "./types";

// ============================================================
// Clipboard Commands
// ============================================================

export async function getRecentItems(
  limit: number = 50,
  offset: number = 0,
): Promise<ClipItem[]> {
  return invoke<ClipItem[]>("get_recent_items", { limit, offset });
}

export async function getItem(id: string): Promise<ClipItem> {
  return invoke<ClipItem>("get_item", { id });
}

export async function searchItems(
  query: string,
  limit: number = 20,
): Promise<ClipItem[]> {
  return invoke<ClipItem[]>("search_items", { query, limit });
}

export async function semanticSearch(
  query: string,
  limit: number = 10,
): Promise<ClipItem[]> {
  return invoke<ClipItem[]>("semantic_search", { query, limit });
}

export async function deleteItem(id: string): Promise<void> {
  return invoke<void>("delete_item", { id });
}

export async function togglePin(id: string): Promise<void> {
  return invoke<void>("toggle_pin", { id });
}

export async function toggleStar(id: string): Promise<void> {
  return invoke<void>("toggle_star", { id });
}

export async function pasteItem(id: string): Promise<void> {
  return invoke<void>("paste_item", { id });
}

export async function clearHistory(): Promise<void> {
  return invoke<void>("clear_history");
}

export async function clearExpiredCredentials(): Promise<void> {
  return invoke<void>("clear_expired_credentials");
}

// ============================================================
// Prediction Commands
// ============================================================

export async function getPredictions(
  limit: number = 8,
): Promise<RankedItem[]> {
  return invoke<RankedItem[]>("get_predictions", { limit });
}

// ============================================================
// Workflow Chain Commands
// ============================================================

export async function getWorkflowChains(
  limit: number = 10,
): Promise<WorkflowChain[]> {
  return invoke<WorkflowChain[]>("get_workflow_chains", { limit });
}

// ============================================================
// Settings Commands
// ============================================================

export async function getAllSettings(): Promise<Record<string, string>> {
  return invoke<Record<string, string>>("get_all_settings");
}

export async function updateSetting(
  key: string,
  value: string,
): Promise<void> {
  return invoke<void>("update_setting", { key, value });
}

export async function getIgnoredApps(): Promise<string[]> {
  return invoke<string[]>("get_ignored_apps");
}

export async function addIgnoredApp(appName: string): Promise<void> {
  return invoke<void>("add_ignored_app", { appName });
}

export async function removeIgnoredApp(appName: string): Promise<void> {
  return invoke<void>("remove_ignored_app", { appName });
}

// ============================================================
// AI Commands
// ============================================================

export async function configureAiProvider(
  provider: string,
  apiKey?: string,
  baseUrl?: string,
  modelName?: string,
): Promise<void> {
  return invoke<void>("configure_ai_provider", { provider, apiKey, baseUrl, modelName });
}

export async function testAiConnection(): Promise<string> {
  return invoke<string>("test_ai_connection");
}

export async function getAiStatus(): Promise<AiStatus> {
  return invoke<AiStatus>("get_ai_status");
}

export async function backfillEmbeddings(): Promise<number> {
  return invoke<number>("backfill_embeddings");
}

// ============================================================
// Auto-Paste Commands
// ============================================================

export async function getPasteRules(): Promise<PasteRule[]> {
  return invoke<PasteRule[]>("get_paste_rules");
}

export async function createPasteRule(rule: Omit<PasteRule, 'id' | 'timesTriggered' | 'lastTriggeredAt' | 'createdAt' | 'updatedAt'>): Promise<string> {
  return invoke<string>("create_paste_rule", { rule });
}

export async function updatePasteRule(rule: PasteRule): Promise<void> {
  return invoke<void>("update_paste_rule", { rule });
}

export async function deletePasteRule(id: string): Promise<void> {
  return invoke<void>("delete_paste_rule", { id });
}

export async function togglePasteRule(id: string): Promise<void> {
  return invoke<void>("toggle_paste_rule", { id });
}

export async function getAutoPasteHistory(limit: number): Promise<AutoPasteEvent[]> {
  return invoke<AutoPasteEvent[]>("get_auto_paste_history", { limit });
}

export async function rateAutoPaste(eventId: string, correct: boolean): Promise<void> {
  return invoke<void>("rate_auto_paste", { eventId, correct });
}

// ============================================================
// Event Listeners
// ============================================================

export function onNewClipItem(
  callback: (item: ClipItem) => void,
): Promise<UnlistenFn> {
  return listen<ClipItem>("clipboard:new-item", (event) => {
    callback(event.payload);
  });
}

export function onClipboardError(
  callback: (error: string) => void,
): Promise<UnlistenFn> {
  return listen<string>("clipboard:error", (event) => {
    callback(event.payload);
  });
}

export function onChainDetected(
  callback: (chain: WorkflowChain) => void,
): Promise<UnlistenFn> {
  return listen<WorkflowChain>("workflow:chain-detected", (event) => {
    callback(event.payload);
  });
}

export function onCredentialDetected(
  callback: (data: { itemId: string; credType: string }) => void,
): Promise<UnlistenFn> {
  return listen<{ itemId: string; credType: string }>(
    "security:credential-detected",
    (event) => {
      callback(event.payload);
    },
  );
}

export function onSettingsChanged(
  callback: (data: { key: string; value: string }) => void,
): Promise<UnlistenFn> {
  return listen<{ key: string; value: string }>("settings:changed", (event) => {
    callback(event.payload);
  });
}

export function onAutoPasteSuccess(
  callback: (result: AutoPasteResult) => void,
): Promise<UnlistenFn> {
  return listen<AutoPasteResult>("autopaste:success", (event) => {
    callback(event.payload);
  });
}
