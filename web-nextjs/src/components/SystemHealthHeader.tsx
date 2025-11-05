"use client";

import { useDashboardData } from "../hooks/useDashboardMetrics";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import {
  Activity,
  CheckCircle,
  XCircle,
  RefreshCw,
  Loader2,
} from "lucide-react";
import React from "react";

const SystemHealthHeader: React.FC = () => {
  const { health, isLoading, refetch } = useDashboardData();
  const [isRefreshing, setIsRefreshing] = React.useState(false);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await refetch();
    } finally {
      setTimeout(() => {
        setIsRefreshing(false);
      }, 500);
    }
  };

  const getHealthStatus = (component: "grpc_server" | "database") => {
    if (isRefreshing) {
      return {
        status: "checking",
        label: "Checking...",
        icon: Loader2,
        className: "bg-yellow-100 text-yellow-700",
        showSpinner: true,
      };
    }

    const currentStatus = health.data?.[component];
    if (component === "grpc_server") {
      if (currentStatus === "connected") {
        return {
          status: "connected",
          label: "Connected",
          icon: CheckCircle,
          className: "bg-green-50 text-green-700 border border-green-200",
          showSpinner: false,
        };
      } else {
        return {
          status: "disconnected",
          label: "Disconnected",
          icon: XCircle,
          className: "bg-red-50 text-red-700 border border-red-200",
          showSpinner: false,
        };
      }
    }

    return {
      status: "connected",
      label: "Connected",
      icon: CheckCircle,
      className: "bg-blue-50 text-blue-700 border border-blue-200",
      showSpinner: false,
    };
  };

  return (
    <div className="flex items-center justify-center">
      <div className="flex items-center space-x-2">
        <Activity className="w-4 h-4 text-gray-600" />
        <span className="text-sm font-medium text-gray-800">System Health</span>

        <span className="text-gray-300">•</span>

        <span className="text-xs text-gray-500">
          {isRefreshing
            ? "Checking..."
            : health.data?.last_check
              ? `${new Date(health.data.last_check).toLocaleTimeString()}`
              : "Never"}
        </span>

        <span className="text-gray-300">•</span>

        {(() => {
          const status = getHealthStatus("grpc_server");
          const IconComponent = status.icon;
          return (
            <div
              className={`inline-flex items-center px-2 py-1 rounded-full ${status.className}`}
            >
              <IconComponent
                className={`w-3 h-3 mr-1 ${status.showSpinner ? "animate-spin" : ""}`}
              />
              <span className="text-xs font-medium">Server</span>
            </div>
          );
        })()}

        {(() => {
          const status = getHealthStatus("database");
          const IconComponent = status.icon;
          return (
            <div
              className={`inline-flex items-center px-2 py-1 rounded-full ${status.className}`}
            >
              <IconComponent
                className={`w-3 h-3 mr-1 ${status.showSpinner ? "animate-spin" : ""}`}
              />
              <span className="text-xs font-medium">DB</span>
            </div>
          );
        })()}

        <span className="text-gray-300">•</span>

        <Button
          variant="ghost"
          size="sm"
          onClick={handleRefresh}
          disabled={isRefreshing || isLoading}
          className="h-6 px-2 text-xs text-gray-600 hover:text-gray-900 hover:bg-gray-100"
        >
          {isRefreshing || isLoading ? (
            <Loader2 className="w-3 h-3 mr-1 animate-spin" />
          ) : (
            <RefreshCw className="w-3 h-3 mr-1" />
          )}
          <span>Refresh</span>
        </Button>
      </div>
    </div>
  );
};

export default SystemHealthHeader;
