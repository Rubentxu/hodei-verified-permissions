"use client";

import React from "react";
import { useRouter } from "next/navigation";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "./ui/card";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import {
  Activity,
  Shield,
  FileText,
  Code,
  Layers,
  CheckCircle,
  XCircle,
  RefreshCw,
  Loader2,
} from "lucide-react";
import { useDashboardData } from "../hooks/useDashboardMetrics";

const Dashboard: React.FC = () => {
  const router = useRouter();
  const { metrics, activity, health, isLoading, isError, error, refetch } =
    useDashboardData();
  const [isRefreshing, setIsRefreshing] = React.useState(false);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await refetch();
    } finally {
      // Mantener el estado de refreshing por al menos 500ms para que sea visible
      setTimeout(() => {
        setIsRefreshing(false);
      }, 500);
    }
  };

  // Determinar estado visual del System Health
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

    // Para database (por ahora siempre conectado)
    return {
      status: "connected",
      label: "Connected",
      icon: CheckCircle,
      className: "bg-green-100 text-green-800",
      showSpinner: false,
    };
  };

  const handleMetricClick = (section: string) => {
    const routes: Record<string, string> = {
      policies: "/policies",
      schemas: "/schemas",
      templates: "/templates",
      "policy-stores": "/policy-stores",
    };
    router.push(routes[section] || "/");
  };

  if (isError) {
    return (
      <div className="space-y-6">
        <Card className="border-red-200">
          <CardHeader>
            <CardTitle className="text-red-600">
              Error Loading Dashboard
            </CardTitle>
            <CardDescription>
              {error?.message || "Failed to load dashboard data"}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Button onClick={handleRefresh} variant="outline">
              <RefreshCw className="w-4 h-4 mr-2" />
              Retry
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Metrics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <Card
          className="cursor-pointer hover:shadow-lg transition-shadow"
          onClick={() => handleMetricClick("policy-stores")}
        >
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Policy Stores</CardTitle>
            <Shield className="w-6 h-6" />
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="text-2xl font-bold flex items-center">
                <Loader2 className="w-5 h-5 mr-2 animate-spin" />
                Loading...
              </div>
            ) : (
              <>
                <div className="text-2xl font-bold">
                  {metrics.data?.metrics?.policyStores?.total ?? 0}
                </div>
                <p className="text-xs text-muted-foreground">
                  Total policy stores
                </p>
                <div className="flex items-center mt-2">
                  <Badge
                    variant={
                      metrics.data?.metrics?.policyStores?.trend?.isPositive
                        ? "default"
                        : "secondary"
                    }
                  >
                    {metrics.data?.metrics?.policyStores?.trend?.isPositive
                      ? "+"
                      : ""}
                    {metrics.data?.metrics?.policyStores?.trend?.value ?? 0}%
                  </Badge>
                  <span className="text-xs text-muted-foreground ml-2">
                    from last week
                  </span>
                </div>
              </>
            )}
          </CardContent>
        </Card>

        <Card
          className="cursor-pointer hover:shadow-lg transition-shadow"
          onClick={() => handleMetricClick("policies")}
        >
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Policies</CardTitle>
            <Code className="w-6 h-6" />
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="text-2xl font-bold flex items-center">
                <Loader2 className="w-5 h-5 mr-2 animate-spin" />
                Loading...
              </div>
            ) : (
              <>
                <div className="text-2xl font-bold">
                  {metrics.data?.metrics?.policies?.total ?? 0}
                </div>
                <p className="text-xs text-muted-foreground">Active policies</p>
                <div className="flex items-center mt-2">
                  <Badge
                    variant={
                      metrics.data?.metrics?.policies?.trend?.isPositive
                        ? "default"
                        : "secondary"
                    }
                  >
                    {metrics.data?.metrics?.policies?.trend?.isPositive
                      ? "+"
                      : ""}
                    {metrics.data?.metrics?.policies?.trend?.value ?? 0}%
                  </Badge>
                  <span className="text-xs text-muted-foreground ml-2">
                    from last week
                  </span>
                </div>
              </>
            )}
          </CardContent>
        </Card>

        <Card
          className="cursor-pointer hover:shadow-lg transition-shadow"
          onClick={() => handleMetricClick("schemas")}
        >
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Schemas</CardTitle>
            <FileText className="w-6 h-6" />
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="text-2xl font-bold flex items-center">
                <Loader2 className="w-5 h-5 mr-2 animate-spin" />
                Loading...
              </div>
            ) : (
              <>
                <div className="text-2xl font-bold">
                  {metrics.data?.metrics?.schemas?.total ?? 0}
                </div>
                <p className="text-xs text-muted-foreground">Entity schemas</p>
                <div className="flex items-center mt-2">
                  <Badge
                    variant={
                      metrics.data?.metrics?.schemas?.trend?.isPositive
                        ? "default"
                        : "secondary"
                    }
                  >
                    {metrics.data?.metrics?.schemas?.trend?.isPositive
                      ? "+"
                      : ""}
                    {metrics.data?.metrics?.schemas?.trend?.value ?? 0}%
                  </Badge>
                  <span className="text-xs text-muted-foreground ml-2">
                    from last week
                  </span>
                </div>
              </>
            )}
          </CardContent>
        </Card>

        <Card
          className="cursor-pointer hover:shadow-lg transition-shadow"
          onClick={() => handleMetricClick("templates")}
        >
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Templates</CardTitle>
            <Layers className="w-6 h-6" />
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="text-2xl font-bold flex items-center">
                <Loader2 className="w-5 h-5 mr-2 animate-spin" />
                Loading...
              </div>
            ) : (
              <>
                <div className="text-2xl font-bold">
                  {metrics.data?.metrics?.templates?.total ?? 0}
                </div>
                <p className="text-xs text-muted-foreground">
                  Policy templates
                </p>
                <div className="flex items-center mt-2">
                  <Badge
                    variant={
                      metrics.data?.metrics?.templates?.trend?.isPositive
                        ? "default"
                        : "secondary"
                    }
                  >
                    {metrics.data?.metrics?.templates?.trend?.isPositive
                      ? "+"
                      : ""}
                    {metrics.data?.metrics?.templates?.trend?.value ?? 0}%
                  </Badge>
                  <span className="text-xs text-muted-foreground ml-2">
                    from last week
                  </span>
                </div>
              </>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Charts Placeholder */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle>Authorization Requests</CardTitle>
            <CardDescription>
              Daily authorization request volume
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="h-[300px] flex items-center justify-center text-gray-500">
              Chart will be displayed here
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Authorization Decisions</CardTitle>
            <CardDescription>
              Breakdown of allow vs deny decisions
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="h-[300px] flex items-center justify-center text-gray-500">
              Chart will be displayed here
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Recent Activity */}
      <Card>
        <CardHeader>
          <CardTitle>Recent Activity</CardTitle>
          <CardDescription>
            Latest system activities and changes
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-6 h-6 animate-spin" />
              <span className="ml-2">Loading activity...</span>
            </div>
          ) : activity.data?.activities &&
            activity.data.activities.length > 0 ? (
            <div className="space-y-4">
              {activity.data.activities.slice(0, 5).map((activityItem) => (
                <div
                  key={activityItem.id}
                  className="flex items-start space-x-4 pb-4 border-b last:border-0"
                >
                  <div className="flex-shrink-0">
                    <Badge variant="outline">{activityItem.type}</Badge>
                  </div>
                  <div className="flex-1">
                    <p className="text-sm font-medium">
                      {activityItem.description}
                    </p>
                    <p className="text-xs text-muted-foreground">
                      {activityItem.user} •{" "}
                      {new Date(activityItem.timestamp).toLocaleString()}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-8 text-gray-500">
              No recent activity
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
};

export default Dashboard;
