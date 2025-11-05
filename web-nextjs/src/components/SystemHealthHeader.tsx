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
        className: "bg-yellow-100 text-yellow-800",
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
          className: "bg-green-100 text-green-800",
          showSpinner: false,
        };
      } else {
        return {
          status: "disconnected",
          label: "Disconnected",
          icon: XCircle,
          className: "bg-red-100 text-red-800",
          showSpinner: false,
        };
      }
    }

    return {
      status: "connected",
      label: "Connected",
      icon: CheckCircle,
      className: "bg-green-100 text-green-800",
      showSpinner: false,
    };
  };

  return (
    <div className="flex flex-col items-center">
      <div className="flex items-center justify-center space-x-3 flex-wrap">
        <Activity className="w-4 h-4 text-gray-700" />
        <span className="text-sm font-semibold text-gray-900">
          System Health
        </span>

        {/* Last Check */}
        <span className="text-xs text-gray-500">
          {isRefreshing
            ? "Checking..."
            : health.data?.last_check
              ? `Last: ${new Date(health.data.last_check).toLocaleTimeString()}`
              : "Never checked"}
        </span>

        {/* Separator */}
        <span className="text-gray-300">|</span>

        {/* Verified-Permissions Server Status */}
        {(() => {
          const status = getHealthStatus("grpc_server");
          const IconComponent = status.icon;
          return (
            <Badge variant="default" className={status.className}>
              <IconComponent
                className={`w-3 h-3 mr-1 ${status.showSpinner ? "animate-spin" : ""}`}
              />
              <span className="text-xs">
                {isRefreshing
                  ? "Checking..."
                  : health.data?.grpc_server === "connected"
                    ? "Verified-Permissions Server"
                    : "Verified-Permissions Server: Disconnected"}
              </span>
            </Badge>
          );
        })()}

        {/* Separator */}
        <span className="text-gray-300">|</span>

        {/* Database Status */}
        {(() => {
          const status = getHealthStatus("database");
          const IconComponent = status.icon;
          return (
            <Badge variant="default" className={status.className}>
              <IconComponent
                className={`w-3 h-3 mr-1 ${status.showSpinner ? "animate-spin" : ""}`}
              />
              <span className="text-xs">
                {isRefreshing ? "Checking..." : "DB"}
              </span>
            </Badge>
          );
        })()}

        {/* Separator */}
        <span className="text-gray-300">|</span>

        {/* Refresh Button */}
        <Button
          variant="ghost"
          size="sm"
          onClick={handleRefresh}
          disabled={isRefreshing || isLoading}
          className="h-7 px-3 text-xs hover:bg-blue-50 hover:border-blue-300 transition-all duration-200"
        >
          {isRefreshing || isLoading ? (
            <Loader2 className="w-3 h-3 mr-1 animate-spin" />
          ) : (
            <RefreshCw className="w-3 h-3 mr-1 transition-transform duration-200 hover:rotate-180" />
          )}
          <span>{isRefreshing || isLoading ? "Refreshing..." : "Refresh"}</span>
        </Button>
      </div>
    </div>
  );
};

export default SystemHealthHeader;
