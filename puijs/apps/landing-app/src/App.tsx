'use client';

import React from 'react';
import {
  Button,
  Card,
  Tag,
  Stack,
  Grid,
  Title,
  Text,
  Icon,
  Navbar,
  NavbarSection,
  Section,
  usePuiTheme,
} from '@pui/components';
import { Layers, Shield, Sparkles, ArrowRight, Code2, Sun, Moon } from 'lucide-react';

export default function App() {
  const { setTheme, isDark } = usePuiTheme();

  return (
    <Section as="main" variant="page" direction="column">
      <Navbar sticky variant="solid" height={64}>
        <NavbarSection align="start">
          <Stack direction="row" align="center" gap={2}>
            <Icon icon={Sparkles} size="md" tone="primary" />
            <Title level={4} size="sm">Phient UI (PUI)</Title>
            <Tag intent="primary" round>puijs.com</Tag>
          </Stack>
        </NavbarSection>
        <NavbarSection align="end">
          <Stack direction="row" align="center" gap={2}>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setTheme(isDark ? 'light' : 'dark')}
              icon={<Icon icon={isDark ? Sun : Moon} size="xs" />}
            >
              {isDark ? 'Light' : 'Dark'}
            </Button>
            <Button variant="primary" size="sm" icon={<Icon icon={Code2} size="xs" />}>GitHub</Button>
          </Stack>
        </NavbarSection>
      </Navbar>

      <Section variant="hero" direction="column" align="center" gap={4}>
        <Tag intent="primary" minimal>REACT 18 & 19 ENTERPRISE DESIGN SYSTEM</Tag>
        <Title level={1} size="3xl" gradient>
          Dense UI Primitives.
          <br />
          Zero Domain Coupling.
        </Title>
        <Text variant="lg" color="secondary" align="center">
          Phient UI delivers Palantir Blueprint-grade trees, tables, dialogs, drawers, and multi-brand themes.
        </Text>
        <Stack direction="row" gap={3} justify="center">
          <Button variant="primary" size="lg" icon={<Icon icon={ArrowRight} size="sm" />}>Explore Workbench</Button>
          <Button variant="outline" size="lg">Read Documentation</Button>
        </Stack>
      </Section>

      <Section variant="section" direction="column" gap={4}>
        <Grid columns={3} gap={4}>
          <Card elevation={2}>
            <Stack direction="column" gap={2}>
              <Icon icon={Layers} size="lg" tone="primary" />
              <Title level={4}>Pure Composable Primitives</Title>
              <Text variant="sm" color="secondary">Tree, Table, Dialog, Drawer, Tag, and Callout designed for dense data.</Text>
            </Stack>
          </Card>
          <Card elevation={2}>
            <Stack direction="column" gap={2}>
              <Icon icon={Shield} size="lg" tone="success" />
              <Title level={4}>Zero Business Coupling</Title>
              <Text variant="sm" color="secondary">Strictly decoupled. Props flow down, callbacks flow up.</Text>
            </Stack>
          </Card>
          <Card elevation={2}>
            <Stack direction="column" gap={2}>
              <Icon icon={Sparkles} size="lg" tone="warning" />
              <Title level={4}>Multi-Brand Theming</Title>
              <Text variant="sm" color="secondary">Foundry, Blueprint, Emerald, and Midnight palettes with instant dark mode.</Text>
            </Stack>
          </Card>
        </Grid>
      </Section>
    </Section>
  );
}
