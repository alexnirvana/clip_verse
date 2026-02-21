import { Box, Button, Flex, Heading, Input, SegmentGroup, Spinner, Text } from "@chakra-ui/react";
import { RecordMasonry } from "@/components/records/RecordMasonry";
import type { ClipboardRecord, RecordFilterType } from "@/types/clipboard";

type Props = {
  statsTotal: number;
  keyword: string;
  filterType: RecordFilterType;
  onFilterChange: (value: RecordFilterType) => void;
  onKeywordChange: (value: string) => void;
  onSearch: () => void;
  onReset: () => void;
  loading: boolean;
  error: string;
  records: ClipboardRecord[];
  emptyStateText: string;
  onDelete: (id: number) => void;
  onToggleFavorite: (id: number, isFavorite: boolean) => void;
};

const filterOptions: Array<{ label: string; value: RecordFilterType }> = [
  { label: "全部", value: "all" },
  { label: "图片", value: "image" },
  { label: "文件", value: "file" },
  { label: "文本", value: "text" },
];

export const HomePage = ({
  statsTotal,
  keyword,
  filterType,
  onFilterChange,
  onKeywordChange,
  onSearch,
  onReset,
  loading,
  error,
  records,
  emptyStateText,
  onDelete,
  onToggleFavorite,
}: Props) => {
  return (
    <>
      <Flex className="panel-flat" p={5} justify="space-between" align="center" gap={4} wrap="wrap">
        <Text>总记录数：{statsTotal}</Text>
        <Flex gap={2} wrap="wrap" align="center">
          <SegmentGroup.Root
            value={filterType}
            onValueChange={(e) => onFilterChange((e.value ?? "all") as RecordFilterType)}
            className="flat-segment"
            size="sm"
          >
            <SegmentGroup.Indicator />
            {filterOptions.map((item) => (
              <SegmentGroup.Item key={item.value} value={item.value}>
                <SegmentGroup.ItemText>{item.label}</SegmentGroup.ItemText>
                <SegmentGroup.ItemHiddenInput />
              </SegmentGroup.Item>
            ))}
          </SegmentGroup.Root>

          <Input
            value={keyword}
            onChange={(e) => onKeywordChange(e.target.value)}
            placeholder="按内容关键词搜索"
            width="260px"
            className="flat-input"
          />
          <Button className="next-btn next-btn-primary" onClick={onSearch} variant="solid">
            搜索
          </Button>
          <Button className="next-btn next-btn-ghost" variant="solid" onClick={onReset}>
            重置
          </Button>
        </Flex>
      </Flex>

      <Box className="panel-flat" p={6}>
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
          <RecordMasonry
            records={records}
            emptyStateText={emptyStateText}
            onDelete={onDelete}
            onToggleFavorite={onToggleFavorite}
          />
        )}
      </Box>
    </>
  );
};
