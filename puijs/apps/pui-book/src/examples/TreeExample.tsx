'use client';

import React, { useState } from 'react';
import { Tree, TreeNode, Card, CardBody, Stack, Text, Tag } from '@pui/components';
import { Folder, Database, FileCode, Layers } from 'lucide-react';

export const TreeExample: React.FC = () => {
  const [selected, setSelected] = useState('phigen');

  const nodes: TreeNode[] = [
    {
      id: 'phiadk',
      label: 'phiadk (Core Python & Rust SDK)',
      icon: <Folder size={15} className="text-amber-500" />,
      isExpanded: true,
      childNodes: [
        { id: 'agents', label: 'agents/', childNodes: [
          { id: 'phigen', label: 'phigen.ts (LLM Synthesis)', icon: <FileCode size={14} /> },
          { id: 'phirag', label: 'phirag.ts (Vector Manifold)', icon: <FileCode size={14} /> },
        ]},
        { id: 'ontologies', label: 'ontologies/', childNodes: [
          { id: 'object', label: 'object.py (Entity Schema)', icon: <Database size={14} /> },
          { id: 'action', label: 'action.py (Action Type)', icon: <Layers size={14} /> },
        ]},
      ],
    },
  ];

  return (
    <Card elevation={1}>
      <CardBody>
        <Stack direction="column" gap={3}>
          <Stack direction="row" justify="between" align="center">
            <Text weight="semibold">Hierarchical Tree View (Palantir Blueprint)</Text>
            <Tag intent="primary" round>Selected: {selected}</Tag>
          </Stack>
          <Tree nodes={nodes} onNodeClick={(node) => setSelected(String(node.id))} />
        </Stack>
      </CardBody>
    </Card>
  );
};
