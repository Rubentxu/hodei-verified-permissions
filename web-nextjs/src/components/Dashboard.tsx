"use client";

import React from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "./ui/card";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import {
  Activity,
  Shield,
  FileText,
  Code,
  Layers,
  CheckCircle,
  RefreshCw,
} from "lucide-react";

const Dashboard: React.FC = () => {
  return (
    <div className="space-y-6">
      {/* Health Status */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center space-x-2">
                <Activity className="w-5 h-5" />
                <span>System Health</span>
              </CardTitle>
              <CardDescription>Real-time status of system components</CardDescription>
            </div>
            <Button variant="outline" size="sm">
              <RefreshCw className="w-4 h-4 mr-2" />
              Refresh
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="flex items-center space-x-3">
              <Badge variant="default">
                <CheckCircle className="w-4 h-4 mr-1" />
                gRPC Server
              </Badge>
              <span className="text-sm text-gray-600">
                Connected
              </span>
            </div>
            <div className="flex items-center space-x-3">
              <Badge variant="default">
                <CheckCircle className="w-4 h-4 mr-1" /> Database
              </Badge>
              <span className="text-sm text-gray-600">Connected</span>
            </div>
            <div className="flex items-center space-x-3">
              <span className="text-sm text-gray-600">
                Last check: Just now
              </span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Metrics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Policy Stores</CardTitle>
            <Shield className="w-6 h-6" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">0</div>
            <p className="text-xs text-muted-foreground">Total policy stores</p>
            <div className="flex items-center mt-2">
              <Badge variant="secondary">0%</Badge>
              <span className="text-xs text-muted-foreground ml-2">from last week</span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Policies</CardTitle>
            <Code className="w-6 h-6" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">156</div>
            <p className="text-xs text-muted-foreground">Active policies</p>
            <div className="flex items-center mt-2">
              <Badge variant="default">+5.4%</Badge>
              <span className="text-xs text-muted-foreground ml-2">from last week</span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Schemas</CardTitle>
            <FileText className="w-6 h-6" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">24</div>
            <p className="text-xs text-muted-foreground">Entity schemas</p>
            <div className="flex items-center mt-2">
              <Badge variant="default">+4.3%</Badge>
              <span className="text-xs text-muted-foreground ml-2">from last week</span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Templates</CardTitle>
            <Layers className="w-6 h-6" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">18</div>
            <p className="text-xs text-muted-foreground">Policy templates</p>
            <div className="flex items-center mt-2">
              <Badge variant="secondary">0%</Badge>
              <span className="text-xs text-muted-foreground ml-2">from last week</span>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Charts Placeholder */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle>Authorization Requests</CardTitle>
            <CardDescription>Daily authorization request volume</CardDescription>
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
            <CardDescription>Breakdown of allow vs deny decisions</CardDescription>
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
          <CardDescription>Latest changes in your policies and schemas</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="text-center py-8 text-muted-foreground">
            <p>No recent activity</p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default Dashboard;