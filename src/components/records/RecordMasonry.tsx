import { Box, Text } from "@chakra-ui/react";
import { RecordCard } from "@/components/records/RecordCard";
import type { ClipboardRecord } from "@/types/clipboard";

type Props = {
  records: ClipboardRecord[];
  emptyStateText: string;
  onDelete: (id: number) => void;
  onToggleFavorite: (id: number, isFavorite: boolean) => void;
};

export const RecordMasonry = ({ records, emptyStateText, onDelete, onToggleFavorite }: Props) => {
  if (records.length === 0) {
    return (
      <Text color="gray.500" py={8} textAlign="center">
        {emptyStateText}
      </Text>
    );
  }

  return (
    <Box className="masonry-wrap">
      {records.map((record) => (
        <Box key={record.id} className="masonry-item">
          <RecordCard record={record} onDelete={onDelete} onToggleFavorite={onToggleFavorite} />
        </Box>
      ))}
    </Box>
  );
};
