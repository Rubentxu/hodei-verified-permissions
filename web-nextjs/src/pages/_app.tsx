import "@/styles/globals.css";
import type { AppProps } from "next/app";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { useState } from "react";
import { MotionConfig } from "framer-motion";
import PolicyStoreOrchestrator from "@/components/PolicyStoreOrchestrator";

export default function App({ Component, pageProps }: AppProps) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            // With SSR, we usually want to set some default staleTime
            // above 0 to avoid refetching immediately on the client
            staleTime: 60 * 1000, // 1 minute
            retry: 2,
            refetchOnWindowFocus: false,
          },
          mutations: {
            retry: 1,
          },
        },
      }),
  );

  return (
    <MotionConfig reducedMotion="always">
      <QueryClientProvider client={queryClient}>
        <Component {...pageProps} />
        <PolicyStoreOrchestrator />
        <ReactQueryDevtools initialIsOpen={false} />
      </QueryClientProvider>
    </MotionConfig>
  );
}
