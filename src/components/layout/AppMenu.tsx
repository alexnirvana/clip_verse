import { Button, HStack } from "@chakra-ui/react";
import type { PageType } from "@/types/clipboard";

type Props = {
  page: PageType;
  onSwitch: (page: PageType) => void;
};

export const AppMenu = ({ page, onSwitch }: Props) => {
  return (
    <HStack gap={2}>
      <Button className={`next-btn ${page === "home" ? "next-btn-primary" : "next-btn-ghost"}`} variant="solid" onClick={() => onSwitch("home")}>
        首页
      </Button>
      <Button
        className={`next-btn ${page === "settings" ? "next-btn-secondary" : "next-btn-ghost"}`}
        variant="solid"
        onClick={() => onSwitch("settings")}
      >
        设置
      </Button>
    </HStack>
  );
};
