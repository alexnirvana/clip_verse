import { Box, Button, Heading, HStack, Text, VStack } from "@chakra-ui/react";
import type { AutoStartSettings, StorageSettings } from "@/types/clipboard";

type Props = {
  settings: StorageSettings | null;
  autoStartSettings: AutoStartSettings | null;
  savingAutoStart: boolean;
  onToggleAutoStart: (nextEnabled: boolean) => void;
};

export const SettingsPage = ({
  settings,
  autoStartSettings,
  savingAutoStart,
  onToggleAutoStart,
}: Props) => {
  const enabled = autoStartSettings?.auto_start_enabled ?? false;

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
        <Box>
          <Text fontWeight="bold" mb={2} color="gray.700">
            系统启动时运行
          </Text>
          <HStack justify="space-between" gap={3}>
            <Text color="gray.600">当前状态：{enabled ? "已开启" : "已关闭（默认）"}</Text>
            <Button
              size="sm"
              colorPalette={enabled ? "orange" : "teal"}
              loading={savingAutoStart}
              onClick={() => onToggleAutoStart(!enabled)}
            >
              {enabled ? "关闭" : "开启"}
            </Button>
          </HStack>
        </Box>
      </VStack>
    </Box>
  );
};
