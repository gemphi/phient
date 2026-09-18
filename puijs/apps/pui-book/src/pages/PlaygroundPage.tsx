'use client';

import React, { useState } from 'react';
import { Card, FormGroup, Input, Select, Switch, Tag, Button, Stack, Grid, Title, Text, Callout, Icon } from '@pui/components';
import { PageShell } from '../layouts/PageShell';
import { Play, Sparkles, RefreshCw } from 'lucide-react';

export const PlaygroundPage: React.FC = () => {
  const [btnText, setBtnText] = useState('Execute Action');
  const [btnVariant, setBtnVariant] = useState('primary');
  const [btnSize, setBtnSize] = useState('md');
  const [loading, setLoading] = useState(false);
  const [tagIntent, setTagIntent] = useState('success');

  return (
    <PageShell
      title="Live Component Playground"
      description="Test component variants, props, and states dynamically in real-time."
      badge={<Tag intent="success" round>Live Sandbox</Tag>}
    >
      <Grid columns={2} gap={6}>
        <Card elevation={1}>
          <Stack direction="column" gap={4}>
            <Title level={5}>Component Props</Title>
            <FormGroup label="Button Text">
              <Input value={btnText} onChange={(e) => setBtnText(e.target.value)} />
            </FormGroup>

            <FormGroup label="Variant">
              <Select value={btnVariant} onChange={(e) => setBtnVariant(e.target.value)}>
                <option value="primary">Primary</option>
                <option value="secondary">Secondary</option>
                <option value="outline">Outline</option>
                <option value="danger">Danger</option>
                <option value="ghost">Ghost</option>
              </Select>
            </FormGroup>

            <FormGroup label="Size">
              <Select value={btnSize} onChange={(e) => setBtnSize(e.target.value)}>
                <option value="sm">Small (sm)</option>
                <option value="md">Default (md)</option>
                <option value="lg">Large (lg)</option>
              </Select>
            </FormGroup>

            <FormGroup label="Loading State">
              <Switch label="Show Loading Spinner" checked={loading} onChange={(e) => setLoading(e.target.checked)} />
            </FormGroup>

            <FormGroup label="Tag Intent">
              <Select value={tagIntent} onChange={(e) => setTagIntent(e.target.value)}>
                <option value="primary">Primary</option>
                <option value="success">Success</option>
                <option value="warning">Warning</option>
                <option value="error">Error</option>
                <option value="info">Info</option>
              </Select>
            </FormGroup>
          </Stack>
        </Card>

        <Card elevation={1}>
          <Stack direction="column" align="center" justify="center" minHeight={300} gap={4}>
            <Button variant={btnVariant as any} size={btnSize as any} loading={loading} icon={<Icon icon={Play} size="xs" />}>
              {btnText}
            </Button>
            <Tag intent={tagIntent as any} round>
              Live Tag State: {tagIntent}
            </Tag>
          </Stack>
        </Card>
      </Grid>
    </PageShell>
  );
};
