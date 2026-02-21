# Clip Verse

Clip Verse 是一个基于 **Tauri + React + Rust** 的桌面剪贴板历史管理应用。

## 当前实现（MVP）

- 文本记录入库（SQLite）
- 文本记录列表查询
- 按关键词搜索文本记录
- 删除文本记录
- 中国时区（UTC+8）时间写入
- 默认数据目录：`~/.clip_verse`

## 启动方式

```bash
npm install
npm run tauri dev
```

## 目录说明

- `src-tauri/`：Rust 后端与 Tauri 命令
- `src/`：React 前端页面
- `docs/`：需求与开发规划文档
