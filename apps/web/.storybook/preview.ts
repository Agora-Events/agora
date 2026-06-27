import type { Preview } from "@storybook/react";
import "../app/globals.css";

const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    backgrounds: {
      default: "agora-base",
      values: [
        { name: "agora-base", value: "#FFFBE9" },
        { name: "white", value: "#ffffff" },
        { name: "dark", value: "#0B151F" },
      ],
    },
    nextjs: {
      appDirectory: true,
    },
  },
};

export default preview;
