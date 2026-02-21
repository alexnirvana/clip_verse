import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  Box,
  Container,
  Heading,
  Text,
  Input,
  Button,
  Flex,
  VStack,
  HStack,
  Image,
  Spinner,
  createToaster,
  Toaster as ChakraToaster,
  Portal,
  Toast,
  Stack,
} from "@chakra-ui/react";

const toaster = createToaster({
  placement: "bottom-end",
  pauseOnPageIdle: true,
});

const Toaster = () => {
  return (
    <Portal>
      <ChakraToaster toaster={toaster} insetInline={{ mdDown: "4" }}>
        {(toast) => (
          <Toast.Root width={{ md: "sm" }}>
            <Stack gap="1" flex="1" maxWidth="100%">
              {toast.title && <Toast.Title>{toast.title}</Toast.Title>}
              {toast.description && (
                <Toast.Description>{toast.description}</Toast.Description>
              )}
            </Stack>
            {toast.closable && <Toast.CloseTrigger />}
          </Toast.Root>
        )}
      </ChakraToaster>
    </Portal>
  );
};

type ClipboardRecord = {
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
};

type DashboardStats = {
  total_records: number;
};

type StorageSettings = {
  database_path: string;
  image_save_path: string;
};

function App() {
  // 格式化时间为易读格式
  const formatTime = (isoString: string) => {
    try {
      const date = new Date(isoString);
      const year = date.getFullYear();
      const month = String(date.getMonth() + 1).padStart(2, "0");
      const day = String(date.getDate()).padStart(2, "0");
      const hours = String(date.getHours()).padStart(2, "0");
      const minutes = String(date.getMinutes()).padStart(2, "0");
      const seconds = String(date.getSeconds()).padStart(2, "0");
      return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
    } catch {
      return isoString;
    }
  };

  const [page, setPage] = useState<"records" | "settings">("records");
  const [keyword, setKeyword] = useState("");
  const [records, setRecords] = useState<ClipboardRecord[]>([]);
  const [stats, setStats] = useState<DashboardStats>({ total_records: 0 });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [storageSettings, setStorageSettings] = useState<StorageSettings | null>(null);

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
    listen("clipboard-new-record", (event) => {
      console.log("收到新记录事件:", event.payload);
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
  }, [keyword]);

  const emptyStateText = keyword.trim()
    ? "没有匹配的记录，请更换关键词重试。"
    : "当前没有任何记录，请先复制一些内容到剪贴板。";

  return (
    <>
      <Toaster />
      <Container maxW="container.xl" py={8}>
        <VStack gap={6} align="stretch">
          <Flex justify="space-between" align="center" gap={4}>
            <Box>
              <Heading size="lg">Clip Verse</Heading>
              <Text color="gray.600">剪贴板历史管理</Text>
            </Box>
            <HStack gap={2}>
              <Button
                variant={page === "records" ? "solid" : "outline"}
                colorScheme="blue"
                onClick={() => setPage("records")}
              >
                记录页面
              </Button>
              <Button
                variant={page === "settings" ? "solid" : "outline"}
                colorScheme="blue"
                onClick={() => {
                  setPage("settings");
                  void loadStorageSettings();
                }}
              >
                设置页面
              </Button>
            </HStack>
          </Flex>

          {page === "records" ? (
            <>
              <Flex
                p={6}
                borderWidth="1px"
                borderRadius="lg"
                bg="white"
                shadow="sm"
                justify="space-between"
                align="center"
                gap={4}
              >
                <Text>总记录数：{stats.total_records}</Text>
                <Flex gap={2}>
                  <Input
                    value={keyword}
                    onChange={(e) => setKeyword(e.target.value)}
                    placeholder="按内容关键词搜索"
                    width="240px"
                  />
                  <Button onClick={() => void loadRecords()} colorScheme="blue">
                    搜索
                  </Button>
                  <Button
                    onClick={() => {
                      setKeyword("");
                      void loadRecords();
                    }}
                  >
                    重置
                  </Button>
                </Flex>
              </Flex>

              <Box p={6} borderWidth="1px" borderRadius="lg" bg="white" shadow="sm">
                <Heading size="md" mb={4}>
                  剪贴板记录列表
                </Heading>
                {error && (
                  <Box mb={4} p={4} bg="red.50" borderRadius="md" color="red.700">
                    {error}
                  </Box>
                )}
                {loading ? (
                  <Flex justify="center" py={8}>
                    <Spinner size="xl" />
                  </Flex>
                ) : records.length === 0 ? (
                  <Text color="gray.500" py={8} textAlign="center">
                    {emptyStateText}
                  </Text>
                ) : (
                  <VStack gap={4} align="stretch">
                    {records.map((record) => (
                      <Box
                        key={record.id}
                        p={4}
                        borderWidth="1px"
                        borderRadius="md"
                        borderColor="gray.200"
                      >
                        <Flex justify="space-between" align="center" mb={3}>
                          <HStack gap={2}>
                            <Box
                              px={2}
                              py={1}
                              bg="blue.100"
                              color="blue.700"
                              borderRadius="md"
                              fontSize="sm"
                            >
                              #{record.id}
                            </Box>
                            <Box
                              px={2}
                              py={1}
                              bg={
                                record.content_type === "text"
                                  ? "green.100"
                                  : record.content_type === "image"
                                  ? "purple.100"
                                  : "orange.100"
                              }
                              color={
                                record.content_type === "text"
                                  ? "green.700"
                                  : record.content_type === "image"
                                  ? "purple.700"
                                  : "orange.700"
                              }
                              borderRadius="md"
                              fontSize="sm"
                            >
                              {record.content_type === "text"
                                ? "文本"
                                : record.content_type === "image"
                                ? "图片"
                                : "文件"}
                            </Box>
                          </HStack>
                          <Text color="gray.500" fontSize="sm">
                            {formatTime(record.created_at)}
                          </Text>
                        </Flex>

                        {record.content_type === "text" ? (
                          <Text
                            mb={3}
                            whiteSpace="pre-wrap"
                            wordBreak="break-word"
                            bg="gray.50"
                            p={3}
                            borderRadius="md"
                          >
                            {record.content}
                          </Text>
                        ) : record.content_type === "file" ? (
                          <VStack gap={2} mb={3} align="stretch">
                            <Flex gap={3} align="center" p={3} bg="gray.50" borderRadius="md">
                              {record.icon_path ? (
                                <Image
                                  src={convertFileSrc(record.icon_path)}
                                  alt="文件图标"
                                  boxSize="48px"
                                  objectFit="contain"
                                />
                              ) : (
                                <Text fontSize="4xl">📄</Text>
                              )}
                              <VStack gap={1} align="start">
                                <Text fontWeight="bold" fontSize="lg">
                                  {record.preview.replace("文件: ", "")}
                                </Text>
                                <Text color="gray.500" fontSize="sm" wordBreak="break-all">
                                  {record.file_path || "未知路径"}
                                </Text>
                              </VStack>
                            </Flex>
                          </VStack>
                        ) : (
                          <VStack gap={2} mb={3} align="stretch">
                            <Box
                              maxW="100%"
                              borderRadius="md"
                              overflow="hidden"
                              borderWidth="1px"
                              borderColor="gray.200"
                            >
                              <Image
                                src={convertFileSrc(record.thumbnail_path || record.image_path || "")}
                                alt="剪贴板图片"
                                objectFit="contain"
                                maxH="400px"
                                width="auto"
                                height="auto"
                              />
                            </Box>
                            <Text color="gray.500" fontSize="sm">
                              {record.preview}
                            </Text>
                          </VStack>
                        )}

                        <Flex justify="space-between" align="center">
                          <Text color="gray.500" fontSize="sm">
                            大小：{record.content_size} 字节
                          </Text>
                          <Button
                            size="sm"
                            colorScheme="red"
                            onClick={() => void handleDelete(record.id)}
                          >
                            删除
                          </Button>
                        </Flex>
                      </Box>
                    ))}
                  </VStack>
                )}
              </Box>
            </>
          ) : (
            <Box p={6} borderWidth="1px" borderRadius="lg" bg="white" shadow="sm">
              <Heading size="md" mb={4}>
                设置页面
              </Heading>
              <VStack gap={4} align="stretch">
                <Box>
                  <Text fontWeight="bold" mb={1}>
                    记录存储数据库地址
                  </Text>
                  <Text color="gray.700" bg="gray.50" p={3} borderRadius="md" wordBreak="break-all">
                    {storageSettings?.database_path || "加载中..."}
                  </Text>
                </Box>
                <Box>
                  <Text fontWeight="bold" mb={1}>
                    图片保存地址
                  </Text>
                  <Text color="gray.700" bg="gray.50" p={3} borderRadius="md" wordBreak="break-all">
                    {storageSettings?.image_save_path || "加载中..."}
                  </Text>
                </Box>
              </VStack>
            </Box>
          )}
        </VStack>
      </Container>
    </>
  );
}

export default App;
