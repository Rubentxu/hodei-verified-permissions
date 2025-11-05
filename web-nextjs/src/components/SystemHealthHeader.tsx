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
    <div className="mt-3 space-y-3">
      {/* Title and Description */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:space-x-3 space-y-2 sm:space-y-0">
        <div className="flex items-center space-x-2">
          <Activity className="w-5 h-5 text-gray-700" />
          <h3 className="text-lg font-semibold text-gray-900">System Health</h3>
        </div>
        <p className="text-sm text-gray-600 sm:ml-2">
          Real-time status of system components
        </p>
      </div>

      {/* Status Grid - Responsive */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 w-full">
        {/* gRPC Server Status */}
        <div className="flex items-center space-x-3">
          {(() => {
            const status = getHealthStatus("grpc_server");
            const IconComponent = status.icon;
            return (
              <Badge variant="default" className={status.className}>
                <IconComponent
                  className={`w-4 h-4 mr-1 ${status.showSpinner ? "animate-spin" : ""}`}
                />
                gRPC Server
              </Badge>
            );
          })()}
          <span className="text-sm text-gray-600">
            {isRefreshing
              ? "Checking..."
              : health.data?.grpc_server === "connected"
                ? "Connected"
                : "Disconnected"}
          </span>
        </div>

        {/* Database Status */}
        <div className="flex items-center space-x-3">
          {(() => {
            const status = getHealthStatus("database");
            const IconComponent = status.icon;
            return (
              <Badge variant="default" className={status.className}>
                <IconComponent
                  className={`w-4 h-4 mr-1 ${status.showSpinner ? "animate-spin" : ""}`}
                />
                Database
              </Badge>
            );
          })()}
          <span className="text-sm text-gray-600">
            {isRefreshing ? "Checking..." : "Connected"}
          </span>
        </div>

        {/* Last Check */}
        <div className="flex items-center space-x-3">
          <span className="text-sm text-gray-600">
            {isRefreshing
              ? "Checking connections..."
              : health.data?.last_check
                ? `Last: ${new Date(health.data.last_check).toLocaleTimeString()}`
                : "Never checked"}
          </span>
        </div>

        {/* Refresh Button */}
        <div className="flex justify-start sm:justify-end">
          <Button
            variant="outline"
            size="sm"
            onClick={handleRefresh}
            disabled={isRefreshing || isLoading}
            className="w-full sm:w-auto transition-all duration-200 hover:bg-blue-50 hover:border-blue-300 hover:shadow-md active:scale-95 disabled:hover:bg-gray-50 disabled:hover:border-gray-300"
          >
            {isRefreshing || isLoading ? (
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
            ) : (
              <RefreshCw className="w-4 h-4 mr-2 transition-transform duration-200 hover:rotate-180" />
            )}
            <span className="transition-all duration-200">
              {isRefreshing || isLoading ? "Refreshing..." : "Refresh"}
            </span>
          </Button>
        </div>
      </div>
    </div>
  );
};

export default SystemHealthHeader;
