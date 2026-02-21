import { Box, Button, Heading, HStack, Text, VStack } from "@chakra-ui/react";
import type {
  AutoStartSettings,
  RecordExpirationSettings,
  StorageSettings,
} from "@/types/clipboard";

type Props = {
  settings: StorageSettings | null;
  autoStartSettings: AutoStartSettings | null;
  recordExpirationSettings: RecordExpirationSettings | null;
  savingAutoStart: boolean;
  savingRecordExpiration: boolean;
  onToggleAutoStart: (nextEnabled: boolean) => void;
  onToggleRecordExpiration: (nextEnabled: boolean) => void;
};

export const SettingsPage = ({
  settings,
  autoStartSettings,
  recordExpirationSettings,
  savingAutoStart,
  savingRecordExpiration,
  onToggleAutoStart,
  onToggleRecordExpiration,
}: Props) => {
  const autoStartEnabled = autoStartSettings?.auto_start_enabled ?? false;
  const expirationEnabled = recordExpirationSettings?.expiration_enabled ?? false;
  const expirationDays = recordExpirationSettings?.expiration_days ?? 200;

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
          <Text fontWeight="bold" mb={1} color="gray.700">
            设置配置文件（JSON）
          </Text>
          <Text className="path-box">{settings?.settings_json_path || "加载中..."}</Text>
        </Box>
        <Box>
          <Text fontWeight="bold" mb={2} color="gray.700">
            系统启动时运行
          </Text>
          <HStack justify="space-between" gap={3}>
            <Text color="gray.600">当前状态：{autoStartEnabled ? "已开启" : "已关闭（默认）"}</Text>
            <Button
              size="sm"
              colorPalette={autoStartEnabled ? "orange" : "teal"}
              loading={savingAutoStart}
              onClick={() => onToggleAutoStart(!autoStartEnabled)}
            >
              {autoStartEnabled ? "关闭" : "开启"}
            </Button>
          </HStack>
        </Box>
        <Box>
          <Text fontWeight="bold" mb={2} color="gray.700">
            记录过期清理
          </Text>
          <HStack justify="space-between" gap={3}>
            <Text color="gray.600">
              当前状态：
              {expirationEnabled ? `已开启（保留最近 ${expirationDays} 天）` : "已关闭（默认）"}
            </Text>
            <Button
              size="sm"
              colorPalette={expirationEnabled ? "orange" : "teal"}
              loading={savingRecordExpiration}
              onClick={() => onToggleRecordExpiration(!expirationEnabled)}
            >
              {expirationEnabled ? "关闭" : "开启"}
            </Button>
          </HStack>
        </Box>
      </VStack>
    </Box>
  );
};
