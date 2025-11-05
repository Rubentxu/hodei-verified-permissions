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
  Database,
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
      icon: Database,
      className: "bg-blue-100 text-blue-800",
      showSpinner: false,
    };
  };

  return (
    <div className="bg-gray-50 rounded-lg p-3 border border-gray-200">
      <div className="flex items-center justify-center">
        <div className="flex items-center space-x-4">
          {/* System Health Label */}
          <div className="flex items-center space-x-2 bg-white px-3 py-1 rounded-full border border-gray-200">
            <Activity className="w-4 h-4 text-indigo-600" />
            <span className="text-sm font-semibold text-gray-900">
              System Health
            </span>
          </div>

          {/* Divider */}
          <div className="h-6 w-px bg-gray-300" />

          {/* Last Check */}
          <div className="bg-white px-3 py-1 rounded-full border border-gray-200">
            <span className="text-xs text-gray-600">
              {isRefreshing
                ? "Checking..."
                : health.data?.last_check
                  ? `Last: ${new Date(health.data.last_check).toLocaleTimeString()}`
                  : "Never checked"}
            </span>
          </div>

          {/* Divider */}
          <div className="h-6 w-px bg-gray-300" />

          {/* Verified-Permissions Server Status */}
          {(() => {
            const status = getHealthStatus("grpc_server");
            const IconComponent = status.icon;
            return (
              <Badge variant="default" className={status.className}>
                <IconComponent
                  className={`w-3 h-3 mr-1 ${status.showSpinner ? "animate-spin" : ""}`}
                />
                <span className="text-xs font-medium">
                  {isRefreshing
                    ? "Checking..."
                    : health.data?.grpc_server === "connected"
                      ? "Verified-Permissions Server"
                      : "Verified-Permissions Server: Disconnected"}
                </span>
              </Badge>
            );
          })()}

          {/* Database Status */}
          {(() => {
            const status = getHealthStatus("database");
            const IconComponent = status.icon;
            return (
              <Badge variant="default" className={status.className}>
                <IconComponent
                  className={`w-3 h-3 mr-1 ${status.showSpinner ? "animate-spin" : ""}`}
                />
                <span className="text-xs font-medium">
                  {isRefreshing ? "Checking..." : "Database"}
                </span>
              </Badge>
            );
          })()}

          {/* Divider */}
          <div className="h-6 w-px bg-gray-300" />

          {/* Refresh Button */}
          <Button
            variant="outline"
            size="sm"
            onClick={handleRefresh}
            disabled={isRefreshing || isLoading}
            className="bg-white hover:bg-indigo-50 border-indigo-200 hover:border-indigo-300 transition-all duration-200 shadow-sm"
          >
            {isRefreshing || isLoading ? (
              <Loader2 className="w-3 h-3 mr-1 animate-spin text-indigo-600" />
            ) : (
              <RefreshCw className="w-3 h-3 mr-1 transition-transform duration-200 hover:rotate-180 text-indigo-600" />
            )}
            <span className="text-xs font-medium text-gray-700">
              {isRefreshing || isLoading ? "Refreshing..." : "Refresh"}
            </span>
          </Button>
        </div>
      </div>
    </div>
  );
};

export default SystemHealthHeader;
