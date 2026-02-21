import { Button, HStack } from "@chakra-ui/react";
import type { PageType } from "@/types/clipboard";

type Props = {
  page: PageType;
  onSwitch: (page: PageType) => void;
};

export const AppMenu = ({ page, onSwitch }: Props) => {
  return (
    <HStack gap={2}>
      <Button
        className="neon-btn"
        variant={page === "home" ? "solid" : "outline"}
        onClick={() => onSwitch("home")}
      >
        首页
      </Button>
      <Button
        className="neon-btn"
        variant={page === "settings" ? "solid" : "outline"}
        onClick={() => onSwitch("settings")}
      >
        设置
      </Button>
    </HStack>
  );
};
