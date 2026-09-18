'use client';

import React from 'react';
import { Card, Grid, Title, Text, Tag, Stack } from '@pui/components';

export const MetricsGrid: React.FC = () => {
  return (
    <Grid columns={4} gap={3}>
      <Card elevation={1}>
        <Stack direction="column" gap={2}>
          <Text variant="xs" color="secondary" weight="semibold">TOTAL ACTIVE AGENTS</Text>
          <Title level={3}>15 Swarms</Title>
          <Tag intent="success" minimal>+3 Synchronized</Tag>
        </Stack>
      </Card>
      <Card elevation={1}>
        <Stack direction="column" gap={2}>
          <Text variant="xs" color="secondary" weight="semibold">PHASE MANIFOLD RESONANCE</Text>
          <Title level={3}>r = 0.984</Title>
          <Tag intent="primary" minimal>Harmonic Lock</Tag>
        </Stack>
      </Card>
      <Card elevation={1}>
        <Stack direction="column" gap={2}>
          <Text variant="xs" color="secondary" weight="semibold">AXIOMATIC MORPHISMS</Text>
          <Title level={3}>14 Modules</Title>
          <Tag intent="info" minimal>100% Type-Safe</Tag>
        </Stack>
      </Card>
      <Card elevation={1}>
        <Stack direction="column" gap={2}>
          <Text variant="xs" color="secondary" weight="semibold">MEMORY OCTAVE CONTINUUM</Text>
          <Title level={3}>16 Layers</Title>
          <Tag intent="warning" minimal>Surface to Deep</Tag>
        </Stack>
      </Card>
    </Grid>
  );
};
