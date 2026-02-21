import { FormEvent, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type ClipboardRecord = {
  id: number;
  content_type: string;
  timestamp: number;
  created_at: string;
  preview: string;
  content_size: number;
  content: string;
};

type DashboardStats = {
  total_records: number;
};

function App() {
  const [content, setContent] = useState("");
  const [keyword, setKeyword] = useState("");
  const [records, setRecords] = useState<ClipboardRecord[]>([]);
  const [stats, setStats] = useState<DashboardStats>({ total_records: 0 });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const emptyStateText = useMemo(() => {
    if (keyword.trim()) return "没有匹配的记录，请更换关键词重试。";
    return "当前没有任何记录，请先添加一条文本。";
  }, [keyword]);

  async function init() {
    try {
      await invoke("init_app");
      await Promise.all([loadRecords(), loadStats()]);
    } catch (e) {
      setError(String(e));
    }
  }

  async function loadRecords() {
    setLoading(true);
    setError("");
    try {
      const result = await invoke<ClipboardRecord[]>("get_text_records", {
        limit: 100,
        keyword: keyword.trim() || null,
      });
      setRecords(result);
    } catch (e) {
      setError(`加载记录失败：${String(e)}`);
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

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    if (!content.trim()) {
      setError("请输入文本内容后再保存。");
      return;
    }
    try {
      await invoke("add_text_record", { content });
      setContent("");
      await Promise.all([loadRecords(), loadStats()]);
    } catch (err) {
      setError(`保存失败：${String(err)}`);
    }
  }

  async function handleDelete(id: number) {
    try {
      await invoke("remove_record", { recordId: id });
      await Promise.all([loadRecords(), loadStats()]);
    } catch (err) {
      setError(`删除失败：${String(err)}`);
    }
  }

  useEffect(() => {
    void init();
  }, []);

  return (
    <main className="app">
      <header className="header">
        <h1>Clip Verse</h1>
        <p>剪贴板历史管理（MVP：文本记录）</p>
      </header>

      <section className="panel">
        <h2>新增文本记录</h2>
        <form onSubmit={handleSubmit} className="form">
          <textarea
            value={content}
            onChange={(e) => setContent(e.target.value)}
            placeholder="请输入要保存的文本"
            rows={4}
          />
          <button type="submit">保存记录</button>
        </form>
      </section>

      <section className="panel stats-row">
        <div>总记录数：{stats.total_records}</div>
        <div className="search-wrap">
          <input
            value={keyword}
            onChange={(e) => setKeyword(e.target.value)}
            placeholder="按内容关键词搜索"
          />
          <button onClick={() => void loadRecords()} type="button">
            搜索
          </button>
          <button
            type="button"
            onClick={() => {
              setKeyword("");
              void loadRecords();
            }}
          >
            重置
          </button>
        </div>
      </section>

      <section className="panel">
        <h2>文本记录列表</h2>
        {error && <p className="error">{error}</p>}
        {loading ? (
          <p>加载中...</p>
        ) : records.length === 0 ? (
          <p>{emptyStateText}</p>
        ) : (
          <ul className="record-list">
            {records.map((record) => (
              <li key={record.id} className="record-item">
                <div className="record-head">
                  <strong>#{record.id}</strong>
                  <span>{record.created_at}</span>
                </div>
                <p className="content">{record.content}</p>
                <div className="record-meta">
                  <span>大小：{record.content_size} 字节</span>
                  <button type="button" onClick={() => void handleDelete(record.id)}>
                    删除
                  </button>
                </div>
              </li>
            ))}
          </ul>
        )}
      </section>
    </main>
  );
}

export default App;
