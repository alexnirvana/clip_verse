import { Box, Button, Flex, HStack, IconButton, Image, Text, VStack } from "@chakra-ui/react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { formatTime } from "@/lib/time";
import type { ClipboardRecord } from "@/types/clipboard";

type Props = {
  record: ClipboardRecord;
  onDelete: (id: number) => void;
  onToggleFavorite: (id: number, isFavorite: boolean) => void;
};

const typeLabelMap: Record<string, string> = {
  text: "文本",
  image: "图片",
  file: "文件",
};

export const RecordCard = ({ record, onDelete, onToggleFavorite }: Props) => {
  const isText = record.content_type === "text";
  const isImage = record.content_type === "image";

  return (
    <Box className="record-card" data-type={record.content_type}>
      <Flex justify="space-between" align="start" mb={3}>
        <HStack gap={2}>
          <Box className="record-tag">#{record.id}</Box>
          <Box className="record-type">{typeLabelMap[record.content_type] || "未知"}</Box>
        </HStack>
        <IconButton
          size="xs"
          variant="ghost"
          aria-label={record.is_favorite ? "取消收藏" : "收藏"}
          className={`favorite-btn ${record.is_favorite ? "is-active" : ""}`}
          onClick={() => onToggleFavorite(record.id, !record.is_favorite)}
        >
          {record.is_favorite ? "♥" : "♡"}
        </IconButton>
      </Flex>

      <Text color="gray.500" fontSize="xs" mb={3}>
        {formatTime(record.created_at)}
      </Text>

      {isText ? (
        <Text className="record-text">{record.content}</Text>
      ) : isImage ? (
        <VStack gap={2} align="stretch" mb={3}>
          <Box borderRadius="md" overflow="hidden" borderWidth="1px" borderColor="gray.200">
            <Image
              src={convertFileSrc(record.thumbnail_path || record.image_path || "")}
              alt="剪贴板图片"
              objectFit="contain"
              maxH="360px"
              w="100%"
            />
          </Box>
          <Text color="gray.600" fontSize="sm">
            {record.preview}
          </Text>
        </VStack>
      ) : (
        <VStack gap={2} align="stretch" mb={3}>
          <Flex gap={3} align="center" p={3} bg="gray.50" borderRadius="md">
            {record.icon_path ? (
              <Image src={convertFileSrc(record.icon_path)} alt="文件图标" boxSize="42px" objectFit="contain" />
            ) : (
              <Text fontSize="2xl">📄</Text>
            )}
            <VStack gap={1} align="start">
              <Text fontWeight="bold" fontSize="sm">
                {record.preview.replace("文件: ", "")}
              </Text>
              <Text color="gray.600" fontSize="xs" wordBreak="break-all">
                {record.file_path || "未知路径"}
              </Text>
            </VStack>
          </Flex>
        </VStack>
      )}

      <Flex justify="space-between" align="center" mt={2}>
        <Text color="gray.600" fontSize="xs">
          大小：{record.content_size} 字节
        </Text>
        <Button size="xs" className="next-btn next-btn-danger" variant="solid" onClick={() => onDelete(record.id)}>
          删除
        </Button>
      </Flex>
    </Box>
  );
};
