'use client';

import React from 'react';
import { Card, Table, Tag, Stack, Text, Title } from '@pui/components';
import { PageShell } from '../layouts/PageShell';
import { COMPONENT_CATALOG } from '../content/components';
import { ButtonExample, TreeExample, TableExample, CalloutExample, TagExample, FormExample, OverlayExample } from '../examples';

export const ComponentsPage: React.FC<{ category?: string }> = ({ category }) => {
  const filtered = category
    ? COMPONENT_CATALOG.filter((c) => c.category.toLowerCase().includes(category.toLowerCase()))
    : COMPONENT_CATALOG;

  const headers = ['Component', 'Category', 'Description', 'Props Specification'];
  const rows = filtered.map((c) => [
    <strong>{c.name}</strong>,
    <Tag intent="primary" minimal size="sm">{c.category}</Tag>,
    c.description,
    <code>{c.propsSummary}</code>,
  ]);

  return (
    <PageShell
      title="Component Catalog"
      description="Pure, state-agnostic React primitives and composite layout structures."
      badge={<Tag intent="primary" round>{filtered.length} Components</Tag>}
    >
      <Card elevation={1}>
        <Stack direction="column" gap={4}>
          <Title level={5}>Interactive Demonstrations</Title>
          <Stack direction="column" gap={4}>
            <ButtonExample />
            <TagExample />
            <CalloutExample />
            <TreeExample />
            <TableExample />
            <FormExample />
            <OverlayExample />
          </Stack>
        </Stack>
      </Card>

      <Card elevation={1}>
        <Stack direction="column" gap={4}>
          <Title level={5}>Component API Specifications</Title>
          <Table headers={headers} rows={rows} />
        </Stack>
      </Card>
    </PageShell>
  );
};
