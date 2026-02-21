import { Box, Heading, Text, VStack } from "@chakra-ui/react";
import type { StorageSettings } from "@/types/clipboard";

type Props = {
  settings: StorageSettings | null;
};

export const SettingsPage = ({ settings }: Props) => {
  return (
    <Box className="panel-flat" p={6}>
      <Heading size="md" mb={4}>
        设置页面
      </Heading>
      <VStack gap={4} align="stretch">
        <Box>
          <Text fontWeight="bold" mb={1} color="gray.700">
            记录存储数据库地址
          </Text>
          <Text className="path-box">{settings?.database_path || "加载中..."}</Text>
        </Box>
        <Box>
          <Text fontWeight="bold" mb={1} color="gray.700">
            图片保存地址
          </Text>
          <Text className="path-box">{settings?.image_save_path || "加载中..."}</Text>
        </Box>
      </VStack>
    </Box>
  );
};
