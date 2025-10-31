'use client';

import {
  Code,
  FileText,
  Layers,
  LayoutDashboard,
  Menu,
  Settings as SettingsIcon,
  Shield,
  TestTube,
  Users,
  X,
} from 'lucide-react';
import Link from 'next/link';
import { useRouter } from 'next/router';
import { PropsWithChildren, useState } from 'react';

const nav = [
  { href: '/dashboard', label: 'Dashboard', icon: <LayoutDashboard className="w-5 h-5" /> },
  { href: '/policy-stores', label: 'Policy Stores', icon: <Shield className="w-5 h-5" /> },
  { href: '/schemas', label: 'Schemas', icon: <FileText className="w-5 h-5" /> },
  { href: '/policies', label: 'Policies', icon: <Code className="w-5 h-5" /> },
  { href: '/templates', label: 'Templates', icon: <Layers className="w-5 h-5" /> },
  { href: '/playground', label: 'Authorization Playground', icon: <TestTube className="w-5 h-5" /> },
  { href: '/identity-sources', label: 'Identity Sources', icon: <Users className="w-5 h-5" /> },
  { href: '/settings', label: 'Settings', icon: <SettingsIcon className="w-5 h-5" /> },
];

export default function AppLayout({ children }: PropsWithChildren) {
  const [open, setOpen] = useState(true);
  const router = useRouter();
  const active = (href: string) => router.pathname.startsWith(href);

  return (
    <div className="flex h-screen bg-gray-50 text-gray-900">
      <aside className={`${open ? 'w-64' : 'w-16'} bg-white border-r border-gray-200 transition-all duration-300 ease-in-out flex flex-col`}>
        <div className="flex items-center justify-between p-4 border-b border-gray-200">
          {open && <h1 className="text-lg font-semibold">Hodei Verified Permissions</h1>}
          <button onClick={() => setOpen(!open)} className="p-2 rounded-md hover:bg-gray-100" aria-label="Toggle sidebar">
            {open ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
          </button>
        </div>
        <nav className="p-3 flex-1 overflow-auto">
          <ul className="space-y-1">
            {nav.map((item) => (
              <li key={item.href}>
                <Link href={item.href} className={`w-full flex items-center ${open ? 'space-x-3 justify-start' : 'justify-center'} p-2 rounded-md transition-colors text-sm ${active(item.href) ? 'bg-blue-50 text-blue-700 border-l-4 border-blue-700' : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'}`}>
                  {item.icon}
                  {open && <span className="font-medium">{item.label}</span>}
                </Link>
              </li>
            ))}
          </ul>
        </nav>
        <div className="p-3 text-xs text-gray-500 border-t border-gray-200">{open ? <p>v1.0.0</p> : <span className="block text-center">v1</span>}</div>
      </aside>

      <div className="flex-1 flex flex-col overflow-hidden">
        <header className="bg-white border-b border-gray-200 px-6 py-4">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-2xl font-bold">{nav.find((n) => active(n.href))?.label ?? 'Hodei Verified Permissions'}</h2>
              <p className="text-sm text-gray-600 mt-1">Manage your verified permissions</p>
            </div>
            <div className="flex items-center space-x-3 text-sm">
              <span className="inline-flex items-center">
                <span className="w-2 h-2 bg-green-500 rounded-full mr-2" /> Connected
              </span>
            </div>
          </div>
        </header>
        <main className="flex-1 overflow-auto p-6">{children}</main>
      </div>
    </div>
  );
}
