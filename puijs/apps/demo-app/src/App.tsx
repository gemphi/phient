'use client';

import React, { useState } from 'react';
import { Callout, Drawer, Title, Text, Divider, Page, Stack } from '@pui/components';
import { CockpitHeader } from './CockpitHeader';
import { CockpitSidebar } from './CockpitSidebar';
import { MetricsGrid } from './MetricsGrid';
import { SwarmTable } from './SwarmTable';

export default function App() {
  const [inspectorOpen, setInspectorOpen] = useState(false);
  const [selectedNode, setSelectedNode] = useState('ontology/geo');

  return (
    <>
      <Page
        header={<CockpitHeader />}
        sidebar={<CockpitSidebar onSelect={setSelectedNode} />}
      >
        <Stack direction="column" gap={6}>
          <Callout intent="primary" title="Palantir Foundry / Blueprint Decoupled Standard">
            All UI components shown here render pure, state-agnostic primitives with zero business logic coupling.
          </Callout>
          <MetricsGrid />
          <SwarmTable onInspect={() => setInspectorOpen(true)} />
        </Stack>
      </Page>

      <Drawer isOpen={inspectorOpen} onClose={() => setInspectorOpen(false)} title="Telemetry Inspector" position="right">
        <Stack direction="column" gap={4}>
          <Title level={5}>Node Details: {selectedNode}</Title>
          <Text variant="sm" color="secondary">
            Axiomatic phase properties and state mutations.
          </Text>
          <Divider />
          <Callout intent="success" title="Health: Optimal">
            Low latency, zero phase drift detected across the Kuramoto coupling cycle.
          </Callout>
        </Stack>
      </Drawer>
    </>
  );
}
