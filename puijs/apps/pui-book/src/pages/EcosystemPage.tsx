'use client';

import React from 'react';
import { Card, Callout, Tag, Stack, Grid, Title, Text, Icon } from '@pui/components';
import { PageShell } from '../layouts/PageShell';
import { Database, Activity, Cpu, Layers } from 'lucide-react';

export const EcosystemPage: React.FC = () => {
  return (
    <PageShell
      title="GemPhi Ecosystem & Topos Architecture"
      description="How Phient UI seamlessly connects with Phient, Phiano, and the PhiADK agent mesh."
      badge={<Tag intent="success" round>Axiomatic Topos</Tag>}
    >
      <Callout intent="primary" title="Unified Subsystem Integration">
        PUI provides the visual and interaction layer for all GemPhi AI agents, continuous phase manifolds, and ontology schemas.
      </Callout>

      <Grid columns={3} gap={4}>
        <Card elevation={1}>
          <Stack direction="column" gap={3}>
            <Icon icon={Cpu} size="lg" tone="primary" />
            <Title level={5}>PhiADK Agent Mesh</Title>
            <Text variant="sm" color="secondary">
              phiadk.agents.phigen, phirag, and phibus emit telemetry streams rendered directly in PUI dashboards.
            </Text>
          </Stack>
        </Card>

        <Card elevation={1}>
          <Stack direction="column" gap={3}>
            <Icon icon={Activity} size="lg" tone="success" />
            <Title level={5}>Phiano Phase Manifold</Title>
            <Text variant="sm" color="secondary">
              Continuous phase representations, Kuramoto coupling, and 16-layer memory octave continuum visualizers.
            </Text>
          </Stack>
        </Card>

        <Card elevation={1}>
          <Stack direction="column" gap={3}>
            <Icon icon={Database} size="lg" tone="warning" />
            <Title level={5}>Axiomatic Ontologies</Title>
            <Text variant="sm" color="secondary">
              Topos schemas: GeoProperty, MediaProperty, CipherProperty, and ActionTypeMetadata.
            </Text>
          </Stack>
        </Card>
      </Grid>
    </PageShell>
  );
};
