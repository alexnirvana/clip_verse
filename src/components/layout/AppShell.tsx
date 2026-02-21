import type { ReactNode } from "react";
import { Box, Container, Flex, Heading, Text, VStack } from "@chakra-ui/react";
import { AppMenu } from "@/components/layout/AppMenu";
import type { PageType } from "@/types/clipboard";

type Props = {
  page: PageType;
  onSwitch: (page: PageType) => void;
  children: ReactNode;
};

export const AppShell = ({ page, onSwitch, children }: Props) => {
  return (
    <Container maxW="container.2xl" py={8}>
      <VStack gap={6} align="stretch">
        <Flex justify="space-between" align="center" gap={4} className="app-header">
          <Box>
            <Heading size="lg" className="app-title">
              Clip Verse
            </Heading>
            <Text className="app-subtitle">暗黑彩色扁平工作台</Text>
          </Box>
          <AppMenu page={page} onSwitch={onSwitch} />
        </Flex>
        {children}
      </VStack>
    </Container>
  );
};
