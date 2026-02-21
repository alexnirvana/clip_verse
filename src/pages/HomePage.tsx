import { Box, Button, Flex, Heading, Input, Spinner, Text } from "@chakra-ui/react";
import { RecordMasonry } from "@/components/records/RecordMasonry";
import type { ClipboardRecord } from "@/types/clipboard";

type Props = {
  statsTotal: number;
  keyword: string;
  onKeywordChange: (value: string) => void;
  onSearch: () => void;
  onReset: () => void;
  loading: boolean;
  error: string;
  records: ClipboardRecord[];
  emptyStateText: string;
  onDelete: (id: number) => void;
};

export const HomePage = ({
  statsTotal,
  keyword,
  onKeywordChange,
  onSearch,
  onReset,
  loading,
  error,
  records,
  emptyStateText,
  onDelete,
}: Props) => {
  return (
    <>
      <Flex className="panel-glow" p={5} justify="space-between" align="center" gap={4} wrap="wrap">
        <Text>总记录数：{statsTotal}</Text>
        <Flex gap={2} wrap="wrap">
          <Input
            value={keyword}
            onChange={(e) => onKeywordChange(e.target.value)}
            placeholder="按内容关键词搜索"
            width="280px"
            className="neon-input"
          />
          <Button className="neon-btn" onClick={onSearch}>
            搜索
          </Button>
          <Button className="neon-btn" variant="outline" onClick={onReset}>
            重置
          </Button>
        </Flex>
      </Flex>

      <Box className="panel-glow" p={6}>
        <Heading size="md" mb={4}>
          时间序 · 记录瀑布流
        </Heading>

        {error && (
          <Box mb={4} p={4} bg="red.900" borderRadius="md" color="red.100" border="1px solid" borderColor="red.500">
            {error}
          </Box>
        )}

        {loading ? (
          <Flex justify="center" py={8}>
            <Spinner size="xl" color="cyan.300" />
          </Flex>
        ) : (
          <RecordMasonry records={records} emptyStateText={emptyStateText} onDelete={onDelete} />
        )}
      </Box>
    </>
  );
};
