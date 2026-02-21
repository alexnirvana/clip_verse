import { ChakraProvider, defaultSystem } from "@chakra-ui/react";

export const Provider = ({ children }: { children: React.ReactNode }) => {
  return <ChakraProvider value={defaultSystem}>{children}</ChakraProvider>;
};
