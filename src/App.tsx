import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { AppToaster, toaster } from "@/components/common/AppToaster";
import { AppShell } from "@/components/layout/AppShell";
import { HomePage } from "@/pages/HomePage";
import { SettingsPage } from "@/pages/SettingsPage";
import type { ClipboardRecord, DashboardStats, PageType, StorageSettings } from "@/types/clipboard";

function App() {
  const [page, setPage] = useState<PageType>("home");
  const [keyword, setKeyword] = useState("");
  const [records, setRecords] = useState<ClipboardRecord[]>([]);
  const [stats, setStats] = useState<DashboardStats>({ total_records: 0 });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [storageSettings, setStorageSettings] = useState<StorageSettings | null>(null);

  const sortedRecords = useMemo(
    () => [...records].sort((a, b) => b.timestamp - a.timestamp),
    [records],
  );

  async function init() {
    try {
      await invoke("init_app");
      await Promise.all([loadRecords(), loadStats(), loadStorageSettings()]);
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
    // 仅在首次挂载时初始化与绑定事件
    // eslint-disable-next-line react-hooks/exhaustive-deps
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
          }
        }}
      >
        {page === "home" ? (
          <HomePage
            statsTotal={stats.total_records}
            keyword={keyword}
            onKeywordChange={setKeyword}
            onSearch={() => void loadRecords()}
            onReset={() => {
              setKeyword("");
              void loadRecords();
            }}
            loading={loading}
            error={error}
            records={sortedRecords}
            emptyStateText={emptyStateText}
            onDelete={(id) => void handleDelete(id)}
          />
        ) : (
          <SettingsPage settings={storageSettings} />
        )}
      </AppShell>
    </>
  );
}

export default App;
