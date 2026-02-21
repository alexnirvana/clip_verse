export type ClipboardRecord = {
  id: number;
  content_type: string;
  timestamp: number;
  created_at: string;
  preview: string;
  content_size: number;
  content: string;
  image_path?: string;
  thumbnail_path?: string;
  file_path?: string;
  icon_path?: string;
  is_favorite: boolean;
};

export type DashboardStats = {
  total_records: number;
};

export type StorageSettings = {
  database_path: string;
  image_save_path: string;
};

export type PageType = "home" | "settings";

export type RecordFilterType = "all" | "image" | "file" | "text";
