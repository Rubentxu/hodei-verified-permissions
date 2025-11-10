import React from "react";
import dynamic from "next/dynamic";
import AppLayout from "../components/AppLayout";

// Dynamically import Settings to prevent SSR issues
const Settings = dynamic(() => import("../components/Settings"), {
  ssr: false,
  loading: () => (
    <div className="flex items-center justify-center h-64">
      <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
    </div>
  ),
});

export default function Page() {
  return (
    <AppLayout>
      <Settings />
    </AppLayout>
  );
}
