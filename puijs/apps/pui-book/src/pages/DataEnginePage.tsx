'use client';

import React from 'react';
import { Stack, Title, Text, Divider, Callout, Section } from '@pui/components';
import { PageShell } from '../layouts/PageShell';
import { StoreExample } from '../examples/StoreExample';
import { ServiceExample } from '../examples/ServiceExample';
import { StreamExample } from '../examples/StreamExample';

export const DataEnginePage: React.FC = () => {
  return (
    <PageShell
      title="Store, Services & API Engine"
      description="Architectural foundation of PUI: Unidirectional reactive state, Dependency Injection service containers, SWR query caching, and real-time streaming."
    >
      <Section direction="column" gap={6}>
        <Stack direction="column" gap={2}>
          <Title level={4}>1. Reactive Observable Micro-Store</Title>
          <Text variant="muted">
            Ultra-fast state slices with fine-grained selector subscriptions via <code>useSyncExternalStore</code> and snapshot history.
          </Text>
        </Stack>
        <StoreExample />

        <Divider />

        <Stack direction="column" gap={2}>
          <Title level={4}>2. Dependency Injection Service Container</Title>
          <Text variant="muted">
            Inversion-of-Control service architecture with typed event bus and lifecycle management.
          </Text>
        </Stack>
        <ServiceExample />

        <Divider />

        <Stack direction="column" gap={2}>
          <Title level={4}>3. Real-Time Token & Telemetry Stream Engine</Title>
          <Text variant="muted">
            Live SSE and NDJSON chunk streaming for AI agent telemetry and real-time operations.
          </Text>
        </Stack>
        <StreamExample />
      </Section>
    </PageShell>
  );
};
