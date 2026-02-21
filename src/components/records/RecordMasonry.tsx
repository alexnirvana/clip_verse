import { Box, Text } from "@chakra-ui/react";
import { RecordCard } from "@/components/records/RecordCard";
import type { ClipboardRecord } from "@/types/clipboard";

type Props = {
  records: ClipboardRecord[];
  emptyStateText: string;
  onDelete: (id: number) => void;
};

export const RecordMasonry = ({ records, emptyStateText, onDelete }: Props) => {
  if (records.length === 0) {
    return (
      <Text color="gray.400" py={8} textAlign="center">
        {emptyStateText}
      </Text>
    );
  }

  return (
    <Box className="masonry-wrap">
      {records.map((record) => (
        <Box
          key={record.id}
          className={`masonry-item ${record.content_type === "image" ? "is-wide" : ""} ${
            record.content_type === "text" && record.content.length > 140 ? "is-wide" : ""
          }`}
        >
          <RecordCard record={record} onDelete={onDelete} />
        </Box>
      ))}
    </Box>
  );
};
