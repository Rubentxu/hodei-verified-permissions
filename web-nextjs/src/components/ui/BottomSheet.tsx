"use client";

import React from "react";
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";

interface BottomSheetProps {
  isOpen: boolean;
  onClose: () => void;
  children: React.ReactNode;
  title: string;
  marginLeft?: number;
}

const BottomSheet: React.FC<BottomSheetProps> = ({
  isOpen,
  onClose,
  children,
  title,
  marginLeft = 0,
}) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-40" onClick={onClose}>
      <div
        className="fixed bottom-0 left-0 right-0 z-50"
        style={{
          marginLeft: `${marginLeft}px`,
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Panel with bottom-to-top animation */}
        <div
          className="bg-white"
          style={{
            transform: isOpen ? "translateY(0)" : "translateY(100%)",
            transformOrigin: "bottom",
            transition: "transform 0.7s cubic-bezier(0.25, 0.1, 0.25, 1)",
            boxShadow: isOpen
              ? "0 -20px 60px -15px rgba(0, 0, 0, 0.15), 0 -4px 20px -2px rgba(0, 0, 0, 0.08)"
              : "0 0 0 rgba(0, 0, 0, 0)",
          }}
        >
          <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-semibold text-gray-900">{title}</h3>
            <Button
              variant="ghost"
              size="sm"
              onClick={onClose}
              className="h-8 w-8 p-0 hover:bg-gray-100"
            >
              <X className="w-4 h-4" />
            </Button>
          </div>
          <div className="p-6 overflow-y-auto max-h-[80vh] min-h-[400px] bg-white">
            {children}
          </div>
        </div>
      </div>
    </div>
  );
};

export default BottomSheet;
