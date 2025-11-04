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
        className="fixed bottom-0 left-0 right-0 bg-white shadow-xl z-50 transform transition-transform duration-300 ease-in-out rounded-t-lg"
        style={{
          transform: isOpen ? "translateY(0)" : "translateY(100%)",
          marginLeft: `${marginLeft}px`,
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between p-4 border-b">
          <h3 className="text-lg font-semibold">{title}</h3>
          <Button variant="ghost" size="sm" onClick={onClose}>
            <X className="w-4 h-4" />
          </Button>
        </div>
        <div className="p-6 overflow-y-auto max-h-[80vh] min-h-[400px]">
          {children}
        </div>
      </div>
    </div>
  );
};

export default BottomSheet;
