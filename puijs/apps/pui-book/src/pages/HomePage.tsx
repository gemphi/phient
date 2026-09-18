'use client';

import React from 'react';
import { Button, Card, Callout, Tag, Stack, Grid, Title, Text, Icon } from '@pui/components';
import { PageShell } from '../layouts/PageShell';
import { ArrowRight, Layers, Shield, Zap, Code2 } from 'lucide-react';

export const HomePage: React.FC<{ onNavigate: (p: string) => void }> = ({ onNavigate }) => {
  return (
    <PageShell
      title="Phient UI (PUI) - Documentation & Workbench"
      description="Enterprise React design system engineered for dense interfaces, knowledge graphs, and AI agent swarms."
      badge={<Tag intent="primary" round>puijs.com</Tag>}
    >
      <Callout intent="primary" title="Zero Business Coupling Standard">
        PUI primitives contain zero business logic or domain shopping carts. Props flow down, callbacks flow up.
      </Callout>

      <Grid columns={3} gap={4}>
        <Card elevation={1}>
          <Stack direction="column" gap={3}>
            <Icon icon={Layers} size="lg" tone="primary" />
            <Title level={5}>Core Primitives</Title>
            <Text variant="sm" color="secondary">
              Tree, Table, Card, Dialog, Drawer, Tag, Callout, Switch, and Slider designed for dense applications.
            </Text>
            <Button variant="outline" size="sm" onClick={() => onNavigate('/components/primitives')}>
              View Catalog
            </Button>
          </Stack>
        </Card>

        <Card elevation={1}>
          <Stack direction="column" gap={3}>
            <Icon icon={Shield} size="lg" tone="success" />
            <Title level={5}>Strict Design Tokens</Title>
            <Text variant="sm" color="secondary">
              Centralized --phi-* design tokens with Palantir Foundry, Blueprint, and Emerald theme palettes.
            </Text>
            <Button variant="outline" size="sm" onClick={() => onNavigate('/tokens')}>
              Inspect Tokens
            </Button>
          </Stack>
        </Card>

        <Card elevation={1}>
          <Stack direction="column" gap={3}>
            <Icon icon={Zap} size="lg" tone="warning" />
            <Title level={5}>Interactive Playground</Title>
            <Text variant="sm" color="secondary">
              Live component testing sandbox with real-time prop mutation and accessibility verification.
            </Text>
            <Button variant="primary" size="sm" onClick={() => onNavigate('/playground')}>
              Open Playground
            </Button>
          </Stack>
        </Card>
      </Grid>
    </PageShell>
  );
};
