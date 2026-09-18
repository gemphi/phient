'use client';

import React from 'react';
import { Card, Table, Tag, Title, Text, Stack } from '@pui/components';
import { PageShell } from '../layouts/PageShell';
import { CORE_TOKENS, ELEVATION_TOKENS } from '../content/tokens';

export const TokensPage: React.FC = () => {
  const headers = ['Token Name', 'CSS Custom Property', 'Default Value', 'Description'];

  const coreRows = CORE_TOKENS.map((t) => [
    <strong>{t.name}</strong>,
    <code>{t.variable}</code>,
    <Text size="sm" mono>{t.value}</Text>,
    t.description,
  ]);

  const elevationRows = ELEVATION_TOKENS.map((t) => [
    <strong>{t.name}</strong>,
    <code>{t.variable}</code>,
    <Text size="sm" mono>{t.value}</Text>,
    t.description,
  ]);

  return (
    <PageShell
      title="Design Tokens"
      description="Centralized CSS custom properties powering light/dark themes, spacing, and elevation."
      badge={<Tag intent="primary" round>--phi-* Standard</Tag>}
    >
      <Card elevation={1}>
        <Stack direction="column" gap={4}>
          <Title level={5}>Core Color & Surface Tokens</Title>
          <Table headers={headers} rows={coreRows} />
        </Stack>
      </Card>

      <Card elevation={1}>
        <Stack direction="column" gap={4}>
          <Title level={5}>Elevation & Shadow Levels</Title>
          <Table headers={headers} rows={elevationRows} />
        </Stack>
      </Card>
    </PageShell>
  );
};
