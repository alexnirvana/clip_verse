import { Box, Button, Flex, HStack, Image, Text, VStack } from "@chakra-ui/react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { formatTime } from "@/lib/time";
import type { ClipboardRecord } from "@/types/clipboard";

type Props = {
  record: ClipboardRecord;
  onDelete: (id: number) => void;
};

const typeLabelMap: Record<string, string> = {
  text: "文本",
  image: "图片",
  file: "文件",
};

export const RecordCard = ({ record, onDelete }: Props) => {
  const isText = record.content_type === "text";
  const isImage = record.content_type === "image";

  return (
    <Box className="record-card" data-type={record.content_type}>
      <Flex justify="space-between" align="center" mb={3}>
        <HStack gap={2}>
          <Box className="record-tag">#{record.id}</Box>
          <Box className="record-type">{typeLabelMap[record.content_type] || "未知"}</Box>
        </HStack>
        <Text color="gray.400" fontSize="xs">
          {formatTime(record.created_at)}
        </Text>
      </Flex>

      {isText ? (
        <Text className="record-text">{record.content}</Text>
      ) : isImage ? (
        <VStack gap={2} align="stretch" mb={3}>
          <Box borderRadius="md" overflow="hidden" borderWidth="1px" borderColor="whiteAlpha.200">
            <Image
              src={convertFileSrc(record.thumbnail_path || record.image_path || "")}
              alt="剪贴板图片"
              objectFit="contain"
              maxH="360px"
              w="100%"
            />
          </Box>
          <Text color="gray.400" fontSize="sm">
            {record.preview}
          </Text>
        </VStack>
      ) : (
        <VStack gap={2} align="stretch" mb={3}>
          <Flex gap={3} align="center" p={3} bg="whiteAlpha.100" borderRadius="md">
            {record.icon_path ? (
              <Image src={convertFileSrc(record.icon_path)} alt="文件图标" boxSize="42px" objectFit="contain" />
            ) : (
              <Text fontSize="2xl">📄</Text>
            )}
            <VStack gap={1} align="start">
              <Text fontWeight="bold" fontSize="sm">
                {record.preview.replace("文件: ", "")}
              </Text>
              <Text color="gray.400" fontSize="xs" wordBreak="break-all">
                {record.file_path || "未知路径"}
              </Text>
            </VStack>
          </Flex>
        </VStack>
      )}

      <Flex justify="space-between" align="center" mt={2}>
        <Text color="gray.400" fontSize="xs">
          大小：{record.content_size} 字节
        </Text>
        <Button size="xs" colorPalette="red" variant="subtle" onClick={() => onDelete(record.id)}>
          删除
        </Button>
      </Flex>
    </Box>
  );
};
