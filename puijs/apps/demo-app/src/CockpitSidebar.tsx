'use client';

import React from 'react';
import {
  Tree,
  TreeNode,
  Title,
  Sidebar,
  SidebarHeader,
  SidebarBody,
  Stack,
} from '@pui/components';
import { Database, Layers } from 'lucide-react';

export const treeNodes: TreeNode[] = [
  {
    id: 'ontology',
    label: 'Ontology Topos',
    icon: <Database size={15} />,
    isExpanded: true,
    childNodes: [
      { id: 'geo', label: 'GeoProperty (Space-Time Series)' },
      { id: 'media', label: 'MediaProperty (Multi-Modal Stream)' },
      { id: 'cipher', label: 'CipherProperty (Homomorphic Crypto)' },
      { id: 'action', label: 'ActionTypeMetadata (Axiom Morphisms)' },
    ],
  },
  {
    id: 'agents',
    label: 'Agent Swarms (GemPhi)',
    icon: <Layers size={15} />,
    isExpanded: true,
    childNodes: [
      { id: 'phigen', label: 'PhiGen Agent (LLM Code Synthesis)' },
      { id: 'phirag', label: 'PhiRAG Agent (Continuous Manifold)' },
      { id: 'philog', label: 'PhiLog Agent (Observability Telemetry)' },
    ],
  },
];

export const CockpitSidebar: React.FC<{ onSelect: (id: string) => void }> = ({ onSelect }) => {
  return (
    <Sidebar defaultCollapsed={false}>
      <SidebarHeader>
        <Stack direction="column" gap={1}>
          <Title level={5} size="sm">System Hierarchy</Title>
        </Stack>
      </SidebarHeader>
      <SidebarBody>
        <Tree nodes={treeNodes} onNodeClick={(node) => onSelect(String(node.id))} />
      </SidebarBody>
    </Sidebar>
  );
};
