'use client';

import React, { useState } from 'react';
import { Page } from '@pui/components';
import { AppHeader } from './AppHeader';
import { AppSidebar } from './AppSidebar';
import { InspectorDrawer } from './InspectorDrawer';

interface AppLayoutProps {
  currentPath: string;
  onNavigate: (path: string) => void;
  searchQuery: string;
  onSearchChange: (q: string) => void;
  children: React.ReactNode;
}

export const AppLayout: React.FC<AppLayoutProps> = ({
  currentPath,
  onNavigate,
  searchQuery,
  onSearchChange,
  children,
}) => {
  const [inspectorOpen, setInspectorOpen] = useState(false);

  return (
    <>
      <Page
        header={
          <AppHeader
            onOpenInspector={() => setInspectorOpen(true)}
            searchQuery={searchQuery}
            onSearchChange={onSearchChange}
          />
        }
        sidebar={<AppSidebar currentPath={currentPath} onNavigate={onNavigate} />}
      >
        {children}
      </Page>
      <InspectorDrawer isOpen={inspectorOpen} onClose={() => setInspectorOpen(false)} />
    </>
  );
};
