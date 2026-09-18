import React, { useState } from 'react';
import { Page } from '@pui/components';
import { Header } from './components/Shell/Header';
import { SidebarNav } from './components/Shell/Sidebar';
import { AgentsPage } from './pages/AgentsPage';
import { OntologyPage } from './pages/OntologyPage';
import { StreamsPage } from './pages/StreamsPage';
import { SpatialPage } from './pages/SpatialPage';
import { OrchestrationPage } from './pages/OrchestrationPage';
import { GovernancePage } from './pages/GovernancePage';
import { DocsPage } from './pages/DocsPage';

export const App: React.FC = () => {
  const [activeTab, setActiveTab] = useState<string>('agents');

  const renderActivePage = () => {
    switch (activeTab) {
      case 'agents': return <AgentsPage />;
      case 'ontology': return <OntologyPage />;
      case 'streams': return <StreamsPage />;
      case 'spatial': return <SpatialPage />;
      case 'orchestration': return <OrchestrationPage />;
      case 'governance': return <GovernancePage />;
      case 'docs': return <DocsPage />;
      default: return <AgentsPage />;
    }
  };

  return (
    <Page
      header={<Header activeTab={activeTab} onSelectTab={setActiveTab} />}
      sidebar={<SidebarNav activeTab={activeTab} onSelectTab={setActiveTab} />}
    >
      {renderActivePage()}
    </Page>
  );
};
