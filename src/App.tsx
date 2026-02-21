import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { AppToaster, toaster } from "@/components/common/AppToaster";
import { AppShell } from "@/components/layout/AppShell";
import { HomePage } from "@/pages/HomePage";
import { SettingsPage } from "@/pages/SettingsPage";
import type {
  AutoStartSettings,
  ClipboardRecord,
  CustomGroup,
  DashboardStats,
  PageType,
  RecordExpirationSettings,
  RecordFilterType,
  StorageSettings,
} from "@/types/clipboard";

function App() {
  const [page, setPage] = useState<PageType>("home");
  const [keyword, setKeyword] = useState("");
  const [filterType, setFilterType] = useState<RecordFilterType>("all");
  const [records, setRecords] = useState<ClipboardRecord[]>([]);
  const [customGroups, setCustomGroups] = useState<CustomGroup[]>([]);
  const [activeGroupId, setActiveGroupId] = useState<number | null>(null);
  const [newGroupName, setNewGroupName] = useState("");
  const [stats, setStats] = useState<DashboardStats>({ total_records: 0 });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [storageSettings, setStorageSettings] = useState<StorageSettings | null>(null);
  const [autoStartSettings, setAutoStartSettings] = useState<AutoStartSettings | null>(null);
  const [savingAutoStart, setSavingAutoStart] = useState(false);
  const [recordExpirationSettings, setRecordExpirationSettings] = useState<RecordExpirationSettings | null>(null);
  const [savingRecordExpiration, setSavingRecordExpiration] = useState(false);

  const filteredRecords = useMemo(() => {
    const sorted = [...records].sort((a, b) => b.timestamp - a.timestamp);
    const typeFiltered =
      filterType === "all"
        ? sorted
        : filterType === "favorite"
          ? sorted.filter((record) => record.is_favorite)
          : sorted.filter((record) => record.content_type === filterType);

    if (activeGroupId === null) {
      return typeFiltered;
    }
    return typeFiltered.filter((record) => record.group_ids.includes(activeGroupId));
  }, [records, filterType, activeGroupId]);

  async function init() {
    try {
      await invoke("init_app");
      await Promise.all([
        loadRecords(),
        loadStats(),
        loadStorageSettings(),
        loadAutoStartSettings(),
        loadRecordExpirationSettings(),
        loadCustomGroups(),
      ]);
    } catch (e) {
      setError(String(e));
    }
  }

  async function loadRecords() {
    setLoading(true);
    setError("");
    try {
      const result = await invoke<ClipboardRecord[]>("get_all_records", {
        limit: 100,
        keyword: keyword.trim() || null,
      });
      setRecords(result);
    } catch (e) {
      setError(`加载记录失败：${String(e)}`);
      toaster.create({
        title: "加载失败",
        description: String(e),
        type: "error",
        duration: 3000,
      });
    } finally {
      setLoading(false);
    }
  }

  async function loadStats() {
    try {
      const result = await invoke<DashboardStats>("get_dashboard_stats");
      setStats(result);
    } catch (e) {
      setError(`加载统计失败：${String(e)}`);
    }
  }

  async function loadStorageSettings() {
    try {
      const result = await invoke<StorageSettings>("get_storage_settings");
      setStorageSettings(result);
    } catch (e) {
      setError(`加载设置失败：${String(e)}`);
    }
  }

  async function loadAutoStartSettings() {
    try {
      const result = await invoke<AutoStartSettings>("get_auto_start_settings");
      setAutoStartSettings(result);
    } catch (e) {
      setError(`加载开机启动设置失败：${String(e)}`);
    }
  }

  async function loadRecordExpirationSettings() {
    try {
      const result = await invoke<RecordExpirationSettings>("get_record_expiration_settings");
      setRecordExpirationSettings(result);
    } catch (e) {
      setError(`加载记录过期设置失败：${String(e)}`);
    }
  }

  async function loadCustomGroups() {
    try {
      const result = await invoke<CustomGroup[]>("get_custom_groups");
      setCustomGroups(result);
    } catch (e) {
      toaster.create({
        title: "加载自定义分组失败",
        description: String(e),
        type: "error",
        duration: 3000,
      });
    }
  }

  async function handleToggleAutoStart(nextEnabled: boolean) {
    setSavingAutoStart(true);
    try {
      await invoke("set_auto_start_settings", { autoStartEnabled: nextEnabled });
      setAutoStartSettings({ auto_start_enabled: nextEnabled });
      toaster.create({
        title: nextEnabled ? "已开启系统启动时运行" : "已关闭系统启动时运行",
        type: "success",
        duration: 2000,
      });
    } catch (e) {
      toaster.create({
        title: "更新系统启动时运行失败",
        description: String(e),
        type: "error",
        duration: 3000,
      });
    } finally {
      setSavingAutoStart(false);
    }
  }

  async function handleToggleRecordExpiration(nextEnabled: boolean) {
    setSavingRecordExpiration(true);
    try {
      await invoke("set_record_expiration_settings", {
        expirationEnabled: nextEnabled,
        expirationDays: nextEnabled ? (recordExpirationSettings?.expiration_days ?? 200) : 200,
      });
      setRecordExpirationSettings({
        expiration_enabled: nextEnabled,
        expiration_days: nextEnabled ? (recordExpirationSettings?.expiration_days ?? 200) : 200,
      });
      toaster.create({
        title: nextEnabled ? "已开启记录过期清理" : "已关闭记录过期清理",
        type: "success",
        duration: 2000,
      });
      await Promise.all([loadRecords(), loadStats()]);
    } catch (e) {
      toaster.create({
        title: "更新记录过期设置失败",
        description: String(e),
        type: "error",
        duration: 3000,
      });
    } finally {
      setSavingRecordExpiration(false);
    }
  }

  async function handleUpdateExpirationDays(days: number) {
    try {
      await invoke("set_record_expiration_settings", {
        expirationEnabled: true,
        expirationDays: days,
      });
      setRecordExpirationSettings((prev) => ({
        expiration_enabled: prev?.expiration_enabled ?? false,
        expiration_days: days,
      }));
    } catch (e) {
      toaster.create({
        title: "更新保留天数失败",
        description: String(e),
        type: "error",
        duration: 3000,
      });
    }
  }

  async function handleDelete(id: number) {
    try {
      await invoke("remove_record", { recordId: id });
      await Promise.all([loadRecords(), loadStats()]);
      toaster.create({
        title: "删除成功",
        type: "success",
        duration: 2000,
      });
    } catch (err) {
      setError(`删除失败：${String(err)}`);
      toaster.create({
        title: "删除失败",
        description: String(err),
        type: "error",
        duration: 3000,
      });
    }
  }

  async function handleToggleFavorite(id: number, isFavorite: boolean) {
    try {
      await invoke("toggle_favorite", { recordId: id, isFavorite });
      setRecords((prev) =>
        prev.map((record) =>
          record.id === id
            ? {
                ...record,
                is_favorite: isFavorite,
              }
            : record,
        ),
      );
    } catch (e) {
      toaster.create({
        title: "收藏操作失败",
        description: String(e),
        type: "error",
        duration: 2600,
      });
    }
  }

  async function handleCreateGroup() {
    const trimmed = newGroupName.trim();
    if (!trimmed) {
      toaster.create({
        title: "分组名称不能为空",
        type: "warning",
        duration: 2000,
      });
      return;
    }

    try {
      await invoke("create_group", { name: trimmed });
      setNewGroupName("");
      await loadCustomGroups();
      toaster.create({ title: "分组创建成功", type: "success", duration: 2000 });
    } catch (e) {
      toaster.create({
        title: "创建分组失败",
        description: String(e),
        type: "error",
        duration: 2600,
      });
    }
  }

  async function handleDeleteGroup(groupId: number) {
    try {
      await invoke("remove_group", { groupId });
      setActiveGroupId((prev) => (prev === groupId ? null : prev));
      await Promise.all([loadCustomGroups(), loadRecords()]);
      toaster.create({ title: "分组已移除", type: "success", duration: 2000 });
    } catch (e) {
      toaster.create({
        title: "移除分组失败",
        description: String(e),
        type: "error",
        duration: 2600,
      });
    }
  }

  async function handleAddRecordGroup(recordId: number, groupId: number) {
    try {
      await invoke("add_record_group", { recordId, groupId });
      await loadRecords();
    } catch (e) {
      toaster.create({
        title: "加入分组失败",
        description: String(e),
        type: "error",
        duration: 2600,
      });
    }
  }

  async function handleRemoveRecordGroup(recordId: number, groupId: number) {
    try {
      await invoke("remove_record_group", { recordId, groupId });
      await loadRecords();
    } catch (e) {
      toaster.create({
        title: "移除分组失败",
        description: String(e),
        type: "error",
        duration: 2600,
      });
    }
  }

  useEffect(() => {
    void init();

    let unlisten: (() => void) | undefined;
    listen("clipboard-new-record", () => {
      void loadRecords();
      void loadStats();
    })
      .then((cleanup) => {
        unlisten = cleanup;
      })
      .catch(console.error);

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const emptyStateText = keyword.trim()
    ? "没有匹配的记录，请更换关键词重试。"
    : "当前没有任何记录，请先复制一些内容到剪贴板。";

  return (
    <>
      <AppToaster />
      <AppShell
        page={page}
        onSwitch={(nextPage) => {
          setPage(nextPage);
          if (nextPage === "settings") {
            void loadStorageSettings();
            void loadAutoStartSettings();
            void loadRecordExpirationSettings();
            void loadCustomGroups();
          }
        }}
      >
        {page === "home" ? (
          <HomePage
            statsTotal={stats.total_records}
            keyword={keyword}
            filterType={filterType}
            onFilterChange={setFilterType}
            onKeywordChange={setKeyword}
            onSearch={() => void loadRecords()}
            onReset={() => {
              setKeyword("");
              setFilterType("all");
              setActiveGroupId(null);
              void loadRecords();
            }}
            loading={loading}
            error={error}
            records={filteredRecords}
            emptyStateText={emptyStateText}
            onDelete={(id) => void handleDelete(id)}
            onToggleFavorite={(id, isFavorite) => void handleToggleFavorite(id, isFavorite)}
            customGroups={customGroups}
            activeGroupId={activeGroupId}
            onGroupFilterChange={setActiveGroupId}
            newGroupName={newGroupName}
            onNewGroupNameChange={setNewGroupName}
            onCreateGroup={() => void handleCreateGroup()}
            onDeleteGroup={(groupId) => void handleDeleteGroup(groupId)}
            onAddRecordGroup={(recordId, groupId) => void handleAddRecordGroup(recordId, groupId)}
            onRemoveRecordGroup={(recordId, groupId) => void handleRemoveRecordGroup(recordId, groupId)}
          />
        ) : (
          <SettingsPage
            settings={storageSettings}
            autoStartSettings={autoStartSettings}
            recordExpirationSettings={recordExpirationSettings}
            savingAutoStart={savingAutoStart}
            savingRecordExpiration={savingRecordExpiration}
            onToggleAutoStart={(nextEnabled) => void handleToggleAutoStart(nextEnabled)}
            onToggleRecordExpiration={(nextEnabled) => void handleToggleRecordExpiration(nextEnabled)}
            onUpdateExpirationDays={(days) => void handleUpdateExpirationDays(days)}
          />
        )}
      </AppShell>
    </>
  );
}

export default App;
