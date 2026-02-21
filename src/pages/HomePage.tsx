import { Box, Button, Flex, Heading, Input, SegmentGroup, Spinner, Text } from "@chakra-ui/react";
import { RecordMasonry } from "@/components/records/RecordMasonry";
import type { ClipboardRecord, CustomGroup, RecordFilterType } from "@/types/clipboard";

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
  customGroups: CustomGroup[];
  activeGroupId: number | null;
  onGroupFilterChange: (groupId: number | null) => void;
  newGroupName: string;
  onNewGroupNameChange: (value: string) => void;
  onCreateGroup: () => void;
  onDeleteGroup: (groupId: number) => void;
  onAddRecordGroup: (recordId: number, groupId: number) => void;
  onRemoveRecordGroup: (recordId: number, groupId: number) => void;
};

const filterOptions: Array<{ label: string; value: RecordFilterType }> = [
  { label: "全部", value: "all" },
  { label: "图片", value: "image" },
  { label: "文件", value: "file" },
  { label: "文本", value: "text" },
  { label: "收藏", value: "favorite" },
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
  customGroups,
  activeGroupId,
  onGroupFilterChange,
  newGroupName,
  onNewGroupNameChange,
  onCreateGroup,
  onDeleteGroup,
  onAddRecordGroup,
  onRemoveRecordGroup,
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

      <Flex className="panel-flat" p={5} justify="space-between" align="center" gap={4} wrap="wrap" mt={4}>
        <Flex gap={2} wrap="wrap" align="center">
          <Button
            size="sm"
            className={`next-btn ${activeGroupId === null ? "next-btn-primary" : "next-btn-ghost"}`}
            onClick={() => onGroupFilterChange(null)}
          >
            全部分组
          </Button>
          {customGroups.map((group) => (
            <Flex key={group.id} align="center" gap={1}>
              <Button
                size="sm"
                className={`next-btn ${activeGroupId === group.id ? "next-btn-primary" : "next-btn-ghost"}`}
                onClick={() => onGroupFilterChange(group.id)}
              >
                {group.name}
              </Button>
              <Button size="xs" variant="ghost" onClick={() => onDeleteGroup(group.id)}>
                ×
              </Button>
            </Flex>
          ))}
        </Flex>

        <Flex gap={2} wrap="wrap" align="center">
          <Input
            value={newGroupName}
            onChange={(e) => onNewGroupNameChange(e.target.value)}
            placeholder="输入新分组名称"
            width="220px"
            className="flat-input"
          />
          <Button className="next-btn next-btn-primary" onClick={onCreateGroup}>
            新建分组
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
            customGroups={customGroups}
            onAddRecordGroup={onAddRecordGroup}
            onRemoveRecordGroup={onRemoveRecordGroup}
          />
        )}
      </Box>
    </>
  );
};
