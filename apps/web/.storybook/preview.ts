import type { Preview } from "@storybook/nextjs";
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
      options: {
        "agora-base": { name: "agora-base", value: "#FFFBE9" },
        white: { name: "white", value: "#ffffff" },
        dark: { name: "dark", value: "#0B151F" }
      }
    },
    nextjs: {
      appDirectory: true,
    },
  },

  initialGlobals: {
    backgrounds: {
      value: "agora-base"
    }
  }
};

export default preview;
