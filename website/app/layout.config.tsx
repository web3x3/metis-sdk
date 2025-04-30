import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";
import { Droplet } from "lucide-react";

/**
 * Shared layout configurations
 *
 * you can configure layouts individually from:
 * Home Layout: app/(home)/layout.tsx
 * Docs Layout: app/docs/layout.tsx
 */
export const baseOptions: BaseLayoutProps = {
  nav: {
    title: "Metis Hyperion Docs",
    url: "/",
    enableSearch: true,
  },
  githubUrl: "https://github.com/MetisProtocol/metis-sdk",
  links: [
    {
      type: "menu",
      text: "Explorers",
      items: [
        {
          text: "Testnet",
          url: "https://hyperion-testnet-explorer.metisdevops.link/",
        }
      ],
    },
    {
      type: "menu",
      text: "Developer Channels",
      items: [
        {
          text: "Telegram",
          url: "https://t.me/metis_dev",
        },
        {
          text: "Twitter",
          url: "https://x.com/metisdevs",
        },
        {
          text: "YouTube",
          url: "https://www.youtube.com/playlist?list=PLpYi1DzeXBB9dzWw3xk4UMbUHWlRs-i9s",
        },
      ],
    },
    {
      icon: <Droplet />,
      text: "Faucet",
      // TODO: update the faucet URL.
      url: "https://faucet.metis.io/",
    },
  ],
};
